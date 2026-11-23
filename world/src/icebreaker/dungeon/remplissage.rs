//! Icebreaker F.52 : the maze that fills the outline of a generated dungeon, an idea taken from donjon.bin.sh (not
//! its code): once the rooms of the mission are placed and linked, corridors are dug in all the rock left inside
//! the outline, on every level, until nothing is left (`assembly.remplissage`).
//!
//! A piece of maze is a tree: it starts from one floor cell of a corridor, through its wall, and grows two cells
//! at a time (a corridor, a wall, a corridor). A cell is dug only if nothing one walks on touches it but the cell
//! it comes from: the maze touches the dungeon at the foot of each tree only, so it never goes around a locked
//! door, and one always walks back out of it. It may loop on itself (`boucles_labyrinthe`), and some of its dead
//! ends are filled again (`elagage`).

use super::plan::{self, Cell, DIRECTIONS, Plan};
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeMap, BTreeSet};

/// What `assembly` says of the maze.
#[derive(Copy, Clone, Debug)]
pub struct Reglages {
    /// Share of the rock left that the maze takes, out of 100 (100: until nothing is left).
    pub part: i32,
    /// Chances out of 100 that the maze goes on from where it is (long winding corridors) rather than from
    /// anywhere (short branches).
    pub suite: i32,
    /// Walls opened between two corridors of a same tree, for 100 cells of maze.
    pub boucles: i32,
    /// Dead ends filled again, out of 100.
    pub elagage: i32,
}

/// Cells of corridor for one of the first trees of the maze.
const ROOT_EVERY: usize = 8;

fn solid(grid: &Plan, in_rooms: &BTreeSet<Cell>, c: Cell) -> bool {
    grid.is_free(c) || (grid.get(c) == '#' && !in_rooms.contains(&c))
}

fn quiet(grid: &Plan, c: Cell) -> bool { grid.is_free(c) || grid.get(c) == '#' }

fn trodden(grid: &Plan, c: Cell) -> bool {
    let s = grid.get(c);
    plan::is_walkable(s) || s == '_' || plan::stairs_direction(s).is_some()
}

/// Digs the maze in `grid` from the floor cells `roots` of its corridors, inside `dedans` and the box `boite`
/// (min x, max x, min y, max y); returns its cells.
pub fn creuser(
    grid: &mut Plan,
    in_rooms: &BTreeSet<Cell>,
    roots: &[Cell],
    dedans: &dyn Fn(Cell) -> bool,
    boite: (i32, i32, i32, i32),
    reglages: Reglages,
    rng: &mut ChaCha8Rng,
) -> BTreeSet<Cell> {
    let (min_x, max_x, min_y, max_y) = boite;
    let inside = |c: Cell| (min_x..=max_x).contains(&c.x) && (min_y..=max_y).contains(&c.y) && dedans(c);
    let levels: BTreeSet<i32> = grid.cells.keys().map(|c| c.level).collect();
    let rock = levels
        .iter()
        .flat_map(|l| (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| Cell::new(x, y, *l))))
        .filter(|c| inside(*c) && grid.is_free(*c))
        .count();
    // Corridors, their walls, what the rooms leave: a third of the rock or so becomes floor.
    let mut budget = if reglages.part >= 100 { usize::MAX } else { rock * reglages.part.max(0) as usize / 300 };

    // The cell `c` may be dug coming from `before`.
    let fine = |grid: &Plan, c: Cell, before: Cell| {
        inside(c)
            && solid(grid, in_rooms, c)
            && grid.get(c.offset(0, 0, 1)) != '_'
            && DIRECTIONS.iter().all(|(dx, dy)| {
                let n = c.offset(*dx, *dy, 0);
                n == before || quiet(grid, n)
            })
    };

    let mut tree: BTreeMap<Cell, usize> = BTreeMap::new();
    let mut order: Vec<Cell> = roots.to_vec();
    super::graph::shuffle(&mut order, rng);
    // One step of the tree `id` from one of its cells `active`; false when it can't grow any more.
    let grow = |grid: &mut Plan, tree: &mut BTreeMap<Cell, usize>, id: usize, active: &mut Vec<Cell>, rng: &mut ChaCha8Rng| {
        while !active.is_empty() {
            let at = if rng.random_range(0..100) < reglages.suite {
                active.len() - 1
            } else {
                rng.random_range(0..active.len())
            };
            let from = active[at];
            let mut ways = DIRECTIONS;
            super::graph::shuffle(&mut ways, rng);
            let way = ways.into_iter().find(|(dx, dy)| {
                let (wall, next) = (from.offset(*dx, *dy, 0), from.offset(2 * dx, 2 * dy, 0));
                fine(grid, wall, from) && fine(grid, next, wall)
            });
            let Some((dx, dy)) = way else {
                active.swap_remove(at);
                continue;
            };
            for cell in [from.offset(dx, dy, 0), from.offset(2 * dx, 2 * dy, 0)] {
                grid.cells.insert(cell, '.');
                tree.insert(cell, id);
            }
            active.push(from.offset(2 * dx, 2 * dy, 0));
            return true;
        }
        false
    };
    // Some of the roots first (one for `ROOT_EVERY` cells of corridor, one a level at least), their trees growing
    // in turn: the maze spreads over the whole dungeon, even when it does not take all the rock. Then the other
    // roots, for the rock the first ones could not reach.
    let mut first: Vec<Cell> = Vec::new();
    for level in &levels {
        first.extend(order.iter().find(|c| c.level == *level));
    }
    first.extend(order.iter().filter(|c| !first.contains(c)).take(order.len() / ROOT_EVERY).copied().collect::<Vec<_>>());
    let mut growing: Vec<(usize, Vec<Cell>)> = first.iter().map(|root| vec![*root]).enumerate().collect();
    while !growing.is_empty() && budget >= 2 {
        let turn = rng.random_range(0..growing.len());
        let (id, active) = &mut growing[turn];
        if grow(grid, &mut tree, *id, active, rng) {
            budget -= 2;
        } else {
            growing.swap_remove(turn);
        }
    }
    for (id, root) in order.into_iter().enumerate().filter(|(_, root)| !first.contains(root)) {
        let mut active = vec![root];
        while budget >= 2 && grow(grid, &mut tree, first.len() + id, &mut active, rng) {
            budget -= 2;
        }
    }

    // Loops: a wall between two corridors of the same tree, opened.
    let wanted = tree.len() * reglages.boucles.max(0) as usize / 100;
    if wanted > 0 {
        let mut walls: Vec<Cell> = tree
            .keys()
            .flat_map(|c| DIRECTIONS.map(|(dx, dy)| c.offset(dx, dy, 0)))
            .filter(|w| solid(grid, in_rooms, *w))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        super::graph::shuffle(&mut walls, rng);
        let mut opened = 0;
        for wall in walls {
            if opened >= wanted {
                break;
            }
            let joins = [((1, 0), (0, 1)), ((0, 1), (1, 0))].into_iter().any(|((dx, dy), (sx, sy))| {
                let (a, b) = (wall.offset(dx, dy, 0), wall.offset(-dx, -dy, 0));
                tree.get(&a).is_some()
                    && tree.get(&a) == tree.get(&b)
                    && quiet(grid, wall.offset(sx, sy, 0))
                    && quiet(grid, wall.offset(-sx, -sy, 0))
            });
            if joins && inside(wall) && grid.get(wall.offset(0, 0, 1)) != '_' {
                let id = tree[&DIRECTIONS
                    .map(|(dx, dy)| wall.offset(dx, dy, 0))
                    .into_iter()
                    .find(|n| tree.contains_key(n))
                    .expect("joins two cells of a tree")];
                grid.cells.insert(wall, '.');
                tree.insert(wall, id);
                opened += 1;
            }
        }
    }

    // Dead ends filled again, back to the fork they leave from.
    if reglages.elagage > 0 {
        let exits = |grid: &Plan, c: Cell| {
            DIRECTIONS.iter().map(|(dx, dy)| c.offset(*dx, *dy, 0)).filter(|n| trodden(grid, *n)).collect::<Vec<_>>()
        };
        let ends: Vec<Cell> = tree.keys().copied().filter(|c| exits(grid, *c).len() == 1).collect();
        for end in ends {
            if rng.random_range(0..100) >= reglages.elagage {
                continue;
            }
            let mut at = end;
            while tree.contains_key(&at) {
                let ways = exits(grid, at);
                if ways.len() != 1 {
                    break;
                }
                grid.cells.insert(at, '#');
                tree.remove(&at);
                at = ways[0];
            }
        }
    }

    // Walls around what was dug.
    for cell in tree.keys() {
        for (dx, dy) in DIRECTIONS {
            grid.cells.entry(cell.offset(dx, dy, 0)).or_insert('#');
        }
    }
    tree.into_keys().collect()
}

#[cfg(test)]
mod tests {
    use super::{
        super::{assemble::assemble, graph, settings::*, silhouette::Silhouette},
        *,
    };

    /// The maze fills the outline, each piece of it touches the dungeon by one cell only (no way around a locked
    /// door), and the dungeon stays sound; pruned and looped, the same.
    #[test]
    fn the_maze_fills_the_outline_and_gets_around_nothing() {
        let mut settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        for (silhouette, boucles, elagage) in [
            (Silhouette::Carre, 0, 0),
            (Silhouette::Croix, 5, 0),
            (Silhouette::Rond, 0, 50),
            (Silhouette::Couronne, 5, 50),
            (Silhouette::Libre, 3, 20),
        ] {
            settings.assembly.silhouette = silhouette;
            settings.assembly.remplissage = 100;
            settings.assembly.boucles_labyrinthe = boucles;
            settings.assembly.elagage = elagage;
            let (mut cover, mut maze, mut dungeons) = (0, 0, 0);
            for seed in 0..12 {
                let graph = graph::generate(&resolved, seed);
                let Some(dungeon) = assemble(&graph, &catalog, seed, &settings.assembly, |_| {}) else {
                    continue;
                };
                assert!(dungeon.check().is_empty(), "{silhouette:?}, graine {seed}");
                let plan = &dungeon.plan;
                let in_rooms: BTreeSet<Cell> = dungeon.rooms.iter().flat_map(|r| r.footprint()).collect();
                assert!(dungeon.labyrinthe.iter().all(|c| plan.get(*c) == '.' && !in_rooms.contains(c)));
                // Each piece of maze: one foot.
                let mut seen = BTreeSet::new();
                for start in &dungeon.labyrinthe {
                    if !seen.insert(*start) {
                        continue;
                    }
                    let (mut queue, mut feet) = (vec![*start], BTreeSet::new());
                    while let Some(cell) = queue.pop() {
                        for (dx, dy) in DIRECTIONS {
                            let n = cell.offset(dx, dy, 0);
                            if dungeon.labyrinthe.contains(&n) {
                                if seen.insert(n) {
                                    queue.push(n);
                                }
                            } else if trodden(plan, n) {
                                feet.insert(n);
                            }
                        }
                        assert!(plan.get(cell.offset(0, 0, 1)) != '_', "{cell:?} sous un trou");
                    }
                    assert_eq!(feet.len(), 1, "{silhouette:?}, graine {seed} : un morceau de labyrinthe à {feet:?}");
                }
                cover += dungeon.couverture(silhouette).map_or(0, |c| c.0);
                maze += dungeon.labyrinthe.len();
                dungeons += 1;
            }
            assert!(dungeons >= 8, "{silhouette:?} : {dungeons} donjons sur 12");
            println!(
                "{silhouette:?} : {} % des cases, {} cases de labyrinthe par donjon ({dungeons} donjons)",
                cover / dungeons,
                maze / dungeons as usize
            );
            assert!(maze > 0);
            // Icebreaker F.54 : the whole outline, but for the corners nothing leads to.
            assert!(silhouette == Silhouette::Libre || cover / dungeons >= 95, "{silhouette:?} : {} %", cover / dungeons);
        }
    }

    /// Half the filling: about half the maze.
    #[test]
    fn the_share_of_the_maze_is_followed() {
        let mut settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        settings.assembly.silhouette = Silhouette::Carre;
        let graph = graph::generate(&resolved, 1);
        let mut cells = |part: i32| {
            settings.assembly.remplissage = part;
            assemble(&graph, &catalog, 1, &settings.assembly, |_| {}).unwrap().labyrinthe.len()
        };
        let (none, half, all) = (cells(0), cells(50), cells(100));
        println!("labyrinthe : {none} cases à 0 %, {half} à 50 %, {all} à 100 %");
        assert_eq!(none, 0);
        assert!(half * 10 >= all * 3 && half * 10 <= all * 7, "{half} cases à 50 % pour {all} à 100 %");
    }
}
