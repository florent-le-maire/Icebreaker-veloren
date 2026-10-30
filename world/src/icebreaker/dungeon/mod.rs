//! Icebreaker: our dungeon generator. A dungeon is first a plan of cells over
//! several levels (`plan`), then voxels drawn by the `IcebreakerDungeon` plot
//! (`site/plot/icebreaker_dungeon.rs`).

pub mod plan;

use plan::{Cell, Plan};
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

/// Reads a data file of the generator.
pub fn read_data(name: &str) -> Result<String, String> {
    let path = common::assets::ASSETS_PATH.join(DATA_DIR).join(name);
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

/// The only room of the first version (task G.1), drawn by hand.
const FIRST_ROOM: &str = "\
#####+#####
#.........#
#..SSSSS..#
#..SSSSS..#
+.........+
#.........#
###########
";

/// Marks the tunnel in front of `door` (a wall cell of a room, opening toward
/// `dir`), `length` cells long, and opens the door. The tunnel climbs to the
/// surface, so the cells above it are kept free too (`~` at the entrance level,
/// `w` above it and along it: cells the plot leaves to the terrain).
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
        plan.cells.insert(cell, if cell.level == door.level { '~' } else { 'w' });
    }
    for cell in sides {
        plan.cells.insert(cell, 'w');
    }
    plan.cells.insert(door, 'D');
    Some(Tunnel { cells, dir })
}

/// Generates the dungeon of an instance.
pub fn generate(settings: &Settings, _seed: u64) -> Result<Dungeon, String> {
    let mut plan = Plan::parse(FIRST_ROOM)?;
    let tunnel = reserve_tunnel(&mut plan, Cell::new(5, 0, 0), (0, -1), settings.render.tunnel_cells)
        .ok_or("pas de place pour le tunnel d'entrée")?;
    // The doors that lead nowhere stay walls.
    for symbol in plan.cells.values_mut() {
        if *symbol == '+' {
            *symbol = '#';
        }
    }
    Ok(Dungeon { plan, tunnel })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_load() { Settings::load().unwrap(); }

    #[test]
    fn first_room_reaches_the_tunnel() {
        let settings = Settings::load().unwrap();
        let dungeon = generate(&settings, 0).unwrap();
        let top = *dungeon.tunnel.cells.last().unwrap();
        assert!(dungeon.plan.reachable(top).contains(&Cell::new(5, 3, 0)));
    }
}
