//! Icebreaker : le comportement de chaque race ennemie (jalon Y, `assets/icebreaker/ennemis/races.ron`).
//! La race d'un ennemi est l'espèce de son corps humanoïde ; seuls les ennemis (alignement `Enemy`)
//! humanoïdes en tiennent compte. Une race absente du fichier se comporte comme dans Veloren.

use crate::{
    assets::{AssetExt, Ron},
    comp::{Body, humanoid},
};
use hashbrown::HashMap;
use serde::Deserialize;

/// Ce qu'une race fait de différent (tout est facultatif : rien = comme Veloren).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Comportement {
    /// Achever : un joueur à terre à moins de ce rayon (blocs) passe en tête des cibles.
    pub acheve: Option<f32>,
    /// Contre de l'achèvement : celui qui vient de frapper l'ennemi (depuis moins de ces secondes)
    /// passe avant un joueur à terre.
    pub provoque_par_coup: f64,
    /// Secondes entre deux choix de cible pendant un combat (10 dans Veloren).
    pub reciblage: Option<f64>,
    /// Part de ses PV (0 à 1) sous laquelle il fuit ; rien = ne fuit pas.
    pub fuite: Option<f32>,
    /// Des groupes font la ronde entre les salles du donjon généré (Y.3).
    pub ronde: Option<Ronde>,
}

/// Les rondes d'une race (Y.3) : des guerriers d'une salle partent ensemble vers les salles voisines,
/// s'arrêtent un moment à chacune, puis reviennent sur leurs pas.
#[derive(Clone, Debug, Deserialize)]
pub struct Ronde {
    /// Nombre de groupes au plus par donjon.
    pub groupes: usize,
    /// Guerriers d'une même salle par groupe, au plus.
    pub membres: usize,
    /// Salles visitées, la sienne comprise.
    pub etapes: usize,
    /// Allure (1 = la course de Veloren).
    pub vitesse: f32,
    /// Secondes d'arrêt dans chaque salle.
    pub pause: f64,
    /// Distance (blocs) à laquelle une salle est atteinte.
    pub arrivee: f32,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IcebreakerRaces {
    /// Par race : "human", "orc", "dwarf", "elf", "draugr", "danari" (les noms de `generation.ron`).
    pub races: HashMap<String, Comportement>,
}

/// Le nom d'une espèce humanoïde dans les fichiers de données.
pub fn nom(species: humanoid::Species) -> &'static str {
    match species {
        humanoid::Species::Danari => "danari",
        humanoid::Species::Dwarf => "dwarf",
        humanoid::Species::Elf => "elf",
        humanoid::Species::Human => "human",
        humanoid::Species::Orc => "orc",
        humanoid::Species::Draugr => "draugr",
    }
}

impl IcebreakerRaces {
    /// Réglages lus dans `assets/icebreaker/ennemis/races.ron` (rien si le fichier manque).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.ennemis.races").map_or_else(|_| Self::default(), |r| r.read().0.clone())
    }

    /// Le comportement d'un corps : celui de sa race s'il est humanoïde et que la race est décrite.
    pub fn de(&self, body: Option<&Body>) -> Option<&Comportement> {
        match body {
            Some(Body::Humanoid(h)) => self.races.get(nom(h.species)),
            _ => None,
        }
    }
}

/// Le comportement de la race d'un corps, lu dans le fichier de données (rien hors humanoïde).
pub fn comportement(body: Option<&Body>) -> Option<Comportement> {
    let body @ Some(Body::Humanoid(_)) = body else {
        return None;
    };
    Ron::<IcebreakerRaces>::load("icebreaker.ennemis.races")
        .ok()
        .and_then(|r| r.read().0.de(body).cloned())
}

impl Comportement {
    /// Rang d'une cible pour cette race (plus petit = choisie d'abord, à égalité la plus proche) :
    /// 0 celui qui vient de le frapper (contre), 1 un joueur à terre à portée d'achèvement, 2 le reste.
    pub fn rang(&self, distance: f32, a_terre: bool, l_a_frappe_depuis: Option<f64>) -> u8 {
        if self.acheve.is_some() && l_a_frappe_depuis.is_some_and(|s| s < self.provoque_par_coup) {
            0
        } else if a_terre && self.acheve.is_some_and(|r| distance < r) {
            1
        } else {
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn danari() -> Comportement {
        Comportement {
            acheve: Some(20.0),
            provoque_par_coup: 3.0,
            ..Comportement::default()
        }
    }

    #[test]
    fn le_danari_achieve_qui_est_a_terre() {
        let d = danari();
        assert!(d.rang(15.0, true, None) < d.rang(2.0, false, None));
        // Trop loin : pas d'achèvement.
        assert_eq!(d.rang(25.0, true, None), 2);
    }

    #[test]
    fn frapper_le_danari_le_detourne() {
        let d = danari();
        assert!(d.rang(10.0, false, Some(1.0)) < d.rang(2.0, true, None));
        // Le coup est trop ancien : il retourne au joueur à terre.
        assert!(d.rang(10.0, false, Some(5.0)) > d.rang(2.0, true, None));
    }

    #[test]
    fn sans_achevement_tout_se_vaut() {
        let c = Comportement::default();
        assert_eq!(c.rang(1.0, true, Some(0.5)), c.rang(9.0, false, None));
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let r = Ron::<IcebreakerRaces>::load_expect("icebreaker.ennemis.races").read().0.clone();
        for race in ["human", "orc", "dwarf", "elf", "draugr", "danari"] {
            assert!(r.races.contains_key(race), "{race} manque dans races.ron");
        }
        let body = Body::Humanoid(humanoid::Body {
            species: humanoid::Species::Danari,
            ..humanoid::Body::random()
        });
        assert!(r.de(Some(&body)).is_some_and(|d| d.acheve.is_some()));
    }
}
