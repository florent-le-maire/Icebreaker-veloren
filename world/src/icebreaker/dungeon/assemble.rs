//! Icebreaker: assembly, from a mission graph to a plan
//! (`docs/icebreaker/design/generation.md`, « Assemblage du donjon »).
//! Translated from `DungeonAssembler.cs` of the Icebreaker project, without
//! the shared walls of castles; the safe room off the entrance is ours (F.31).
//!
//! The rooms of the tree (main path and branches) are placed one by one from
//! the entrance: a template drawn among those of the role, placed near a free
//! door of the parent room, then a corridor dug by a 3D path search (A*) that
//! knows how to lay stairs. The depth in the graph gives the level (depth 1 →
//! level -1). Loops and drops are dug afterwards between rooms already placed;
//! a drop is a one-way shaft. Without room, they are given up (the dungeon
//! stays finishable without them). If a room of the tree finds no place, the
//! attempt starts over (same seed, further draws).

use super::{
    graph::{MissionGraph, Passage, PassageKind},
    plan::{self, Cell, DIRECTIONS, Plan},
    remplissage,
    rooms::{RoomCatalog, RoomRole, RoomTemplate},
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use super::silhouette::Silhouette;
use serde::Deserialize;
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
};

/// `generation.ron` → `assembly`.
#[derive(Clone, Debug, Deserialize)]
pub struct AssemblySettings {
    /// Complete attempts before giving up.
    pub attempts: usize,
    /// Tries to place a room.
    pub placement_tries: usize,
    /// Tries for a loop or a drop (given up otherwise).
    pub extra_passage_tries: usize,
    /// Cells at most of the corridor of a loop or a drop (Icebreaker F.2: a loop
    /// that would be a long detour is given up instead).
    pub max_extra_length: usize,
    /// Cells between a door and the next room.
    pub spacing: super::settings::Range,
    /// Cells examined by a corridor search.
    pub search_limit: usize,
    /// Margin around the dungeon where a corridor may go.
    pub margin: i32,
    /// Icebreaker : each dungeon has a dominant theme (drawn from its seed); a room of that theme weighs this
    /// many times more than one without (1 = no theme).
    #[serde(default = "default_theme_bias")]
    pub theme_bias: f64,
    /// Icebreaker F.46 : the outline rooms and corridors stay inside (`Libre`: none), and how many cells it
    /// reaches on each side of its middle for a dungeon of `OUTLINE_ROOMS` rooms (F.49: more for a bigger one,
    /// less for a smaller one, as the square root of its rooms: the same crowding in every size).
    #[serde(default)]
    pub silhouette: Silhouette,
    #[serde(default = "default_etendue")]
    pub etendue: i32,
    /// Icebreaker F.47 : how much the corridors wander, from 0 (the shortest way) to 100 (a maze): the cost of a
    /// step grows with a noise drawn for each cell of the dungeon.
    #[serde(default)]
    pub sinuosite: i32,
    /// Icebreaker F.47 : corridors that lead nowhere, dug from the other corridors.
    #[serde(default = "no_dead_end")]
    pub impasses: super::settings::Range,
    /// Icebreaker F.48 : chances out of 100 that a room is put right against the one it comes from, door to
    /// door, without a corridor (the « salles collées » of `design/generation.md`); when it does not fit, a
    /// corridor as usual.
    #[serde(default)]
    pub collees: i32,
    /// Icebreaker F.52 : the maze dug in the rock left inside the outline once the mission is assembled
    /// (`remplissage.rs`): the share of that rock it takes out of 100 (0: none), the walls opened between two of
    /// its corridors for 100 cells of maze, and its dead ends filled again out of 100.
    #[serde(default)]
    pub remplissage: i32,
    #[serde(default)]
    pub boucles_labyrinthe: i32,
    #[serde(default)]
    pub elagage: i32,
    /// Icebreaker F.53 : rooms out of the mission (dead ends, drawn as the rooms of a branch), put far from the
    /// room they hang on, where the outline is still empty.
    #[serde(default = "no_dead_end")]
    pub salles_en_plus: super::settings::Range,
}

fn no_dead_end() -> super::settings::Range { super::settings::Range(0, 0) }

/// Icebreaker F.48 : rooms and doors tried to put a room against the one it comes from.
const GLUE_TRIES: usize = 12;

/// Icebreaker F.47 : cells of a dead end, and places tried for each.
const DEAD_END_LENGTH: (i32, i32) = (2, 5);
const DEAD_END_TRIES: usize = 30;

fn default_etendue() -> i32 { 40 }

fn default_theme_bias() -> f64 { 1.0 }

/// Icebreaker : the dominant theme of the dungeon of `seed` (none if no room names a theme).
pub fn dominant_theme(catalog: &RoomCatalog, seed: u64) -> Option<String> {
    let themes: Vec<String> = catalog.themes().into_iter().collect();
    if themes.is_empty() {
        return None;
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(29));
    Some(themes[rng.random_range(0..themes.len())].clone())
}

/// A room of the graph placed in the dungeon: cell (0, 0, 0) of its template
/// is at `origin`.
#[derive(Clone, Debug)]
pub struct PlacedRoom {
    pub graph_room: usize,
    pub template: RoomTemplate,
    pub origin: Cell,
}

impl PlacedRoom {
    pub fn to_dungeon(&self, c: Cell) -> Cell {
        Cell::new(self.origin.x + c.x, self.origin.y + c.y, self.origin.level + c.level)
    }

    /// The cells of the room (all that is not empty in its plan).
    pub fn footprint(&self) -> impl Iterator<Item = Cell> + '_ {
        self.template.plan.cells.keys().map(|c| self.to_dungeon(*c))
    }

    /// Icebreaker F.31 : the open floor cell nearest to the middle of the room (where one arrives in the safe room).
    pub fn middle(&self) -> Cell { self.to_dungeon(self.template.middle()) }
}

/// The result of the assembly.
#[derive(Clone, Debug)]
pub struct Assembled {
    pub plan: Plan,
    /// Icebreaker F.31 : where the party arrives, a floor cell of the safe room.
    pub arrival: Cell,
    /// Placed rooms, indexed by room of the graph.
    pub rooms: Vec<PlacedRoom>,
    /// Locked doors: the door, the key it needs.
    pub locks: Vec<(Cell, usize)>,
    /// Keys lying on a floor: the cell, the lock it opens.
    pub keys: Vec<(Cell, usize)>,
    /// The passages of the graph really dug.
    pub passages: Vec<Passage>,
    /// Attempts it took (1 = at once).
    pub attempts: usize,
    /// Icebreaker F.46 : the middle of the outline (`assembly.silhouette`), the entrance being at (0, 0), and
    /// the cells from it to its edge.
    pub centre: (i32, i32),
    pub etendue: i32,
    /// Icebreaker F.52 : the cells of the maze that fills the outline.
    pub labyrinthe: BTreeSet<Cell>,
}

impl Assembled {
    /// Checks the plan cell by cell: leaving the safe room and picking the keys
    /// up, one reaches the whole dungeon (locked doors closed without their
    /// key); from any cell, one can come back to the safe room.
    pub fn check(&self) -> Vec<String> {
        let plan = &self.plan;
        let start = self.arrival;
        let search = |next: &dyn Fn(Cell) -> Vec<Cell>| {
            let mut seen = BTreeSet::from([start]);
            let mut queue = VecDeque::from([start]);
            while let Some(cell) = queue.pop_front() {
                for n in next(cell) {
                    if seen.insert(n) {
                        queue.push_back(n);
                    }
                }
            }
            seen
        };
        let mut keys = BTreeSet::new();
        let reached = loop {
            let closed: BTreeSet<Cell> = self
                .locks
                .iter()
                .filter(|(_, lock)| !keys.contains(lock))
                .map(|(door, _)| *door)
                .collect();
            let reached = search(&|cell| {
                plan.moves(cell).into_iter().filter(|n| !closed.contains(n)).collect()
            });
            let before = keys.len();
            keys.extend(self.keys.iter().filter(|(c, _)| reached.contains(c)).map(|(_, k)| *k));
            if keys.len() == before {
                break reached;
            }
        };
        let mut errors = Vec::new();
        let practicable: Vec<Cell> = plan
            .cells
            .iter()
            .filter(|(_, s)| plan::is_walkable(**s) || plan::stairs_direction(**s).is_some())
            .map(|(c, _)| *c)
            .collect();
        let unreached: Vec<_> = practicable.iter().filter(|c| !reached.contains(c)).collect();
        if let Some(first) = unreached.first() {
            errors.push(format!("{} case(s) inaccessible(s), par ex. {first:?}", unreached.len()));
        }
        let mut reverse: BTreeMap<Cell, Vec<Cell>> = BTreeMap::new();
        for cell in &practicable {
            for next in plan.moves(*cell) {
                reverse.entry(next).or_default().push(*cell);
            }
        }
        let can_return = search(&|cell| reverse.get(&cell).cloned().unwrap_or_default());
        let trapped: Vec<_> = practicable.iter().filter(|c| !can_return.contains(c)).collect();
        if let Some(first) = trapped.first() {
            errors.push(format!(
                "{} case(s) d'où l'on ne peut pas revenir à l'entrée, par ex. {first:?}",
                trapped.len()
            ));
        }
        errors
    }

    /// Icebreaker F.51 : how much of its outline the dungeon takes, out of 100: the cells of the outline that hold
    /// something (room, corridor, wall) on a level at least, and the width taken (both ways) of the width of the
    /// outline. `None` without an outline.
    pub fn couverture(&self, silhouette: Silhouette) -> Option<(i32, i32)> {
        if silhouette == Silhouette::Libre {
            return None;
        }
        let ((cx, cy), r) = (self.centre, self.etendue);
        let taken: BTreeSet<(i32, i32)> = self.plan.cells.keys().map(|c| (c.x - cx, c.y - cy)).collect();
        let outline: Vec<(i32, i32)> = (-r..=r)
            .flat_map(|x| (-r..=r).map(move |y| (x, y)))
            .filter(|(x, y)| silhouette.dedans(*x, *y, r))
            .collect();
        let span = |cells: &mut dyn Iterator<Item = (i32, i32)>| {
            let cells: Vec<_> = cells.collect();
            let side = |of: fn(&(i32, i32)) -> i32| {
                let (min, max) = (cells.iter().map(of).min(), cells.iter().map(of).max());
                min.zip(max).map_or(0, |(min, max)| max - min + 1)
            };
            side(|c| c.0) + side(|c| c.1)
        };
        let inside = outline.iter().filter(|c| taken.contains(c)).count() as i32;
        let width = span(&mut taken.iter().copied());
        let whole = span(&mut outline.iter().copied());
        Some((100 * inside / (outline.len() as i32).max(1), (100 * width / whole.max(1)).min(100)))
    }
}

/// Assembles `graph`; `None` if no attempt succeeded. `log` gets the reason of
/// each failed attempt.
pub fn assemble(
    graph: &MissionGraph,
    catalog: &RoomCatalog,
    seed: u64,
    settings: &AssemblySettings,
    mut log: impl FnMut(String),
) -> Option<Assembled> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let theme = dominant_theme(catalog, seed);
    for attempt in 1..=settings.attempts {
        let mut state = Attempt::new(graph, catalog, &mut rng, settings, theme.clone(), seed);
        match state.run() {
            Err(failure) => log(format!("tentative {attempt} : {failure}")),
            Ok(()) => {
                let dungeon = state.finish(attempt);
                let errors = dungeon.check();
                if errors.is_empty() {
                    return Some(dungeon);
                }
                log(format!("tentative {attempt} : {}", errors.join(" ; ")));
            },
        }
    }
    None
}

struct Attempt<'a> {
    graph: &'a MissionGraph,
    catalog: &'a RoomCatalog,
    rng: &'a mut ChaCha8Rng,
    settings: &'a AssemblySettings,
    grid: Plan,
    placed: BTreeMap<usize, PlacedRoom>,
    used_connectors: BTreeSet<Cell>,
    locks: Vec<(Cell, usize)>,
    passages: Vec<Passage>,
    tree_degree: Vec<usize>,
    degree: Vec<usize>,
    theme: Option<String>,
    placement_failure: String,
    /// Icebreaker F.46 : the middle of the outline, and the cells from it to its edge for this dungeon.
    centre: (i32, i32),
    etendue: i32,
    /// Icebreaker F.47 : the seed of the dungeon (the noise of the corridors), the cells of the rooms and the floor
    /// cells of the corridors (where a dead end may start).
    seed: u64,
    in_rooms: BTreeSet<Cell>,
    corridors: Vec<Cell>,
    /// Icebreaker F.52 : the cells of the maze.
    labyrinthe: BTreeSet<Cell>,
}

/// Icebreaker F.46 : inside an outline, times more tries to place a room, and cells farther it may end up from the
/// door it comes from.
const OUTLINE_TRIES: usize = 3;
const OUTLINE_SLACK: usize = 14;
/// Icebreaker F.53 : without an outline, cells a room out of the mission may end up from the door it comes from.
const FAR_SLACK: i32 = 20;

/// Icebreaker F.49 : the rooms of the dungeon `assembly.etendue` is given for, and the widest outline (the dungeon
/// must hold in the area kept for it, `placement.emprise`).
const OUTLINE_ROOMS: f32 = 18.0;
pub const OUTLINE_WIDEST: i32 = 56;

/// A free door of a placed room: the door, the free cell in front, the way out.
type FreeDoor = (Cell, Cell, (i32, i32));

fn template_size(t: &RoomTemplate) -> (i32, i32) {
    let (_, max_x, _, max_y) = t.plan.bounds().unwrap_or_default();
    (max_x + 1, max_y + 1)
}

impl<'a> Attempt<'a> {
    fn new(
        graph: &'a MissionGraph,
        catalog: &'a RoomCatalog,
        rng: &'a mut ChaCha8Rng,
        settings: &'a AssemblySettings,
        theme: Option<String>,
        seed: u64,
    ) -> Self {
        let crowd = (graph.rooms.len() as f32 / OUTLINE_ROOMS).sqrt();
        Self {
            etendue: ((settings.etendue as f32 * crowd).round() as i32).clamp(8, OUTLINE_WIDEST),
            seed,
            in_rooms: BTreeSet::new(),
            corridors: Vec::new(),
            labyrinthe: BTreeSet::new(),
            theme,
            graph,
            catalog,
            rng,
            settings,
            grid: Plan::default(),
            placed: BTreeMap::new(),
            used_connectors: BTreeSet::new(),
            locks: Vec::new(),
            passages: Vec::new(),
            tree_degree: vec![0; graph.rooms.len()],
            degree: vec![0; graph.rooms.len()],
            placement_failure: String::new(),
            centre: (0, 0),
        }
    }

    /// Icebreaker F.46 : whether the cell is inside the outline of the dungeon (the same on every level).
    fn dedans(&self, c: Cell) -> bool {
        self.settings.silhouette.dedans(c.x - self.centre.0, c.y - self.centre.1, self.etendue)
    }

    /// Icebreaker F.46 : lays the outline around the entrance (at the origin): its middle as near the middle of
    /// the entrance as the shape allows (a ring has no middle to stand in), the whole entrance inside; the side is
    /// drawn. The dungeon then spreads from the middle, and stays within `etendue` cells or so of where the party
    /// arrives (the area kept for it, `placement.emprise`). No outline: nothing drawn, the dungeon of the seed is
    /// the one it was.
    fn place_outline(&mut self, entrance: &RoomTemplate) -> bool {
        if self.settings.silhouette == Silhouette::Libre {
            return true;
        }
        let (width, depth) = template_size(entrance);
        let first = self.rng.random_range(0..DIRECTIONS.len());
        for away in 0..=self.etendue {
            for turn in 0..DIRECTIONS.len() {
                let (dx, dy) = DIRECTIONS[(first + turn) % DIRECTIONS.len()];
                let at = |away: i32| (width / 2 + dx * away, depth / 2 + dy * away);
                self.centre = at(away);
                if entrance.plan.cells.keys().all(|c| self.dedans(*c)) {
                    // Not in the middle (a ring): a little farther in, off the edge it has just cleared.
                    if away > 0 {
                        self.centre = at(away + self.etendue / 4);
                        if !entrance.plan.cells.keys().all(|c| self.dedans(*c)) {
                            self.centre = at(away);
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    fn pick<T: Clone>(&mut self, items: &[T]) -> T { items[self.rng.random_range(0..items.len())].clone() }

    /// A free door of the parent to go on from. Icebreaker F.46 : inside an outline, rather one with room ahead
    /// (free cells straight on, squared), so the dungeon turns at the edge and fills the shape instead of failing
    /// against it.
    fn pick_door(&mut self, doors: &[FreeDoor]) -> FreeDoor {
        if self.settings.silhouette == Silhouette::Libre {
            return self.pick(doors);
        }
        let reach = 2 * self.etendue;
        let weights: Vec<i32> = doors
            .iter()
            .map(|(_, outside, (dx, dy))| {
                let run = (0..reach)
                    .take_while(|k| {
                        let c = outside.offset(dx * k, dy * k, 0);
                        self.dedans(c) && self.grid.is_free(c)
                    })
                    .count() as i32;
                run * run + 1
            })
            .collect();
        let mut roll = self.rng.random_range(0..weights.iter().sum::<i32>());
        for (door, weight) in doors.iter().zip(&weights) {
            roll -= weight;
            if roll < 0 {
                return *door;
            }
        }
        doors[doors.len() - 1]
    }

    /// Icebreaker : a template drawn by weight (`@poids`), the rooms of the dominant theme of the dungeon
    /// weighing `theme_bias` times more.
    fn pick_template(&mut self, items: &[&'a RoomTemplate]) -> &'a RoomTemplate {
        let weight = |t: &RoomTemplate| {
            let themed = self.theme.as_ref().is_some_and(|theme| t.themes.contains(theme));
            t.weight * if themed { self.settings.theme_bias.max(0.0) } else { 1.0 }
        };
        let total: f64 = items.iter().map(|t| weight(t)).sum();
        if total <= 0.0 {
            return self.pick(items);
        }
        let mut roll = self.rng.random_range(0.0..total);
        for t in items {
            roll -= weight(t);
            if roll < 0.0 {
                return t;
            }
        }
        items[items.len() - 1]
    }

    /// Templates of the role with enough doors for the passages of the tree
    /// (otherwise those with the most), and preferably for the loops and
    /// drops too.
    fn templates_for(&self, room: usize) -> Vec<&'a RoomTemplate> {
        let all = self.catalog.with_role(self.graph.rooms[room].role);
        let enough: Vec<_> = all.iter().copied().filter(|t| t.door_count() >= self.tree_degree[room]).collect();
        if enough.is_empty() {
            let most = all.iter().map(|t| t.door_count()).max().unwrap_or(0);
            return all.into_iter().filter(|t| t.door_count() == most).collect();
        }
        let ideal: Vec<_> = enough.iter().copied().filter(|t| t.door_count() >= self.degree[room]).collect();
        if ideal.is_empty() { enough } else { ideal }
    }

    fn run(&mut self) -> Result<(), String> {
        let (tree, extras) = self.split_tree();
        for p in &tree {
            self.tree_degree[p.from] += 1;
            self.tree_degree[p.to] += 1;
        }
        for p in &self.graph.passages {
            self.degree[p.from] += 1;
            self.degree[p.to] += 1;
        }
        // The entrance first.
        let templates = self.templates_for(0);
        if templates.is_empty() {
            return Err("aucune salle d'entrée".into());
        }
        let template = self.pick_template(&templates);
        if !self.place_outline(template) {
            return Err("l'entrée ne tient pas dans la silhouette".into());
        }
        self.write(PlacedRoom {
            graph_room: 0,
            template: template.clone(),
            origin: Cell::new(0, 0, 0),
        });

        // The tree: each passage links a placed room to a room still to place.
        // Icebreaker F.53 : the rooms out of the mission wait for the others to be placed.
        let mut later = Vec::new();
        let mut queue = VecDeque::from([0]);
        while let Some(parent) = queue.pop_front() {
            for passage in tree.iter().filter(|p| p.from == parent || p.to == parent) {
                let child = passage.other(parent);
                if self.placed.contains_key(&child) {
                    continue;
                }
                if self.graph.rooms[child].en_plus {
                    later.push((parent, child, passage.clone()));
                    continue;
                }
                if !self.place_child(parent, child, passage, false) {
                    return Err(format!(
                        "pas de place pour la salle {child} ({}) après la salle {parent} : {}",
                        self.graph.rooms[child].role.name(),
                        self.placement_failure
                    ));
                }
                self.passages.push(passage.clone());
                queue.push_back(child);
            }
        }

        for passage in extras {
            if self.connect_extra(&passage) {
                self.passages.push(passage);
            }
        }
        // Icebreaker F.53 : far from the room it hangs on, else next to it as any room.
        for (parent, child, passage) in later {
            if !self.place_child(parent, child, &passage, true) && !self.place_child(parent, child, &passage, false) {
                return Err(format!("pas de place pour la salle en plus {child} : {}", self.placement_failure));
            }
            self.passages.push(passage);
        }
        // Icebreaker F.47 : nothing drawn when there is none: the dungeon of the seed is the one it was.
        if self.settings.impasses.1 > 0 {
            for _ in 0..self.settings.impasses.roll(self.rng) {
                self.dig_dead_end();
            }
        }
        // Icebreaker F.52 : the same: without filling, no draw.
        if self.settings.remplissage > 0 {
            self.fill();
        }
        Ok(())
    }

    /// Icebreaker F.52 : the maze in the rock left inside the outline, from the corridors (`remplissage.rs`). The
    /// more the corridors wander (`sinuosite`), the longer its own.
    fn fill(&mut self) {
        let (silhouette, (cx, cy), r) = (self.settings.silhouette, self.centre, self.etendue);
        let reglages = remplissage::Reglages {
            part: self.settings.remplissage,
            suite: 50 + self.settings.sinuosite.clamp(0, 100) / 2,
            boucles: self.settings.boucles_labyrinthe,
            elagage: self.settings.elagage,
        };
        // Without an outline, the maze stays within the dungeon as it is.
        let boite = match (silhouette, self.grid.bounds()) {
            (Silhouette::Libre, Some(bounds)) => bounds,
            _ => (cx - r, cx + r, cy - r, cy + r),
        };
        self.labyrinthe = remplissage::creuser(
            &mut self.grid,
            &self.in_rooms,
            &self.corridors,
            &|c| silhouette.dedans(c.x - cx, c.y - cy, r),
            boite,
            reglages,
            self.rng,
        );
    }

    /// Icebreaker F.47 : a corridor that leads nowhere, straight from a cell of another corridor through its wall.
    /// It touches nothing one walks on but the cell it starts from (no way around a locked door, no hole above):
    /// given up if no place is found.
    fn dig_dead_end(&mut self) -> bool {
        for _ in 0..DEAD_END_TRIES {
            if self.corridors.is_empty() {
                return false;
            }
            let from = self.pick(&self.corridors.clone());
            let (dx, dy) = self.pick(&DIRECTIONS);
            let length = self.rng.random_range(DEAD_END_LENGTH.0..=DEAD_END_LENGTH.1);
            let cells: Vec<Cell> = (1..=length).map(|k| from.offset(dx * k, dy * k, 0)).collect();
            // Rock, or the wall of a corridor (never of a room).
            let solid = |c: Cell| self.grid.is_free(c) || (self.grid.get(c) == '#' && !self.in_rooms.contains(&c));
            let quiet = |c: Cell| self.grid.is_free(c) || self.grid.get(c) == '#';
            let fine = cells.iter().enumerate().all(|(i, c)| {
                let before = if i == 0 { from } else { cells[i - 1] };
                self.dedans(*c)
                    && solid(*c)
                    && self.grid.get(c.offset(0, 0, 1)) != '_'
                    && DIRECTIONS.iter().all(|(nx, ny)| {
                        let n = c.offset(*nx, *ny, 0);
                        n == before || quiet(n)
                    })
            });
            if fine {
                let corridor: Vec<(Cell, char)> = cells.into_iter().map(|c| (c, '.')).collect();
                self.carve(&corridor);
                return true;
            }
        }
        false
    }

    /// Icebreaker F.47 : what a step into `cell` costs more than 1, for a corridor that wanders: a noise of the
    /// cell (0 to 3, the same all along the assembly of this seed) times the sinuosity.
    fn detour(&self, cell: Cell) -> i32 {
        if self.settings.sinuosite <= 0 {
            return 0;
        }
        let mut h = self.seed ^ 0x51_4E_05;
        for v in [cell.x, cell.y, cell.level] {
            h = (h ^ v as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h ^= h >> 29;
        }
        (h % 4) as i32 * self.settings.sinuosite / 25
    }

    /// Spanning tree = passages in the order of the graph (main path and
    /// branches first), without the drops.
    fn split_tree(&self) -> (Vec<Passage>, Vec<Passage>) {
        let mut group: Vec<usize> = (0..self.graph.rooms.len()).collect();
        fn find(group: &mut [usize], r: usize) -> usize {
            if group[r] != r {
                group[r] = find(group, group[r]);
            }
            group[r]
        }
        let (mut tree, mut extras) = (Vec::new(), Vec::new());
        for p in &self.graph.passages {
            let (a, b) = (find(&mut group, p.from), find(&mut group, p.to));
            if p.kind != PassageKind::Drop && a != b {
                group[a] = b;
                tree.push(p.clone());
            } else {
                extras.push(p.clone());
            }
        }
        (tree, extras)
    }

    /// `far` (Icebreaker F.53) : anywhere ahead of a door of the parent, as far as the outline goes.
    fn place_child(&mut self, parent: usize, child: usize, passage: &Passage, far: bool) -> bool {
        let mut templates = self.templates_for(child);
        // Icebreaker F.55 : a room that is all height, only where the dungeon goes down as many levels as it climbs.
        let descent = self.graph.rooms[child].level - self.graph.rooms[parent].level;
        templates.retain(|t| super::vertical::hauteur(t).is_none_or(|levels| levels == descent));
        // Icebreaker F.56 : several levels at once: by a tower (or Z stairs) of that height when there is one.
        if descent >= 2 && matches!(self.graph.rooms[child].role, RoomRole::Hall | RoomRole::GuardRoom) {
            let tall: Vec<&'a RoomTemplate> =
                self.catalog.rooms.iter().filter(|t| super::vertical::hauteur(t) == Some(descent)).collect();
            if !tall.is_empty() {
                templates = tall;
            }
        }
        if templates.is_empty() {
            self.placement_failure = "aucune salle pour ce rôle".into();
            return false;
        }
        let (mut overlaps, mut no_corridor) = (0, 0);
        let level = -self.graph.rooms[child].level;
        // Icebreaker F.48 : nothing drawn at 0: the dungeon of the seed is the one it was.
        if !far
            && self.settings.collees > 0
            && self.rng.random_range(0..100) < self.settings.collees
            && self.glue_child(parent, child, passage, &templates, level)
        {
            return true;
        }
        // Icebreaker F.46 : inside an outline there is less room: more tries, and try after try the room may sit
        // farther from the door it comes from, ahead and aside (a longer corridor, that bends), up to
        // `OUTLINE_SLACK` cells: it ends up where there is room left.
        let outlined = self.settings.silhouette != Silhouette::Libre;
        let tries = self.settings.placement_tries * if outlined || far { OUTLINE_TRIES } else { 1 };
        for attempt in 0..tries {
            let slack = match (far, outlined) {
                (true, true) => self.etendue,
                (true, false) => FAR_SLACK,
                (false, true) => (attempt * OUTLINE_SLACK / tries) as i32,
                (false, false) => 0,
            };
            let mut from: Vec<FreeDoor> = self.free_connectors(&self.placed[&parent]).collect();
            // Icebreaker F.55 : out of a room that is all height, rather by a door on the level of the child.
            if super::vertical::hauteur(&self.placed[&parent].template).is_some()
                && from.iter().any(|(_, outside, _)| outside.level == level)
            {
                from.retain(|(_, outside, _)| outside.level == level);
            }
            if from.is_empty() {
                self.placement_failure = format!(
                    "plus de porte libre (après {overlaps} chevauchement(s), {no_corridor} couloir(s) \
                     impossible(s))"
                );
                return false;
            }
            let (from_door, start, (dx, dy)) = self.pick_door(&from);

            // A door facing the parent's when there is one (Icebreaker F.2: the room
            // then lies just beyond it, and turned rooms nearly always have one), and
            // among those, one at the parent's level when there is one: a flat
            // corridor, the stairs of the room do the descent.
            let template = self.pick_template(&templates);
            // Icebreaker F.55 : one comes into a room that is all height by its top.
            let top = super::vertical::hauteur(template);
            let doors: Vec<_> =
                template.doors().copied().filter(|c| top.is_none_or(|top| c.cell.level == top)).collect();
            let facing: Vec<_> = doors.iter().copied().filter(|c| c.direction() == (-dx, -dy)).collect();
            let candidates = if facing.is_empty() { doors } else { facing };
            let flat: Vec<_> =
                candidates.iter().copied().filter(|c| level + c.outside.level == start.level).collect();
            let to = self.pick(if flat.is_empty() { &candidates } else { &flat });
            let faces = to.direction() == (-dx, -dy);

            // Where the front of the child's door should be: facing the parent's door.
            // A door that does not face: the room would stretch toward the parent, put it
            // farther (the corridor goes around).
            let (width, depth) = template_size(template);
            let ahead = self.settings.spacing.roll(self.rng)
                + 2 * (start.level - (level + to.outside.level)).abs()
                + if faces { 0 } else { width.max(depth) }
                + if slack > 0 { self.rng.random_range(0..=slack) } else { 0 };
            let side = self.rng.random_range(-1 - slack..=1 + slack);
            let goal = Cell::new(
                start.x + dx * ahead - dy * side,
                start.y + dy * ahead + dx * side,
                level + to.outside.level,
            );
            let origin = Cell::new(goal.x - to.outside.x, goal.y - to.outside.y, level);
            let room = PlacedRoom {
                graph_room: child,
                template: template.clone(),
                origin,
            };
            if !self.fits(&room)
                || !self.grid.is_free(goal)
                || !self.dedans(goal)
                || self.reserved().contains(&goal)
                || room.footprint().any(|c| c == goal)
            {
                overlaps += 1;
                continue;
            }

            let child_door = room.to_dungeon(to.cell);
            let written = self.write(room);
            let Some(corridor) = self.route(start, goal, false) else {
                self.erase(&written, child);
                no_corridor += 1;
                continue;
            };
            self.carve(&corridor);
            self.open_door(from_door);
            self.open_door(child_door);
            if let Some(lock) = passage.lock {
                self.locks.push((child_door, lock));
            }
            return true;
        }
        self.placement_failure =
            format!("{overlaps} chevauchement(s), {no_corridor} couloir(s) impossible(s)");
        false
    }

    /// Icebreaker F.48 : the child right against its parent: its door on the cell in front of a free door of the
    /// parent, facing it, on the same level; one walks from a door into the other.
    fn glue_child(
        &mut self,
        parent: usize,
        child: usize,
        passage: &Passage,
        templates: &[&'a RoomTemplate],
        level: i32,
    ) -> bool {
        for _ in 0..GLUE_TRIES {
            let from: Vec<FreeDoor> = self.free_connectors(&self.placed[&parent]).collect();
            if from.is_empty() {
                return false;
            }
            let (from_door, start, (dx, dy)) = self.pick(&from);
            let template = self.pick_template(templates);
            let facing: Vec<_> = template
                .doors()
                .copied()
                .filter(|c| c.direction() == (-dx, -dy) && level + c.cell.level == start.level)
                .collect();
            if facing.is_empty() {
                continue;
            }
            let to = self.pick(&facing);
            let origin = Cell::new(start.x - to.cell.x, start.y - to.cell.y, level);
            let room = PlacedRoom {
                graph_room: child,
                template: template.clone(),
                origin,
            };
            // As `fits`, but its door takes the cell kept free in front of the door of the parent.
            let reserved = self.reserved();
            if !room
                .footprint()
                .all(|c| self.grid.is_free(c) && self.dedans(c) && (c == start || !reserved.contains(&c)))
            {
                continue;
            }
            let child_door = room.to_dungeon(to.cell);
            self.write(room);
            self.open_door(from_door);
            self.open_door(child_door);
            if let Some(lock) = passage.lock {
                self.locks.push((child_door, lock));
            }
            return true;
        }
        false
    }

    /// A loop (corridor both ways) or a drop (one-way shaft) between two
    /// placed rooms.
    fn connect_extra(&mut self, passage: &Passage) -> bool {
        let drop = passage.kind == PassageKind::Drop;
        let froms: Vec<FreeDoor> = self.free_connectors(&self.placed[&passage.from]).collect();
        let tos: Vec<FreeDoor> = self.free_connectors(&self.placed[&passage.to]).collect();
        let mut pairs = Vec::new();
        for a in &froms {
            for b in &tos {
                if a.1 != b.1 && (!drop || b.1.level < a.1.level) {
                    let distance = (a.1.x - b.1.x).abs() + (a.1.y - b.1.y).abs();
                    pairs.push((distance, self.rng.random::<u32>(), *a, *b));
                }
            }
        }
        pairs.sort_by_key(|(d, r, ..)| (*d, *r));
        for (_, _, (from_door, start, _), (to_door, goal, _)) in
            pairs.into_iter().take(self.settings.extra_passage_tries)
        {
            if let Some(corridor) = self.route(start, goal, drop)
                && corridor.len() <= self.settings.max_extra_length * (50 + self.settings.sinuosite.max(0) as usize) / 50
            {
                self.carve(&corridor);
                self.open_door(from_door);
                self.open_door(to_door);
                return true;
            }
        }
        false
    }

    /// The possible doors of the room still closed, with the free cell in
    /// front and the way out.
    fn free_connectors<'b>(&'b self, room: &'b PlacedRoom) -> impl Iterator<Item = FreeDoor> + 'b {
        room.template.doors().filter_map(move |c| {
            let door = room.to_dungeon(c.cell);
            let outside = room.to_dungeon(c.outside);
            (!self.used_connectors.contains(&door) && self.grid.is_free(outside) && self.dedans(outside))
                .then_some((door, outside, (outside.x - door.x, outside.y - door.y)))
        })
    }

    /// The cells in front of the unused doors of placed rooms: no room is
    /// placed there and no corridor goes through or along them, so that these
    /// doors stay usable.
    fn reserved(&self) -> BTreeSet<Cell> {
        self.placed
            .values()
            .flat_map(|room| self.free_connectors(room).map(|(_, outside, _)| outside).collect::<Vec<_>>())
            .collect()
    }

    fn fits(&self, room: &PlacedRoom) -> bool {
        let reserved = self.reserved();
        room.footprint().all(|c| self.grid.is_free(c) && !reserved.contains(&c) && self.dedans(c))
    }

    fn write(&mut self, room: PlacedRoom) -> Vec<Cell> {
        let mut written = Vec::new();
        for (cell, symbol) in &room.template.plan.cells {
            // Connectors stay closed until used: `+` → wall, `*` → floor.
            let symbol = match symbol {
                '+' => '#',
                '*' => '.',
                s => *s,
            };
            let at = room.to_dungeon(*cell);
            if self.grid.is_free(at) {
                self.grid.cells.insert(at, symbol);
                written.push(at);
            }
        }
        self.in_rooms.extend(written.iter().copied());
        self.placed.insert(room.graph_room, room);
        written
    }

    fn erase(&mut self, written: &[Cell], graph_room: usize) {
        for cell in written {
            self.grid.cells.remove(cell);
            self.in_rooms.remove(cell);
        }
        self.placed.remove(&graph_room);
    }

    fn open_door(&mut self, door: Cell) {
        self.grid.cells.insert(door, 'D');
        self.used_connectors.insert(door);
    }

    /// Lays the cells of the corridor, then walls on the free cells around
    /// (at the same level).
    fn carve(&mut self, corridor: &[(Cell, char)]) {
        for (cell, symbol) in corridor {
            self.grid.cells.insert(*cell, *symbol);
            if *symbol == '.' {
                self.corridors.push(*cell);
            }
        }
        for (cell, _) in corridor {
            for (dx, dy) in DIRECTIONS {
                self.grid.cells.entry(cell.offset(dx, dy, 0)).or_insert('#');
            }
        }
    }

    /// A* corridor search between two free cells. A flat step costs 1; stairs
    /// (2 cells long, one level) cost 5; for a drop, the corridor goes through
    /// exactly one shaft down to the level of arrival (and no stairs: the
    /// drop stays one-way).
    fn route(&self, start: Cell, goal: Cell, drop: bool) -> Option<Vec<(Cell, char)>> {
        let m = self.settings.margin;
        let (mut min_x, mut max_x) = (start.x.min(goal.x) - m, start.x.max(goal.x) + m);
        let (mut min_y, mut max_y) = (start.y.min(goal.y) - m, start.y.max(goal.y) + m);
        for cell in self.grid.cells.keys() {
            min_x = min_x.min(cell.x - m);
            max_x = max_x.max(cell.x + m);
            min_y = min_y.min(cell.y - m);
            max_y = max_y.max(cell.y + m);
        }
        let (min_level, max_level) = (start.level.min(goal.level), start.level.max(goal.level));
        let mut reserved = self.reserved();
        reserved.remove(&start);
        reserved.remove(&goal);
        let free = |c: Cell| {
            (min_x..=max_x).contains(&c.x)
                && (min_y..=max_y).contains(&c.y)
                && (min_level..=max_level).contains(&c.level)
                && self.grid.is_free(c)
                && self.dedans(c)
                && c != start
                && !reserved.contains(&c)
                && DIRECTIONS.iter().all(|(dx, dy)| !reserved.contains(&c.offset(*dx, *dy, 0)))
        };
        let estimate =
            |c: Cell| (c.x - goal.x).abs() + (c.y - goal.y).abs() + 3 * (c.level - goal.level).abs();
        // Icebreaker F.2: behind the top of stairs, at their level, nothing one could
        // walk on (the steps would end in the wall of that corridor): free, or a wall.
        let blind = |c: Cell| {
            let s = self.grid.get(c);
            c != start && c != goal && !(plan::is_walkable(s) || s == '_' || plan::stairs_direction(s).is_some())
        };

        type State = (Cell, bool);
        let origin: State = (start, false);
        let mut cost: BTreeMap<State, i32> = BTreeMap::from([(origin, 0)]);
        let mut came_from: BTreeMap<State, (State, Vec<(Cell, char)>)> = BTreeMap::new();
        let mut open = BinaryHeap::new();
        let mut counter = 0u32;
        open.push(Reverse((estimate(start), counter, origin)));
        let mut examined = 0;

        while let Some(Reverse((_, _, state))) = open.pop() {
            examined += 1;
            // Icebreaker F.47 : a wandering corridor looks farther.
            if examined > self.settings.search_limit * (1 + self.settings.sinuosite.max(0) as usize / 25) {
                break;
            }
            let (cell, fallen) = state;
            if cell == goal && (!drop || fallen) {
                // Rebuild; a path going back over its own cells can't be dug.
                let mut steps = Vec::new();
                let mut at = state;
                while at != origin {
                    let (previous, carved) = &came_from[&at];
                    steps.push(carved.clone());
                    at = *previous;
                }
                steps.reverse();
                let mut corridor = vec![(start, '.')];
                corridor.extend(steps.into_iter().flatten());
                let distinct: BTreeSet<Cell> = corridor.iter().map(|(c, _)| *c).collect();
                // Icebreaker F.2: nor pass behind the top of its own stairs.
                let blind_tops = corridor.iter().all(|(c, s)| {
                    plan::stairs_direction(*s).is_none_or(|(dx, dy)| !distinct.contains(&c.offset(dx, dy, 0)))
                });
                return (distinct.len() == corridor.len() && blind_tops).then_some(corridor);
            }
            let mut visit = |target: State, step: i32, carved: Vec<(Cell, char)>| {
                let new_cost = cost[&state] + step;
                if cost.get(&target).is_some_and(|known| *known <= new_cost) {
                    return;
                }
                cost.insert(target, new_cost);
                came_from.insert(target, (state, carved));
                counter += 1;
                open.push(Reverse((new_cost + estimate(target.0), counter, target)));
            };
            for (dx, dy) in DIRECTIONS {
                let next = cell.offset(dx, dy, 0);
                if free(next) {
                    visit((next, fallen), 1 + self.detour(next), vec![(next, '.')]);
                }
                if !drop {
                    // Up: stairs here, a hole above them, a landing on the level above.
                    let up = next.offset(dx, dy, 1);
                    if free(next) && free(next.offset(0, 0, 1)) && free(up) && blind(next.offset(dx, dy, 0)) {
                        visit((up, fallen), 5, vec![
                            (next, plan::stairs_symbol(dx, dy)),
                            (next.offset(0, 0, 1), '_'),
                            (up, '.'),
                        ]);
                    }
                    // Down: a hole ahead, stairs below (climbing toward us), a landing below.
                    let stairs = next.offset(0, 0, -1);
                    let down = stairs.offset(dx, dy, 0);
                    if free(next) && free(stairs) && free(down) && blind(stairs.offset(-dx, -dy, 0)) {
                        visit((down, fallen), 5, vec![
                            (next, '_'),
                            (stairs, plan::stairs_symbol(-dx, -dy)),
                            (down, '.'),
                        ]);
                    }
                } else if !fallen && cell.level > goal.level {
                    // A shaft down to the level of arrival.
                    let shaft: Vec<Cell> = (goal.level + 1..=cell.level)
                        .map(|l| Cell::new(next.x, next.y, l))
                        .collect();
                    let landing = Cell::new(next.x, next.y, goal.level);
                    if shaft.iter().all(|c| free(*c)) && free(landing) {
                        let mut carved: Vec<(Cell, char)> = shaft.into_iter().map(|c| (c, '_')).collect();
                        carved.push((landing, '.'));
                        visit((landing, true), 3, carved);
                    }
                }
            }
        }
        None
    }

    /// Keys on a floor of their rooms.
    fn finish(&mut self, attempts: usize) -> Assembled {
        let mut keys = Vec::new();
        for room in self.graph.rooms.iter().filter(|r| r.key.is_some()) {
            let placed = &self.placed[&room.id];
            let spots: Vec<Cell> = placed
                .template
                .plan
                .cells
                .iter()
                .filter(|(_, s)| **s == '.')
                .map(|(c, _)| placed.to_dungeon(*c))
                .collect();
            let spot = self.pick(&spots);
            keys.push((spot, room.key.expect("filtered")));
        }
        Assembled {
            plan: self.grid.clone(),
            arrival: self.placed[&self.graph.safe_room()].middle(),
            rooms: (0..self.graph.rooms.len()).map(|r| self.placed[&r].clone()).collect(),
            locks: self.locks.clone(),
            keys,
            passages: self.passages.clone(),
            attempts,
            centre: self.centre,
            etendue: self.etendue,
            labyrinthe: self.labyrinthe.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{graph, settings::*},
        *,
    };

    #[test]
    fn dungeons_assemble_and_are_finishable() {
        let settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        for size in settings.sizes.keys() {
            let resolved = settings
                .resolve(&Mission {
                    size: size.clone(),
                    ..Mission::default()
                })
                .unwrap();
            let mut failures = 0;
            for seed in 0..100 {
                let graph = graph::generate(&resolved, seed);
                match assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) {
                    Some(dungeon) => {
                        assert!(dungeon.check().is_empty());
                        // The goal is the deepest room.
                        let goal_level = dungeon.rooms[graph.goal].origin.level;
                        assert!(dungeon.rooms.iter().all(|r| r.origin.level >= goal_level));
                    },
                    None => failures += 1,
                }
            }
            assert!(failures <= 2, "{size} : {failures} donjons sur 100 non assemblés");
        }
    }


    /// Cells dug outside the rooms (corridors, their stairs and holes) and,
    /// among the corridor stairs, those whose top faces a walkable cell at the
    /// lower level (Icebreaker F.2: steps that end in a wall of a corridor).
    fn corridor_stats(dungeon: &Assembled) -> (usize, usize) {
        let in_room: BTreeSet<Cell> = dungeon.rooms.iter().flat_map(|r| r.footprint()).collect();
        let plan = &dungeon.plan;
        let (mut cells, mut bad_stairs) = (0, 0);
        for (cell, symbol) in &plan.cells {
            if in_room.contains(cell) {
                continue;
            }
            let stairs = plan::stairs_direction(*symbol);
            if plan::is_walkable(*symbol) || stairs.is_some() || *symbol == '_' {
                cells += 1;
            }
            if let Some((dx, dy)) = stairs {
                let behind = plan.get(cell.offset(dx, dy, 0));
                if plan::is_walkable(behind) || behind == '_' || plan::stairs_direction(behind).is_some() {
                    bad_stairs += 1;
                }
            }
        }
        (cells, bad_stairs)
    }

    #[test]
    fn corridors_are_short_and_stairs_lead_somewhere() {
        let settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        for size in settings.sizes.keys() {
            let resolved = settings
                .resolve(&Mission {
                    size: size.clone(),
                    ..Mission::default()
                })
                .unwrap();
            let (mut cells, mut rooms, mut bad_stairs, mut dungeons) = (0, 0, 0, 0);
            let (mut extras, mut extras_dug) = (0, 0);
            for seed in 0..100 {
                let graph = graph::generate(&resolved, seed);
                let assembled =
                    assemble(&graph, &catalog, seed, &settings.assembly, |_| {});
                if let Some(dungeon) = assembled {
                    let (c, b) = corridor_stats(&dungeon);
                    cells += c;
                    bad_stairs += b;
                    rooms += dungeon.rooms.len();
                    dungeons += 1;
                    let tree = dungeon.rooms.len() - 1;
                    extras += graph.passages.len() - tree;
                    extras_dug += dungeon.passages.len() - tree;
                }
            }
            let per_room = cells as f32 / rooms as f32;
            println!("{size} : {dungeons} donjons, {per_room:.1} cases de couloir par salle, {bad_stairs} escalier(s) sur un couloir, {extras_dug} boucles et chutes creusées sur {extras}");
            // Before Icebreaker F.2: 13.6 to 15.0 cells per room.
            assert!(per_room <= 7.5, "{size} : {per_room:.1} cases de couloir par salle");
            assert_eq!(bad_stairs, 0, "{size} : des escaliers de couloir débouchent sur un couloir");
            // Loops join rooms placed far apart: most are longer than the cap and given up.
            assert!(extras_dug * 10 >= extras, "{size} : {extras_dug} boucles et chutes creusées sur {extras}");
        }
    }

    /// Icebreaker F.46 : with an outline, dungeons still assemble, and nothing one walks on lies outside it.
    #[test]
    fn dungeons_stay_inside_their_outline() {
        let mut settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        for silhouette in Silhouette::TOUTES {
            if silhouette == Silhouette::Libre {
                continue;
            }
            settings.assembly.silhouette = silhouette;
            // Tighter, more seeds fail (a third at 32 cells for the narrow shapes).
            settings.assembly.etendue = 44;
            let (mut failures, mut tries) = (0, 0);
            for seed in 0..50 {
                let graph = graph::generate(&resolved, seed);
                let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) else {
                    failures += 1;
                    continue;
                };
                tries += dungeon.attempts;
                assert!(dungeon.check().is_empty());
                let ((cx, cy), r) = (dungeon.centre, dungeon.etendue);
                for (cell, symbol) in &dungeon.plan.cells {
                    assert!(
                        *symbol == '#' || silhouette.dedans(cell.x - cx, cell.y - cy, r),
                        "{silhouette:?}, graine {seed} : {cell:?} hors de la silhouette"
                    );
                }
            }
            let mean = tries as f32 / (50 - failures).max(1) as f32;
            println!("{silhouette:?} : {failures} échec(s) sur 50, {mean:.1} tentative(s) par donjon");
            // A seed that fails gives way to the next one (`SEED_TRIES`).
            assert!(failures <= 10, "{silhouette:?} : {failures} donjons sur 50 non assemblés");
        }
    }

    /// Icebreaker F.47 : corridors grow longer with the sinuosity, dead ends are dug, and the dungeon stays sound.
    #[test]
    fn corridors_wander_and_dead_ends_are_dug() {
        let mut settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        // Room for the corridors to wander.
        settings.assembly.spacing = Range(4, 7);
        let mut measure = |sinuosite: i32, impasses: Range| {
            settings.assembly.sinuosite = sinuosite;
            settings.assembly.impasses = impasses;
            let (mut cells, mut dungeons) = (0, 0);
            for seed in 0..40 {
                let graph = graph::generate(&resolved, seed);
                if let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) {
                    assert!(dungeon.check().is_empty(), "sinuosité {sinuosite}, graine {seed}");
                    let (c, bad_stairs) = corridor_stats(&dungeon);
                    assert_eq!(bad_stairs, 0);
                    cells += c;
                    dungeons += 1;
                }
            }
            assert!(dungeons >= 36, "sinuosité {sinuosite} : {dungeons} donjons sur 40");
            cells as f32 / dungeons as f32
        };
        let straight = measure(0, Range(0, 0));
        let winding = measure(100, Range(0, 0));
        let dead_ends = measure(0, Range(6, 6));
        println!("couloirs par donjon : {straight:.0} cases droit, {winding:.0} sinueux, {dead_ends:.0} avec 6 impasses");
        assert!(winding >= straight * 1.2, "{winding:.0} cases sinueux pour {straight:.0} droit");
        // Six dead ends of 2 to 5 cells, most of them dug.
        assert!(dead_ends >= straight + 9.0, "{dead_ends:.0} cases avec impasses pour {straight:.0} sans");
    }

    /// Icebreaker F.48 : rooms put against each other, door to door: fewer corridors, and the dungeon stays sound.
    #[test]
    fn rooms_are_glued_door_to_door() {
        let mut settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let mut measure = |collees: i32| {
            settings.assembly.collees = collees;
            let (mut cells, mut glued, mut dungeons) = (0, 0, 0);
            for seed in 0..40 {
                let graph = graph::generate(&resolved, seed);
                if let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) {
                    assert!(dungeon.check().is_empty(), "collées {collees}, graine {seed}");
                    cells += corridor_stats(&dungeon).0;
                    // Two doors side by side: two rooms against each other.
                    let doors = dungeon.plan.cells.iter().filter(|(_, s)| **s == 'D');
                    glued += doors
                        .filter(|(c, _)| DIRECTIONS.iter().any(|(dx, dy)| dungeon.plan.get(c.offset(*dx, *dy, 0)) == 'D'))
                        .count()
                        / 2;
                    dungeons += 1;
                }
            }
            assert!(dungeons >= 36, "collées {collees} : {dungeons} donjons sur 40");
            (cells as f32 / dungeons as f32, glued as f32 / dungeons as f32)
        };
        let (apart, none) = measure(0);
        let (close, glued) = measure(100);
        println!("par donjon : {apart:.0} cases de couloir sans salles collées, {close:.0} avec ; {glued:.1} paires collées");
        assert_eq!(none, 0.0);
        assert!(glued >= 5.0, "{glued:.1} paires de salles collées par donjon");
        assert!(close <= apart * 0.8, "{close:.0} cases de couloir collé pour {apart:.0}");
    }

    /// Icebreaker F.3: a medium dungeon has 12 to 18 rooms on average.
    #[test]
    fn medium_dungeons_are_big_enough() {
        let settings = Settings::load().unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let (mut rooms, mut dungeons, mut shaped) = (0, 0, 0);
        for seed in 0..40 {
            let graph = graph::generate(&resolved, seed);
            let catalog = settings.catalog(seed).unwrap();
            if let Some(dungeon) =
                assemble(&graph, &catalog, seed, &settings.assembly, |_| {})
            {
                assert!(dungeon.rooms.len() >= 10, "graine {seed} : {} salles", dungeon.rooms.len());
                // Icebreaker F.5: generated rooms of varied shapes among them.
                let generated = dungeon.rooms.iter().filter(|r| r.template.id.starts_with("forme-")).count();
                assert!(generated >= 1, "graine {seed} : aucune salle générée");
                shaped += generated;
                rooms += dungeon.rooms.len();
                dungeons += 1;
            }
        }
        assert!(shaped >= 3 * dungeons, "{shaped} salles de formes variées sur {dungeons} donjons");
        let mean = rooms as f32 / dungeons as f32;
        println!("medium : {mean:.1} salles en moyenne sur {dungeons} donjons, dont {shaped} de formes variées");
        assert!((12.0..=18.0).contains(&mean), "medium : {mean:.1} salles en moyenne");
    }

    /// Icebreaker F.51 : how much of its outline a dungeon takes (the figures to beat: before the filling of
    /// F.52, a tenth to a quarter of the cells).
    #[test]
    fn the_cover_of_the_outline_is_measured() {
        let mut settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let graph = graph::generate(&resolved, 3);
        let free = assemble(&graph, &catalog, 3, &settings.assembly, |_| {}).unwrap();
        assert_eq!(free.couverture(Silhouette::Libre), None);
        for silhouette in [Silhouette::Carre, Silhouette::Croix, Silhouette::Rond, Silhouette::Couronne] {
            settings.assembly.silhouette = silhouette;
            let (mut cells, mut width, mut dungeons) = (0, 0, 0);
            for seed in 0..20 {
                let graph = graph::generate(&resolved, seed);
                if let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) {
                    let (c, w) = dungeon.couverture(silhouette).unwrap();
                    assert!((1..=100).contains(&c) && (1..=100).contains(&w), "{silhouette:?} : {c} %, {w} %");
                    cells += c;
                    width += w;
                    dungeons += 1;
                }
            }
            assert!(dungeons > 0);
            println!(
                "{silhouette:?} : {} % des cases, {} % de la largeur ({dungeons} donjons)",
                cells / dungeons,
                width / dungeons
            );
        }
    }

    /// Icebreaker F.53 : rooms out of the mission are added, farther from the room they hang on than the others,
    /// and the dungeon stays sound, filled or not.
    #[test]
    fn rooms_out_of_the_mission_are_placed_far() {
        let mut settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let before = graph::generate(&settings.resolve(&Mission::default()).unwrap(), 5).rooms.len();
        settings.assembly.salles_en_plus = Range(5, 5);
        settings.assembly.silhouette = Silhouette::Carre;
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let (mut far, mut near, mut dungeons) = (0, 0, 0);
        for seed in 0..30 {
            settings.assembly.remplissage = if seed % 2 == 0 { 100 } else { 0 };
            let graph = graph::generate(&resolved, seed);
            assert!(graph.check().is_empty(), "graine {seed}");
            assert_eq!(graph.rooms.iter().filter(|r| r.en_plus).count(), 5);
            if seed == 5 {
                assert_eq!(graph.rooms.len(), before + 5);
            }
            let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) else {
                continue;
            };
            assert!(dungeon.check().is_empty(), "graine {seed}");
            for passage in &dungeon.passages {
                let (a, b) = (dungeon.rooms[passage.from].middle(), dungeon.rooms[passage.to].middle());
                let distance = (a.x - b.x).abs() + (a.y - b.y).abs();
                if graph.rooms[passage.to].en_plus {
                    far += distance;
                } else if passage.kind != PassageKind::Drop {
                    near += distance * 5 / (dungeon.passages.len() as i32 - 5);
                }
            }
            dungeons += 1;
        }
        assert!(dungeons >= 24, "{dungeons} donjons sur 30");
        println!("d'une salle à la suivante : {} cases, à une salle en plus : {}", near / 5 / dungeons, far / 5 / dungeons);
        assert!(far > 2 * near, "salles en plus à {far} cases, les autres à {near}");
    }
}
