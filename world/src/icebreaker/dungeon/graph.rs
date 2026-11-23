//! Icebreaker: the mission graph, what a dungeon is before knowing where its
//! rooms are (`docs/icebreaker/design/generation.md`, « Génération des
//! donjons »). Translated from `MissionGraphGenerator.cs` of the Icebreaker
//! project.

use super::{rooms::RoomRole, settings::Resolved};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeSet, VecDeque};

/// A room of the graph. `level`: depth, 0 = the entrance's, larger = lower.
/// `key`: the key found in the room (the lock it opens).
#[derive(Clone, Debug)]
pub struct MissionRoom {
    pub id: usize,
    pub role: RoomRole,
    pub level: i32,
    pub on_main_path: bool,
    pub key: Option<usize>,
    /// Icebreaker F.53 : a room out of the mission, placed where the outline is still empty.
    pub en_plus: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PassageKind {
    /// Same depth, both ways.
    Corridor,
    /// Change of depth, both ways.
    Stairs,
    /// Fall toward a deeper room: one way only, from `from` to `to`.
    Drop,
}

/// `lock`: the key needed to go through.
#[derive(Clone, Debug)]
pub struct Passage {
    pub from: usize,
    pub to: usize,
    pub kind: PassageKind,
    pub lock: Option<usize>,
}

impl Passage {
    /// One can go from `room` to the other end (lock aside).
    pub fn leads_from(&self, room: usize) -> bool {
        room == self.from || (room == self.to && self.kind != PassageKind::Drop)
    }

    pub fn other(&self, room: usize) -> usize { if room == self.from { self.to } else { self.from } }
}

/// Rooms with a role and a depth, passages between them. Room 0 is the
/// entrance.
#[derive(Clone, Debug)]
pub struct MissionGraph {
    pub rooms: Vec<MissionRoom>,
    pub passages: Vec<Passage>,
    pub goal: usize,
}

impl MissionGraph {
    /// Checks that a group leaving the entrance reaches the goal and every
    /// room (picking the keys up on the way), and can always come back to the
    /// entrance (no drop traps it). Empty: a sound dungeon.
    pub fn check(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let mut keys = BTreeSet::new();
        let reached = loop {
            let reached = self.reachable(0, &keys);
            let before = keys.len();
            keys.extend(reached.iter().filter_map(|r| self.rooms[*r].key));
            if keys.len() == before {
                break reached;
            }
        };
        if !reached.contains(&self.goal) {
            errors.push("l'objectif est inatteignable depuis l'entrée".to_string());
        }
        for room in self.rooms.iter().filter(|r| !reached.contains(&r.id)) {
            errors.push(format!("salle {} ({}) inatteignable", room.id, room.role.name()));
        }
        let all_keys: BTreeSet<usize> = self.rooms.iter().filter_map(|r| r.key).collect();
        for room in &self.rooms {
            if !self.reachable(room.id, &all_keys).contains(&0) {
                errors.push(format!(
                    "impossible de revenir à l'entrée depuis la salle {} ({})",
                    room.id,
                    room.role.name()
                ));
            }
        }
        errors
    }

    /// Icebreaker F.31 : the safe room where the party arrives.
    pub fn safe_room(&self) -> usize {
        self.rooms.iter().rposition(|r| r.role == RoomRole::SafeRoom).expect("every graph has its safe room")
    }

    /// Rooms reachable from `start` with the given keys.
    pub fn reachable(&self, start: usize, keys: &BTreeSet<usize>) -> BTreeSet<usize> {
        let mut seen = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(room) = queue.pop_front() {
            for passage in &self.passages {
                if !passage.leads_from(room) || passage.lock.is_some_and(|l| !keys.contains(&l)) {
                    continue;
                }
                let next = passage.other(room);
                if seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        seen
    }

    /// The graph as text, for the logs.
    pub fn describe(&self) -> String {
        let mut text = String::new();
        for room in &self.rooms {
            text.push_str(&format!(
                "{}{} {} (étage -{}){}\n",
                if room.id == self.goal { "→ " } else { "" },
                room.id,
                room.role.name(),
                room.level,
                room.key.map(|k| format!(", clé {k}")).unwrap_or_default()
            ));
        }
        for p in &self.passages {
            text.push_str(&format!(
                "{} {} {}{}\n",
                p.from,
                match p.kind {
                    PassageKind::Corridor => "—",
                    PassageKind::Stairs => "≡",
                    PassageKind::Drop => "↓",
                },
                p.to,
                p.lock.map(|l| format!(" (verrou {l})")).unwrap_or_default()
            ));
        }
        text
    }
}

pub fn shuffle<T>(items: &mut [T], rng: &mut impl RngExt) {
    for i in (1..items.len()).rev() {
        let j = rng.random_range(0..=i);
        items.swap(i, j);
    }
}

#[derive(Default)]
struct Builder {
    rooms: Vec<MissionRoom>,
    passages: Vec<Passage>,
    zone: Vec<usize>,
}

impl Builder {
    fn level(&self, room: usize) -> i32 { self.rooms[room].level }

    fn count(&self, role: RoomRole) -> usize { self.rooms.iter().filter(|r| r.role == role).count() }

    fn add_room(&mut self, role: RoomRole, level: i32, on_main_path: bool, zone: usize) -> usize {
        let id = self.rooms.len();
        self.rooms.push(MissionRoom {
            id,
            role,
            level,
            on_main_path,
            key: None,
            en_plus: false,
        });
        self.zone.push(zone);
        id
    }

    fn connect(&mut self, from: usize, to: usize, kind: Option<PassageKind>) {
        let kind = kind.unwrap_or(if self.level(from) == self.level(to) {
            PassageKind::Corridor
        } else {
            PassageKind::Stairs
        });
        self.passages.push(Passage {
            from,
            to,
            kind,
            lock: None,
        });
    }

    /// Number of passages between `start` and each room, either way.
    fn distances(&self, start: usize) -> Vec<usize> {
        let mut distances = vec![usize::MAX; self.rooms.len()];
        distances[start] = 0;
        let mut queue = VecDeque::from([start]);
        while let Some(room) = queue.pop_front() {
            for p in self.passages.iter().filter(|p| p.from == room || p.to == room) {
                let next = p.other(room);
                if distances[next] == usize::MAX {
                    distances[next] = distances[room] + 1;
                    queue.push_back(next);
                }
            }
        }
        distances
    }

    /// Pairs (a, b) of a same zone, at least `min_distance` passages apart,
    /// accepted by `accept`, in a stable order (the seed is enough).
    fn far_pairs(&self, min_distance: usize, accept: impl Fn(usize, usize) -> bool) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        for a in 0..self.rooms.len() {
            let distances = self.distances(a);
            for b in 0..self.rooms.len() {
                if a != b && self.zone[a] == self.zone[b] && distances[b] >= min_distance && accept(a, b) {
                    pairs.push((a, b));
                }
            }
        }
        pairs
    }

    /// The role of a room of a branch: drawn by weight among the roles whose
    /// rule is respected (neighbour, distance to the main path, maximum),
    /// otherwise the fallback role.
    fn pick_branch_role(
        &self,
        rng: &mut ChaCha8Rng,
        settings: &Resolved,
        parent: RoomRole,
        distance: i32,
    ) -> RoomRole {
        let allowed: Vec<(RoomRole, f64)> = settings
            .weights
            .iter()
            .filter(|(role, weight)| {
                let ok = settings.rules.get(role).is_none_or(|rule| {
                    (rule.next_to.is_empty() || rule.next_to.contains(&parent))
                        && distance >= rule.min_distance_from_main_path
                        && rule.max.is_none_or(|max| self.count(**role) < max)
                });
                ok && **weight > 0.0
            })
            .map(|(role, weight)| (*role, *weight))
            .collect();
        let total: f64 = allowed.iter().map(|(_, w)| w).sum();
        let mut roll = rng.random::<f64>() * total;
        for (role, weight) in &allowed {
            roll -= weight;
            if roll < 0.0 {
                return *role;
            }
        }
        allowed.last().map_or(settings.fallback, |(role, _)| *role)
    }

    /// Adds a branch at the level of its anchor; returns its last room.
    fn add_branch(&mut self, rng: &mut ChaCha8Rng, settings: &Resolved, anchor: usize, zone: usize) -> usize {
        let mut previous = anchor;
        let length = settings.shape.branch_length.roll(rng);
        for distance in 1..=length {
            let role = self.pick_branch_role(rng, settings, self.rooms[previous].role, distance);
            let room = self.add_room(role, self.level(anchor), false, zone);
            self.connect(previous, room, None);
            previous = room;
        }
        previous
    }
}

/// Makes the mission graph. Steps: the main path from the entrance to the
/// goal, cut into deeper and deeper levels; locks on the main path, each with
/// a branch hiding its key *before* it; other branches (dead ends); loops and
/// drops, only between rooms of a same zone (between two locks) so that they
/// never get around a lock. Drops only add a shortcut to a graph already
/// connected both ways: one can always come back up.
pub fn generate(settings: &Resolved, seed: u64) -> MissionGraph {
    let shape = &settings.shape;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut b = Builder::default();

    // 1. Main path and levels: the cuts choose where one goes down a level.
    let main_count = shape.main_path.roll(&mut rng).max(2) as usize;
    let level_count = (shape.levels.roll(&mut rng).max(1) as usize).min(main_count);
    let mut candidates: Vec<usize> = (1..main_count).collect();
    shuffle(&mut candidates, &mut rng);
    let cuts: BTreeSet<usize> = candidates.into_iter().take(level_count - 1).collect();
    let mut level = 0;
    let mut main = Vec::new();
    // Icebreaker F.56 : a change of level may go down several levels at once (a tower); never to the goal. Their
    // own dice, and none drawn without towers.
    let mut towers = (settings.tours > 0).then(|| ChaCha8Rng::seed_from_u64(seed ^ 0x70_0125));
    for i in 0..main_count {
        if cuts.contains(&i) {
            level += match &mut towers {
                Some(dice) if i < main_count - 1 => {
                    if dice.random_range(0..100) < settings.tours { settings.tour_etages.roll(dice).max(1) } else { 1 }
                },
                _ => 1,
            };
        }
        let role = if i == 0 {
            RoomRole::Entrance
        } else if i == main_count - 1 {
            settings.goal
        } else {
            RoomRole::Hall
        };
        main.push(b.add_room(role, level, true, 0));
        if i > 0 {
            b.connect(main[i - 1], main[i], None);
        }
    }

    // 2. Locks: on the passage between main[e] and main[e + 1], never right
    // after the entrance. The zone of a main room = locks crossed to reach it.
    let mut lock_edges: Vec<usize> = (1..main_count.saturating_sub(1)).collect();
    shuffle(&mut lock_edges, &mut rng);
    lock_edges.truncate(shape.locks.roll(&mut rng).max(0) as usize);
    lock_edges.sort_unstable();
    let mut zone_of_main = vec![0; main_count];
    for i in 1..main_count {
        zone_of_main[i] = zone_of_main[i - 1] + usize::from(lock_edges.contains(&(i - 1)));
    }
    for (i, room) in main.iter().enumerate() {
        b.zone[*room] = zone_of_main[i];
    }

    // At most `max_branches_per_room` branches per room: a room has few doors.
    let mut branches_at = vec![0; main_count];
    let mut pick_anchor = |rng: &mut ChaCha8Rng, first: usize, last: usize, required: bool| {
        let free: Vec<usize> = (first..=last)
            .filter(|i| branches_at[*i] < shape.max_branches_per_room)
            .collect();
        if free.is_empty() {
            return required.then(|| rng.random_range(first..=last));
        }
        let anchor = free[rng.random_range(0..free.len())];
        branches_at[anchor] += 1;
        Some(anchor)
    };

    let mut branches_left = (shape.branches.roll(&mut rng).max(0) as usize).max(lock_edges.len());
    for (k, edge) in lock_edges.iter().enumerate() {
        let index = b
            .passages
            .iter()
            .position(|p| p.from == main[*edge] && p.to == main[edge + 1])
            .expect("the main path is connected");
        b.passages[index].lock = Some(k);
        // A guard room watches the locked door (never the entrance nor the goal).
        if settings.guard_before_locks {
            b.rooms[main[*edge]].role = RoomRole::GuardRoom;
        }
    }
    for (k, edge) in lock_edges.iter().enumerate() {
        // The key is in a branch hooked between the previous lock and this one.
        let first = if k == 0 { 0 } else { lock_edges[k - 1] + 1 };
        let anchor = pick_anchor(&mut rng, first, *edge, true).expect("required");
        let end = b.add_branch(&mut rng, settings, main[anchor], zone_of_main[anchor]);
        b.rooms[end].key = Some(k);
        branches_left = branches_left.saturating_sub(1);
    }

    // 3. Other branches, anywhere but on the goal.
    while branches_left > 0 {
        if let Some(anchor) = pick_anchor(&mut rng, 0, main_count - 2, false) {
            b.add_branch(&mut rng, settings, main[anchor], zone_of_main[anchor]);
        }
        branches_left -= 1;
    }

    // 4. Loops then drops, between distant rooms of a same zone.
    for _ in 0..shape.loops.roll(&mut rng).max(0) {
        let candidates = b.far_pairs(3, |a, c| (b.level(a) - b.level(c)).abs() <= 1);
        if candidates.is_empty() {
            break;
        }
        let (from, to) = candidates[rng.random_range(0..candidates.len())];
        b.connect(from, to, None);
    }
    for _ in 0..shape.drops.roll(&mut rng).max(0) {
        let candidates = b.far_pairs(2, |a, c| b.level(c) > b.level(a));
        if candidates.is_empty() {
            break;
        }
        let (from, to) = candidates[rng.random_range(0..candidates.len())];
        b.connect(from, to, Some(PassageKind::Drop));
    }

    // 5. Icebreaker F.20 : the rooms every dungeon has (portal, altar), each a dead end hooked on a room that has
    // few passages, far enough from the entrance. Their own dice: the graph above is the one the seed always gave.
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x5BEC_1A1);
    let goal = main[main_count - 1];
    let rooms = b.rooms.len() as f64;
    let levels = b.rooms.iter().map(|r| r.level).max().unwrap_or(0) as f64 + 1.0;
    for (role, special) in &settings.specials {
        let extra = special.extra;
        let chance = (extra.base + extra.per_room * rooms + extra.per_level * levels).clamp(0.0, 1.0);
        let count = special.min + (special.min..special.max).filter(|_| rng.random_bool(chance)).count();
        for _ in 0..count {
            let distances = b.distances(main[0]);
            // The safe room (added below, F.31) takes a door of the entrance.
            let passages = |room: usize| {
                b.passages.iter().filter(|p| p.from == room || p.to == room).count() + usize::from(room == main[0])
            };
            let anchors = |min_distance: usize| -> Vec<usize> {
                let free = |r: &&MissionRoom| {
                    r.id != goal && !RoomRole::SPECIAL.contains(&r.role) && distances[r.id] + 1 >= min_distance
                };
                let fewest = b.rooms.iter().filter(free).map(|r| passages(r.id)).min();
                b.rooms.iter().filter(free).filter(|r| Some(passages(r.id)) == fewest).map(|r| r.id).collect()
            };
            let mut found = anchors(special.min_distance_from_entrance);
            if found.is_empty() {
                found = anchors(0);
            }
            let anchor = found[rng.random_range(0..found.len())];
            let room = b.add_room(*role, b.level(anchor), false, b.zone[anchor]);
            b.connect(anchor, room, None);
        }
    }

    // 5 bis. Icebreaker F.53 : the rooms out of the mission, each a dead end hooked on a room that has few
    // passages; the assembly puts them far from it, where the outline is still empty. Their own dice, and none
    // drawn without them.
    if settings.en_plus.1 > 0 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0xE2_9105);
        for _ in 0..settings.en_plus.roll(&mut rng).max(0) {
            let passages = |room: usize| {
                b.passages.iter().filter(|p| p.from == room || p.to == room).count() + usize::from(room == main[0])
            };
            let free = |r: &&MissionRoom| r.id != goal && !r.en_plus && !RoomRole::SPECIAL.contains(&r.role);
            let fewest = b.rooms.iter().filter(free).map(|r| passages(r.id)).min();
            let anchors: Vec<usize> =
                b.rooms.iter().filter(free).filter(|r| Some(passages(r.id)) == fewest).map(|r| r.id).collect();
            let anchor = anchors[rng.random_range(0..anchors.len())];
            let role = b.pick_branch_role(&mut rng, settings, b.rooms[anchor].role, 2);
            let room = b.add_room(role, b.level(anchor), false, b.zone[anchor]);
            b.rooms[room].en_plus = true;
            b.connect(anchor, room, None);
        }
    }

    // 6. Icebreaker F.31 : the safe room where the party arrives, a dead end off the entrance, last so that the
    // rest of the graph stays the one the seed always gave.
    let safe = b.add_room(RoomRole::SafeRoom, b.level(main[0]), false, b.zone[main[0]]);
    b.connect(main[0], safe, None);

    MissionGraph {
        rooms: b.rooms,
        passages: b.passages,
        goal: main[main_count - 1],
    }
}

#[cfg(test)]
mod tests {
    use super::{super::settings::*, *};

    #[test]
    fn a_thousand_seeds_per_size_are_sound() {
        let settings = Settings::load().unwrap();
        for size in settings.sizes.keys() {
            let mission = Mission {
                size: size.clone(),
                ..Mission::default()
            };
            let resolved = settings.resolve(&mission).unwrap();
            for seed in 0..1000 {
                let graph = generate(&resolved, seed);
                let errors = graph.check();
                assert!(errors.is_empty(), "{size}, graine {seed} : {errors:?}\n{}", graph.describe());

                let goal = &graph.rooms[graph.goal];
                assert_eq!(goal.role, RoomRole::ChiefLair);
                assert!(graph.rooms.iter().all(|r| r.level <= goal.level));
                assert_eq!(graph.rooms.iter().filter(|r| r.role == RoomRole::ChiefLair).count(), 1);
                // Icebreaker F.31 : one safe room, a dead end off the entrance.
                let safe: Vec<&MissionRoom> = graph.rooms.iter().filter(|r| r.role == RoomRole::SafeRoom).collect();
                assert_eq!(safe.len(), 1, "{size} {seed}");
                let doors: Vec<&Passage> =
                    graph.passages.iter().filter(|p| p.from == safe[0].id || p.to == safe[0].id).collect();
                assert_eq!(doors.len(), 1, "{size} {seed} : l'abri n'est pas un cul-de-sac");
                assert_eq!(doors[0].other(safe[0].id), 0, "{size} {seed} : l'abri n'est pas sur l'entrée");

                // Icebreaker F.20 : the rooms every dungeon has, each a dead end off the goal.
                for (role, special) in &resolved.specials {
                    let placed: Vec<&MissionRoom> = graph.rooms.iter().filter(|r| r.role == *role).collect();
                    assert!((special.min..=special.max).contains(&placed.len()), "{size} {seed} : {} {}", placed.len(), role.name());
                    for room in placed {
                        let doors: Vec<&Passage> = graph.passages.iter().filter(|p| p.from == room.id || p.to == room.id).collect();
                        assert_eq!(doors.len(), 1, "{size} {seed} : {} n'est pas un cul-de-sac", role.name());
                        assert_ne!(doors[0].other(room.id), graph.goal);
                    }
                }
                // Believable inhabitants.
                for (role, rule) in &resolved.rules {
                    let count = graph.rooms.iter().filter(|r| r.role == *role).count();
                    assert!(rule.max.is_none_or(|max| count <= max), "{size} {seed} : trop de {}", role.name());
                }
                for p in graph.passages.iter().filter(|p| p.lock.is_some()) {
                    assert_eq!(graph.rooms[p.from].role, RoomRole::GuardRoom);
                }
                for room in graph.rooms.iter().filter(|r| r.role == RoomRole::Nest && !r.on_main_path) {
                    let parent = graph.passages.iter().find(|p| p.to == room.id).map(|p| p.from);
                    assert!(
                        parent.is_some_and(|p| !graph.rooms[p].on_main_path),
                        "{size} {seed} : nid accroché au chemin principal"
                    );
                }
                for room in graph.rooms.iter().filter(|r| r.role == RoomRole::Barracks) {
                    let parent = graph
                        .passages
                        .iter()
                        .find(|p| p.to == room.id && p.kind != PassageKind::Drop)
                        .map(|p| graph.rooms[p.from].role);
                    assert!(
                        matches!(parent, Some(RoomRole::Entrance | RoomRole::GuardRoom)),
                        "{size} {seed} : caserne après {parent:?}"
                    );
                }
            }
        }
    }

    #[test]
    /// Icebreaker F.20 : a second portal or altar is rare, less so in a large dungeon; and the rooms every dungeon
    /// has leave the rest of its graph as the seed always made it.
    fn extra_special_rooms_are_rare() {
        let settings = Settings::load().unwrap();
        let count = |size: &str| {
            let resolved = settings.resolve(&Mission { size: size.into(), ..Mission::default() }).unwrap();
            let (mut rooms, mut extras) = (0, 0);
            for seed in 0..1000 {
                let graph = generate(&resolved, seed);
                let special = graph.rooms.iter().filter(|r| RoomRole::SPECIAL.contains(&r.role)).count();
                rooms += graph.rooms.len();
                extras += special - resolved.specials.iter().map(|(_, s)| s.min).sum::<usize>();
            }
            println!("{size} : {extras} salles spéciales en plus sur 1000 donjons, {:.1} salles en moyenne", rooms as f32 / 1000.0);
            extras
        };
        let (small, large) = (count("small"), count("large"));
        assert!(small < large && large < 300, "en plus : {small} (petits), {large} (grands)");

        let resolved = settings.resolve(&Mission::default()).unwrap();
        let mut bare = resolved.clone();
        bare.specials.clear();
        for seed in 0..50 {
            let (with, without) = (generate(&resolved, seed), generate(&bare, seed));
            let rooms = |g: &MissionGraph| -> Vec<(RoomRole, i32)> {
                g.rooms.iter().filter(|r| r.role != RoomRole::SafeRoom).map(|r| (r.role, r.level)).collect()
            };
            let kept = rooms(&with).iter().zip(&rooms(&without)).all(|(a, b)| a == b);
            assert!(kept && without.rooms.len() < with.rooms.len(), "graine {seed}");
        }
    }

    #[test]
    fn same_seed_same_graph() {
        let settings = Settings::load().unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        assert_eq!(generate(&resolved, 42).describe(), generate(&resolved, 42).describe());
    }
}
