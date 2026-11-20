//! Icebreaker : le portail du retour au village et l'autel de résurrection (jalon T, T.10 et T.12,
//! `assets/icebreaker/portail.ron`, `docs/icebreaker/design/donjon.md`, « Portail et autel »).
//! Deux objets des salles spéciales du donjon. Le portail est éteint : un joueur l'allume par une
//! interaction de quelques secondes ; allumé, qui le traverse rentre au village. L'autel relève les
//! morts dont on porte le cristal (réglages dans `mort.ron`).

use crate::{
    assets::{AssetExt, Ron},
    comp::{Body, Pos, object},
    consts::MAX_INTERACT_RANGE,
    interaction::InteractionKind,
};
use serde::Deserialize;
use vek::Vec3;

#[derive(Clone, Debug, Deserialize)]
pub struct IcebreakerPortail {
    /// Secondes d'interaction pour allumer le portail.
    pub duree_allumage: f32,
    /// On traverse le portail à moins de tant de blocs de son axe…
    pub rayon_traversee: f32,
    /// … et à moins de tant de blocs au-dessus (ou au-dessous) de son pied.
    pub hauteur_traversee: f32,
    /// La lumière du portail allumé : couleur (rouge, vert, bleu, de 0 à 1) et force.
    pub lumiere: (f32, f32, f32),
    pub force_lumiere: f32,
    /// La lueur du portail éteint (couleur et force) : assez pour le voir dans une salle sombre.
    pub lumiere_eteint: (f32, f32, f32),
    pub force_eteint: f32,
    /// La lueur de l'autel : couleur et force.
    pub lumiere_autel: (f32, f32, f32),
    pub force_autel: f32,
}

impl Default for IcebreakerPortail {
    fn default() -> Self {
        Self {
            duree_allumage: 5.0,
            rayon_traversee: 1.5,
            hauteur_traversee: 3.0,
            lumiere: (0.3, 0.6, 1.0),
            force_lumiere: 6.0,
            lumiere_eteint: (0.9, 0.85, 0.8),
            force_eteint: 4.0,
            lumiere_autel: (1.0, 0.75, 0.45),
            force_autel: 3.0,
        }
    }
}

impl IcebreakerPortail {
    /// Réglages lus dans `assets/icebreaker/portail.ron` (valeurs par défaut si le fichier manque).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.portail").map_or_else(|_| Self::default(), |p| p.read().0.clone())
    }

    /// La lumière que donne un de nos objets : couleur (rouge, vert, bleu) et force.
    pub fn lumiere_de(&self, body: &Body) -> Option<((f32, f32, f32), f32)> {
        match *body {
            PORTAIL => Some((self.lumiere_eteint, self.force_eteint)),
            PORTAIL_ACTIF => Some((self.lumiere, self.force_lumiere)),
            AUTEL => Some((self.lumiere_autel, self.force_autel)),
            _ => None,
        }
    }

    /// Un joueur en `pos` traverse le portail allumé posé en `portail`.
    pub fn traverse(&self, portail: Vec3<f32>, pos: Vec3<f32>) -> bool {
        portail.xy().distance_squared(pos.xy()) <= self.rayon_traversee.powi(2)
            && (pos.z - portail.z).abs() <= self.hauteur_traversee
    }
}

pub const PORTAIL: Body = Body::Object(object::Body::IcebreakerPortail);
pub const PORTAIL_ACTIF: Body = Body::Object(object::Body::IcebreakerPortailActif);
pub const AUTEL: Body = Body::Object(object::Body::IcebreakerAutel);

/// L'interaction (de quelques secondes) qu'un objet propose : allumer le portail éteint, prier à
/// l'autel. Le portail allumé ne propose rien : on le traverse.
pub fn rituel_de(body: &Body) -> Option<InteractionKind> {
    match *body {
        PORTAIL => Some(InteractionKind::IcebreakerAllumerPortail),
        AUTEL => Some(InteractionKind::IcebreakerAutel),
        _ => None,
    }
}

/// À portée de l'objet `cible`, qui propose bien ce rituel.
pub fn peut_faire(rituel: InteractionKind, pos: Pos, pos_cible: Pos, cible: Option<&Body>) -> bool {
    pos.0.distance_squared(pos_cible.0) <= MAX_INTERACT_RANGE.powi(2)
        && cible.and_then(rituel_de) == Some(rituel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fichier_de_donnees_lisible() {
        let p = Ron::<IcebreakerPortail>::load_expect("icebreaker.portail").read().0.clone();
        assert!(p.duree_allumage > 0.0 && p.rayon_traversee > 0.0 && p.hauteur_traversee > 0.0);
    }

    #[test]
    fn on_traverse_pres_de_l_axe() {
        let p = IcebreakerPortail::default();
        let portail = Vec3::new(100.0, 50.0, 20.0);
        assert!(p.traverse(portail, Vec3::new(100.5, 50.8, 20.0)));
        // À côté de l'anneau, ou à un autre étage : non.
        assert!(!p.traverse(portail, Vec3::new(102.0, 50.0, 20.0)));
        assert!(!p.traverse(portail, Vec3::new(100.0, 50.0, 27.0)));
    }

    #[test]
    fn chaque_objet_a_son_rituel() {
        assert_eq!(rituel_de(&PORTAIL), Some(InteractionKind::IcebreakerAllumerPortail));
        assert_eq!(rituel_de(&AUTEL), Some(InteractionKind::IcebreakerAutel));
        assert_eq!(rituel_de(&PORTAIL_ACTIF), None);
        let (ici, loin) = (Pos(Vec3::zero()), Pos(Vec3::new(9.0, 0.0, 0.0)));
        let allumer = InteractionKind::IcebreakerAllumerPortail;
        assert!(peut_faire(allumer, ici, Pos(Vec3::new(2.0, 0.0, 0.0)), Some(&PORTAIL)));
        assert!(!peut_faire(allumer, ici, loin, Some(&PORTAIL)));
        assert!(!peut_faire(allumer, ici, ici, Some(&PORTAIL_ACTIF)));
        assert!(!peut_faire(allumer, ici, ici, Some(&AUTEL)));
    }
}
