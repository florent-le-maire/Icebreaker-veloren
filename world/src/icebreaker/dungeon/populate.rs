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

/// The kinds of creatures of the plans, by marker (lowercase: awake).
pub const KINDS: [(&str, char); 5] = [
    ("boss", 'x'),
    ("chief", 'c'),
    ("archer", 'a'),
    ("brute", 'b'),
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
        let catalog = RoomCatalog::load().unwrap();
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
}
