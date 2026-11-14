//! Icebreaker Y.9: how many enemies aim at each target, counted every tick
//! for the agents (`common::icebreaker_races::IcebreakerFocus`): they run in
//! parallel and cannot see the targets of one another. An enemy choosing its
//! target reads it to spread among the players (`choose_target`).
//! `RUST_LOG=veloren_server::sys::icebreaker_focus=debug` logs every change
//! (target, enemies on it, whether it is downed).

use common::{
    comp::{Agent, Alignment, CharacterState, Health, Player, is_downed},
    icebreaker_races::IcebreakerFocus,
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use specs::{Join, ReadStorage, Write};

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Write<'a, IcebreakerFocus>,
        ReadStorage<'a, Agent>,
        ReadStorage<'a, Alignment>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, CharacterState>,
        ReadStorage<'a, Player>,
    );

    const NAME: &'static str = "icebreaker_focus";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(_job: &mut Job<Self>, (mut focus, agents, alignments, uids, healths, states, players): Self::SystemData) {
        let debug = tracing::enabled!(tracing::Level::DEBUG);
        let before = if debug { focus.0.clone() } else { Default::default() };
        focus.0.clear();
        for (agent, _) in (&agents, &alignments).join().filter(|(_, a)| matches!(a, Alignment::Enemy)) {
            let Some(target) = agent.target.filter(|t| t.hostile) else { continue };
            if let Some(uid) = uids.get(target.target) {
                let n = focus.0.entry(*uid).or_insert(0);
                *n = n.saturating_add(1);
            }
        }
        if debug && focus.0 != before {
            // The downed players, aimed at or not.
            let downed: Vec<Uid> = (&uids, &players, &healths, &states)
                .join()
                .filter(|(_, _, h, s)| is_downed(Some(*h), Some(*s)))
                .map(|(uid, ..)| *uid)
                .collect();
            tracing::debug!(focus = ?focus.0, a_terre = ?downed, "Icebreaker : cibles des ennemis");
        }
    }
}
