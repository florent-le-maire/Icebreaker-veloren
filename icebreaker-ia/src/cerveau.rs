//! Le cerveau d'un ennemi : son état, sa cible, et ce qu'il veut faire à ce tick.
//!
//! Il ne connaît que ce que son [`Observation`] contient : ceux qu'il perçoit à ce tick. D'une cible
//! perdue de vue, il ne garde que la dernière position où il l'a perçue.

use crate::profil::Profil;
use vek::Vec3;

/// Une créature ou un joueur, vu du cerveau (le serveur y met l'`Uid` de Veloren).
pub type Id = u64;

/// Un adversaire que l'ennemi perçoit à ce tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Percu {
    pub id: Id,
    pub pos: Vec3<f32>,
    /// Il sait qui c'est (sinon : un soupçon, `perception::Niveau`).
    pub net: bool,
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
    pub pos: Vec3<f32>,
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
    /// Il a un soupçon, ou il est venu voir un endroit (« ? »).
    Enquete,
    Combat,
    Fuite,
}

/// Ce que l'ennemi veut faire ; le serveur l'exécute.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intention {
    /// Vaquer : l'errance et les ordres de Veloren.
    Repos,
    /// Se battre contre cette cible, qu'il perçoit.
    Combattre(Id),
    /// S'éloigner de cet endroit.
    Fuir(Vec3<f32>),
    /// Courir à cet endroit.
    AllerVoir(Vec3<f32>),
    /// Rester sur place et regarder vers cet endroit.
    Regarder(Vec3<f32>),
    /// Rentrer à son poste.
    Rentrer(Vec3<f32>),
}

/// Distance à laquelle un endroit est atteint quand le profil ne dit rien (`arrivee`).
const ARRIVEE: f32 = 2.0;
/// La fouille du regard : il regarde un point à `REGARD` blocs, qui tourne de `TOUR` radians par seconde.
const REGARD: f32 = 4.0;
const TOUR: f32 = 1.0;
/// Le bruit de la mort de sa victime : moins de `VICTIME` secondes après, à moins de `VICTIME_PRES` blocs
/// de là où il l'a vue.
const VICTIME: f64 = 1.0;
const VICTIME_PRES: f32 = 6.0;

/// Un endroit où il est venu voir.
#[derive(Clone, Copy, Debug)]
struct Enquete {
    lieu: Vec3<f32>,
    depuis: f64,
    /// Arrivé sur place à cette heure.
    sur_place: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Cerveau {
    etat: Etat,
    cible: Option<Id>,
    /// Quand il a choisi sa cible pour la dernière fois.
    choisie_a: f64,
    /// Quand et où il a perçu sa cible pour la dernière fois.
    percue_a: f64,
    percue_en: Vec3<f32>,
    /// Ceux qu'il soupçonne, et depuis quand sans interruption.
    soupcons: Vec<(Id, f64)>,
    enquete: Option<Enquete>,
    /// Sa place : où il rentre après un combat ou une enquête.
    poste: Option<Vec3<f32>>,
    /// Il doit rentrer à son poste.
    rentre: bool,
    /// Le dernier bruit entendu : où, quand.
    bruit: Option<(Vec3<f32>, f64)>,
    /// Sa victime vient de mourir là, à cette heure : le bruit de cette mort n'est pas à aller voir.
    victime: Option<(Vec3<f32>, f64)>,
}

impl Default for Cerveau {
    fn default() -> Self {
        Self {
            etat: Etat::Repos,
            cible: None,
            choisie_a: 0.0,
            percue_a: 0.0,
            percue_en: Vec3::zero(),
            soupcons: Vec::new(),
            enquete: None,
            poste: None,
            rentre: false,
            bruit: None,
            victime: None,
        }
    }
}

impl Cerveau {
    pub fn etat(&self) -> Etat { self.etat }

    pub fn cible(&self) -> Option<Id> { self.cible }

    /// Il entend un bruit fait à cet endroit, à cette heure (`ouie.rs` dit s'il l'entend).
    pub fn entendre(&mut self, lieu: Vec3<f32>, temps: f64) {
        if self.bruit.is_none_or(|(_, avant)| temps >= avant) {
            self.bruit = Some((lieu, temps));
        }
    }

    pub fn tick(&mut self, obs: &Observation, profil: &Profil) -> Intention {
        let poste = *self.poste.get_or_insert(obs.pos);
        let arrivee = profil.arrivee.unwrap_or(ARRIVEE);
        // Trop loin de son poste (`laisse`) : il ne se bat plus, il rentre.
        let dehors = profil.laisse.is_some_and(|laisse| obs.pos.distance(poste) > laisse);

        // Ses soupçons : ceux qu'il entrevoit, depuis quand sans interruption.
        self.soupcons.retain(|(id, _)| obs.percus.iter().any(|p| p.id == *id && !p.net));
        for p in obs.percus.iter().filter(|p| !p.net) {
            if !self.soupcons.iter().any(|(id, _)| *id == p.id) {
                self.soupcons.push((p.id, obs.temps));
            }
        }

        // Sa cible : il note où il la perçoit ; morte, ou lui trop loin de son poste, il la lâche.
        let mut vue = false;
        if let Some(cible) = self.cible {
            if let Some(p) = obs.percus.iter().find(|p| p.id == cible) {
                vue = true;
                self.percue_a = obs.temps;
                self.percue_en = p.pos;
            }
            if !obs.cible_en_vie || dehors {
                self.cible = None;
                self.rentre = true;
                // Le bruit entendu pendant ce combat était le sien.
                self.bruit = None;
                if !obs.cible_en_vie {
                    self.victime = Some((self.percue_en, obs.temps));
                }
            }
        }

        // Il (re)choisit : sans cible, à intervalles, ou dès que sa cible tombe à terre.
        let depuis = obs.temps - self.choisie_a;
        let a_terre = self.cible.is_some_and(|c| obs.percus.iter().any(|p| p.id == c && p.a_terre));
        let a_choisir = !dehors
            && (self.cible.is_none()
                || profil.reciblage_de(obs.moi).is_some_and(|delai| depuis > delai)
                || (a_terre && depuis > 0.5));
        if a_choisir && let Some(choisie) = self.choisir(obs, profil) {
            vue = true;
            self.cible = Some(choisie.id);
            self.choisie_a = obs.temps;
            self.percue_a = obs.temps;
            self.percue_en = choisie.pos;
            self.enquete = None;
        }

        if let Some(cible) = self.cible {
            let fuit = profil.fuite.is_some_and(|seuil| obs.pv < seuil);
            // Perdue de vue : il sait encore un moment où elle était.
            if vue || obs.temps - self.percue_a <= profil.poursuite.unwrap_or(0.0) {
                return if fuit {
                    self.etat = Etat::Fuite;
                    Intention::Fuir(self.percue_en)
                } else if vue {
                    self.etat = Etat::Combat;
                    Intention::Combattre(cible)
                } else {
                    self.etat = Etat::Combat;
                    Intention::AllerVoir(self.percue_en)
                };
            }
            self.cible = None;
            if fuit {
                // Le fuyard se calme là où il est : c'est sa nouvelle place.
                self.poste = Some(obs.pos);
                self.rentre = false;
            } else {
                // Il va voir ce qu'il sait de plus récent : où il l'a vue, ou un bruit entendu depuis.
                let lieu = self.bruit.take().filter(|(_, quand)| *quand > self.percue_a).map_or(self.percue_en, |(lieu, _)| lieu);
                self.enquete = Some(Enquete { lieu, depuis: obs.temps, sur_place: None });
            }
        }

        // Un bruit : il va voir (pas celui qui fuirait, ni celui que sa laisse retient).
        if let Some((lieu, quand)) = self.bruit.take()

            && !self.victime.is_some_and(|(ou, mort)| quand - mort < VICTIME && lieu.distance(ou) < VICTIME_PRES)
            && !dehors
            && !profil.fuite.is_some_and(|seuil| obs.pv < seuil)
        {
            self.enquete = Some(Enquete { lieu, depuis: obs.temps, sur_place: None });
        }

        // Un soupçon : il s'arrête et regarde.
        if !dehors && let Some(p) = obs.percus.iter().find(|p| !p.net) {
            self.etat = Etat::Enquete;
            return Intention::Regarder(p.pos);
        }

        // Il va voir un endroit, y fouille du regard, puis rentre.
        if let Some(enquete) = &mut self.enquete {
            if enquete.sur_place.is_none() && obs.pos.distance(enquete.lieu) <= arrivee {
                enquete.sur_place = Some(obs.temps);
            }
            let intention = match enquete.sur_place {
                None if obs.temps - enquete.depuis <= profil.trajet.unwrap_or(0.0) => {
                    Some(Intention::AllerVoir(enquete.lieu))
                },
                Some(debut) if obs.temps - debut <= profil.fouille.unwrap_or(0.0) => {
                    let angle = (obs.temps - debut) as f32 * TOUR;
                    Some(Intention::Regarder(obs.pos + Vec3::new(angle.cos(), angle.sin(), 0.0) * REGARD))
                },
                _ => None,
            };
            if let Some(intention) = intention {
                self.etat = Etat::Enquete;
                return intention;
            }
            self.enquete = None;
            self.rentre = true;
        }

        self.etat = Etat::Repos;
        if self.rentre {
            let poste = self.poste.unwrap_or(poste);
            if obs.pos.distance(poste) > arrivee {
                return Intention::Rentrer(poste);
            }
            self.rentre = false;
        }
        Intention::Repos
    }

    /// La meilleure cible parmi ceux qu'il a repérés (reconnus, déjà sa cible, ou soupçonnés
    /// depuis `soupcon` secondes) : le plus petit rang, puis le plus petit score (`cible.rs`).
    fn choisir<'a>(&self, obs: &'a Observation, profil: &Profil) -> Option<&'a Percu> {
        let acheve = profil.acheve_a(obs.meute);
        let repere = |p: &Percu| {
            p.net
                || self.cible == Some(p.id)
                || self.soupcons.iter().any(|(id, depuis)| {
                    *id == p.id && obs.temps - depuis >= profil.soupcon.unwrap_or(0.0)
                })
        };
        obs.percus
            .iter()
            .filter(|p| repere(p))
            .map(|p| {
                let distance = p.pos.distance(obs.pos);
                // Sans se compter parmi ceux qui la visent déjà.
                let dessus = p.dessus.saturating_sub(u8::from(self.cible == Some(p.id)));
                let rang = profil.rang(distance, p.a_terre, p.l_a_frappe_depuis, acheve);
                (rang, profil.score(distance, dessus), p)
            })
            .min_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)))
            .map(|(_, _, p)| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orc() -> Profil {
        Profil {
            reciblage: Some(3.0),
            repartition: Some(8.0),
            soupcon: Some(1.0),
            poursuite: Some(3.0),
            fouille: Some(6.0),
            trajet: Some(20.0),
            arrivee: Some(2.0),
            ..Profil::default()
        }
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

    fn en(x: f32, y: f32) -> Vec3<f32> { Vec3::new(x, y, 0.0) }

    /// Un joueur debout, reconnu, à `distance` blocs à l'est de l'origine.
    fn joueur(id: Id, distance: f32) -> Percu {
        Percu { id, pos: en(distance, 0.0), net: true, a_terre: false, l_a_frappe_depuis: None, dessus: 0 }
    }

    /// L'observation d'un ennemi en pleine forme à l'origine, seul de sa race (`moi` 0 : reciblage
    /// à 75 %).
    fn obs(temps: f64, percus: Vec<Percu>) -> Observation {
        Observation { temps, moi: 0, pos: Vec3::zero(), pv: 1.0, meute: 1, percus, cible_en_vie: true }
    }

    fn obs_en(temps: f64, pos: Vec3<f32>, percus: Vec<Percu>) -> Observation {
        Observation { pos, ..obs(temps, percus) }
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

    /// Sa cible morte, il en prend une autre ; sinon il rentre à son poste, sans fouiller.
    #[test]
    fn sa_cible_meurt_il_en_prend_une_autre_ou_rentre() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        let morte = Observation { cible_en_vie: false, ..obs(0.5, vec![joueur(2, 12.0)]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Combattre(2));
        // Il a couru 30 blocs ; sa cible meurt : il rentre d'où il vient.
        let morte = Observation { cible_en_vie: false, ..obs_en(1.0, en(30.0, 0.0), vec![]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Rentrer(Vec3::zero()));
        assert_eq!(c.etat(), Etat::Repos);
        // Rentré, il vaque.
        assert_eq!(c.tick(&obs(9.0, vec![]), &orc()), Intention::Repos);
    }

    /// « Pour Veloren » : un joueur loin ou discret est d'abord soupçonné (l'ennemi se tourne et
    /// regarde), puis repéré s'il reste visible.
    #[test]
    fn un_soupcon_qui_dure_devient_un_combat() {
        let loin = |id| Percu { net: false, ..joueur(id, 25.0) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![loin(1)]), &orc()), Intention::Regarder(en(25.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
        assert_eq!(c.cible(), None);
        assert_eq!(c.tick(&obs(0.9, vec![loin(1)]), &orc()), Intention::Regarder(en(25.0, 0.0)));
        assert_eq!(c.tick(&obs(1.1, vec![loin(1)]), &orc()), Intention::Combattre(1));
    }

    /// Un soupçon qui disparaît repart de zéro : on passe pendant qu'il regarde ailleurs.
    #[test]
    fn un_soupcon_interrompu_repart_de_zero() {
        let loin = Percu { net: false, ..joueur(1, 25.0) };
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![loin.clone()]), &orc());
        assert_eq!(c.tick(&obs(0.8, vec![]), &orc()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.tick(&obs(1.2, vec![loin.clone()]), &orc()), Intention::Regarder(en(25.0, 0.0)));
        assert_eq!(c.tick(&obs(2.3, vec![loin]), &orc()), Intention::Combattre(1));
    }

    /// `creatures.md` : « Une cible perdue de vue est poursuivie 3 s vers sa dernière position
    /// connue, puis la créature passe en alerte à cet endroit » ; elle y fouille 6 s, puis rentre.
    #[test]
    fn cible_perdue_il_court_ou_il_l_a_vue_fouille_puis_rentre() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        // Le joueur passe derrière un mur : l'ennemi ne sait que là où il l'a vu.
        assert_eq!(c.tick(&obs(1.0, vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        assert_eq!(c.etat(), Etat::Combat);
        // Après 3 s, il n'a plus de cible : il enquête là-bas.
        assert_eq!(c.tick(&obs_en(3.5, en(6.0, 0.0), vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
        assert_eq!(c.cible(), None);
        // Arrivé, il fouille du regard, pas toujours du même côté.
        let sur_place = en(9.0, 0.0);
        let Intention::Regarder(a) = c.tick(&obs_en(5.0, sur_place, vec![]), &orc()) else {
            panic!("il doit regarder autour de lui")
        };
        let Intention::Regarder(b) = c.tick(&obs_en(7.0, sur_place, vec![]), &orc()) else {
            panic!("il doit regarder autour de lui")
        };
        assert!(a.distance(b) > 1.0, "il regarde toujours au même endroit");
        assert_eq!(c.etat(), Etat::Enquete);
        // 6 s plus tard, il rentre à son poste (là où il était au premier tick), puis vaque.
        assert_eq!(c.tick(&obs_en(11.5, sur_place, vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.tick(&obs_en(15.0, en(1.0, 0.0), vec![]), &orc()), Intention::Repos);
    }

    #[test]
    fn revue_pendant_la_fouille_le_combat_reprend() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        c.tick(&obs_en(3.5, en(6.0, 0.0), vec![]), &orc());
        c.tick(&obs_en(5.0, en(9.0, 0.0), vec![]), &orc());
        assert_eq!(c.tick(&obs_en(6.0, en(9.0, 0.0), vec![joueur(1, 14.0)]), &orc()), Intention::Combattre(1));
        assert_eq!(c.etat(), Etat::Combat);
    }

    /// Un endroit qu'il n'arrive pas à rejoindre : il renonce après `trajet` secondes.
    #[test]
    fn il_renonce_a_un_endroit_qu_il_n_atteint_pas() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        c.tick(&obs_en(3.5, en(5.0, 0.0), vec![]), &orc());
        assert_eq!(c.tick(&obs_en(20.0, en(5.0, 0.0), vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        assert_eq!(c.tick(&obs_en(24.0, en(5.0, 0.0), vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
    }

    /// Y.5 : avec une laisse, il ne s'éloigne pas de son poste : il lâche sa cible et rentre, et
    /// n'en reprend pas tant qu'il est dehors.
    #[test]
    fn la_laisse_le_ramene_a_son_poste() {
        let nain = Profil { laisse: Some(15.0), ..orc() };
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &nain);
        let dehors = en(16.0, 0.0);
        let devant = Percu { pos: en(20.0, 0.0), ..joueur(1, 0.0) };
        assert_eq!(c.tick(&obs_en(4.0, dehors, vec![devant.clone()]), &nain), Intention::Rentrer(Vec3::zero()));
        assert_eq!(c.cible(), None);
        assert_eq!(c.tick(&obs_en(4.5, dehors, vec![devant.clone()]), &nain), Intention::Rentrer(Vec3::zero()));
        // De retour dans son rayon, il se bat de nouveau.
        assert_eq!(c.tick(&obs_en(6.0, en(8.0, 0.0), vec![devant]), &nain), Intention::Combattre(1));
    }

    /// Y.1 : un danari fuit sous 30 % de ses PV ; un orc ne fuit pas.
    #[test]
    fn le_danari_blesse_fuit_pas_l_orc() {
        let blesse = Observation { pv: 0.2, ..obs(0.0, vec![joueur(1, 3.0)]) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&blesse, &danari()), Intention::Fuir(en(3.0, 0.0)));
        assert_eq!(c.etat(), Etat::Fuite);
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&blesse, &orc()), Intention::Combattre(1));
        // Sans personne à fuir, il se repose.
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&Observation { pv: 0.2, ..obs(0.0, vec![]) }, &danari()), Intention::Repos);
    }

    /// `creatures.md`, « le fuyard n'oublie pas » : il fuit encore une menace qu'il ne voit plus,
    /// puis se calme là où il est : c'est sa nouvelle place, il ne revient pas voir.
    #[test]
    fn le_fuyard_se_calme_la_ou_il_est() {
        let mut c = Cerveau::default();
        c.tick(&Observation { pv: 0.2, ..obs(0.0, vec![joueur(1, 3.0)]) }, &danari());
        let loin = en(-20.0, 0.0);
        let seul = |temps| Observation { pv: 0.2, ..obs_en(temps, loin, vec![]) };
        assert_eq!(c.tick(&seul(2.0), &danari()), Intention::Fuir(en(3.0, 0.0)));
        assert_eq!(c.tick(&seul(4.0), &danari()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.tick(&seul(30.0), &danari()), Intention::Repos);
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

    /// « Entendre un bruit → enquête : il va voir l'endroit, fouille un moment, puis rentre à son
    /// poste. »
    #[test]
    fn un_bruit_le_fait_venir_voir_fouiller_puis_rentrer() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![]), &orc());
        c.entendre(en(15.0, 0.0), 1.0);
        assert_eq!(c.tick(&obs(1.0, vec![]), &orc()), Intention::AllerVoir(en(15.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
        let sur_place = en(14.0, 0.0);
        assert!(matches!(c.tick(&obs_en(5.0, sur_place, vec![]), &orc()), Intention::Regarder(_)));
        assert_eq!(c.tick(&obs_en(11.5, sur_place, vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
    }

    /// Un bruit plus récent déplace l'enquête.
    #[test]
    fn un_nouveau_bruit_deplace_l_enquete() {
        let mut c = Cerveau::default();
        c.entendre(en(15.0, 0.0), 0.0);
        c.tick(&obs(0.0, vec![]), &orc());
        c.entendre(en(0.0, 12.0), 2.0);
        assert_eq!(c.tick(&obs(2.0, vec![]), &orc()), Intention::AllerVoir(en(0.0, 12.0)));
    }

    /// « Il entend aussi pendant un combat » : il ne lâche pas la cible qu'il voit ; s'il la perd,
    /// il va voir le bruit entendu depuis plutôt que l'endroit où il l'a vue.
    #[test]
    fn en_combat_il_retient_le_bruit_pour_quand_il_perd_sa_cible() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        c.entendre(en(0.0, -14.0), 0.5);
        assert_eq!(c.tick(&obs(1.0, vec![joueur(1, 10.0)]), &orc()), Intention::Combattre(1));
        // Perdue de vue : ce bruit est plus vieux que la dernière fois qu'il l'a vue.
        assert_eq!(c.tick(&obs(2.0, vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        c.entendre(en(0.0, -14.0), 3.0);
        assert_eq!(c.tick(&obs(4.5, vec![]), &orc()), Intention::AllerVoir(en(0.0, -14.0)));
        assert_eq!(c.etat(), Etat::Enquete);
    }

    /// Le bruit de son propre combat ne le fait pas fouiller quand sa cible meurt : il rentre.
    #[test]
    fn sa_cible_morte_il_oublie_le_bruit_de_son_combat() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        c.entendre(en(3.0, 0.0), 0.5);
        let morte = Observation { cible_en_vie: false, ..obs_en(1.0, en(30.0, 0.0), vec![]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Rentrer(Vec3::zero()));
        // Le bruit de la mort de sa victime lui arrive juste après : il ne va pas la voir.
        c.entendre(en(3.5, 0.0), 1.03);
        assert_eq!(c.tick(&obs_en(1.1, en(30.0, 0.0), vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
        // Une autre mort, ailleurs ou plus tard : il y va.
        c.entendre(en(3.5, 20.0), 1.2);
        assert_eq!(c.tick(&obs_en(1.2, en(30.0, 0.0), vec![]), &orc()), Intention::AllerVoir(en(3.5, 20.0)));
    }

    /// Celui qui fuirait (PV sous son seuil) ne va pas voir un bruit.
    #[test]
    fn un_blesse_qui_fuit_ne_va_pas_voir_un_bruit() {
        let mut c = Cerveau::default();
        c.entendre(en(15.0, 0.0), 0.0);
        assert_eq!(c.tick(&Observation { pv: 0.2, ..obs(0.0, vec![]) }, &danari()), Intention::Repos);
    }

    /// `menace.md`, principe 1 : « les créatures ne trichent pas ». Deux cerveaux qui reçoivent les
    /// mêmes observations décident pareil : rien d'autre n'entre dans la décision.
    #[test]
    fn memes_observations_memes_decisions() {
        let suite = [
            obs(0.0, vec![joueur(1, 10.0)]),
            obs(1.0, vec![]),
            obs_en(3.5, en(6.0, 0.0), vec![]),
            obs_en(5.0, en(9.0, 0.0), vec![]),
            obs_en(12.0, en(9.0, 0.0), vec![]),
        ];
        let (mut a, mut b) = (Cerveau::default(), Cerveau::default());
        for o in &suite {
            assert_eq!(a.tick(o, &orc()), b.tick(o, &orc()));
        }
    }
}
