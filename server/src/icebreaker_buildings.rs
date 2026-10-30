//! Icebreaker: the village buildings (V.6). Each is one of the town's houses
//! nearest to the square, its NPC standing at the door. In the top view of the
//! village, the client asks for the buildings (to show their names and
//! highlight the one under the cursor), and a click on one for its panel. The
//! plots to build on (jalon C) are shown and clicked likewise.

use crate::{
    client::Client,
    events::ServerEvent,
    icebreaker_services::IcebreakerServices,
    icebreaker_treasury::IcebreakerTreasury,
    icebreaker_village::{self, Chantier, IcebreakerVillage, VillageConfig},
    icebreaker_warehouse::IcebreakerWarehouse,
};
use common::{
    comp::{Content, LocalizationArg, item::ItemDesc, item::ItemI18n},
    event::IcebreakerBuildingEvent,
    resources::TimeOfDay,
};
use common_net::msg::ServerGeneral;
use specs::{Read, ReadStorage};
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
}

impl Building {
    /// In the order of the village's houses, nearest to the square first.
    pub const ALL: [Building; 5] = [
        Building::Warehouse,
        Building::Treasury,
        Building::Intendance,
        Building::Infirmary,
        Building::Departure,
    ];

    fn name(self) -> &'static str {
        match self {
            Building::Warehouse => "Entrepôt",
            Building::Treasury => "Caisse",
            Building::Intendance => "Intendance",
            Building::Infirmary => "Infirmerie",
            Building::Departure => "Départ",
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
/// come from the village, which changes as it grows.
#[derive(Default)]
pub struct IcebreakerBuildings(Vec<(Place, Aabr<i32>)>);

impl IcebreakerBuildings {
    /// The buildings of `Building::ALL` with their bounds, if any.
    pub fn new(bounds: impl IntoIterator<Item = Option<Aabr<i32>>>) -> Self {
        Self(
            Building::ALL
                .into_iter()
                .zip(bounds)
                .filter_map(|(building, bounds)| Some((building, bounds?)))
                .inspect(|(building, bounds)| {
                    info!(?building, ?bounds, "Icebreaker: village building")
                })
                .map(|(building, bounds)| (Place::Building(building), bounds))
                .collect(),
        )
    }

    /// The buildings and the plots to build on (rank, bounds).
    pub fn places(
        &self,
        plots: impl IntoIterator<Item = (usize, Aabr<i32>)>,
    ) -> Vec<(Place, Aabr<i32>)> {
        self.0
            .iter()
            .copied()
            .chain(plots.into_iter().map(|(rank, bounds)| (Place::Plot(rank), bounds)))
            .collect()
    }
}

/// The place at `wpos`, if any.
fn place_at(places: &[(Place, Aabr<i32>)], wpos: Vec2<f32>) -> Option<Place> {
    let wpos = wpos.map(|e| e.floor() as i32);
    places
        .iter()
        .find(|(_, bounds)| bounds.contains_point(wpos))
        .map(|(place, _)| *place)
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
type Panel = (Content, Vec<Content>, Vec<(Content, Vec<String>)>);

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
) -> Panel {
    let plain = |text: String| Content::Plain(text);
    let building = match place {
        Place::Building(building) => building,
        Place::Plot(rank) => {
            let config = icebreaker_village::config();
            if let Some(chantier) = village.chantier(rank) {
                return (
                    plain("Chantier".to_string()),
                    vec![plain(format!(
                        "Une maison est en construction : finie dans {:.1} h de carte.",
                        hours_left(chantier.fin, now)
                    ))],
                    Vec::new(),
                );
            }
            return (
                plain("Terrain à bâtir".to_string()),
                vec![
                    plain(format!(
                        "Une maison : {} pièces d'or de la caisse (la caisse en a {}).",
                        config.prix_maison,
                        treasury.coins()
                    )),
                    plain(format!(
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
    let mut actions = Vec::new();
    let lines = match building {
        Building::Warehouse => {
            let items = warehouse.load_items();
            if items.is_empty() {
                vec![plain("L'entrepôt est vide.".to_string())]
            } else {
                let i18n = ItemI18n::new_expect();
                items
                    .iter()
                    .map(|item| {
                        Content::localized_with_args("icebreaker-bilan-item", [
                            ("amount", LocalizationArg::from(u64::from(item.amount()))),
                            ("item", LocalizationArg::from(item.i18n(&i18n).0)),
                        ])
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
                plain(format!("La caisse du village : {} pièces d'or.", treasury.coins())),
                plain("Elle paie les chantiers du village.".to_string()),
            ]
        },
        Building::Intendance => vec![
            plain("Potions et vivres, aux prix de la ville.".to_string()),
            plain("L'intendant, à la porte, vend et rachète (T).".to_string()),
        ],
        Building::Infirmary => {
            let price = services.heal_price();
            actions = vec![button(format!("Me soigner ({price} or)"), &["soigner"])];
            vec![plain(format!("Soin complet : {price} pièces d'or."))]
        },
        Building::Departure => std::iter::once(plain(
            "Le chef d'expédition, à la porte, organise les départs (E) :".to_string(),
        ))
        .chain(
            crate::icebreaker_expedition::destinations()
                .into_iter()
                .map(|(_, lieu)| {
                    plain(if lieu.actif {
                        format!("- {}", lieu.nom)
                    } else {
                        format!("- {} (pas encore ouvert)", lieu.nom)
                    })
                }),
        )
        .collect(),
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
    );

    fn handle(
        events: impl ExactSizeIterator<Item = Self>,
        (clients, buildings, warehouse, treasury, services, village, time_of_day): Self::SystemData<'_>,
    ) {
        let now = time_of_day.0;
        let places = buildings.places(village.open_plots());
        for event in events {
            let Some(client) = clients.get(event.entity) else {
                continue;
            };
            match event.wpos {
                Some(wpos) => {
                    if let Some(place) = place_at(&places, wpos) {
                        let (title, lines, action) =
                            panel(place, &warehouse, &treasury, &services, &village, now);
                        client.send_fallible(ServerGeneral::IcebreakerPanel(title, lines, action));
                    }
                },
                None => {
                    let config = icebreaker_village::config();
                    client.send_fallible(ServerGeneral::IcebreakerBuildings(
                        places
                            .iter()
                            .map(|(place, bounds)| match place {
                                Place::Building(building) => {
                                    (Content::Plain(building.name().to_string()), *bounds, false)
                                },
                                Place::Plot(rank) => {
                                    let name = plot_name(&config, village.chantier(*rank), now);
                                    (Content::Plain(name), *bounds, true)
                                },
                            })
                            .collect(),
                    ))
                },
            }
        }
    }
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
        let buildings =
            IcebreakerBuildings::new([Some(square(0)), None, Some(square(20)), Some(square(40))]);
        let places = buildings.places([(7, square(60))]);
        let at = |x, y| place_at(&places, Vec2::new(x, y));
        assert_eq!(at(5.5, 5.0), Some(Place::Building(Building::Warehouse)));
        assert_eq!(at(25.0, 9.9), Some(Place::Building(Building::Intendance)));
        assert_eq!(at(45.0, 1.0), Some(Place::Building(Building::Infirmary)));
        assert_eq!(at(65.0, 1.0), Some(Place::Plot(7)));
        assert_eq!(at(15.0, 5.0), None);
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
