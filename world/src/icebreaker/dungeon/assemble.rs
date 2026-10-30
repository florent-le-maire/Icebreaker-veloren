//! Icebreaker: assembly, from a mission graph to a plan
//! (`docs/icebreaker/design/generation.md`, « Assemblage du donjon »).
//! Translated from `DungeonAssembler.cs` of the Icebreaker project, without
//! the shared walls of castles; the tunnel up to the surface is ours.
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
    Tunnel,
    graph::{MissionGraph, Passage, PassageKind},
    plan::{self, Cell, DIRECTIONS, Plan},
    reserve_tunnel,
    rooms::{RoomCatalog, RoomTemplate},
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
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
    /// Cells between a door and the next room.
    pub spacing: super::settings::Range,
    /// Cells examined by a corridor search.
    pub search_limit: usize,
    /// Margin around the dungeon where a corridor may go.
    pub margin: i32,
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
}

/// The result of the assembly.
#[derive(Clone, Debug)]
pub struct Assembled {
    pub plan: Plan,
    pub tunnel: Tunnel,
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
}

impl Assembled {
    /// Checks the plan cell by cell: leaving the tunnel and picking the keys
    /// up, one reaches the whole dungeon (locked doors closed without their
    /// key); from any cell, one can come back to the tunnel.
    pub fn check(&self) -> Vec<String> {
        let plan = &self.plan;
        let start = *self.tunnel.cells.last().expect("a tunnel has cells");
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
}

/// Assembles `graph`; `None` if no attempt succeeded. `log` gets the reason of
/// each failed attempt.
pub fn assemble(
    graph: &MissionGraph,
    catalog: &RoomCatalog,
    seed: u64,
    settings: &AssemblySettings,
    tunnel_cells: i32,
    mut log: impl FnMut(String),
) -> Option<Assembled> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    for attempt in 1..=settings.attempts {
        let mut state = Attempt::new(graph, catalog, &mut rng, settings, tunnel_cells);
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
    tunnel_cells: i32,
    grid: Plan,
    placed: BTreeMap<usize, PlacedRoom>,
    used_connectors: BTreeSet<Cell>,
    locks: Vec<(Cell, usize)>,
    passages: Vec<Passage>,
    tree_degree: Vec<usize>,
    degree: Vec<usize>,
    tunnel: Option<Tunnel>,
    placement_failure: String,
}

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
        tunnel_cells: i32,
    ) -> Self {
        Self {
            graph,
            catalog,
            rng,
            settings,
            tunnel_cells,
            grid: Plan::default(),
            placed: BTreeMap::new(),
            used_connectors: BTreeSet::new(),
            locks: Vec::new(),
            passages: Vec::new(),
            tree_degree: vec![0; graph.rooms.len()],
            degree: vec![0; graph.rooms.len()],
            tunnel: None,
            placement_failure: String::new(),
        }
    }

    fn pick<T: Clone>(&mut self, items: &[T]) -> T { items[self.rng.random_range(0..items.len())].clone() }

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
        // The tunnel up to the surface takes a door of the entrance too.
        self.tree_degree[0] += 1;
        self.degree[0] += 1;

        // The entrance, and the tunnel up to the surface through one of its doors.
        let templates = self.templates_for(0);
        if templates.is_empty() {
            return Err("aucune salle d'entrée".into());
        }
        let template = self.pick(&templates);
        self.write(PlacedRoom {
            graph_room: 0,
            template: template.clone(),
            origin: Cell::new(0, 0, 0),
        });
        let mut doors: Vec<FreeDoor> = self.free_connectors(&self.placed[&0]).collect();
        super::graph::shuffle(&mut doors, self.rng);
        // Keep a door for each passage of the entrance.
        if doors.len() < self.tree_degree[0] {
            return Err("pas de porte libre pour le tunnel".into());
        }
        for (door, _, dir) in doors {
            if let Some(tunnel) = reserve_tunnel(&mut self.grid, door, dir, self.tunnel_cells) {
                self.used_connectors.insert(door);
                self.tunnel = Some(tunnel);
                break;
            }
        }
        if self.tunnel.is_none() {
            return Err("pas de place pour le tunnel".into());
        }

        // The tree: each passage links a placed room to a room still to place.
        let mut queue = VecDeque::from([0]);
        while let Some(parent) = queue.pop_front() {
            for passage in tree.iter().filter(|p| p.from == parent || p.to == parent) {
                let child = passage.other(parent);
                if self.placed.contains_key(&child) {
                    continue;
                }
                if !self.place_child(parent, child, passage) {
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
        Ok(())
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

    fn place_child(&mut self, parent: usize, child: usize, passage: &Passage) -> bool {
        let templates = self.templates_for(child);
        if templates.is_empty() {
            self.placement_failure = "aucune salle pour ce rôle".into();
            return false;
        }
        let (mut overlaps, mut no_corridor) = (0, 0);
        let level = -self.graph.rooms[child].level;
        for _ in 0..self.settings.placement_tries {
            let from: Vec<FreeDoor> = self.free_connectors(&self.placed[&parent]).collect();
            if from.is_empty() {
                self.placement_failure = format!(
                    "plus de porte libre (après {overlaps} chevauchement(s), {no_corridor} couloir(s) \
                     impossible(s))"
                );
                return false;
            }
            let (from_door, start, (dx, dy)) = self.pick(&from);

            // Preferably a door facing the parent's: the room then lies beyond it.
            let template = self.pick(&templates);
            let doors: Vec<_> = template.doors().copied().collect();
            let facing: Vec<_> = doors.iter().copied().filter(|c| c.direction() == (-dx, -dy)).collect();
            let use_facing = !facing.is_empty() && self.rng.random_range(0..4) > 0;
            let to = self.pick(if use_facing { &facing } else { &doors });
            let faces = facing.iter().any(|c| c.cell == to.cell);

            // Where the front of the child's door should be: facing the parent's door.
            // A door that does not face: the room would stretch toward the parent, put it
            // farther (the corridor goes around).
            let (width, depth) = template_size(template);
            let ahead = self.settings.spacing.roll(self.rng)
                + 2 * (start.level - (level + to.outside.level)).abs()
                + if faces { 0 } else { width.max(depth) };
            let side = self.rng.random_range(-3..=3);
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
            if let Some(corridor) = self.route(start, goal, drop) {
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
            (!self.used_connectors.contains(&door) && self.grid.is_free(outside))
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
        room.footprint().all(|c| self.grid.is_free(c) && !reserved.contains(&c))
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
        self.placed.insert(room.graph_room, room);
        written
    }

    fn erase(&mut self, written: &[Cell], graph_room: usize) {
        for cell in written {
            self.grid.cells.remove(cell);
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
                && c != start
                && !reserved.contains(&c)
                && DIRECTIONS.iter().all(|(dx, dy)| !reserved.contains(&c.offset(*dx, *dy, 0)))
        };
        let estimate =
            |c: Cell| (c.x - goal.x).abs() + (c.y - goal.y).abs() + 3 * (c.level - goal.level).abs();

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
            if examined > self.settings.search_limit {
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
                return (distinct.len() == corridor.len()).then_some(corridor);
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
                    visit((next, fallen), 1, vec![(next, '.')]);
                }
                if !drop {
                    // Up: stairs here, a hole above them, a landing on the level above.
                    let up = next.offset(dx, dy, 1);
                    if free(next) && free(next.offset(0, 0, 1)) && free(up) {
                        visit((up, fallen), 5, vec![
                            (next, plan::stairs_symbol(dx, dy)),
                            (next.offset(0, 0, 1), '_'),
                            (up, '.'),
                        ]);
                    }
                    // Down: a hole ahead, stairs below (climbing toward us), a landing below.
                    let stairs = next.offset(0, 0, -1);
                    let down = stairs.offset(dx, dy, 0);
                    if free(next) && free(stairs) && free(down) {
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
            tunnel: self.tunnel.clone().expect("placed in run"),
            rooms: (0..self.graph.rooms.len()).map(|r| self.placed[&r].clone()).collect(),
            locks: self.locks.clone(),
            keys,
            passages: self.passages.clone(),
            attempts,
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
        let catalog = RoomCatalog::load().unwrap();
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
                match assemble(&graph, &catalog, seed, &settings.assembly, settings.render.tunnel_cells, |_| {}) {
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
}

