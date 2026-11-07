//! Icebreaker : la balise lumineuse (jalon J, `assets/icebreaker/balise.ron`).
//! `/balise poser` (la touche « Poser une balise ») pose aux pieds du joueur un objet au sol qui
//! éclaire dans la couleur choisie (`/balise couleur <nom>`). Une seule charge : elle revient
//! `delai_recharge` secondes après la pose (`sys::icebreaker_balise` prévient), ou tout de suite
//! quand on ramasse une balise au sol (E, `events::inventory_manip`).

use crate::{Server, icebreaker_slots::IcebreakerSlots, state_ext::StateExt};
use common::{
    cmd::ServerChatCommand,
    comp::{self, ChatType, Content, Inventory, Item, LightEmitter},
    icebreaker_balise::{BALISE, IcebreakerBalise, est_balise},
    resources::{ProgramTime, Time},
};
use common_net::{msg::ServerGeneral, sync::WorldSyncExt};
use hashbrown::HashMap;
use specs::{Builder, Entity as EcsEntity, World, WorldExt};
use vek::{Rgb, Vec3};

/// Icebreaker : l'état des balises de la partie en cours (rien n'est sauvegardé).
#[derive(Default)]
pub struct IcebreakerBalises {
    /// Quand chaque joueur a posé sa dernière balise (temps du serveur) ; absent : charge prête.
    pub posee_a: HashMap<EcsEntity, f64>,
    /// Couleur choisie par chaque joueur (nom dans `balise.ron`).
    pub couleur: HashMap<EcsEntity, String>,
    /// Balises au sol, de la plus ancienne à la plus récente, avec leur poseur.
    pub au_sol: Vec<(EcsEntity, EcsEntity)>,
}

impl IcebreakerBalises {
    /// La charge de `joueur` revient tout de suite ; vrai si elle était en recharge.
    pub fn rendre_charge(&mut self, joueur: EcsEntity) -> bool {
        self.posee_a.remove(&joueur).is_some()
    }

    /// `balise` n'est plus au sol (ramassée ou effacée).
    pub fn retirer(&mut self, balise: EcsEntity) { self.au_sol.retain(|(b, _)| *b != balise); }
}

/// Les balises à éteindre pour qu'en en posant une de plus, `poseur` en ait au plus `max` au sol et
/// le serveur au plus `max_total` : les plus anciennes de `poseur`, puis les plus anciennes de tous.
pub fn a_eteindre<T: Copy + PartialEq>(
    au_sol: &[(T, T)],
    poseur: T,
    max: usize,
    max_total: usize,
) -> Vec<T> {
    let siennes = au_sol
        .iter()
        .filter(|(_, p)| *p == poseur)
        .map(|(b, _)| *b)
        .collect::<Vec<_>>();
    let trop = siennes.len().saturating_sub(max.saturating_sub(1));
    let mut eteindre = siennes.into_iter().take(trop).collect::<Vec<_>>();
    // Icebreaker : plafond pour tout le serveur, tous joueurs confondus.
    let trop_total = (au_sol.len() - eteindre.len()).saturating_sub(max_total.saturating_sub(1));
    let autres = au_sol
        .iter()
        .map(|(b, _)| *b)
        .filter(|b| !eteindre.contains(b))
        .take(trop_total)
        .collect::<Vec<_>>();
    eteindre.extend(autres);
    eteindre
}

/// `joueur` est dans un donjon : un emplacement d'expédition dans la campagne (décision 0016), ou
/// un lieu lancé seul qui n'est pas le village (même test que `/rentrer`).
fn en_donjon(ecs: &World, joueur: EcsEntity) -> bool {
    if world::icebreaker::is_campaign() {
        ecs.read_storage::<comp::Pos>()
            .get(joueur)
            .is_some_and(|pos| ecs.read_resource::<IcebreakerSlots>().at(pos.0.xy()).is_some())
    } else {
        world::icebreaker::is_instance() && !world::icebreaker::is_village()
    }
}

/// Donne une balise à `inventory` s'il n'en a pas (à l'arrivée dans un donjon).
pub fn donner_si_absente(inventory: &mut Inventory) {
    if !inventory.slots().flatten().any(est_balise) {
        let _ = inventory.push(Item::new_from_asset_expect(BALISE));
    }
}

fn dire(server: &Server, joueur: EcsEntity, texte: String) {
    server.notify_client(
        joueur,
        ServerGeneral::server_msg(ChatType::CommandInfo, Content::Plain(texte)),
    );
}

pub(crate) fn handle_balise(
    server: &mut Server,
    client: EcsEntity,
    _target: EcsEntity,
    args: Vec<String>,
    _action: &ServerChatCommand,
) -> Result<(), Content> {
    let regles = IcebreakerBalise::actuel();
    match args.first().map(String::as_str) {
        Some("poser") => poser(server, client, &regles),
        Some("couleur") => {
            let Some(nom) = args.get(1) else {
                return Err(Content::Plain(format!("Couleurs : {}.", noms(&regles))));
            };
            if regles.couleur(nom).is_none() {
                return Err(Content::Plain(format!(
                    "Pas de couleur « {nom} » ; couleurs : {}.",
                    noms(&regles)
                )));
            }
            let nom = nom.to_lowercase();
            server
                .state
                .ecs()
                .write_resource::<IcebreakerBalises>()
                .couleur
                .insert(client, nom.clone());
            dire(server, client, format!("Balise : couleur {nom}."));
            Ok(())
        },
        None => {
            ouvrir_menu(server, client, &regles);
            Ok(())
        },
        _ => Err(Content::Plain(format!(
            "/balise, /balise poser, ou /balise couleur <{}>.",
            noms(&regles).replace(", ", "|")
        ))),
    }
}

fn noms(regles: &IcebreakerBalise) -> String {
    regles
        .couleurs
        .iter()
        .map(|(nom, _)| nom.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Le choix de la couleur (J.3) : le panneau des bâtiments du village (jalon X), avec un bouton
/// par couleur qui envoie `/balise couleur <nom>` ; le client le montre hors de la vue de dessus.
fn ouvrir_menu(server: &Server, client: EcsEntity, regles: &IcebreakerBalise) {
    let actuelle = server
        .state
        .ecs()
        .read_resource::<IcebreakerBalises>()
        .couleur
        .get(&client)
        .cloned()
        .unwrap_or_else(|| regles.couleur_par_defaut().to_string());
    let lignes = vec![(
        Content::Plain(format!(
            "Couleur actuelle : {actuelle}. La touche « Poser une balise » en pose une à ses pieds ; \
             E sur une balise posée la retire et rend la charge."
        )),
        Vec::new(),
    )];
    let boutons = regles
        .couleurs
        .iter()
        .map(|(nom, _)| {
            let mut libelle = nom.clone();
            if let Some(premiere) = libelle.get_mut(0..1) {
                premiere.make_ascii_uppercase();
            }
            (Content::Plain(libelle), vec![
                "balise".to_string(),
                "couleur".to_string(),
                nom.clone(),
            ])
        })
        .collect();
    server.notify_client(
        client,
        ServerGeneral::IcebreakerPanel(Content::Plain("Balise lumineuse".to_string()), lignes, boutons),
    );
}

fn poser(server: &mut Server, client: EcsEntity, regles: &IcebreakerBalise) -> Result<(), Content> {
    let ecs = server.state.ecs();
    if !en_donjon(ecs, client) {
        return Err(Content::Plain("La balise se pose dans les donjons.".to_string()));
    }
    if !ecs
        .read_storage::<Inventory>()
        .get(client)
        .is_some_and(|inventory| inventory.slots().flatten().any(est_balise))
    {
        return Err(Content::Plain("Il faut une balise lumineuse dans son sac.".to_string()));
    }
    let Some(pos) = ecs.read_storage::<comp::Pos>().get(client).copied() else {
        return Err(Content::Plain("Il faut un personnage en jeu.".to_string()));
    };
    let maintenant = ecs.read_resource::<Time>().0;
    let (col, eteindre) = {
        let balises = ecs.read_resource::<IcebreakerBalises>();
        if let Some(posee) = balises.posee_a.get(&client) {
            let reste = regles.reste_recharge(*posee, maintenant);
            if reste > 0.0 {
                return Err(Content::Plain(format!(
                    "Balise en recharge ({} s).",
                    reste.ceil()
                )));
            }
        }
        let nom = balises
            .couleur
            .get(&client)
            .map_or(regles.couleur_par_defaut(), String::as_str);
        let col = regles.couleur(nom).unwrap_or(Rgb::one());
        (col, a_eteindre(&balises.au_sol, client, regles.max_au_sol, regles.max_total))
    };
    for vieille in eteindre {
        server.state.ecs().write_resource::<IcebreakerBalises>().retirer(vieille);
        if let Err(err) = server.state.delete_entity_recorded(vieille) {
            tracing::warn!(?err, "Icebreaker : balise à éteindre déjà partie");
        }
    }

    // Un objet au sol (E le désigne comme les autres), qui ne se fusionne pas et ne s'efface pas
    // tout seul (pas de `create_item_drop`), avec une lumière.
    let item = Item::new_from_asset_expect(BALISE);
    let item_body = comp::body::item::Body::from(&item);
    let body = comp::Body::Item(item_body);
    let program_time = ProgramTime(server.state.get_program_time());
    let balise = server
        .state
        .ecs_mut()
        .create_entity_synced()
        .with(comp::PickupItem::new(item, program_time, false))
        .with(pos)
        .with(comp::Vel(Vec3::zero()))
        .with(item_body.orientation(&mut rand::rng()))
        .with(item_body.mass())
        .with(item_body.density())
        .with(body.collider())
        .with(body)
        .with(LightEmitter {
            col,
            strength: regles.force,
            flicker: regles.vacillement,
            animated: true,
            dir: None,
        })
        .build();
    let mut balises = server.state.ecs().write_resource::<IcebreakerBalises>();
    balises.au_sol.push((balise, client));
    balises.posee_a.insert(client, maintenant);
    drop(balises);
    server.notify_client(
        client,
        ServerGeneral::IcebreakerBaliseRecharge(regles.delai_recharge as f32),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eteindre_les_plus_anciennes() {
        // (balise, poseur) : le joueur 1 a posé 10, 11, 12 ; le joueur 2 a posé 20.
        let au_sol = [(10, 1), (20, 2), (11, 1), (12, 1)];
        assert_eq!(a_eteindre(&au_sol, 1, 6, 200), Vec::<i32>::new());
        assert_eq!(a_eteindre(&au_sol, 1, 3, 200), vec![10]);
        assert_eq!(a_eteindre(&au_sol, 1, 2, 200), vec![10, 11]);
        assert_eq!(a_eteindre(&au_sol, 1, 1, 200), vec![10, 11, 12]);
        assert_eq!(a_eteindre(&au_sol, 2, 1, 200), vec![20]);
        assert_eq!(a_eteindre(&au_sol, 3, 1, 200), Vec::<i32>::new());
        // Plafond total : 4 au sol, 4 au plus → la plus ancienne de tous (celle du joueur 1),
        // même pour un autre poseur.
        assert_eq!(a_eteindre(&au_sol, 3, 200, 4), vec![10]);
        assert_eq!(a_eteindre(&au_sol, 3, 200, 3), vec![10, 20]);
        // Les deux plafonds : la sienne d'abord, puis la plus ancienne qui reste.
        assert_eq!(a_eteindre(&au_sol, 1, 3, 3), vec![10, 20]);
        assert_eq!(a_eteindre(&au_sol, 1, 3, 4), vec![10]);
    }
}
