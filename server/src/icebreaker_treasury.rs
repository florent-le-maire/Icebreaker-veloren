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
