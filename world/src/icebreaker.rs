//! Icebreaker: encounter instances. A server started with
//! `ICEBREAKER_INSTANCE_SITE` set generates a small world around one chosen
//! site, without the wildlife and roaming monsters of a whole living world.

pub mod dungeon;

use crate::site::{
    Site, SiteKind,
    plot::{Plot, PlotKind},
};
use common::{assets::AssetExt, store::Id};
use std::{collections::HashSet, sync::RwLock};
use tracing::{info, warn};

const INSTANCE_SITE_VAR: &str = "ICEBREAKER_INSTANCE_SITE";
const TOWN_SIZE_VAR: &str = "ICEBREAKER_TOWN_SIZE";
const CAMPAIGN_VAR: &str = "ICEBREAKER_CAMPAGNE";

/// How far apart (in chunks) the village and the dungeon slots of the
/// campaign are placed, so that their zones and views stay apart.
pub const SLOT_DISTANCE: i32 = 32;

/// How many places are tried for the village's town.
pub const TOWN_PLACE_ATTEMPTS: usize = 12;
/// A town with this many plots is big enough: no other place is tried.
pub const TOWN_ENOUGH_PLOTS: usize = 60;

/// Whether this server runs an encounter instance.
pub fn is_instance() -> bool { std::env::var_os(INSTANCE_SITE_VAR).is_some() }

/// The site kind the instance is built around: a dungeon (e.g. `Haniwa`) or,
/// for the village, a town (e.g. `Town`).
pub fn instance_site() -> Option<SiteKind> {
    let name = std::env::var(INSTANCE_SITE_VAR).ok()?;
    let kind = site_kind(&name);
    if kind.is_none() {
        warn!(?name, "Unknown {INSTANCE_SITE_VAR}, no site placed");
    }
    kind
}

/// Whether this server runs the whole campaign (decision 0016): the village
/// and, far from it, slots of dungeons that expeditions go to.
pub fn is_campaign() -> bool { std::env::var_os(CAMPAIGN_VAR).is_some() && is_village() }

/// The dungeon slots of the campaign: their site names (as in
/// `icebreaker.ps1 -Site`), from `assets/icebreaker/campagne.ron`.
pub fn campaign_slot_names() -> Vec<String> {
    if !is_campaign() {
        return Vec::new();
    }
    common::assets::Ron::<CampaignConfig>::load_expect("icebreaker.campagne")
        .read()
        .0
        .emplacements
        .iter()
        .flat_map(|(site, count)| std::iter::repeat_n(site.clone(), *count))
        .collect()
}

#[derive(Clone, Debug, serde::Deserialize)]
struct CampaignConfig {
    /// Dungeon slots: site name, how many.
    emplacements: Vec<(String, usize)>,
}

/// The site kind of a name of `icebreaker.ps1 -Site`.
pub fn site_kind(name: &str) -> Option<SiteKind> {
    let kind = match name {
        "Town" => SiteKind::Refactor,
        "CliffTown" => SiteKind::CliffTown,
        "DesertCity" => SiteKind::DesertCity,
        "SavannahTown" => SiteKind::SavannahTown,
        "CoastalTown" => SiteKind::CoastalTown,
        "Gnarling" => SiteKind::Gnarling,
        "Adlet" => SiteKind::Adlet,
        "Haniwa" => SiteKind::Haniwa,
        "Terracotta" => SiteKind::Terracotta,
        "Myrmidon" => SiteKind::Myrmidon,
        "Sahagin" => SiteKind::Sahagin,
        "Cultist" => SiteKind::Cultist,
        "DwarvenMine" => SiteKind::DwarvenMine,
        "ChapelSite" => SiteKind::ChapelSite,
        "VampireCastle" => SiteKind::VampireCastle,
        "Genere" => SiteKind::IcebreakerDungeon,
        _ => return None,
    };
    Some(kind)
}

/// Where players arrive at a dungeon of our generator: out of its entrance
/// tunnel.
pub fn dungeon_entrance(site: &Site) -> Option<vek::Vec2<i32>> {
    site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.entrance()),
        _ => None,
    })
}

/// A house of the village, built or a plot to build on.
#[derive(Clone, Copy, Debug)]
pub struct VillageHouse {
    pub plot: Id<Plot>,
    /// In front of the door, on the road (world position).
    pub front: vek::Vec2<i32>,
    /// World bounds.
    pub bounds: vek::Aabr<i32>,
}

/// The houses of the village, nearest to `square` (world position) first: the
/// first ones hold its buildings, the next ones grow it. None if the town has
/// no `House` plots (other kinds of towns).
pub fn village_houses(site: &Site, square: vek::Vec2<i32>) -> Vec<VillageHouse> {
    let mut houses = site
        .plots
        .iter()
        .filter_map(|(id, plot)| match plot.kind() {
            PlotKind::House(house) => {
                let tiles = plot.find_bounds();
                // The door tile is the house's; its neighbour out of the house is
                // the road in front.
                let front = [
                    vek::Vec2::unit_x(),
                    -vek::Vec2::unit_x(),
                    vek::Vec2::unit_y(),
                    -vek::Vec2::unit_y(),
                ]
                .into_iter()
                .map(|dir| house.door_tile + dir)
                .find(|tile| !plot.tiles().any(|t| t == *tile))
                .unwrap_or(house.door_tile);
                Some(VillageHouse {
                    plot: id,
                    front: site.tile_center_wpos(front),
                    bounds: vek::Aabr {
                        min: site.tile_wpos(tiles.min),
                        max: site.tile_wpos(tiles.max + 1),
                    },
                })
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    houses.sort_by_key(|house| house.front.distance_squared(square));
    houses
}

/// The village as it stands: the plots of its town not built (yet), which are
/// not drawn, and how many people live there (for rtsim). Set by the server
/// once the world is generated, before any chunk is.
struct Village {
    hidden: HashSet<Id<Plot>>,
    inhabitants: u32,
}

static VILLAGE: RwLock<Option<Village>> = RwLock::new(None);

/// Sets what of the village's town stands: the `built` houses, its square and
/// roads, its fields; and how many people live there.
pub fn set_village(site: &Site, built: &HashSet<Id<Plot>>, inhabitants: u32) {
    let hidden = site
        .plots
        .iter()
        .filter(|(id, plot)| {
            !built.contains(id)
                && !matches!(
                    plot.kind(),
                    PlotKind::Plaza(_)
                        | PlotKind::Road(_)
                        | PlotKind::Bridge(_)
                        | PlotKind::FarmField(_)
                )
        })
        .map(|(id, _)| id)
        .collect::<HashSet<_>>();
    info!(
        built = built.len(),
        hidden = hidden.len(),
        inhabitants,
        "Icebreaker village"
    );
    *VILLAGE.write().unwrap_or_else(|e| e.into_inner()) = Some(Village {
        hidden,
        inhabitants,
    });
}

/// Whether this plot of the village is not built (yet).
pub(crate) fn is_hidden(plot: Id<Plot>) -> bool {
    VILLAGE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .is_some_and(|village| village.hidden.contains(&plot))
}

/// How many people live in the village, once set.
pub fn village_inhabitants() -> Option<u32> {
    VILLAGE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|village| village.inhabitants)
}

/// What the mission of a dungeon of our generator counts: its leader,
/// creatures and chests.
pub fn dungeon_mission(site: &Site) -> Option<crate::site::plot::MissionTargets> {
    site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.mission_targets()),
        _ => None,
    })
}

/// Generates the chunks of the dungeon of our generator in `world` and checks
/// its blocks against its plan (`examples/icebreaker_dungeon.rs`). `None` if
/// the world has no such dungeon; otherwise the plan and the problems found.
pub fn check_generated_dungeon(
    world: &crate::World,
    index: crate::IndexRef,
) -> Option<(String, Vec<String>)> {
    use common::terrain::{BlockKind, TerrainChunkSize};
    use common::vol::{ReadVol, RectVolSize};
    use std::collections::HashMap;

    let site = world
        .civs()
        .sites()
        .filter_map(|site| site.site_tmp)
        .map(|id| &index.sites[id])
        .find(|site| site.kind == Some(SiteKind::IcebreakerDungeon))?;
    let dungeon = site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon),
        _ => None,
    })?;
    let chunk_size = TerrainChunkSize::RECT_SIZE.map(|e| e as i32);
    let bounds = dungeon.bounds();
    let key_of = |wpos: vek::Vec2<i32>| wpos.map2(chunk_size, |e, s| e.div_euclid(s));
    let (low, high) = (key_of(bounds.min), key_of(bounds.max));
    let mut chunks = HashMap::new();
    let mut spawned = Vec::new();
    for x in low.x..=high.x {
        for y in low.y..=high.y {
            let key = vek::Vec2::new(x, y);
            let generated = world.generate_chunk(index, key, None, || false, None).ok();
            if let Some((_, supplement)) = &generated {
                spawned.extend(supplement
                    .entity_spawns
                    .iter()
                    .flat_map(|spawn| match spawn {
                        common::generation::EntitySpawn::Entity(e) => vec![e.pos],
                        common::generation::EntitySpawn::Group(g) => g.iter().map(|e| e.pos).collect(),
                    })
                    .filter(|pos| bounds.contains_point(pos.xy().as_())));
            }
            chunks.insert(key, generated.map(|(chunk, _)| chunk));
        }
    }
    let mut problems = dungeon.check_blocks(|wpos| {
        let key = key_of(wpos.xy());
        let chunk = chunks.get(&key)?.as_ref()?;
        let local = wpos - (key * chunk_size).with_z(0);
        Some(
            chunk
                .get(local)
                .copied()
                .unwrap_or(common::terrain::Block::new(BlockKind::Air, Default::default())),
        )
    });
    let plan = &dungeon.dungeon().plan;
    // One creature on each marker of the plan; others (from caves) are only counted.
    let mut others = spawned.clone();
    for (cell, symbol) in &plan.cells {
        if dungeon::populate::kind_of(*symbol).is_some() {
            let pos = dungeon.spawn_pos(*cell);
            let here = spawned.iter().filter(|p| **p == pos).count();
            if here != 1 {
                problems.push(format!("{here} créature(s) au lieu d'une sur '{symbol}' en {cell:?}"));
            }
            others.retain(|p| *p != pos);
        }
    }
    let report = format!(
        "{}\n{}\n{} créature(s) apparue(s) dans le donjon, {} autre(s) dans la zone",
        dungeon.dungeon().summary,
        plan.to_text(),
        spawned.len() - others.len(),
        others.len()
    );
    Some((report, problems))
}

/// Whether the instance is the village (built around a town).
pub fn is_village() -> bool { instance_site().is_some_and(is_town) }

/// The size of the village's town (`Town` only, 0.03 to 1.0), given by
/// `ICEBREAKER_TOWN_SIZE`, instead of Veloren's random size which mostly
/// makes hamlets.
pub fn village_town_size() -> Option<f32> {
    if !is_village() {
        return None;
    }
    let size = std::env::var(TOWN_SIZE_VAR).ok()?;
    match size.parse::<f32>() {
        Ok(size) => Some(size.clamp(0.03, 1.0)),
        Err(_) => {
            warn!(?size, "Invalid {TOWN_SIZE_VAR}, random town size");
            None
        },
    }
}

/// Whether `kind` is a town.
pub fn is_town(kind: SiteKind) -> bool {
    matches!(
        kind,
        SiteKind::Refactor
            | SiteKind::CliffTown
            | SiteKind::DesertCity
            | SiteKind::SavannahTown
            | SiteKind::CoastalTown
    )
}
