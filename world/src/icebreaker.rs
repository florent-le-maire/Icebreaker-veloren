//! Icebreaker: encounter instances. A server started with
//! `ICEBREAKER_INSTANCE_SITE` set generates a small world around one chosen
//! site, without the wildlife and roaming monsters of a whole living world.

// Icebreaker O : le plateau du lieu d'entraînement.
pub mod arene;
pub mod butin;
pub mod dungeon;
pub mod grotte;
pub mod races;

use crate::site::{
    Site, SiteKind,
    plot::{Plot, PlotKind},
};
use common::{assets::AssetExt, store::Id, terrain::SpriteKind};
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};
use strum::IntoEnumIterator;
use tracing::{info, warn};

const INSTANCE_SITE_VAR: &str = "ICEBREAKER_INSTANCE_SITE";
const TOWN_SIZE_VAR: &str = "ICEBREAKER_TOWN_SIZE";
const CAMPAIGN_VAR: &str = "ICEBREAKER_CAMPAGNE";
/// Icebreaker O : le nom de site du lieu d'entraînement, un monde sans site (`arene`).
const ENTRAINEMENT: &str = "Entrainement";

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
    if kind.is_none() && name != ENTRAINEMENT {
        warn!(?name, "Unknown {INSTANCE_SITE_VAR}, no site placed");
    }
    kind
}

/// Icebreaker O : ce serveur est le lieu d'entraînement (`icebreaker.ps1 entrainement`) : aucun
/// site, un plateau nivelé (`arene`).
pub fn is_entrainement() -> bool {
    std::env::var(INSTANCE_SITE_VAR).is_ok_and(|name| name == ENTRAINEMENT)
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
        "Grotte" => SiteKind::IcebreakerGrotte,
        _ => return None,
    };
    Some(kind)
}

/// Icebreaker F.8: where a site of `kind` may be placed in `world_dims` (in
/// chunks). A dungeon of our generator spreads far around its entrance: the
/// entrance keeps `entrance_margin` chunks from the edges of the world, past
/// which the dungeon could not be finished.
pub fn site_area(kind: SiteKind, world_dims: vek::Aabr<i32>) -> vek::Aabr<i32> {
    if !matches!(kind, SiteKind::IcebreakerDungeon) {
        return world_dims;
    }
    let margin = match dungeon::Settings::load() {
        Ok(settings) => settings.placement.entrance_margin,
        Err(e) => {
            warn!(?e, "Icebreaker: dungeon settings unreadable, no margin from the edges");
            0
        },
    };
    let area = vek::Aabr {
        min: world_dims.min + margin,
        max: world_dims.max - margin,
    };
    if area.min.x < area.max.x && area.min.y < area.max.y {
        area
    } else {
        warn!(margin, ?world_dims, "Icebreaker: world too small for the dungeon's margin");
        world_dims
    }
}

/// Where players arrive at a dungeon of our generator: out of its entrance
/// tunnel.
pub fn dungeon_entrance(site: &Site) -> Option<vek::Vec2<i32>> {
    site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.get().entrance()),
        PlotKind::IcebreakerGrotte(grotte) => Some(grotte.get().entrance()),
        _ => None,
    })
}

/// A house of the village, built or a plot to build on.
#[derive(Clone, Copy, Debug)]
pub struct VillageHouse {
    pub plot: Id<Plot>,
    /// In front of the door, on the road (world position).
    pub front: vek::Vec2<i32>,
    /// Just inside the door (the door tile's centre, world position), where the
    /// building's NPC stands (X.5).
    pub inside: vek::Vec2<i32>,
    /// World bounds.
    pub bounds: vek::Aabr<i32>,
    /// Ground altitude at the door (the client projects the house at that
    /// height in the top view, X.2).
    pub alt: i32,
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
                    inside: site.tile_center_wpos(house.door_tile),
                    bounds: vek::Aabr {
                        min: site.tile_wpos(tiles.min),
                        max: site.tile_wpos(tiles.max + 1),
                    },
                    alt: house.alt,
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
    /// The signs of each building, by the door tile of its house (X.4).
    enseignes: HashMap<vek::Vec2<i32>, Vec<SpriteKind>>,
}

static VILLAGE: RwLock<Option<Village>> = RwLock::new(None);

/// Sets what of the village's town stands: the `built` houses, its square and
/// roads, its fields; how many people live there; and the signs (names of
/// `SpriteKind`) around the door of each building's house (X.4).
pub fn set_village(
    site: &Site,
    built: &HashSet<Id<Plot>>,
    inhabitants: u32,
    signs: &[(Id<Plot>, Vec<String>)],
) {
    let enseignes = signs
        .iter()
        .filter_map(|(id, names)| {
            let PlotKind::House(house) = site.plot(*id).kind() else {
                return None;
            };
            let sprites = names
                .iter()
                .filter_map(|name| {
                    let sprite = SpriteKind::iter().find(|kind| format!("{kind:?}") == *name);
                    if sprite.is_none() {
                        warn!(name, "Icebreaker: unknown sign sprite");
                    }
                    sprite
                })
                .collect();
            Some((house.door_tile, sprites))
        })
        .collect();
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
        enseignes,
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

/// The signs around the door of the house whose door tile is `door_tile`, if
/// it is a building of the village (X.4).
pub(crate) fn enseignes(door_tile: vek::Vec2<i32>) -> Vec<SpriteKind> {
    VILLAGE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|village| village.enseignes.get(&door_tile).cloned())
        .unwrap_or_default()
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
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.get().mission_targets()),
        PlotKind::IcebreakerGrotte(grotte) => Some(grotte.get().mission_targets()),
        _ => None,
    })
}

/// Icebreaker I.11: draws a new dungeon (of our generator) or cave of `seed`
/// in place of the one of `site`, within the area kept for it; the chunks
/// generated from now on draw it. `Ok(false)`: the site has none, or none fit
/// (the current one stays).
pub fn regenerate_slot(world: &crate::World, site: &Site, seed: u64) -> Result<bool, String> {
    let land = crate::Land::from_sim(world.sim());
    for plot in site.plots() {
        let replaced = match plot.kind() {
            PlotKind::IcebreakerDungeon(dungeon) => dungeon.regenerate(&land, site.origin, seed)?,
            PlotKind::IcebreakerGrotte(grotte) => grotte.regenerate(&land, site.origin, seed)?,
            _ => continue,
        };
        if !replaced {
            warn!(origin = ?site.origin, seed, "Icebreaker: no new layout fits, the slot keeps its own");
        }
        return Ok(replaced);
    }
    Ok(false)
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
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.get()),
        _ => None,
    })?;
    let chunk_size = TerrainChunkSize::RECT_SIZE.map(|e| e as i32);
    let bounds = dungeon.bounds();
    let key_of = |wpos: vek::Vec2<i32>| wpos.map2(chunk_size, |e, s| e.div_euclid(s));
    let (low, high) = (key_of(bounds.min), key_of(bounds.max));
    let mut chunks = HashMap::new();
    let mut spawned = Vec::new();
    // Icebreaker B.14: what each of them drops.
    let mut loots = Vec::new();
    for x in low.x..=high.x {
        for y in low.y..=high.y {
            let key = vek::Vec2::new(x, y);
            let generated = world.generate_chunk(index, key, None, || false, None).ok();
            if let Some((_, supplement)) = &generated {
                for spawn in &supplement.entity_spawns {
                    if let common::generation::EntitySpawn::Entity(e) = spawn {
                        loots.push((e.pos, e.loot.clone()));
                    }
                }
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
        if let Some(kind) = dungeon::populate::kind_of(*symbol) {
            let pos = dungeon.spawn_pos(*cell);
            let here = spawned.iter().filter(|p| **p == pos).count();
            if here != 1 {
                problems.push(format!("{here} créature(s) au lieu d'une sur '{symbol}' en {cell:?}"));
            }
            let d = dungeon.dungeon();
            let expected = common::lottery::LootSpec::LootTable(
                d.butin.creature(kind, d.paliers.get(cell).copied().unwrap_or(1)).to_string(),
            );
            if loots.iter().any(|(p, loot)| *p == pos && *loot != expected) {
                problems.push(format!("créature '{symbol}' en {cell:?} : butin autre que {expected:?}"));
            }
            others.retain(|p| *p != pos);
        }
        // Icebreaker B.5: each chest holds the table of its tier.
        if *symbol == '$' {
            let pos = dungeon.spawn_pos(*cell).map(|e| e.floor() as i32);
            let key = key_of(pos.xy());
            let table = chunks
                .get(&key)
                .and_then(|c| c.as_ref())
                .and_then(|c| c.meta().sprite_cfg_at(pos - (key * chunk_size).with_z(0)))
                .and_then(|cfg| cfg.loot_table.clone());
            let d = dungeon.dungeon();
            let expected = d.butin.coffre(d.paliers.get(cell).copied().unwrap_or(1)).to_string();
            if table.as_ref() != Some(&expected) {
                problems.push(format!("coffre en {cell:?} : butin {table:?} au lieu de {expected}"));
            }
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


/// Generates the chunks of the cave in `world` and checks its blocks
/// (`examples/icebreaker_grotte.rs`): walking from the mouth (steps of one
/// block up, any fall down), every pocket is reached (Icebreaker K.2). `None`
/// if the world has no cave; otherwise a summary and the problems found.
pub fn check_generated_grotte(
    world: &crate::World,
    index: crate::IndexRef,
) -> Option<(String, Vec<String>)> {
    use common::terrain::TerrainChunkSize;
    use common::vol::{ReadVol, RectVolSize};
    use std::collections::{HashMap, HashSet, VecDeque};

    let site = world
        .civs()
        .sites()
        .filter_map(|site| site.site_tmp)
        .map(|id| &index.sites[id])
        .find(|site| site.kind == Some(SiteKind::IcebreakerGrotte))?;
    let plot = site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerGrotte(grotte) => Some(grotte.get()),
        _ => None,
    })?;
    let chunk_size = TerrainChunkSize::RECT_SIZE.map(|e| e as i32);
    let bounds = plot.bounds();
    let key_of = |wpos: vek::Vec2<i32>| wpos.map2(chunk_size, |e, s| e.div_euclid(s));
    let (low, high) = (key_of(bounds.min), key_of(bounds.max));
    let mut chunks = HashMap::new();
    let mut spawned: Vec<vek::Vec3<f32>> = Vec::new();
    // Icebreaker B.14: what each of them drops.
    let mut loots = Vec::new();
    for x in low.x..=high.x {
        for y in low.y..=high.y {
            let key = vek::Vec2::new(x, y);
            let generated = world.generate_chunk(index, key, None, || false, None).ok();
            if let Some((_, supplement)) = &generated {
                for spawn in &supplement.entity_spawns {
                    if let common::generation::EntitySpawn::Entity(e) = spawn {
                        loots.push((e.pos.map(|c| c.floor() as i32), e.loot.clone()));
                    }
                }
                spawned.extend(supplement.entity_spawns.iter().flat_map(|spawn| match spawn {
                    common::generation::EntitySpawn::Entity(e) => vec![e.pos],
                    common::generation::EntitySpawn::Group(g) => g.iter().map(|e| e.pos).collect(),
                }));
            }
            chunks.insert(key, generated.map(|(chunk, _)| chunk));
        }
    }
    let hollow = |wpos: vek::Vec3<i32>| {
        let key = key_of(wpos.xy());
        let local = wpos - (key * chunk_size).with_z(0);
        chunks
            .get(&key)
            .and_then(|c| c.as_ref())
            .and_then(|c| c.get(local).ok())
            .is_some_and(|b| !b.is_solid())
    };
    let block_at = |wpos: vek::Vec3<i32>| {
        let key = key_of(wpos.xy());
        let local = wpos - (key * chunk_size).with_z(0);
        chunks
            .get(&key)
            .and_then(|c| c.as_ref())
            .and_then(|c| c.get(local).ok())
            .copied()
    };
    let grotte = plot.grotte();
    let volume = grotte.bounds();
    // Standing at p: solid under, two blocks of air.
    let standing = |p: vek::Vec3<i32>| !hollow(p - vek::Vec3::unit_z()) && hollow(p) && hollow(p + vek::Vec3::unit_z());
    // From the mouth, down to the ground.
    let mut start = grotte.mouth + vek::Vec3::unit_z() * 3;
    while hollow(start - vek::Vec3::unit_z()) && start.z > volume.min.z {
        start.z -= 1;
    }
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let column = p.xy() + vek::Vec2::new(dx, dy);
            if !volume.contains_point(column.with_z(p.z)) {
                continue;
            }
            // One step up, or down as far as it falls, through air only.
            let mut z = p.z + 1;
            if !(hollow(column.with_z(z)) && hollow(column.with_z(z + 1))) {
                continue;
            }
            while z > volume.min.z && hollow(column.with_z(z - 1)) {
                z -= 1;
            }
            let next = column.with_z(z);
            if standing(next) && seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    let mut problems = Vec::new();
    for (i, pocket) in grotte.pockets.iter().enumerate() {
        let aabb = pocket.aabb();
        if !seen.iter().any(|p| aabb.contains_point(*p)) {
            problems.push(format!("poche {i} (niveau {}) inaccessible depuis la bouche : {:?}", pocket.level, pocket.center));
        }
    }
    // Along each bore, the floor under its axis never steps more than one
    // block (one walks it both ways).
    for (i, bore) in grotte.bores.iter().enumerate() {
        let n = ((bore.b - bore.a).as_::<f32>().magnitude().ceil() as i32).max(1);
        let mut last: Option<(i32, vek::Vec3<i32>)> = None;
        for k in 0..=n {
            let p = (bore.a.as_::<f32>() + (bore.b - bore.a).as_::<f32>() * (k as f32 / n as f32))
                .map(|e| e.round() as i32);
            // The entrance bore may run in the open for a while: the terrain decides there.
            if i == 0 && (p.z + 1..=p.z + 12).all(|z| hollow(p.xy().with_z(z))) {
                last = None;
                continue;
            }
            let mut z = p.z;
            while z > volume.min.z && hollow(p.xy().with_z(z)) {
                z -= 1;
            }
            if let Some((prev, at)) = last
                && (z - prev).abs() > 1
            {
                problems.push(format!("boyau {i} : marche de {} blocs entre {at:?} et {:?}", (z - prev).abs(), p.xy().with_z(z)));
            }
            last = Some((z, p.xy().with_z(z)));
        }
    }
    for (i, end) in grotte.dead_ends.iter().enumerate() {
        if !hollow(end.b) {
            problems.push(format!("cul-de-sac {i} bouché en {:?}", end.b));
        }
    }
    // `ICEBREAKER_GROTTE_ATTEINT=x,y`: the heights reached on foot in that column.
    if let Ok(column) = std::env::var("ICEBREAKER_GROTTE_ATTEINT")
        && let Some((x, y)) = column.split_once(',')
        && let (Ok(x), Ok(y)) = (x.trim().parse::<i32>(), y.trim().parse::<i32>())
    {
        let mut heights: Vec<i32> = seen.iter().filter(|p| p.x == x && p.y == y).map(|p| p.z).collect();
        heights.sort();
        problems.push(format!("atteint en ({x}, {y}) : {heights:?}"));
    }
    // `ICEBREAKER_GROTTE_COLONNE=x,y`: the blocks of that column, to look at a problem.
    if let Ok(column) = std::env::var("ICEBREAKER_GROTTE_COLONNE")
        && let Some((x, y)) = column.split_once(',')
        && let (Ok(x), Ok(y)) = (x.trim().parse::<i32>(), y.trim().parse::<i32>())
    {
        let line: String = (volume.min.z..=volume.max.z)
            .rev()
            .map(|z| if hollow(vek::Vec3::new(x, y, z)) { '.' } else { '#' })
            .collect();
        problems.push(format!("colonne ({x}, {y}) de z = {} (gauche) à {} : {line}", volume.max.z, volume.min.z));
    }
    let pockets: Vec<String> = grotte
        .pockets
        .iter()
        .map(|p| format!("{:?} r {:?} niveau {} sol {}", p.center, p.radius, p.level, p.floor().z))
        .collect();
    problems.extend(std::env::var("ICEBREAKER_GROTTE_COLONNE").ok().map(|_| format!("poches :\n  {}", pockets.join("\n  "))));
    let concretions: Vec<String> = grotte
        .speleothems
        .iter()
        .map(|s| format!("{:?} {:?} h {}", s.kind, s.base, s.height))
        .collect();
    problems.extend(std::env::var("ICEBREAKER_GROTTE_COLONNE").ok().map(|_| format!("concrétions :\n  {}", concretions.join("\n  "))));
    let bores: Vec<String> = grotte
        .bores
        .iter()
        .map(|b| {
            let along = (b.a.xy() - b.b.xy()).as_::<f32>().magnitude().max(1.0);
            format!("{:?} → {:?} (r {}, pente {:.2})", b.a, b.b, b.radius, (b.a.z - b.b.z).abs() as f32 / along)
        })
        .collect();
    // The inhabitants (K.5): each one stands on the floor and appears once;
    // each chest is drawn.
    let mut appeared = 0;
    for inhabitant in &grotte.inhabitants {
        let n = spawned.iter().filter(|p| p.map(|e| e.floor() as i32) == inhabitant.pos).count();
        appeared += n;
        let expected = inhabitant.loot.clone().map(common::lottery::LootSpec::LootTable);
        if expected.is_none() || loots.iter().any(|(p, loot)| *p == inhabitant.pos && Some(loot) != expected.as_ref()) {
            problems.push(format!("habitant {} en {:?} : butin autre que {expected:?}", inhabitant.asset, inhabitant.pos));
        }
        if n != 1 {
            problems.push(format!("habitant {} apparu {n} fois en {:?}", inhabitant.asset, inhabitant.pos));
        }
        // The bores round their floor as they are drawn: a block off is fine,
        // the creature falls or is pushed out (the mission knows the position).
        if ![0, 1, -1].iter().any(|dz| standing(inhabitant.pos + vek::Vec3::unit_z() * *dz)) {
            problems.push(format!("habitant {} dans la roche ou en l'air en {:?}", inhabitant.asset, inhabitant.pos));
        }
    }
    let chest = crate::icebreaker::dungeon::settings::sprite_kind(&grotte.chest);
    for (i, pos) in grotte.chests.iter().enumerate() {
        if block_at(*pos).and_then(|b| b.get_sprite()) != chest {
            problems.push(format!("coffre absent en {pos:?}"));
        }
        // Icebreaker B.5: the table of its tier, not the sprite's.
        let key = key_of(pos.xy());
        let table = chunks
            .get(&key)
            .and_then(|c| c.as_ref())
            .and_then(|c| c.meta().sprite_cfg_at(*pos - (key * chunk_size).with_z(0)))
            .and_then(|cfg| cfg.loot_table.clone());
        let expected = grotte
            .butin
            .as_ref()
            .map(|b| b.coffre(grotte.chest_tiers.get(i).copied().unwrap_or(1)).to_string());
        if table != expected {
            problems.push(format!("coffre en {pos:?} : butin {table:?} au lieu de {expected:?}"));
        }
    }
    // The ores (K.4): one stands on them (a mineable sprite is solid); counted
    // by the level of the pocket they lie in.
    let mut ores = vec![0usize; grotte.pockets.iter().map(|p| p.level + 1).max().unwrap_or(1) as usize];
    for p in &seen {
        let mined = block_at(*p - vek::Vec3::unit_z())
            .and_then(|b| b.get_sprite())
            .is_some_and(|s| s.mine_tool() == Some(common::comp::item::tool::ToolKind::Pick));
        if mined && let Some(pocket) = grotte.pockets.iter().find(|pocket| pocket.aabb().contains_point(*p)) {
            ores[pocket.level as usize] += 1;
        }
    }
    if ores.iter().sum::<usize>() == 0 {
        problems.push("aucun filon sur lequel marcher".into());
    }
    let summary = format!(
        "{}\nbouche en {:?}, arrivée en {:?}, {} blocs atteints à pied, filons par niveau {:?}, {} habitant(s) ({appeared} \
         apparus), {} coffre(s)\nboyaux :\n  {}",
        grotte.summary,
        grotte.mouth,
        plot.entrance(),
        seen.len(),
        ores,
        grotte.inhabitants.len(),
        grotte.chests.len(),
        bores.join("\n  ")
    );
    Some((summary, problems))
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

/// Icebreaker: the corner nearest to `near` (within `radius`) of an area of
/// `size` blocks, with `margin` more blocks around, whose ground is all at the
/// same height (X.15), and that height. `ground` gives the height where one
/// can stand on the land, none elsewhere (roof, tree, water).
pub fn find_flat_origin(
    near: vek::Vec2<i32>,
    size: vek::Vec2<i32>,
    margin: i32,
    radius: i32,
    ground: &mut impl FnMut(vek::Vec2<i32>) -> Option<i32>,
) -> Option<(vek::Vec2<i32>, i32)> {
    let mut heights = HashMap::new();
    let mut offsets = (-radius..=radius)
        .flat_map(|y| (-radius..=radius).map(move |x| vek::Vec2::new(x, y)))
        .filter(|d| d.x * d.x + d.y * d.y <= radius * radius)
        .collect::<Vec<_>>();
    offsets.sort_by_key(|d| d.x * d.x + d.y * d.y);
    offsets.into_iter().map(|d| near + d).find_map(|corner| {
        let mut level = None;
        for y in -margin..size.y + margin {
            for x in -margin..size.x + margin {
                let wpos = corner + vek::Vec2::new(x, y);
                let height = *heights.entry(wpos).or_insert_with(|| ground(wpos));
                match (height, level) {
                    (None, _) => return None,
                    (Some(h), Some(l)) if h != l => return None,
                    (Some(h), _) => level = Some(h),
                }
            }
        }
        Some((corner, level?))
    })
}

#[cfg(test)]
mod flat_tests {
    use super::find_flat_origin;
    use vek::Vec2;

    #[test]
    fn la_zone_plate_la_plus_proche() {
        // Flat (10) for x >= 5, a slope before; nothing to stand on at y = 3.
        let mut ground = |p: Vec2<i32>| (p.y != 3).then(|| if p.x >= 5 { 10 } else { p.x });
        let found = find_flat_origin(Vec2::new(0, 0), Vec2::new(4, 1), 1, 10, &mut ground);
        assert_eq!(found, Some((Vec2::new(6, 0), 10)));
        // Too far for the radius.
        assert_eq!(find_flat_origin(Vec2::new(0, 0), Vec2::new(4, 1), 1, 3, &mut ground), None);
    }
}
