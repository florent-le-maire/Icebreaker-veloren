//! Icebreaker : pluie de projectiles sur une zone visée (ultime « Volée
//! mortelle », A.23 ; règles et réglages : `common::icebreaker_pluie`).
//! L'attaque à distance qui porte `icebreaker_pluie` émet un
//! `IcebreakerPluieEvent` ; la pluie vit ici le temps de ses salves : un cercle
//! au sol à chaque tic (les joueurs voient où ça tombe), puis les projectiles
//! qui partent du ciel, salve après salve.

use crate::Server;
use common::{
    comp::{Body, LightEmitter, Pos, Projectile},
    event::{EventBus, IcebreakerPluieEvent, ShootEvent},
    icebreaker_pluie::Pluie,
    outcome::Outcome,
    resources::Time,
    terrain::TerrainGrid,
    util::Dir,
    vol::ReadVol,
};
use common_ecs::{Job, Origin, Phase, System};
use rand::RngExt;
use specs::{Entities, Entity, Read, Write, WorldExt};
use vek::Vec3;

/// Secondes pendant lesquelles le cercle reste après la dernière salve (le
/// temps que les projectiles touchent le sol).
const RESTE_APRES: f64 = 0.6;

/// Une pluie en cours.
pub struct PluieEnCours {
    lanceur: Entity,
    centre: Vec3<f32>,
    /// Hauteur d'où partent les projectiles (réduite sous un plafond).
    hauteur: f32,
    pluie: Pluie,
    nee: f64,
    tirees: u32,
    body: Body,
    projectile: Projectile,
    light: Option<LightEmitter>,
    speed: f32,
}

/// Icebreaker : les pluies en cours sur ce serveur.
#[derive(Default)]
pub struct IcebreakerPluies(Vec<PluieEnCours>);

/// Une pluie commence : on mesure la place sous le plafond et on la range.
pub fn handle_icebreaker_pluie(server: &mut Server, ev: IcebreakerPluieEvent) {
    let ecs = server.state().ecs();
    // Le centre est posé sur le sol : on part d'un peu au-dessus.
    let centre = ev.centre + Vec3::unit_z() * 0.3;
    let libre = ecs
        .read_resource::<TerrainGrid>()
        .ray(centre, centre + Vec3::unit_z() * ev.pluie.hauteur)
        .until(|b| b.is_filled())
        .cast()
        .0;
    let nee = ecs.read_resource::<Time>().0;
    ecs.write_resource::<IcebreakerPluies>().0.push(PluieEnCours {
        lanceur: ev.lanceur,
        centre,
        hauteur: (libre - 1.0).clamp(1.5, ev.pluie.hauteur.max(1.5)),
        pluie: ev.pluie,
        nee,
        tirees: 0,
        body: ev.body,
        projectile: ev.projectile,
        light: ev.light,
        speed: ev.speed,
    });
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Write<'a, IcebreakerPluies>,
        Read<'a, EventBus<ShootEvent>>,
        Read<'a, EventBus<Outcome>>,
    );

    const NAME: &'static str = "icebreaker_pluie";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(_job: &mut Job<Self>, (entities, time, mut pluies, tirs, outcomes): Self::SystemData) {
        if pluies.0.is_empty() {
            return;
        }
        let mut tirs = tirs.emitter();
        let mut rng = rand::rng();
        pluies.0.retain_mut(|p| {
            let age = time.0 - p.nee;
            if age > p.pluie.duree() + RESTE_APRES {
                return false;
            }
            outcomes.emit_now(Outcome::IcebreakerZone {
                pos: p.centre,
                radius: p.pluie.rayon,
            });
            let dues = p.pluie.salves_dues(age);
            // Le lanceur parti, les projectiles tombent sans propriétaire.
            let lanceur = entities.is_alive(p.lanceur).then_some(p.lanceur);
            while p.tirees < dues {
                p.tirees += 1;
                for _ in 0..p.pluie.par_salve {
                    let depart = p.pluie.depart(p.centre, p.hauteur, (rng.random(), rng.random()));
                    tirs.emit(ShootEvent {
                        entity: lanceur,
                        source_vel: None,
                        pos: Pos(depart),
                        dir: Dir::down(),
                        body: p.body,
                        projectile: p.projectile.clone(),
                        light: p.light,
                        speed: p.speed,
                        object: None,
                        marker: None,
                    });
                }
            }
            true
        });
    }
}
