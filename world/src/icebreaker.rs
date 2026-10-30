//! Icebreaker: encounter instances. A server started with
//! `ICEBREAKER_INSTANCE_SITE` set generates a small world around one chosen
//! site, without the wildlife and roaming monsters of a whole living world.

use crate::site::SiteKind;
use tracing::warn;

const INSTANCE_SITE_VAR: &str = "ICEBREAKER_INSTANCE_SITE";

/// Whether this server runs an encounter instance.
pub fn is_instance() -> bool { std::env::var_os(INSTANCE_SITE_VAR).is_some() }

/// The site kind the instance is built around: a dungeon (e.g. `Haniwa`) or,
/// for the village, a town (e.g. `Town`).
pub fn instance_site() -> Option<SiteKind> {
    let name = std::env::var(INSTANCE_SITE_VAR).ok()?;
    let kind = match name.as_str() {
        "Town" => SiteKind::Refactor,
        "CliffTown" => SiteKind::CliffTown,
        "DesertCity" => SiteKind::DesertCity,
        "SavannahTown" => SiteKind::SavannahTown,
        "CoastalTown" => SiteKind::CoastalTown,
        "Gnarling" => SiteKind::Gnarling,
        "Adlet" => SiteKind::Adlet,
        "Haniwa" => SiteKind::Haniwa,
        "Terracotta" => SiteKind::Terracotta,
        "Myrmidon" => SiteKind::Myrmidon,
        "Sahagin" => SiteKind::Sahagin,
        "Cultist" => SiteKind::Cultist,
        "DwarvenMine" => SiteKind::DwarvenMine,
        "ChapelSite" => SiteKind::ChapelSite,
        "VampireCastle" => SiteKind::VampireCastle,
        _ => {
            warn!(?name, "Unknown {INSTANCE_SITE_VAR}, no site placed");
            return None;
        },
    };
    Some(kind)
}

/// Whether the instance is the village (built around a town).
pub fn is_village() -> bool { instance_site().is_some_and(is_town) }

/// Whether `kind` is a town.
pub fn is_town(kind: SiteKind) -> bool {
    matches!(
        kind,
        SiteKind::Refactor
            | SiteKind::CliffTown
            | SiteKind::DesertCity
            | SiteKind::SavannahTown
            | SiteKind::CoastalTown
    )
}
