//! Icebreaker : les minerais des filons de la grotte (U.18, `assets/icebreaker/grottes/minerais.ron`,
//! `docs/icebreaker/design/filons.md`). Un filon est une bande de blocs de couleur prise dans la
//! roche ; le jeu reconnaît le minerai d'un bloc à sa couleur exacte. Le générateur de la grotte
//! (`world::icebreaker::grotte`) les pose, le serveur les casse à la pioche (dureté, objet donné).

use crate::{
    assets::{AssetExt, Ron},
    terrain::{Block, BlockKind, SpriteKind},
};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};
use vek::Rgb;

/// La forme d'un filon.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct FormeFilon {
    /// Blocs parcourus le long de la paroi (min, max).
    pub longueur: (i32, i32),
    /// Largeur de la bande (min, max).
    pub epaisseur: (i32, i32),
    /// Blocs pris dans la roche sous la surface.
    pub profondeur: i32,
    /// De 0 (droit) à 1 (tourne sans cesse).
    pub sinuosite: f32,
    /// Part des blocs de surface qui ressortent d'un bloc dans le vide.
    pub relief: f32,
}

/// Un minerai.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Minerai {
    pub couleur: (u8, u8, u8),
    /// Ses blocs luisent (roche luisante).
    pub lueur: bool,
    /// Coups de pioche pour qu'un bloc cède, énergie payée à chaque coup.
    pub coups: u8,
    pub energie: f32,
    /// L'objet que donne un bloc qui cède, et combien (min, max).
    pub objet: String,
    pub quantite: (u32, u32),
    /// Distance autour du bloc visé où les blocs du même minerai cèdent avec lui.
    pub morceau: f32,
    pub filon: FormeFilon,
    /// Poids au tirage d'un filon, par niveau de la grotte (le dernier vaut au-delà).
    pub poids: Vec<f32>,
    /// Poids au tirage des filons de la salle au minerai.
    pub salle: f32,
    /// Icebreaker U.22 : la pépite de Veloren (nom de sprite minable à la pioche) posée sur le dessus
    /// des blocs du filon, et la part de ces blocs qui en portent une (0 : aucune).
    #[serde(default)]
    pub pepite: Option<String>,
    #[serde(default)]
    pub pepites: f32,
}

impl Minerai {
    /// Un bloc de ce minerai.
    pub fn bloc(&self) -> Block {
        let (r, g, b) = self.couleur;
        let kind = if self.lueur { BlockKind::GlowingRock } else { BlockKind::Rock };
        Block::new(kind, Rgb::new(r, g, b))
    }

    /// Son poids au niveau `niveau`.
    pub fn poids_au(&self, niveau: i32) -> f32 {
        let i = (niveau.max(0) as usize).min(self.poids.len().saturating_sub(1));
        self.poids.get(i).copied().unwrap_or(0.0)
    }

    /// U.22 : le sprite de sa pépite, s'il en a une.
    pub fn sprite_pepite(&self) -> Option<SpriteKind> { ron::from_str(self.pepite.as_deref()?).ok() }

    /// La destruction (`icebreaker_destruction`) d'un bloc de ce minerai.
    pub fn matiere(&self) -> crate::icebreaker_destruction::Matiere {
        crate::icebreaker_destruction::Matiere { coups: self.coups, energie: self.energie }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Fichier {
    minerais: BTreeMap<String, Minerai>,
}

/// Les minerais, dans l'ordre de leur nom ; un minerai se désigne par sa place dans la liste.
#[derive(Clone, Debug, Default)]
pub struct Minerais {
    liste: Vec<(String, Minerai)>,
}

impl Minerais {
    fn depuis(fichier: Fichier) -> Result<Self, String> {
        let mut erreurs = Vec::new();
        let mut couleurs = BTreeMap::new();
        for (nom, m) in &fichier.minerais {
            let f = &m.filon;
            let mut faux = |quoi: &str| erreurs.push(format!("{nom} : {quoi}"));
            if m.coups == 0 || m.energie < 0.0 {
                faux("au moins un coup, une énergie positive ou nulle");
            }
            if m.objet.is_empty() || m.quantite.1 < m.quantite.0 {
                faux("un objet, et une quantité (min, max) avec min ≤ max");
            }
            if m.morceau < 0.0 {
                faux("morceau positif ou nul");
            }
            if let Some(nom) = &m.pepite
                && m.sprite_pepite().and_then(|s| s.mine_tool()) != Some(crate::comp::tool::ToolKind::Pick)
            {
                faux(&format!("pépite « {nom} » : pas un sprite qui se mine à la pioche"));
            }
            if !(0.0..=1.0).contains(&m.pepites) {
                faux("pepites entre 0 et 1");
            }
            if f.longueur.0 < 1 || f.longueur.1 < f.longueur.0 || f.epaisseur.0 < 1 || f.epaisseur.1 < f.epaisseur.0 {
                faux("filon : longueur et epaisseur (min, max) avec 1 ≤ min ≤ max");
            }
            if f.profondeur < 1 || !(0.0..=1.0).contains(&f.sinuosite) || !(0.0..=1.0).contains(&f.relief) {
                faux("filon : profondeur ≥ 1, sinuosite et relief entre 0 et 1");
            }
            if m.poids.is_empty() || m.poids.iter().chain([&m.salle]).any(|p| *p < 0.0) {
                faux("poids : au moins une valeur ; poids et salle positifs ou nuls");
            }
            if let Some(autre) = couleurs.insert(m.couleur, nom) {
                erreurs.push(format!("{nom} et {autre} ont la même couleur"));
            }
        }
        if fichier.minerais.is_empty() {
            erreurs.push("au moins un minerai".into());
        }
        if !fichier.minerais.values().any(|m| m.poids.iter().any(|p| *p > 0.0)) {
            erreurs.push("aucun minerai n'a de poids dans les zones".into());
        }
        if !fichier.minerais.values().any(|m| m.salle > 0.0) {
            erreurs.push("aucun minerai n'a de poids dans la salle".into());
        }
        if erreurs.is_empty() {
            Ok(Self { liste: fichier.minerais.into_iter().collect() })
        } else {
            Err(erreurs.join(" ; "))
        }
    }

    /// Lit le texte d'un fichier comme `minerais.ron`.
    pub fn lire(texte: &str) -> Result<Self, String> {
        let fichier: Fichier = ron::from_str(texte).map_err(|e| format!("minerais.ron illisible : {e}"))?;
        Self::depuis(fichier)
    }

    pub fn liste(&self) -> &[(String, Minerai)] { &self.liste }

    pub fn get(&self, i: usize) -> Option<&Minerai> { self.liste.get(i).map(|(_, m)| m) }

    /// Le minerai d'un bloc, s'il en est un (sa place et lui).
    pub fn de_bloc(&self, block: &Block) -> Option<(usize, &Minerai)> {
        if !matches!(block.kind(), BlockKind::Rock | BlockKind::GlowingRock) {
            return None;
        }
        let c = block.get_color()?;
        self.liste
            .iter()
            .enumerate()
            .find(|(_, (_, m))| m.couleur == (c.r, c.g, c.b) && m.bloc().kind() == block.kind())
            .map(|(i, (_, m))| (i, m))
    }

    /// Tire un minerai selon les poids `poids` (un par minerai) ; `tirage` va de 0 à 1.
    fn tirer_par(&self, tirage: f32, poids: impl Fn(&Minerai) -> f32) -> Option<usize> {
        let total: f32 = self.liste.iter().map(|(_, m)| poids(m)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut reste = tirage.clamp(0.0, 1.0) * total;
        let mut dernier = None;
        for (i, (_, m)) in self.liste.iter().enumerate() {
            let p = poids(m);
            if p <= 0.0 {
                continue;
            }
            if reste < p {
                return Some(i);
            }
            reste -= p;
            dernier = Some(i);
        }
        dernier
    }

    /// Le minerai d'un filon des zones au niveau `niveau`.
    pub fn tirer(&self, niveau: i32, tirage: f32) -> Option<usize> { self.tirer_par(tirage, |m| m.poids_au(niveau)) }

    /// Le minerai d'un filon de la salle au minerai.
    pub fn tirer_salle(&self, tirage: f32) -> Option<usize> { self.tirer_par(tirage, |m| m.salle) }
}

/// Les minerais du fichier ; s'il est illisible, aucun (pas de filon, rien à reconnaître).
pub fn minerais() -> &'static Minerais {
    static MINERAIS: LazyLock<Minerais> = LazyLock::new(|| {
        Ron::<Fichier>::load("icebreaker.grottes.minerais")
            .map_err(|e| format!("{e:?}"))
            .and_then(|f| Minerais::depuis(f.read().0.clone()))
            .unwrap_or_else(|e| {
                tracing::error!("Icebreaker : assets/icebreaker/grottes/minerais.ron illisible ({e})");
                Minerais::default()
            })
    });
    &MINERAIS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fichier() -> Minerais {
        let fichier = Ron::<Fichier>::load_expect("icebreaker.grottes.minerais").read().0.clone();
        Minerais::depuis(fichier).expect("minerais.ron")
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let m = fichier();
        assert!(m.liste().len() >= 5);
        // Chaque minerai donne un objet qui existe.
        for (nom, minerai) in m.liste() {
            crate::comp::Item::new_from_asset(&minerai.objet).unwrap_or_else(|e| panic!("{nom} : {e:?}"));
            // U.22 : chacun a sa pépite, qui donne le même minerai que son filon.
            let pepite = minerai.sprite_pepite().unwrap_or_else(|| panic!("{nom} : pas de pépite"));
            assert!(minerai.pepites > 0.0, "{nom}");
            let donne = crate::comp::Item::try_reclaim_from_block(Block::air(pepite), None)
                .unwrap_or_else(|| panic!("{nom} : la pépite ne donne rien"));
            assert!(
                donne.iter().any(|(_, item)| item.item_definition_id().itemdef_id() == Some(minerai.objet.as_str())),
                "{nom} : la pépite ne donne pas {}",
                minerai.objet
            );
        }
    }

    #[test]
    fn un_bloc_de_minerai_se_reconnait_a_sa_couleur() {
        let m = fichier();
        for (i, (nom, minerai)) in m.liste().iter().enumerate() {
            assert_eq!(m.de_bloc(&minerai.bloc()).map(|(j, _)| j), Some(i), "{nom}");
        }
        // La roche ordinaire, et un sprite, ne sont pas du minerai.
        assert!(m.de_bloc(&Block::new(BlockKind::Rock, Rgb::new(80, 80, 95))).is_none());
        assert!(m.de_bloc(&Block::air(crate::terrain::SpriteKind::Iron)).is_none());
        // La bonne couleur sur la mauvaise matière non plus.
        let (r, g, b) = m.liste()[0].1.couleur;
        assert!(m.de_bloc(&Block::new(BlockKind::Wood, Rgb::new(r, g, b))).is_none());
    }

    /// Plus on descend, plus c'est rare et cher : l'or n'est qu'au fond et coûte plus d'énergie que le cuivre.
    #[test]
    fn les_filons_sont_plus_riches_au_fond() {
        let m = fichier();
        let trouves = |niveau: i32| -> std::collections::BTreeSet<&str> {
            (0..100).filter_map(|k| m.tirer(niveau, k as f32 / 100.0)).map(|i| m.liste()[i].0.as_str()).collect()
        };
        let (haut, fond) = (trouves(0), trouves(2));
        assert!(haut.contains("cuivre") && !haut.contains("or"), "{haut:?}");
        assert!(fond.contains("or"), "{fond:?}");
        // Au-delà du dernier niveau, le dernier poids.
        assert_eq!(trouves(9), fond);
        let par_nom = |n: &str| &m.liste().iter().find(|(nom, _)| nom == n).unwrap().1;
        assert!(par_nom("or").energie > par_nom("cuivre").energie);
        assert!(m.tirer_salle(0.0).is_some() && m.tirer_salle(0.999).is_some());
    }

    #[test]
    fn un_fichier_faux_est_refuse() {
        let bon = r#"(minerais: { "a": (couleur: (1, 2, 3), lueur: false, coups: 2, energie: 5.0, objet: "x",
            quantite: (1, 1), morceau: 0.0, filon: (longueur: (4, 6), epaisseur: (1, 2), profondeur: 1,
            sinuosite: 0.5, relief: 0.2), poids: [1.0], salle: 1.0) })"#;
        assert!(Minerais::lire(bon).is_ok());
        for (de, vers) in [
            ("coups: 2", "coups: 0"),
            ("quantite: (1, 1)", "quantite: (2, 1)"),
            ("sinuosite: 0.5", "sinuosite: 1.5"),
            ("epaisseur: (1, 2)", "epaisseur: (0, 2)"),
            ("poids: [1.0]", "poids: []"),
            ("salle: 1.0) }", "salle: 1.0, pepite: Some(\"Bones\"), pepites: 0.5) }"),
            ("salle: 1.0) }", "salle: 1.0, pepite: Some(\"Copper\"), pepites: 1.5) }"),
        ] {
            assert!(Minerais::lire(&bon.replace(de, vers)).is_err(), "{vers}");
        }
        let deux = bon.replace(r#"{ "a": "#, r#"{ "b": (couleur: (1, 2, 3), lueur: false, coups: 2, energie: 5.0,
            objet: "x", quantite: (1, 1), morceau: 0.0, filon: (longueur: (4, 6), epaisseur: (1, 2), profondeur: 1,
            sinuosite: 0.5, relief: 0.2), poids: [1.0], salle: 1.0), "a": "#);
        assert!(Minerais::lire(&deux).unwrap_err().contains("même couleur"));
    }
}
