//! Icebreaker: our dungeon generator. A dungeon is first a plan of cells over
//! several levels (`plan`), then voxels drawn by the `IcebreakerDungeon` plot
//! (`site/plot/icebreaker_dungeon.rs`).

pub mod assemble;
pub mod graph;
pub mod plan;
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
        if let Some(assembled) = assembled {
            let rooms: Vec<String> = assembled
                .rooms
                .iter()
                .map(|r| format!("{} {}", graph.rooms[r.graph_room].role.name(), r.template.id))
                .collect();
            return Ok(Dungeon {
                plan: assembled.plan,
                tunnel: assembled.tunnel,
                summary: format!(
                    "{} {}, graine {seed} ({} tentative(s)) ; salles : {}\n{}",
                    mission.size,
                    mission.objective,
                    assembled.attempts,
                    rooms.join(", "),
                    graph.describe()
                ),
            });
        }
    }
    Err(format!("aucun donjon assemblé :\n{}", failures.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_load() { Settings::load().unwrap(); }

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
