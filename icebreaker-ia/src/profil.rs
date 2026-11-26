//! Les réglages d'un ennemi : ceux de sa race posés sur ceux qui sont communs à toutes
//! (`assets/icebreaker/ia/profils.ron`). Tout est facultatif : une race ne dit que ce qui la distingue.

use serde::Deserialize;
use std::collections::HashMap;

/// Les réglages d'un ennemi, une fois sa race posée sur le commun.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Profil {
    // --- Choix de cible (Y.2, Y.9) ---
    /// Achever : un joueur à terre à moins de ce rayon (blocs) passe en tête des cibles.
    pub acheve: Option<f32>,
    /// Il n'achève qu'à plusieurs : ce nombre de membres de la race (lui compris) à moins de
    /// `meute_rayon` blocs ; seul, il vise d'abord un joueur debout. Rien = il achève toujours.
    pub meute: Option<usize>,
    pub meute_rayon: Option<f32>,
    /// Contre de l'achèvement : celui qui vient de le frapper (depuis moins de ces secondes) passe
    /// avant un joueur à terre.
    pub provoque_par_coup: Option<f64>,
    /// Secondes entre deux choix de cible pendant un combat, ± 25 % selon la créature.
    pub reciblage: Option<f64>,
    /// Blocs « ajoutés » à la distance d'une cible par allié qui la vise déjà (0 = le plus proche).
    pub repartition: Option<f32>,
    // --- États ---
    /// Part de ses PV (0 à 1) sous laquelle il fuit ; rien = ne fuit pas.
    pub fuite: Option<f32>,
    /// Secondes sans percevoir sa cible après lesquelles il l'abandonne.
    pub oubli: Option<f64>,
}

impl Profil {
    /// Ces réglages posés sur `base` : ce qui est dit ici l'emporte, le reste vient de `base`.
    pub fn sur(&self, base: &Profil) -> Profil {
        Profil {
            acheve: self.acheve.or(base.acheve),
            meute: self.meute.or(base.meute),
            meute_rayon: self.meute_rayon.or(base.meute_rayon),
            provoque_par_coup: self.provoque_par_coup.or(base.provoque_par_coup),
            reciblage: self.reciblage.or(base.reciblage),
            repartition: self.repartition.or(base.repartition),
            fuite: self.fuite.or(base.fuite),
            oubli: self.oubli.or(base.oubli),
        }
    }
}

/// Le fichier `profils.ron`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Profils {
    #[serde(default)]
    pub commun: Profil,
    /// Par race : "human", "orc", "dwarf", "elf", "draugr", "danari".
    #[serde(default)]
    pub races: HashMap<String, Profil>,
}

impl Profils {
    /// Le profil d'un ennemi de cette race (une race absente du fichier : le commun).
    pub fn profil(&self, race: &str) -> Profil {
        self.races.get(race).map_or_else(|| self.commun.clone(), |r| r.sur(&self.commun))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_race_l_emporte_sur_le_commun_champ_par_champ() {
        let profils = Profils {
            commun: Profil { reciblage: Some(3.0), repartition: Some(8.0), ..Profil::default() },
            races: HashMap::from([("danari".to_string(), Profil {
                reciblage: Some(2.0),
                fuite: Some(0.3),
                ..Profil::default()
            })]),
        };
        let danari = profils.profil("danari");
        assert_eq!(danari.reciblage, Some(2.0));
        assert_eq!(danari.repartition, Some(8.0));
        assert_eq!(danari.fuite, Some(0.3));
        // Une race que le fichier ne décrit pas : le commun.
        assert_eq!(profils.profil("orc"), profils.commun);
    }
}
