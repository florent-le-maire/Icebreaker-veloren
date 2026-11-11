//! Icebreaker : le plan du village (touche U, X.1, X.11), réglé par
//! `assets/icebreaker/vue_village.ron`. Le HUD (`hud/icebreaker.rs`, jalon N)
//! dessine la fenêtre du village : une case par bâtiment, placée d'après son
//! emprise réelle ; la caméra ne bouge pas. Ici : les réglages, et quand le plan
//! se ferme de lui-même : le joueur n'est plus au village (téléporté par un
//! départ d'expédition ou un retour), il est mort, ou le serveur ne lui envoie
//! plus de bâtiments.

use common::assets::{AssetExt, Ron};
use serde::Deserialize;
use std::collections::HashMap;
use vek::*;

#[derive(Clone, Debug, Deserialize)]
pub struct Reglages {
    /// Couleur (rouge, vert, bleu de 0 à 1) du point du joueur.
    pub couleur_joueur: (f32, f32, f32),
    /// Distance de l'entrée au-delà de laquelle le plan se ferme de lui-même.
    pub rayon_sortie: f32,
    /// L'apparence de chaque sorte de bâtiment (X.4), par id (`Building::id`,
    /// « terrain », « chantier »).
    #[serde(default)]
    pub apparences: HashMap<String, Apparence>,
}

/// Comment un bâtiment se dessine sur le plan : couleur du cadre et du nom,
/// icône du HUD (nom connu de `hud::icebreaker_icone`, ou rien).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Apparence {
    pub couleur: (f32, f32, f32),
    #[serde(default)]
    pub icone: String,
}

impl Default for Apparence {
    fn default() -> Self {
        Self {
            couleur: (1.0, 1.0, 1.0),
            icone: String::new(),
        }
    }
}

impl Reglages {
    pub fn charger() -> Self {
        Ron::<Self>::load_expect("icebreaker.vue_village")
            .read()
            .0
            .clone()
    }

    /// L'apparence d'un bâtiment de sorte `kind` (blanche, sans icône, si
    /// elle n'est pas réglée).
    pub fn apparence(&self, kind: &str) -> Apparence {
        self.apparences.get(kind).cloned().unwrap_or_default()
    }
}

/// Pourquoi le plan se ferme sans la touche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sortie {
    /// Le joueur n'est plus près de là où il a ouvert le plan.
    Eloigne,
    Mort,
    /// Le serveur n'a envoyé aucun bâtiment : pas de village ici.
    HorsVillage,
}

/// Si le plan, ouvert en `entree`, doit se fermer : le joueur (à `position`,
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
        assert_eq!(
            sortie(
                entree,
                Some(Vec3::new(150.0, 100.0, 60.0)),
                false,
                false,
                120.0
            ),
            None
        );
        // Un fantôme n'a pas de position : seuls la mort et le village comptent.
        assert_eq!(sortie(entree, None, false, false, 120.0), None);
    }

    #[test]
    fn la_vue_se_ferme_quand_le_joueur_est_teleporte_mort_ou_hors_du_village() {
        let entree = Vec3::new(100.0, 100.0, 50.0);
        assert_eq!(
            sortie(
                entree,
                Some(Vec3::new(1100.0, 100.0, 50.0)),
                false,
                false,
                120.0
            ),
            Some(Sortie::Eloigne)
        );
        assert_eq!(
            sortie(entree, Some(entree), true, false, 120.0),
            Some(Sortie::Mort)
        );
        assert_eq!(
            sortie(entree, None, false, true, 120.0),
            Some(Sortie::HorsVillage)
        );
        // Hors du village l'emporte : c'est le message le plus utile.
        assert_eq!(
            sortie(entree, Some(entree), true, true, 120.0),
            Some(Sortie::HorsVillage)
        );
    }
}

