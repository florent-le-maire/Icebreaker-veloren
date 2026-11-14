//! Icebreaker Y.3: the rounds of a race in the generated dungeons
//! (`assets/icebreaker/ennemis/races.ron` → `ronde`, planned by
//! `world::icebreaker::dungeon::rondes`). The members of a round walk together
//! to its next stop (an rtsim « go to » the agent follows when idle), wait
//! there a moment, then go on; at the end of the route they come back. A
//! member fighting is left alone and takes the round up again after.

use crate::icebreaker_mission::IcebreakerMission;
use common::{
    comp::{Agent, Health, Pos},
    icebreaker_races::Ronde,
    resources::Time,
    rtsim::{DungeonSlot, NpcActivity},
    store::Id,
};
use common_ecs::{Job, Origin, Phase, System};
use hashbrown::HashMap;
use specs::{Entities, Entity, Join, LendJoin, Read, ReadStorage, Write, WriteStorage};
use vek::*;
use world::site::Site;

/// Where a round is on its route.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tour {
    pub etape: usize,
    /// Going back toward the first stop.
    pub retour: bool,
    /// When the first member reached the current stop.
    pub arrivee: Option<f64>,
}

impl Tour {
    /// The next stop of a route of `n` stops, back and forth.
    fn suivante(&mut self, n: usize) {
        if n < 2 {
            return;
        }
        if self.retour && self.etape == 0 {
            self.retour = false;
        } else if !self.retour && self.etape + 1 == n {
            self.retour = true;
        }
        if self.retour { self.etape -= 1 } else { self.etape += 1 }
    }

    /// Moves the round on: once a member (`members`, their positions) has
    /// been at the stop `stop` for `ronde.pause` seconds, the next stop.
    pub fn avance(&mut self, ronde: &Ronde, n: usize, stop: Vec3<f32>, members: &[Vec3<f32>], now: f64) {
        let there = members.iter().any(|m| m.distance(stop) < ronde.arrivee);
        match self.arrivee {
            None if there => self.arrivee = Some(now),
            Some(t) if now - t >= ronde.pause => {
                self.suivante(n);
                self.arrivee = None;
            },
            _ => {},
        }
    }
}

/// The rounds going on: by dungeon and first member (a new plan, new rounds).
#[derive(Default)]
pub struct IcebreakerRondes(HashMap<(Id<Site>, Vec3<i32>), Tour>);

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, IcebreakerMission>,
        Write<'a, IcebreakerRondes>,
        Read<'a, Time>,
        ReadStorage<'a, DungeonSlot>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Health>,
        WriteStorage<'a, Agent>,
    );

    const NAME: &'static str = "icebreaker_rondes";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (entities, missions, mut rondes, time, slots, positions, healths, mut agents): Self::SystemData,
    ) {
        if missions.iter().all(|(_, m)| m.rondes.is_empty() || m.ronde.is_none()) {
            return;
        }
        // The living creatures of the dungeons, by the slot where they appeared.
        let by_slot: HashMap<Vec3<i32>, Entity> = (&entities, &slots, &agents, healths.maybe())
            .join()
            .filter(|(_, slot, _, health)| slot.nth == 0 && health.is_none_or(|h| !h.is_dead))
            .map(|(entity, slot, _, _)| (slot.wpos, entity))
            .collect();

        let mut seen = Vec::new();
        for (site, mission) in missions.iter() {
            let Some(ronde) = &mission.ronde else { continue };
            for (route, members) in &mission.rondes {
                let (Some(first), false) = (members.first(), route.is_empty()) else {
                    continue;
                };
                let key = (site, *first);
                seen.push(key);
                let tour = rondes.0.entry(key).or_default();
                let present: Vec<Entity> = members.iter().filter_map(|m| by_slot.get(m).copied()).collect();
                let stop = route[tour.etape.min(route.len() - 1)].as_::<f32>() + Vec3::new(0.5, 0.5, 0.0);
                let walking: Vec<Vec3<f32>> = present
                    .iter()
                    .filter(|e| agents.get(**e).is_some_and(|a| a.target.is_none() && !a.icebreaker_asleep))
                    .filter_map(|e| positions.get(*e).map(|p| p.0))
                    .collect();
                let before = tour.etape;
                tour.avance(ronde, route.len(), stop, &walking, time.0);
                if tour.etape != before {
                    tracing::debug!(?first, etape = tour.etape, "Icebreaker : la ronde repart vers la salle suivante");
                }
                let stop = route[tour.etape.min(route.len() - 1)].as_::<f32>() + Vec3::new(0.5, 0.5, 0.0);
                for (i, entity) in present.iter().enumerate() {
                    if let Some(agent) = agents.get_mut(*entity) {
                        // Side by side, not on one another.
                        let offset = Vec3::new((i as f32 - 1.0) * 1.5, 0.0, 0.0);
                        agent.rtsim_controller.activity = Some(NpcActivity::Goto(stop + offset, ronde.vitesse));
                    }
                }
            }
        }
        rondes.0.retain(|key, _| seen.contains(key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ronde() -> Ronde {
        Ronde {
            groupes: 1,
            membres: 3,
            etapes: 3,
            vitesse: 0.5,
            pause: 4.0,
            arrivee: 3.0,
        }
    }

    #[test]
    fn la_ronde_fait_l_aller_et_le_retour() {
        let mut t = Tour::default();
        let etapes: Vec<usize> = (0..6)
            .map(|_| {
                t.suivante(3);
                t.etape
            })
            .collect();
        assert_eq!(etapes, vec![1, 2, 1, 0, 1, 2]);
    }

    #[test]
    fn la_ronde_s_arrete_avant_de_repartir() {
        let r = ronde();
        let stop = Vec3::new(10.0, 0.0, 0.0);
        let mut t = Tour::default();
        // Loin : on n'y est pas.
        t.avance(&r, 3, stop, &[Vec3::zero()], 0.0);
        assert_eq!(t, Tour::default());
        // Arrivé, puis la pause.
        t.avance(&r, 3, stop, &[Vec3::new(9.0, 0.0, 0.0)], 1.0);
        assert_eq!(t.arrivee, Some(1.0));
        t.avance(&r, 3, stop, &[Vec3::new(9.0, 0.0, 0.0)], 3.0);
        assert_eq!(t.etape, 0);
        t.avance(&r, 3, stop, &[Vec3::new(9.0, 0.0, 0.0)], 5.0);
        assert_eq!((t.etape, t.arrivee), (1, None));
    }
}
