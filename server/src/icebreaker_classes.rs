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
    icebreaker_competences::{Acquis, IcebreakerCompetences, JeuArmes, jeu_en_main, registre},
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
    /// Icebreaker : l'icône de la classe dans l'interface (jalon N), parmi les
    /// icônes d'armes du HUD : epees, epee, baton, arc, hache, marteau, sceptre.
    #[serde(default)]
    pub icone: String,
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

/// Icebreaker : l'inventaire de départ d'une classe décrite par nos données (jalon A.7) : son
/// équipement (armure, pioche en mains secondaires…) où les armes actives sont remplacées par celles
/// du jeu d'armes `jeu` (main gauche vidée s'il n'en a pas).
pub fn inventaire_avec_jeu(class: &Class, jeu: &JeuArmes) -> Inventory {
    use common::comp::{Item, inventory::slot::EquipSlot};
    use common::resources::Time;
    let mut inventory = starting_inventory(class);
    let (droite, gauche) = &jeu.armes;
    // Les mains d'abord vidées : une arme à deux mains ne tient pas avec un bouclier déjà là.
    inventory.replace_loadout_item(EquipSlot::ActiveOffhand, None, Time(0.0));
    inventory.replace_loadout_item(EquipSlot::ActiveMainhand, None, Time(0.0));
    inventory.replace_loadout_item(
        EquipSlot::ActiveMainhand,
        Some(Item::new_from_asset_expect(droite)),
        Time(0.0),
    );
    inventory.replace_loadout_item(
        EquipSlot::ActiveOffhand,
        gauche.as_deref().map(Item::new_from_asset_expect),
        Time(0.0),
    );
    inventory
}

/// Icebreaker : l'inventaire de départ d'une recrue de `class` qui porte le lot `acquis` : les armes
/// du premier jeu d'armes de son lot si la classe est décrite par nos données, sinon l'équipement de
/// la classe tel quel.
pub fn inventaire_du_lot(class: &Class, acquis: &Acquis) -> Inventory {
    match jeu_du_lot(&class.id, acquis) {
        Some(jeu) => inventaire_avec_jeu(class, jeu),
        None => starting_inventory(class),
    }
}

/// Icebreaker : le jeu d'armes d'un lot, pour une classe décrite par nos données. Un lot vide
/// (recrue d'avant le jalon A) donne le jeu par défaut de sa classe.
pub fn jeu_du_lot(classe: &str, acquis: &Acquis) -> Option<&'static JeuArmes> {
    let registre = registre();
    if !registre.classe_geree(classe) {
        return None;
    }
    let (id, _) = if acquis.est_vide() {
        registre.jeu_par_defaut(classe)?
    } else {
        let id = acquis.jeux.keys().next()?;
        (id.as_str(), registre.jeu(id)?)
    };
    registre.jeu(id)
}

/// Icebreaker : où en est un personnage avec ce qu'il tient en mains (jalon A.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EtatArme {
    /// Mains vides : rien à dire.
    Rien,
    /// Le jeu d'armes en mains est appris.
    Appris,
    /// Le jeu d'armes de sa classe, pas encore appris : son nom.
    NonAppris(String),
    /// Une arme en main droite qui n'est pas un jeu d'armes de sa classe.
    Etrangere,
}

impl EtatArme {
    /// Ce qu'on dit au joueur quand il entre dans cet état.
    pub fn message(&self) -> Option<String> {
        match self {
            Self::Rien | Self::Appris => None,
            Self::NonAppris(nom) => {
                Some(format!("{nom} : arme non apprise. Attaques de base seulement."))
            },
            Self::Etrangere => Some(
                "Cette arme n'est pas de votre classe. Attaques de base seulement.".to_string(),
            ),
        }
    }
}

/// Icebreaker : l'état d'un personnage qui a nos compétences, d'après ses armes en mains (A.7).
pub fn etat_arme(competences: &IcebreakerCompetences, inventaire: &Inventory) -> EtatArme {
    match jeu_en_main(&competences.classe, Some(inventaire)) {
        Some((id, _)) if competences.acquis.jeux.contains_key(id) => EtatArme::Appris,
        Some((_, jeu)) => EtatArme::NonAppris(jeu.nom.clone()),
        None if inventaire.equipped(EquipSlot::ActiveMainhand).is_some() => EtatArme::Etrangere,
        None => EtatArme::Rien,
    }
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
    use common::icebreaker_competences::registre;

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
            // K.4 : une pioche en mains secondaires, pour les filons de la grotte.
            assert_eq!(
                equipped(&inventory, EquipSlot::InactiveMainhand).as_deref(),
                Some("common.items.tool.pickaxe_stone"),
                "{id}"
            );
            // X.8 : deux icônes, l'arme d'abord.
            let icones = icones(&class);
            assert_eq!(icones.len(), 2, "{id}");
            assert_eq!(icones[0], ItemKey::Simple(weapon.to_string()), "{id}");
        }
        assert_eq!(classes().len(), 3);
    }

    // Jalon A.7 : le jeu d'armes d'un lot décide des armes en mains.
    #[test]
    fn icebreaker_classes_a_weapon_set_replaces_the_weapons_in_hand() {
        let class = class("guerrier").unwrap();
        for (id, jeu) in registre().jeux_de("guerrier") {
            let inventory = inventaire_avec_jeu(&class, jeu);
            assert_eq!(jeu_en_main("guerrier", Some(&inventory)).map(|(i, _)| i), Some(id), "{id}");
            // Le reste de l'équipement reste celui de la classe.
            assert!(equipped(&inventory, EquipSlot::Armor(ArmorSlot::Chest)).is_some(), "{id}");
            assert_eq!(
                equipped(&inventory, EquipSlot::InactiveMainhand).as_deref(),
                Some("common.items.tool.pickaxe_stone"),
                "{id}"
            );
        }
    }

    #[test]
    fn icebreaker_classes_the_message_follows_the_weapons_in_hand() {
        let class = class("guerrier").unwrap();
        let marteau = registre().jeu("marteau").unwrap();
        let inventory = inventaire_avec_jeu(&class, marteau);
        let mut competences =
            IcebreakerCompetences::nouveau("guerrier", Acquis::par_defaut("guerrier")).unwrap();
        // Le marteau n'est pas appris : le message nomme le jeu.
        let etat = etat_arme(&competences, &inventory);
        assert_eq!(etat, EtatArme::NonAppris(marteau.nom.clone()));
        assert_eq!(
            etat.message(),
            Some(format!("{} : arme non apprise. Attaques de base seulement.", marteau.nom))
        );
        // La grande épée, apprise : rien à dire.
        let epee = inventaire_du_lot(&class, &competences.acquis);
        assert_eq!(etat_arme(&competences, &epee), EtatArme::Appris);
        assert_eq!(EtatArme::Appris.message(), None);
        // Une arme d'une autre classe en main droite.
        let arc = LoadoutBuilder::empty()
            .active_mainhand(Some(common::comp::Item::new_from_asset_expect(
                "common.items.weapons.bow.starter",
            )))
            .build();
        let arc = Inventory::with_loadout_humanoid(arc);
        assert_eq!(etat_arme(&competences, &arc), EtatArme::Etrangere);
        assert!(EtatArme::Etrangere.message().unwrap().starts_with("Cette arme n'est pas de votre classe."));
        // Mains vides.
        let vide = Inventory::with_loadout_humanoid(LoadoutBuilder::empty().build());
        assert_eq!(etat_arme(&competences, &vide), EtatArme::Rien);
        assert_eq!(EtatArme::Rien.message(), None);
        // Le marteau appris : plus de message.
        competences.acquis.jeux.insert("marteau".into(), Default::default());
        assert_eq!(etat_arme(&competences, &inventory), EtatArme::Appris);
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
