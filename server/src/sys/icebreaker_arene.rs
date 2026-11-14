//! Icebreaker : mort au lieu d'entraînement (jalon O.5, `assets/icebreaker/arene.ron`). « À terre »
//! marche comme en expédition (`sys/icebreaker_mort.rs`) ; un joueur mort réapparaît seul à
//! l'entrée après `delai_reapparition` secondes : la réapparition de Veloren le remet debout, PV
//! pleins, au point d'apparition, avec tout ce qu'il porte.
//! Les ennemis ne sortent pas de l'arène : l'entrée et les mannequins restent sûrs (hors tirs).

use common::{
    comp::{Agent, Alignment, Health, Player, Pos},
    event::{EventBus, RespawnEvent},
    resources::Time,
};
use common_ecs::{Job, Origin, Phase, System};
use hashbrown::HashMap;
use specs::{Entities, Entity, Join, Read, ReadStorage, Write, WriteStorage};
use vek::Vec2;

/// Le lieu d'entraînement en cours : le centre de l'arène (rien ailleurs), et depuis quand chaque
/// joueur est mort (temps du serveur).
#[derive(Default)]
pub struct IcebreakerArene {
    pub centre: Option<Vec2<f32>>,
    pub morts: HashMap<Entity, f64>,
}

/// Où remettre un ennemi en `pos` qui sort de l'arène (centre, rayon) : juste dedans.
pub fn retenu(pos: Vec2<f32>, centre: Vec2<f32>, rayon: f32) -> Option<Vec2<f32>> {
    let ecart = pos - centre;
    (ecart.magnitude_squared() > rayon * rayon)
        .then(|| centre + ecart.try_normalized().unwrap_or(Vec2::unit_y()) * (rayon - 1.0))
}

/// Un joueur mort depuis `depuis` réapparaît-il au temps `maintenant` ?
pub fn doit_reapparaitre(depuis: f64, maintenant: f64, delai: f64) -> bool { maintenant - depuis >= delai }

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Write<'a, IcebreakerArene>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Player>,
        Read<'a, EventBus<RespawnEvent>>,
        ReadStorage<'a, Agent>,
        ReadStorage<'a, Alignment>,
        WriteStorage<'a, Pos>,
    );

    const NAME: &'static str = "icebreaker_arene";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (entities, time, mut arene, healths, players, respawn, agents, alignements, mut positions): Self::SystemData,
    ) {
        let Some(centre) = arene.centre else {
            return;
        };
        let config = common::icebreaker_arene::config();
        let delai = config.delai_reapparition;
        // Les ennemis restent dans l'arène.
        for (_, alignement, pos) in (&agents, &alignements, &mut positions).join() {
            if matches!(alignement, Alignment::Enemy)
                && let Some(dedans) = retenu(pos.0.xy(), centre, config.rayon)
            {
                pos.0 = dedans.with_z(pos.0.z);
            }
        }
        let mut encore = HashMap::new();
        for (entity, health, _) in (&entities, &healths, &players).join() {
            if !health.is_dead {
                continue;
            }
            let depuis = arene.morts.get(&entity).copied().unwrap_or(time.0);
            if doit_reapparaitre(depuis, time.0, delai) {
                respawn.emit_now(RespawnEvent(entity));
            } else {
                encore.insert(entity, depuis);
            }
        }
        arene.morts = encore;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arene_retient_les_ennemis() {
        let centre = Vec2::new(100.0, 100.0);
        assert_eq!(retenu(Vec2::new(110.0, 100.0), centre, 24.0), None);
        assert_eq!(retenu(Vec2::new(130.0, 100.0), centre, 24.0), Some(Vec2::new(123.0, 100.0)));
    }

    #[test]
    fn arene_reapparition_apres_le_delai() {
        assert!(!doit_reapparaitre(10.0, 12.0, 4.0));
        assert!(doit_reapparaitre(10.0, 14.0, 4.0));
        assert!(doit_reapparaitre(10.0, 60.0, 4.0));
    }
}
