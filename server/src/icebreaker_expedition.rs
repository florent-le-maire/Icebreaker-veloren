//! Icebreaker: the expeditions leave the village from the « Départ » house
//! (V.7 to V.9). Its leader offers by dialogue the destinations: the places of
//! the campaign (`assets/icebreaker/lieux.ron`, shared with the main menu)
//! other than the village.

use crate::{
    icebreaker_services::say,
    icebreaker_warehouse::{chunk_of, village_npc},
};
use common::{
    assets::{AssetExt, Ron},
    comp::{self, Agent, Content, Controller, agent::TradingBehavior},
    event::NpcBuilder,
    rtsim::{Dialogue, DialogueId, DialogueKind, Response},
    uid::Uid,
};
use serde::Deserialize;
use server_agent::data::{AgentData, AgentEmitters};
use tracing::info;
use vek::*;

/// Where the expedition leader stands (none outside the village).
#[derive(Default)]
pub struct IcebreakerExpedition {
    leader_pos: Option<Vec3<f32>>,
}

impl IcebreakerExpedition {
    pub fn new(leader_pos: Option<Vec3<f32>>) -> Self {
        if let Some(pos) = leader_pos {
            info!(?pos, "Icebreaker: village expedition leader");
        }
        Self { leader_pos }
    }

    pub fn leader_chunk(&self) -> Option<Vec2<i32>> { self.leader_pos.map(chunk_of) }

    pub fn leader(&self) -> Option<(NpcBuilder, comp::Pos)> {
        let pos = self.leader_pos?;
        let npc = village_npc(
            "Chef d'expédition".to_string(),
            pos,
            TradingBehavior::Departure,
            Vec::new(),
        );
        Some((npc, comp::Pos(pos)))
    }
}

/// The answer to leave nowhere.
pub const NOT_NOW: u16 = u16::MAX;

/// A place of the campaign (fields of `lieux.ron` used here).
#[derive(Clone, Debug, Deserialize)]
pub struct Lieu {
    pub nom: String,
    pub arguments: Vec<String>,
    pub port: u16,
    pub actif: bool,
}

/// The places of the campaign, in the order of `lieux.ron`.
pub fn lieux() -> Vec<Lieu> {
    Ron::<Vec<Lieu>>::load_expect("icebreaker.lieux")
        .read()
        .0
        .clone()
}

/// The destinations of expeditions: the places other than the village, with
/// their index in `lieux.ron`.
pub fn destinations() -> Vec<(usize, Lieu)> {
    lieux()
        .into_iter()
        .enumerate()
        .filter(|(_, place)| place.arguments.first().is_none_or(|lieu| lieu != "village"))
        .collect()
}

/// On talk: asks `player` where the expedition goes.
pub fn offer_destinations(controller: &mut Controller, player: Uid) {
    let id = DialogueId(rand::random());
    let mut responses = destinations()
        .into_iter()
        .map(|(index, place)| {
            let msg = if place.actif {
                place.nom
            } else {
                format!("{} (pas encore ouvert)", place.nom)
            };
            (index as u16, Response::from(Content::Plain(msg)))
        })
        .collect::<Vec<_>>();
    responses.push((NOT_NOW, Content::Plain("Pas maintenant".to_string()).into()));
    say(controller, player, id, DialogueKind::Start);
    say(controller, player, id, DialogueKind::Question {
        tag: 0,
        msg: Content::Plain("Je prépare les expéditions. Où veux-tu partir ?".to_string()),
        responses,
    });
}

/// On the player's answer to [`offer_destinations`].
pub fn answer_destination(
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
    let place = lieux().get(usize::from(response_id)).cloned();
    let reply = match place {
        _ if response_id == NOT_NOW => "Reviens quand tu seras prêt.".to_string(),
        Some(place) if !place.actif => {
            format!("{} n'est pas encore ouvert.", place.nom)
        },
        Some(place) => format!(
            "Expédition vers {} : le départ n'est pas encore organisé, passe par le menu \
             principal.",
            place.nom
        ),
        None => return,
    };
    agent_data.chat_npc_if_allowed_to_speak(Content::Plain(reply), agent, emitters);
    say(controller, player, dialogue.id, DialogueKind::End);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_destinations_are_the_places_but_the_village() {
        let destinations = destinations();
        assert!(!destinations.is_empty());
        assert!(
            destinations
                .iter()
                .all(|(index, place)| place.nom == lieux()[*index].nom
                    && place.arguments[0] != "village")
        );
    }
}
