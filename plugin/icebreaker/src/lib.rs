//! Icebreaker : règles du jeu Icebreaker en plugin serveur de Veloren.
//!
//! Voir `docs/icebreaker/PLAN-MOD.md` : les fonctionnalités passent ici une à
//! une, à mesure que l'interface du plugin le permet.

wit_bindgen::generate!({
    path: "../wit/veloren.wit",
    world: "server-plugin",
});

use exports::veloren::plugin::{events, server_events};
use veloren::plugin::{
    actions,
    types::{GameMode, JoinResult, PlayerId, Uid},
};

struct Icebreaker;

impl events::Guest for Icebreaker {
    fn load(mode: GameMode) {
        actions::register_command("icebreaker");
        let mode = match mode {
            GameMode::Server => "serveur",
            GameMode::Client => "client",
            GameMode::SinglePlayer => "solo",
        };
        println!("Icebreaker {} chargé ({mode})", env!("CARGO_PKG_VERSION"));
    }
}

impl server_events::Guest for Icebreaker {
    fn join(_player_name: String, _player_id: PlayerId) -> JoinResult { JoinResult::None }

    fn command(command: String, _args: Vec<String>, _player: Uid) -> Result<Vec<String>, String> {
        match command.as_str() {
            "icebreaker" => Ok(vec![format!(
                "Icebreaker {} : le mod est chargé.",
                env!("CARGO_PKG_VERSION")
            )]),
            _ => Err(format!("Commande inconnue : {command}")),
        }
    }
}

export!(Icebreaker);
