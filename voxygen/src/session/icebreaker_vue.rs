//! Icebreaker : le plan du village (touche U, X.1, X.11), réglé par
//! `assets/icebreaker/vue_village.ron`. Le HUD dessine un panneau centré avec
//! une case par bâtiment, placée d'après son emprise réelle et mise à l'échelle
//! pour tenir dans le panneau ; la caméra ne bouge pas. Le plan se ferme de
//! lui-même quand le joueur n'est plus au village : téléporté (départ
//! d'expédition, retour), mort, ou quand le serveur ne lui envoie plus de
//! bâtiments.

use common::{
    assets::{AssetExt, Ron},
    comp,
};
use serde::Deserialize;
use std::collections::HashMap;
use vek::*;

#[derive(Clone, Debug, Deserialize)]
pub struct Reglages {
    /// Largeur et hauteur du panneau, en fraction de l'écran (0 à 1).
    pub taille: (f32, f32),
    /// Vide gardé entre le bord du panneau et le plan, en fraction de la
    /// hauteur de l'écran.
    pub marge: f32,
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

/// Ce que le HUD dessine : le panneau (-1 à 1, y vers le haut), chaque
/// bâtiment (nom, rectangle, sous le curseur, apparence), le point du joueur
/// (rien pour un fantôme) et sa couleur.
pub struct Plan {
    pub panneau: Aabr<f32>,
    pub batiments: Vec<(comp::Content, Aabr<f32>, bool, Apparence)>,
    pub joueur: Option<Vec2<f32>>,
    pub couleur_joueur: (f32, f32, f32),
}

impl Default for Plan {
    fn default() -> Self {
        Self {
            panneau: Aabr {
                min: Vec2::zero(),
                max: Vec2::zero(),
            },
            batiments: Vec::new(),
            joueur: None,
            couleur_joueur: (1.0, 1.0, 1.0),
        }
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

/// Le cadre du plan à l'écran (-1 à 1, y vers le haut) : le panneau de
/// `taille` (largeur, hauteur en fraction de l'écran), centré, moins `marge`
/// (fraction de la hauteur de l'écran, la même en pixels sur les quatre côtés ;
/// `aspect` = largeur / hauteur de l'écran).
pub fn cadre(taille: (f32, f32), marge: f32, aspect: f32) -> Aabr<f32> {
    let demi = Vec2::new(taille.0 - marge / aspect, taille.1 - marge);
    Aabr {
        min: -demi,
        max: demi,
    }
}

/// Le plan du village : les emprises (en blocs) ramenées dans `cadre`, à la
/// même échelle en x et en y (un carré reste carré à l'écran, d'où `aspect`),
/// l'ensemble centré dans le cadre ; et le joueur (`None` pour un fantôme)
/// placé de la même façon. Les rectangles sont dans l'ordre des emprises.
pub fn plan(
    emprises: &[Aabr<i32>],
    joueur: Option<Vec2<f32>>,
    cadre: Aabr<f32>,
    aspect: f32,
) -> (Vec<Aabr<f32>>, Option<Vec2<f32>>) {
    let Some(monde) = emprises
        .iter()
        .map(|e| Aabr {
            min: e.min.as_::<f32>(),
            max: e.max.as_::<f32>(),
        })
        .reduce(|a, b| a.union(b))
    else {
        return (Vec::new(), None);
    };
    let taille_monde = monde.size().map(|d| d.max(1.0));
    let taille_cadre = cadre.size();
    let k = (taille_cadre.w * aspect / taille_monde.w).min(taille_cadre.h / taille_monde.h);
    let centre_monde = monde.center();
    let centre_cadre = cadre.center();
    let placer = |p: Vec2<f32>| {
        centre_cadre
            + Vec2::new(
                (p.x - centre_monde.x) * k / aspect,
                (p.y - centre_monde.y) * k,
            )
    };
    let rects = emprises
        .iter()
        .map(|e| Aabr {
            min: placer(e.min.as_::<f32>()),
            max: placer(e.max.as_::<f32>()),
        })
        .collect();
    (rects, joueur.map(placer))
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

#[cfg(test)]
mod tests_plan {
    use super::*;

    fn carre(x: i32, y: i32, cote: i32) -> Aabr<i32> {
        Aabr {
            min: Vec2::new(x, y),
            max: Vec2::new(x + cote, y + cote),
        }
    }

    fn proche(a: Vec2<f32>, b: Vec2<f32>) -> bool { (a - b).magnitude() < 1e-4 }

    fn cadre_carre() -> Aabr<f32> {
        Aabr {
            min: Vec2::new(-0.8, -0.8),
            max: Vec2::new(0.8, 0.8),
        }
    }

    #[test]
    fn un_batiment_seul_remplit_le_cadre_en_gardant_ses_proportions() {
        let emprise = Aabr {
            min: Vec2::new(10, 10),
            max: Vec2::new(30, 20),
        };
        let (rects, _) = plan(&[emprise], None, cadre_carre(), 1.0);
        assert_eq!(rects.len(), 1);
        let r = rects[0];
        assert!(proche(r.center(), Vec2::zero()), "{r:?}");
        assert!((r.size().w - 1.6).abs() < 1e-4, "{r:?}");
        assert!((r.size().h - 0.8).abs() < 1e-4, "{r:?}");
    }

    #[test]
    fn deux_batiments_gardent_leurs_places_et_la_meme_echelle() {
        let (rects, _) = plan(
            &[carre(0, 0, 10), carre(90, 0, 10)],
            None,
            cadre_carre(),
            1.0,
        );
        let (a, b) = (rects[0], rects[1]);
        assert!(proche(a.size().into(), b.size().into()), "{a:?} {b:?}");
        assert!((a.size().w - a.size().h).abs() < 1e-4, "{a:?}");
        assert!(a.max.x < b.min.x, "{a:?} {b:?}");
        // L'ensemble est centré dans le cadre et le remplit en largeur.
        assert!(proche((a.min + b.max) / 2.0, Vec2::zero()), "{a:?} {b:?}");
        assert!(
            (a.min.x + 0.8).abs() < 1e-4 && (b.max.x - 0.8).abs() < 1e-4,
            "{a:?} {b:?}"
        );
    }

    #[test]
    fn l_echelle_est_uniforme_sur_un_ecran_large() {
        let c = cadre((0.5, 0.8), 0.1, 2.0);
        assert!(
            proche(c.min, Vec2::new(-0.45, -0.7)) && proche(c.max, Vec2::new(0.45, 0.7)),
            "{c:?}"
        );
        let (rects, _) = plan(&[carre(0, 0, 10)], None, c, 2.0);
        let r = rects[0];
        // Un carré en blocs reste carré en pixels : moitié moins large en NDC.
        assert!((r.size().w * 2.0 - r.size().h).abs() < 1e-4, "{r:?}");
        assert!((r.size().h - 1.4).abs() < 1e-4, "{r:?}");
    }

    #[test]
    fn le_joueur_est_place_comme_les_batiments_et_un_fantome_n_a_pas_de_point() {
        let emprises = [carre(0, 0, 10), carre(90, 30, 10)];
        let (rects, joueur) = plan(&emprises, Some(Vec2::new(50.0, 20.0)), cadre_carre(), 1.0);
        assert!(proche(joueur.unwrap(), Vec2::zero()), "{joueur:?}");
        let (_, coin) = plan(&emprises, Some(Vec2::zero()), cadre_carre(), 1.0);
        assert!(
            proche(coin.unwrap(), rects[0].min),
            "{coin:?} {:?}",
            rects[0]
        );
        assert_eq!(plan(&emprises, None, cadre_carre(), 1.0).1, None);
        let (vide, personne) = plan(&[], Some(Vec2::zero()), cadre_carre(), 1.0);
        assert!(vide.is_empty());
        assert_eq!(personne, None);
    }
}
