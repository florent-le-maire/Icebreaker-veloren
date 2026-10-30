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
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use hashbrown::{HashMap, HashSet};
use specs::{Entities, Entity, Join, LendJoin, Read, ReadStorage, Write, WriteStorage};
use std::f32::consts::FRAC_PI_2;
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

/// Icebreaker: who sleeps in the generated dungeon of this server.
#[derive(Default)]
pub struct IcebreakerSleep {
    /// Spawn positions (dungeon slots) of the creatures asleep at first.
    spots: HashSet<Vec3<i32>>,
    rules: SleepRules,
    /// Slots already woken up: they don't fall asleep again.
    woken: HashSet<Vec3<i32>>,
    asleep: HashMap<Entity, Sleeper>,
}

impl IcebreakerSleep {
    pub fn new(spots: HashSet<Vec3<i32>>, rules: SleepRules) -> Self {
        Self {
            spots,
            rules,
            ..Self::default()
        }
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
                || !sleep.spots.contains(&slot.wpos)
                || sleep.woken.contains(&slot.wpos)
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
}
