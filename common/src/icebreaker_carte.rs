//! Icebreaker CA : la carte 3D de l'expédition (un objet du sac). Elle montre en volume ce que le
//! personnage a exploré, qu'on fait tourner à la souris, et les objectifs en rouge. Les réglages sont
//! dans `assets/icebreaker/carte.ron`. Ce module décide (ce qui est découvert, l'image) ; `voxygen` ne
//! fait que brancher (terrain chargé, souris, affichage).

use crate::assets::{AssetExt, Ron};
use hashbrown::HashMap;
use serde::Deserialize;
use std::collections::VecDeque;
use vek::{Rgb, Vec3};

/// L'objet « carte de l'expédition » : dans le sac, il ouvre la carte.
pub const CARTE: &str = "common.items.icebreaker.carte";

/// Les réglages de la carte (`assets/icebreaker/carte.ron`).
#[derive(Clone, Debug, Deserialize)]
pub struct Reglages {
    /// Côté d'une case de la carte, en blocs.
    pub maille: i32,
    /// Jusqu'où on découvre autour de soi, en blocs parcourus dans l'air (un mur arrête).
    pub rayon: u32,
    /// Secondes entre deux découvertes.
    pub periode: f32,
    /// Cases gardées au plus ; au-delà, la carte ne grandit plus.
    pub max_cases: usize,
}

impl Default for Reglages {
    fn default() -> Self {
        Self {
            maille: 2,
            rayon: 24,
            periode: 0.5,
            max_cases: 200_000,
        }
    }
}

impl Reglages {
    /// Les réglages du jeu ; un fichier absent ou invalide donne ceux par défaut (avec un message).
    pub fn charger() -> Self {
        match Ron::<Self>::load("icebreaker.carte") {
            Ok(r) => {
                let reglages = r.read().0.clone();
                match reglages.verifier() {
                    Ok(()) => reglages,
                    Err(e) => {
                        tracing::warn!("Icebreaker carte.ron : {e}");
                        Self::default()
                    },
                }
            },
            Err(e) => {
                tracing::warn!("Icebreaker carte.ron illisible : {e}");
                Self::default()
            },
        }
    }

    pub fn verifier(&self) -> Result<(), String> {
        if self.maille < 1 {
            return Err(format!("maille {} : au moins 1 bloc", self.maille));
        }
        if self.rayon < 1 {
            return Err("rayon : au moins 1 bloc".to_string());
        }
        if self.periode <= 0.0 {
            return Err(format!("periode {} : elle doit être positive", self.periode));
        }
        Ok(())
    }
}

/// Ce qu'il y a à un endroit du monde, vu par la carte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bloc {
    /// On peut voir à travers (air, eau).
    Air,
    /// Un bloc plein, de cette couleur.
    Plein(Rgb<u8>),
    /// Terrain pas chargé : on ne sait pas.
    Inconnu,
}

const VOISINS: [Vec3<i32>; 6] = [
    Vec3::new(1, 0, 0),
    Vec3::new(-1, 0, 0),
    Vec3::new(0, 1, 0),
    Vec3::new(0, -1, 0),
    Vec3::new(0, 0, 1),
    Vec3::new(0, 0, -1),
];

/// Ce qu'un personnage a découvert du lieu : les cases de surface (sol, murs, plafonds) qu'il a vues.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Exploration {
    /// Case (position du monde divisée par la maille) → couleur du bloc.
    cases: HashMap<Vec3<i32>, Rgb<u8>>,
}

impl Exploration {
    pub fn cases(&self) -> impl Iterator<Item = (Vec3<i32>, Rgb<u8>)> + '_ {
        self.cases.iter().map(|(c, col)| (*c, *col))
    }

    pub fn len(&self) -> usize { self.cases.len() }

    pub fn is_empty(&self) -> bool { self.cases.is_empty() }

    /// La case d'une position du monde.
    pub fn case_de(wpos: Vec3<i32>, maille: i32) -> Vec3<i32> { wpos.map(|e| e.div_euclid(maille.max(1))) }

    pub fn connait(&self, wpos: Vec3<i32>, maille: i32) -> bool {
        self.cases.contains_key(&Self::case_de(wpos, maille))
    }

    /// Découvre autour de `depuis` : on avance dans l'air, de `rayon` blocs au plus, et on note les blocs
    /// pleins qu'on touche. Un mur arrête la découverte ; le terrain pas chargé aussi. Rend `true` si la
    /// carte a grandi.
    pub fn explorer(&mut self, depuis: Vec3<i32>, bloc: impl Fn(Vec3<i32>) -> Bloc, reglages: &Reglages) -> bool {
        if bloc(depuis) != Bloc::Air {
            return false;
        }
        let avant = self.cases.len();
        let mut vus = hashbrown::HashSet::new();
        let mut file = VecDeque::new();
        vus.insert(depuis);
        file.push_back((depuis, 0u32));
        while let Some((pos, pas)) = file.pop_front() {
            for voisin in VOISINS.map(|d| pos + d) {
                match bloc(voisin) {
                    Bloc::Air => {
                        if pas < reglages.rayon && vus.insert(voisin) {
                            file.push_back((voisin, pas + 1));
                        }
                    },
                    Bloc::Plein(couleur) => {
                        let case = Self::case_de(voisin, reglages.maille);
                        if self.cases.len() < reglages.max_cases || self.cases.contains_key(&case) {
                            self.cases.entry(case).or_insert(couleur);
                        }
                    },
                    Bloc::Inconnu => {},
                }
            }
        }
        self.cases.len() > avant
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIERRE: Rgb<u8> = Rgb::new(90, 90, 90);

    fn reglages() -> Reglages {
        Reglages {
            maille: 1,
            rayon: 40,
            ..Reglages::default()
        }
    }

    /// Deux salles de 5 de côté creusées dans la pierre, séparées par un mur en x = 5 ; `porte` y perce
    /// un trou.
    fn deux_salles(porte: bool) -> impl Fn(Vec3<i32>) -> Bloc {
        move |p| {
            let dans = |x0: i32| (x0..x0 + 5).contains(&p.x) && (0..5).contains(&p.y) && (0..5).contains(&p.z);
            if dans(0) || dans(6) || (porte && p == Vec3::new(5, 2, 0)) {
                Bloc::Air
            } else {
                Bloc::Plein(PIERRE)
            }
        }
    }

    fn au_dela_du_mur(e: &Exploration) -> usize { e.cases().filter(|(c, _)| c.x > 5).count() }

    #[test]
    fn un_mur_cache_la_salle_voisine() {
        // CA (Flo, 2026-11-27) : la carte montre « ce que mon personnage a exploré ».
        let mut e = Exploration::default();
        assert!(e.explorer(Vec3::new(2, 2, 2), deux_salles(false), &reglages()));
        assert!(e.connait(Vec3::new(2, 2, -1), 1), "le sol de ma salle");
        assert!(e.connait(Vec3::new(5, 2, 2), 1), "le mur, vu de mon côté");
        assert_eq!(au_dela_du_mur(&e), 0, "rien de la salle d'à côté");
    }

    #[test]
    fn une_porte_ouverte_montre_la_salle_voisine() {
        let mut e = Exploration::default();
        e.explorer(Vec3::new(2, 2, 2), deux_salles(true), &reglages());
        assert!(e.connait(Vec3::new(8, 2, -1), 1), "le sol de l'autre salle");
    }

    #[test]
    fn on_ne_decouvre_pas_plus_loin_que_le_rayon() {
        // Un couloir droit de 200 blocs ; rayon 10.
        let couloir = |p: Vec3<i32>| {
            if (0..200).contains(&p.x) && p.y == 0 && p.z == 0 {
                Bloc::Air
            } else {
                Bloc::Plein(PIERRE)
            }
        };
        let r = Reglages { rayon: 10, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(50, 0, 0), couloir, &r);
        assert!(e.connait(Vec3::new(58, 0, -1), 1));
        assert!(!e.connait(Vec3::new(62, 0, -1), 1));
        assert!(!e.connait(Vec3::new(38, 0, -1), 1));
    }

    #[test]
    fn la_carte_ne_retrecit_jamais() {
        // On garde ce qu'on a vu en s'éloignant.
        let monde = deux_salles(true);
        let r = Reglages { rayon: 4, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(1, 2, 2), &monde, &r);
        let avant: Vec<_> = e.cases().collect();
        assert!(!avant.is_empty());
        assert!(e.explorer(Vec3::new(9, 2, 2), &monde, &r));
        for (case, _) in avant {
            assert!(e.connait(case, 1));
        }
        // Rien de nouveau au même endroit.
        assert!(!e.explorer(Vec3::new(9, 2, 2), &monde, &r));
    }

    #[test]
    fn meme_endroit_meme_carte() {
        let mut a = Exploration::default();
        let mut b = Exploration::default();
        a.explorer(Vec3::new(2, 2, 2), deux_salles(true), &reglages());
        b.explorer(Vec3::new(2, 2, 2), deux_salles(true), &reglages());
        assert!(!a.is_empty());
        assert_eq!(a, b);
    }

    #[test]
    fn le_plafond_de_cases_est_tenu() {
        let r = Reglages { max_cases: 20, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(2, 2, 2), deux_salles(true), &r);
        assert_eq!(e.len(), 20);
    }

    #[test]
    fn le_terrain_pas_charge_arrete_la_decouverte() {
        // Tout ce qui est en x >= 3 n'est pas chargé : on ne le traverse pas et on ne le dessine pas.
        let monde = deux_salles(true);
        let coupe = |p: Vec3<i32>| if p.x >= 3 { Bloc::Inconnu } else { monde(p) };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(1, 2, 2), coupe, &reglages());
        assert!(!e.is_empty());
        assert_eq!(e.cases().filter(|(c, _)| c.x >= 3).count(), 0);
    }

    #[test]
    fn une_case_regroupe_les_blocs_de_la_maille() {
        let r = Reglages { maille: 2, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(2, 2, 2), deux_salles(false), &r);
        // Le sol (z = -1) tombe dans les cases z = -1 ; 5 blocs de large → 3 cases.
        assert_eq!(Exploration::case_de(Vec3::new(3, 3, -1), 2), Vec3::new(1, 1, -1));
        let sol = e.cases().filter(|(c, _)| c.z == -1).count();
        assert_eq!(sol, 9);
        assert_eq!(e.cases().next().map(|(_, col)| col), Some(PIERRE));
    }

    #[test]
    fn depuis_un_bloc_plein_on_ne_decouvre_rien() {
        let mut e = Exploration::default();
        assert!(!e.explorer(Vec3::new(5, 0, 0), deux_salles(false), &reglages()));
        assert!(e.is_empty());
    }

    #[test]
    fn fichier_de_reglages_lisible() {
        let r = Ron::<Reglages>::load_expect("icebreaker.carte").read().0.clone();
        assert!(r.verifier().is_ok());
        assert!(Reglages { maille: 0, ..Reglages::default() }.verifier().is_err());
    }
}
