//! La table de sorts d'un mage (`design/creatures.md` : « compétences utilisées » ; IA.6) : parmi les
//! sorts qu'il a et peut payer, il tire celui qui convient à la distance et à la situation.

use crate::tirage::tirer;
use serde::Deserialize;

/// Une ligne de la table de sorts (`profils.ron` → `sorts`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Sort {
    /// Le nom du sort : la fin de l'identifiant de la capacité de Veloren ("firebomb" pour
    /// `common.abilities.staff.firebomb`).
    pub sort: String,
    /// Il le lance sur une cible entre `min` et `max` blocs.
    #[serde(default)]
    pub min: f32,
    pub max: f32,
    /// Son poids dans le tirage.
    pub poids: f32,
    /// Secondes avant de le relancer.
    #[serde(default)]
    pub recharge: f64,
    /// Seulement si la cible approche au moins à cette vitesse (blocs par seconde).
    #[serde(default)]
    pub approche: Option<f32>,
}

/// Ce que le mage sait de sa cible au moment de choisir.
#[derive(Clone, Copy, Debug)]
pub struct Situation {
    /// Blocs jusqu'à sa cible.
    pub distance: f32,
    /// Vitesse à laquelle elle se rapproche (négative : elle s'éloigne).
    pub approche: f32,
}

impl Sort {
    /// La cible est-elle à portée de ce sort ?
    pub fn porte(&self, distance: f32) -> bool { (self.min..=self.max).contains(&distance) }
}

/// Le sort qu'il lance : tiré, selon les poids, parmi ceux de la table qu'il a et peut payer (`prets`),
/// qui portent, dont la condition est remplie et qui ne rechargent pas (`lances` : son dernier
/// lancer de chaque sort). Rien si aucun ne convient.
pub fn choisir<'a>(
    table: &'a [Sort],
    prets: &[String],
    situation: Situation,
    lances: &[(String, f64)],
    temps: f64,
    hasard: f32,
) -> Option<&'a Sort> {
    let possibles: Vec<(&Sort, f32)> = table
        .iter()
        .filter(|s| prets.contains(&s.sort) && s.porte(situation.distance))
        .filter(|s| s.approche.is_none_or(|vitesse| situation.approche >= vitesse))
        .filter(|s| !lances.iter().any(|(nom, quand)| *nom == s.sort && temps - quand < s.recharge))
        .map(|s| (s, s.poids))
        .collect();
    tirer(&possibles, hasard)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort(nom: &str, min: f32, max: f32) -> Sort {
        Sort { sort: nom.to_string(), min, max, poids: 1.0, recharge: 0.0, approche: None }
    }

    fn table() -> Vec<Sort> {
        vec![
            sort("boule", 8.0, 40.0),
            sort("rayon", 3.0, 14.0),
            Sort { recharge: 6.0, ..sort("onde", 0.0, 5.0) },
            Sort { approche: Some(8.0), poids: 20.0, recharge: 6.0, ..sort("onde", 0.0, 10.0) },
        ]
    }

    fn prets(noms: &[&str]) -> Vec<String> { noms.iter().map(|n| n.to_string()).collect() }

    fn a(distance: f32) -> Situation { Situation { distance, approche: 0.0 } }

    fn choisi(situation: Situation, prets_: &[&str], lances: &[(String, f64)], hasard: f32) -> Option<String> {
        choisir(&table(), &prets(prets_), situation, lances, 100.0, hasard).map(|s| s.sort.clone())
    }

    const TOUS: [&str; 3] = ["boule", "rayon", "onde"];

    /// « Le mage choisit son sort selon la distance » : loin la boule, tout près l'onde.
    #[test]
    fn il_choisit_selon_la_distance() {
        for hasard in [0.0, 0.5, 0.99] {
            assert_eq!(choisi(a(25.0), &TOUS, &[], hasard).as_deref(), Some("boule"));
            assert_eq!(choisi(a(1.5), &TOUS, &[], hasard).as_deref(), Some("onde"));
        }
        // Entre les deux, il tire entre ceux qui portent.
        assert_eq!(choisi(a(10.0), &TOUS, &[], 0.1).as_deref(), Some("boule"));
        assert_eq!(choisi(a(10.0), &TOUS, &[], 0.9).as_deref(), Some("rayon"));
        // Trop loin pour tout : rien.
        assert_eq!(choisi(a(60.0), &TOUS, &[], 0.5), None);
    }

    /// « … et la situation » : une cible qui lui fonce dessus appelle l'onde qui repousse.
    #[test]
    fn une_cible_qui_fonce_appelle_l_onde() {
        let fonce = Situation { distance: 9.0, approche: 10.0 };
        let sorties = (0..100).filter(|i| choisi(fonce, &TOUS, &[], *i as f32 / 100.0).as_deref() == Some("onde")).count();
        assert!(sorties > 80, "{sorties} ondes sur 100");
        // Elle marche : l'onde n'est pas un choix à cette distance.
        let marche = Situation { distance: 9.0, approche: 2.0 };
        assert!((0..100).all(|i| choisi(marche, &TOUS, &[], i as f32 / 100.0).as_deref() != Some("onde")));
    }

    /// Un sort ne se relance pas avant sa recharge ; il ne choisit que parmi ceux qu'il a et peut payer.
    #[test]
    fn recharge_et_sorts_prets() {
        let lance = |quand: f64| vec![("onde".to_string(), quand)];
        assert_eq!(choisi(a(4.0), &TOUS, &lance(97.0), 0.99).as_deref(), Some("rayon"));
        assert_eq!(choisi(a(1.0), &TOUS, &lance(97.0), 0.5), None);
        assert_eq!(choisi(a(1.0), &TOUS, &lance(93.0), 0.5).as_deref(), Some("onde"));
        // Sans le rayon (pas appris, ou pas assez d'énergie).
        assert_eq!(choisi(a(10.0), &["boule", "onde"], &[], 0.9).as_deref(), Some("boule"));
        assert_eq!(choisi(a(10.0), &[], &[], 0.5), None);
    }
}
