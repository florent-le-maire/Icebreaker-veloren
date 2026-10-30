//! Icebreaker: encounter instances. A server started with
//! `ICEBREAKER_INSTANCE_SITE` set generates a small world around one chosen
//! site, without the wildlife and roaming monsters of a whole living world.

pub mod dungeon;

use crate::site::{Site, SiteKind, plot::PlotKind};
use tracing::warn;

const INSTANCE_SITE_VAR: &str = "ICEBREAKER_INSTANCE_SITE";
const TOWN_SIZE_VAR: &str = "ICEBREAKER_TOWN_SIZE";

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
    let kind = match name.as_str() {
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
        _ => {
            warn!(?name, "Unknown {INSTANCE_SITE_VAR}, no site placed");
            return None;
        },
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
    let mut chunks = HashMap::new();
    let problems = dungeon.check_blocks(|wpos| {
        let key = wpos.xy().map2(chunk_size, |e, s| e.div_euclid(s));
        let chunk = chunks.entry(key).or_insert_with(|| {
            world
                .generate_chunk(index, key, None, || false, None)
                .ok()
                .map(|(chunk, _)| chunk)
        });
        let chunk = chunk.as_ref()?;
        let local = wpos - (key * chunk_size).with_z(0);
        Some(chunk.get(local).copied().unwrap_or(common::terrain::Block::new(BlockKind::Air, Default::default())))
    });
    Some((dungeon.dungeon().plan.to_text(), problems))
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
