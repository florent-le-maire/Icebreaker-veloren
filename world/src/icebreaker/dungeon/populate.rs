//! Icebreaker: people and loot of an assembled dungeon, by role of room
//! (`docs/icebreaker/design/generation.md`, « Habitants crédibles »).
//! Translated from `DungeonPopulator.cs` of the Icebreaker project.
//!
//! The markers of the room templates are only proposals: they are removed,
//! then each creature or chest goes first on a proposal of the same kind (the
//! archer on the balcony), otherwise on a free floor cell of the room. Never
//! on a key or a cell touching a door (the way stays free). The chief always
//! guards the goal room.

use super::{
    assemble::Assembled,
    graph::{MissionGraph, shuffle},
    plan::{self, Cell, DIRECTIONS},
    settings::Resolved,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeMap, BTreeSet};

/// The kinds of creatures of the plans, by marker (lowercase: awake), the most
/// important first. `beast` (Icebreaker F.4) is what lives in the nests.
pub const KINDS: [(&str, char); 6] = [
    ("boss", 'x'),
    ("chief", 'c'),
    ("archer", 'a'),
    ("brute", 'b'),
    ("beast", 'w'),
    ("lookout", 'g'),
];

/// The kind of creature of a marker (any case), if it has one.
pub fn kind_of(marker: char) -> Option<&'static str> {
    KINDS
        .iter()
        .find(|(_, m)| *m == marker.to_ascii_lowercase())
        .map(|(k, _)| *k)
}

/// Places the people and the chests in the plan of `dungeon` (as markers);
/// returns how many creatures and chests.
pub fn populate(dungeon: &mut Assembled, graph: &MissionGraph, settings: &Resolved, seed: u64) -> (usize, usize) {
    // Draws of their own: the people don't change the shape of the dungeon.
    let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_mul(31).wrapping_add(7));
    let keys: BTreeSet<Cell> = dungeon.keys.iter().map(|(c, _)| *c).collect();
    let (mut creatures, mut chests) = (0, 0);
    for room in &dungeon.rooms {
        let plan = &mut dungeon.plan;
        // Proposals of the template (removed from the plan) and free cells of the room.
        let mut proposals: BTreeMap<char, Vec<Cell>> = BTreeMap::new();
        let mut free = Vec::new();
        for (template_cell, symbol) in &room.template.plan.cells {
            let cell = room.to_dungeon(*template_cell);
            let marker = *symbol == '$' || kind_of(*symbol).is_some();
            if marker {
                plan.cells.insert(cell, '.');
                proposals
                    .entry(symbol.to_ascii_lowercase())
                    .or_default()
                    .push(cell);
            }
            let touches_door = DIRECTIONS
                .iter()
                .any(|(dx, dy)| plan.get(cell.offset(*dx, *dy, 0)) == 'D');
            if (marker || *symbol == '.') && plan.get(cell) == '.' && !keys.contains(&cell) && !touches_door {
                free.push(cell);
            }
        }
        shuffle(&mut free, &mut rng);
        for list in proposals.values_mut() {
            shuffle(list, &mut rng);
        }
        let mut usable: BTreeSet<Cell> = free.iter().copied().collect();
        let mut take = |plan: &mut plan::Plan, proposal: char, symbol: char| -> bool {
            let spot = proposals
                .get(&proposal)
                .into_iter()
                .flatten()
                .chain(&free)
                .find(|c| usable.contains(c))
                .copied();
            if let Some(cell) = spot {
                usable.remove(&cell);
                plan.cells.insert(cell, symbol);
            }
            spot.is_some()
        };

        let graph_room = &graph.rooms[room.graph_room];
        let population = settings.population.get(&graph_room.role).cloned().unwrap_or_default();
        let mut counts = population.creatures.clone();
        // A mission is won by killing the chief: he guards the goal room, whatever its role.
        if graph_room.id == graph.goal && !counts.contains_key("boss") {
            counts.insert("chief".into(), super::settings::Range(1, 1));
        }
        // The most important first, in case the room lacks room.
        for (kind, marker) in KINDS {
            let Some(range) = counts.get(kind) else {
                continue;
            };
            let symbol = if population.awake { marker } else { marker.to_ascii_uppercase() };
            for _ in 0..range.roll(&mut rng).max(0) {
                if !take(plan, marker, symbol) {
                    break;
                }
                creatures += 1;
            }
        }
        for _ in 0..population.chests.roll(&mut rng).max(0) {
            if !take(plan, '$', '$') {
                break;
            }
            chests += 1;
        }
    }

    // Icebreaker F.4: the corridors too. Their free floor cells (out of the rooms,
    // not next to a door, stairs or a hole), by slices of `per_cells`.
    let in_room: BTreeSet<Cell> = dungeon.rooms.iter().flat_map(|r| r.footprint()).collect();
    let plan = &mut dungeon.plan;
    let mut cells: Vec<Cell> = plan
        .cells
        .iter()
        .filter(|(cell, symbol)| {
            **symbol == '.'
                && !in_room.contains(cell)
                && DIRECTIONS.iter().all(|(dx, dy)| {
                    let s = plan.get(cell.offset(*dx, *dy, 0));
                    s != 'D' && s != '_' && plan::stairs_direction(s).is_none()
                })
        })
        .map(|(cell, _)| *cell)
        .collect();
    shuffle(&mut cells, &mut rng);
    let corridors = &settings.corridors;
    for _ in 0..cells.len() / corridors.per_cells.max(1) {
        for (kind, marker) in KINDS {
            let Some(range) = corridors.creatures.get(kind) else {
                continue;
            };
            for _ in 0..range.roll(&mut rng).max(0) {
                let Some(cell) = cells.pop() else {
                    break;
                };
                plan.cells.insert(cell, marker);
                creatures += 1;
            }
        }
    }
    (creatures, chests)
}

#[cfg(test)]
mod tests {
    use super::{
        super::{assemble, graph, rooms::RoomCatalog, settings::*},
        *,
    };

    #[test]
    fn people_follow_the_roles() {
        let settings = Settings::load().unwrap();
        let catalog = RoomCatalog::load().unwrap().with_orientations(settings.variety.orientations).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        for seed in 0..40 {
            let graph = graph::generate(&resolved, seed);
            let Some(mut dungeon) =
                assemble::assemble(&graph, &catalog, seed, &settings.assembly, 4, |_| {})
            else {
                continue;
            };
            let (creatures, _) = populate(&mut dungeon, &graph, &resolved, seed);
            assert!(creatures > 0);
            let symbols: Vec<char> = dungeon.plan.cells.values().copied().collect();
            // Exactly one chief, in the goal room.
            assert_eq!(symbols.iter().filter(|s| s.eq_ignore_ascii_case(&'c')).count(), 1, "graine {seed}");
            let goal = &dungeon.rooms[graph.goal];
            assert!(goal.footprint().any(|c| dungeon.plan.get(c) == 'c'));
            // Nobody next to a door.
            for (cell, symbol) in &dungeon.plan.cells {
                if kind_of(*symbol).is_some() || *symbol == '$' {
                    assert!(
                        DIRECTIONS.iter().all(|(dx, dy)| dungeon.plan.get(cell.offset(*dx, *dy, 0)) != 'D'),
                        "graine {seed} : {symbol} devant une porte en {cell:?}"
                    );
                }
            }
            // Still finishable.
            assert!(dungeon.check().is_empty());
        }
    }

    /// Icebreaker F.4: a medium dungeon by default holds 30 to 50 creatures,
    /// some of them in the corridors.
    #[test]
    fn dungeons_are_lively() {
        let settings = Settings::load().unwrap();
        let catalog = RoomCatalog::load().unwrap().with_orientations(settings.variety.orientations).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let (mut total, mut in_corridors, mut dungeons) = (0, 0, 0);
        for seed in 0..40 {
            let graph = graph::generate(&resolved, seed);
            let Some(mut dungeon) = assemble::assemble(&graph, &catalog, seed, &settings.assembly, 4, |_| {})
            else {
                continue;
            };
            let (creatures, _) = populate(&mut dungeon, &graph, &resolved, seed);
            let in_room: BTreeSet<Cell> = dungeon.rooms.iter().flat_map(|r| r.footprint()).collect();
            for (cell, symbol) in &dungeon.plan.cells {
                if kind_of(*symbol).is_some() && !in_room.contains(cell) {
                    in_corridors += 1;
                    // Not on stairs, nor next to a door, stairs or a hole.
                    for (dx, dy) in DIRECTIONS {
                        let s = dungeon.plan.get(cell.offset(dx, dy, 0));
                        assert!(s != 'D' && s != '_' && plan::stairs_direction(s).is_none(), "graine {seed} : {cell:?}");
                    }
                }
            }
            total += creatures;
            dungeons += 1;
            assert!(dungeon.check().is_empty());
        }
        let mean = total as f32 / dungeons as f32;
        println!("medium : {mean:.1} créatures en moyenne, dont {in_corridors} en couloir sur {dungeons} donjons");
        assert!((30.0..=50.0).contains(&mean), "medium : {mean:.1} créatures en moyenne");
        assert!(in_corridors >= dungeons, "{in_corridors} créatures de couloir sur {dungeons} donjons");
    }
}
