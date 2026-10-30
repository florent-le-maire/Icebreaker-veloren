use crate::{
    client::Client,
    icebreaker_expedition::{self, IcebreakerExpedition, LEAVE_RANGE},
};
use common::{
    comp::{ChatType, Content, DisconnectReason, Pos},
    event::{ClientDisconnectEvent, EventBus},
    resources::Time,
    uid::IdMaps,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Join, Read, ReadExpect, ReadStorage};

/// Icebreaker: the expedition being prepared at the « Départ » house (V.9).
/// Its members who disconnect or go too far from the leader leave it; once
/// everyone was ready for `DEPARTURE_DELAY`, they all go to its destination.
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Read<'a, Time>,
        Read<'a, IdMaps>,
        Read<'a, IcebreakerExpedition>,
        ReadExpect<'a, EventBus<ClientDisconnectEvent>>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Client>,
    );

    const NAME: &'static str = "icebreaker_expedition";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (time, id_maps, expedition, disconnects, positions, clients): Self::SystemData,
    ) {
        let Some(leader) = expedition.leader_pos() else {
            return;
        };
        let (departure, announce) = icebreaker_expedition::tick(time.0, |uid| {
            id_maps
                .uid_entity(uid)
                .and_then(|entity| positions.get(entity))
                .is_some_and(|pos| pos.0.distance(leader) <= LEAVE_RANGE)
        });
        let tell_all = |line: String| {
            for client in (&clients).join() {
                client.send_fallible(ServerGeneral::server_msg(
                    ChatType::Meta,
                    Content::Plain(line.clone()),
                ));
            }
        };
        announce.into_iter().for_each(tell_all);

        let Some((lieu, members)) = departure else {
            return;
        };
        if let Err(err) = icebreaker_expedition::ensure_running(&lieu) {
            tell_all(err);
            return;
        }
        for entity in members.into_iter().filter_map(|uid| id_maps.uid_entity(uid)) {
            if let Some(client) = clients.get(entity) {
                client.send_fallible(ServerGeneral::IcebreakerTravel(lieu.port));
            }
            // Saves the character before it arrives there.
            disconnects.emit_now(ClientDisconnectEvent(entity, DisconnectReason::Kicked));
        }
    }
}
