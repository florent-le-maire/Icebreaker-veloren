//! Icebreaker : les rondes de chaque race ennemie (Y.3, `assets/icebreaker/ennemis/races.ron`) et le
//! compte de qui vise qui (Y.9). La race d'un ennemi est l'espèce de son corps humanoïde. Le reste du
//! comportement d'une race (choix de cible, fuite) est dans `icebreaker_ia` depuis le jalon IA.

use crate::{
    assets::{AssetExt, Ron},
    comp::humanoid,
    uid::Uid,
};
use hashbrown::HashMap;
use serde::Deserialize;

/// Ce qu'une race fait de différent hors du cerveau (tout est facultatif).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Comportement {
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
}

/// Y.9 : combien d'ennemis visent déjà chaque cible (par `Uid`), compté à chaque tick par le
/// serveur (`server/src/sys/icebreaker_focus.rs`) : les agents ne voient pas les cibles des autres.
#[derive(Clone, Debug, Default)]
pub struct IcebreakerFocus(pub HashMap<Uid, u8>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fichier_de_donnees_lisible() {
        let r = Ron::<IcebreakerRaces>::load_expect("icebreaker.ennemis.races").read().0.clone();
        for race in ["human", "orc", "dwarf", "elf", "draugr", "danari"] {
            assert!(r.races.contains_key(race), "{race} manque dans races.ron");
        }
        assert!(r.races["human"].ronde.is_some());
    }
}
