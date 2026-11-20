//! Icebreaker : le portail du retour au village (jalon T, T.10, `assets/icebreaker/portail.ron`).
//! Un joueur allume le portail d'une salle du donjon par une interaction de quelques secondes
//! (`IcebreakerRituel::AllumerPortail`, émis à la fin de l'interaction) ; allumé, le portail change
//! de corps et le reste. Un joueur qui le traverse (`sys::icebreaker_portail`) rentre au village
//! comme par `/rentrer`, chacun pour soi.

use crate::Server;
use common::{
    comp::{self, ChatType},
    event::{ChangeBodyEvent, EventBus, IcebreakerRituel, IcebreakerRituelEvent},
    icebreaker_portail::{IcebreakerPortail, PORTAIL, PORTAIL_ACTIF},
    outcome::Outcome,
    uid::IdMaps,
};
use common_net::msg::ServerGeneral;
use specs::WorldExt;
use vek::Rgb;

pub fn handle_rituel(server: &mut Server, ev: IcebreakerRituelEvent) {
    match ev.rituel {
        IcebreakerRituel::AllumerPortail(cible) => allumer(server, cible),
        // Les refus (déjà au village, plus de personnage) ne regardent que celui qui traverse.
        IcebreakerRituel::Traverser => {
            if let Err(refus) = crate::icebreaker_return::rentrer(server, ev.acteur) {
                server.notify_client(ev.acteur, ServerGeneral::server_msg(ChatType::CommandError, refus));
            }
        },
        IcebreakerRituel::Autel(cible) => crate::icebreaker_autel::prier(server, ev.acteur, cible),
    }
}

/// La lumière d'un de nos objets (portail éteint ou allumé, autel), d'après `portail.ron`.
pub fn lumiere_de(reglages: &IcebreakerPortail, body: &comp::Body) -> Option<comp::LightEmitter> {
    reglages.lumiere_de(body).map(|((r, g, b), force)| comp::LightEmitter {
        col: Rgb::new(r, g, b),
        strength: force,
        flicker: 0.3,
        animated: true,
        dir: None,
    })
}

/// Le portail éteint `cible` s'allume : son corps change, il éclaire, et il le reste.
fn allumer(server: &mut Server, cible: common::uid::Uid) {
    let ecs = server.state.ecs();
    let Some(portail) = ecs.read_resource::<IdMaps>().uid_entity(cible) else {
        return;
    };
    if ecs.read_storage::<comp::Body>().get(portail) != Some(&PORTAIL) {
        return;
    }
    let reglages = IcebreakerPortail::actuel();
    ecs.read_resource::<EventBus<ChangeBodyEvent>>().emit_now(ChangeBodyEvent {
        entity: portail,
        new_body: PORTAIL_ACTIF,
        permanent_change: None,
    });
    if let Some(lumiere) = lumiere_de(&reglages, &PORTAIL_ACTIF) {
        let _ = ecs.write_storage::<comp::LightEmitter>().insert(portail, lumiere);
    }
    if let Some(pos) = ecs.read_storage::<comp::Pos>().get(portail) {
        ecs.read_resource::<EventBus<Outcome>>().emit_now(Outcome::PortalActivated { pos: pos.0 });
    }
}

/// Icebreaker : `/aller autel|portail` (admin, pour les essais) : devant l'autel ou le portail le plus
/// proche, parmi ceux de tous les donjons générés du monde.
pub(crate) fn handle_aller(
    server: &mut Server,
    client: specs::Entity,
    _target: specs::Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), common::comp::Content> {
    use common::comp::Content;
    let role = match args.first().map(String::as_str) {
        Some("autel") => "altar",
        Some("portail") => "portal",
        _ => return Err(Content::Plain("/aller autel ou /aller portail".to_string())),
    };
    let ici = server
        .state
        .ecs()
        .read_storage::<comp::Pos>()
        .get(client)
        .map(|pos| pos.0)
        .ok_or_else(|| Content::Plain("Il faut être en jeu.".to_string()))?;
    let devant = server
        .index
        .sites
        .iter()
        .flat_map(|(_, site)| world::icebreaker::dungeon_specials(site))
        .filter(|(nom, _, _)| *nom == role)
        .map(|(_, _, devant)| devant)
        .min_by(|a, b| a.distance_squared(ici).total_cmp(&b.distance_squared(ici)))
        .ok_or_else(|| Content::Plain(format!("Aucun {} dans les donjons de ce monde.", args[0])))?;
    server
        .state
        .mut_resource::<EventBus<common::event::TeleportToPositionEvent>>()
        .emit_now(common::event::TeleportToPositionEvent {
            entity: client,
            position: devant,
        });
    Ok(())
}
