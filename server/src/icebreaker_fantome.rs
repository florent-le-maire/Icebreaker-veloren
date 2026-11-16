//! Icebreaker : le fantôme du village a un corps (jalon FT, décision 0021).
//!
//! Avec le registre, un joueur sans personnage n'est plus le spectateur de Veloren : il reçoit
//! un corps de fantôme (`common::icebreaker_fantome`) qu'il contrôle comme un personnage. Rien
//! n'est lu ni écrit en base : le fantôme n'existe que le temps d'être en jeu. Il n'a ni vie (rien
//! ne le blesse) ni sac.

use crate::{Server, SpawnPoint, client::Client};
use common::{
    ViewDistances,
    comp::{self, Presence, PresenceKind},
    uid::Uid,
};
use common_net::msg::ServerGeneral;
use specs::{Entity, WorldExt};
use vek::Vec3;

/// Le joueur `entity` entre au village en fantôme, sur le point d'arrivée.
pub fn entrer(server: &mut Server, entity: Entity, view_distances: ViewDistances) {
    let state = &mut server.state;
    let spawn_point = state.ecs().read_resource::<SpawnPoint>().0;
    let Some(uid) = state.read_component_copied::<Uid>(entity) else {
        return;
    };
    let alias = state
        .ecs()
        .read_storage::<comp::Player>()
        .get(entity)
        .map(|player| player.alias.clone())
        .unwrap_or_default();
    let body = common::icebreaker_fantome::CORPS;

    // Ce qu'il faut pour être contrôlé et vu, comme un personnage qui entre en jeu
    // (`initialize_character_data`, `update_character_data`), sans ce qui vient de la base.
    state.write_component_ignore_entity_dead(entity, comp::Controller::default());
    state.write_component_ignore_entity_dead(entity, comp::Pos(spawn_point));
    state.write_component_ignore_entity_dead(entity, comp::Vel(Vec3::zero()));
    state.write_component_ignore_entity_dead(entity, comp::Ori::default());
    state.write_component_ignore_entity_dead(entity, body.collider());
    state.write_component_ignore_entity_dead(entity, body);
    state.write_component_ignore_entity_dead(entity, body.mass());
    state.write_component_ignore_entity_dead(entity, body.density());
    state.write_component_ignore_entity_dead(entity, comp::CharacterState::default());
    state.write_component_ignore_entity_dead(entity, comp::CharacterActivity::default());
    state.write_component_ignore_entity_dead(entity, comp::Alignment::Owned(uid));
    state.write_component_ignore_entity_dead(entity, comp::Buffs::default());
    state.write_component_ignore_entity_dead(entity, comp::Auras::default());
    state.write_component_ignore_entity_dead(entity, comp::EnteredAuras::default());
    state.write_component_ignore_entity_dead(entity, comp::Combo::default());
    state.write_component_ignore_entity_dead(entity, comp::Stance::default());
    state.write_component_ignore_entity_dead(entity, comp::Energy::new(body));
    state.write_component_ignore_entity_dead(entity, comp::Poise::new(body));
    state.write_component_ignore_entity_dead(
        entity,
        comp::Stats::new(comp::Content::Plain(alias), body),
    );
    state.write_component_ignore_entity_dead(entity, comp::SkillSet::default());
    state.write_component_ignore_entity_dead(entity, comp::Inventory::with_empty());
    state.write_component_ignore_entity_dead(entity, comp::ForceUpdate::forced());
    state.write_component_ignore_entity_dead(
        entity,
        Presence::new(view_distances, PresenceKind::IcebreakerFantome),
    );

    // Le client a demandé à entrer en spectateur : la réponse d'un personnage lui dit qu'il a un
    // corps (`client/src/lib.rs`, `CharacterSuccess`).
    if let Some(client) = state.ecs().read_storage::<Client>().get(entity) {
        client.send_fallible(ServerGeneral::CharacterSuccess);
    }
}
