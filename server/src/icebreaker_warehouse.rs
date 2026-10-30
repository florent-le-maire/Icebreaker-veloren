//! Icebreaker: the village warehouse, shared by every player of the campaign.
//!
//! The warehouse is the inventory of the storekeeper, an NPC standing on the
//! village square who accepts any exchange (`TradingBehavior::Storage`): in
//! the trade window, players put items in and take items out. Its content is
//! saved in the campaign directory after each exchange, and given back to the
//! storekeeper each time it spawns with its chunk.

use common::{
    comp::{self, Agent, Alignment, Body, Content, Inventory, Item, agent::TradingBehavior},
    event::NpcBuilder,
    terrain::TerrainChunkSize,
    vol::RectVolSize,
};
use std::path::PathBuf;
use tracing::{info, warn};
use vek::*;

/// File of the campaign directory holding the warehouse content.
const WAREHOUSE_FILE: &str = "entrepot.ron";

#[derive(Default)]
pub struct IcebreakerWarehouse {
    path: Option<PathBuf>,
    storekeeper_pos: Option<Vec3<f32>>,
}

impl IcebreakerWarehouse {
    /// A warehouse kept in `campaign_dir`, whose storekeeper stands at
    /// `storekeeper_pos` (none outside the village).
    pub fn new(campaign_dir: Option<PathBuf>, storekeeper_pos: Option<Vec3<f32>>) -> Self {
        let path = campaign_dir.map(|dir| dir.join(WAREHOUSE_FILE));
        if let (Some(pos), Some(path)) = (storekeeper_pos, &path) {
            info!(?pos, ?path, "Icebreaker: village storekeeper and warehouse");
        }
        Self {
            storekeeper_pos: storekeeper_pos.filter(|_| path.is_some()),
            path,
        }
    }

    /// The chunk the storekeeper spawns with.
    pub fn storekeeper_chunk(&self) -> Option<Vec2<i32>> { self.storekeeper_pos.map(chunk_of) }

    /// The storekeeper, holding the warehouse content.
    pub fn storekeeper(&self) -> Option<(NpcBuilder, comp::Pos)> {
        let pos = self.storekeeper_pos?;
        let npc = village_npc(
            "Magasinier".to_string(),
            pos,
            TradingBehavior::Storage,
            self.load_items(),
        );
        Some((npc, comp::Pos(pos)))
    }

    fn load_items(&self) -> Vec<Item> {
        let Some(path) = &self.path else {
            return Vec::new();
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        ron::from_str(&text).unwrap_or_else(|err| {
            warn!(?err, ?path, "Icebreaker: unreadable warehouse, starting empty");
            Vec::new()
        })
    }

    /// Saves the storekeeper's inventory as the warehouse content.
    pub fn save(&self, inventory: &Inventory) {
        let Some(path) = &self.path else {
            return;
        };
        let items = inventory.slots().flatten().collect::<Vec<_>>();
        match write_campaign_file(path, &items) {
            Ok(()) => info!(items = items.len(), "Icebreaker: warehouse saved"),
            Err(err) => warn!(?err, ?path, "Icebreaker: could not save the warehouse"),
        }
    }
}

/// The chunk holding `pos`.
pub(crate) fn chunk_of(pos: Vec3<f32>) -> Vec2<i32> {
    pos.xy()
        .as_::<i32>()
        .map2(TerrainChunkSize::RECT_SIZE, |e, sz| e.div_euclid(sz as i32))
}

/// An NPC of the village standing at `pos`, trading as `trading`, holding
/// `items`.
pub(crate) fn village_npc(
    name: String,
    pos: Vec3<f32>,
    trading: TradingBehavior,
    items: Vec<Item>,
) -> NpcBuilder {
    let body = Body::Humanoid(comp::humanoid::Body::random());
    let loadout = comp::inventory::loadout_builder::LoadoutBuilder::from_default(&body).build();
    let mut inventory = Inventory::with_loadout(loadout, body);
    for item in items {
        if let Err((item, _)) = inventory.push(item) {
            warn!(item = ?item.item_definition_id(), %name, "Icebreaker: inventory full, item lost");
        }
    }

    let mut agent = Agent::from_body(&body)
        .with_patrol_origin(pos)
        .with_idle_wander_factor(0.0);
    agent.behavior.allow(comp::agent::BehaviorCapability::TRADE);
    agent.behavior.trading_behavior = trading;

    NpcBuilder::new(
        comp::Stats::new(Content::Plain(name), body),
        body,
        Alignment::Npc,
    )
    .with_health(comp::Health::new(body))
    .with_inventory(inventory)
    .with_agent(agent)
}

/// Writes `value` in RON to `path`, aside then renamed, so that a crash never
/// leaves half a file.
pub(crate) fn write_campaign_file(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> std::io::Result<()> {
    let text = ron::to_string(value).map_err(std::io::Error::other)?;
    let tmp = path.with_extension("ron.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_storekeeper_gets_back_the_saved_warehouse() {
        let dir = std::env::temp_dir().join(format!("icebreaker-warehouse-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let warehouse = IcebreakerWarehouse::new(Some(dir.clone()), Some(Vec3::zero()));

        let mut stored = Inventory::with_empty();
        let mut apples = Item::new_from_asset_expect("common.items.food.apple");
        apples.set_amount(7).unwrap();
        stored.push(apples).unwrap();
        stored
            .push(Item::new_from_asset_expect("common.items.weapons.sword.starter"))
            .unwrap();
        warehouse.save(&stored);

        let (npc, _) = warehouse.storekeeper().unwrap();
        let mut items = npc
            .inventory
            .slots()
            .flatten()
            .map(|item| {
                let id = item.item_definition_id();
                (id.itemdef_id().unwrap_or_default().to_string(), item.amount())
            })
            .collect::<Vec<_>>();
        items.sort();
        assert_eq!(items, vec![
            ("common.items.food.apple".to_string(), 7),
            ("common.items.weapons.sword.starter".to_string(), 1),
        ]);
        assert!(matches!(
            npc.agent.unwrap().behavior.trading_behavior,
            TradingBehavior::Storage
        ));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
