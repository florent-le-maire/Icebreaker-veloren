//! Icebreaker: the village buildings (V.6). Each is one of the town's houses
//! nearest to the square, its NPC standing at the door. In the top view of the
//! village, the client asks for the buildings (to show their names and
//! highlight the one under the cursor), and a click on one for its panel. The
//! plots to build on (jalon C) are shown and clicked likewise.

use crate::{
    events::ServerEvent,
    icebreaker_panneaux::IcebreakerPanneaux,
    icebreaker_registre::{self, IcebreakerRegistre},
    icebreaker_village::{IcebreakerVillage, VillageConfig},
    sys::icebreaker_zone::Zone,
};
use common::{
    comp::{Content, Player, Presence, PresenceKind},
    event::IcebreakerBuildingEvent,
    icebreaker_panneau::{Batiment, Demande, Panneau, Sorte, texte},
    uid::Uid,
};
use specs::{Read, ReadStorage, Write};
use tracing::info;
use vek::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Building {
    Warehouse,
    Treasury,
    Intendance,
    Infirmary,
    /// Where the expeditions leave from (V.7).
    Departure,
    /// Where the recruits wait, and the common registry (jalon R).
    Charrette,
}

impl Building {
    /// In the order of the village's houses, nearest to the square first.
    pub const ALL: [Building; 6] = [
        Building::Warehouse,
        Building::Treasury,
        Building::Intendance,
        Building::Infirmary,
        Building::Departure,
        Building::Charrette,
    ];

    /// Its id in the data files (`village.ron`, `vue_village.ron`) and the
    /// messages to the client (X.4).
    pub fn id(self) -> &'static str {
        match self {
            Building::Warehouse => "entrepot",
            Building::Treasury => "caisse",
            Building::Intendance => "intendance",
            Building::Infirmary => "infirmerie",
            Building::Departure => "depart",
            Building::Charrette => "charrette",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Building::Warehouse => "Entrepôt",
            Building::Treasury => "Caisse",
            Building::Intendance => "Intendance",
            Building::Infirmary => "Infirmerie",
            Building::Departure => "Départ",
            Building::Charrette => "Charrette",
        }
    }
}

/// A place of the village shown in the top view: a building, or a plot to
/// build on (its rank among the town's houses).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Building(Building),
    Plot(usize),
}

/// Where each building is (none outside the village); the plots to build on
/// come from the village, which changes as it grows. The top view only works
/// from inside the village's zone (X.1): elsewhere in the campaign (a dungeon
/// slot), the list sent is empty and the client leaves the view.
#[derive(Default)]
pub struct IcebreakerBuildings {
    /// Each place, its bounds and its ground altitude (the client projects it
    /// at that height, X.2).
    places: Vec<(Place, Aabr<i32>, i32)>,
    zone: Option<Zone>,
}

impl IcebreakerBuildings {
    /// The buildings of `Building::ALL` with their bounds and altitude, if any.
    pub fn new(bounds: impl IntoIterator<Item = Option<(Aabr<i32>, i32)>>) -> Self {
        Self {
            places: Building::ALL
                .into_iter()
                .zip(bounds)
                .filter_map(|(building, bounds)| Some((building, bounds?)))
                .inspect(|(building, (bounds, alt))| {
                    info!(?building, ?bounds, alt, "Icebreaker: village building")
                })
                .map(|(building, (bounds, alt))| (Place::Building(building), bounds, alt))
                .collect(),
            zone: None,
        }
    }

    /// The village's zone, out of which the top view does not work (none: no
    /// limit, for a village alone).
    pub fn with_zone(self, zone: Option<Zone>) -> Self { Self { zone, ..self } }

    /// Whether a player at `pos` can see the village's top view.
    pub fn in_village(&self, pos: Vec2<f32>) -> bool {
        self.zone.is_none_or(|zone| zone.push_back(pos).is_none())
    }

    /// The buildings and the plots to build on (rank, bounds, altitude).
    pub fn places(
        &self,
        plots: impl IntoIterator<Item = (usize, Aabr<i32>, i32)>,
    ) -> Vec<(Place, Aabr<i32>, i32)> {
        self.places
            .iter()
            .copied()
            .chain(
                plots
                    .into_iter()
                    .map(|(rank, bounds, alt)| (Place::Plot(rank), bounds, alt)),
            )
            .collect()
    }
}

/// The place at `wpos`, if any.
fn place_at(places: &[(Place, Aabr<i32>, i32)], wpos: Vec2<f32>) -> Option<Place> {
    let wpos = wpos.map(|e| e.floor() as i32);
    places
        .iter()
        .find(|(_, bounds, _)| bounds.contains_point(wpos))
        .map(|(place, ..)| *place)
}

/// Hours of map time left until map time `fin`, at map time `now`.
pub(crate) fn hours_left(fin: f64, now: f64) -> f64 { ((fin - now) / 3600.0).max(0.0) }

/// Icebreaker : les bâtiments et les terrains du plan (jalon N), chacun avec ce
/// qu'il faut en savoir sans l'ouvrir : l'or de la caisse, le prix du soin, les
/// recrues du jour, les inscrits au départ, le prix d'un terrain, la fin d'un
/// chantier.
pub fn liste(
    places: &[(Place, Aabr<i32>, i32)],
    village: &IcebreakerVillage,
    config: &VillageConfig,
    caisse: u64,
    prix_soin: u64,
    recrues: usize,
    inscrits: Option<usize>,
    now: f64,
) -> Vec<Batiment> {
    let pluriel = |n: usize, mot: &str| format!("{n} {mot}{}", if n > 1 { "s" } else { "" });
    places
        .iter()
        .map(|(place, emprise, altitude)| {
            let (nom, sorte, etat) = match place {
                Place::Building(building) => {
                    let etat = match building {
                        Building::Warehouse | Building::Intendance => String::new(),
                        Building::Treasury => format!("{caisse} or"),
                        Building::Infirmary => format!("Soin : {prix_soin} or"),
                        Building::Departure => inscrits
                            .map_or_else(|| "Personne".to_string(), |n| pluriel(n, "inscrit")),
                        Building::Charrette => pluriel(recrues, "recrue"),
                    };
                    (building.name(), building.id(), etat)
                },
                // A plot's kind tells its look (X.4): "terrain", or "chantier"
                // while a house is being built.
                Place::Plot(rank) => match village.chantier(*rank) {
                    Some(chantier) => (
                        "Chantier",
                        "chantier",
                        format!(
                            "Fini dans {}",
                            crate::icebreaker_panneaux::heures(hours_left(chantier.fin, now).ceil())
                        ),
                    ),
                    // « Terrain » : le nom tient dans la case du plan ; son panneau dit le reste.
                    None => (
                        "Terrain",
                        "terrain",
                        format!("{} or", config.prix_maison),
                    ),
                },
            };
            Batiment {
                nom: texte(nom),
                emprise: *emprise,
                altitude: *altitude,
                sorte: sorte.to_string(),
                etat: texte(etat),
            }
        })
        .collect()
}

/// Icebreaker : le serveur retient ce que le joueur a d'ouvert (jalon N) ; le
/// système `sys::icebreaker_panneaux` envoie les bâtiments et le panneau, puis
/// ce qui change.
impl ServerEvent for IcebreakerBuildingEvent {
    type SystemData<'a> = (
        Read<'a, IcebreakerBuildings>,
        Read<'a, IcebreakerVillage>,
        Write<'a, IcebreakerPanneaux>,
    );

    fn handle(
        events: impl ExactSizeIterator<Item = Self>,
        (buildings, village, mut panneaux): Self::SystemData<'_>,
    ) {
        for event in events {
            match event.demande {
                Demande::Ouvrir => panneaux.ouvrir(event.entity),
                Demande::Batiment(wpos) => {
                    let places = buildings.places(village.open_plots());
                    if let Some(place) = place_at(&places, wpos) {
                        panneaux.choisir(event.entity, place);
                    }
                },
                Demande::FermerPanneau => panneaux.fermer_panneau(event.entity),
                Demande::Fermer => panneaux.fermer(event.entity),
            }
        }
    }
}

/// The cart's panel for the player of `presence`: the cart is renewed on a new
/// day of map time, and filled at once if a ghost has no one to take.
pub fn charrette_panneau(
    registre: &mut IcebreakerRegistre,
    now: f64,
    presence: Option<&Presence>,
    player: Option<&Player>,
    presences: &ReadStorage<Presence>,
    players: &ReadStorage<Player>,
) -> Panneau {
    let vide = |message: &str| {
        Panneau::nouveau("armure", Building::Charrette.name(), "").message(message, Sorte::Refus)
    };
    if !icebreaker_registre::active() {
        return vide("Pas de registre ici.");
    }
    let Some(player) = player else {
        return vide("Il faut être connecté pour voir le registre.");
    };
    let uuid = player.uuid().to_string();
    let current = match presence.map(|p| p.kind) {
        Some(PresenceKind::Character(id)) => Some(id.0),
        _ => None,
    };
    let config = icebreaker_registre::config();
    let mut rng = rand::rng();
    registre.renouveler(icebreaker_registre::jour_de(now), &config, &mut rng);
    let joueurs = icebreaker_registre::joueurs(presences, players);
    if current.is_none() && !registre.de_quoi_jouer(&uuid, &joueurs) {
        registre.completer(&config, &mut rng);
    }
    icebreaker_registre::panneau_pour(registre, &uuid, current, &joueurs)
}

/// `/echanger <entrepot|intendance>` (the panels of the top view, V.12):
/// opens the trade window with the storekeeper or the intendant, wherever the
/// player is in the village.
pub(crate) fn handle_echanger(
    server: &mut crate::Server,
    client: specs::Entity,
    _target: specs::Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), Content> {
    use common::{
        comp::{
            Agent,
            agent::{Mark, TradingBehavior},
            invite::InviteKind,
        },
        event::{EventBus, InitiateInviteEvent},
    };
    use specs::{Join, WorldExt};
    let wanted = |agent: &Agent| match args.first().map(String::as_str) {
        Some("entrepot") => matches!(agent.behavior.trading_behavior, TradingBehavior::Storage),
        Some("intendance") => agent.mark == Some(Mark::Intendant),
        _ => false,
    };
    let ecs = server.state.ecs();
    let npc = (&ecs.read_storage::<Agent>(), &ecs.read_storage::<Uid>())
        .join()
        .find(|(agent, _)| wanted(agent))
        .map(|(_, uid)| *uid);
    let Some(npc) = npc else {
        return Err(Content::Plain(
            "Personne à ce bâtiment pour l'instant (au village seulement).".to_string(),
        ));
    };
    ecs.read_resource::<EventBus<InitiateInviteEvent>>()
        .emit_now(InitiateInviteEvent(client, npc, InviteKind::Trade));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_finds_the_building_under_it() {
        let square = |x: i32| Aabr {
            min: Vec2::new(x, 0),
            max: Vec2::new(x + 10, 10),
        };
        let buildings = IcebreakerBuildings::new([
            Some((square(0), 0)),
            None,
            Some((square(20), 0)),
            Some((square(40), 0)),
        ]);
        let places = buildings.places([(7, square(60), 0)]);
        let at = |x, y| place_at(&places, Vec2::new(x, y));
        assert_eq!(at(5.5, 5.0), Some(Place::Building(Building::Warehouse)));
        assert_eq!(at(25.0, 9.9), Some(Place::Building(Building::Intendance)));
        assert_eq!(at(45.0, 1.0), Some(Place::Building(Building::Infirmary)));
        assert_eq!(at(65.0, 1.0), Some(Place::Plot(7)));
        assert_eq!(at(15.0, 5.0), None);
    }

    #[test]
    fn the_top_view_only_works_inside_the_village_zone() {
        let without_zone = IcebreakerBuildings::new([]);
        assert!(without_zone.in_village(Vec2::new(5000.0, 0.0)));
        let with_zone = IcebreakerBuildings::new([]).with_zone(Some(Zone {
            center: Vec2::new(100.0, 100.0),
            radius: 50.0,
        }));
        assert!(with_zone.in_village(Vec2::new(120.0, 100.0)));
        assert!(!with_zone.in_village(Vec2::new(1000.0, 100.0)));
    }

    #[test]
    fn the_plan_tells_each_place_s_state_without_opening_it() {
        let square = |x: i32| Aabr {
            min: Vec2::new(x, 0),
            max: Vec2::new(x + 10, 10),
        };
        let buildings = IcebreakerBuildings::new(
            Building::ALL.iter().enumerate().map(|(i, _)| Some((square(20 * i as i32), 0))),
        );
        let places = buildings.places([(7, square(200), 0)]);
        let config = crate::icebreaker_village::config();
        let village = IcebreakerVillage::default();
        let liste = liste(&places, &village, &config, 420, 60, 3, None, 0.0);
        let etat = |sorte: &str| {
            liste
                .iter()
                .find(|b| b.sorte == sorte)
                .map(|b| b.etat.clone())
                .unwrap_or_else(|| panic!("no {sorte} in {liste:?}"))
        };
        assert_eq!(etat("caisse"), texte("420 or"));
        assert_eq!(etat("infirmerie"), texte("Soin : 60 or"));
        assert_eq!(etat("charrette"), texte("3 recrues"));
        assert_eq!(etat("depart"), texte("Personne"));
        assert_eq!(etat("entrepot"), texte(""));
        assert_eq!(etat("terrain"), texte(format!("{} or", config.prix_maison)));
        let terrain = liste.iter().find(|b| b.sorte == "terrain").unwrap();
        assert_eq!(terrain.nom, texte("Terrain"));

        let seul = super::liste(&places, &village, &config, 0, 60, 1, Some(1), 0.0);
        let etat = |sorte: &str| seul.iter().find(|b| b.sorte == sorte).unwrap().etat.clone();
        assert_eq!(etat("charrette"), texte("1 recrue"));
        assert_eq!(etat("depart"), texte("1 inscrit"));
    }
}
