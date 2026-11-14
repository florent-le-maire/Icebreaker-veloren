//! Icebreaker : à terre et mort (jalon T, `assets/icebreaker/mort.ron`).
//! - À terre, on meurt passé `delai_a_terre` secondes si personne ne relève (T.1) : Veloren garde
//!   à terre sans limite ; ici on tue après le délai, et la mort suit son chemin habituel (la
//!   santé à zéro sans protection donne `DestroyEvent`). Le décompte s'arrête tant qu'un allié
//!   est en train de relever (T.9).
//! - Quand tous les joueurs d'un emplacement d'expédition sont morts, tous rentrent au village en
//!   fantôme (T.5), comme « Rendre au registre ».

use crate::{client::Client, icebreaker_slots::IcebreakerSlots};
use common::{
    comp::{CharacterState, ChatType, Content, Health, Player, Pos, is_downed},
    icebreaker_mort::{IcebreakerMort, en_releve},
    resources::{DeltaTime, Time},
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use hashbrown::HashMap;
use specs::{Entities, Entity, Join, Read, ReadExpect, ReadStorage, Write, WriteStorage};

/// Icebreaker : depuis quand chaque entité est à terre (temps du serveur), et quand chaque mort
/// d'une expédition perdue a été renvoyé au village.
#[derive(Default)]
pub struct IcebreakerATerre {
    pub a_terre: HashMap<Entity, f64>,
    pub renvoyes: HashMap<Entity, f64>,
}

/// Secondes avant de renvoyer encore un mort qui serait resté dans l'emplacement.
const RENVOI_ATTENTE: f64 = 10.0;

/// L'expédition d'un emplacement a péri : des joueurs y sont, et tous sont morts.
pub fn expedition_perie(morts: impl IntoIterator<Item = bool>) -> bool {
    let mut aucun = true;
    for mort in morts {
        if !mort {
            return false;
        }
        aucun = false;
    }
    !aucun
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Write<'a, IcebreakerATerre>,
        WriteStorage<'a, Health>,
        ReadStorage<'a, CharacterState>,
        Read<'a, IcebreakerSlots>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Client>,
        // Icebreaker : le décompte de la mission, pour le bilan de ceux qu'on renvoie (N.8).
        Read<'a, crate::icebreaker_mission::IcebreakerMission>,
        ReadExpect<'a, crate::rtsim::RtSim>,
        // Icebreaker T.9 : le décompte s'arrête pendant la relève.
        Read<'a, DeltaTime>,
        ReadStorage<'a, Uid>,
    );

    const NAME: &'static str = "icebreaker_mort";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            time,
            mut etat,
            mut healths,
            character_states,
            slots,
            players,
            positions,
            clients,
            missions,
            rtsim,
            dt,
            uids,
        ): Self::SystemData,
    ) {
        let regles = IcebreakerMort::actuel();
        let maintenant = time.0;

        // T.1 : à terre trop longtemps, on meurt.
        let mut encore = HashMap::new();
        let mut a_tuer = Vec::new();
        for (entity, health, character_state) in (&entities, &healths, &character_states).join() {
            if !is_downed(Some(health), Some(character_state)) {
                continue;
            }
            let mut depuis = etat.a_terre.get(&entity).copied().unwrap_or(maintenant);
            // T.9 : un allié relève : le temps passé ne compte pas.
            if uids
                .get(entity)
                .is_some_and(|uid| en_releve(*uid, character_states.join()))
            {
                depuis += f64::from(dt.0);
            }
            if regles.doit_mourir(depuis, maintenant) {
                a_tuer.push(entity);
            } else {
                encore.insert(entity, depuis);
            }
        }
        etat.a_terre = encore;
        for entity in a_tuer {
            if let Some(mut health) = healths.get_mut(entity) {
                health.kill();
            }
        }

        // T.5 : le dernier tombe, tout le monde rentre.
        if !crate::icebreaker_registre::active() || slots.0.is_empty() {
            return;
        }
        etat.renvoyes
            .retain(|_, quand| maintenant - *quand < RENVOI_ATTENTE);
        let joueurs = (&entities, &players, &positions, &healths, &clients)
            .join()
            .map(|(entity, _, pos, health, client)| (entity, pos.0.xy(), health.is_dead, client))
            .collect::<Vec<_>>();
        for slot in &slots.0 {
            let dedans = joueurs
                .iter()
                .filter(|(_, pos, _, _)| slot.holds(*pos))
                .collect::<Vec<_>>();
            if !expedition_perie(dedans.iter().map(|(_, _, mort, _)| *mort)) {
                continue;
            }
            for (entity, _, _, client) in dedans {
                if etat.renvoyes.contains_key(entity) {
                    continue;
                }
                etat.renvoyes.insert(*entity, maintenant);
                client.send_fallible(ServerGeneral::server_msg(
                    ChatType::Meta,
                    Content::Plain("L'expédition a péri : retour au village.".to_string()),
                ));
                // Icebreaker : le bilan de l'expédition perdue (N.8) ; le sac reste sur place.
                client.send_fallible(ServerGeneral::IcebreakerBilan(
                    crate::icebreaker_bilan::panneau(&crate::icebreaker_bilan::Retour {
                        lieu: crate::icebreaker_expedition::nom_du_site(&slot.name),
                        decompte: crate::icebreaker_bilan::decompte_de(&missions, &rtsim, slot),
                        peri: true,
                        ..Default::default()
                    }),
                ));
                client.send_fallible(ServerGeneral::IcebreakerJouer(None));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perie_quand_tous_sont_morts() {
        assert!(expedition_perie([true, true]));
        assert!(!expedition_perie([true, false]));
        assert!(!expedition_perie([]));
    }
}
