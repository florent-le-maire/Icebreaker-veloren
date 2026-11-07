//! Icebreaker B.5 and B.14: the loot of the generated dungeon and of the cave
//! by depth (`assets/icebreaker/butin.ron`, `docs/icebreaker/PLAN-BOUCLES.md`).
//! A place knows how deep each chest and creature lies; the settings turn that
//! depth into a tier and the tier into a Veloren loot table of useful things
//! only.

use serde::Deserialize;

/// `assets/icebreaker/butin.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct Butin {
    /// The loot table of a chest, by tier (the first one for tier 1).
    pub coffres: Vec<String>,
    /// What a creature drops, by tier (B.14).
    pub creatures: Vec<String>,
    /// What the chief and the boss drop, whatever their tier.
    pub chef: String,
    pub boss: String,
}

impl Butin {
    pub fn parse(text: &str) -> Result<Self, String> {
        let butin: Self = ron::from_str(text).map_err(|e| format!("butin.ron illisible : {e}"))?;
        if butin.coffres.is_empty() || butin.creatures.len() != butin.coffres.len() {
            return Err("butin.ron : au moins une table de coffre, autant de tables de créature".into());
        }
        Ok(butin)
    }

    pub fn load() -> Result<Self, String> {
        let path = common::assets::ASSETS_PATH.join("icebreaker/butin.ron");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))?;
        Self::parse(&text)
    }

    /// How many tiers there are.
    pub fn paliers(&self) -> usize { self.coffres.len() }

    /// The tier (1 to `paliers`) of something at `depth` among `depths` depths
    /// (0 = the entrance's, `depths - 1` = the deepest): equal parts.
    pub fn palier(&self, depth: usize, depths: usize) -> usize {
        1 + (depth.min(depths.saturating_sub(1)) * self.paliers()) / depths.max(1)
    }

    /// The loot table of a chest of `palier`.
    pub fn coffre(&self, palier: usize) -> &str { &self.coffres[palier.clamp(1, self.paliers()) - 1] }

    /// What a creature of `kind` (as in the dungeon's populate: "chief",
    /// "boss", or another) drops at `palier`.
    pub fn creature(&self, kind: &str, palier: usize) -> &str {
        match kind {
            "chief" => &self.chef,
            "boss" => &self.boss,
            _ => &self.creatures[palier.clamp(1, self.paliers()) - 1],
        }
    }

    /// Every table named here.
    pub fn tables(&self) -> impl Iterator<Item = &String> {
        self.coffres.iter().chain(&self.creatures).chain([&self.chef, &self.boss])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{
        assets::AssetExt,
        lottery::{LootSpec, Lottery},
    };

    /// Mean number of coins dropped by `table` over a few draws.
    fn coins(table: &str) -> f32 {
        let lottery = Lottery::<LootSpec<String>>::load_expect(table).read();
        let draws = 400;
        let total: u32 = (0..draws)
            .flat_map(|_| lottery.choose().to_items().unwrap_or_default())
            .filter(|(_, item)| item.item_definition_id().itemdef_id() == Some("common.items.utility.coins"))
            .map(|(n, item)| n * item.amount())
            .sum();
        total as f32 / draws as f32
    }

    #[test]
    fn tables_load_and_hold_only_useful_things() {
        let butin = Butin::load().unwrap();
        assert_eq!(butin.paliers(), 3);
        for table in butin.tables() {
            let path = common::assets::ASSETS_PATH.join(table.replace('.', "/") + ".ron");
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{table} : {e}"));
            for useless in ["recipes", "quest", "veloritefrag", "firework", "surprise_egg"] {
                assert!(!text.contains(useless), "{table} : {useless}");
            }
            Lottery::<LootSpec<String>>::load_expect(table);
        }
        for table in butin.coffres.iter().chain([&butin.chef, &butin.boss]) {
            let lottery = Lottery::<LootSpec<String>>::load_expect(table).read();
            for _ in 0..50 {
                assert!(!lottery.choose().to_items().unwrap_or_default().is_empty(), "{table} : vide");
            }
        }
    }

    /// Deeper is richer, and the chief and the boss drop more than a chest.
    #[test]
    fn deeper_is_richer() {
        let butin = Butin::load().unwrap();
        for tables in [&butin.coffres, &butin.creatures] {
            let means: Vec<f32> = tables.iter().map(|t| coins(t)).collect();
            assert!(means.windows(2).all(|w| w[0] < w[1]), "{tables:?} : {means:?}");
        }
        let deepest = coins(butin.coffre(3));
        assert!(coins(&butin.chef) > deepest && coins(&butin.boss) > coins(&butin.chef));
    }

    #[test]
    fn tiers_cut_the_depths_in_equal_parts() {
        let butin = Butin::load().unwrap();
        let tiers: Vec<usize> = (0..9).map(|d| butin.palier(d, 9)).collect();
        assert_eq!(tiers, [1, 1, 1, 2, 2, 2, 3, 3, 3]);
        assert_eq!((0..3).map(|d| butin.palier(d, 3)).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(butin.palier(0, 1), 1);
        assert_eq!(butin.palier(5, 2), 2);
        assert_eq!(butin.coffre(3), "common.loot_tables.icebreaker.butin.coffre_3");
        assert_eq!(butin.creature("brute", 2), "common.loot_tables.icebreaker.butin.creature_2");
        assert_eq!(butin.creature("chief", 1), butin.chef);
        assert_eq!(butin.creature("boss", 1), butin.boss);
    }
}
