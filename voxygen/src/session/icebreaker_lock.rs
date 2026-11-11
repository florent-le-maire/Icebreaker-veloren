//! Icebreaker : verrouillage d'une cible, façon Dark Souls
//! (`assets/icebreaker/verrouillage.ron`). Une touche verrouille l'ennemi le plus
//! proche du centre de l'écran ; la caméra le suit, la molette passe à l'ennemi
//! voisin, un cercle le marque à l'écran (HUD) et sert de viseur :
//! les attaques partent vers lui (X.12).

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
    /// Combien de secondes le rappel des touches reste affiché après un verrouillage (N.7).
    #[serde(default = "duree_rappel_defaut")]
    pub duree_rappel: f32,
    /// Combien de secondes « Cible perdue » reste affiché (N.7).
    #[serde(default = "duree_perdue_defaut")]
    pub duree_perdue: f32,
}

fn duree_rappel_defaut() -> f32 { 3.0 }

fn duree_perdue_defaut() -> f32 { 1.5 }

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

/// Ce que le HUD dit du verrouillage, dans la zone « action en cours » (N.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avis {
    Rien,
    /// Le rappel des touches (déverrouiller, cible suivante).
    Rappel,
    /// Le verrouillage a sauté tout seul (cible trop loin ou morte).
    Perdue,
}

/// Les minuteurs des avis : le rappel part au verrouillage et à chaque changement de cible,
/// « Cible perdue » quand le verrouillage saute seul. Le dernier événement l'emporte.
#[derive(Clone, Copy, Debug, Default)]
pub struct Avertissements {
    rappel: f32,
    perdue: f32,
}

impl Avertissements {
    /// Verrouillage ou changement de cible : le rappel repart pour `duree` secondes.
    pub fn rappeler(&mut self, duree: f32) {
        self.rappel = duree;
        self.perdue = 0.0;
    }

    /// Le verrouillage a sauté tout seul : « Cible perdue » pour `duree` secondes.
    pub fn perdre(&mut self, duree: f32) {
        self.perdue = duree;
        self.rappel = 0.0;
    }

    /// Le joueur déverrouille lui-même : plus d'avis.
    pub fn effacer(&mut self) { *self = Self::default(); }

    pub fn avancer(&mut self, dt: f32) {
        self.rappel = (self.rappel - dt).max(0.0);
        self.perdue = (self.perdue - dt).max(0.0);
    }

    /// `verrouille` : une cible est verrouillée en ce moment (le rappel n'a de sens que là).
    pub fn avis(&self, verrouille: bool) -> Avis {
        if self.perdue > 0.0 && !verrouille {
            Avis::Perdue
        } else if self.rappel > 0.0 && verrouille {
            Avis::Rappel
        } else {
            Avis::Rien
        }
    }
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

/// Le point visé sur la cible, là où le HUD pose le cercle : à `ancrage` de sa `hauteur`
/// au-dessus de ses `pieds`. Les attaques partent vers ce point tant qu'elle est verrouillée.
pub fn point_vise(pieds: Vec3<f32>, hauteur: f32, ancrage: f32) -> Vec3<f32> {
    pieds + Vec3::unit_z() * hauteur * ancrage
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
    fn le_rappel_s_affiche_puis_s_efface() {
        let mut a = Avertissements::default();
        assert_eq!(a.avis(true), Avis::Rien);
        a.rappeler(3.0);
        assert_eq!(a.avis(true), Avis::Rappel);
        a.avancer(2.0);
        assert_eq!(a.avis(true), Avis::Rappel);
        a.avancer(1.5);
        assert_eq!(a.avis(true), Avis::Rien);
        // Un changement de cible relance le rappel ; le déverrouillage du joueur l'efface.
        a.rappeler(3.0);
        a.effacer();
        assert_eq!(a.avis(false), Avis::Rien);
    }

    #[test]
    fn cible_perdue_seulement_quand_le_verrou_saute() {
        let mut a = Avertissements::default();
        a.rappeler(3.0);
        a.perdre(1.5);
        // Plus de verrou : « Cible perdue », pas le rappel.
        assert_eq!(a.avis(false), Avis::Perdue);
        a.avancer(1.6);
        assert_eq!(a.avis(false), Avis::Rien);
        // Un nouveau verrouillage chasse « Cible perdue ».
        a.perdre(1.5);
        a.rappeler(3.0);
        assert_eq!(a.avis(true), Avis::Rappel);
        assert_eq!(a.avis(false), Avis::Rien);
    }

    #[test]
    fn vise_le_cercle() {
        // Cible de 2 blocs posée en (3, 4, 1) : cercle au milieu du corps, à 2 de haut.
        let pieds = Vec3::new(3.0, 4.0, 1.0);
        assert_eq!(point_vise(pieds, 2.0, 0.5), Vec3::new(3.0, 4.0, 2.0));
        assert_eq!(point_vise(pieds, 2.0, 0.0), pieds);
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
