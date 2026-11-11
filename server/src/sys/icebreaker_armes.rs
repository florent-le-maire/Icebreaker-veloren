//! Icebreaker : dit une fois au joueur quand il tient une arme qu'il n'a pas apprise (jalon A.7).
//! Une fois par seconde, on regarde les armes en mains de chaque personnage qui a nos compétences ;
//! le message ne part que quand l'état change.

use crate::{
    client::Client,
    icebreaker_classes::{EtatArme, etat_arme},
};
use common::{
    comp::{ChatType, Content, Inventory},
    icebreaker_competences::IcebreakerCompetences,
    resources::Time,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Entities, Entity, Join, Read, ReadStorage, Write};
use std::collections::HashMap;

/// Le dernier état connu par personnage, et le moment du dernier passage.
#[derive(Default)]
pub struct IcebreakerArmes {
    last_run: f64,
    etats: HashMap<Entity, EtatArme>,
}

impl IcebreakerArmes {
    /// Enregistre l'état de `entity` ; le message à envoyer si l'état vient de changer.
    fn changer(&mut self, entity: Entity, etat: EtatArme) -> Option<String> {
        if self.etats.get(&entity) == Some(&etat) {
            return None;
        }
        let message = etat.message();
        self.etats.insert(entity, etat);
        message
    }
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Write<'a, IcebreakerArmes>,
        ReadStorage<'a, IcebreakerCompetences>,
        ReadStorage<'a, Inventory>,
        ReadStorage<'a, Client>,
    );

    const NAME: &'static str = "icebreaker_armes";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (entities, time, mut armes, competences, inventaires, clients): Self::SystemData,
    ) {
        if time.0 - armes.last_run < 1.0 {
            return;
        }
        armes.last_run = time.0;
        armes.etats.retain(|entity, _| entities.is_alive(*entity));
        for (entity, competences, inventaire, client) in
            (&entities, &competences, &inventaires, &clients).join()
        {
            if let Some(message) = armes.changer(entity, etat_arme(competences, inventaire)) {
                client.send_fallible(ServerGeneral::server_msg(
                    ChatType::CommandInfo,
                    Content::Plain(message),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn icebreaker_armes_le_message_ne_part_qu_au_changement() {
        let mut world = World::new();
        let e = world.create_entity().build();
        let mut armes = IcebreakerArmes::default();
        assert_eq!(armes.changer(e, EtatArme::Rien), None);
        let non_appris = EtatArme::NonAppris("Marteau".into());
        assert_eq!(
            armes.changer(e, non_appris.clone()).as_deref(),
            Some("Marteau : arme non apprise. Attaques de base seulement.")
        );
        assert_eq!(armes.changer(e, non_appris), None, "une seule fois");
        assert_eq!(armes.changer(e, EtatArme::Appris), None);
        assert!(armes.changer(e, EtatArme::Etrangere).is_some());
    }
}
