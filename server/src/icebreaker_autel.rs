//! Icebreaker : l'autel de résurrection (jalon T, T.12).

use crate::Server;
use common::uid::Uid;
use specs::Entity as EcsEntity;

/// `prieur` a prié jusqu'au bout à l'autel `autel` (T.12, à venir).
pub fn prier(_server: &mut Server, _prieur: EcsEntity, _autel: Uid) {}
