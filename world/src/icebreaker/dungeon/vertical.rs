//! Icebreaker F.55 : rooms that are all height, generated (`vertical` of `generation.ron`): the dungeon lacked
//! relief, a flight of stairs of one level here and there. They are rooms of several levels as the drawn ones
//! (`salles/hall_escalier.txt`), made of the symbols of the plans (`^ > v <` stairs, `_` holes, `O` pillars, `+`
//! doors): what draws the blocks, walks the plan and shows it in the workshop takes them as any room.
//!
//! - the Z stairs: two flights side by side going opposite ways, one above the other, a landing at each end;
//!   doors on every level, at the landings;
//! - the tower (F.56): a square well several levels deep, a flight of stairs along each wall in turn (a level a
//!   wall), a landing with its doors in each corner; in its middle a void one sees the whole well through, and
//!   falls into, or a pillar of rock.

use super::{
    plan::{Cell, Plan, stairs_symbol},
    rooms::{RoomCatalog, RoomRole, RoomTemplate},
    settings::Range,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;

/// `generation.ron` → `vertical`.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VerticalSettings {
    ///     /// together (0: none).
    pub escaliers_z: i32,
    /// Levels a Z stairs climbs.
    pub z_etages: Range,
    /// Icebreaker F.56 : chances out of 100 that the main path, where it changes level, goes down several levels
    /// at once (`tour_etages`) by a tower (or by Z stairs of that height).
    pub tours: i32,
    pub tour_etages: Range,
    /// Cells from a wall of a tower to the one across (3 to 7).
    pub tour_largeur: i32,
    /// Chances out of 100 that the middle of a tower is a void (else a pillar).
    pub tour_vide: i32,
}

impl Default for VerticalSettings {
    fn default() -> Self { Self { escaliers_z: 0, z_etages: Range(1, 3), tours: 0, tour_etages: Range(2, 4), tour_largeur: 3, tour_vide: 50 } }
}

impl VerticalSettings {
    pub fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !(0..=90).contains(&self.escaliers_z) {
            errors.push("vertical : escaliers_z de 0 à 90".into());
        }
        if self.z_etages.0 < 1 || self.z_etages.1 < self.z_etages.0 || self.z_etages.1 > 8 {
            errors.push("vertical : z_etages de 1 à 8, min ≤ max".into());
        }
        if !(0..=100).contains(&self.tours) || !(0..=100).contains(&self.tour_vide) {
            errors.push("vertical : tours et tour_vide de 0 à 100".into());
        }
        if self.tour_etages.0 < 2 || self.tour_etages.1 < self.tour_etages.0 || self.tour_etages.1 > 8 {
            errors.push("vertical : tour_etages de 2 à 8, min ≤ max".into());
        }
        if !(3..=7).contains(&self.tour_largeur) {
            errors.push("vertical : tour_largeur de 3 à 7".into());
        }
        errors
    }
}

/// The id of the generated rooms that are all height begin so.
pub const Z: &str = "escalier-z-";
pub const TOUR: &str = "tour-";

/// The levels a room that is all height climbs (`None`: any other room). Such a room is only placed where the
/// dungeon goes down that many levels: one comes in by its top and leaves by its foot.
pub fn hauteur(room: &RoomTemplate) -> Option<i32> {
    (room.id.starts_with(Z) || room.id.starts_with(TOUR)).then(|| room.plan.cells.keys().map(|c| c.level).max().unwrap_or(0))
}

/// A room being drawn: `interior` cells wide and deep, walls around, `levels` levels above the lowest.
struct Dessin {
    plan: Plan,
    width: i32,
    depth: i32,
    levels: i32,
}

impl Dessin {
    /// All floor on the lowest level, void above, walls around.
    fn new(width: i32, depth: i32, levels: i32) -> Self {
        let mut plan = Plan::default();
        for level in 0..=levels {
            for x in -1..=width {
                for y in -1..=depth {
                    let wall = x < 0 || y < 0 || x == width || y == depth;
                    let symbol = if wall { '#' } else if level == 0 { '.' } else { '_' };
                    plan.cells.insert(Cell::new(x + 1, y + 1, level), symbol);
                }
            }
        }
        Self { plan, width, depth, levels }
    }

    fn set(&mut self, (x, y): (i32, i32), level: i32, symbol: char) {
        if (-1..=self.width).contains(&x) && (-1..=self.depth).contains(&y) && (0..=self.levels).contains(&level) {
            self.plan.cells.insert(Cell::new(x + 1, y + 1, level), symbol);
        }
    }

    /// Stairs on `at`, climbing toward `(dx, dy)`, and the hole above them.
    fn stairs(&mut self, at: (i32, i32), level: i32, (dx, dy): (i32, i32)) {
        self.set(at, level, stairs_symbol(dx, dy));
        self.set(at, level + 1, '_');
    }

    /// The doors of an end of the Z stairs (the column `x`): north, south, and the end wall.
    fn landing_doors(&mut self, x: i32, level: i32) {
        self.set((x, -1), level, '+');
        self.set((x, self.depth), level, '+');
        self.set((if x == 0 { -1 } else { self.width }, 1), level, '+');
    }

    fn template(self, id: &str, weight: f64, rng: &mut ChaCha8Rng) -> Result<RoomTemplate, String> {
        let room = RoomTemplate::from_plan(id, vec![RoomRole::Hall], Vec::new(), Vec::new(), self.plan)?;
        Ok(RoomTemplate { weight, ..room.oriented(rng.random_range(0..8))? })
    }
}

/// The Z stairs climbing `levels` levels: 3 cells by 2; on an even level one climbs eastward along the north
/// wall, on an odd one westward along the south wall; the landings are the west and east ends, each with a door
/// on its three walls (never two doors side by side: the cell in front of each stays free for its corridor).
fn escalier_z(levels: i32) -> Dessin {
    let mut d = Dessin::new(3, 2, levels);
    for level in 0..=levels {
        let even = level % 2 == 0;
        // Where one stands on this level, and its doors.
        let x = if even { 0 } else { 2 };
        for y in 0..2 {
            d.set((x, y), level, '.');
        }
        d.landing_doors(x, level);
        if level < levels {
            if even {
                d.stairs((1, 0), level, (1, 0));
            } else {
                d.stairs((1, 1), level, (-1, 0));
            }
        }
    }
    // The lowest level is all floor: the doors of its east end too.
    d.landing_doors(2, 0);
    d
}

/// Icebreaker F.56 : the tower going down `levels` levels, `n` cells from wall to wall. Along the wall `s` (north,
/// east, south, west in turn, a level each), from a corner to the next: floor, the stairs in the middle, then
/// floor one level up to the corner, the landing. The lowest level is all floor: what falls lands there.
fn tour(levels: i32, n: i32, vide: bool) -> Dessin {
    let mut d = Dessin::new(n, n, levels);
    if !vide {
        for level in 0..=levels {
            for x in 1..n - 1 {
                for y in 1..n - 1 {
                    d.set((x, y), level, '#');
                }
            }
        }
    }
    let corner = |s: i32| [(0, 0), (n - 1, 0), (n - 1, n - 1), (0, n - 1)][s.rem_euclid(4) as usize];
    let way = |s: i32| [(1, 0), (0, 1), (-1, 0), (0, -1)][s.rem_euclid(4) as usize];
    let at = |s: i32, t: i32| (corner(s).0 + way(s).0 * t, corner(s).1 + way(s).1 * t);
    let middle = (n - 1) / 2;
    for level in 0..levels {
        for t in 0..middle {
            d.set(at(level, t), level, '.');
        }
        d.stairs(at(level, middle), level, way(level));
        for t in middle + 1..n {
            d.set(at(level, t), level + 1, '.');
        }
    }
    // The doors of a landing, on the two walls of its corner; the lowest level has those of its four corners.
    let mut doors = |s: i32, level: i32| {
        let ((x, y), (dx, dy), (px, py)) = (corner(s), way(s), way(s + 3));
        d.set((x - dx, y - dy), level, '+');
        d.set((x + px, y + py), level, '+');
    };
    for level in 0..=levels {
        doors(level, level);
    }
    for s in 1..4 {
        doors(s, 0);
    }
    d
}

impl RoomCatalog {
    /// Icebreaker F.55 : the catalog with the rooms that are all height. Same seed, same rooms; none asked for,
    /// none added and no draw.
    pub fn with_vertical(mut self, settings: &VerticalSettings, seed: u64) -> Result<Self, String> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x2E_57A1);
        let mut generated = Vec::new();
        if settings.escaliers_z > 0 {
            let heights = settings.z_etages.0..=settings.z_etages.1;
            // As the share of the generated shapes, against the weight of the rooms the catalog has; each height
            // weighs it all: only the one that fits the descent is drawn from.
            let drawn: f64 = self.with_role(RoomRole::Hall).iter().map(|t| t.weight).sum();
            let part = settings.escaliers_z.min(90) as f64 / 100.0;
            let weight = drawn.max(1.0) * part / (1.0 - part);
            for levels in heights {
                generated.push(escalier_z(levels).template(&format!("{Z}{levels}"), weight, &mut rng)?);
            }
        }
        // Icebreaker F.56 : a tower for each height the path may go down at once; as heavy as all the rooms of
        // passage (Z stairs of the same height share the descents with it).
        if settings.tours > 0 {
            let drawn: f64 = self.with_role(RoomRole::Hall).iter().map(|t| t.weight).sum();
            for levels in settings.tour_etages.0..=settings.tour_etages.1 {
                let vide = rng.random_range(0..100) < settings.tour_vide;
                let room = tour(levels, settings.tour_largeur, vide);
                generated.push(room.template(&format!("{TOUR}{levels}"), drawn.max(1.0), &mut rng)?);
            }
        }
        if !generated.is_empty() {
            self.rooms.extend(generated);
            self.rooms.sort_by(|a, b| a.id.cmp(&b.id));
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icebreaker::dungeon::plan;

    /// From any door of the room, every cell one stands on is reached, and one comes back.
    pub fn sound(room: &RoomTemplate) {
        let floors: Vec<Cell> = room
            .plan
            .cells
            .iter()
            .filter(|(_, s)| plan::is_walkable(**s) || plan::stairs_direction(**s).is_some())
            .map(|(c, _)| *c)
            .collect();
        assert!(room.door_count() >= 2, "{} : {} porte(s)", room.id, room.door_count());
        for door in room.doors() {
            let reached = room.plan.reachable(door.inside);
            for cell in &floors {
                assert!(reached.contains(cell), "{} : {cell:?} hors d'atteinte de {:?}", room.id, door.cell);
                assert!(room.plan.reachable(*cell).contains(&door.inside), "{} : pas de retour de {cell:?}", room.id);
            }
        }
        // A hole ends on something to stand on.
        for (cell, symbol) in &room.plan.cells {
            if *symbol == '_' {
                let mut below = *cell;
                while room.plan.get(below) == '_' {
                    below = below.offset(0, 0, -1);
                }
                let s = room.plan.get(below);
                assert!(plan::is_walkable(s) || plan::stairs_direction(s).is_some(), "{} : vide sans fond {cell:?}", room.id);
            }
        }
    }

    #[test]
    fn z_stairs_are_climbed_and_come_down() {
        let catalog = RoomCatalog { rooms: Vec::new() };
        let settings = VerticalSettings { escaliers_z: 30, z_etages: Range(1, 4), ..VerticalSettings::default() };
        let catalog = catalog.with_vertical(&settings, 7).unwrap();
        assert_eq!(catalog.rooms.len(), 4);
        for room in &catalog.rooms {
            sound(room);
            let levels = room.plan.cells.keys().map(|c| c.level).max().unwrap();
            // Doors on every level, the top one included.
            for level in 0..=levels {
                assert!(room.doors().any(|d| d.cell.level == level), "{} : pas de porte à l'étage {level}", room.id);
            }
        }
        assert!(RoomCatalog { rooms: Vec::new() }.with_vertical(&VerticalSettings::default(), 7).unwrap().rooms.is_empty());
    }

    /// In a dungeon: Z stairs are placed, they bring down from a level to the next, and the dungeon stays sound.
    #[test]
    fn z_stairs_join_the_levels_of_a_dungeon() {
        use crate::icebreaker::dungeon::{assemble::assemble, graph, settings::*};
        let mut settings = Settings::load().unwrap();
        settings.vertical.escaliers_z = 30;
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let (mut placed, mut used, mut dungeons) = (0, 0, 0);
        for seed in 0..30 {
            let catalog = settings.catalog(seed).unwrap();
            let graph = graph::generate(&resolved, seed);
            let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) else {
                continue;
            };
            assert!(dungeon.check().is_empty(), "graine {seed}");
            for room in dungeon.rooms.iter().filter(|r| r.template.id.starts_with(Z)) {
                placed += 1;
                // Doors opened on two levels at least: the room is climbed.
                let levels: std::collections::BTreeSet<i32> = room
                    .template
                    .doors()
                    .map(|d| room.to_dungeon(d.cell))
                    .filter(|c| dungeon.plan.get(*c) == 'D')
                    .map(|c| c.level)
                    .collect();
                used += usize::from(levels.len() >= 2);
            }
            dungeons += 1;
        }
        println!("{placed} escaliers en Z sur {dungeons} donjons, dont {used} montés d'un étage à l'autre");
        assert!(dungeons >= 27 && placed * 3 >= dungeons && used * 10 >= placed * 8);
    }

    /// Icebreaker F.56 : towers of every height and width, void or pillar, are climbed and come down.
    #[test]
    fn towers_are_climbed_and_come_down() {
        for (largeur, vide) in [(3, 0), (3, 100), (4, 100), (5, 0), (5, 100), (7, 100)] {
            let settings = VerticalSettings {
                tours: 50,
                tour_etages: Range(2, 6),
                tour_largeur: largeur,
                tour_vide: vide,
                ..VerticalSettings::default()
            };
            assert!(settings.errors().is_empty());
            let catalog = RoomCatalog { rooms: Vec::new() }.with_vertical(&settings, 3).unwrap();
            assert_eq!(catalog.rooms.len(), 5);
            for room in &catalog.rooms {
                sound(room);
                let levels = hauteur(room).unwrap();
                for level in 0..=levels {
                    assert!(room.doors().any(|d| d.cell.level == level), "{} : pas de porte à l'étage {level}", room.id);
                }
                // The middle: a void above the lowest level, or a pillar.
                let middle = room.plan.get(Cell::new((largeur + 2) / 2, (largeur + 2) / 2, 1));
                assert_eq!(middle, if vide == 100 { '_' } else { '#' }, "{}", room.id);
            }
        }
    }

    /// In a dungeon: the main path goes down several levels at once by a tower, entered by its top, left by its foot.
    #[test]
    fn towers_bring_down_several_levels() {
        use crate::icebreaker::dungeon::{assemble::assemble, graph, settings::*};
        let mut settings = Settings::load().unwrap();
        settings.vertical.tours = 100;
        settings.vertical.tour_largeur = 5;
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let (mut placed, mut used, mut dungeons, mut deepest) = (0, 0, 0, 0);
        for seed in 0..30 {
            let catalog = settings.catalog(seed).unwrap();
            let graph = graph::generate(&resolved, seed);
            assert!(graph.check().is_empty(), "graine {seed}");
            let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) else {
                continue;
            };
            assert!(dungeon.check().is_empty(), "graine {seed}");
            for room in dungeon.rooms.iter().filter(|r| r.template.id.starts_with(TOUR)) {
                placed += 1;
                let levels: std::collections::BTreeSet<i32> = room
                    .template
                    .doors()
                    .filter(|d| dungeon.plan.get(room.to_dungeon(d.cell)) == 'D')
                    .map(|d| d.cell.level)
                    .collect();
                let top = hauteur(&room.template).unwrap();
                used += usize::from(levels.contains(&0) && levels.contains(&top));
            }
            deepest = deepest.max(graph.rooms.iter().map(|r| r.level).max().unwrap());
            dungeons += 1;
        }
        println!("{placed} tours sur {dungeons} donjons, dont {used} prises du haut au pied ; jusqu'à {deepest} étages de fond");
        assert!(dungeons >= 24 && placed >= dungeons && used * 10 >= placed * 8 && deepest >= 6);
    }
}
