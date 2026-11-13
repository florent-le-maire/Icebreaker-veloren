//! Icebreaker H: the enemies of a race (`assets/icebreaker/ennemis/`). A
//! dungeon or a cave is peopled by one race: its species, drawn from the seed
//! of the place, replaces the `humanoid` body of the enemy files; any other
//! creature (the lone monsters, the factions of Veloren) keeps its own body.

use common::generation::{BodyBuilder, EntityConfig, EntityInfo};
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
/// the body, so it fits the species).
pub fn entity(asset: &str, species: Option<&str>, pos: Vec3<f32>, rng: &mut ChaCha8Rng) -> EntityInfo {
    let mut config = EntityConfig::from_asset_expect_owned(asset);
    if let (Some(species), BodyBuilder::RandomWith(body)) = (species, &config.body)
        && body == HUMANOID
    {
        config = config.with_body(BodyBuilder::RandomWith(species.to_string()));
    }
    EntityInfo::at(pos).with_entity_config(config, Some(asset), rng, None)
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
                let info = entity(asset, Some(name), Vec3::zero(), &mut rng);
                assert!(
                    matches!(info.body, Body::Humanoid(b) if b.species == species),
                    "{asset} en {name} : {:?}",
                    info.body
                );
            }
        }
        // The chief holds the staff of the gnarling chief (its abilities).
        let chief = entity("icebreaker.ennemis.chef", Some("orc"), Vec3::zero(), &mut rng);
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
            let info = entity(asset, Some("elf"), Vec3::zero(), &mut rng);
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
        let spider = entity("common.entity.wild.aggressive.cave_spider", Some("orc"), Vec3::zero(), &mut rng);
        assert!(!matches!(spider.body, Body::Humanoid(_)));
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
