#[cfg(feature = "worldgen")] use crate::rtsim;
use crate::icebreaker_slots::IcebreakerSlots;
use common::{
    comp::{PickupItem, Player, Pos},
    event::{DeleteEvent, EventBus},
    terrain::{TerrainChunkSize, TerrainGrid},
    vol::RectVolSize,
};
use common_ecs::{Job, Origin, Phase, System};
use common_state::TerrainChanges;
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, Write, WriteExpect};
use tracing::info;
#[cfg(feature = "worldgen")] use tracing::warn;
use vek::*;

#[cfg(feature = "worldgen")]
type RtSimData<'a> = WriteExpect<'a, rtsim::RtSim>;
#[cfg(not(feature = "worldgen"))]
type RtSimData<'a> = ();

/// Icebreaker I.12: what a new plan for a slot needs.
#[cfg(feature = "worldgen")]
type NewPlanData<'a> = (
    ReadExpect<'a, std::sync::Arc<world::World>>,
    ReadExpect<'a, world::IndexOwned>,
    WriteExpect<'a, crate::chunk_generator::ChunkGenerator>,
    Write<'a, crate::icebreaker_mission::IcebreakerMission>,
    Write<'a, super::icebreaker_sleep::IcebreakerSleep>,
);
#[cfg(not(feature = "worldgen"))]
type NewPlanData<'a> = ();

/// Icebreaker: a dungeon slot of the campaign is made new again when the last
/// player leaves it (I.9): its traces (killed creatures, opened chests) are
/// forgotten and its chunks dropped, to be generated again, creatures and
/// chests back, for the next expedition. A generated dungeon or a cave gets a
/// new plan (I.12), with its arrival, mission and sleepers.
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Write<'a, IcebreakerSlots>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Pos>,
        WriteExpect<'a, TerrainGrid>,
        Write<'a, TerrainChanges>,
        RtSimData<'a>,
        // Icebreaker : les sacs des morts disparaissent à la remise à neuf (T.3).
        (Entities<'a>, ReadStorage<'a, PickupItem>, Read<'a, EventBus<DeleteEvent>>),
        NewPlanData<'a>,
    );

    const NAME: &'static str = "icebreaker_slots";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    #[cfg_attr(not(feature = "worldgen"), expect(unused_variables, unused_mut))]
    fn run(
        _job: &mut Job<Self>,
        (
            mut slots,
            players,
            positions,
            mut terrain,
            mut terrain_changes,
            mut rtsim,
            (entities, pickup_items, delete_bus),
            mut new_plan,
        ): Self::SystemData,
    ) {
        if slots.0.is_empty() {
            return;
        }
        let players = (&players, &positions)
            .join()
            .map(|(_, pos)| pos.0.xy())
            .collect::<Vec<_>>();
        for slot in slots.0.iter_mut() {
            let occupied = players.iter().any(|pos| slot.holds(*pos));
            if slot.occupied && !occupied {
                #[cfg(feature = "worldgen")]
                let forgotten = rtsim.forget_dungeon_traces(|wpos| slot.holds(wpos.xy().as_()));
                #[cfg(not(feature = "worldgen"))]
                let forgotten = (0, 0);
                let mut delete = delete_bus.emitter();
                for (sac, _, _) in (&entities, &pickup_items, &positions)
                    .join()
                    .filter(|(_, item, pos)| {
                        // Icebreaker : les sacs des morts et les balises lumineuses (jalon J).
                        (common::icebreaker_mort::est_sac(item.item())
                            || common::icebreaker_balise::est_balise(item.item()))
                            && slot.holds(pos.0.xy())
                    })
                {
                    delete.emit(DeleteEvent(sac));
                }
                let chunk = |wpos: Vec2<f32>| {
                    wpos.as_::<i32>()
                        .map2(TerrainChunkSize::RECT_SIZE, |e, sz| e.div_euclid(sz as i32))
                };
                let min = chunk(slot.center - slot.zone_radius);
                let max = chunk(slot.center + slot.zone_radius);
                let mut dropped = 0;
                for x in min.x..=max.x {
                    for y in min.y..=max.y {
                        let key = Vec2::new(x, y);
                        if terrain.remove(key).is_some() {
                            terrain_changes.removed_chunks.insert(key);
                            dropped += 1;
                        }
                        // Icebreaker I.12: no chunk of the former plan arrives later.
                        #[cfg(feature = "worldgen")]
                        new_plan.2.cancel_if_pending(key);
                    }
                }
                info!(
                    slot = %slot.name,
                    center = ?slot.center,
                    dropped,
                    ?forgotten,
                    "Icebreaker: dungeon slot made new"
                );
                // Icebreaker I.12: a new plan, once the traces of the former one are forgotten.
                #[cfg(feature = "worldgen")]
                {
                    let seed = rand::random();
                    new_layout(slot, &new_plan.0, new_plan.1.as_index_ref(), &mut new_plan.3, &mut new_plan.4, seed);
                }
            }
            slot.occupied = occupied;
        }
    }
}

/// Icebreaker I.12: draws a new plan (of `seed`) for the generated dungeon or
/// the cave of `slot`, then sets its arrival, its mission and its sleepers.
/// Other slots, or none fitting, keep theirs.
#[cfg(feature = "worldgen")]
pub fn new_layout(
    slot: &mut crate::icebreaker_slots::Slot,
    world: &world::World,
    index: world::IndexRef,
    mission: &mut crate::icebreaker_mission::IcebreakerMission,
    sleep: &mut super::icebreaker_sleep::IcebreakerSleep,
    seed: u64,
) {
    let site = &index.sites[slot.site];
    match world::icebreaker::regenerate_slot(world, site, seed) {
        Ok(true) => {
            slot.arrival = crate::icebreaker_slots::arrival_pos(world, index, site, site.origin);
            crate::icebreaker_mission::set_dungeon(mission, sleep, slot.site, site);
            info!(slot = %slot.name, seed, arrival = ?slot.arrival, "Icebreaker: new plan for the dungeon slot");
        },
        Ok(false) => {},
        Err(e) => warn!(?e, slot = %slot.name, "Icebreaker: no new plan for the dungeon slot"),
    }
}
