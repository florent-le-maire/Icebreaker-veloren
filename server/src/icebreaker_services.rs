//! Icebreaker: the village buildings run by an NPC on the square (V.4).
//!
//! - The intendant is a Veloren merchant whose stock and prices come from the
//!   town's economy (potions, food, and coins to buy what players bring back).
//! - The healer (`TradingBehavior::Infirmary`) heals fully for a price, by
//!   dialogue: talking to them (E) asks a question, the "heal" answer gives the
//!   coins (checked and moved by the dialogue event) and the server heals.

use crate::{
    icebreaker_warehouse::{chunk_of, village_npc},
    rtsim::tick::trader_loadout,
    sys::terrain::SpawnEntityData,
};
use common::{
    assets::AssetExt,
    calendar::Calendar,
    comp::{
        self, Agent, Content, ControlEvent, Controller, Item,
        agent::{Mark, TradingBehavior},
        inventory::{item::ItemDef, loadout_builder::LoadoutBuilder},
    },
    event::NpcBuilder,
    generation::{EntityConfig, EntityInfo},
    resources::TimeOfDay,
    rtsim::{Dialogue, DialogueId, DialogueKind, Response},
    trade::{Good, SiteInformation},
    uid::Uid,
};

use server_agent::data::{AgentData, AgentEmitters};
use std::sync::Arc;
use tracing::{info, warn};
use vek::*;

/// The healer's answers.
pub const HEAL: u16 = 0;
pub const NO_THANKS: u16 = 1;
/// Not an answer offered to the player: what the server turns "heal" into when
/// the player could not pay.
pub const NOT_PAID: u16 = 2;

const INTENDANT_CONFIG: &str = "common.entity.village.merchant";

#[derive(Default)]
pub struct IcebreakerServices {
    intendant: Option<EntityInfo>,
    healer_pos: Option<Vec3<f32>>,
    heal_price: u32,
}

impl IcebreakerServices {
    /// The intendant at `intendant_pos`, trading with the town's `economy`,
    /// and the healer at `healer_pos` (none outside the village).
    pub fn new(
        intendant_pos: Option<Vec3<f32>>,
        economy: Option<SiteInformation>,
        healer_pos: Option<Vec3<f32>>,
        heal_price: u32,
    ) -> Self {
        let intendant = intendant_pos.and_then(|pos| {
            let Some(economy) = economy else {
                warn!("Icebreaker: the town has no economy, no intendant");
                return None;
            };
            info!(?pos, "Icebreaker: village intendant");
            Some(
                EntityInfo::at(pos)
                    .with_entity_config(
                        EntityConfig::from_asset_expect_owned(INTENDANT_CONFIG),
                        Some(INTENDANT_CONFIG),
                        &mut rand::rng(),
                        None,
                    )
                    .with_alignment(comp::Alignment::Npc)
                    .with_economy(&economy)
                    .with_lazy_loadout(intendant_loadout)
                    .with_name(Content::Plain("Intendant".to_string()))
                    .with_agent_mark(Mark::Merchant),
            )
        });
        if let Some(pos) = healer_pos {
            info!(?pos, heal_price, "Icebreaker: village healer");
        }
        Self {
            intendant,
            healer_pos,
            heal_price,
        }
    }

    pub fn intendant_chunk(&self) -> Option<Vec2<i32>> {
        self.intendant.as_ref().map(|info| chunk_of(info.pos))
    }

    pub fn intendant(&self) -> Option<(NpcBuilder, comp::Pos)> {
        let SpawnEntityData::Npc(data) = SpawnEntityData::from_entity_info(self.intendant.clone()?)
        else {
            return None;
        };
        let (mut npc, pos) = data.to_npc_builder();
        // Stays at its stall, and greets as the intendant.
        npc.agent = npc.agent.map(|mut agent| {
            agent.mark = Some(Mark::Intendant);
            agent.with_patrol_origin(pos.0).with_idle_wander_factor(0.0)
        });
        Some((npc, pos))
    }

    pub fn healer_chunk(&self) -> Option<Vec2<i32>> { self.healer_pos.map(chunk_of) }

    pub fn healer(&self) -> Option<(NpcBuilder, comp::Pos)> {
        let pos = self.healer_pos?;
        let npc = village_npc(
            "Infirmier".to_string(),
            pos,
            TradingBehavior::Infirmary {
                price: self.heal_price,
            },
            Vec::new(),
        );
        Some((npc, comp::Pos(pos)))
    }
}

/// What the intendant sells: basic supplies, and coins to buy the rest.
fn intendant_loadout(
    loadout_builder: LoadoutBuilder,
    economy: Option<&SiteInformation>,
    _time: Option<&(TimeOfDay, Calendar)>,
) -> LoadoutBuilder {
    trader_loadout(
        loadout_builder,
        economy,
        |good| matches!(good, Good::Potions | Good::Food | Good::Coin),
        |_| true,
        1000.0..3000.0,
    )
}

fn coins() -> Arc<ItemDef> {
    Arc::<ItemDef>::load_cloned(crate::icebreaker_treasury::COINS).expect("coins item")
}

fn say(controller: &mut Controller, player: Uid, id: DialogueId, kind: DialogueKind) {
    controller.push_event(ControlEvent::Dialogue(player, Dialogue { id, kind }));
}

/// On talk: asks `player` whether to heal them for `price` coins, or tells
/// them they are not hurt (`None`).
pub fn offer_healing(
    controller: &mut Controller,
    agent_data: &AgentData,
    agent: &Agent,
    emitters: &mut AgentEmitters,
    player: Uid,
    price: Option<u32>,
) {
    let Some(price) = price else {
        agent_data.chat_npc_if_allowed_to_speak(
            Content::Plain("Tu n'es pas blessé. Reviens me voir après le donjon.".to_string()),
            agent,
            emitters,
        );
        return;
    };
    let id = DialogueId(rand::random());
    say(controller, player, id, DialogueKind::Start);
    say(controller, player, id, DialogueKind::Question {
        tag: 0,
        msg: Content::Plain(format!(
            "Je tiens l'infirmerie. Je te soigne entièrement pour {price} pièces d'or."
        )),
        responses: vec![
            (HEAL, Response {
                msg: Content::Plain(format!("Me soigner ({price} or)")),
                given_item: Some((coins(), price)),
            }),
            (NO_THANKS, Content::Plain("Non merci".to_string()).into()),
        ],
    });
}

/// On the player's answer to [`offer_healing`] (the healing itself is done by
/// the dialogue event, which takes the coins).
pub fn answer_healing(
    controller: &mut Controller,
    agent_data: &AgentData,
    agent: &Agent,
    emitters: &mut AgentEmitters,
    player: Uid,
    dialogue: &Dialogue<true>,
) {
    let DialogueKind::Response { response_id, .. } = dialogue.kind else {
        return;
    };
    let reply = match response_id {
        HEAL => "Voilà, te revoilà sur pied.".to_string(),
        NOT_PAID => match agent.behavior.trading_behavior {
            TradingBehavior::Infirmary { price } => {
                format!("Il te faut {price} pièces d'or pour les soins.")
            },
            _ => return,
        },
        _ => "Comme tu veux. Reviens si tu es blessé.".to_string(),
    };
    agent_data.chat_npc_if_allowed_to_speak(Content::Plain(reply), agent, emitters);
    say(controller, player, dialogue.id, DialogueKind::End);
}

/// The coins of a healing are not kept: the healer's inventory never fills.
pub fn drop_payment(healer: &mut comp::Inventory) { healer.drain().for_each(|_: Item| {}); }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_intendant_sells_supplies_and_the_healer_asks_its_price() {
        let economy = SiteInformation {
            id: 7,
            unconsumed_stock: [(Good::Potions, 200.0), (Good::Food, 200.0), (Good::Coin, 5000.0)]
                .into_iter()
                .collect(),
        };
        let services =
            IcebreakerServices::new(Some(Vec3::zero()), Some(economy), Some(Vec3::unit_x()), 60);

        let (intendant, _) = services.intendant().unwrap();
        let agent = intendant.agent.unwrap();
        assert_eq!(agent.mark, Some(Mark::Intendant));
        assert!(matches!(
            agent.behavior.trading_behavior,
            TradingBehavior::RequireBalanced { trade_site: 7 }
        ));
        assert!(intendant.inventory.slots().flatten().next().is_some(), "empty stock");

        let (healer, _) = services.healer().unwrap();
        assert!(matches!(
            healer.agent.unwrap().behavior.trading_behavior,
            TradingBehavior::Infirmary { price: 60 }
        ));

        assert!(IcebreakerServices::new(None, None, None, 60).intendant().is_none());
    }
}
