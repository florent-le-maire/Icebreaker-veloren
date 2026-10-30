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

/// Tells the world which of the town's `houses` (ranked) stand, and how many
/// people live in the village.
pub fn set_village(site: &Site, houses: &[VillageHouse]) {
    let config = config();
    let saved = village_file().and_then(|path| std::fs::read_to_string(path).ok());
    let built = standing_ranks(&config, saved.as_deref())
        .into_iter()
        .filter_map(|rank| houses.get(rank))
        .map(|house| house.plot)
        .collect::<HashSet<_>>();
    let inhabitants = built.len() as u32 * config.habitants_par_maison;
    if let Some(plot) = houses.iter().find(|house| !built.contains(&house.plot)) {
        info!(bounds = ?plot.bounds, "Icebreaker: nearest plot to build on");
    }
    world::icebreaker::set_village(site, &built, inhabitants);
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
}
