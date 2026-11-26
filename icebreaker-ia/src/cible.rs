//! Le choix de cible (Y.2, Y.9 ; venu de `common/src/icebreaker_races.rs`) : parmi les joueurs qu'il
//! perçoit, un ennemi prend d'abord un joueur debout (tous à terre : ils sont achevés), et les
//! ennemis se répartissent entre les joueurs. Une race qui achève (danari) prend d'abord le joueur à
//! terre, à plusieurs seulement, sauf si quelqu'un vient de la frapper.

use crate::profil::Profil;

impl Profil {
    /// La race achève-t-elle les joueurs à terre, avec `proches` membres de la race (lui compris)
    /// autour de lui ? Seulement si elle achève (`acheve`), et à plusieurs si `meute` le demande.
    pub fn acheve_a(&self, proches: usize) -> bool {
        self.acheve.is_some() && self.meute.is_none_or(|m| proches >= m)
    }

    /// Rang d'une cible (plus petit = choisie d'abord) : 0 celui qui vient de frapper une race
    /// qui achève (le contre) ; puis, si la race achève ici (`acheve`), 1 un joueur à terre à
    /// portée et 2 le reste ; sinon 1 un joueur debout et 2 un joueur à terre (tous à terre :
    /// ils sont achevés quand même).
    pub fn rang(&self, distance: f32, a_terre: bool, l_a_frappe_depuis: Option<f64>, acheve: bool) -> u8 {
        if self.acheve.is_some()
            && l_a_frappe_depuis.is_some_and(|s| s < self.provoque_par_coup.unwrap_or(0.0))
        {
            0
        } else if acheve {
            if a_terre && self.acheve.is_some_and(|r| distance < r) { 1 } else { 2 }
        } else if a_terre {
            2
        } else {
            1
        }
    }

    /// À rang égal, la cible au plus petit score : sa distance, plus `repartition` blocs par allié
    /// qui la vise déjà (`dessus`, sans se compter) : les ennemis se répartissent.
    pub fn score(&self, distance: f32, dessus: u8) -> f32 {
        distance + self.repartition.unwrap_or(0.0) * dessus as f32
    }

    /// Secondes entre deux choix de cible en combat pour la créature `id` (rien = jamais, sauf
    /// cible perdue ou à terre), décalé de ± 25 % selon la créature pour que tout un groupe ne
    /// change pas de cible au même instant.
    pub fn reciblage_de(&self, id: u64) -> Option<f64> {
        Some(self.reciblage? * (0.75 + 0.5 * (id % 8) as f64 / 7.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orc() -> Profil { Profil { reciblage: Some(3.0), repartition: Some(8.0), ..Profil::default() } }

    fn danari() -> Profil {
        Profil { acheve: Some(20.0), provoque_par_coup: Some(3.0), meute: Some(2), ..orc() }
    }

    #[test]
    fn un_joueur_debout_passe_avant_un_joueur_a_terre() {
        let o = orc();
        assert!(!o.acheve_a(5));
        // Même tout près, le joueur à terre attend.
        assert!(o.rang(20.0, false, None, false) < o.rang(1.0, true, None, false));
        // Tous à terre : même rang, le plus proche est achevé.
        assert_eq!(o.rang(3.0, true, None, false), o.rang(9.0, true, None, false));
        assert!(o.score(3.0, 0) < o.score(9.0, 0));
    }

    #[test]
    fn le_danari_acheve_a_plusieurs_seulement() {
        let d = danari();
        // Seul : comme les autres.
        assert!(!d.acheve_a(1));
        assert!(d.rang(15.0, false, None, d.acheve_a(1)) < d.rang(2.0, true, None, d.acheve_a(1)));
        // À deux : le joueur à terre d'abord, s'il est à portée.
        assert!(d.acheve_a(2));
        assert!(d.rang(15.0, true, None, true) < d.rang(2.0, false, None, true));
        assert_eq!(d.rang(25.0, true, None, true), 2);
    }

    #[test]
    fn frapper_le_danari_le_detourne() {
        let d = danari();
        assert!(d.rang(10.0, false, Some(1.0), true) < d.rang(2.0, true, None, true));
        // Le coup est trop ancien : il retourne au joueur à terre.
        assert!(d.rang(10.0, false, Some(5.0), true) > d.rang(2.0, true, None, true));
        // Un orc ne se laisse pas détourner d'un joueur debout par un coup.
        let o = orc();
        assert_eq!(o.rang(10.0, false, Some(1.0), false), o.rang(2.0, false, None, false));
    }

    #[test]
    fn les_ennemis_se_repartissent() {
        let o = orc();
        // Deux joueurs debout à 5 et 9 blocs : le deuxième ennemi prend le plus loin, libre.
        assert!(o.score(9.0, 0) < o.score(5.0, 1));
        // Mais pas un joueur bien plus loin.
        assert!(o.score(30.0, 0) > o.score(5.0, 1));
        // Autant d'ennemis sur chacun : le plus proche.
        assert!(o.score(5.0, 1) < o.score(9.0, 1));
    }

    #[test]
    fn le_reciblage_est_decale_par_creature() {
        let o = orc();
        let delais: Vec<f64> = (0..8).map(|id| o.reciblage_de(id).unwrap()).collect();
        assert!(delais.iter().all(|d| (2.25..=3.75).contains(d)));
        assert!(delais.windows(2).all(|w| w[0] < w[1]));
        // Sans réglage : pas de reciblage périodique.
        assert_eq!(Profil::default().reciblage_de(3), None);
    }
}
