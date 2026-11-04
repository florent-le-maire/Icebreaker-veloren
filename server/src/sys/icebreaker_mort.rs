//! Icebreaker : à terre, on meurt passé `delai_a_terre` secondes si personne ne relève (jalon T,
//! `assets/icebreaker/mort.ron`). Veloren garde à terre sans limite ; ici on tue après le délai,
//! et la mort suit son chemin habituel (la santé à zéro sans protection donne `DestroyEvent`).

use common::{
    comp::{CharacterState, Health, is_downed},
    icebreaker_mort::IcebreakerMort,
    resources::Time,
};
use common_ecs::{Job, Origin, Phase, System};
use hashbrown::HashMap;
use specs::{Entities, Entity, Join, Read, ReadStorage, Write, WriteStorage};

/// Icebreaker : depuis quand chaque entité est à terre (temps du serveur).
#[derive(Default)]
pub struct IcebreakerATerre {
    pub a_terre: HashMap<Entity, f64>,
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Write<'a, IcebreakerATerre>,
        WriteStorage<'a, Health>,
        ReadStorage<'a, CharacterState>,
    );

    const NAME: &'static str = "icebreaker_mort";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (entities, time, mut etat, mut healths, character_states): Self::SystemData,
    ) {
        let regles = IcebreakerMort::actuel();
        let maintenant = time.0;
        let mut encore = HashMap::new();
        let mut a_tuer = Vec::new();
        for (entity, health, character_state) in (&entities, &healths, &character_states).join() {
            if !is_downed(Some(health), Some(character_state)) {
                continue;
            }
            let depuis = etat.a_terre.get(&entity).copied().unwrap_or(maintenant);
            if regles.doit_mourir(depuis, maintenant) {
                a_tuer.push(entity);
            } else {
                encore.insert(entity, depuis);
            }
        }
        etat.a_terre = encore;
        for entity in a_tuer {
            if let Some(mut health) = healths.get_mut(entity) {
                health.kill();
            }
        }
    }
}
