//! Icebreaker DF : la difficulté d'une expédition, de 1 à 6 (`assets/icebreaker/difficulte.ron`,
//! `docs/icebreaker/design/difficulte.md`). Une ligne par difficulté : nombre d'ennemis, PV, dégâts,
//! habileté et classes. Elle vaut pour les trois lieux (donjon généré, grotte, forêt).

use crate::assets::{AssetExt, Ron};
use hashbrown::HashMap;
use serde::Deserialize;

/// Une difficulté : ce qu'elle change aux ennemis du lieu.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct Niveau {
    /// Facteur du nombre d'ennemis (le chef et le boss restent uniques).
    pub nombre: f32,
    /// Facteur des PV de la fiche de l'ennemi.
    pub pv: f32,
    /// Facteur des dégâts que l'ennemi inflige.
    pub degats: f32,
    /// Cran d'habileté : une clé de `crans`.
    pub ia: String,
    /// Poids de chaque rôle quand le lieu tire un ennemi (vide : le tirage du lieu).
    pub classes: HashMap<String, f32>,
    /// Son numéro, de 1 à 6 (0 : aucune, sans fichier) ; posé par `IcebreakerDifficulte::niveau`.
    #[serde(skip)]
    pub numero: u8,
    /// Les compétences de son cran, par rôle ; posées par `IcebreakerDifficulte::niveau`.
    #[serde(skip)]
    pub competences: HashMap<String, String>,
}

impl Default for Niveau {
    /// La difficulté qui ne change rien.
    fn default() -> Self {
        Self {
            nombre: 1.0,
            pv: 1.0,
            degats: 1.0,
            ia: String::new(),
            classes: HashMap::new(),
            numero: 0,
            competences: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IcebreakerDifficulte {
    /// La difficulté d'une expédition qui n'en demande pas.
    pub defaut: u8,
    /// Les difficultés, de 1 à `niveaux.len()`.
    pub niveaux: Vec<Niveau>,
    /// Par cran d'habileté et par rôle : les compétences qui remplacent celles de la fiche.
    pub crans: HashMap<String, HashMap<String, String>>,
}

/// Le rôle d'une fiche d'ennemi : son nom de fichier (`icebreaker.ennemis.guerrier` → `guerrier`).
pub fn role(asset: &str) -> &str { asset.rsplit('.').next().unwrap_or(asset) }

impl IcebreakerDifficulte {
    /// Réglages lus dans `assets/icebreaker/difficulte.ron` (rien si le fichier manque : aucune
    /// difficulté ne change quoi que ce soit).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.difficulte").map_or_else(|_| Self::default(), |r| r.read().0.clone())
    }

    /// La plus haute difficulté (6).
    pub fn max(&self) -> u8 { self.niveaux.len().min(u8::MAX as usize) as u8 }

    /// Une difficulté demandée, ramenée entre 1 et `max` ; rien : celle par défaut.
    pub fn borne(&self, demandee: Option<u8>) -> u8 { demandee.unwrap_or(self.defaut).clamp(1, self.max().max(1)) }

    /// La ligne d'une difficulté (rien : celle par défaut), avec son numéro et les compétences de
    /// son cran ; sans fichier, celle qui ne change rien.
    pub fn niveau(&self, demandee: Option<u8>) -> Niveau {
        let numero = self.borne(demandee);
        let Some(ligne) = self.niveaux.get(numero as usize - 1) else {
            return Niveau::default();
        };
        Niveau {
            numero,
            competences: self.crans.get(&ligne.ia).cloned().unwrap_or_default(),
            ..ligne.clone()
        }
    }
}

/// La difficulté demandée par le lanceur pour un lieu lancé seul (`-Difficulte`, variable
/// `ICEBREAKER_DIFFICULTE`).
pub fn du_lanceur() -> Option<u8> { std::env::var("ICEBREAKER_DIFFICULTE").ok()?.trim().parse().ok() }

/// La ligne de la difficulté `voulue` par l'expédition ; sinon celle du lanceur, sinon celle par
/// défaut.
pub fn niveau(voulue: Option<u8>) -> Niveau { IcebreakerDifficulte::actuel().niveau(voulue.or_else(du_lanceur)) }

impl Niveau {
    /// Les compétences que son cran donne à l'ennemi de la fiche `asset` (rien : celles de sa fiche).
    pub fn competences(&self, asset: &str) -> Option<&str> { self.competences.get(role(asset)).map(String::as_str) }

    /// Le poids du rôle de la fiche `asset` dans un tirage (1 si la difficulté ne pèse pas les
    /// rôles, ou si elle ne connaît pas celui-ci : un monstre, un chef).
    pub fn poids(&self, asset: &str) -> f32 { self.classes.get(role(asset)).copied().unwrap_or(1.0).max(0.0) }

    /// Tire une fiche parmi `fiches` selon les poids des rôles ; `hasard` est tiré entre 0 et 1.
    /// Si tous les poids sont nuls (un lieu qui n'a que des mages, en difficulté 1), le tirage du
    /// lieu reste : mieux vaut un mage qu'une salle vide.
    pub fn tirer<'a, S: AsRef<str>>(&self, fiches: &'a [S], hasard: f32) -> Option<&'a S> {
        let total: f32 = fiches.iter().map(|f| self.poids(f.as_ref())).sum();
        if total <= 0.0 {
            return fiches.get(((hasard * fiches.len() as f32) as usize).min(fiches.len().saturating_sub(1)));
        }
        let mut reste = hasard.clamp(0.0, 1.0) * total;
        let mut dernier = None;
        for fiche in fiches {
            let poids = self.poids(fiche.as_ref());
            if poids <= 0.0 {
                continue;
            }
            dernier = Some(fiche);
            if reste < poids {
                break;
            }
            reste -= poids;
        }
        dernier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetExt;

    const ROLES: [&str; 3] = ["guerrier", "archer", "mage"];

    fn fichier() -> IcebreakerDifficulte { Ron::<IcebreakerDifficulte>::load_expect("icebreaker.difficulte").read().0.clone() }

    #[test]
    fn six_difficultes_croissantes() {
        let d = fichier();
        assert_eq!(d.max(), 6);
        let lignes: Vec<Niveau> = (1..=6).map(|n| d.niveau(Some(n))).collect();
        for paire in lignes.windows(2) {
            assert!(paire[0].nombre < paire[1].nombre && paire[0].pv < paire[1].pv && paire[0].degats < paire[1].degats);
        }
    }

    #[test]
    fn la_difficulte_par_defaut_ne_change_rien() {
        let d = fichier();
        assert_eq!(d.defaut, 3);
        let n = d.niveau(None);
        assert_eq!((n.nombre, n.pv, n.degats), (1.0, 1.0, 1.0));
        assert!(n.classes.is_empty());
        for role in ROLES {
            assert_eq!(n.competences(&format!("icebreaker.ennemis.{role}")), None);
        }
    }

    #[test]
    fn une_difficulte_hors_bornes_est_ramenee() {
        let d = fichier();
        assert_eq!((d.borne(Some(0)), d.borne(Some(9)), d.borne(None)), (1, 6, 3));
        // Sans fichier : rien ne change, rien ne panique.
        let vide = IcebreakerDifficulte::default();
        assert_eq!(vide.niveau(Some(4)), Niveau::default());
    }

    #[test]
    fn crans_et_roles_connus() {
        let d = fichier();
        for n in 1..=6 {
            let niveau = d.niveau(Some(n));
            let cran = d.crans.get(&niveau.ia).unwrap_or_else(|| panic!("cran inconnu en difficulté {n} : {}", niveau.ia));
            for (role, competences) in cran {
                assert!(ROLES.contains(&role.as_str()), "rôle inconnu : {role}");
                crate::skillset_builder::SkillSetBuilder::default().with_asset_expect(competences);
            }
            for role in niveau.classes.keys() {
                assert!(ROLES.contains(&role.as_str()), "rôle inconnu en difficulté {n} : {role}");
            }
        }
        // Le chef garde ses compétences à toutes les difficultés.
        assert_eq!(d.niveau(Some(6)).competences("icebreaker.ennemis.chef"), None);
        assert_eq!(d.niveau(Some(6)).competences("icebreaker.ennemis.mage"), Some("common.skillset.preset.rank5.staff"));
        assert_eq!(d.niveau(Some(6)).numero, 6);
    }

    #[test]
    fn pas_de_mage_en_difficulte_1() {
        let d = fichier();
        let fiches = ["icebreaker.ennemis.archer", "icebreaker.ennemis.mage"];
        let un = d.niveau(Some(1));
        let deux = d.niveau(Some(2));
        let mut mages = 0;
        for i in 0..100 {
            let hasard = i as f32 / 100.0;
            assert_eq!(un.tirer(&fiches, hasard), Some(&fiches[0]));
            mages += (deux.tirer(&fiches, hasard) == Some(&fiches[1])) as i32;
        }
        // Archer 2, mage 1 : un tiers de mages.
        assert!((30..=36).contains(&mages), "{mages}");
        // Un lieu qui n'a que des mages en garde.
        assert_eq!(un.tirer(&fiches[1..], 0.5), Some(&fiches[1]));
        // Un monstre n'est pas pesé.
        assert_eq!(un.poids("common.entity.wild.aggressive.cave_spider"), 1.0);
        assert!(un.tirer::<&str>(&[], 0.5).is_none());
    }
}
