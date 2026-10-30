//! Icebreaker: the village treasury, shared by every player of the campaign.
//!
//! Each player keeps their own coins and may give some to the treasurer, an
//! NPC standing on the village square who takes coins and gives nothing back
//! (`TradingBehavior::Treasury`). The total is saved in the campaign
//! directory, shown in the treasurer's name and announced to every player.

use crate::icebreaker_warehouse::{chunk_of, village_npc, write_campaign_file};
use common::{
    comp::{self, Content, Inventory, agent::TradingBehavior},
    event::NpcBuilder,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};
use vek::*;

/// The item the treasury takes.
pub const COINS: &str = "common.items.utility.coins";

/// File of the campaign directory holding the treasury.
const TREASURY_FILE: &str = "caisse.ron";

/// What is saved of the treasury.
#[derive(Default, Serialize, Deserialize)]
struct SavedTreasury {
    coins: u64,
}

#[derive(Default)]
pub struct IcebreakerTreasury {
    path: Option<PathBuf>,
    treasurer_pos: Option<Vec3<f32>>,
    coins: u64,
}

impl IcebreakerTreasury {
    /// A treasury kept in `campaign_dir`, whose treasurer stands at
    /// `treasurer_pos` (none outside the village).
    pub fn new(campaign_dir: Option<PathBuf>, treasurer_pos: Option<Vec3<f32>>) -> Self {
        let path = campaign_dir.map(|dir| dir.join(TREASURY_FILE));
        let coins = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|text| {
                ron::from_str::<SavedTreasury>(&text).unwrap_or_else(|err| {
                    warn!(?err, "Icebreaker: unreadable treasury, starting empty");
                    SavedTreasury::default()
                })
            })
            .unwrap_or_default()
            .coins;
        if let (Some(pos), Some(_)) = (treasurer_pos, &path) {
            info!(?pos, coins, "Icebreaker: village treasurer and treasury");
        }
        Self {
            treasurer_pos: treasurer_pos.filter(|_| path.is_some()),
            path,
            coins,
        }
    }

    pub fn coins(&self) -> u64 { self.coins }

    /// Takes `amount` coins out of the treasury (a village construction, C.3),
    /// if it holds as many.
    pub fn spend(&mut self, amount: u64) -> bool {
        if self.coins < amount {
            return false;
        }
        self.coins -= amount;
        self.save();
        true
    }

    /// The chunk the treasurer spawns with.
    pub fn treasurer_chunk(&self) -> Option<Vec2<i32>> { self.treasurer_pos.map(chunk_of) }

    /// The treasurer, named after the treasury.
    pub fn treasurer(&self) -> Option<(NpcBuilder, comp::Pos)> {
        let pos = self.treasurer_pos?;
        let npc = village_npc(self.treasurer_name(), pos, TradingBehavior::Treasury, Vec::new());
        Some((npc, comp::Pos(pos)))
    }

    /// The treasurer's name, showing the treasury to everyone.
    pub fn treasurer_name(&self) -> String { format!("Trésorier (caisse : {} or)", self.coins) }

    /// Moves the coins the treasurer was given into the treasury; returns how
    /// many.
    pub fn collect(&mut self, treasurer: &mut Inventory) -> u64 {
        let given = treasurer
            .drain()
            .map(|item| {
                if item.item_definition_id().itemdef_id() == Some(COINS) {
                    u64::from(item.amount())
                } else {
                    // The treasurer only accepts coins: nothing else should get here.
                    warn!(item = ?item.item_definition_id(), "Icebreaker: treasurer got a non-coin item, lost");
                    0
                }
            })
            .sum::<u64>();
        if given > 0 {
            self.coins += given;
            self.save();
        }
        given
    }

    fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        match write_campaign_file(path, &SavedTreasury { coins: self.coins }) {
            Ok(()) => info!(coins = self.coins, "Icebreaker: treasury saved"),
            Err(err) => warn!(?err, ?path, "Icebreaker: could not save the treasury"),
        }
    }

    /// The message telling everyone about a gift to the treasury.
    pub fn gift_message(&self, giver: &str, given: u64) -> Content {
        Content::Plain(format!(
            "{giver} verse {given} or à la caisse du village (caisse : {} or).",
            self.coins
        ))
    }
}

impl IcebreakerTreasury {
    /// Puts `amount` coins in the treasury (`/caisse`, V.10).
    pub fn deposit(&mut self, amount: u64) {
        self.coins += amount;
        self.save();
    }
}

/// Renames the treasurer after the treasury.
pub fn refresh_treasurer_name(ecs: &specs::World) {
    use specs::{LendJoin, WorldExt};
    let name = ecs.read_resource::<IcebreakerTreasury>().treasurer_name();
    let agents = ecs.read_storage::<comp::Agent>();
    let mut stats = ecs.write_storage::<comp::Stats>();
    let mut treasurers = (&agents, &mut stats).lend_join();
    while let Some((agent, mut stats)) = treasurers.next() {
        if matches!(agent.behavior.trading_behavior, TradingBehavior::Treasury) {
            stats.name = Content::Plain(name.clone());
        }
    }
}

/// Takes up to `amount` coins (all of them for `None`) from the inventory of
/// `entity`; returns how many.
pub fn take_coins(ecs: &specs::World, entity: specs::Entity, amount: Option<u64>) -> u64 {
    use common::comp::{item::MaterialStatManifest, tool::AbilityMap};
    use specs::WorldExt;
    let coins = crate::icebreaker_services::coins();
    let mut inventories = ecs.write_storage::<Inventory>();
    let Some(mut inventory) = inventories.get_mut(entity) else {
        return 0;
    };
    let has = inventory.item_count(&coins);
    let amount = amount.map_or(has, |amount| amount.min(has));
    let Ok(amount32) = u32::try_from(amount) else {
        return 0;
    };
    if amount32 == 0 {
        return 0;
    }
    let taken = inventory.remove_item_amount(
        &coins,
        amount32,
        &ecs.read_resource::<AbilityMap>(),
        &ecs.read_resource::<MaterialStatManifest>(),
    );
    if taken.is_some() { amount } else { 0 }
}

/// `/caisse <montant|tout>` (the treasury's panel, V.10): gives coins to the
/// village treasury.
pub(crate) fn handle_caisse(
    server: &mut crate::Server,
    client: specs::Entity,
    _target: specs::Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), Content> {
    use specs::WorldExt;
    if !world::icebreaker::is_village() {
        return Err(Content::Plain("La caisse est au village.".to_string()));
    }
    let amount = match args.first().map(String::as_str) {
        Some("tout") => None,
        Some(n) => Some(n.parse::<u64>().map_err(|_| {
            Content::Plain("Combien ? (/caisse <montant> ou /caisse tout)".to_string())
        })?),
        None => return Err(Content::Plain("Combien ? (/caisse <montant> ou /caisse tout)".to_string())),
    };
    let ecs = server.state.ecs();
    let given = take_coins(ecs, client, amount);
    if given == 0 {
        return Err(Content::Plain("Tu n'as pas d'or sur toi.".to_string()));
    }
    let alias = ecs
        .read_storage::<comp::Player>()
        .get(client)
        .map_or_else(|| "Quelqu'un".to_string(), |player| player.alias.clone());
    let message = {
        let mut treasury = ecs.write_resource::<IcebreakerTreasury>();
        treasury.deposit(given);
        treasury.gift_message(&alias, given)
    };
    refresh_treasurer_name(ecs);
    server.notify_players(common_net::msg::ServerGeneral::server_msg(
        comp::ChatType::Meta,
        message,
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::comp::Item;

    #[test]
    fn coins_given_to_the_treasurer_stay_in_the_treasury() {
        let dir = std::env::temp_dir().join(format!("icebreaker-treasury-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut treasury = IcebreakerTreasury::new(Some(dir.clone()), Some(Vec3::zero()));
        assert_eq!(treasury.coins(), 0);

        let mut treasurer = Inventory::with_empty();
        let mut coins = Item::new_from_asset_expect(COINS);
        coins.set_amount(25).unwrap();
        treasurer.push(coins).unwrap();
        assert_eq!(treasury.collect(&mut treasurer), 25);
        assert_eq!(treasurer.slots().flatten().count(), 0);

        let treasury = IcebreakerTreasury::new(Some(dir.clone()), Some(Vec3::zero()));
        assert_eq!(treasury.coins(), 25);
        assert_eq!(treasury.treasurer_name(), "Trésorier (caisse : 25 or)");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
