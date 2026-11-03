//! Icebreaker: the classes of the recruits of the cart (jalon R, decision
//! 0017). A class is a starting equipment and two skills of its weapon (R.6),
//! read from `assets/icebreaker/classes.ron`.

use common::{
    assets::{AssetExt, Ron},
    comp::{Inventory, ItemKey, SkillSet, inventory::loadout_builder::LoadoutBuilder},
    skillset_builder::SkillSetBuilder,
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
    /// Skill set asset: the skills unlocked at the start (R.6).
    pub competences: String,
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

/// The items drawn as icons for a recruit of `class` (X.8): its weapon in
/// hand and its chest armour.
pub fn icones(class: &Class) -> Vec<ItemKey> {
    use common::comp::inventory::slot::{ArmorSlot, EquipSlot};
    let inventory = starting_inventory(class);
    [EquipSlot::ActiveMainhand, EquipSlot::Armor(ArmorSlot::Chest)]
        .into_iter()
        .filter_map(|slot| inventory.equipped(slot))
        .map(ItemKey::from)
        .collect()
}

/// The inventory a recruit of `class` starts with: its equipment only (the
/// items every character gets are added by the character creator).
pub fn starting_inventory(class: &Class) -> Inventory {
    let loadout = LoadoutBuilder::empty()
        .with_asset_expect(&class.equipement, &mut rand::rng(), None)
        .build();
    Inventory::with_loadout_humanoid(loadout)
}

/// The skills a recruit of `class` starts with (R.6); they fill the first
/// ability slots by themselves.
pub fn starting_skill_set(class: &Class) -> SkillSet {
    SkillSetBuilder::from_asset_expect(&class.competences).build()
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
            // X.8 : deux icônes, l'arme d'abord.
            let icones = icones(&class);
            assert_eq!(icones.len(), 2, "{id}");
            assert_eq!(icones[0], ItemKey::Simple(weapon.to_string()), "{id}");
        }
        assert_eq!(classes().len(), 3);
    }

    #[test]
    fn icebreaker_classes_start_with_two_skills_on_slots_1_and_2() {
        use common::comp::{ActiveAbilities, BASE_ABILITY_LIMIT, ability::AuxiliaryAbility};
        for class in classes() {
            let inventory = starting_inventory(&class);
            // Panics in tests if a skill lacks its group or a prerequisite.
            let skill_set = starting_skill_set(&class);
            let slots = ActiveAbilities::default_limited(BASE_ABILITY_LIMIT)
                .auxiliary_set(Some(&inventory), Some(&skill_set))
                .into_owned();
            let id = &class.id;
            assert_eq!(slots.len(), BASE_ABILITY_LIMIT, "{id}");
            assert!(
                matches!(slots[0], AuxiliaryAbility::MainWeapon(_))
                    && matches!(slots[1], AuxiliaryAbility::MainWeapon(_))
                    && slots[0] != slots[1],
                "{id}: {slots:?}"
            );
            assert!(
                slots[2..].iter().all(|s| *s == AuxiliaryAbility::Empty),
                "{id}: {slots:?}"
            );
        }
    }
}
