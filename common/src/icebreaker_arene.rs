//! Icebreaker : le lieu d'entraînement (jalon O, `assets/icebreaker/arene.ron`,
//! `docs/icebreaker/PLAN-ENTRAINEMENT.md`). Un plateau nivelé dans un monde sans site : l'arène est
//! un disque au milieu, l'entrée est sur le plateau, hors du disque. Tout y est libre : classe,
//! armes, compétences, ennemis et alliés à la demande.

use crate::assets::{AssetExt, Ron};
use serde::Deserialize;
use vek::Vec2;

#[derive(Clone, Debug, Deserialize)]
pub struct AreneConfig {
    /// Rayon de l'arène (en blocs) : le disque où viennent les ennemis.
    pub rayon: f32,
    /// Rayon du plateau nivelé ; les joueurs ne le quittent pas.
    pub rayon_terrain: f32,
    /// Jusqu'où chercher le plateau depuis le centre de la carte (en tronçons).
    pub recherche: i32,
    /// Blocs d'air au-dessus du plateau (arbres et reliefs coupés).
    pub hauteur_libre: i32,
    /// Blocs de terre posés sous le plateau, là où le terrain est plus bas.
    pub profondeur: i32,
    /// Couleur du sol du plateau, et de l'anneau qui marque le bord de l'arène.
    pub couleur_sol: (u8, u8, u8),
    pub couleur_bord: (u8, u8, u8),
    /// L'entrée (apparition et réapparition), depuis le centre de l'arène (x vers l'est, y vers
    /// le nord).
    pub entree: (f32, f32),
    /// La classe d'un personnage qui arrive sans arme d'une de nos classes en mains.
    pub classe_par_defaut: String,
    /// Le niveau de toutes les compétences à l'arrivée et après un changement de classe.
    pub niveau: u8,
    /// Où les ennemis viennent : sur un cercle de ce rayon autour du centre de l'arène.
    pub rayon_apparition: f32,
    /// Combien d'ennemis ou d'alliés une commande fait venir au plus, et combien l'arène en porte.
    pub max_par_appel: u32,
    pub max_en_piste: u32,
    /// Les races d'ennemis : (espèce de Veloren, nom affiché).
    pub races: Vec<(String, String)>,
    /// Les rôles d'ennemis : (id, nom affiché) ; le fichier est `icebreaker.ennemis.<id>`.
    pub roles: Vec<(String, String)>,
    /// Les alliés : (id, nom affiché, fichier d'entité de Veloren).
    pub allies: Vec<(String, String, String)>,
    /// Le premier mannequin de la rangée, depuis le centre de l'arène.
    pub mannequins: (f32, f32),
    /// Secondes avant qu'un joueur mort réapparaisse à l'entrée.
    pub delai_reapparition: f64,
}

impl AreneConfig {
    pub fn entree(&self) -> Vec2<f32> { Vec2::from(self.entree) }
}

pub fn config() -> AreneConfig { Ron::<AreneConfig>::load_expect("icebreaker.arene").read().0.clone() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fichier_de_donnees_lisible() {
        let c = config();
        assert!(c.rayon > 5.0 && c.rayon_terrain > c.rayon);
        assert!(c.recherche > 0 && c.hauteur_libre > 0 && c.profondeur > 0);
        assert!(c.max_par_appel > 0 && c.max_en_piste >= c.max_par_appel);
        assert!(!c.races.is_empty() && !c.roles.is_empty() && !c.allies.is_empty());
        assert!(c.rayon_apparition < c.rayon && c.delai_reapparition >= 0.0);
        let mannequins = Vec2::<f32>::from(c.mannequins).magnitude();
        assert!(mannequins > c.rayon + 2.0 && mannequins < c.rayon_terrain - 12.0);
    }

    #[test]
    fn entree_sur_le_plateau_hors_de_l_arene() {
        let c = config();
        let d = c.entree().magnitude();
        assert!(d > c.rayon + 2.0, "l'entrée est dans l'arène");
        assert!(d < c.rayon_terrain - 3.0, "l'entrée est hors du plateau");
    }
}
