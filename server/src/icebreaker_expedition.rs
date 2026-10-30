//! Icebreaker: the expeditions leave the village from the « Départ » house
//! (V.7 to V.9). Its leader offers by dialogue the destinations: the places of
//! the campaign (`assets/icebreaker/lieux.ron`, shared with the main menu)
//! other than the village. Going there (V.8): the village server starts the
//! place's server if needed, and tells the client to go (`IcebreakerTravel`);
//! the client leaves and connects there with the same character.

use crate::{
    client::Client,
    icebreaker_services::say,
    icebreaker_warehouse::{chunk_of, village_npc},
};
use common::{
    assets::{AssetExt, Ron},
    comp::{self, Agent, ChatType, Content, Controller, agent::TradingBehavior},
    event::NpcBuilder,
    rtsim::{Dialogue, DialogueId, DialogueKind, Response},
    uid::Uid,
};
use common_net::msg::ServerGeneral;
use serde::Deserialize;
use server_agent::data::{AgentData, AgentEmitters};
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    process::Command,
    time::Duration,
};
use tracing::{info, warn};
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
        // The travel itself is started by the dialogue event (`travel`).
        Some(place) => format!("En route vers {} !", place.nom),
        None => return,
    };
    agent_data.chat_npc_if_allowed_to_speak(Content::Plain(reply), agent, emitters);
    say(controller, player, dialogue.id, DialogueKind::End);
}

/// The destination chosen by this answer to the expedition leader, if one
/// can be gone to.
pub fn chosen_destination(response_id: u16) -> Option<Lieu> {
    destinations()
        .into_iter()
        .find(|(index, lieu)| *index == usize::from(response_id) && lieu.actif)
        .map(|(_, lieu)| lieu)
}

/// The village, where `/rentrer` leads.
pub fn village() -> Option<Lieu> {
    lieux()
        .into_iter()
        .find(|lieu| lieu.arguments.first().is_some_and(|lieu| lieu == "village"))
}

/// Starts the server of `lieu` if nothing listens on its port yet, from the
/// root of the repository (where this server runs): `icebreaker.ps1` in a
/// console of its own on Windows, `icebreaker.sh` in the background elsewhere.
pub fn ensure_running(lieu: &Lieu) -> Result<(), String> {
    let local = SocketAddr::from((Ipv4Addr::LOCALHOST, lieu.port));
    if TcpStream::connect_timeout(&local, Duration::from_millis(300)).is_ok() {
        return Ok(());
    }
    #[cfg(windows)]
    let mut command = {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        let mut command = Command::new("powershell");
        command
            .args(["-NoExit", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg("icebreaker.ps1")
            .args(&lieu.arguments)
            .args(["-Port", &lieu.port.to_string()])
            .creation_flags(CREATE_NEW_CONSOLE);
        command
    };
    #[cfg(not(windows))]
    let mut command = {
        // `icebreaker.sh <lieu>`, the site and port by environment variables.
        let mut command = Command::new("sh");
        command
            .arg("icebreaker.sh")
            .args(lieu.arguments.first())
            .env("ICEBREAKER_PORT", lieu.port.to_string());
        if let Some(site) = lieu.arguments.windows(2).find(|w| w[0] == "-Site").map(|w| &w[1]) {
            command.env("ICEBREAKER_SITE", site);
        }
        command
    };
    command
        .spawn()
        .map(|_| info!(lieu = %lieu.nom, port = lieu.port, "Icebreaker: place server started"))
        .map_err(|err| format!("Icebreaker : impossible de lancer {} : {err}", lieu.nom))
}

/// Sends the player of `client` to `lieu`, starting its server if needed.
pub fn travel(client: &Client, lieu: &Lieu) {
    match ensure_running(lieu) {
        Ok(()) => client.send_fallible(ServerGeneral::IcebreakerTravel(lieu.port)),
        Err(err) => {
            warn!(%err, "Icebreaker: no travel");
            client.send_fallible(ServerGeneral::server_msg(ChatType::Meta, Content::Plain(err)));
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_place_already_listening_is_not_started_again() {
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let lieu = Lieu {
            nom: "Essai".to_string(),
            // Would fail to start: nothing is started.
            arguments: vec!["nulle-part".to_string()],
            port: listener.local_addr().unwrap().port(),
            actif: true,
        };
        assert_eq!(ensure_running(&lieu), Ok(()));
        assert!(village().is_some_and(|village| village.actif));
        assert!(chosen_destination(NOT_NOW).is_none());
    }

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
