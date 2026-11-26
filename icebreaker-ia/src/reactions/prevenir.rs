//! Aller prévenir (`design/creatures.md`) : « celui qui tire "aller prévenir" court vers l'allié non
//! alerté le plus proche (il connaît les postes des siens) et crie ; le cri porte un signalement (où
//! il a vu les intrus, quand) ».

use crate::{
    cerveau::{Allie, Id, Observation},
    profil::Profil,
};
use vek::Vec3;

/// Distance à l'allié à laquelle il crie quand le profil ne dit rien (`prevenir_a`).
const A_PORTEE_DE_VOIX: f32 = 6.0;

/// Une course vers un allié, avec ce qu'il a à lui dire.
#[derive(Clone, Copy, Debug)]
pub struct Prevenir {
    /// L'allié qu'il va prévenir.
    pub qui: Id,
    /// Le signalement : où il a vu les intrus (ou entendu tomber un allié), et quand.
    pub lieu: Vec3<f32>,
    pub vu_a: f64,
    /// Parti à cette heure.
    pub depuis: f64,
}

/// Où il en est.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Etape {
    /// Il court vers l'allié, qui est là.
    Courir(Vec3<f32>),
    /// Il est à portée de voix : il crie.
    Crier,
    /// L'allié a disparu, ou la course a trop duré.
    Renoncer,
}

/// L'allié à prévenir : le plus proche qui ne sait rien. Personne si un voisin est déjà parti
/// prévenir (« pas deux messagers »).
pub fn qui<'a>(obs: &'a Observation) -> Option<&'a Allie> {
    if obs.allies.iter().any(|a| a.messager) {
        return None;
    }
    obs.allies
        .iter()
        .filter(|a| !a.alerte)
        .min_by(|a, b| a.pos.distance_squared(obs.pos).total_cmp(&b.pos.distance_squared(obs.pos)))
}

impl Prevenir {
    pub fn etape(&self, obs: &Observation, profil: &Profil) -> Etape {
        let Some(allie) = obs.allies.iter().find(|a| a.id == self.qui) else {
            return Etape::Renoncer;
        };
        if allie.pos.distance(obs.pos) <= profil.prevenir_a.unwrap_or(A_PORTEE_DE_VOIX) {
            Etape::Crier
        } else if obs.temps - self.depuis > profil.trajet.unwrap_or(0.0) {
            Etape::Renoncer
        } else {
            Etape::Courir(allie.pos)
        }
    }
}
