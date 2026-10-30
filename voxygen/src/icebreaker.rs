//! Icebreaker: travelling between the places of the campaign (V.8). A server
//! tells the client to go to another place (`IcebreakerTravel`): the session
//! notes where and with which character, and goes back to the main menu,
//! which connects there; the character selection then plays that character.

use common::character::CharacterId;
use std::sync::Mutex;

struct Travel {
    /// Where to connect, `host:port`; taken by the main menu.
    address: Option<String>,
    /// Who to play there; taken by the character selection.
    character: Option<CharacterId>,
}

static TRAVEL: Mutex<Option<Travel>> = Mutex::new(None);

fn travel() -> std::sync::MutexGuard<'static, Option<Travel>> {
    TRAVEL.lock().unwrap_or_else(|e| e.into_inner())
}

/// Goes to the place listening on `port`, on the host of `current_server`
/// (`host:port`), with `character`.
pub fn go(current_server: &str, port: u16, character: Option<CharacterId>) {
    let host = current_server
        .rsplit_once(':')
        .map_or(current_server, |(host, _)| host);
    let host = if host.is_empty() { "localhost" } else { host };
    *travel() = Some(Travel {
        address: Some(format!("{host}:{port}")),
        character,
    });
}

/// Where to connect, if travelling.
pub fn take_address() -> Option<String> { travel().as_mut()?.address.take() }

/// Who to play, if travelling.
pub fn take_character() -> Option<CharacterId> { travel().take()?.character }
