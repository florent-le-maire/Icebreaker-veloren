//! Icebreaker: encounter instances. A server started with
//! `ICEBREAKER_INSTANCE_SITE` set generates a small world around one chosen
//! site, without the wildlife and roaming monsters of a whole living world.

// Icebreaker O : le plateau du lieu d'entraînement.
pub mod arene;
pub mod butin;
pub mod dungeon;
// Icebreaker FO.4 : la région Forêt.
pub mod foret;
pub mod grotte;
pub mod races;
// Icebreaker VF : le plan du village fixe.
pub mod village_plan;

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

/// Icebreaker U.16 : la pioche casse tout (`common::icebreaker_destruction`) dans les tronçons
/// d'une Grotte ou d'un donjon généré ; leur bord est la limite, ailleurs Veloren garde ses règles.
pub fn se_creuse_ici(world: &crate::World, index: crate::IndexRef, wpos: vek::Vec2<i32>) -> bool {
    use common::terrain::CoordinateConversions;
    world.sim().get(wpos.wpos_to_cpos()).is_some_and(|chunk| {
        chunk.sites.iter().any(|site| {
            matches!(
                index.sites.get(*site).kind,
                Some(SiteKind::IcebreakerGrotte | SiteKind::IcebreakerDungeon)
            )
        })
    })
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
        "Foret" => SiteKind::IcebreakerForet,
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

/// Icebreaker U.8: where players arrive under the ground (the safe room of
/// the cave, which has no mouth at the surface): a block one stands in.
/// Icebreaker F.31: and the safe room of a dungeon of our generator.
pub fn dungeon_arrival_under(site: &Site) -> Option<vek::Vec3<i32>> {
    site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerGrotte(grotte) => Some(grotte.get().grotte().arrival),
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.get().arrival()),
        _ => None,
    })
}

/// Icebreaker U.11: the shape (a preset of `formes.ron`) and the seed of the
/// cave of `site`; F.33: or of its generated dungeon. First what it is
/// (`Grotte`, `Donjon`).
pub fn forme_info(site: &Site) -> Option<(&'static str, String, u64)> {
    site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerGrotte(grotte) => {
            let plot = grotte.get();
            Some(("Grotte", plot.grotte().profil.clone(), plot.grotte().seed))
        },
        PlotKind::IcebreakerDungeon(dungeon) => {
            let plot = dungeon.get();
            let (profil, seed) = plot.forme();
            Some(("Donjon", profil.to_string(), seed))
        },
        // Icebreaker FO.4.
        PlotKind::IcebreakerForet(foret) => {
            let plot = foret.get();
            Some(("Forêt", plot.lieu().profil.clone(), plot.lieu().seed))
        },
        _ => None,
    })
}

/// Icebreaker F.33: the presets a party may choose the shape of its generated
/// dungeon from (`donjons/formes.ron`, read again at each call).
pub fn dungeon_presets() -> Vec<String> {
    dungeon::formes::Formes::load().map_or_else(|_| Vec::new(), |formes| formes.profils.into_keys().collect())
}

/// Icebreaker U.11: the presets a party may choose the shape of its cave
/// from: the profiles of `formes.ron`, read again at each call (the workshop
/// adds some while the server runs).
pub fn grotte_presets() -> Vec<String> {
    grotte::Settings::load()
        .ok()
        .and_then(|settings| settings.formes)
        .map_or_else(Vec::new, |formes| formes.profils.keys().cloned().collect())
}

/// Icebreaker : the objects of the special rooms of the dungeon of `site` (`/aller`): the role of the room
/// (`portal`, `altar`), where the object stands, and a place in front of it.
pub fn dungeon_specials(site: &Site) -> Vec<(&'static str, vek::Vec3<f32>, vek::Vec3<f32>)> {
    site.plots()
        .filter_map(|plot| match plot.kind() {
            PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.get().special_places()),
            _ => None,
        })
        .flatten()
        .map(|(role, center, front)| (role.name(), center, front))
        .collect()
}

/// Where players arrive at a site of ours, seen from above (a dungeon of our generator: above its safe room, F.31;
/// the real place is `dungeon_arrival_under`).
pub fn dungeon_entrance(site: &Site) -> Option<vek::Vec2<i32>> {
    site.plots().find_map(|plot| match plot.kind() {
        PlotKind::IcebreakerDungeon(dungeon) => Some(dungeon.get().arrival().xy()),
        PlotKind::IcebreakerGrotte(grotte) => Some(grotte.get().entrance()),
        // Icebreaker FO.4: the clearing at the edge of the forest.
        PlotKind::IcebreakerForet(foret) => Some(foret.get().entrance()),
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

/// Icebreaker VF : la hauteur du sol de la place du village fixe (où l'on apparaît).
pub fn village_sol(site: &Site) -> Option<i32> {
    let plaza = site.plazas().next()?;
    site.tiles.get(site.plot(plaza).root_tile()).hard_alt
}

/// Icebreaker VF.5 : le sol du village fixe pour le plan vu de dessus : la
/// place, les chemins et les terrains (sorte, nom, emprise en blocs). La sorte
/// d'un terrain est son décor (`champ`, `cimetiere`, `enclos`) ou `terrain_nu`.
pub fn village_fond(site: &Site) -> Vec<(String, String, vek::Aabr<i32>)> {
    let Ok(plan) = village_plan::VillagePlan::load() else {
        return Vec::new();
    };
    // Une ville tirée au hasard n'a pas été construite d'après le plan.
    if village_lieux(site).is_empty() {
        return Vec::new();
    }
    let blocs = |r: village_plan::Rect| {
        let t = r.aabr();
        vek::Aabr {
            min: site.tile_wpos(t.min),
            max: site.tile_wpos(t.max),
        }
    };
    // La place sans nom : le point du joueur s'y trouve souvent.
    let mut fond = vec![("place".to_string(), String::new(), blocs(plan.place))];
    fond.extend(plan.chemins.iter().map(|r| ("chemin".to_string(), String::new(), blocs(*r))));
    fond.extend(
        plan.lieux
            .iter()
            .filter(|lieu| lieu.forme == village_plan::Forme::Terrain)
            .map(|lieu| {
                let (sorte, nom) = match lieu.decor {
                    Some(village_plan::Decor::Champ) => ("champ", "Champ"),
                    Some(village_plan::Decor::Cimetiere) => ("cimetiere", "Cimetière"),
                    Some(village_plan::Decor::Enclos) => ("enclos", "Écurie"),
                    None => ("terrain_nu", ""),
                };
                (sorte.to_string(), nom.to_string(), blocs(lieu.rect()))
            }),
    );
    fond
}

/// Icebreaker VF : les lieux bâtis du village fixe (pas les terrains de
/// décor), par id du plan (`village_plan.ron`), avec leur palier. Vide si le
/// site n'a pas été construit d'après le plan.
pub fn village_lieux(site: &Site) -> Vec<(String, u32, VillageHouse)> {
    let Ok(plan) = village_plan::VillagePlan::load() else {
        return Vec::new();
    };
    plan.lieux
        .iter()
        .filter(|lieu| lieu.forme != village_plan::Forme::Terrain)
        .filter_map(|lieu| {
            let door = lieu.porte_tuile();
            let tile = site.tiles.get(door);
            let plot = tile.plot?;
            if matches!(site.plot(plot).kind(), PlotKind::IcebreakerTerrain(_)) {
                return None;
            }
            let bati = lieu.bati_aabr();
            Some((lieu.id.clone(), lieu.palier, VillageHouse {
                plot,
                front: site.tile_center_wpos(door + lieu.porte.dir()),
                inside: site.tile_center_wpos(door),
                bounds: vek::Aabr {
                    min: site.tile_wpos(bati.min),
                    max: site.tile_wpos(bati.max),
                },
                alt: tile.hard_alt.unwrap_or(0),
            }))
        })
        .collect()
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
                        // Icebreaker VF.4 : le terrain du village dessine le cimetière et l'enclos.
                        | PlotKind::IcebreakerTerrain(_)
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
/// (the current one stays). `profil` (Icebreaker U.11, F.33): the shape asked
/// for a cave or a generated dungeon.
pub fn regenerate_slot(world: &crate::World, site: &Site, seed: u64, profil: Option<&str>) -> Result<bool, String> {
    let land = crate::Land::from_sim(world.sim());
    for plot in site.plots() {
        let replaced = match plot.kind() {
            PlotKind::IcebreakerDungeon(dungeon) => dungeon.regenerate(&land, site.origin, seed, profil)?,
            PlotKind::IcebreakerGrotte(grotte) => grotte.regenerate(&land, site.origin, seed, profil)?,
            // Icebreaker FO.4.
            PlotKind::IcebreakerForet(foret) => foret.regenerate(&land, site.origin, seed, profil)?,
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
    // Icebreaker F.33 : the party arrives where the server puts it (`icebreaker_slots::arrival_pos`), in the safe
    // room, not on the ground above.
    let arrival = dungeon.arrival();
    let landed = world.find_accessible_pos_above(index, arrival.xy(), arrival.z - 1);
    if landed.z as i32 != arrival.z {
        let block = |z: i32| {
            let key = key_of(arrival.xy());
            chunks
                .get(&key)
                .and_then(|c| c.as_ref())
                .and_then(|c| c.get(arrival.with_z(z) - (key * chunk_size).with_z(0)).ok().copied())
        };
        problems.push(format!(
            "arrivée {arrival:?} : le groupe apparaît en z {} (sous lui {:?}, à sa place {:?})",
            landed.z as i32,
            block(arrival.z - 1),
            block(arrival.z)
        ));
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
/// block up, any fall down), every zone is reached (Icebreaker K.2, U.4). `None`
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
    let mut start = grotte.arrival + vek::Vec3::unit_z();
    while hollow(start - vek::Vec3::unit_z()) && start.z > volume.min.z {
        start.z -= 1;
    }
    // Icebreaker U.13: walked twice, through the walls to break (they are broken) and without
    // breaking anything.
    let walk = |breaking: bool| {
        let hollow = |p: vek::Vec3<i32>| hollow(p) || (breaking && grotte.wall(p));
        let standing = |p: vek::Vec3<i32>| !hollow(p - vek::Vec3::unit_z()) && hollow(p) && hollow(p + vek::Vec3::unit_z());
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
        seen
    };
    let (seen, open) = (walk(true), walk(false));
    let mut problems = Vec::new();
    // Icebreaker U.4: the trail reaches every zone (its anchor, a block off
    // at most: the terrain may round a floor).
    for (i, zone) in grotte.caverne.zones.iter().enumerate() {
        let anchor = grotte.monde(zone.ancre);
        if ![0, 1, -1].iter().any(|dz| seen.contains(&(anchor + vek::Vec3::unit_z() * *dz))) {
            problems.push(format!("zone {i} ({:?}, niveau {}) inaccessible depuis la bouche : {anchor:?}", zone.genre, grotte.levels[i]));
        }
        // Icebreaker U.13: a zone behind a wall is not reached without breaking.
        if grotte.caverne.fermees.contains(&i) && [0, 1, -1].iter().any(|dz| open.contains(&(anchor + vek::Vec3::unit_z() * *dz))) {
            problems.push(format!("zone {i} ({:?}) atteinte sans casser son mur : {anchor:?}", zone.genre));
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
    let zones: Vec<String> = grotte
        .caverne
        .zones
        .iter()
        .enumerate()
        .map(|(i, z)| format!("{:?} en {:?}, rayons {:?}, niveau {}, {} sols", z.genre, grotte.monde(z.ancre), z.rayon, grotte.levels[i], z.sols.len()))
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
        // The terrain may round a floor as it is drawn: a block off is fine,
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
    // Icebreaker U.19: the veins, counted by the level of the zone they lie in.
    let mut ores = vec![0usize; grotte.niveaux.max(1) as usize];
    for vein in grotte.veins.iter().filter(|v| !v.in_ore_room) {
        if let Some(p) = vein.rock.first() {
            let last = ores.len() - 1;
            ores[(grotte.level_at(*p).max(0) as usize).min(last)] += 1;
        }
    }
    if ores.iter().sum::<usize>() == 0 {
        problems.push("aucun filon dans les zones".into());
    }
    // Icebreaker U.15: the ore room, full, behind its wall (a zone closed by a wall is checked above).
    let room = grotte.veins.iter().filter(|v| v.in_ore_room).count();
    match grotte.caverne.minerai {
        Some(m) if grotte.caverne.fermees.contains(&m) && room > 0 => {},
        m => problems.push(format!("salle au minerai absente, ouverte ou vide : {m:?}, {room} filon(s)")),
    }
    // Icebreaker U.19: every block of a vein is a block of its ore.
    let minerais = common::icebreaker_minerais::minerais();
    let missing = grotte
        .vein_blocks
        .iter()
        .filter(|(p, ore)| block_at(**p).and_then(|b| minerais.de_bloc(&b)).map(|(i, _)| i) != Some(**ore))
        .count();
    if missing > 0 {
        problems.push(format!("{missing} bloc(s) de filon sans leur minerai sur {}", grotte.vein_blocks.len()));
    }
    let summary = format!(
        "{}\nbouche en {:?}, arrivée en {:?}, {} blocs atteints à pied, filons par niveau {:?}, {room} dans la salle au minerai, {} blocs de minerai, {} habitant(s) ({appeared} \
         apparus), {} coffre(s)\nzones :\n  {}",
        grotte.summary,
        grotte.mouth,
        plot.entrance(),
        seen.len(),
        ores,
        grotte.vein_blocks.len(),
        grotte.inhabitants.len(),
        grotte.chests.len(),
        zones.join("\n  ")
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

/// Icebreaker VF : le village est le village fixe (décision 0022), construit
/// d'après `village_plan.ron`, et non une ville tirée au hasard.
pub fn is_village_fixe(kind: SiteKind) -> bool { is_village() && kind == SiteKind::Refactor }

/// Icebreaker VF : un relief jusqu'à cette hauteur (blocs) suffit au village
/// fixe : on ne cherche pas d'autre place.
pub const VILLAGE_RELIEF_OK: f32 = 8.0;

/// Icebreaker VF : l'écart de hauteur (blocs) du terrain d'origine sous le
/// village fixe et sa bordure s'il était posé au chunk `loc` ; aucun s'il y a
/// de l'eau ou si le plan est illisible.
pub fn village_relief(sim: &crate::sim::WorldSim, loc: vek::Vec2<i32>) -> Option<f32> {
    use common::terrain::TerrainChunkSize;
    use common::vol::RectVolSize;
    let plan = village_plan::VillagePlan::load().ok()?;
    let emprise = plan.emprise();
    let centre = loc.map2(TerrainChunkSize::RECT_SIZE, |e, sz: u32| e * sz as i32 + sz as i32 / 2);
    // La taille d'une tuile de site (`TILE_SIZE`, privée à `site::tile`).
    let tile = crate::site::Site::default().tile_wpos(vek::Vec2::one()).x;
    let land = crate::Land::from_sim(sim);
    let (mut bas, mut haut) = (f32::MAX, f32::MIN);
    for y in emprise.min.y - plan.marge..emprise.max.y + plan.marge {
        for x in emprise.min.x - plan.marge..emprise.max.x + plan.marge {
            let wpos = centre + vek::Vec2::new(x, y) * tile + tile / 2;
            if sim.get_wpos(wpos).is_none_or(|chunk| chunk.river.near_water()) {
                return None;
            }
            let alt = land.get_alt_approx(wpos);
            bas = bas.min(alt);
            haut = haut.max(alt);
        }
    }
    Some(haut - bas)
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

/// Icebreaker : la première hauteur, de `z_from` en montant, où l'on tient debout
/// (`plein` dessous, `libre` sur trois blocs). Au-dessus de `max_z`, le tronçon
/// n'a que de l'air : la hauteur `max_z` compte, pour un sol au sommet du tronçon
/// (la place d'un village plat), sans quoi on restait à `z_from`, sous terre.
pub fn first_standing_z(
    z_from: i32,
    max_z: i32,
    plein: impl Fn(i32) -> bool,
    libre: impl Fn(i32) -> bool,
) -> Option<i32> {
    (z_from..=max_z).find(|z| plein(z - 1) && (0..3).all(|dz| libre(z + dz)))
}

#[cfg(test)]
mod standing_tests {
    use super::first_standing_z;

    #[test]
    fn le_sol_au_sommet_du_troncon() {
        // Plein jusqu'à 192, le tronçon s'arrête à 193 (air au-dessus).
        let plein = |z: i32| z <= 192;
        assert_eq!(first_standing_z(191, 193, plein, |z| !plein(z)), Some(193));
    }

    #[test]
    fn le_premier_sol_en_montant() {
        // Une poche d'air de 3 blocs à 100, la surface à 120.
        let plein = |z: i32| z < 100 || (103..120).contains(&z);
        assert_eq!(first_standing_z(90, 200, plein, |z| !plein(z)), Some(100));
        assert_eq!(first_standing_z(101, 200, plein, |z| !plein(z)), Some(120));
    }
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
