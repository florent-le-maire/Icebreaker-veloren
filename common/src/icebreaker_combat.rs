//! Icebreaker : combat létal façon Dark Souls (`assets/icebreaker/combat.ron`).
//! Les dégâts des attaques sont multipliés selon la cible : une créature
//! ordinaire tombe en quelques coups, un joueur aussi. Les projectiles ne
//! dérivent pas avec le tireur (X.13).

use crate::assets::{AssetExt, Ron};
use serde::Deserialize;
use vek::Vec3;

#[derive(Clone, Debug, Deserialize)]
pub struct IcebreakerCombat {
    /// Multiplicateur des dégâts reçus par une créature (PNJ).
    pub degats_contre_creatures: f32,
    /// Multiplicateur des dégâts reçus par un joueur.
    pub degats_contre_joueurs: f32,
    /// Part de la vitesse du tireur reprise par ses projectiles (1 : comme
    /// Veloren).
    #[serde(default = "part_vitesse_tireur_defaut")]
    pub part_vitesse_tireur: f32,
}

fn part_vitesse_tireur_defaut() -> f32 { 1.0 }

impl IcebreakerCombat {
    /// Multiplicateur d'une attaque ; sans attaquant (piège, environnement),
    /// rien ne change.
    pub fn multiplicateur(&self, attaquant: bool, cible_joueur: bool) -> f32 {
        match (attaquant, cible_joueur) {
            (false, _) => 1.0,
            (true, true) => self.degats_contre_joueurs,
            (true, false) => self.degats_contre_creatures,
        }
    }

    /// Vitesse d'un projectile lancé à `vitesse_propre` par un tireur allant à
    /// `vitesse_tireur` (X.13).
    pub fn vitesse_projectile(
        &self,
        vitesse_propre: Vec3<f32>,
        vitesse_tireur: Vec3<f32>,
    ) -> Vec3<f32> {
        vitesse_propre + vitesse_tireur * self.part_vitesse_tireur
    }

    /// Vitesse d'un projectile selon `assets/icebreaker/combat.ron` (comme
    /// Veloren si le fichier manque).
    pub fn vitesse_projectile_actuelle(
        vitesse_propre: Vec3<f32>,
        vitesse_tireur: Vec3<f32>,
    ) -> Vec3<f32> {
        Ron::<Self>::load("icebreaker.combat").map_or(vitesse_propre + vitesse_tireur, |c| {
            c.read()
                .0
                .vitesse_projectile(vitesse_propre, vitesse_tireur)
        })
    }

    /// Multiplicateur lu dans `assets/icebreaker/combat.ron` (1 si le fichier
    /// manque).
    pub fn multiplicateur_actuel(attaquant: bool, cible_joueur: bool) -> f32 {
        Ron::<Self>::load("icebreaker.combat")
            .map_or(1.0, |c| c.read().0.multiplicateur(attaquant, cible_joueur))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplicateur_selon_la_cible() {
        let c = IcebreakerCombat {
            degats_contre_creatures: 5.0,
            degats_contre_joueurs: 9.0,
            part_vitesse_tireur: 0.0,
        };
        assert_eq!(c.multiplicateur(true, false), 5.0);
        assert_eq!(c.multiplicateur(true, true), 9.0);
        assert_eq!(c.multiplicateur(false, true), 1.0);
    }

    #[test]
    fn le_tir_ne_derive_pas_avec_le_tireur() {
        let mut c = IcebreakerCombat {
            degats_contre_creatures: 1.0,
            degats_contre_joueurs: 1.0,
            part_vitesse_tireur: 0.0,
        };
        // Flèche vers +y, tireur qui se déplace de côté (+x) : elle part droit.
        let fleche = Vec3::new(0.0, 50.0, 0.0);
        let de_cote = Vec3::new(6.0, 0.0, 0.0);
        assert_eq!(c.vitesse_projectile(fleche, de_cote), fleche);
        // Comme Veloren : la vitesse du tireur s'ajoute.
        c.part_vitesse_tireur = 1.0;
        assert_eq!(
            c.vitesse_projectile(fleche, de_cote),
            Vec3::new(6.0, 50.0, 0.0)
        );
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let c = Ron::<IcebreakerCombat>::load_expect("icebreaker.combat")
            .read()
            .0
            .clone();
        // Épée de départ (5–7 × 0,6) contre un gnarling (50 PV, ~10 % d'armure) : ~3
        // coups.
        let coups_gnarling = 50.0 / (6.0 * 0.6 * 0.9 * c.degats_contre_creatures);
        assert!((2.5..=4.0).contains(&coups_gnarling), "{coups_gnarling}");
        // Coup de gnarling (2,5 à 4) contre un joueur de 100 PV : 3 à 4 coups en
        // moyenne.
        let coups_joueur = 100.0 / (3.25 * c.degats_contre_joueurs);
        assert!((3.0..=4.5).contains(&coups_joueur), "{coups_joueur}");
    }
}
