use crate::{
    client::Client,
    icebreaker_expedition::{self, IcebreakerExpedition},
    icebreaker_return::Arrival,
    icebreaker_slots::IcebreakerSlots,
};
use common::{
    comp::{ChatType, Content, DisconnectReason, Inventory, Player, Pos},
    event::{ClientDisconnectEvent, EventBus, TeleportToPositionEvent},
    resources::Time,
    uid::IdMaps,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Join, Read, ReadExpect, ReadStorage, Write, WriteStorage};
use vek::*;

/// Icebreaker: the expedition being prepared at the « Départ » house (V.9).
/// Its members who disconnect or leave the village (into a dungeon slot) leave
/// it; every second, it is shown to the players of the village (V.11); once
/// everyone was ready for `DEPARTURE_DELAY`, they all go to its destination:
/// in the campaign (decision 0016), teleported to a free dungeon slot of this
/// server; otherwise, to the server of the place (V.8).
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Read<'a, Time>,
        Read<'a, IdMaps>,
        Write<'a, IcebreakerExpedition>,
        Read<'a, IcebreakerSlots>,
        ReadExpect<'a, EventBus<ClientDisconnectEvent>>,
        ReadExpect<'a, EventBus<TeleportToPositionEvent>>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Client>,
        WriteStorage<'a, Inventory>,
        WriteStorage<'a, Arrival>,
    );

    const NAME: &'static str = "icebreaker_expedition";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            time,
            id_maps,
            mut expedition,
            slots,
            disconnects,
            teleports,
            positions,
            players,
            clients,
            mut inventories,
            mut arrivals,
        ): Self::SystemData,
    ) {
        if expedition.leader_pos().is_none() {
            return;
        }
        let campaign = !slots.0.is_empty();
        let player_positions = (&players, &positions)
            .join()
            .map(|(_, pos)| pos.0.xy())
            .collect::<Vec<_>>();
        let free_slot = |lieu: &icebreaker_expedition::Lieu| {
            lieu.site()
                .and_then(|site| slots.free(site, &player_positions))
                .cloned()
        };
        let (departure, announce) = icebreaker_expedition::tick(
            time.0,
            |uid| {
                id_maps
                    .uid_entity(uid)
                    .and_then(|entity| positions.get(entity))
                    .is_some_and(|pos| slots.at(pos.0.xy()).is_none())
            },
            |lieu| !campaign || free_slot(lieu).is_some(),
        );
        let tell_all = |line: String| {
            for client in (&clients).join() {
                client.send_fallible(ServerGeneral::server_msg(
                    ChatType::Meta,
                    Content::Plain(line.clone()),
                ));
            }
        };
        announce.into_iter().for_each(tell_all);

        // The expedition, shown every second to the players of the village.
        if time.0 - expedition.last_shown >= 1.0 {
            expedition.last_shown = time.0;
            let summary = icebreaker_expedition::summary(time.0).map(|(title, members)| {
                (Content::Plain(title), members)
            });
            for (client, pos) in (&clients, &positions).join() {
                // Nothing for those away in a dungeon slot.
                let shown = summary.clone().filter(|_| slots.at(pos.0.xy()).is_none());
                client.send_fallible(ServerGeneral::IcebreakerExpedition(shown));
            }
        }

        let Some((lieu, members)) = departure else {
            return;
        };
        let members = members
            .into_iter()
            .filter_map(|uid| id_maps.uid_entity(uid))
            .collect::<Vec<_>>();

        // The campaign: teleported to the slot, side by side; what they carry
        // now is what `/rentrer` will count from.
        if campaign {
            let Some(slot) = free_slot(&lieu) else {
                return;
            };
            for (i, entity) in members.into_iter().enumerate() {
                let offset = Vec3::new((i % 4) as f32 * 2.0, (i / 4) as f32 * 2.0, 0.0);
                teleports.emit_now(TeleportToPositionEvent {
                    entity,
                    position: slot.arrival + offset,
                });
                if let Some(mut inventory) = inventories.get_mut(entity) {
                    // Icebreaker : une balise lumineuse pour le donjon (jalon J).
                    crate::icebreaker_balise::donner_si_absente(&mut inventory);
                    let _ = arrivals.insert(entity, Arrival::of(&inventory));
                }
            }
            return;
        }

        // A village run alone: to the server of the place.
        if let Err(err) = icebreaker_expedition::ensure_running(&lieu) {
            tell_all(err);
            return;
        }
        for entity in members {
            if let Some(client) = clients.get(entity) {
                client.send_fallible(ServerGeneral::IcebreakerTravel(lieu.port));
            }
            // Saves the character before it arrives there.
            disconnects.emit_now(ClientDisconnectEvent(entity, DisconnectReason::Kicked));
        }
    }
}
