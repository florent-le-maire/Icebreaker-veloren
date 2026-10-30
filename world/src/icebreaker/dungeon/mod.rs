//! Icebreaker: our dungeon generator. A dungeon is first a plan of cells over
//! several levels (`plan`), then voxels drawn by the `IcebreakerDungeon` plot
//! (`site/plot/icebreaker_dungeon.rs`).

pub mod plan;
pub mod rooms;

use plan::{Cell, Plan};
use rooms::{RoomCatalog, RoomRole};
use serde::Deserialize;

/// Where the dungeon data lives, under `assets/`.
const DATA_DIR: &str = "icebreaker/donjons";

/// How a plan becomes voxels (`generation.ron` → `render`).
#[derive(Clone, Debug, Deserialize)]
pub struct RenderSettings {
    /// Blocks per cell, horizontally.
    pub cell_size: i32,
    /// Blocks per level (floor included). Stairs climb one level over one
    /// cell, one block per step: at most `cell_size`.
    pub level_height: i32,
    /// Cells of the tunnel from the surface down to the entrance room.
    pub tunnel_cells: i32,
    pub wall_color: (u8, u8, u8),
    pub floor_color: (u8, u8, u8),
}

/// The settings of the generator (`assets/icebreaker/donjons/generation.ron`).
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub render: RenderSettings,
}

impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let settings: Self =
            ron::from_str(text).map_err(|e| format!("generation.ron illisible : {e}"))?;
        let render = &settings.render;
        if render.cell_size < 3 || render.level_height < 3 || render.level_height > render.cell_size
        {
            return Err("render : il faut 3 ≤ level_height ≤ cell_size".into());
        }
        if render.tunnel_cells < 1 {
            return Err("render : tunnel_cells doit être au moins 1".into());
        }
        Ok(settings)
    }

    pub fn load() -> Result<Self, String> { Self::parse(&read_data("generation.ron")?) }
}

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
    Ok(Dungeon { plan, tunnel })
}

/// Generates the dungeon of an instance.
pub fn generate(settings: &Settings, _seed: u64) -> Result<Dungeon, String> {
    let catalog = RoomCatalog::load()?;
    // A single room to look at (`ICEBREAKER_DONJON_SALLE`, `-Salle` of the launcher).
    let room = match std::env::var("ICEBREAKER_DONJON_SALLE") {
        Ok(id) if !id.is_empty() => catalog.get(&id).ok_or(format!("salle inconnue : {id}"))?,
        _ => catalog
            .with_role(RoomRole::Entrance)
            .into_iter()
            .next()
            .ok_or("aucune salle d'entrée")?,
    };
    single_room(settings, room)
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
