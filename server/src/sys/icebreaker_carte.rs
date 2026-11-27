use crate::{
    client::Client,
    icebreaker_carte::IcebreakerCartes,
    icebreaker_mission::IcebreakerMission,
    icebreaker_slots::IcebreakerSlots,
    rtsim::RtSim,
};
use common::{
    comp::{Pos, Presence, PresenceKind},
    resources::Time,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, Write};

/// Icebreaker CA : les objectifs du lieu où est chaque joueur, pour sa carte de l'expédition. Regardés
/// toutes les `envoi` secondes (`assets/icebreaker/carte.ron`) et envoyés seulement s'ils ont changé :
/// à l'arrivée (même dans un lieu sans chef : la carte s'y remplit quand même), quand le chef tombe, et
/// « pas en expédition » au retour au village.
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Write<'a, IcebreakerCartes>,
        Read<'a, IcebreakerMission>,
        Read<'a, IcebreakerSlots>,
        ReadExpect<'a, RtSim>,
        ReadStorage<'a, Client>,
        ReadStorage<'a, Presence>,
        ReadStorage<'a, Pos>,
    );

    const NAME: &'static str = "icebreaker_carte";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (entities, time, mut cartes, missions, slots, rtsim, clients, presences, positions): Self::SystemData,
    ) {
        if time.0 - cartes.vu < cartes.envoi {
            return;
        }
        cartes.vu = time.0;
        let en_jeu = |entity| {
            presences
                .get(entity)
                .is_some_and(|presence: &Presence| matches!(presence.kind, PresenceKind::Character(_)))
        };
        // Qui est parti, ou revenu à l'écran des personnages, recevra tout à son retour.
        cartes
            .envoyes
            .retain(|entity, _| entities.is_alive(*entity) && clients.contains(*entity) && en_jeu(*entity));
        // Un donjon lancé seul : sa mission est celle du serveur.
        let seul = world::icebreaker::is_instance() && !world::icebreaker::is_village();
        for (entity, client, pos) in (&entities, &clients, &positions).join() {
            if !en_jeu(entity) {
                continue;
            }
            let slot = slots.at(pos.0.xy());
            let mission = if seul {
                missions.iter().next().map(|(_, mission)| mission)
            } else {
                slot.and_then(|slot| missions.of(slot.site))
            };
            let objectifs = (seul || slot.is_some()).then(|| {
                mission
                    .map(|mission| mission.objectifs(|slot| rtsim.is_dungeon_npc_killed(slot)))
                    .unwrap_or_default()
            });
            let envoyes = cartes.envoyes.entry(entity).or_default();
            if *envoyes != objectifs {
                client.send_fallible(ServerGeneral::IcebreakerObjectifs(objectifs.clone()));
                *envoyes = objectifs;
            }
        }
    }
}
