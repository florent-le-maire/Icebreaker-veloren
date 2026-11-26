//! Icebreaker H: the enemies of a race (`assets/icebreaker/ennemis/`). A
//! dungeon or a cave is peopled by one race: its species, drawn from the seed
//! of the place, replaces the `humanoid` body of the enemy files; any other
//! creature (the lone monsters, the factions of Veloren) keeps its own body.

use common::{
    generation::{BodyBuilder, EntityConfig, EntityInfo},
    icebreaker_difficulte::Niveau,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use vek::Vec3;

/// The body of the enemy files: it takes the species of the place.
const HUMANOID: &str = "humanoid";

/// The species of the place of `seed`, among `species` (none if empty). The
/// same seed always gives the same species.
pub fn species_of(species: &[String], seed: u64) -> Option<&String> {
    if species.is_empty() {
        return None;
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(13));
    species.get(rng.random_range(0..species.len()))
}

/// The entity of `asset` at `pos`; a humanoid of `species` if the file says
/// `RandomWith("humanoid")` and a species is given (its name is drawn after
/// the body, so it fits the species). Icebreaker DF: its health and its damage follow the difficulty of the
/// place.
pub fn entity(asset: &str, species: Option<&str>, pos: Vec3<f32>, rng: &mut ChaCha8Rng, difficulte: &Niveau) -> EntityInfo {
    let mut config = EntityConfig::from_asset_expect_owned(asset);
    if let (Some(species), BodyBuilder::RandomWith(body)) = (species, &config.body)
        && body == HUMANOID
    {
        config = config.with_body(BodyBuilder::RandomWith(species.to_string()));
    }
    let mut info = EntityInfo::at(pos).with_entity_config(config, Some(asset), rng, None);
    // Only the health of our files (`icebreaker_pv`): a creature of Veloren keeps the one of its body.
    info.icebreaker_pv = info.icebreaker_pv.map(|pv| pv * difficulte.pv);
    info.icebreaker_degats = Some(difficulte.degats_de(asset));
    // Icebreaker IA: its role and its skill tier, for its brain (`ia/profils.ron`).
    info.icebreaker_ia = Some((common::icebreaker_difficulte::role(asset).to_string(), difficulte.ia.clone()));
    // Icebreaker DF: the skills of the rank of its difficulty, in place of those of its file.
    if let Some(competences) = difficulte.competences(asset) {
        info.skillset_asset = Some(competences.to_string());
    }
    info
}

/// Icebreaker O : un ennemi à la demande (lieu d'entraînement), tiré au hasard.
pub fn entity_au_hasard(asset: &str, species: Option<&str>, pos: Vec3<f32>, difficulte: &Niveau) -> EntityInfo {
    entity(asset, species, pos, &mut ChaCha8Rng::from_seed(rand::rng().random()), difficulte)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::comp::{Body, humanoid};

    const ENEMIES: [&str; 4] = [
        "icebreaker.ennemis.guerrier",
        "icebreaker.ennemis.archer",
        "icebreaker.ennemis.mage",
        "icebreaker.ennemis.chef",
    ];
    const SPECIES: [(&str, humanoid::Species); 6] = [
        ("human", humanoid::Species::Human),
        ("orc", humanoid::Species::Orc),
        ("dwarf", humanoid::Species::Dwarf),
        ("elf", humanoid::Species::Elf),
        ("draugr", humanoid::Species::Draugr),
        ("danari", humanoid::Species::Danari),
    ];

    #[test]
    fn enemies_take_the_species_of_the_place() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        for asset in ENEMIES {
            for (name, species) in SPECIES {
                let info = entity(asset, Some(name), Vec3::zero(), &mut rng, &Niveau::default());
                assert!(
                    matches!(info.body, Body::Humanoid(b) if b.species == species),
                    "{asset} en {name} : {:?}",
                    info.body
                );
            }
        }
        // The chief holds the staff of the gnarling chief (its abilities).
        let chief = entity("icebreaker.ennemis.chef", Some("orc"), Vec3::zero(), &mut rng, &Niveau::default());
        let inventory = common::comp::Inventory::with_loadout(chief.loadout.build(), chief.body);
        assert!(
            inventory
                .equipped(common::comp::inventory::slot::EquipSlot::ActiveMainhand)
                .is_some_and(|i| i.item_definition_id().itemdef_id() == Some("common.items.npc_weapons.biped_small.gnarling.chieftain")),
            "le chef n'a pas le bâton du chef gnarling"
        );
    }

    // Icebreaker H.6: the health of the file, no staying at 1 HP.
    #[test]
    fn enemies_have_the_health_of_their_file() {
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        for (asset, pv) in [("icebreaker.ennemis.guerrier", 50.0), ("icebreaker.ennemis.chef", 150.0)] {
            let info = entity(asset, Some("elf"), Vec3::zero(), &mut rng, &Niveau::default());
            assert_eq!(info.icebreaker_pv, Some(pv), "{asset}");
            let mut health = common::comp::Health::new(info.body);
            health.icebreaker_ennemi(info.icebreaker_pv);
            assert_eq!((health.maximum(), health.current()), (pv, pv));
            assert!(!health.death_protection && !health.can_have_death_protection);
        }
    }

    #[test]
    fn other_creatures_keep_their_body() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let spider = entity("common.entity.wild.aggressive.cave_spider", Some("orc"), Vec3::zero(), &mut rng, &Niveau::default());
        assert!(!matches!(spider.body, Body::Humanoid(_)));
    }

    // Icebreaker DF: the health and the damage of the difficulty of the place.
    #[test]
    fn enemies_follow_the_difficulty() {
        let difficultes = common::icebreaker_difficulte::IcebreakerDifficulte::actuel();
        let mut rng = ChaCha8Rng::seed_from_u64(4);
        for (n, pv, degats) in [(1, 30.0, 0.35), (3, 50.0, 1.0), (6, 80.0, 1.45)] {
            let niveau = difficultes.niveau(Some(n));
            let info = entity("icebreaker.ennemis.guerrier", Some("orc"), Vec3::zero(), &mut rng, &niveau);
            assert!((info.icebreaker_pv.unwrap() - pv).abs() < 0.01, "difficulté {n} : {:?} PV", info.icebreaker_pv);
            assert_eq!(info.icebreaker_degats, Some(degats), "difficulté {n}");
            // Icebreaker IA: its brain gets its role and its skill tier.
            assert_eq!(info.icebreaker_ia, Some(("guerrier".to_string(), niveau.ia.clone())), "difficulté {n}");
            // The factor stays when the buffs reset the stats, each tick.
            let mut stats = common::comp::Stats::new(common::comp::Content::dummy(), info.body);
            stats.icebreaker_regler_degats(degats);
            stats.attack_damage_modifier *= 2.0;
            stats.reset_temp_modifiers();
            assert_eq!(stats.attack_damage_modifier, degats);
        }
        // The skills of the rank: those of the file up to 3, rank 3 at 4 and 5, rank 5 at 6; the chief keeps his.
        for (n, rang) in [(1, "rank1"), (3, "rank1"), (4, "rank3"), (5, "rank3"), (6, "rank5")] {
            let niveau = difficultes.niveau(Some(n));
            for (role, arme) in [("guerrier", "sword"), ("archer", "bow"), ("mage", "staff")] {
                let info = entity(&format!("icebreaker.ennemis.{role}"), Some("elf"), Vec3::zero(), &mut rng, &niveau);
                assert_eq!(info.skillset_asset, Some(format!("common.skillset.preset.{rang}.{arme}")), "{role} en {n}");
            }
            let chef = entity("icebreaker.ennemis.chef", Some("elf"), Vec3::zero(), &mut rng, &niveau);
            assert_eq!(chef.skillset_asset.as_deref(), Some("common.skillset.preset.rank5.fullskill"));
        }
        // A creature of Veloren keeps the health of its body.
        let spider = entity("common.entity.wild.aggressive.cave_spider", None, Vec3::zero(), &mut rng, &difficultes.niveau(Some(6)));
        assert_eq!((spider.icebreaker_pv, spider.icebreaker_degats), (None, Some(1.45)));
        // Icebreaker DF.9: at difficulty 1 the mage, who already hits less, is lowered less than the others.
        let mage = entity("icebreaker.ennemis.mage", Some("orc"), Vec3::zero(), &mut rng, &difficultes.niveau(Some(1)));
        assert_eq!(mage.icebreaker_degats, Some(0.6));
    }

    #[test]
    fn one_species_per_seed() {
        let species: Vec<String> = SPECIES.iter().map(|(n, _)| n.to_string()).collect();
        let mut seen = std::collections::BTreeSet::new();
        for seed in 0..60 {
            let a = species_of(&species, seed).unwrap();
            assert_eq!(Some(a), species_of(&species, seed));
            seen.insert(a.clone());
        }
        assert_eq!(seen.len(), species.len(), "toutes les races finissent par sortir");
        assert!(species_of(&[], 3).is_none());
    }
}
