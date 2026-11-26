//! Le cerveau d'un ennemi : son état, sa cible, et ce qu'il veut faire à ce tick.
//!
//! IA.1 (socle) : repos, combat, fuite. Il ne connaît que ce que son [`Observation`] contient.

use crate::profil::Profil;

/// Une créature ou un joueur, vu du cerveau (le serveur y met l'`Uid` de Veloren).
pub type Id = u64;

/// Un adversaire que l'ennemi perçoit à ce tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Percu {
    pub id: Id,
    /// Blocs.
    pub distance: f32,
    pub a_terre: bool,
    /// Il a frappé l'ennemi il y a tant de secondes (rien : pas lui, ou jamais).
    pub l_a_frappe_depuis: Option<f64>,
    /// Combien d'ennemis le visent déjà (l'observateur compris, s'il le vise).
    pub dessus: u8,
}

/// Ce qu'un ennemi sait à ce tick.
#[derive(Clone, Debug, Default)]
pub struct Observation {
    /// L'heure du serveur, en secondes.
    pub temps: f64,
    /// Lui-même (sert à décaler ses horloges de celles de ses voisins).
    pub moi: Id,
    /// Part de ses PV, de 0 à 1.
    pub pv: f32,
    /// Membres de sa race autour de lui, lui compris.
    pub meute: usize,
    /// Les adversaires qu'il perçoit.
    pub percus: Vec<Percu>,
    /// Sa cible du tick précédent est-elle encore en vie (et attaquable) ?
    pub cible_en_vie: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etat {
    Repos,
    Combat,
    Fuite,
}

/// Ce que l'ennemi veut faire ; le serveur l'exécute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intention {
    /// Vaquer : l'errance et les ordres de Veloren.
    Repos,
    /// Se battre contre cette cible.
    Combattre(Id),
    /// S'éloigner de cette menace.
    Fuir(Id),
}

#[derive(Clone, Debug)]
pub struct Cerveau {
    etat: Etat,
    cible: Option<Id>,
    /// Quand il a choisi sa cible pour la dernière fois.
    choisie_a: f64,
    /// Quand il a perçu sa cible pour la dernière fois.
    percue_a: f64,
}

impl Default for Cerveau {
    fn default() -> Self { Self { etat: Etat::Repos, cible: None, choisie_a: 0.0, percue_a: 0.0 } }
}

impl Cerveau {
    pub fn etat(&self) -> Etat { self.etat }

    pub fn cible(&self) -> Option<Id> { self.cible }

    pub fn tick(&mut self, obs: &Observation, profil: &Profil) -> Intention {
        // Sa cible : morte, ou plus perçue depuis trop longtemps, il l'abandonne.
        if let Some(cible) = self.cible {
            if obs.percus.iter().any(|p| p.id == cible) {
                self.percue_a = obs.temps;
            }
            let oubliee = profil.oubli.is_some_and(|oubli| obs.temps - self.percue_a > oubli);
            if !obs.cible_en_vie || oubliee {
                self.cible = None;
            }
        }

        // Il (re)choisit : sans cible, à intervalles, ou dès que sa cible tombe à terre.
        let depuis = obs.temps - self.choisie_a;
        let a_terre = self.cible.is_some_and(|c| obs.percus.iter().any(|p| p.id == c && p.a_terre));
        let a_choisir = self.cible.is_none()
            || profil.reciblage_de(obs.moi).is_some_and(|delai| depuis > delai)
            || (a_terre && depuis > 0.5);
        if a_choisir && let Some(choisie) = self.choisir(obs, profil) {
            self.cible = Some(choisie);
            self.choisie_a = obs.temps;
            self.percue_a = obs.temps;
        }

        let Some(cible) = self.cible else {
            self.etat = Etat::Repos;
            return Intention::Repos;
        };
        if profil.fuite.is_some_and(|seuil| obs.pv < seuil) {
            self.etat = Etat::Fuite;
            Intention::Fuir(cible)
        } else {
            self.etat = Etat::Combat;
            Intention::Combattre(cible)
        }
    }

    /// La meilleure cible parmi ceux qu'il perçoit : le plus petit rang, puis le plus petit score
    /// (`cible.rs`).
    fn choisir(&self, obs: &Observation, profil: &Profil) -> Option<Id> {
        let acheve = profil.acheve_a(obs.meute);
        obs.percus
            .iter()
            .map(|p| {
                // Sans se compter parmi ceux qui la visent déjà.
                let dessus = p.dessus.saturating_sub(u8::from(self.cible == Some(p.id)));
                let rang = profil.rang(p.distance, p.a_terre, p.l_a_frappe_depuis, acheve);
                (rang, profil.score(p.distance, dessus), p.id)
            })
            .min_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)))
            .map(|(_, _, id)| id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orc() -> Profil {
        Profil { reciblage: Some(3.0), repartition: Some(8.0), oubli: Some(6.0), ..Profil::default() }
    }

    fn danari() -> Profil {
        Profil {
            acheve: Some(20.0),
            meute: Some(2),
            meute_rayon: Some(20.0),
            provoque_par_coup: Some(3.0),
            fuite: Some(0.3),
            ..orc()
        }
    }

    fn joueur(id: Id, distance: f32) -> Percu {
        Percu { id, distance, a_terre: false, l_a_frappe_depuis: None, dessus: 0 }
    }

    /// L'observation d'un ennemi en pleine forme, seul de sa race (`moi` 0 : reciblage à 75 %).
    fn obs(temps: f64, percus: Vec<Percu>) -> Observation {
        Observation { temps, moi: 0, pv: 1.0, meute: 1, percus, cible_en_vie: true }
    }

    #[test]
    fn sans_personne_il_reste_au_repos() {
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![]), &orc()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.cible(), None);
    }

    #[test]
    fn il_combat_celui_qu_il_percoit() {
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![joueur(7, 10.0)]), &orc()), Intention::Combattre(7));
        assert_eq!(c.etat(), Etat::Combat);
    }

    /// `creatures.md`, choix de cible : un joueur debout passe avant un joueur à terre.
    #[test]
    fn il_prend_le_joueur_debout_avant_le_joueur_a_terre() {
        let mut c = Cerveau::default();
        let a_terre = Percu { a_terre: true, ..joueur(1, 2.0) };
        assert_eq!(c.tick(&obs(0.0, vec![a_terre, joueur(2, 15.0)]), &orc()), Intention::Combattre(2));
    }

    /// `creatures.md` : « elle garde sa cible tant qu'elle la perçoit » ; le reciblage (Y.9) ne
    /// revient que toutes les `reciblage` secondes.
    #[test]
    fn il_garde_sa_cible_jusqu_au_reciblage() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        // Un joueur plus proche arrive : pas avant le délai (3 s × 0,75 pour `moi` 0).
        let deux = vec![joueur(1, 10.0), joueur(2, 3.0)];
        assert_eq!(c.tick(&obs(1.0, deux.clone()), &orc()), Intention::Combattre(1));
        assert_eq!(c.tick(&obs(2.5, deux), &orc()), Intention::Combattre(2));
    }

    /// Y.9 : il ne se compte pas parmi ceux qui visent déjà sa cible (sinon il la quitterait pour
    /// un joueur plus loin dès le premier reciblage).
    #[test]
    fn il_ne_se_compte_pas_sur_sa_propre_cible() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 5.0), joueur(2, 9.0)]), &orc());
        let sur_lui = vec![Percu { dessus: 1, ..joueur(1, 5.0) }, joueur(2, 9.0)];
        assert_eq!(c.tick(&obs(10.0, sur_lui), &orc()), Intention::Combattre(1));
    }

    /// Y.9 : une cible qui tombe à terre fait choisir tout de suite.
    #[test]
    fn sa_cible_tombe_a_terre_il_en_change_aussitot() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0), joueur(2, 12.0)]), &orc());
        let tombe = vec![Percu { a_terre: true, ..joueur(1, 3.0) }, joueur(2, 12.0)];
        assert_eq!(c.tick(&obs(1.0, tombe), &orc()), Intention::Combattre(2));
    }

    #[test]
    fn sa_cible_meurt_il_en_prend_une_autre_ou_se_repose() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        let morte = Observation { cible_en_vie: false, ..obs(0.5, vec![joueur(2, 12.0)]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Combattre(2));
        let morte = Observation { cible_en_vie: false, ..obs(1.0, vec![]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
    }

    /// Il abandonne une cible qu'il ne perçoit plus depuis `oubli` secondes.
    #[test]
    fn il_abandonne_une_cible_qu_il_ne_percoit_plus() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        assert_eq!(c.tick(&obs(5.0, vec![]), &orc()), Intention::Combattre(1));
        assert_eq!(c.tick(&obs(6.5, vec![]), &orc()), Intention::Repos);
        // Revue entre-temps, le compte repart.
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        c.tick(&obs(5.0, vec![joueur(1, 3.0)]), &orc());
        assert_eq!(c.tick(&obs(6.5, vec![]), &orc()), Intention::Combattre(1));
    }

    /// Y.1 : un danari fuit sous 30 % de ses PV ; un orc ne fuit pas.
    #[test]
    fn le_danari_blesse_fuit_pas_l_orc() {
        let blesse = Observation { pv: 0.2, ..obs(0.0, vec![joueur(1, 3.0)]) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&blesse, &danari()), Intention::Fuir(1));
        assert_eq!(c.etat(), Etat::Fuite);
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&blesse, &orc()), Intention::Combattre(1));
        // Sans personne à fuir, il se repose.
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&Observation { pv: 0.2, ..obs(0.0, vec![]) }, &danari()), Intention::Repos);
    }

    /// Y.2 : les danari à plusieurs achèvent le joueur à terre, sauf si quelqu'un vient de les frapper.
    #[test]
    fn les_danari_a_plusieurs_achevent_sauf_si_on_les_frappe() {
        let a_terre = Percu { a_terre: true, ..joueur(1, 15.0) };
        let a_deux = |percus| Observation { meute: 2, ..obs(0.0, percus) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&a_deux(vec![a_terre.clone(), joueur(2, 2.0)]), &danari()), Intention::Combattre(1));
        // Seul, il fait comme les autres.
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![a_terre.clone(), joueur(2, 2.0)]), &danari()), Intention::Combattre(2));
        // Frappé à l'instant par le joueur debout : c'est lui d'abord.
        let frappe = Percu { l_a_frappe_depuis: Some(1.0), ..joueur(2, 2.0) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&a_deux(vec![a_terre, frappe]), &danari()), Intention::Combattre(2));
    }
}
