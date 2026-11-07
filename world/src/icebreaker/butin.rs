//! Icebreaker B.5: the loot of the generated dungeon and of the cave by depth
//! (`assets/icebreaker/butin.ron`, `docs/icebreaker/PLAN-BOUCLES.md`). A place
//! knows how deep each chest lies; the settings turn that depth into a tier
//! and the tier into a Veloren loot table of useful things only.

use serde::Deserialize;

/// `assets/icebreaker/butin.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct Butin {
    /// The loot table of a chest, by tier (the first one for tier 1).
    pub coffres: Vec<String>,
}

impl Butin {
    pub fn parse(text: &str) -> Result<Self, String> {
        let butin: Self = ron::from_str(text).map_err(|e| format!("butin.ron illisible : {e}"))?;
        if butin.coffres.is_empty() {
            return Err("butin.ron : au moins une table de coffre".into());
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
    pub fn coffre(&self, palier: usize) -> &str {
        &self.coffres[palier.clamp(1, self.paliers()) - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{
        assets::AssetExt,
        lottery::{LootSpec, Lottery},
    };

    #[test]
    fn tables_load_and_hold_only_useful_things() {
        let butin = Butin::load().unwrap();
        assert_eq!(butin.paliers(), 3);
        for table in &butin.coffres {
            let path = common::assets::ASSETS_PATH.join(table.replace('.', "/") + ".ron");
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{table} : {e}"));
            for useless in ["recipes", "quest", "veloritefrag", "firework", "surprise_egg"] {
                assert!(!text.contains(useless), "{table} : {useless}");
            }
            let lottery = Lottery::<LootSpec<String>>::load_expect(table).read();
            for _ in 0..50 {
                let items = lottery.choose().to_items().unwrap_or_default();
                assert!(!items.is_empty(), "{table} : coffre vide");
            }
        }
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
    }
}
