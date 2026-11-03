//! Icebreaker: the classes of the recruits of the cart (jalon R, decision
//! 0017). For now a class is a starting equipment, read from
//! `assets/icebreaker/classes.ron`.

use common::{
    assets::{AssetExt, Ron},
    comp::{Inventory, inventory::loadout_builder::LoadoutBuilder},
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Class {
    /// What the class is called in commands and in the registry file.
    pub id: String,
    /// What the players read.
    pub nom: String,
    /// Loadout asset: armour and weapons in hand.
    pub equipement: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ClassesConfig {
    classes: Vec<Class>,
}

/// The playable classes.
pub fn classes() -> Vec<Class> {
    Ron::<ClassesConfig>::load_expect("icebreaker.classes")
        .read()
        .0
        .classes
        .clone()
}

/// The class called `id`, if any.
pub fn class(id: &str) -> Option<Class> { classes().into_iter().find(|class| class.id == id) }

/// The inventory a recruit of `class` starts with: its equipment only (the
/// items every character gets are added by the character creator).
pub fn starting_inventory(class: &Class) -> Inventory {
    let loadout = LoadoutBuilder::empty()
        .with_asset_expect(&class.equipement, &mut rand::rng(), None)
        .build();
    Inventory::with_loadout_humanoid(loadout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::comp::{
        inventory::slot::{ArmorSlot, EquipSlot},
        item::ItemDefinitionId,
    };

    fn equipped(inventory: &Inventory, slot: EquipSlot) -> Option<String> {
        inventory
            .equipped(slot)
            .and_then(|item| match item.item_definition_id() {
                ItemDefinitionId::Simple(id) => Some(id.to_string()),
                _ => None,
            })
    }

    #[test]
    fn icebreaker_classes_give_their_weapon_and_armour() {
        for (id, weapon) in [
            ("guerrier", "common.items.weapons.sword.starter"),
            ("mage", "common.items.weapons.staff.starter_staff"),
            ("archer", "common.items.weapons.bow.starter"),
        ] {
            let class = class(id).unwrap_or_else(|| panic!("class {id} missing"));
            let inventory = starting_inventory(&class);
            assert_eq!(
                equipped(&inventory, EquipSlot::ActiveMainhand).as_deref(),
                Some(weapon),
                "{id}"
            );
            assert!(equipped(&inventory, EquipSlot::Armor(ArmorSlot::Chest)).is_some(), "{id}");
            assert!(equipped(&inventory, EquipSlot::Glider).is_some(), "{id}");
        }
        assert_eq!(classes().len(), 3);
    }
}
