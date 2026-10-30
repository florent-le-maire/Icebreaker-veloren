use crate::{client::Client, icebreaker_village::IcebreakerVillage};
use common::{
    comp::{ChatType, Content},
    resources::TimeOfDay,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Join, Read, ReadStorage, Write};

/// Icebreaker: the village's chantiers (C.3) are done once their map time has
/// come; their houses then stand (drawn at the next start of the server until
/// C.4).
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Write<'a, IcebreakerVillage>,
        Read<'a, TimeOfDay>,
        ReadStorage<'a, Client>,
    );

    const NAME: &'static str = "icebreaker_village";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(_job: &mut Job<Self>, (mut village, time_of_day, clients): Self::SystemData) {
        for _ in village.finish_due(time_of_day.0) {
            for client in (&clients).join() {
                client.send_fallible(ServerGeneral::server_msg(
                    ChatType::Meta,
                    Content::Plain("Une nouvelle maison est bâtie au village !".to_string()),
                ));
            }
        }
    }
}
