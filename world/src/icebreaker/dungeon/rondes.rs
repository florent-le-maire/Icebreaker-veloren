//! Icebreaker Y.3: the rounds of a race (`assets/icebreaker/ennemis/races.ron`
//! → `ronde`). The awake warriors (`b`) of a room leave together toward the
//! next rooms, through passages one can walk both ways without a key, then
//! come back the same way (the server moves them: `server/src/sys/icebreaker_rondes.rs`).

use super::{
    assemble::Assembled,
    graph::{MissionGraph, PassageKind},
    plan::Cell,
    populate,
};
use common::icebreaker_races::Ronde;

/// A round: the cells of its stops (the middle of each room), and the cells
/// where its members appear.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedRound {
    pub route: Vec<Cell>,
    pub members: Vec<Cell>,
}

/// The floor cell nearest the middle of a room: a free floor or a creature.
fn middle(dungeon: &Assembled, room: usize) -> Option<Cell> {
    let cells: Vec<Cell> = dungeon.rooms.get(room)?.footprint().collect();
    let walkable: Vec<Cell> = cells
        .iter()
        .copied()
        .filter(|c| {
            let s = dungeon.plan.get(*c);
            s == '.' || populate::kind_of(s).is_some()
        })
        .collect();
    let n = walkable.len().max(1) as f32;
    let (sx, sy) = walkable.iter().fold((0.0, 0.0), |(x, y), c| (x + c.x as f32, y + c.y as f32));
    let (mx, my) = (sx / n, sy / n);
    walkable.into_iter().min_by(|a, b| {
        let d = |c: &Cell| (c.x as f32 - mx).powi(2) + (c.y as f32 - my).powi(2);
        d(a).total_cmp(&d(b)).then(a.cmp(b))
    })
}

/// The rooms of a round from `home`: each next one a neighbour not yet
/// visited, through a corridor or stairs without a lock.
fn rooms_from(graph: &MissionGraph, home: usize, stops: usize) -> Vec<usize> {
    let mut route = vec![home];
    while route.len() < stops {
        let last = *route.last().unwrap();
        let next = graph
            .passages
            .iter()
            .filter(|p| p.lock.is_none() && p.kind != PassageKind::Drop && (p.from == last || p.to == last))
            .map(|p| p.other(last))
            .filter(|r| !route.contains(r))
            .min();
        match next {
            Some(r) => route.push(r),
            None => break,
        }
    }
    route
}

/// The rounds of a peopled dungeon: up to `ronde.groupes` rooms (not the goal
/// room) with awake warriors, each one going through at least two rooms.
pub fn place_rounds(dungeon: &Assembled, graph: &MissionGraph, ronde: &Ronde) -> Vec<PlannedRound> {
    let mut rounds = Vec::new();
    for (room, placed) in dungeon.rooms.iter().enumerate() {
        if rounds.len() >= ronde.groupes {
            break;
        }
        if room == graph.goal {
            continue;
        }
        let mut members: Vec<Cell> = placed.footprint().filter(|c| dungeon.plan.get(*c) == 'b').collect();
        members.sort();
        members.truncate(ronde.membres);
        if members.is_empty() {
            continue;
        }
        let route: Vec<Cell> = rooms_from(graph, room, ronde.etapes)
            .into_iter()
            .filter_map(|r| middle(dungeon, r))
            .collect();
        if route.len() >= 2 {
            rounds.push(PlannedRound { route, members });
        }
    }
    rounds
}

#[cfg(test)]
mod tests {
    use super::{
        super::{assemble, graph, settings::*},
        *,
    };

    fn ronde() -> Ronde {
        Ronde {
            groupes: 2,
            membres: 3,
            etapes: 3,
            vitesse: 0.5,
            pause: 4.0,
            arrivee: 3.0,
        }
    }

    #[test]
    fn rounds_go_through_rooms_one_can_walk() {
        let settings = Settings::load().unwrap();
        let catalog = settings.catalog(0).unwrap();
        let resolved = settings.resolve(&Mission::default()).unwrap();
        let mut with_rounds = 0;
        for seed in 0..30 {
            let graph = graph::generate(&resolved, seed);
            let Some(mut dungeon) = assemble::assemble(&graph, &catalog, seed, &settings.assembly, 4, |_| {})
            else {
                continue;
            };
            populate::populate(&mut dungeon, &graph, &resolved, seed);
            let rounds = place_rounds(&dungeon, &graph, &ronde());
            assert!(rounds.len() <= 2);
            with_rounds += usize::from(!rounds.is_empty());
            for round in &rounds {
                assert!((2..=3).contains(&round.route.len()), "graine {seed}");
                assert!((1..=3).contains(&round.members.len()));
                // Members are awake warriors, stops are floors.
                assert!(round.members.iter().all(|c| dungeon.plan.get(*c) == 'b'));
                for stop in &round.route {
                    let s = dungeon.plan.get(*stop);
                    assert!(s == '.' || populate::kind_of(s).is_some(), "graine {seed} : arrêt sur {s:?}");
                }
                // Never in the goal room.
                let goal: Vec<Cell> = dungeon.rooms[graph.goal].footprint().collect();
                assert!(round.members.iter().all(|c| !goal.contains(c)));
            }
        }
        assert!(with_rounds > 10, "{with_rounds} donjons sur 30 ont une ronde");
    }
}
