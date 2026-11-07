//! Icebreaker: the asleep creatures of a generated dungeon (G.9). They lie on
//! their back, do nothing (`Agent::icebreaker_asleep`, skipped by the agent
//! system) and snore « Zzz » in a speech bubble. They wake up when hit, when
//! they hear a fight (a sound reaching them) or when a player comes close
//! (closer if sneaking); one waking up wakes the sleepers around it.

use common::{
    comp::{Agent, CharacterState, Content, Health, Ori, Player, Pos, UnresolvedChatMsg, agent::AgentEvent},
    event::{ChatEvent, EventBus},
    resources::Time,
    rtsim::DungeonSlot,
    store::Id,
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use hashbrown::{HashMap, HashSet};
use specs::{Entities, Entity, Join, LendJoin, Read, ReadStorage, Write, WriteStorage};
use std::f32::consts::FRAC_PI_2;
use world::site::Site;
use vek::*;

/// How asleep creatures wake up (`generation.ron` → `sleep`).
#[derive(Clone, Debug, Default)]
pub struct SleepRules {
    pub wake_radius: f32,
    pub wake_radius_sneaking: f32,
    pub alert_radius: f32,
    pub zzz_interval: f64,
}

impl SleepRules {
    /// Whether a sleeper at `pos` wakes up: hit, hearing a sound, or a player
    /// (position, sneaking) close enough.
    pub fn wakes(&self, pos: Vec3<f32>, hit: bool, heard: bool, players: &[(Vec3<f32>, bool)]) -> bool {
        hit || heard
            || players.iter().any(|(player, sneaking)| {
                let radius = if *sneaking { self.wake_radius_sneaking } else { self.wake_radius };
                player.distance_squared(pos) < radius.powi(2)
            })
    }
}

struct Sleeper {
    slot: Vec3<i32>,
    /// Its health when it fell asleep: less means it was hit.
    health: f32,
    /// Its orientation standing, given back when it wakes up.
    standing: Ori,
    next_zzz: f64,
}

/// Icebreaker: who sleeps in the generated dungeons of this server, by site
/// (one per dungeon slot of the campaign, I.10).
#[derive(Default)]
pub struct IcebreakerSleep {
    /// Spawn positions (dungeon slots) of the creatures asleep at first, by
    /// dungeon.
    spots: Vec<(Id<Site>, HashSet<Vec3<i32>>)>,
    pub(crate) rules: SleepRules,
    /// Slots already woken up: they don't fall asleep again.
    woken: HashSet<Vec3<i32>>,
    asleep: HashMap<Entity, Sleeper>,
}

impl IcebreakerSleep {
    pub fn new(rules: SleepRules) -> Self {
        Self {
            rules,
            ..Self::default()
        }
    }

    /// Sets who sleeps at first in the dungeon of `site`; the slots of its
    /// former sleepers may fall asleep again.
    pub fn set(&mut self, site: Id<Site>, spots: HashSet<Vec3<i32>>) {
        if let Some(i) = self.spots.iter().position(|(s, _)| *s == site) {
            let (_, old) = self.spots.swap_remove(i);
            self.woken.retain(|wpos| !old.contains(wpos));
        }
        if !spots.is_empty() {
            self.spots.push((site, spots));
        }
    }

    fn sleeps(&self, wpos: &Vec3<i32>) -> bool {
        !self.woken.contains(wpos) && self.spots.iter().any(|(_, spots)| spots.contains(wpos))
    }
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Write<'a, IcebreakerSleep>,
        Read<'a, Time>,
        Read<'a, EventBus<ChatEvent>>,
        ReadStorage<'a, DungeonSlot>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, CharacterState>,
        WriteStorage<'a, Agent>,
        WriteStorage<'a, Ori>,
    );

    const NAME: &'static str = "icebreaker_sleep";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            mut sleep,
            time,
            chat_bus,
            slots,
            positions,
            healths,
            uids,
            players,
            char_states,
            mut agents,
            mut orientations,
        ): Self::SystemData,
    ) {
        if sleep.spots.is_empty() {
            return;
        }
        let sleep = &mut *sleep;
        let interval = sleep.rules.zzz_interval;

        // Creatures of the asleep slots fall asleep once they appear.
        for (entity, slot, agent, health, ori) in
            (&entities, &slots, &mut agents, healths.maybe(), &orientations).join()
        {
            if agent.icebreaker_asleep
                || slot.nth != 0
                || !sleep.sleeps(&slot.wpos)
            {
                continue;
            }
            agent.icebreaker_asleep = true;
            // Not all snoring together.
            let offset = interval * (entity.id() % 5) as f64 / 5.0;
            sleep.asleep.insert(entity, Sleeper {
                slot: slot.wpos,
                health: health.map_or(0.0, |h| h.current()),
                standing: ori.to_horizontal(),
                next_zzz: time.0 + offset,
            });
        }
        sleep.asleep.retain(|entity, _| entities.is_alive(*entity));

        // Who wakes up, then the sleepers around them.
        let players: Vec<(Vec3<f32>, bool)> = (&players, &positions, char_states.maybe())
            .join()
            .map(|(_, pos, state)| (pos.0, state.is_some_and(|s| s.is_stealthy())))
            .collect();
        let waking: Vec<Vec3<f32>> = sleep
            .asleep
            .iter()
            .filter_map(|(entity, sleeper)| {
                let pos = positions.get(*entity)?.0;
                let hit = healths.get(*entity).is_some_and(|h| h.current() < sleeper.health);
                let heard = agents.get(*entity).is_some_and(|a| {
                    a.inbox.iter().any(|event| matches!(event, AgentEvent::ServerSound(_)))
                });
                sleep.rules.wakes(pos, hit, heard, &players).then_some(pos)
            })
            .collect();
        let alert = sleep.rules.alert_radius;
        let woken: Vec<Entity> = sleep
            .asleep
            .keys()
            .copied()
            .filter(|entity| {
                positions.get(*entity).is_some_and(|pos| {
                    waking.iter().any(|w| w.distance_squared(pos.0) <= alert.powi(2))
                })
            })
            .collect();

        let mut chat = chat_bus.emitter();
        let mut say = |entity: Entity, text: &str| {
            if let Some(uid) = uids.get(entity) {
                chat.emit(ChatEvent {
                    msg: UnresolvedChatMsg::npc(*uid, Content::Plain(text.to_string())),
                    from_client: false,
                });
            }
        };
        for entity in woken {
            let Some(sleeper) = sleep.asleep.remove(&entity) else { continue };
            sleep.woken.insert(sleeper.slot);
            if let Some(agent) = agents.get_mut(entity) {
                agent.icebreaker_asleep = false;
            }
            if let Some(ori) = orientations.get_mut(entity) {
                *ori = sleeper.standing;
            }
            say(entity, "Hein ?!");
        }

        // The others lie on their back and snore.
        for (entity, sleeper) in sleep.asleep.iter_mut() {
            if let Some(ori) = orientations.get_mut(*entity) {
                *ori = sleeper.standing.pitched_up(FRAC_PI_2);
            }
            if let Some(agent) = agents.get_mut(*entity) {
                agent.inbox.retain(|event| !matches!(event, AgentEvent::ServerSound(_)));
            }
            if time.0 >= sleeper.next_zzz {
                sleeper.next_zzz = time.0 + interval;
                say(*entity, "Zzz…");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleepers_wake_when_hit_heard_or_approached() {
        let rules = SleepRules {
            wake_radius: 6.0,
            wake_radius_sneaking: 2.5,
            alert_radius: 12.0,
            zzz_interval: 5.0,
        };
        let pos = Vec3::new(10.0, 10.0, 0.0);
        let at = |dx: f32| Vec3::new(10.0 + dx, 10.0, 0.0);
        assert!(!rules.wakes(pos, false, false, &[]));
        assert!(rules.wakes(pos, true, false, &[]));
        assert!(rules.wakes(pos, false, true, &[]));
        assert!(rules.wakes(pos, false, false, &[(at(5.0), false)]));
        assert!(!rules.wakes(pos, false, false, &[(at(7.0), false)]));
        // Sneaking, one gets much closer.
        assert!(!rules.wakes(pos, false, false, &[(at(4.0), true)]));
        assert!(rules.wakes(pos, false, false, &[(at(2.0), true)]));
    }

    #[test]
    fn each_dungeon_has_its_sleepers() {
        let spot = |x| Vec3::new(x, 0, 10);
        let mut sleep = IcebreakerSleep::new(SleepRules::default());
        sleep.set(Id::new(0), [spot(1)].into_iter().collect());
        sleep.set(Id::new(1), [spot(1000), spot(1001)].into_iter().collect());
        assert!(sleep.sleeps(&spot(1)) && sleep.sleeps(&spot(1000)));
        sleep.woken.insert(spot(1000));
        assert!(!sleep.sleeps(&spot(1000)));
        // A new plan for dungeon 1: its old sleepers are gone, and forgotten.
        sleep.set(Id::new(1), [spot(2000)].into_iter().collect());
        assert!(!sleep.sleeps(&spot(1001)) && sleep.sleeps(&spot(2000)));
        assert!(sleep.woken.is_empty());
        assert!(sleep.sleeps(&spot(1)));
    }
}
