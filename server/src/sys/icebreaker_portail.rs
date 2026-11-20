//! Icebreaker : traverser le portail allumé (jalon T, T.10). Un joueur vivant et debout qui passe
//! dans l'anneau d'un portail allumé rentre au village : le retour lui-même
//! (`icebreaker_portail::handle_rituel`) se fait hors des systèmes, par un évènement.

use common::{
    comp::{Body, CharacterState, Health, Player, Pos, is_downed},
    event::{EventBus, IcebreakerRituel, IcebreakerRituelEvent},
    icebreaker_portail::{IcebreakerPortail, PORTAIL_ACTIF},
};
use common_ecs::{Job, Origin, Phase, System};
use common::resources::Time;
use hashbrown::HashMap;
use specs::{Entities, Entity, Join, Read, ReadStorage, Write};

/// Icebreaker : quand chaque joueur a traversé un portail (temps du serveur) : le retour au village
/// prend un instant, on ne le demande pas deux fois.
#[derive(Default)]
pub struct IcebreakerTraversees(pub HashMap<Entity, f64>);

/// Secondes avant qu'un même joueur puisse traverser encore.
const ATTENTE: f64 = 5.0;

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, CharacterState>,
        Read<'a, EventBus<IcebreakerRituelEvent>>,
        Read<'a, Time>,
        Write<'a, IcebreakerTraversees>,
    );

    const NAME: &'static str = "icebreaker_portail";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (entities, bodies, positions, players, healths, etats, rituels, time, mut traversees): Self::SystemData,
    ) {
        let portails = (&bodies, &positions)
            .join()
            .filter(|(body, _)| **body == PORTAIL_ACTIF)
            .map(|(_, pos)| pos.0)
            .collect::<Vec<_>>();
        traversees.0.retain(|_, quand| time.0 - *quand < ATTENTE);
        if portails.is_empty() {
            return;
        }
        let reglages = IcebreakerPortail::actuel();
        let mut rituels = rituels.emitter();
        for (acteur, _, pos, health) in (&entities, &players, &positions, &healths).join() {
            let debout = !health.is_dead && !is_downed(Some(health), etats.get(acteur));
            if debout
                && !traversees.0.contains_key(&acteur)
                && portails.iter().any(|portail| reglages.traverse(*portail, pos.0))
            {
                traversees.0.insert(acteur, time.0);
                rituels.emit(IcebreakerRituelEvent {
                    acteur,
                    rituel: IcebreakerRituel::Traverser,
                });
            }
        }
    }
}
