use crate::{
    client::Client,
    icebreaker_entrainement::{self, BODY, IcebreakerEntrainement, Serie},
};
use common::{
    comp::{Body, ChatType, Content, Health, Pos, Vel},
    resources::Time,
    uid::{IdMaps, Uid},
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, WriteStorage};
use std::collections::HashMap;
use vek::*;

/// What is followed of one training dummy.
#[derive(Default)]
struct Suivi {
    /// The time of the last hit seen (`Health::last_change`).
    vu: f64,
    /// A series per attacker.
    series: HashMap<Uid, Serie>,
    /// Where a still dummy is held (X.15), none if it keeps the knockback.
    poste: Option<Vec3<f32>>,
}

/// Icebreaker: the training dummies (X.14) count each attacker's hits; after
/// `fin_de_serie` seconds without a hit, each attacker is told what they did
/// and the dummy gets all its health back. Also the dummies of `/dummy`.
/// The still dummies of the training ground are held at their post (X.15).
#[derive(Default)]
pub struct Sys {
    suivis: HashMap<Uid, Suivi>,
}

impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Time>,
        Read<'a, IdMaps>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, Body>,
        WriteStorage<'a, Health>,
        ReadStorage<'a, Client>,
        Read<'a, IcebreakerEntrainement>,
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
    );

    const NAME: &'static str = "icebreaker_entrainement";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        job: &mut Job<Self>,
        (
            entities,
            time,
            id_maps,
            uids,
            bodies,
            mut healths,
            clients,
            entrainement,
            mut positions,
            mut velocities,
        ): Self::SystemData,
    ) {
        let fin_de_serie = icebreaker_entrainement::config().fin_de_serie;
        let suivis = &mut job.own.suivis;
        let mut a_remettre = Vec::new();
        let mut a_tenir = Vec::new();
        for (entity, uid, _, health, pos) in (&entities, &uids, &bodies, &healths, &positions)
            .join()
            .filter(|(_, _, body, _, _)| **body == BODY)
        {
            // A dummy seen for the first time is just at its post.
            let suivi = suivis.entry(*uid).or_insert_with(|| Suivi {
                poste: entrainement.poste_immobile(pos.0),
                ..Suivi::default()
            });
            if let Some(poste) = suivi.poste {
                a_tenir.push((entity, poste));
            }
            let change = health.last_change;
            if change.time.0 != suivi.vu && change.amount < 0.0 {
                suivi.vu = change.time.0;
                if let Some(by) = change.by {
                    suivi
                        .series
                        .entry(by.uid())
                        .and_modify(|serie| serie.noter(change.time.0, -change.amount))
                        .or_insert_with(|| Serie::new(change.time.0, -change.amount));
                }
            }
            let dernier = suivi.series.values().map(|serie| serie.dernier).fold(f64::MIN, f64::max);
            if !suivi.series.is_empty() && time.0 - dernier >= fin_de_serie {
                for (attaquant, serie) in suivi.series.drain() {
                    if let Some(client) = id_maps
                        .uid_entity(attaquant)
                        .and_then(|attaquant| clients.get(attaquant))
                    {
                        client.send_fallible(ServerGeneral::server_msg(
                            ChatType::CommandInfo,
                            Content::Plain(serie.bilan()),
                        ));
                    }
                }
                suivi.vu = 0.0;
                a_remettre.push(entity);
            }
        }
        for entity in a_remettre {
            if let Some(mut health) = healths.get_mut(entity) {
                *health = Health::new(BODY);
            }
        }
        // The still dummies stay at their post: no knockback, only falling.
        for (entity, poste) in a_tenir {
            if let (Some(pos), Some(vel)) = (positions.get_mut(entity), velocities.get_mut(entity)) {
                if pos.0.xy() != poste.xy() {
                    pos.0 = poste.xy().with_z(pos.0.z);
                }
                if vel.0.x != 0.0 || vel.0.y != 0.0 || vel.0.z > 0.0 {
                    vel.0 = Vec3::new(0.0, 0.0, vel.0.z.min(0.0));
                }
            }
        }
        // The dummies gone with their chunk.
        suivis.retain(|uid, _| id_maps.uid_entity(*uid).is_some());
    }
}
