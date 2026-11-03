//! Icebreaker: the village buildings (V.6). Each is one of the town's houses
//! nearest to the square, its NPC standing at the door. In the top view of the
//! village, the client asks for the buildings (to show their names and
//! highlight the one under the cursor), and a click on one for its panel. The
//! plots to build on (jalon C) are shown and clicked likewise.

use crate::{
    client::Client,
    events::ServerEvent,
    icebreaker_registre::{self, IcebreakerRegistre},
    icebreaker_services::IcebreakerServices,
    icebreaker_treasury::IcebreakerTreasury,
    icebreaker_village::{self, Chantier, IcebreakerVillage, VillageConfig},
    icebreaker_warehouse::IcebreakerWarehouse,
    sys::icebreaker_zone::Zone,
};
use common::{
    comp::{
        Content, ItemKey, LocalizationArg, Player, Pos, Presence, PresenceKind,
        item::ItemDesc, item::ItemI18n,
    },
    event::IcebreakerBuildingEvent,
    resources::TimeOfDay,
    uid::Uid,
};
use common_net::msg::ServerGeneral;
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
fn hours_left(fin: f64, now: f64) -> f64 { ((fin - now) / 3600.0).max(0.0) }

/// What a plot to build on is called on the top view.
fn plot_name(config: &VillageConfig, chantier: Option<Chantier>, now: f64) -> String {
    match chantier {
        Some(chantier) => format!("Chantier ({:.0} h)", hours_left(chantier.fin, now).ceil()),
        None => format!("Terrain à bâtir ({} or)", config.prix_maison),
    }
}

/// What a panel shows: title, lines, and buttons (label, chat command).
/// A line of a panel: its text, and the items drawn as icons before it (X.3).
type Ligne = (Content, Vec<ItemKey>);
type Panel = (Content, Vec<Ligne>, Vec<(Content, Vec<String>)>);

/// A button sending the chat command `command`.
fn button(label: impl Into<String>, command: &[&str]) -> (Content, Vec<String>) {
    (
        Content::Plain(label.into()),
        command.iter().map(|word| word.to_string()).collect(),
    )
}

/// The panel of `place` at map time `now`.
fn panel(
    place: Place,
    warehouse: &IcebreakerWarehouse,
    treasury: &IcebreakerTreasury,
    services: &IcebreakerServices,
    village: &IcebreakerVillage,
    now: f64,
    player: Option<Uid>,
) -> Panel {
    let plain = |text: String| Content::Plain(text);
    let ligne = |text: String| (Content::Plain(text), Vec::new());
    let building = match place {
        Place::Building(building) => building,
        Place::Plot(rank) => {
            let config = icebreaker_village::config();
            if let Some(chantier) = village.chantier(rank) {
                return (
                    plain("Chantier".to_string()),
                    vec![ligne(format!(
                        "Une maison est en construction : finie dans {:.1} h de carte.",
                        hours_left(chantier.fin, now)
                    ))],
                    Vec::new(),
                );
            }
            return (
                plain("Terrain à bâtir".to_string()),
                vec![
                    ligne(format!(
                        "Une maison : {} pièces d'or de la caisse (la caisse en a {}).",
                        config.prix_maison,
                        treasury.coins()
                    )),
                    ligne(format!(
                        "Chantier : {} h de carte, puis de nouveaux habitants.",
                        config.heures_chantier
                    )),
                ],
                vec![button(format!("Bâtir ({} or)", config.prix_maison), &[
                    "batir",
                    &rank.to_string(),
                ])],
            );
        },
    };
    let actions;
    let lines = match building {
        Building::Warehouse => {
            actions = vec![button("Ouvrir l'échange", &["echanger", "entrepot"])];
            let items = warehouse.load_items();
            if items.is_empty() {
                vec![ligne("L'entrepôt est vide.".to_string())]
            } else {
                let i18n = ItemI18n::new_expect();
                items
                    .iter()
                    .map(|item| {
                        (
                            Content::localized_with_args("icebreaker-bilan-item", [
                                ("amount", LocalizationArg::from(u64::from(item.amount()))),
                                ("item", LocalizationArg::from(item.i18n(&i18n).0)),
                            ]),
                            vec![ItemKey::from(item)],
                        )
                    })
                    .collect()
            }
        },
        Building::Treasury => {
            actions = vec![
                button("Verser 10 or", &["caisse", "10"]),
                button("Verser 50 or", &["caisse", "50"]),
                button("Verser 100 or", &["caisse", "100"]),
                button("Verser tout mon or", &["caisse", "tout"]),
            ];
            vec![
                ligne(format!("La caisse du village : {} pièces d'or.", treasury.coins())),
                ligne("Elle paie les chantiers du village.".to_string()),
            ]
        },
        Building::Intendance => {
            actions = vec![button("Ouvrir l'échange", &["echanger", "intendance"])];
            vec![
                ligne("Potions et vivres, aux prix de la ville.".to_string()),
                ligne("L'intendant vend, et rachète ce que vous rapportez.".to_string()),
            ]
        },
        Building::Infirmary => {
            let price = services.heal_price();
            actions = vec![button(format!("Me soigner ({price} or)"), &["soigner"])];
            vec![ligne(format!("Soin complet : {price} pièces d'or."))]
        },
        // The expedition leader's question and answers, as buttons (V.11).
        Building::Departure => {
            let Some(player) = player else {
                return (plain(building.name().to_string()), Vec::new(), Vec::new());
            };
            let (question, buttons) = crate::icebreaker_expedition::panel_for(player);
            actions = buttons
                .into_iter()
                .map(|(label, command)| (plain(label), command))
                .collect();
            vec![ligne(question)]
        },
        // Built by the event handler, which knows the registry.
        Building::Charrette => {
            return (plain(building.name().to_string()), Vec::new(), Vec::new());
        },
    };
    (plain(building.name().to_string()), lines, actions)
}

impl ServerEvent for IcebreakerBuildingEvent {
    type SystemData<'a> = (
        ReadStorage<'a, Client>,
        Read<'a, IcebreakerBuildings>,
        Read<'a, IcebreakerWarehouse>,
        Read<'a, IcebreakerTreasury>,
        Read<'a, IcebreakerServices>,
        Read<'a, IcebreakerVillage>,
        Read<'a, TimeOfDay>,
        ReadStorage<'a, Uid>,
        Write<'a, IcebreakerRegistre>,
        ReadStorage<'a, Presence>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Pos>,
    );

    fn handle(
        events: impl ExactSizeIterator<Item = Self>,
        (
            clients,
            buildings,
            warehouse,
            treasury,
            services,
            village,
            time_of_day,
            uids,
            mut registre,
            presences,
            players,
            positions,
        ): Self::SystemData<'_>,
    ) {
        let now = time_of_day.0;
        let places = buildings.places(village.open_plots());
        for event in events {
            let Some(client) = clients.get(event.entity) else {
                continue;
            };
            // Out of the village (a dungeon slot of the campaign): no building,
            // which makes the client leave the top view (X.1).
            let in_village = positions
                .get(event.entity)
                .is_some_and(|pos| buildings.in_village(pos.0.xy()));
            match event.wpos {
                Some(_) if !in_village => {},
                None if !in_village => {
                    client.send_fallible(ServerGeneral::IcebreakerBuildings(Vec::new()));
                },
                Some(wpos) => {
                    if let Some(Place::Building(Building::Charrette)) = place_at(&places, wpos) {
                        let panel = charrette_panel(
                            &mut registre,
                            now,
                            presences.get(event.entity),
                            players.get(event.entity),
                            &presences,
                            &players,
                        );
                        client.send_fallible(ServerGeneral::IcebreakerPanel(
                            Content::Plain(Building::Charrette.name().to_string()),
                            panel
                                .0
                                .into_iter()
                                .map(|(line, icons)| (Content::Plain(line), icons))
                                .collect(),
                            panel
                                .1
                                .into_iter()
                                .map(|(label, command)| (Content::Plain(label), command))
                                .collect(),
                        ));
                    } else if let Some(place) = place_at(&places, wpos) {
                        let (title, lines, action) =
                            panel(
                            place,
                            &warehouse,
                            &treasury,
                            &services,
                            &village,
                            now,
                            uids.get(event.entity).copied(),
                        );
                        client.send_fallible(ServerGeneral::IcebreakerPanel(title, lines, action));
                    }
                },
                None => {
                    let config = icebreaker_village::config();
                    client.send_fallible(ServerGeneral::IcebreakerBuildings(
                        places
                            .iter()
                            .map(|(place, bounds, alt)| match place {
                                Place::Building(building) => (
                                    Content::Plain(building.name().to_string()),
                                    *bounds,
                                    *alt,
                                    building.id().to_string(),
                                ),
                                // A plot's kind tells its look (X.4): "terrain", or
                                // "chantier" while a house is being built.
                                Place::Plot(rank) => {
                                    let chantier = village.chantier(*rank);
                                    let name = plot_name(&config, chantier, now);
                                    let kind = if chantier.is_some() { "chantier" } else { "terrain" };
                                    (Content::Plain(name), *bounds, *alt, kind.to_string())
                                },
                            })
                            .collect(),
                    ))
                },
            }
        }
    }
}

/// The cart's panel for the player of `presence`: the cart is renewed on a new
/// day of map time, and filled at once if a ghost has no one to take.
fn charrette_panel(
    registre: &mut IcebreakerRegistre,
    now: f64,
    presence: Option<&Presence>,
    player: Option<&Player>,
    presences: &ReadStorage<Presence>,
    players: &ReadStorage<Player>,
) -> (Vec<(String, Vec<ItemKey>)>, Vec<(String, Vec<String>)>) {
    if !icebreaker_registre::active() {
        return (
            vec![("Pas de registre ici.".to_string(), Vec::new())],
            Vec::new(),
        );
    }
    let Some(player) = player else {
        return (Vec::new(), Vec::new());
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
    icebreaker_registre::panel_for(registre, &uuid, current, &joueurs)
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
    fn a_plot_is_named_after_its_price_or_its_chantier() {
        let config = icebreaker_village::config();
        assert!(plot_name(&config, None, 0.0).contains(&config.prix_maison.to_string()));
        let chantier = Chantier {
            rang: 7,
            fin: 3.0 * 3600.0,
        };
        assert_eq!(plot_name(&config, Some(chantier), 0.5 * 3600.0), "Chantier (3 h)");
    }
}
