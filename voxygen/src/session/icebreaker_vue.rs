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

/// Le rectangle à l'écran (-1 à 1, y vers le haut) de l'emprise `bounds` vue
/// à la hauteur `z` par la matrice vue-projection de la caméra ; rien si un
/// coin passe derrière la caméra.
pub fn rect_ecran(vue_proj: &Mat4<f32>, bounds: Aabr<f32>, z: f32) -> Option<Aabr<f32>> {
    let coins = [
        Vec2::new(bounds.min.x, bounds.min.y),
        Vec2::new(bounds.max.x, bounds.min.y),
        Vec2::new(bounds.min.x, bounds.max.y),
        Vec2::new(bounds.max.x, bounds.max.y),
    ];
    let mut rect: Option<Aabr<f32>> = None;
    for coin in coins {
        let p = *vue_proj * Vec4::new(coin.x, coin.y, z, 1.0);
        if p.w <= 0.0 {
            return None;
        }
        let p = p.xy() / p.w;
        rect = Some(rect.map_or(Aabr { min: p, max: p }, |r| Aabr {
            min: r.min.map2(p, f32::min),
            max: r.max.map2(p, f32::max),
        }));
    }
    rect
}

#[cfg(test)]
mod tests_rect {
    use super::*;

    /// Une caméra 100 blocs au-dessus de l'origine qui regarde vers le bas, le
    /// haut de l'écran vers +y.
    fn camera() -> Mat4<f32> {
        let proj = Mat4::perspective_rh_zo(1.0, 2.0, 0.1, 1000.0);
        let vue = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 100.0),
            Vec3::zero(),
            Vec3::unit_y(),
        );
        proj * vue
    }

    #[test]
    fn un_batiment_sous_la_camera_est_au_milieu_et_plus_grand_s_il_est_plus_haut() {
        let bounds = Aabr {
            min: Vec2::new(-10.0, -10.0),
            max: Vec2::new(10.0, 10.0),
        };
        let bas = rect_ecran(&camera(), bounds, 0.0).unwrap();
        assert!((bas.min + bas.max).magnitude() < 1e-4, "{bas:?}");
        assert!(bas.max.x > 0.0 && bas.max.y > 0.0);
        // Deux fois plus large que haut : l'écran l'est.
        assert!((bas.max.y - 2.0 * bas.max.x).abs() < 1e-4, "{bas:?}");
        let haut = rect_ecran(&camera(), bounds, 50.0).unwrap();
        assert!(haut.max.y > bas.max.y * 1.9, "{haut:?} {bas:?}");
        // Un bâtiment décalé vers +x et plus haut glisse encore plus vers la droite.
        let decale = Aabr {
            min: Vec2::new(20.0, -10.0),
            max: Vec2::new(40.0, 10.0),
        };
        let d_bas = rect_ecran(&camera(), decale, 0.0).unwrap();
        let d_haut = rect_ecran(&camera(), decale, 50.0).unwrap();
        assert!(d_haut.min.x > d_bas.min.x && d_bas.min.x > 0.0);
        // Derrière la caméra : rien.
        assert_eq!(rect_ecran(&camera(), bounds, 150.0), None);
    }
}
