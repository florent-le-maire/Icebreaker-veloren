//! Icebreaker : verrouillage d'une cible, façon Dark Souls
//! (`assets/icebreaker/verrouillage.ron`). Une touche verrouille l'ennemi le plus
//! proche du centre de l'écran ; la caméra le suit, la molette passe à l'ennemi
//! voisin, un cercle le marque à l'écran (HUD).

use common::assets::{AssetExt, Ron};
use serde::Deserialize;
use std::f32::consts::PI;
use vek::*;

#[derive(Clone, Debug, Deserialize)]
pub struct Reglages {
    /// Distance (blocs) jusqu'à laquelle on peut verrouiller un ennemi.
    pub rayon: f32,
    /// Distance (blocs) au-delà de laquelle le verrouillage saute.
    pub rayon_perte: f32,
    /// Inclinaison de la caméra vers le bas (radians), pour garder le personnage en vue.
    pub inclinaison: f32,
    /// Vitesse à laquelle la caméra se tourne vers la cible.
    pub vitesse_camera: f32,
    /// Temps minimum (s) entre deux changements de cible à la molette.
    pub delai_molette: f32,
    /// Où le cercle se pose sur le corps de la cible : 0 aux pieds, 1 au sommet de la tête.
    #[serde(default = "ancrage_defaut")]
    pub ancrage: f32,
    /// Rayon du cercle à l'écran (pixels) ; le halo et les traits en découlent.
    #[serde(default = "rayon_anneau_defaut")]
    pub rayon_anneau: f32,
}

fn ancrage_defaut() -> f32 { 0.5 }

fn rayon_anneau_defaut() -> f32 { 13.0 }

impl Reglages {
    pub fn charger() -> Self {
        Ron::<Self>::load_expect("icebreaker.verrouillage")
            .read()
            .0
            .clone()
    }
}

/// La cible verrouillée.
pub struct Verrou {
    pub cible: specs::Entity,
    /// Temps écoulé depuis le dernier changement de cible à la molette.
    pub depuis_molette: f32,
}

/// Lacet (yaw) de la caméra vers `v`, comme `Camera::forward` : 0 vers +y, croît vers +x.
fn lacet(v: Vec2<f32>) -> f32 { v.x.atan2(v.y) }

/// Écart d'angle ramené entre -π et π.
fn ecart(a: f32) -> f32 { (a + PI).rem_euclid(2.0 * PI) - PI }

/// L'ennemi à verrouiller : le plus proche du centre de l'écran (direction `avant_xy` de
/// la caméra), les plus proches l'emportant à angle égal, à moins de `rayon` d'`origine`.
pub fn choisir<E: Copy>(
    origine: Vec3<f32>,
    avant_xy: Vec2<f32>,
    candidats: &[(E, Vec3<f32>)],
    rayon: f32,
) -> Option<E> {
    let lacet_avant = lacet(avant_xy);
    candidats
        .iter()
        .filter(|(_, pos)| pos.distance(origine) <= rayon)
        .map(|(e, pos)| {
            let angle = ecart(lacet((pos - origine).xy()) - lacet_avant).abs();
            (*e, angle + 0.5 * pos.distance(origine) / rayon)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, _)| e)
}

/// L'ennemi voisin de la cible `actuelle` (vue depuis `origine`), à droite ou à gauche ;
/// après le dernier d'un côté, on repart du premier de l'autre.
pub fn suivante<E: Copy + PartialEq>(
    origine: Vec3<f32>,
    actuelle: (E, Vec3<f32>),
    candidats: &[(E, Vec3<f32>)],
    rayon: f32,
    droite: bool,
) -> Option<E> {
    let lacet_actuel = lacet((actuelle.1 - origine).xy());
    let ecarts = candidats
        .iter()
        .filter(|(e, pos)| *e != actuelle.0 && pos.distance(origine) <= rayon)
        .map(|(e, pos)| {
            let d = ecart(lacet((pos - origine).xy()) - lacet_actuel);
            (*e, if droite { d } else { -d })
        });
    // Le plus petit écart positif, sinon (on fait le tour) le plus négatif.
    ecarts
        .min_by(|a, b| {
            let cle = |d: f32| if d > 0.0 { d } else { d + 2.0 * PI };
            cle(a.1).total_cmp(&cle(b.1))
        })
        .map(|(e, _)| e)
}

/// Orientation de la caméra (lacet, tangage, roulis) pour regarder `cible` depuis `origine`.
pub fn orientation_vers(origine: Vec3<f32>, cible: Vec3<f32>, inclinaison: f32) -> Vec3<f32> {
    let d = cible - origine;
    let tangage = -(d.z / d.magnitude().max(0.001)).clamp(-1.0, 1.0).asin();
    Vec3::new(lacet(d.xy()), tangage + inclinaison, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const O: Vec3<f32> = Vec3::new(0.0, 0.0, 0.0);

    #[test]
    fn choisit_le_plus_proche_du_centre() {
        // La caméra regarde vers +y : l'ennemi devant l'emporte sur celui à droite, plus proche.
        let candidats = [(1, Vec3::new(5.0, 1.0, 0.0)), (2, Vec3::new(0.5, 10.0, 0.0))];
        assert_eq!(choisir(O, Vec2::unit_y(), &candidats, 20.0), Some(2));
        // Hors de portée : rien.
        assert_eq!(choisir(O, Vec2::unit_y(), &candidats, 3.0), None);
    }

    #[test]
    fn passe_au_voisin_et_fait_le_tour() {
        // Devant (+y), à droite (+x) et à gauche (-x).
        let devant = (1, Vec3::new(0.0, 10.0, 0.0));
        let droite = (2, Vec3::new(10.0, 10.0, 0.0));
        let gauche = (3, Vec3::new(-10.0, 10.0, 0.0));
        let tous = [devant, droite, gauche];
        assert_eq!(suivante(O, devant, &tous, 20.0, true), Some(2));
        assert_eq!(suivante(O, devant, &tous, 20.0, false), Some(3));
        // Au bout à droite, on repart de la gauche.
        assert_eq!(suivante(O, droite, &tous, 20.0, true), Some(3));
        assert_eq!(suivante(O, devant, &[devant], 20.0, true), None);
    }

    #[test]
    fn camera_tournee_vers_la_cible() {
        let ori = orientation_vers(O, Vec3::new(10.0, 0.0, 0.0), 0.0);
        assert!((ori.x - PI / 2.0).abs() < 1e-4 && ori.y.abs() < 1e-4);
        // Cible plus bas : la caméra regarde vers le bas (tangage positif).
        assert!(orientation_vers(O, Vec3::new(0.0, 10.0, -5.0), 0.0).y > 0.0);
        // Le fichier de réglages se lit.
        assert!(Reglages::charger().rayon > 0.0);
    }
}
