//! Les réglages d'un ennemi : ceux de sa race posés sur ceux qui sont communs à toutes
//! (`assets/icebreaker/ia/profils.ron`). Tout est facultatif : une race ne dit que ce qui la distingue.

use serde::Deserialize;
use std::collections::HashMap;

/// Les réglages d'un ennemi, une fois sa race posée sur le commun.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Profil {
    // --- Vue (IA.2) ---
    /// Jusqu'où il voit devant lui (blocs) ; au-delà de `vue_nette`, il ne fait que soupçonner.
    pub vue: Option<f32>,
    /// Jusqu'où il reconnaît tout de suite ce qu'il voit.
    pub vue_nette: Option<f32>,
    /// Largeur de son cône de vue, en degrés.
    pub cone: Option<f32>,
    /// Rayon dans lequel il perçoit aussi dans son dos.
    pub dos: Option<f32>,
    /// Secondes pendant lesquelles il doit garder un soupçon en vue pour le repérer.
    pub soupcon: Option<f64>,
    // --- Chercher (IA.2) ---
    /// Secondes pendant lesquelles il court vers la dernière position connue d'une cible perdue de vue.
    pub poursuite: Option<f64>,
    /// Secondes qu'il passe à fouiller du regard l'endroit où il est venu voir.
    pub fouille: Option<f64>,
    /// Secondes au plus pour rejoindre l'endroit à fouiller ; au-delà, il renonce.
    pub trajet: Option<f64>,
    /// Distance (blocs) à laquelle un endroit est atteint.
    pub arrivee: Option<f32>,
    /// Blocs au plus entre lui et son poste pendant un combat ; au-delà, il lâche sa cible et rentre.
    pub laisse: Option<f32>,
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
    // --- Fuite ---
    /// Part de ses PV (0 à 1) sous laquelle il fuit ; rien = ne fuit pas.
    pub fuite: Option<f32>,
}

impl Profil {
    /// Ces réglages posés sur `base` : ce qui est dit ici l'emporte, le reste vient de `base`.
    pub fn sur(&self, base: &Profil) -> Profil {
        Profil {
            vue: self.vue.or(base.vue),
            vue_nette: self.vue_nette.or(base.vue_nette),
            cone: self.cone.or(base.cone),
            dos: self.dos.or(base.dos),
            soupcon: self.soupcon.or(base.soupcon),
            poursuite: self.poursuite.or(base.poursuite),
            fouille: self.fouille.or(base.fouille),
            trajet: self.trajet.or(base.trajet),
            arrivee: self.arrivee.or(base.arrivee),
            laisse: self.laisse.or(base.laisse),
            acheve: self.acheve.or(base.acheve),
            meute: self.meute.or(base.meute),
            meute_rayon: self.meute_rayon.or(base.meute_rayon),
            provoque_par_coup: self.provoque_par_coup.or(base.provoque_par_coup),
            reciblage: self.reciblage.or(base.reciblage),
            repartition: self.repartition.or(base.repartition),
            fuite: self.fuite.or(base.fuite),
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

    /// Chaque réglage se pose sur le commun : un champ oublié dans `sur` ferait perdre le réglage
    /// d'une race sans que rien ne le dise.
    #[test]
    fn aucun_champ_n_est_oublie_par_la_fusion() {
        let tout = Profil {
            vue: Some(1.0),
            vue_nette: Some(1.0),
            cone: Some(1.0),
            dos: Some(1.0),
            soupcon: Some(1.0),
            poursuite: Some(1.0),
            fouille: Some(1.0),
            trajet: Some(1.0),
            arrivee: Some(1.0),
            laisse: Some(1.0),
            acheve: Some(1.0),
            meute: Some(1),
            meute_rayon: Some(1.0),
            provoque_par_coup: Some(1.0),
            reciblage: Some(1.0),
            repartition: Some(1.0),
            fuite: Some(1.0),
        };
        assert_eq!(tout.sur(&Profil::default()), tout);
        assert_eq!(Profil::default().sur(&tout), tout);
    }
}
