//! Icebreaker: our dungeon generator. A dungeon is first a plan of cells over
//! several levels (`plan`), then voxels drawn by the `IcebreakerDungeon` plot
//! (`site/plot/icebreaker_dungeon.rs`).

pub mod assemble;
pub mod graph;
pub mod plan;
pub mod populate;
pub mod rooms;
pub mod settings;

use plan::{Cell, Plan};
use rooms::{RoomCatalog, RoomRole};
pub use settings::{Mission, RenderSettings, Settings};

/// Where the dungeon data lives, under `assets/`.
const DATA_DIR: &str = "icebreaker/donjons";

/// The path of a data file of the generator.
pub fn data_path(name: &str) -> std::path::PathBuf {
    common::assets::ASSETS_PATH.join(DATA_DIR).join(name)
}

/// Reads a data file of the generator.
pub fn read_data(name: &str) -> Result<String, String> {
    let path = data_path(name);
    std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))
}

/// The tunnel from the surface down to a door of the entrance room: its cells
/// at the entrance level, from the door outward, and its direction.
#[derive(Clone, Debug)]
pub struct Tunnel {
    pub cells: Vec<Cell>,
    pub dir: (i32, i32),
}

/// A generated dungeon, ready to be drawn.
#[derive(Clone, Debug)]
pub struct Dungeon {
    pub plan: Plan,
    pub tunnel: Tunnel,
    /// What the dungeon is made of, for the logs.
    pub summary: String,
    /// Who lives there.
    pub faction: settings::Faction,
    /// Locked doors (door cells), and the cells where their keys lie, with
    /// the key of each.
    pub locks: Vec<(Cell, settings::KeySettings)>,
    pub keys: Vec<(Cell, settings::KeySettings)>,
}

/// Marks the tunnel in front of `door` (a wall cell of a room, opening toward
/// `dir`), `length` cells long, and opens the door. The tunnel climbs to the
/// surface, so the cells above it are kept free too (`~` at the entrance level,
/// `%` above it and along it: cells the plot leaves to the terrain).
/// Returns `None` if the cells are not free.
pub fn reserve_tunnel(plan: &mut Plan, door: Cell, dir: (i32, i32), length: i32) -> Option<Tunnel> {
    let cells: Vec<Cell> = (1..=length)
        .map(|k| door.offset(dir.0 * k, dir.1 * k, 0))
        .collect();
    let (side_x, side_y) = (dir.1, dir.0);
    let footprint = cells
        .iter()
        .flat_map(|c| [*c, c.offset(0, 0, 1)])
        .collect::<Vec<_>>();
    let sides = footprint
        .iter()
        .flat_map(|c| [c.offset(side_x, side_y, 0), c.offset(-side_x, -side_y, 0)])
        .collect::<Vec<_>>();
    if footprint.iter().chain(&sides).any(|c| !plan.is_free(*c)) {
        return None;
    }
    for cell in footprint {
        plan.cells.insert(cell, if cell.level == door.level { '~' } else { '%' });
    }
    for cell in sides {
        plan.cells.insert(cell, '%');
    }
    plan.cells.insert(door, 'D');
    Some(Tunnel { cells, dir })
}

/// Closes the connectors left unused: a door stays a wall, a pit stays floor.
pub fn close_connectors(plan: &mut Plan) {
    for symbol in plan.cells.values_mut() {
        match *symbol {
            '+' => *symbol = '#',
            '*' => *symbol = '.',
            _ => {},
        }
    }
}

/// A single room, its tunnel through the first door that has room for it.
pub fn single_room(settings: &Settings, room: &rooms::RoomTemplate) -> Result<Dungeon, String> {
    let mut plan = room.plan.clone();
    let tunnel = room
        .doors()
        .find_map(|door| {
            let mut tried = plan.clone();
            let tunnel =
                reserve_tunnel(&mut tried, door.cell, door.direction(), settings.render.tunnel_cells)?;
            plan = tried;
            Some(tunnel)
        })
        .ok_or("pas de place pour le tunnel d'entrée")?;
    close_connectors(&mut plan);
    Ok(Dungeon {
        plan,
        tunnel,
        summary: format!("salle seule : {}", room.id),
        faction: settings
            .factions
            .get(&settings.default_faction)
            .cloned()
            .ok_or("faction par défaut inconnue")?,
        locks: Vec::new(),
        keys: Vec::new(),
    })
}

/// Seeds tried one after the other when a dungeon can't be assembled.
const SEED_TRIES: u64 = 10;

/// Generates the dungeon of a mission: its graph, then the assembly of the
/// rooms. A seed whose dungeon can't be assembled gives way to the next one.
pub fn generate(settings: &Settings, mission: &Mission) -> Result<Dungeon, String> {
    let catalog = RoomCatalog::load()?;
    // A single room to look at (`ICEBREAKER_DONJON_SALLE`, `-Salle` of the launcher).
    if let Ok(id) = std::env::var("ICEBREAKER_DONJON_SALLE")
        && !id.is_empty()
    {
        let room = catalog.get(&id).ok_or(format!("salle inconnue : {id}"))?;
        return single_room(settings, room);
    }

    let resolved = settings.resolve(mission)?;
    let missing: Vec<&str> = std::iter::once(RoomRole::Entrance)
        .chain([resolved.goal, resolved.fallback, RoomRole::Hall, RoomRole::GuardRoom])
        .chain(resolved.weights.keys().copied())
        .filter(|role| catalog.with_role(*role).is_empty())
        .map(RoomRole::name)
        .collect();
    if !missing.is_empty() {
        return Err(format!("aucune salle pour : {}", missing.join(", ")));
    }
    // Each lock has its own key (`keys` of the settings, in the order of the locks).
    let with_keys = |cells: &[(Cell, usize)]| {
        cells
            .iter()
            .map(|(cell, lock)| {
                let key = settings.keys.get(*lock).ok_or(format!("pas de clé pour le verrou {lock}"))?;
                Ok((*cell, key.clone()))
            })
            .collect::<Result<Vec<_>, String>>()
    };
    let mut failures = Vec::new();
    for seed in mission.seed..mission.seed + SEED_TRIES {
        let graph = graph::generate(&resolved, seed);
        let assembled = assemble::assemble(
            &graph,
            &catalog,
            seed,
            &settings.assembly,
            settings.render.tunnel_cells,
            |failure| failures.push(format!("graine {seed}, {failure}")),
        );
        if let Some(mut assembled) = assembled {
            let (creatures, chests) = populate::populate(&mut assembled, &graph, &resolved, seed);
            let rooms: Vec<String> = assembled
                .rooms
                .iter()
                .map(|r| format!("{} {}", graph.rooms[r.graph_room].role.name(), r.template.id))
                .collect();
            return Ok(Dungeon {
                plan: assembled.plan,
                tunnel: assembled.tunnel,
                summary: format!(
                    "{} {} ({}), graine {seed} ({} tentative(s)), {creatures} créature(s), {chests} \
                     coffre(s) ; salles : {}\n{}",
                    mission.size,
                    mission.objective,
                    resolved.modifiers.join(", "),
                    assembled.attempts,
                    rooms.join(", "),
                    graph.describe()
                ),
                faction: resolved.faction.clone(),
                locks: with_keys(&assembled.locks)?,
                keys: with_keys(&assembled.keys)?,
            });
        }
    }
    Err(format!("aucun donjon assemblé :\n{}", failures.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn settings_load() { Settings::load().unwrap(); }

    #[test]
    fn threat_changes_the_dungeon() {
        let settings = Settings::load().unwrap();
        // (cells, creatures) over a few seeds.
        let measure = |threat: i32| {
            (0..20).fold((0, 0), |(cells, creatures), seed| {
                let mission = Mission {
                    seed,
                    threat: Some(threat),
                    ..Mission::default()
                };
                let dungeon = generate(&settings, &mission).unwrap();
                let plan = &dungeon.plan;
                let people = plan.cells.values().filter(|s| populate::kind_of(**s).is_some()).count();
                (cells + plan.cells.len(), creatures + people)
            })
        };
        let (low, high) = (measure(1), measure(5));
        assert!(high.0 > low.0, "menace 5 pas plus grande : {low:?} / {high:?}");
        assert!(high.1 > 2 * low.1, "menace 5 pas plus peuplée : {low:?} / {high:?}");
    }

    #[test]
    fn every_parameter_resolves() {
        let settings = Settings::load().unwrap();
        let p = &settings.parameters;
        for region in p.regions.keys() {
            for season in p.seasons.keys() {
                for objective in settings.goal_roles.keys() {
                    let mission = Mission {
                        region: Some(region.clone()),
                        season: Some(season.clone()),
                        objective: objective.clone(),
                        density: Some("high".into()),
                        ..Mission::default()
                    };
                    let dungeon = generate(&settings, &mission).unwrap_or_else(|e| panic!("{mission:?} : {e}"));
                    assert!(!dungeon.plan.cells.is_empty());
                }
            }
        }
        let unknown = Mission {
            region: Some("lune".into()),
            ..Mission::default()
        };
        assert!(settings.resolve(&unknown).unwrap_err().contains("région inconnue"));
    }

    #[test]
    fn faction_creatures_exist() {
        let settings = Settings::load().unwrap();
        for (name, faction) in &settings.factions {
            for asset in faction.creatures.values().flatten() {
                let path = common::assets::ASSETS_PATH.join(asset.replace('.', "/") + ".ron");
                assert!(path.exists(), "faction {name} : {asset} introuvable");
            }
        }
    }

    #[test]
    fn each_locked_door_has_its_own_key() {
        let settings = Settings::load().unwrap();
        for key in &settings.keys {
            for asset in [&key.item, &key.loot] {
                let path = common::assets::ASSETS_PATH.join(asset.replace('.', "/") + ".ron");
                assert!(path.exists(), "clé : {asset} introuvable");
            }
        }
        let mut locked = 0;
        for size in ["small", "medium", "large"] {
            for seed in 0..30 {
                let mission = Mission {
                    seed,
                    size: size.into(),
                    ..Mission::default()
                };
                let dungeon = generate(&settings, &mission).unwrap();
                let items: BTreeSet<_> = dungeon.locks.iter().map(|(_, k)| &k.item).collect();
                assert_eq!(items.len(), dungeon.locks.len(), "{size} {seed} : deux portes, même clé");
                let keys: BTreeSet<_> = dungeon.keys.iter().map(|(_, k)| &k.item).collect();
                assert_eq!(items, keys, "{size} {seed} : clés et portes ne vont pas ensemble");
                for (cell, _) in &dungeon.locks {
                    assert_eq!(dungeon.plan.get(*cell), 'D', "{size} {seed} : porte fermée hors d'une porte");
                }
                for (cell, _) in &dungeon.keys {
                    assert_eq!(dungeon.plan.get(*cell), '.', "{size} {seed} : clé hors du sol libre");
                }
                locked += dungeon.locks.len();
            }
        }
        assert!(locked > 0, "aucune porte fermée en 90 donjons");
    }

    #[test]
    fn every_room_alone_reaches_its_tunnel() {
        let settings = Settings::load().unwrap();
        let catalog = RoomCatalog::load().unwrap();
        for room in &catalog.rooms {
            let dungeon = single_room(&settings, room).unwrap();
            let top = *dungeon.tunnel.cells.last().unwrap();
            let reached = dungeon.plan.reachable(top);
            for (cell, symbol) in &dungeon.plan.cells {
                if plan::is_walkable(*symbol) {
                    assert!(reached.contains(cell), "{} : {cell:?} inaccessible", room.id);
                }
            }
        }
    }
}
