//! Icebreaker : la balise lumineuse (jalon J). Prévient quand la charge revient, et oublie les
//! joueurs partis et les balises effacées (fin d'expédition, emplacement libéré).

use crate::{client::Client, icebreaker_balise::IcebreakerBalises};
use common::{
    comp::{ChatType, Content},
    icebreaker_balise::IcebreakerBalise,
    resources::Time,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Entities, Read, ReadStorage, Write};

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Write<'a, IcebreakerBalises>,
        ReadStorage<'a, Client>,
    );

    const NAME: &'static str = "icebreaker_balise";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(_job: &mut Job<Self>, (entities, time, mut balises, clients): Self::SystemData) {
        if balises.posee_a.is_empty() && balises.au_sol.is_empty() && balises.couleur.is_empty() {
            return;
        }
        let regles = IcebreakerBalise::actuel();
        let maintenant = time.0;
        balises.posee_a.retain(|joueur, posee| {
            if !entities.is_alive(*joueur) {
                return false;
            }
            if regles.reste_recharge(*posee, maintenant) > 0.0 {
                return true;
            }
            if let Some(client) = clients.get(*joueur) {
                client.send_fallible(ServerGeneral::server_msg(
                    ChatType::CommandInfo,
                    Content::Plain("Balise prête.".to_string()),
                ));
            }
            false
        });
        balises
            .au_sol
            .retain(|(balise, _)| entities.is_alive(*balise));
        balises.couleur.retain(|joueur, _| entities.is_alive(*joueur));
    }
}
