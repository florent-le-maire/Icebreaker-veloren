//! Icebreaker: the village that grows (jalon C). The whole town is generated,
//! but only some of its houses stand: the others are plots to build on. Its
//! houses are ranked from the nearest to the main square; the first ones hold
//! the village buildings and always stand. Which ones stand is kept in the
//! campaign directory; the start and the inhabitants per house are in
//! `assets/icebreaker/village.ron`.

use crate::icebreaker_buildings::Building;
use common::assets::{AssetExt, Ron};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use tracing::{info, warn};
use world::{icebreaker::VillageHouse, site::Site};

/// File of the campaign directory holding what of the village stands.
const VILLAGE_FILE: &str = "village.ron";

#[derive(Clone, Debug, Deserialize)]
pub struct VillageConfig {
    /// Houses standing at the start of the campaign (buildings included).
    pub maisons_au_depart: usize,
    /// Inhabitants per standing house.
    pub habitants_par_maison: u32,
    /// Plots to build on offered at a time, nearest to the square first.
    pub terrains_ouverts: usize,
    /// What a house costs, in coins of the treasury.
    pub prix_maison: u32,
    /// How long a house takes to build, in hours of map time.
    pub heures_chantier: f64,
}

pub fn config() -> VillageConfig {
    Ron::<VillageConfig>::load_expect("icebreaker.village")
        .read()
        .0
        .clone()
}

/// What is saved of the village: the ranks of its standing houses.
#[derive(Debug, Default, Deserialize, Serialize)]
struct SavedVillage {
    construites: Vec<usize>,
}

fn village_file() -> Option<PathBuf> {
    std::env::var_os("ICEBREAKER_CAMPAIGN_DIR").map(|dir| PathBuf::from(dir).join(VILLAGE_FILE))
}

/// The ranks of the standing houses: those saved in the campaign, else the
/// first ones; the buildings' always.
fn standing_ranks(config: &VillageConfig, saved: Option<&str>) -> Vec<usize> {
    let saved = saved.and_then(|text| {
        ron::from_str::<SavedVillage>(text)
            .inspect_err(|err| warn!(?err, "Icebreaker: unreadable village, as at the start"))
            .ok()
    });
    let mut ranks = saved.map_or_else(
        || (0..config.maisons_au_depart).collect(),
        |saved| saved.construites,
    );
    ranks.extend(0..Building::ALL.len());
    ranks.sort_unstable();
    ranks.dedup();
    ranks
}

/// The first `count` plots to build on (rank, house): the houses not standing,
/// nearest to the square first.
fn open_plots(
    houses: &[VillageHouse],
    standing: &[usize],
    count: usize,
) -> Vec<(usize, VillageHouse)> {
    houses
        .iter()
        .copied()
        .enumerate()
        .filter(|(rank, _)| !standing.contains(rank))
        .take(count)
        .collect()
}

/// The village as it stands (none outside the village).
#[derive(Default)]
pub struct IcebreakerVillage {
    /// The town's houses, nearest to the square first.
    houses: Vec<VillageHouse>,
    /// The ranks of the standing houses.
    standing: Vec<usize>,
}

impl IcebreakerVillage {
    /// The village of the town of `site`, whose `houses` are ranked: tells the
    /// world which ones stand and how many people live there.
    pub fn new(site: &Site, houses: &[VillageHouse]) -> Self {
        let config = config();
        let saved = village_file().and_then(|path| std::fs::read_to_string(path).ok());
        let standing = standing_ranks(&config, saved.as_deref());
        let built = standing
            .iter()
            .filter_map(|rank| houses.get(*rank))
            .map(|house| house.plot)
            .collect::<HashSet<_>>();
        let inhabitants = built.len() as u32 * config.habitants_par_maison;
        world::icebreaker::set_village(site, &built, inhabitants);
        let village = Self {
            houses: houses.to_vec(),
            standing,
        };
        info!(plots = ?village.open_plots(), "Icebreaker: plots to build on");
        village
    }

    /// The plots to build on now: (rank, bounds).
    pub fn open_plots(&self) -> Vec<(usize, vek::Aabr<i32>)> {
        open_plots(&self.houses, &self.standing, config().terrains_ouverts)
            .into_iter()
            .map(|(rank, house)| (rank, house.bounds))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_village_starts_small_and_keeps_its_buildings() {
        let config = config();
        let start = standing_ranks(&config, None);
        assert_eq!(start, (0..config.maisons_au_depart).collect::<Vec<_>>());
        assert!(start.len() >= Building::ALL.len());

        // Saved: houses 0, 1 and 9; the buildings (0 to 3) always stand.
        let saved = standing_ranks(&config, Some("(construites: [9, 1, 0])"));
        assert_eq!(saved, vec![0, 1, 2, 3, 9]);
    }

    #[test]
    fn the_plots_offered_are_the_nearest_free_houses() {
        let house = |x| VillageHouse {
            plot: common::store::Id::new(x as u64),
            front: vek::Vec2::new(x, 0),
            bounds: vek::Aabr {
                min: vek::Vec2::new(x, 0),
                max: vek::Vec2::new(x + 5, 5),
            },
        };
        let houses = (0..10).map(house).collect::<Vec<_>>();
        let ranks = open_plots(&houses, &[0, 1, 2, 3, 5], 3)
            .into_iter()
            .map(|(rank, _)| rank)
            .collect::<Vec<_>>();
        assert_eq!(ranks, vec![4, 6, 7]);
    }
}
