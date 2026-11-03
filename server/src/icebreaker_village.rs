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
    /// The signs around the door of each building (by `Building::id`): names of
    /// `SpriteKind`, either side of the door (X.4).
    #[serde(default)]
    pub enseignes: std::collections::HashMap<String, Vec<String>>,
}

pub fn config() -> VillageConfig {
    Ron::<VillageConfig>::load_expect("icebreaker.village")
        .read()
        .0
        .clone()
}

/// What is saved of the village: the ranks of its standing houses, and the
/// houses being built.
#[derive(Debug, Default, Deserialize, Serialize)]
struct SavedVillage {
    construites: Vec<usize>,
    #[serde(default)]
    chantiers: Vec<Chantier>,
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

/// A house being built on a plot (C.3): its rank, and when it is done (map
/// time, `TimeOfDay`).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Chantier {
    pub rang: usize,
    pub fin: f64,
}

/// The village as it stands (none outside the village).
#[derive(Default)]
pub struct IcebreakerVillage {
    /// The town's houses, nearest to the square first.
    houses: Vec<VillageHouse>,
    /// The ranks of the standing houses.
    standing: Vec<usize>,
    chantiers: Vec<Chantier>,
    path: Option<PathBuf>,
}

impl IcebreakerVillage {
    /// The village of the town of `site`, whose `houses` are ranked: tells the
    /// world which ones stand and how many people live there.
    pub fn new(site: &Site, houses: &[VillageHouse]) -> Self {
        let config = config();
        let path = village_file();
        let saved = path.as_ref().and_then(|path| std::fs::read_to_string(path).ok());
        let standing = standing_ranks(&config, saved.as_deref());
        let chantiers = saved
            .as_deref()
            .and_then(|text| ron::from_str::<SavedVillage>(text).ok())
            .map(|saved| saved.chantiers)
            .unwrap_or_default();
        let built = standing
            .iter()
            .filter_map(|rank| houses.get(*rank))
            .map(|house| house.plot)
            .collect::<HashSet<_>>();
        let inhabitants = built.len() as u32 * config.habitants_par_maison;
        let signs = Building::ALL
            .iter()
            .enumerate()
            .filter_map(|(i, building)| {
                let sprites = config.enseignes.get(building.id())?.clone();
                Some((houses.get(i)?.plot, sprites))
            })
            .collect::<Vec<_>>();
        world::icebreaker::set_village(site, &built, inhabitants, &signs);
        let village = Self {
            houses: houses.to_vec(),
            standing,
            chantiers,
            path,
        };
        info!(plots = ?village.open_plots(), chantiers = ?village.chantiers, "Icebreaker: plots to build on");
        village
    }

    /// The plots to build on now (chantiers included): (rank, bounds, altitude).
    pub fn open_plots(&self) -> Vec<(usize, vek::Aabr<i32>, i32)> {
        open_plots(&self.houses, &self.standing, config().terrains_ouverts)
            .into_iter()
            .map(|(rank, house)| (rank, house.bounds, house.alt))
            .collect()
    }

    /// The house being built on the plot of rank `rang`, if any.
    pub fn chantier(&self, rang: usize) -> Option<Chantier> {
        self.chantiers.iter().find(|c| c.rang == rang).copied()
    }

    /// Why a house cannot be built on the plot of rank `rang`, if it cannot.
    pub fn cannot_build(&self, rang: usize) -> Option<String> {
        if self.chantier(rang).is_some() {
            Some("Un chantier est déjà en cours sur ce terrain.".to_string())
        } else if !self.open_plots().iter().any(|(rank, ..)| *rank == rang) {
            Some("Ce terrain n'est pas à bâtir.".to_string())
        } else {
            None
        }
    }

    /// Starts building a house on the plot of rank `rang` at map time `now`
    /// (check `cannot_build` first); returns when it is done.
    pub fn build(&mut self, rang: usize, now: f64) -> f64 {
        let fin = now + config().heures_chantier * 3600.0;
        self.chantiers.push(Chantier { rang, fin });
        self.save();
        fin
    }

    /// The chantiers done at map time `now`: their houses now stand (seen at
    /// the next start of the server until C.4). Returns their ranks.
    pub fn finish_due(&mut self, now: f64) -> Vec<usize> {
        let (done, going): (Vec<_>, Vec<_>) = self.chantiers.iter().partition(|c| c.fin <= now);
        if done.is_empty() {
            return Vec::new();
        }
        self.chantiers = going;
        let ranks = done.iter().map(|c| c.rang).collect::<Vec<_>>();
        self.standing.extend(&ranks);
        self.standing.sort_unstable();
        self.save();
        ranks
    }

    fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let saved = SavedVillage {
            construites: self.standing.clone(),
            chantiers: self.chantiers.clone(),
        };
        match crate::icebreaker_warehouse::write_campaign_file(path, &saved) {
            Ok(()) => info!(?saved, "Icebreaker: village saved"),
            Err(err) => warn!(?err, ?path, "Icebreaker: could not save the village"),
        }
    }
}

/// `/batir <terrain>` (the « Bâtir » button of a plot's panel, C.3): starts
/// building a house on the plot of this rank, paid by the treasury.
pub(crate) fn handle_batir(
    server: &mut crate::Server,
    client: specs::Entity,
    _target: specs::Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), common::comp::Content> {
    use crate::icebreaker_treasury::IcebreakerTreasury;
    use common::comp::{self, Content};
    use specs::WorldExt;

    let Some(rang) = args.first().and_then(|arg| arg.parse::<usize>().ok()) else {
        return Err(Content::Plain("Quel terrain ? (/batir <numéro>)".to_string()));
    };
    let ecs = server.state.ecs();
    if let Some(why) = ecs.read_resource::<IcebreakerVillage>().cannot_build(rang) {
        return Err(Content::Plain(why));
    }
    let config = config();
    let price = u64::from(config.prix_maison);
    {
        let mut treasury = ecs.write_resource::<IcebreakerTreasury>();
        if !treasury.spend(price) {
            return Err(Content::Plain(format!(
                "La caisse n'a que {} pièces d'or ; une maison en coûte {price}.",
                treasury.coins()
            )));
        }
    }
    // The treasurer's name shows the treasury.
    crate::icebreaker_treasury::refresh_treasurer_name(ecs);
    let now = ecs.read_resource::<common::resources::TimeOfDay>().0;
    ecs.write_resource::<IcebreakerVillage>().build(rang, now);
    let alias = ecs
        .read_storage::<comp::Player>()
        .get(client)
        .map_or_else(|| "Quelqu'un".to_string(), |player| player.alias.clone());
    server.notify_players(common_net::msg::ServerGeneral::server_msg(
        comp::ChatType::Meta,
        Content::Plain(format!(
            "{alias} lance un chantier de maison au village ({price} or de la caisse) : fini dans \
             {} h de carte.",
            config.heures_chantier
        )),
    ));
    Ok(())
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

        // Saved: houses 0, 1 and 9; the buildings (0 to 5) always stand.
        let saved = standing_ranks(&config, Some("(construites: [9, 1, 0])"));
        assert_eq!(saved, vec![0, 1, 2, 3, 4, 5, 9]);
    }

    #[test]
    fn the_plots_offered_are_the_nearest_free_houses() {
        let house = |x| VillageHouse {
            plot: common::store::Id::new(x as u64),
            front: vek::Vec2::new(x, 0),
            inside: vek::Vec2::new(x, 1),
            alt: 0,
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

    #[test]
    fn a_house_is_built_on_a_free_plot_and_stands_once_done() {
        let house = |x| VillageHouse {
            plot: common::store::Id::new(x as u64),
            front: vek::Vec2::new(x, 0),
            inside: vek::Vec2::new(x, 1),
            alt: 0,
            bounds: vek::Aabr {
                min: vek::Vec2::new(x, 0),
                max: vek::Vec2::new(x + 5, 5),
            },
        };
        let mut village = IcebreakerVillage {
            houses: (0..20).map(house).collect(),
            standing: (0..7).collect(),
            chantiers: Vec::new(),
            path: None,
        };
        assert!(village.cannot_build(2).is_some(), "a standing house");
        assert!(village.cannot_build(19).is_some(), "too far, not offered yet");
        assert!(village.cannot_build(7).is_none());

        let fin = village.build(7, 1000.0);
        assert_eq!(fin, 1000.0 + config().heures_chantier * 3600.0);
        assert!(village.cannot_build(7).is_some(), "already being built");
        assert_eq!(village.finish_due(fin - 1.0), Vec::<usize>::new());
        assert_eq!(village.finish_due(fin), vec![7]);
        assert!(village.standing.contains(&7) && village.chantier(7).is_none());

        let saved = ron::from_str::<SavedVillage>(
            "(construites: [0, 1], chantiers: [(rang: 8, fin: 5.0)])",
        )
        .unwrap();
        assert_eq!(saved.chantiers, vec![Chantier { rang: 8, fin: 5.0 }]);
    }
}
