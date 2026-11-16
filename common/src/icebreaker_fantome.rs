//! Icebreaker : le fantôme du village (jalon FT, décision 0021, `assets/icebreaker/fantome.ron`).
//! Au village, un joueur sans personnage a un corps : un fantôme bleu un peu transparent, qui
//! marche, saute haut et retombe lentement (il plane, il ne vole pas).

use crate::{
    assets::{AssetExt, Ron},
    comp::{Body, object},
};
use serde::Deserialize;
use std::sync::LazyLock;

/// Le corps du fantôme.
pub const CORPS: Body = Body::Object(object::Body::IcebreakerFantome);

#[derive(Clone, Debug, Deserialize)]
pub struct FantomeConfig {
    /// Part des pixels du fantôme qui sont dessinés (0 : invisible, 1 : opaque).
    pub opacite: f32,
    /// Part de la gravité qui s'applique au fantôme (1 : comme tout le monde).
    pub gravite: f32,
    /// Vitesse de chute au plus, en blocs par seconde.
    pub chute_max: f32,
    /// Élan du saut, en multiple de celui d'un humain.
    pub saut: f32,
    /// Accélération au sol (un humain : 100).
    pub acceleration: f32,
}

/// Les réglages, lus une fois.
pub fn config() -> &'static FantomeConfig {
    static CONFIG: LazyLock<FantomeConfig> =
        LazyLock::new(|| Ron::<FantomeConfig>::load_expect("icebreaker.fantome").read().0.clone());
    &CONFIG
}

pub fn est_fantome(body: &Body) -> bool { *body == CORPS }

/// La gravité que subit `body` : réduite pour le fantôme.
pub fn gravite(body: &Body, gravite: f32) -> f32 {
    if est_fantome(body) { gravite * config().gravite } else { gravite }
}

/// La vitesse verticale de `body` après un pas de physique : la chute du fantôme est plafonnée.
pub fn chute(body: &Body, vitesse_z: f32) -> f32 { chute_avec(config(), body, vitesse_z) }

fn chute_avec(config: &FantomeConfig, body: &Body, vitesse_z: f32) -> f32 {
    if est_fantome(body) { vitesse_z.max(-config.chute_max) } else { vitesse_z }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icebreaker_fantome_fichier_de_donnees_lisible() {
        let c = config();
        assert!(c.opacite > 0.0 && c.opacite <= 1.0);
        assert!(c.gravite > 0.0 && c.gravite <= 1.0, "il plane, il ne vole pas");
        assert!(c.chute_max > 0.0 && c.saut > 0.0 && c.acceleration > 0.0);
    }

    #[test]
    fn icebreaker_fantome_la_chute_est_plafonnee_pour_lui_seul() {
        let c = FantomeConfig {
            opacite: 0.5,
            gravite: 0.3,
            chute_max: 4.0,
            saut: 1.5,
            acceleration: 80.0,
        };
        assert_eq!(chute_avec(&c, &CORPS, -30.0), -4.0);
        assert_eq!(chute_avec(&c, &CORPS, -2.0), -2.0);
        assert_eq!(chute_avec(&c, &CORPS, 9.0), 9.0, "le saut n'est pas plafonné");
        let humain = Body::Humanoid(crate::comp::humanoid::Body::random());
        assert_eq!(chute_avec(&c, &humain, -30.0), -30.0);
        assert!(est_fantome(&CORPS) && !est_fantome(&humain));
    }
}
