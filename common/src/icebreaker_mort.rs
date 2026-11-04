//! Icebreaker : à terre, relève et mort (jalon T, `assets/icebreaker/mort.ron`).
//! À 0 PV un humanoïde tombe à terre (état de Veloren) ; un allié le relève avec une part de ses
//! PV ; passé un délai, il meurt. Mort en expédition, il laisse un sac qu'un allié fouille.

use crate::assets::{AssetExt, Ron};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct IcebreakerMort {
    /// Secondes à terre avant de mourir.
    pub delai_a_terre: f64,
    /// Part des PV maximum rendue à la relève (0 à 1).
    pub pv_releve: f32,
    /// Secondes pour fouiller le sac d'un mort.
    pub duree_fouille: f32,
}

impl Default for IcebreakerMort {
    fn default() -> Self {
        Self {
            delai_a_terre: 30.0,
            pv_releve: 0.25,
            duree_fouille: 3.0,
        }
    }
}

impl IcebreakerMort {
    /// Réglages lus dans `assets/icebreaker/mort.ron` (valeurs par défaut si le fichier manque).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.mort").map_or_else(|_| Self::default(), |m| m.read().0.clone())
    }

    /// Secondes qui restent à un joueur tombé à terre à `depuis`, à l'instant `maintenant`.
    pub fn reste_a_terre(&self, depuis: f64, maintenant: f64) -> f64 {
        (self.delai_a_terre - (maintenant - depuis)).max(0.0)
    }

    /// Le délai à terre est écoulé : le joueur meurt.
    pub fn doit_mourir(&self, depuis: f64, maintenant: f64) -> bool {
        maintenant - depuis >= self.delai_a_terre
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delai_a_terre() {
        let m = IcebreakerMort::default();
        assert!(!m.doit_mourir(10.0, 39.9));
        assert!(m.doit_mourir(10.0, 40.0));
        assert_eq!(m.reste_a_terre(10.0, 17.0), 23.0);
        assert_eq!(m.reste_a_terre(10.0, 99.0), 0.0);
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let m = Ron::<IcebreakerMort>::load_expect("icebreaker.mort").read().0.clone();
        assert!(m.delai_a_terre > 0.0);
        assert!((0.0..=1.0).contains(&m.pv_releve));
        assert!(m.duree_fouille > 0.0);
    }
}
