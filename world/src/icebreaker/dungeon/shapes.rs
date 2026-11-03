//! Icebreaker F.5: generated rooms of varied shapes, mixed with the drawn ones
//! (`docs/icebreaker/design/generation.md`, « Variété des salles »).
//! Translated from `RoomShapes.cs` of the Icebreaker project: an outline
//! (chamfered corners, L, T, cross, octagon, ellipse, cave) drawn at random,
//! walls around it (diagonals too), doors `+` in the straight walls, pillars
//! and chests by role, in an orientation at random. Each generated room passes
//! the same checks as a drawn one.

use super::{
    plan::{Cell, Plan},
    rooms::{RoomCatalog, RoomRole, RoomTemplate},
    settings::Range,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The outlines of the generated rooms.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    /// A rectangle with its corners cut.
    Chamfer,
    L,
    T,
    Cross,
    Octagon,
    /// A circle or an ellipse, by the proportions drawn.
    Ellipse,
    /// An irregular cave (a cellular automaton in an ellipse).
    Cave,
}

impl Shape {
    pub const ALL: [Shape; 7] = [
        Shape::Chamfer,
        Shape::L,
        Shape::T,
        Shape::Cross,
        Shape::Octagon,
        Shape::Ellipse,
        Shape::Cave,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Shape::Chamfer => "chamfer",
            Shape::L => "l",
            Shape::T => "t",
            Shape::Cross => "cross",
            Shape::Octagon => "octagon",
            Shape::Ellipse => "ellipse",
            Shape::Cave => "cave",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name().eq_ignore_ascii_case(name))
    }
}

/// What the generated rooms of a role get.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct ShapeRole {
    /// Pillars in the middle of the floor.
    pub pillars: Range,
    /// Chests (`$`) against a wall.
    pub chests: Range,
    /// Shapes allowed for this role (all those of `shapes` if absent).
    pub shapes: Option<Vec<Shape>>,
}

impl Default for ShapeRole {
    fn default() -> Self {
        Self {
            pillars: Range(0, 0),
            chests: Range(0, 0),
            shapes: None,
        }
    }
}

/// `generation.ron` → `variety.shapes`.
#[derive(Clone, Debug, Deserialize)]
pub struct ShapeSettings {
    /// Share aimed at of generated rooms among the rooms of a role (0: none,
    /// 0.5: as many as drawn ones).
    pub share: f64,
    /// Width and depth of the rooms, walls included, in cells.
    pub width: Range,
    pub depth: Range,
    pub doors: Range,
    /// Narrowest floor of a branch (L, T, cross) and of the passages of a cave.
    pub min_bar: i32,
    /// Weight of each shape in the draw.
    pub shapes: BTreeMap<String, u32>,
    /// Roles that get generated rooms, and what they get. The others only have
    /// drawn rooms.
    pub roles: BTreeMap<String, ShapeRole>,
}

impl ShapeSettings {
    pub fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !(0.0..=0.9).contains(&self.share) {
            errors.push(format!("share entre 0 et 0,9 (pas {})", self.share));
        }
        if self.min_bar < 2 {
            errors.push("min_bar vaut au moins 2".into());
        }
        let least = 2 * self.min_bar + 2;
        if self.width.0 < least
            || self.depth.0 < least
            || self.width.1 < self.width.0
            || self.depth.1 < self.depth.0
        {
            errors.push(format!(
                "width et depth : au moins {least} cases (deux branches de min_bar et leurs murs)"
            ));
        }
        if self.doors.0 < 1 || self.doors.1 < self.doors.0 {
            errors.push("doors : au moins une porte".into());
        }
        for name in self.shapes.keys() {
            if Shape::parse(name).is_none() {
                errors.push(format!("forme inconnue : {name}"));
            }
        }
        if self.shapes.values().sum::<u32>() == 0 {
            errors.push("shapes : des poids, au moins un non nul".into());
        }
        for (name, role) in &self.roles {
            if RoomRole::parse(name).is_none() {
                errors.push(format!("roles : rôle inconnu {name}"));
            }
            if let Some(shapes) = &role.shapes
                && !shapes.iter().any(|s| self.weight(*s) > 0)
            {
                errors.push(format!("roles.{name} : aucune forme permise de poids non nul"));
            }
        }
        errors
    }

    fn weight(&self, shape: Shape) -> u32 { self.shapes.get(shape.name()).copied().unwrap_or(0) }
}

const TRIES_PER_ROOM: usize = 30;

impl RoomCatalog {
    /// The catalog with, for each role of `settings.roles`, enough generated
    /// rooms to make the share `settings.share`. Same seed, same rooms.
    pub fn with_shapes(mut self, settings: &ShapeSettings, seed: u64) -> Result<Self, String> {
        if settings.share <= 0.0 {
            return Ok(self);
        }
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x5A11_E5);
        let mut generated = Vec::new();
        for (name, garnish) in &settings.roles {
            let role = RoomRole::parse(name).ok_or(format!("variety.shapes.roles : rôle inconnu {name}"))?;
            let drawn = self.with_role(role).len() as f64;
            let count = ((settings.share * drawn / (1.0 - settings.share)).round() as usize).max(1);
            for i in 0..count {
                generated.push(generate(&format!("forme-{name}-{i}"), role, garnish, settings, &mut rng)?);
            }
        }
        self.rooms.extend(generated);
        self.rooms.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(self)
    }
}

/// A generated room for `role`.
pub fn generate(
    id: &str,
    role: RoomRole,
    garnish: &ShapeRole,
    settings: &ShapeSettings,
    rng: &mut ChaCha8Rng,
) -> Result<RoomTemplate, String> {
    let shapes: Vec<Shape> = garnish
        .shapes
        .clone()
        .unwrap_or_else(|| Shape::ALL.to_vec())
        .into_iter()
        .filter(|s| settings.weight(*s) > 0)
        .collect();
    if shapes.is_empty() {
        return Err(format!("salles générées : aucune forme permise pour le rôle {}", role.name()));
    }
    let mut last = String::new();
    for _ in 0..TRIES_PER_ROOM {
        let shape = pick(&shapes, settings, rng);
        let Some(plan) = draw(shape, garnish, settings, rng) else {
            continue;
        };
        let id = format!("{id}-{}", shape.name());
        let room = RoomTemplate::from_plan(&id, vec![role], Vec::new(), Vec::new(), plan)
            .and_then(|room| room.oriented(rng.random_range(0..8)));
        match room {
            Ok(room) => return Ok(room),
            Err(e) => last = e,
        }
    }
    Err(format!("salles générées : aucune salle valide pour le rôle {} ({last})", role.name()))
}

fn pick(shapes: &[Shape], settings: &ShapeSettings, rng: &mut ChaCha8Rng) -> Shape {
    let total: u32 = shapes.iter().map(|s| settings.weight(*s)).sum();
    let mut roll = rng.random_range(0..total) as i64;
    for shape in shapes {
        roll -= settings.weight(*shape) as i64;
        if roll < 0 {
            return *shape;
        }
    }
    shapes[shapes.len() - 1]
}

/// A grid of characters, `' '` outside.
struct Grid {
    width: i32,
    depth: i32,
    cells: Vec<char>,
}

impl Grid {
    fn new(width: i32, depth: i32) -> Self {
        Self {
            width,
            depth,
            cells: vec![' '; (width * depth) as usize],
        }
    }

    fn at(&self, (x, y): (i32, i32)) -> char {
        if x < 0 || y < 0 || x >= self.width || y >= self.depth {
            ' '
        } else {
            self.cells[(y * self.width + x) as usize]
        }
    }

    fn set(&mut self, (x, y): (i32, i32), symbol: char) {
        self.cells[(y * self.width + x) as usize] = symbol;
    }

    fn all(&self) -> Vec<(i32, i32)> {
        (0..self.depth).flat_map(|y| (0..self.width).map(move |x| (x, y))).collect()
    }
}

const SIDES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

fn square((x, y): (i32, i32), radius: i32) -> impl Iterator<Item = (i32, i32)> {
    (-radius..=radius).flat_map(move |dx| (-radius..=radius).map(move |dy| (x + dx, y + dy)))
}

fn around(c: (i32, i32), radius: i32) -> impl Iterator<Item = (i32, i32)> {
    square(c, radius).filter(move |n| *n != c)
}

fn far(a: (i32, i32), b: (i32, i32), distance: i32) -> bool {
    (a.0 - b.0).abs() + (a.1 - b.1).abs() >= distance
}

/// Is `i` in the band of `thickness` cells centred in `size`?
fn band(i: i32, size: i32, thickness: i32) -> bool {
    i >= (size - thickness) / 2 && i < (size - thickness) / 2 + thickness
}

/// The plan of a room, or nothing if the draw could not place its doors.
fn draw(shape: Shape, garnish: &ShapeRole, settings: &ShapeSettings, rng: &mut ChaCha8Rng) -> Option<Plan> {
    // A cave needs room: the larger half of the sizes.
    let roll = |range: Range, rng: &mut ChaCha8Rng| {
        let low = if shape == Shape::Cave { (range.0 + range.1) / 2 } else { range.0 };
        rng.random_range(low..=range.1)
    };
    let (width, depth) = (roll(settings.width, rng), roll(settings.depth, rng));
    let floor = outline(shape, width - 2, depth - 2, settings.min_bar, rng);
    // A cave that came out as a plain rectangle is drawn again.
    if floor.is_rectangle() {
        return None;
    }

    // The floor is surrounded by a wall, diagonals too (no slit between two corners).
    let mut grid = Grid::new(width, depth);
    for (x, y) in grid.all() {
        if x > 0 && y > 0 && x < width - 1 && y < depth - 1 && floor.get((x - 1, y - 1)) {
            grid.set((x, y), '.');
        }
    }
    for c in grid.all() {
        if grid.at(c) == ' ' && around(c, 1).any(|n| grid.at(n) == '.') {
            grid.set(c, '#');
        }
    }

    // Doors: in a straight wall (walls on both sides), floor inside, outside
    // beyond; far from each other.
    let mut candidates = Vec::new();
    for (x, y) in grid.all() {
        if grid.at((x, y)) != '#' {
            continue;
        }
        for (dx, dy) in SIDES {
            let through = grid.at((x + dx, y + dy)) == ' ' && grid.at((x - dx, y - dy)) == '.';
            let straight = grid.at((x + dy, y + dx)) == '#' && grid.at((x - dy, y - dx)) == '#';
            let one_out = SIDES.iter().filter(|(nx, ny)| grid.at((x + nx, y + ny)) == ' ').count() == 1;
            if through && straight && one_out {
                candidates.push(((x, y), (dx, dy)));
            }
        }
    }
    let door_count = settings.doors.roll(rng).max(1) as usize;
    if candidates.len() < door_count {
        return None;
    }
    let mut doors = vec![candidates[rng.random_range(0..candidates.len())]];
    while doors.len() < door_count {
        let farthest = candidates
            .iter()
            .map(|c| {
                let distance = doors
                    .iter()
                    .map(|d| (c.0.0 - d.0.0).abs() + (c.0.1 - d.0.1).abs())
                    .min()
                    .unwrap_or(0);
                (distance as f64 + rng.random::<f64>(), *c)
            })
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, c)| c)?;
        doors.push(farthest);
    }
    for (cell, _) in &doors {
        grid.set(*cell, '+');
    }
    let thresholds: Vec<(i32, i32)> = doors.iter().map(|((x, y), (dx, dy))| (x - dx, y - dy)).collect();

    // Pillars: in the open floor, far from the doors and from each other.
    let mut pillars: Vec<(i32, i32)> = Vec::new();
    let wanted = garnish.pillars.roll(rng).max(0) as usize;
    for _ in 0..200 {
        if pillars.len() >= wanted {
            break;
        }
        let cell = (rng.random_range(0..width), rng.random_range(0..depth));
        if grid.at(cell) == '.'
            && around(cell, 1).all(|n| grid.at(n) == '.')
            && thresholds.iter().all(|t| far(*t, cell, 4))
            && pillars.iter().all(|p| far(*p, cell, 4))
        {
            pillars.push(cell);
            grid.set(cell, 'O');
        }
    }

    // Chests: against a wall, not in front of a door.
    let wanted = garnish.chests.roll(rng).max(0) as usize;
    let mut placed = 0;
    for _ in 0..200 {
        if placed >= wanted {
            break;
        }
        let cell = (rng.random_range(0..width), rng.random_range(0..depth));
        if grid.at(cell) == '.'
            && around(cell, 1).any(|n| grid.at(n) == '#')
            && thresholds.iter().all(|t| far(*t, cell, 3))
        {
            grid.set(cell, '$');
            placed += 1;
        }
    }

    let cells = grid
        .all()
        .into_iter()
        .filter(|c| grid.at(*c) != ' ')
        .map(|(x, y)| (Cell::new(x, y, 0), grid.at((x, y))))
        .collect();
    Some(Plan { cells })
}

/// A floor mask of `w` × `h` cells.
struct Mask {
    w: i32,
    h: i32,
    cells: Vec<bool>,
}

impl Mask {
    fn new(w: i32, h: i32, inside: impl Fn(i32, i32) -> bool) -> Self {
        let cells = (0..h).flat_map(|v| (0..w).map(move |u| (u, v))).map(|(u, v)| inside(u, v)).collect();
        Self { w, h, cells }
    }

    fn get(&self, (u, v): (i32, i32)) -> bool {
        u >= 0 && v >= 0 && u < self.w && v < self.h && self.cells[(v * self.w + u) as usize]
    }

    /// Does the floor fill its own frame?
    fn is_rectangle(&self) -> bool {
        let cells: Vec<(i32, i32)> =
            (0..self.h).flat_map(|v| (0..self.w).map(move |u| (u, v))).filter(|c| self.get(*c)).collect();
        let Some(first) = cells.first() else { return true };
        let (mut min, mut max) = (*first, *first);
        for (u, v) in &cells {
            min = (min.0.min(*u), min.1.min(*v));
            max = (max.0.max(*u), max.1.max(*v));
        }
        cells.len() as i32 == (max.0 - min.0 + 1) * (max.1 - min.1 + 1)
    }
}

/// The floor of the shape in a frame of `w` × `h` (inside the walls).
fn outline(shape: Shape, w: i32, h: i32, bar: i32, rng: &mut ChaCha8Rng) -> Mask {
    let (cx, cy) = ((w - 1) as f64 / 2.0, (h - 1) as f64 / 2.0);
    match shape {
        Shape::Chamfer | Shape::Octagon => {
            let cut = if shape == Shape::Octagon {
                w.min(h) / 3
            } else {
                rng.random_range(2..=3.max(w.min(h) / 4 + 1))
            };
            Mask::new(w, h, |u, v| u.min(w - 1 - u) + v.min(h - 1 - v) >= cut)
        },
        Shape::Ellipse => {
            let (rx, ry) = (w as f64 / 2.0, h as f64 / 2.0);
            Mask::new(w, h, |u, v| {
                ((u as f64 - cx) / rx).powi(2) + ((v as f64 - cy) / ry).powi(2) <= 1.0
            })
        },
        Shape::L => {
            let (tx, ty) = (rng.random_range(bar..=w - bar), rng.random_range(bar..=h - bar));
            Mask::new(w, h, |u, v| u < tx || v >= h - ty)
        },
        Shape::T => {
            let (ty, stem) = (rng.random_range(bar..=h - bar), rng.random_range(bar..=bar.max(w - 4)));
            Mask::new(w, h, |u, v| v < ty || band(u, w, stem))
        },
        Shape::Cross => {
            let (tx, ty) = (rng.random_range(bar..=bar.max(w - 4)), rng.random_range(bar..=bar.max(h - 4)));
            Mask::new(w, h, |u, v| band(u, w, tx) || band(v, h, ty))
        },
        Shape::Cave => cave(w, h, bar, rng),
    }
}

/// A cave: floor sown at random in an ellipse, smoothed (a cellular
/// automaton), then opened (erosion and dilation over a square of `bar` - 2
/// cells) so that no passage is too narrow; the largest piece in one go is kept.
fn cave(w: i32, h: i32, bar: i32, rng: &mut ChaCha8Rng) -> Mask {
    let (cx, cy) = ((w - 1) as f64 / 2.0, (h - 1) as f64 / 2.0);
    let (rx, ry) = (w as f64 / 2.0, h as f64 / 2.0);
    let in_ellipse = |u: i32, v: i32| ((u as f64 - cx) / rx).powi(2) + ((v as f64 - cy) / ry).powi(2) <= 1.0;
    let seeds: Vec<bool> = (0..w * h).map(|_| rng.random::<f64>() < 0.62).collect();
    let mut floor = Mask::new(w, h, |u, v| in_ellipse(u, v) && seeds[(v * w + u) as usize]);
    for _ in 0..4 {
        floor = Mask::new(w, h, |u, v| {
            in_ellipse(u, v) && square((u, v), 1).filter(|c| floor.get(*c)).count() >= 5
        });
    }
    let radius = 1.max((bar - 2) / 2);
    let eroded = Mask::new(w, h, |u, v| square((u, v), radius).all(|c| floor.get(c)));
    let opened = Mask::new(w, h, |u, v| square((u, v), radius).any(|c| eroded.get(c)));
    largest(&opened)
}

/// The largest piece of floor in one go (neighbours by the sides).
fn largest(floor: &Mask) -> Mask {
    let (w, h) = (floor.w, floor.h);
    let mut seen = vec![false; (w * h) as usize];
    let mut best: Vec<(i32, i32)> = Vec::new();
    for v in 0..h {
        for u in 0..w {
            if !floor.get((u, v)) || seen[(v * w + u) as usize] {
                continue;
            }
            let mut part = Vec::new();
            let mut queue = std::collections::VecDeque::from([(u, v)]);
            seen[(v * w + u) as usize] = true;
            while let Some(cell) = queue.pop_front() {
                part.push(cell);
                for (du, dv) in SIDES {
                    let next = (cell.0 + du, cell.1 + dv);
                    if floor.get(next) && !seen[(next.1 * w + next.0) as usize] {
                        seen[(next.1 * w + next.0) as usize] = true;
                        queue.push_back(next);
                    }
                }
            }
            if part.len() > best.len() {
                best = part;
            }
        }
    }
    Mask::new(w, h, |u, v| best.contains(&(u, v)))
}

#[cfg(test)]
mod tests {
    use super::{super::settings::Settings, *};

    #[test]
    fn every_role_gets_valid_rooms_of_every_shape() {
        let settings = Settings::load().unwrap();
        let shapes = &settings.variety.shapes;
        let mut seen = std::collections::BTreeSet::new();
        let widest = shapes.width.1.max(shapes.depth.1);
        for seed in 0..20 {
            let catalog = RoomCatalog::load().unwrap().with_shapes(shapes, seed).unwrap();
            for room in catalog.rooms.iter().filter(|r| r.id.starts_with("forme-")) {
                assert!(
                    room.door_count() >= shapes.doors.0 as usize,
                    "{} : {} porte(s)",
                    room.id,
                    room.door_count()
                );
                let (min_x, max_x, min_y, max_y) = room.plan.bounds().unwrap();
                let (w, d) = (max_x - min_x + 1, max_y - min_y + 1);
                assert!(w <= widest && d <= widest, "{} : {w} × {d}", room.id);
                // Not a plain rectangle: the floor does not fill its own frame, or a pillar stands inside.
                let floor: Vec<&Cell> = room.plan.cells.iter().filter(|(_, s)| **s != '#' && **s != '+').map(|(c, _)| c).collect();
                let (fw, fd) = (
                    floor.iter().map(|c| c.x).max().unwrap() - floor.iter().map(|c| c.x).min().unwrap() + 1,
                    floor.iter().map(|c| c.y).max().unwrap() - floor.iter().map(|c| c.y).min().unwrap() + 1,
                );
                let pillar = room.plan.cells.values().any(|s| *s == 'O');
                assert!(floor.len() < (fw * fd) as usize || pillar, "{} : un rectangle", room.id);
                seen.insert(room.roles[0]);
            }
        }
        for name in shapes.roles.keys() {
            assert!(seen.contains(&RoomRole::parse(name).unwrap()), "{name} : aucune salle générée");
        }
        // Each shape draws something, alone.
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        for shape in Shape::ALL {
            let mask = outline(shape, 10, 8, shapes.min_bar, &mut rng);
            assert!(mask.cells.iter().filter(|c| **c).count() >= 12, "{shape:?} : trop peu de sol");
        }
    }
}
