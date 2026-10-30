//! Icebreaker: the village buildings (V.6). Each is one of the town's houses
//! nearest to the square, its NPC standing at the door. In the top view of the
//! village, the client asks for the buildings (to show their names and
//! highlight the one under the cursor), and a click on one for its panel.

use crate::{
    client::Client,
    events::ServerEvent,
    icebreaker_services::IcebreakerServices,
    icebreaker_treasury::IcebreakerTreasury,
    icebreaker_warehouse::IcebreakerWarehouse,
};
use common::{
    comp::{Content, LocalizationArg, item::ItemDesc, item::ItemI18n},
    event::IcebreakerBuildingEvent,
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
}

impl Building {
    /// In the order of the village's houses, nearest to the square first.
    pub const ALL: [Building; 4] = [
        Building::Warehouse,
        Building::Treasury,
        Building::Intendance,
        Building::Infirmary,
    ];

    fn name(self) -> &'static str {
        match self {
            Building::Warehouse => "Entrepôt",
            Building::Treasury => "Caisse",
            Building::Intendance => "Intendance",
            Building::Infirmary => "Infirmerie",
        }
    }
}

/// Where each building is (none outside the village).
#[derive(Default)]
pub struct IcebreakerBuildings(Vec<(Building, Aabr<i32>)>);

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
                .collect(),
        )
    }

    /// The building at `wpos`, if any.
    pub fn at(&self, wpos: Vec2<f32>) -> Option<Building> {
        let wpos = wpos.map(|e| e.floor() as i32);
        self.0
            .iter()
            .find(|(_, bounds)| bounds.contains_point(wpos))
            .map(|(building, _)| *building)
    }
}

/// The panel of `building`: title and lines.
fn panel(
    building: Building,
    warehouse: &IcebreakerWarehouse,
    treasury: &IcebreakerTreasury,
    services: &IcebreakerServices,
) -> (Content, Vec<Content>) {
    let plain = |text: String| Content::Plain(text);
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
        Building::Treasury => vec![
            plain(format!("La caisse du village : {} pièces d'or.", treasury.coins())),
            plain("Le trésorier, à la porte, prend l'or (T).".to_string()),
        ],
        Building::Intendance => vec![
            plain("Potions et vivres, aux prix de la ville.".to_string()),
            plain("L'intendant, à la porte, vend et rachète (T).".to_string()),
        ],
        Building::Infirmary => vec![
            plain(format!(
                "Soin complet : {} pièces d'or.",
                services.heal_price()
            )),
            plain("L'infirmier, à la porte, soigne (E).".to_string()),
        ],
    };
    (plain(building.name().to_string()), lines)
}

impl ServerEvent for IcebreakerBuildingEvent {
    type SystemData<'a> = (
        ReadStorage<'a, Client>,
        Read<'a, IcebreakerBuildings>,
        Read<'a, IcebreakerWarehouse>,
        Read<'a, IcebreakerTreasury>,
        Read<'a, IcebreakerServices>,
    );

    fn handle(
        events: impl ExactSizeIterator<Item = Self>,
        (clients, buildings, warehouse, treasury, services): Self::SystemData<'_>,
    ) {
        for event in events {
            let Some(client) = clients.get(event.entity) else {
                continue;
            };
            match event.wpos {
                Some(wpos) => {
                    if let Some(building) = buildings.at(wpos) {
                        let (title, lines) = panel(building, &warehouse, &treasury, &services);
                        client.send_fallible(ServerGeneral::IcebreakerPanel(title, lines));
                    }
                },
                None => client.send_fallible(ServerGeneral::IcebreakerBuildings(
                    buildings
                        .0
                        .iter()
                        .map(|(building, bounds)| {
                            (Content::Plain(building.name().to_string()), *bounds)
                        })
                        .collect(),
                )),
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
        assert_eq!(buildings.at(Vec2::new(5.5, 5.0)), Some(Building::Warehouse));
        assert_eq!(buildings.at(Vec2::new(25.0, 9.9)), Some(Building::Intendance));
        assert_eq!(buildings.at(Vec2::new(45.0, 1.0)), Some(Building::Infirmary));
        assert_eq!(buildings.at(Vec2::new(15.0, 5.0)), None);
    }
}
