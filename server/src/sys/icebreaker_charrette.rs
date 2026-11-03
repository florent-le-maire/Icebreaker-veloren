//! Icebreaker: keeps the recruits standing in front of the cart in step with
//! the cart (X.8), once a second: an NPC appears for each recruit whose chunk
//! is loaded, is marked by its name once it exists, and goes when the recruit
//! is hired or the cart renewed.

use crate::{
    icebreaker_charrette::{self, IcebreakerCharrette, RecrueCharrette},
    icebreaker_registre::{self, IcebreakerRegistre},
    icebreaker_warehouse::chunk_of,
};
use common::{
    comp::{Alignment, Anchor, Content, Ori, Pos, Stats},
    event::{CreateNpcEvent, DeleteEvent, EventBus},
    resources::Time,
    terrain::TerrainGrid,
};
use common_ecs::{Job, Origin, Phase, System};
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, Write, WriteStorage};

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Read<'a, IcebreakerRegistre>,
        Write<'a, IcebreakerCharrette>,
        ReadExpect<'a, TerrainGrid>,
        ReadStorage<'a, Stats>,
        ReadStorage<'a, Alignment>,
        WriteStorage<'a, RecrueCharrette>,
        Read<'a, EventBus<CreateNpcEvent>>,
        Read<'a, EventBus<DeleteEvent>>,
    );

    const NAME: &'static str = "icebreaker_charrette";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            time,
            registre,
            mut charrette,
            terrain,
            stats,
            alignments,
            mut marks,
            create_npc_events,
            delete_events,
        ): Self::SystemData,
    ) {
        if !icebreaker_registre::active()
            || charrette.post(0).is_none()
            || time.0 - charrette.last_run < 1.0
        {
            return;
        }
        charrette.last_run = time.0;

        // The NPCs just created get their mark, by name.
        let wanted: Vec<String> = registre
            .charrette()
            .iter()
            .map(|r| r.nom.clone())
            .collect();
        let mut newly_marked = Vec::new();
        for (entity, stat, alignment, ()) in (&entities, &stats, &alignments, !&marks).join() {
            if *alignment != Alignment::Npc {
                continue;
            }
            if let Content::Plain(name) = &stat.name
                && charrette.asked_for(name)
            {
                newly_marked.push((entity, name.clone()));
            }
        }
        for (entity, name) in newly_marked {
            charrette.done(&name);
            let _ = marks.insert(entity, RecrueCharrette(name));
        }

        let present: Vec<(specs::Entity, String)> = (&entities, &marks)
            .join()
            .map(|(entity, mark)| (entity, mark.0.clone()))
            .collect();
        let present_names: Vec<String> = present.iter().map(|(_, n)| n.clone()).collect();
        let (a_creer, a_retirer) = icebreaker_charrette::synchroniser(&wanted, &present_names);

        let mut creations = create_npc_events.emitter();
        for (index, recrue) in registre.charrette().iter().enumerate() {
            if !a_creer.contains(&recrue.nom) || charrette.pending(&recrue.nom, time.0) {
                continue;
            }
            let Some(pos) = charrette.post(index) else {
                continue;
            };
            // Only with its chunk: the NPC goes with it, and comes back with it.
            let chunk = chunk_of(pos);
            if terrain.get_key(chunk).is_none() {
                continue;
            }
            if let Some(npc) = icebreaker_charrette::recruit_npc(recrue, pos) {
                charrette.asked(&recrue.nom, time.0);
                tracing::info!(nom = recrue.nom, ?pos, "Icebreaker: recruit stands at the cart");
                creations.emit(CreateNpcEvent {
                    pos: Pos(pos),
                    ori: Ori::default(),
                    npc: npc.with_anchor(Anchor::Chunk(chunk)),
                });
            }
        }
        let mut deletions = delete_events.emitter();
        for (entity, name) in present {
            if a_retirer.contains(&name) {
                tracing::info!(nom = name, "Icebreaker: recruit leaves the cart");
                deletions.emit(DeleteEvent(entity));
            }
        }
    }
}
