//! Icebreaker : la balise lumineuse (jalon J, `assets/icebreaker/balise.ron`).
//! Une touche pose aux pieds du joueur une balise qui éclaire dans la couleur choisie, pour se
//! repérer dans le donjon. Une seule charge : elle revient `delai_recharge` secondes après la pose,
//! ou tout de suite quand on ramasse une balise au sol (E).

use crate::assets::{AssetExt, Ron};
use serde::Deserialize;
use vek::Rgb;

#[derive(Clone, Debug, Deserialize)]
pub struct IcebreakerBalise {
    /// Secondes avant que la charge revienne après une pose.
    pub delai_recharge: f64,
    /// Balises au sol par joueur au plus ; au-delà, la plus ancienne s'éteint (le client n'affiche
    /// que les 20 lumières dynamiques les plus proches).
    pub max_au_sol: usize,
    /// Balises au sol sur tout le serveur au plus, tous joueurs confondus ; au-delà, la plus
    /// ancienne s'éteint.
    #[serde(default = "max_total_par_defaut")]
    pub max_total: usize,
    /// Force de la lumière (comme une lanterne : 1 à 10).
    pub force: f32,
    /// Vacillement de la lumière (0 : fixe).
    pub vacillement: f32,
    /// Couleurs proposées : (nom, (rouge, vert, bleu) de 0 à 255). La première est celle par défaut.
    pub couleurs: Vec<(String, (u8, u8, u8))>,
}

impl Default for IcebreakerBalise {
    fn default() -> Self {
        Self {
            delai_recharge: 10.0,
            max_au_sol: 200,
            max_total: max_total_par_defaut(),
            force: 4.0,
            vacillement: 0.3,
            couleurs: vec![("blanc".to_string(), (255, 255, 255))],
        }
    }
}

fn max_total_par_defaut() -> usize { 200 }

/// L'objet « balise lumineuse » : dans l'inventaire, il permet de poser ; au sol, il éclaire.
pub const BALISE: &str = "common.items.icebreaker.balise_lumineuse";

/// `item` est une balise lumineuse.
pub fn est_balise(item: &crate::comp::Item) -> bool {
    matches!(
        item.item_definition_id(),
        crate::comp::item::ItemDefinitionId::Simple(id) if id == BALISE
    )
}

impl IcebreakerBalise {
    /// Réglages lus dans `assets/icebreaker/balise.ron` (valeurs par défaut si le fichier manque).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.balise")
            .map_or_else(|_| Self::default(), |b| b.read().0.clone())
    }

    /// La couleur nommée `nom` (sans tenir compte des majuscules), en 0 à 1.
    pub fn couleur(&self, nom: &str) -> Option<Rgb<f32>> {
        self.couleurs
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(nom))
            .map(|(_, (r, g, b))| Rgb::new(*r, *g, *b).map(|c| c as f32 / 255.0))
    }

    /// Le nom de la couleur par défaut.
    pub fn couleur_par_defaut(&self) -> &str {
        self.couleurs.first().map_or("blanc", |(nom, _)| nom.as_str())
    }

    /// Secondes avant que la charge revienne, si on a posé à `posee` ; 0 : prête.
    pub fn reste_recharge(&self, posee: f64, maintenant: f64) -> f64 {
        (self.delai_recharge - (maintenant - posee)).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recharge() {
        let b = IcebreakerBalise::default();
        assert_eq!(b.reste_recharge(5.0, 5.0), 10.0);
        assert_eq!(b.reste_recharge(5.0, 12.0), 3.0);
        assert_eq!(b.reste_recharge(5.0, 15.0), 0.0);
        assert_eq!(b.reste_recharge(5.0, 99.0), 0.0);
    }

    #[test]
    fn couleurs() {
        let b = IcebreakerBalise {
            couleurs: vec![
                ("rouge".to_string(), (255, 0, 0)),
                ("bleu".to_string(), (0, 0, 255)),
            ],
            ..Default::default()
        };
        assert_eq!(b.couleur("Rouge"), Some(Rgb::new(1.0, 0.0, 0.0)));
        assert_eq!(b.couleur("vert"), None);
        assert_eq!(b.couleur_par_defaut(), "rouge");
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let b = Ron::<IcebreakerBalise>::load_expect("icebreaker.balise").read().0.clone();
        assert!(b.delai_recharge > 0.0);
        assert!(b.max_au_sol >= 1);
        assert!(b.max_total >= 1);
        assert!(b.force > 0.0);
        assert!(!b.couleurs.is_empty());
        for (nom, _) in &b.couleurs {
            assert!(!nom.contains(' '), "un nom de couleur sans espace : {nom}");
        }
    }
}
