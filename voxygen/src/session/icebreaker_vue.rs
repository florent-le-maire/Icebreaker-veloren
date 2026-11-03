//! Icebreaker : la vue du village de dessus (touche U, X.1), réglée par
//! `assets/icebreaker/vue_village.ron`. La vue se ferme d'elle-même quand le
//! joueur n'est plus au village : téléporté (départ d'expédition, retour),
//! mort, ou quand le serveur ne lui envoie plus de bâtiments.

use common::assets::{AssetExt, Ron};
use serde::Deserialize;
use vek::*;

#[derive(Clone, Debug, Deserialize)]
pub struct Reglages {
    /// Hauteur de la caméra au-dessus du personnage à l'entrée dans la vue.
    pub hauteur: f32,
    /// Hauteur au-dessus du sol du plan visé (à mi-hauteur des maisons).
    pub plan: f32,
    /// Distance de l'entrée au-delà de laquelle la vue se ferme d'elle-même.
    pub rayon_sortie: f32,
}

impl Reglages {
    pub fn charger() -> Self {
        Ron::<Self>::load_expect("icebreaker.vue_village")
            .read()
            .0
            .clone()
    }
}

/// Pourquoi la vue se ferme sans la touche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sortie {
    /// Le joueur n'est plus près de là où il est entré dans la vue.
    Eloigne,
    Mort,
    /// Le serveur n'a envoyé aucun bâtiment : pas de village ici.
    HorsVillage,
}

/// Si la vue, entrée en `entree`, doit se fermer : le joueur (à `position`,
/// inconnue pour un fantôme) s'est éloigné de plus de `rayon_sortie`, il est
/// mort, ou la dernière liste de bâtiments reçue était vide.
pub fn sortie(
    entree: Vec3<f32>,
    position: Option<Vec3<f32>>,
    mort: bool,
    hors_village: bool,
    rayon_sortie: f32,
) -> Option<Sortie> {
    if hors_village {
        Some(Sortie::HorsVillage)
    } else if mort {
        Some(Sortie::Mort)
    } else if position.is_some_and(|pos| pos.distance_squared(entree) > rayon_sortie.powi(2)) {
        Some(Sortie::Eloigne)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_vue_reste_tant_que_le_joueur_est_pres_vivant_et_au_village() {
        let entree = Vec3::new(100.0, 100.0, 50.0);
        assert_eq!(sortie(entree, Some(Vec3::new(150.0, 100.0, 60.0)), false, false, 120.0), None);
        // Un fantôme n'a pas de position : seuls la mort et le village comptent.
        assert_eq!(sortie(entree, None, false, false, 120.0), None);
    }

    #[test]
    fn la_vue_se_ferme_quand_le_joueur_est_teleporte_mort_ou_hors_du_village() {
        let entree = Vec3::new(100.0, 100.0, 50.0);
        assert_eq!(
            sortie(entree, Some(Vec3::new(1100.0, 100.0, 50.0)), false, false, 120.0),
            Some(Sortie::Eloigne)
        );
        assert_eq!(sortie(entree, Some(entree), true, false, 120.0), Some(Sortie::Mort));
        assert_eq!(sortie(entree, None, false, true, 120.0), Some(Sortie::HorsVillage));
        // Hors du village l'emporte : c'est le message le plus utile.
        assert_eq!(sortie(entree, Some(entree), true, true, 120.0), Some(Sortie::HorsVillage));
    }
}
