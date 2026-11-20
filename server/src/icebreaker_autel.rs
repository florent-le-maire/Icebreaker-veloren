//! Icebreaker : l'autel de résurrection (jalon T, T.12, `assets/icebreaker/mort.ron` → `autel`,
//! `docs/icebreaker/design/donjon.md`, « Portail et autel »).
//! Un vivant qui porte des cristaux de morts prie à l'autel (une interaction de quelques secondes) :
//! tous ses cristaux sont consommés, chaque mort encore en bas revient près de l'autel avec une part
//! de ses PV, et le prieur perd une part de sa vie du moment, une seule fois.

use crate::{Server, icebreaker_slots::IcebreakerSlots};
use common::{
    comp::{self, ChatType, Content},
    icebreaker_mort::{Autel, IcebreakerMort},
    icebreaker_portail::AUTEL,
    terrain::TerrainGrid,
    uid::{IdMaps, Uid},
};
use common_net::msg::ServerGeneral;
use specs::{Entity as EcsEntity, WorldExt};
use vek::Vec3;

/// La vie qui reste au prieur après la prière ; rien s'il n'a pas assez de vie pour prier.
pub fn cout(pv: f32, autel: &Autel) -> Option<f32> {
    (pv >= autel.pv_minimum).then(|| pv * (1.0 - autel.part_vie_perdue.clamp(0.0, 1.0)))
}

/// Les morts à relever : ceux dont un cristal est porté et qui sont encore là, une fois chacun.
pub fn a_relever(cristaux: &[Uid], morts: &[Uid]) -> Vec<Uid> {
    let mut releves = Vec::new();
    for uid in cristaux {
        if morts.contains(uid) && !releves.contains(uid) {
            releves.push(*uid);
        }
    }
    releves
}

/// Les places autour de l'autel où poser les ressuscités : huit points sur un cercle, ceux où l'on
/// tient debout (`libre`) ; aucune de libre : la place de repli.
pub fn places(centre: Vec3<f32>, libre: impl Fn(Vec3<f32>) -> bool, repli: Vec3<f32>) -> Vec<Vec3<f32>> {
    const RAYON: f32 = 3.0;
    let libres = (0..8)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::FRAC_PI_4;
            centre + Vec3::new(angle.cos(), angle.sin(), 0.0) * RAYON
        })
        .filter(|place| libre(*place))
        .collect::<Vec<_>>();
    if libres.is_empty() { vec![repli] } else { libres }
}

fn refus(server: &Server, prieur: EcsEntity, texte: &str) {
    server.notify_client(
        prieur,
        ServerGeneral::server_msg(ChatType::CommandError, Content::Plain(texte.to_string())),
    );
}

/// `prieur` a prié jusqu'au bout à l'autel `autel`.
pub fn prier(server: &mut Server, prieur: EcsEntity, autel: Uid) {
    let reglages = IcebreakerMort::actuel().autel;
    let ecs = server.state.ecs();
    let Some(autel) = ecs.read_resource::<IdMaps>().uid_entity(autel) else {
        return;
    };
    let positions = ecs.read_storage::<comp::Pos>();
    let (Some(centre), Some(ici)) = (positions.get(autel).map(|p| p.0), positions.get(prieur).map(|p| p.0)) else {
        return;
    };
    if ecs.read_storage::<comp::Body>().get(autel) != Some(&AUTEL) {
        return;
    }
    let pv = ecs.read_storage::<comp::Health>().get(prieur).filter(|h| !h.is_dead).map(|h| h.current());
    let Some(reste) = pv.and_then(|pv| cout(pv, &reglages)) else {
        drop(positions);
        return refus(server, prieur, &format!(
            "Il faut au moins {} PV pour prier à l'autel.",
            reglages.pv_minimum
        ));
    };

    // Les cristaux portés, et les joueurs morts encore dans le même emplacement que l'autel.
    let cristaux = ecs
        .read_storage::<comp::Inventory>()
        .get(prieur)
        .map(|inventory| {
            inventory
                .slots()
                .flatten()
                .filter_map(|objet| objet.icebreaker_proprietaire().map(|(uid, _)| uid))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let morts = {
        use specs::Join;
        let slots = ecs.read_resource::<IcebreakerSlots>();
        let slot = slots.at(centre.xy());
        (
            &ecs.read_storage::<Uid>(),
            &ecs.read_storage::<comp::Player>(),
            &ecs.read_storage::<comp::Health>(),
            &positions,
        )
            .join()
            .filter(|(_, _, health, pos)| health.is_dead && slot.is_none_or(|slot| slot.holds(pos.0.xy())))
            .map(|(uid, _, _, _)| *uid)
            .collect::<Vec<_>>()
    };
    let releves = a_relever(&cristaux, &morts);
    if releves.is_empty() {
        drop(positions);
        return refus(server, prieur, if cristaux.is_empty() {
            "Il faut le cristal d'un compagnon mort pour prier à l'autel."
        } else {
            "Les morts dont tu portes le cristal ne sont plus ici."
        });
    }
    drop(positions);

    // Tous les cristaux sont consommés, même ceux d'un mort qui n'est plus là.
    if let Some(mut inventory) = ecs.write_storage::<comp::Inventory>().get_mut(prieur) {
        crate::icebreaker_mort::prendre_cristaux(&mut inventory);
    }
    let places = {
        let terrain = ecs.read_resource::<TerrainGrid>();
        places(centre, |place| terrain.is_space(place.map(|e| e.floor() as i32)), ici)
    };
    let mut noms = Vec::new();
    for (i, uid) in releves.iter().enumerate() {
        let Some(mort) = ecs.read_resource::<IdMaps>().uid_entity(*uid) else {
            continue;
        };
        if let Some(mut health) = ecs.write_storage::<comp::Health>().get_mut(mort) {
            health.revive();
            health.set_fraction(reglages.pv_resurrection);
        }
        if let Some(pos) = ecs.write_storage::<comp::Pos>().get_mut(mort) {
            pos.0 = places[i % places.len()];
        }
        if let Some(vel) = ecs.write_storage::<comp::Vel>().get_mut(mort) {
            vel.0 = Vec3::zero();
        }
        if let Some(mut etat) = ecs.write_storage::<comp::CharacterState>().get_mut(mort) {
            *etat = comp::CharacterState::default();
        }
        if let Some(mut combo) = ecs.write_storage::<comp::Combo>().get_mut(mort) {
            combo.reset();
        }
        if let Some(physique) = ecs.write_storage::<comp::PhysicsState>().get_mut(mort) {
            physique.reset();
        }
        if let Some(force) = ecs.write_storage::<comp::ForceUpdate>().get_mut(mort) {
            force.update();
        }
        if let Some(player) = ecs.read_storage::<comp::Player>().get(mort) {
            noms.push(player.alias.clone());
        }
    }
    // Le prix : une part de la vie du moment, une seule fois.
    if let Some(mut health) = ecs.write_storage::<comp::Health>().get_mut(prieur) {
        let maximum = health.maximum().max(1.0);
        health.set_fraction(reste / maximum);
    }
    let prieur_nom = ecs
        .read_storage::<comp::Player>()
        .get(prieur)
        .map_or_else(|| "Quelqu'un".to_string(), |player| player.alias.clone());
    let annonce = format!("{prieur_nom} a relevé {} à l'autel.", noms.join(", "));
    server.notify_players(ServerGeneral::server_msg(ChatType::Meta, Content::Plain(annonce)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u64) -> Uid { Uid(std::num::NonZeroU64::new(n).unwrap()) }

    #[test]
    fn la_moitie_de_la_vie_du_moment() {
        let autel = Autel::default();
        assert_eq!(cout(80.0, &autel), Some(40.0));
        assert_eq!(cout(2.0, &autel), Some(1.0));
        // À 1 PV, on ne peut pas prier.
        assert_eq!(cout(1.0, &autel), None);
    }

    #[test]
    fn un_cristal_releve_son_mort_s_il_est_encore_la() {
        // Deux cristaux du même mort, un d'un mort parti, un mort sans cristal.
        let cristaux = [uid(1), uid(1), uid(2), uid(3)];
        let morts = [uid(1), uid(3), uid(4)];
        assert_eq!(a_relever(&cristaux, &morts), [uid(1), uid(3)]);
        assert!(a_relever(&[], &morts).is_empty());
    }

    #[test]
    fn les_ressuscites_arrivent_autour_de_l_autel() {
        let centre = Vec3::new(10.0, 10.0, 5.0);
        let repli = Vec3::new(0.0, 0.0, 5.0);
        let toutes = places(centre, |_| true, repli);
        assert_eq!(toutes.len(), 8);
        assert!(toutes.iter().all(|p| (p.distance(centre) - 3.0).abs() < 1e-3));
        // Un mur à l'est : ces places sautent ; rien de libre : à côté du prieur.
        assert!(places(centre, |p| p.x < 11.0, repli).iter().all(|p| p.x < 11.0));
        assert_eq!(places(centre, |_| false, repli), [repli]);
    }
}
