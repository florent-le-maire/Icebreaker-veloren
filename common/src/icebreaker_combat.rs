//! Icebreaker : combat létal façon Dark Souls (`assets/icebreaker/combat.ron`).
//! Les dégâts des attaques sont multipliés selon la cible : une créature
//! ordinaire tombe en quelques coups, un joueur aussi. Contre une créature, le
//! multiplicateur dépend du type d'attaque (L.5 : l'épée en 3 coups, une arme à
//! distance en 5 à 6, un rayon par petites touches) et un coup ne retire pas
//! plus d'une part de ses PV max (pas de coup fatal d'un coup sans gagner en
//! puissance). Les projectiles ne dérivent pas avec le tireur (X.13).

use crate::{
    assets::{AssetExt, Ron},
    combat::AttackSource,
};
use serde::Deserialize;
use vek::Vec3;

#[derive(Clone, Debug, Deserialize)]
pub struct IcebreakerCombat {
    /// Multiplicateur des dégâts reçus par une créature (PNJ) : corps à corps,
    /// et tout type d'attaque sans valeur propre ci-dessous.
    pub degats_contre_creatures: f32,
    /// Multiplicateur des dégâts reçus par un joueur.
    pub degats_contre_joueurs: f32,
    /// Contre une créature : flèches, sorts lancés, objets lancés.
    #[serde(default)]
    pub creatures_projectile: Option<f32>,
    /// Contre une créature : rayons (lance-flammes…), à chaque touche.
    #[serde(default)]
    pub creatures_rayon: Option<f32>,
    /// Contre une créature : explosions (boule de feu, flèche explosive…).
    #[serde(default)]
    pub creatures_explosion: Option<f32>,
    /// Contre une créature : ondes de choc.
    #[serde(default)]
    pub creatures_onde: Option<f32>,
    /// Contre une créature : arcs électriques et zones persistantes.
    #[serde(default)]
    pub creatures_autre: Option<f32>,
    /// Part des PV max d'une créature qu'un coup peut retirer au plus, à
    /// multiplier par le bonus de dégâts de l'attaquant (absent : pas de
    /// plafond).
    #[serde(default)]
    pub plafond_coup_creature: Option<f32>,
    /// Part de la vitesse du tireur reprise par ses projectiles (1 : comme
    /// Veloren).
    #[serde(default = "part_vitesse_tireur_defaut")]
    pub part_vitesse_tireur: f32,
}

fn part_vitesse_tireur_defaut() -> f32 { 1.0 }

impl IcebreakerCombat {
    /// Multiplicateur d'une attaque ; sans attaquant (piège, environnement),
    /// rien ne change.
    pub fn multiplicateur(&self, attaquant: bool, cible_joueur: bool, source: AttackSource) -> f32 {
        match (attaquant, cible_joueur) {
            (false, _) => 1.0,
            (true, true) => self.degats_contre_joueurs,
            (true, false) => match source {
                AttackSource::Melee => None,
                AttackSource::Projectile => self.creatures_projectile,
                AttackSource::Beam => self.creatures_rayon,
                AttackSource::Explosion => self.creatures_explosion,
                AttackSource::GroundShockwave
                | AttackSource::AirShockwave
                | AttackSource::UndodgeableShockwave => self.creatures_onde,
                AttackSource::Arc | AttackSource::Pool => self.creatures_autre,
            }
            .unwrap_or(self.degats_contre_creatures),
        }
    }

    /// Dégâts qu'un coup peut faire au plus à une créature de `pv_max` PV,
    /// relevé par le bonus de dégâts de l'attaquant (`modificateur`, 1 au
    /// départ) ; `None` : pas de plafond.
    pub fn plafond(&self, pv_max: f32, modificateur: f32) -> Option<f32> {
        self.plafond_coup_creature
            .map(|part| pv_max * part * modificateur.max(1.0))
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
    pub fn multiplicateur_actuel(attaquant: bool, cible_joueur: bool, source: AttackSource) -> f32 {
        Ron::<Self>::load("icebreaker.combat").map_or(1.0, |c| {
            c.read().0.multiplicateur(attaquant, cible_joueur, source)
        })
    }

    /// Plafond d'un coup lu dans `assets/icebreaker/combat.ron` (aucun si le
    /// fichier manque).
    pub fn plafond_actuel(pv_max: f32, modificateur: f32) -> Option<f32> {
        Ron::<Self>::load("icebreaker.combat")
            .ok()
            .and_then(|c| c.read().0.plafond(pv_max, modificateur))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neutre() -> IcebreakerCombat {
        IcebreakerCombat {
            degats_contre_creatures: 1.0,
            degats_contre_joueurs: 1.0,
            creatures_projectile: None,
            creatures_rayon: None,
            creatures_explosion: None,
            creatures_onde: None,
            creatures_autre: None,
            plafond_coup_creature: None,
            part_vitesse_tireur: 0.0,
        }
    }

    #[test]
    fn multiplicateur_selon_la_cible() {
        let c = IcebreakerCombat {
            degats_contre_creatures: 5.0,
            degats_contre_joueurs: 9.0,
            ..neutre()
        };
        assert_eq!(c.multiplicateur(true, false, AttackSource::Melee), 5.0);
        assert_eq!(c.multiplicateur(true, true, AttackSource::Melee), 9.0);
        assert_eq!(c.multiplicateur(false, true, AttackSource::Melee), 1.0);
        // Sans valeur propre, un tir compte comme le corps à corps.
        assert_eq!(c.multiplicateur(true, false, AttackSource::Projectile), 5.0);
    }

    #[test]
    fn multiplicateur_selon_le_type_d_attaque() {
        let c = IcebreakerCombat {
            degats_contre_creatures: 5.0,
            degats_contre_joueurs: 9.0,
            creatures_projectile: Some(2.0),
            creatures_rayon: Some(1.5),
            creatures_onde: Some(3.0),
            ..neutre()
        };
        assert_eq!(c.multiplicateur(true, false, AttackSource::Projectile), 2.0);
        assert_eq!(c.multiplicateur(true, false, AttackSource::Beam), 1.5);
        assert_eq!(c.multiplicateur(true, false, AttackSource::AirShockwave), 3.0);
        assert_eq!(c.multiplicateur(true, false, AttackSource::Explosion), 5.0);
        // Contre un joueur, le type d'attaque ne change rien.
        assert_eq!(c.multiplicateur(true, true, AttackSource::Projectile), 9.0);
    }

    #[test]
    fn plafond_d_un_coup() {
        let mut c = neutre();
        assert_eq!(c.plafond(50.0, 1.0), None);
        c.plafond_coup_creature = Some(0.4);
        assert_eq!(c.plafond(50.0, 1.0), Some(20.0));
        // Gagner en puissance relève le plafond ; un malus ne le baisse pas.
        assert_eq!(c.plafond(50.0, 1.5), Some(30.0));
        assert_eq!(c.plafond(50.0, 0.5), Some(20.0));
    }

    #[test]
    fn le_tir_ne_derive_pas_avec_le_tireur() {
        let mut c = neutre();
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
        // Repères : arme de départ (puissance 0,6) contre un gnarling (50 PV,
        // ~10 % d'armure), dégâts de base des compétences de Veloren.
        let pv = 50.0;
        let plafond = c.plafond(pv, 1.0).unwrap_or(f32::INFINITY);
        let coup = |base: f32, source| {
            (base * 0.6 * 0.9 * c.multiplicateur(true, false, source)).min(plafond)
        };
        // Épée (5–7) : ~3 coups.
        let epee = pv / coup(6.0, AttackSource::Melee);
        assert!((2.5..=4.0).contains(&epee), "épée {epee}");
        // Arc, tir simple (7) et bâton, boule de feu (13) : 5 à 6 coups.
        let fleche = pv / coup(7.0, AttackSource::Projectile);
        assert!((5.0..=6.0).contains(&fleche), "flèche {fleche}");
        let boule = pv / coup(13.0, AttackSource::Explosion);
        assert!((5.0..=6.0).contains(&boule), "boule de feu {boule}");
        // Lance-flammes (5 par touche, 3 touches/s) : moins qu'une flèche par
        // touche, au moins 8 touches.
        let touche = coup(5.0, AttackSource::Beam);
        assert!(touche < coup(7.0, AttackSource::Projectile));
        assert!(pv / touche >= 8.0, "lance-flammes {}", pv / touche);
        // Pas de plafond par coup (retiré le 2026-11-15) : un tir chargé (25)
        // retire plus de la moitié des PV.
        assert!(coup(25.0, AttackSource::Projectile) > pv * 0.5);
        // Coup de gnarling (2,5 à 4) contre un joueur de 100 PV : 3 à 4 coups en
        // moyenne.
        let coups_joueur = 100.0 / (3.25 * c.degats_contre_joueurs);
        assert!((3.0..=4.5).contains(&coups_joueur), "{coups_joueur}");
    }
}
