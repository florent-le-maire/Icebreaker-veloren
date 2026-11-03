//! Icebreaker: the classes of the recruits of the cart (jalon R, decision
//! 0017). A class is a starting equipment and skills of its weapon (R.6), in
//! a chosen order on the ability slots (R.8), read from
//! `assets/icebreaker/classes.ron`.

use common::{
    assets::{AssetExt, Ron},
    comp::{
        ActiveAbilities, BASE_ABILITY_LIMIT, Inventory, ItemKey, SkillSet,
        ability::AuxiliaryAbility,
        inventory::{item::tool::AbilityKind, loadout_builder::LoadoutBuilder, slot::EquipSlot},
        skills::Skill,
    },
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
    /// Skills put on the first ability slots, in this order (R.8); the other
    /// unlocked skills follow. Empty: Veloren's order.
    #[serde(default)]
    pub touches: Vec<Skill>,
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

/// The skills a recruit of `class` starts with (R.6).
pub fn starting_skill_set(class: &Class) -> SkillSet {
    SkillSetBuilder::from_asset_expect(&class.competences).build()
}

/// The ability slots of a recruit of `class` (R.8): its `touches` first, then
/// the other unlocked abilities of its weapon in Veloren's order.
pub fn starting_abilities(
    class: &Class,
    inventory: &Inventory,
    skill_set: &SkillSet,
) -> ActiveAbilities {
    let default = ActiveAbilities::default_limited(BASE_ABILITY_LIMIT);
    if class.touches.is_empty() {
        return default;
    }
    let weapon_abilities = inventory
        .equipped(EquipSlot::ActiveMainhand)
        .and_then(|item| item.item_config())
        .map(|config| config.abilities.abilities.as_slice())
        .unwrap_or_default();
    let index_of = |skill: &Skill| {
        weapon_abilities.iter().position(|ability| match ability {
            AbilityKind::Simple(s, _) => *s == Some(*skill),
            AbilityKind::Contextualized { abilities, .. } => {
                abilities.iter().any(|(_, (s, _))| *s == Some(*skill))
            },
        })
    };
    let mut slots: Vec<AuxiliaryAbility> = class
        .touches
        .iter()
        .filter_map(index_of)
        .map(AuxiliaryAbility::MainWeapon)
        .collect();
    for ability in default
        .auxiliary_set(Some(inventory), Some(skill_set))
        .iter()
    {
        if *ability != AuxiliaryAbility::Empty && !slots.contains(ability) {
            slots.push(*ability);
        }
    }
    slots.resize(BASE_ABILITY_LIMIT, AuxiliaryAbility::Empty);
    ActiveAbilities::from_auxiliary(
        std::iter::once((ActiveAbilities::active_auxiliary_key(Some(inventory)), slots))
            .collect(),
        Some(BASE_ABILITY_LIMIT),
    )
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
    fn icebreaker_classes_start_with_their_skills_on_the_first_slots() {
        use common::comp::ability::AbilityInput;
        // R.8 : le guerrier a ses trois postures puis leurs deux prérequis.
        for (id, count) in [("guerrier", 5), ("mage", 2), ("archer", 2)] {
            let class = class(id).unwrap_or_else(|| panic!("class {id} missing"));
            let inventory = starting_inventory(&class);
            // Panics in tests if a skill lacks its group or a prerequisite.
            let skill_set = starting_skill_set(&class);
            let abilities = starting_abilities(&class, &inventory, &skill_set);
            let slots = abilities
                .auxiliary_set(Some(&inventory), Some(&skill_set))
                .into_owned();
            let ids: Vec<_> = (0..BASE_ABILITY_LIMIT)
                .map(|i| {
                    abilities
                        .get_ability(
                            AbilityInput::Auxiliary(i),
                            Some(&inventory),
                            Some(&skill_set),
                            None,
                        )
                        .ability_id(None, Some(&inventory), Some(&skill_set), None, None, None)
                })
                .collect();
            if id == "guerrier" {
                assert_eq!(
                    ids[..3],
                    [
                        Some("common.abilities.sword.defensive_riposte"),
                        Some("common.abilities.sword.heavy_sweep"),
                        Some("common.abilities.sword.cleaving_whirlwind_slice"),
                    ],
                    "{slots:?}"
                );
            }
            assert_eq!(slots.len(), BASE_ABILITY_LIMIT, "{id}");
            let skills = &slots[..count];
            assert!(
                skills
                    .iter()
                    .all(|s| matches!(s, AuxiliaryAbility::MainWeapon(_)))
                    && skills
                        .iter()
                        .enumerate()
                        .all(|(i, s)| !skills[..i].contains(s)),
                "{id}: {slots:?}"
            );
            assert!(
                slots[count..].iter().all(|s| *s == AuxiliaryAbility::Empty),
                "{id}: {slots:?}"
            );
        }
    }
}
