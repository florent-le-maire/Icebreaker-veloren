//! Icebreaker: the classes of the recruits of the cart (jalon R, decision
//! 0017), read from `assets/icebreaker/classes.ron`. A class is a name, an
//! icon and a base equipment (armour, pickaxe). Since jalon A (A.18) its
//! weapons, skills, ultimates and passives are described by our own data
//! (`assets/icebreaker/jeux_armes/`, `ultimes/`, `passifs/`) and carried by
//! the `IcebreakerCompetences` component; no Veloren skill set is involved.

use common::{
    assets::{AssetExt, Ron},
    comp::{
        Inventory, ItemKey,
        inventory::{loadout_builder::LoadoutBuilder, slot::EquipSlot},
    },
    icebreaker_competences::{Acquis, IcebreakerCompetences, JeuArmes, jeu_en_main, registre},
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Class {
    /// What the class is called in commands and in the registry file.
    pub id: String,
    /// What the players read.
    pub nom: String,
    /// Loadout asset: armour and pickaxe; the weapon in hand is only a
    /// fallback (the real one comes from the weapon set of the recruit's lot).
    pub equipement: String,
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
pub fn icones(class: &Class, acquis: &Acquis) -> Vec<ItemKey> {
    use common::comp::inventory::slot::{ArmorSlot, EquipSlot};
    // Icebreaker A.8 : l'arme est celle du jeu d'armes du lot de la recrue.
    let inventory = inventaire_du_lot(class, acquis);
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
    let mut inventory = starting_inventory(class);
    prendre_jeu(&mut inventory, jeu);
    inventory
}

/// Icebreaker : met en mains les armes du jeu d'armes `jeu`, à la place de celles qu'on tenait
/// (jalon O : le changement d'arme du lieu d'entraînement).
pub fn prendre_jeu(inventory: &mut Inventory, jeu: &JeuArmes) {
    use common::comp::{Item, inventory::slot::EquipSlot};
    use common::resources::Time;
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
            let des_icones = icones(&class, &Acquis::par_defaut(id));
            assert_eq!(des_icones.len(), 2, "{id}");
            assert_eq!(des_icones[0], ItemKey::Simple(weapon.to_string()), "{id}");
            // A.8 : l'icône d'arme suit le lot (le marteau, pas l'épée).
            if id == "guerrier" {
                let marteau = Acquis::lot("marteau", &[], None, None);
                let icones = icones(&class, &marteau);
                assert_eq!(
                    icones[0],
                    ItemKey::Simple("common.items.weapons.hammer.starter_hammer".to_string())
                );
            }
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

    // A.18 : plus de compétences de Veloren, chaque classe est décrite par nos données.
    #[test]
    fn icebreaker_classes_are_all_described_by_our_data() {
        for class in classes() {
            assert!(registre().classe_geree(&class.id), "{}", class.id);
            let inventory = inventaire_du_lot(&class, &Acquis::par_defaut(&class.id));
            assert!(
                jeu_en_main(&class.id, Some(&inventory)).is_some(),
                "{} : le lot par défaut ne donne pas d'arme reconnue",
                class.id
            );
        }
    }
}
