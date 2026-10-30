//! Icebreaker: `/rentrer`, leaving an encounter instance for the village (V.5).
//!
//! The summary is what the character carries more than when it arrived in the
//! instance (coins, then items), told to every player of the instance. The
//! player is then disconnected and joins the village by hand (main menu); the
//! character, saved on disconnect, keeps what it carries since characters are
//! shared by every instance (I.3).

use crate::{Server, icebreaker_treasury::COINS};
use common::{
    comp::{
        self, ChatType, Content, Inventory, Item, LocalizationArg,
        item::{ItemDefinitionIdOwned, ItemDesc, ItemI18n},
    },
    event::{ClientDisconnectEvent, EventBus},
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
    if !world::icebreaker::is_instance() || world::icebreaker::is_village() {
        return Err(Content::Plain("Tu es déjà au village.".to_string()));
    }
    let ecs = server.state.ecs();
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
    let header = match (coins, items.len()) {
        (0, 0) => format!("{alias} rentre au village les mains vides."),
        (coins, 0) => format!("{alias} rentre au village avec {coins} pièces d'or."),
        (0, kinds) => format!("{alias} rentre au village avec {kinds} sortes d'objets :"),
        (coins, kinds) => {
            format!("{alias} rentre au village avec {coins} pièces d'or et {kinds} sortes d'objets :")
        },
    };
    let i18n = ItemI18n::new_expect();
    let mut summary = vec![Content::Plain(header)];
    summary.extend(items.iter().map(|(item, amount)| {
        Content::localized_with_args("icebreaker-bilan-item", [
            ("amount", LocalizationArg::from(*amount)),
            ("item", LocalizationArg::from(item.i18n(&i18n).0)),
        ])
    }));
    for line in summary {
        server.notify_players(ServerGeneral::server_msg(ChatType::Meta, line));
    }

    server.notify_client(
        client,
        ServerGeneral::Disconnect(DisconnectReason::Kicked(
            "Retour au village : cliquer sur « Village » au menu principal.".to_string(),
        )),
    );
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
