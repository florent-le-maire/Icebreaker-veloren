//! Icebreaker: `/rentrer`, leaving an encounter instance for the village (V.5).
//!
//! The summary is what the character carries more than when it arrived in the
//! instance (coins, then items), told to every player of the instance. In the
//! campaign (decision 0016), the player is teleported back to the village
//! square; from a place run alone, the player goes back to the village by
//! itself (V.8), its server started if needed; the character, saved on disconnect, keeps what it carries since
//! characters are shared by every instance (I.3).

use crate::{Server, SpawnPoint, icebreaker_slots::IcebreakerSlots, icebreaker_treasury::COINS};
use common::{
    comp::{
        self, ChatType, Content, Inventory, Item, ItemKey, LocalizationArg,
        item::{ItemDefinitionIdOwned, ItemDesc, ItemI18n},
    },
    event::{ClientDisconnectEvent, EventBus, TeleportToPositionEvent},
};
use common_net::msg::{DisconnectReason, ServerGeneral};
use common::cmd::ServerChatCommand;
use specs::{Component, DenseVecStorage, Entity as EcsEntity, WorldExt};
use std::collections::HashMap;

/// How much of each item the character carried when entering the instance.
#[derive(Clone, Debug, Default)]
pub struct Arrival(HashMap<ItemDefinitionIdOwned, u64>);

impl Component for Arrival {
    type Storage = DenseVecStorage<Self>;
}

impl Arrival {
    pub fn of(inventory: &Inventory) -> Self { Self(counts(inventory).0) }
}

/// Item counts of `inventory`, and one item of each kind (for its name).
fn counts(inventory: &Inventory) -> (HashMap<ItemDefinitionIdOwned, u64>, Vec<&Item>) {
    let mut counts = HashMap::<_, u64>::new();
    let mut kinds = Vec::new();
    for item in inventory.slots().flatten() {
        let id = item.item_definition_id().to_owned();
        let count = counts.entry(id).or_default();
        if *count == 0 {
            kinds.push(item);
        }
        *count += u64::from(item.amount());
    }
    (counts, kinds)
}

/// What `inventory` holds more than at `arrival`: coins, then (item, amount).
pub fn gains<'a>(arrival: &Arrival, inventory: &'a Inventory) -> (u64, Vec<(&'a Item, u64)>) {
    let (now, kinds) = counts(inventory);
    let gained = |id: &ItemDefinitionIdOwned| {
        now.get(id)
            .copied()
            .unwrap_or(0)
            .saturating_sub(arrival.0.get(id).copied().unwrap_or(0))
    };
    let coins_id = ItemDefinitionIdOwned::Simple(COINS.to_string());
    let coins = gained(&coins_id);
    let items = kinds
        .into_iter()
        .filter_map(|item| {
            let id = item.item_definition_id().to_owned();
            let amount = gained(&id);
            (id != coins_id && amount > 0).then_some((item, amount))
        })
        .collect();
    (coins, items)
}

pub(crate) fn handle_rentrer(
    server: &mut Server,
    client: EcsEntity,
    _target: EcsEntity,
    _args: Vec<String>,
    _action: &ServerChatCommand,
) -> Result<(), Content> {
    rentrer(server, client)
}

/// Icebreaker : `client` rentre au village, par `/rentrer` ou en traversant un portail allumé (T.10).
pub fn rentrer(server: &mut Server, client: EcsEntity) -> Result<(), Content> {
    let ecs = server.state.ecs();
    // In the campaign (decision 0016), from a dungeon slot of this server.
    let campaign = world::icebreaker::is_campaign();
    let in_slot = ecs
        .read_storage::<comp::Pos>()
        .get(client)
        .is_some_and(|pos| ecs.read_resource::<IcebreakerSlots>().at(pos.0.xy()).is_some());
    let away = if campaign {
        in_slot
    } else {
        world::icebreaker::is_instance() && !world::icebreaker::is_village()
    };
    if !away {
        return Err(Content::Plain("Tu es déjà au village.".to_string()));
    }
    let (Some(arrival), Some(inventory)) = (
        ecs.read_storage::<Arrival>().get(client).cloned(),
        ecs.read_storage::<Inventory>().get(client).cloned(),
    ) else {
        return Err(Content::Plain(
            "Il faut un personnage en jeu pour rentrer.".to_string(),
        ));
    };
    let alias = ecs
        .read_storage::<comp::Player>()
        .get(client)
        .map_or_else(|| "Quelqu'un".to_string(), |player| player.alias.clone());

    let (coins, items) = gains(&arrival, &inventory);
    let sortes = |kinds: usize| if kinds > 1 { "sortes" } else { "sorte" };
    let header = match (coins, items.len()) {
        (0, 0) => format!("{alias} rentre au village les mains vides."),
        (coins, 0) => format!("{alias} rentre au village avec {coins} pièces d'or."),
        (0, kinds) => {
            format!("{alias} rentre au village avec {kinds} {} d'objets :", sortes(kinds))
        },
        (coins, kinds) => format!(
            "{alias} rentre au village avec {coins} pièces d'or et {kinds} {} d'objets :",
            sortes(kinds)
        ),
    };
    // Icebreaker : en campagne la liste des objets ne suit pas dans le chat (elle est dans le
    // bilan) : la ligne finit par un point.
    let header = match header.strip_suffix(" :") {
        Some(debut) if campaign => format!("{debut}."),
        _ => header,
    };
    let i18n = ItemI18n::new_expect();
    let mut summary = vec![Content::Plain(header)];
    // Icebreaker : en campagne, les objets sont dans le bilan (une fenêtre) envoyé à celui qui
    // rentre ; le chat ne garde que la ligne d'en-tête.
    if !campaign {
        summary.extend(items.iter().map(|(item, amount)| {
            Content::localized_with_args("icebreaker-bilan-item", [
                ("amount", LocalizationArg::from(*amount)),
                ("item", LocalizationArg::from(item.i18n(&i18n).0)),
            ])
        }));
    }
    for line in summary {
        server.notify_players(ServerGeneral::server_msg(ChatType::Meta, line));
    }
    if campaign {
        // Icebreaker : le bilan de retour (N.8), pour le joueur qui rentre.
        let retour = {
            let ecs = server.state.ecs();
            let slots = ecs.read_resource::<IcebreakerSlots>();
            let slot = ecs
                .read_storage::<comp::Pos>()
                .get(client)
                .and_then(|pos| slots.at(pos.0.xy()));
            crate::icebreaker_bilan::Retour {
                lieu: slot
                    .map(|slot| crate::icebreaker_expedition::nom_du_site(&slot.name))
                    .unwrap_or_default(),
                decompte: slot.and_then(|slot| {
                    crate::icebreaker_bilan::decompte_de(
                        &ecs.read_resource::<crate::icebreaker_mission::IcebreakerMission>(),
                        &ecs.read_resource::<crate::rtsim::RtSim>(),
                        slot,
                    )
                }),
                or: coins,
                objets: items
                    .iter()
                    .map(|(item, amount)| {
                        (ItemKey::from(*item), u32::try_from(*amount).unwrap_or(u32::MAX))
                    })
                    .collect(),
                peri: false,
                // Icebreaker : la maîtrise d'arme gagnée en bas (jalon A, A.9).
                maitrise: crate::icebreaker_maitrise::progres_de(ecs, client),
            }
        };
        server.notify_client(
            client,
            ServerGeneral::IcebreakerBilan(crate::icebreaker_bilan::panneau(&retour)),
        );
    }

    // In the campaign: back on the village square, counting from there again.
    if campaign {
        // Icebreaker: new recruits at the cart for whoever has no one (jalon R).
        if crate::icebreaker_registre::active() {
            // Icebreaker : ce que le personnage a acquis en bas repart dans sa fiche (jalon A).
            crate::icebreaker_registre::sauver_acquis(server.state.ecs(), client);
            let config = crate::icebreaker_registre::config();
            server
                .state
                .ecs()
                .write_resource::<crate::icebreaker_registre::IcebreakerRegistre>()
                .completer(&config, &mut rand::rng());
        }
        let square = server.state.ecs().read_resource::<SpawnPoint>().0;
        server
            .state
            .ecs()
            .write_storage::<Arrival>()
            .insert(client, Arrival::of(&inventory))
            .ok();
        server
            .state
            .mut_resource::<EventBus<TeleportToPositionEvent>>()
            .emit_now(TeleportToPositionEvent {
                entity: client,
                position: square,
            });
        return Ok(());
    }

    // The client goes to the village by itself (V.8); disconnecting it here too
    // saves the character before it gets there.
    match crate::icebreaker_expedition::village() {
        Some(village) => {
            let clients = server.state.ecs().read_storage::<crate::client::Client>();
            if let Some(client) = clients.get(client) {
                crate::icebreaker_expedition::travel(client, &village);
            }
        },
        None => server.notify_client(
            client,
            ServerGeneral::Disconnect(DisconnectReason::Kicked(
                "Retour au village : cliquer sur « Village » au menu principal.".to_string(),
            )),
        ),
    }
    server
        .state
        .mut_resource::<EventBus<ClientDisconnectEvent>>()
        .emit_now(ClientDisconnectEvent(client, comp::DisconnectReason::Kicked));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, amount: u32) -> Item {
        let mut item = Item::new_from_asset_expect(id);
        item.set_amount(amount).unwrap();
        item
    }

    #[test]
    fn the_summary_counts_only_what_was_gained() {
        let mut inventory = Inventory::with_empty();
        inventory.push(item(COINS, 10)).unwrap();
        inventory.push(item("common.items.food.apple", 3)).unwrap();
        let arrival = Arrival::of(&inventory);
        let (coins, items) = gains(&arrival, &inventory);
        assert_eq!((coins, items.len()), (0, 0));

        inventory.push(item(COINS, 25)).unwrap();
        inventory.push(item("common.items.food.apple", 2)).unwrap();
        inventory
            .push(item("common.items.consumable.potion_minor", 1))
            .unwrap();
        let (coins, items) = gains(&arrival, &inventory);
        assert_eq!(coins, 25);
        let mut items = items
            .iter()
            .map(|(item, amount)| (item.item_definition_id().to_owned(), *amount))
            .collect::<Vec<_>>();
        items.sort_by_key(|(_, amount)| *amount);
        assert_eq!(items, vec![
            (
                ItemDefinitionIdOwned::Simple("common.items.consumable.potion_minor".to_string()),
                1
            ),
            (ItemDefinitionIdOwned::Simple("common.items.food.apple".to_string()), 2),
        ]);
    }
}
