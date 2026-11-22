//! Icebreaker F.33: the shapes of the generated dungeon (`assets/icebreaker/donjons/formes.ron`), as the
//! profiles of `grottes/formes.ron` for the cave. A profile changes some settings of `generation.ron` (the
//! graph of every size, the spacing of the rooms, the depth); those it leaves out keep their value. The seed
//! draws one by weight, or the party chooses one (`/expedition N <profil>`, `-Profil` of the launcher); the
//! dungeon workshop (`.\icebreaker.ps1 donjon-atelier`) writes them.

use super::settings::{Range, Settings};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a profile changes (`None`: the value of `generation.ron`).
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Forme {
    /// The graph, for every size (`sizes` of `generation.ron`).
    pub main_path: Option<Range>,
    pub levels: Option<Range>,
    pub branches: Option<Range>,
    pub branch_length: Option<Range>,
    pub locks: Option<Range>,
    pub loops: Option<Range>,
    pub drops: Option<Range>,
    /// Cells between a door and the next room (`assembly.spacing`).
    pub spacing: Option<Range>,
    /// Blocks of rock at least above the dungeon (`placement.profondeur`).
    pub profondeur: Option<i32>,
}

impl Forme {
    /// `settings` changed by the profile.
    pub fn appliquer(&self, mut settings: Settings) -> Settings {
        for shape in settings.sizes.values_mut() {
            let set = |field: &mut Range, value: Option<Range>| {
                if let Some(value) = value {
                    *field = value;
                }
            };
            set(&mut shape.main_path, self.main_path);
            set(&mut shape.levels, self.levels);
            set(&mut shape.branches, self.branches);
            set(&mut shape.branch_length, self.branch_length);
            set(&mut shape.locks, self.locks);
            set(&mut shape.loops, self.loops);
            set(&mut shape.drops, self.drops);
        }
        if let Some(spacing) = self.spacing {
            settings.assembly.spacing = spacing;
        }
        if let Some(profondeur) = self.profondeur {
            settings.placement.profondeur = profondeur;
        }
        settings
    }

    /// What is wrong with the profile (empty: nothing).
    pub fn erreurs(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let ranges = [
            ("main_path", self.main_path, 2),
            ("levels", self.levels, 1),
            ("branches", self.branches, 0),
            ("branch_length", self.branch_length, 1),
            ("locks", self.locks, 0),
            ("loops", self.loops, 0),
            ("drops", self.drops, 0),
            ("spacing", self.spacing, 1),
        ];
        for (name, range, lowest) in ranges {
            if range.is_some_and(|r| !r.valid(lowest)) {
                errors.push(format!("{name} : plage invalide (au moins {lowest}, min ≤ max)"));
            }
        }
        if self.profondeur.is_some_and(|p| p < 1) {
            errors.push("profondeur : au moins 1".into());
        }
        errors
    }
}

/// A profile: its weight when the seed draws (0: only chosen), and what it changes.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profil {
    pub poids: f64,
    pub forme: Forme,
}

/// `formes.ron`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Formes {
    pub profils: BTreeMap<String, Profil>,
}

/// The file of the profiles, under `assets/icebreaker/donjons/`.
pub const FICHIER: &str = "formes.ron";

impl Formes {
    /// The profiles (none if the file is absent).
    pub fn load() -> Result<Self, String> {
        if !super::data_path(FICHIER).exists() {
            return Ok(Self::default());
        }
        Self::parse(&super::read_data(FICHIER)?)
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let formes: Self = ron::from_str(text).map_err(|e| format!("{FICHIER} : {e}"))?;
        let errors: Vec<String> = formes
            .profils
            .iter()
            .flat_map(|(nom, p)| {
                let poids = (p.poids < 0.0).then(|| format!("{nom} : poids négatif"));
                poids.into_iter().chain(p.forme.erreurs().into_iter().map(move |e| format!("{nom} : {e}")))
            })
            .collect();
        if errors.is_empty() { Ok(formes) } else { Err(format!("{FICHIER} : {}", errors.join(" ; "))) }
    }

    /// The profile of a dungeon: the one asked for, or one drawn by the seed among those of positive weight
    /// (none if there is no such profile).
    pub fn tirer(&self, graine: u64, voulu: Option<&str>) -> Option<(&str, &Forme)> {
        if let Some((nom, profil)) = voulu.and_then(|v| self.profils.get_key_value(v)) {
            return Some((nom, &profil.forme));
        }
        let total: f64 = self.profils.values().map(|p| p.poids.max(0.0)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut tirage = ChaCha8Rng::seed_from_u64(graine ^ 0xF0E5).random_range(0.0..total);
        for (nom, profil) in self.profils.iter().filter(|(_, p)| p.poids > 0.0) {
            tirage -= profil.poids;
            if tirage < 0.0 {
                return Some((nom, &profil.forme));
            }
        }
        self.profils.iter().rfind(|(_, p)| p.poids > 0.0).map(|(nom, p)| (nom.as_str(), &p.forme))
    }
}

impl Forme {
    /// Icebreaker F.34: the lines of the profile in `formes.ron` (what it changes only).
    fn bloc(&self, nom: &str, poids: f64) -> String {
        let plage = |name: &str, r: Option<Range>| r.map(|r| format!("{name}: ({}, {})", r.0, r.1));
        let champs: Vec<String> = [
            plage("main_path", self.main_path),
            plage("levels", self.levels),
            plage("branches", self.branches),
            plage("branch_length", self.branch_length),
            plage("locks", self.locks),
            plage("loops", self.loops),
            plage("drops", self.drops),
            plage("spacing", self.spacing),
            self.profondeur.map(|p| format!("profondeur: {p}")),
        ]
        .into_iter()
        .flatten()
        .collect();
        format!(
            "        \"{nom}\": (\n            poids: {poids:?},\n            forme: ({}),\n        ),\n",
            champs.join(", ")
        )
    }
}

/// Icebreaker F.34: `formes.ron` with the profile `nom` added, or replaced if it is there (the comments of the file
/// kept), for the dungeon workshop.
pub fn poser_profil(texte: &str, nom: &str, poids: f64, forme: &Forme) -> Result<String, String> {
    icebreaker_caverne::presets::nom_valide(nom)?;
    if !(0.0..=100.0).contains(&poids) {
        return Err(format!("poids refusé : {poids} (0 à 100)"));
    }
    let erreurs = forme.erreurs();
    if !erreurs.is_empty() {
        return Err(erreurs.join(" ; "));
    }
    let sortie = icebreaker_caverne::presets::poser_entree(texte, nom, &forme.bloc(nom, poids))?;
    let formes = Formes::parse(&sortie)?;
    if formes.profils.get(nom).is_none_or(|p| p.forme != *forme || p.poids != poids) {
        return Err(format!("le preset « {nom} » n'a pas été écrit comme prévu"));
    }
    if formes.tirer(0, None).is_none() {
        return Err("il faut au moins un preset de poids non nul (tiré par la graine)".into());
    }
    Ok(if texte.contains("\r\n") { sortie.replace('\n', "\r\n") } else { sortie })
}

/// Icebreaker F.34: `formes.ron` without the profile `nom` (nor the comment over it).
pub fn retirer_profil(texte: &str, nom: &str) -> Result<String, String> {
    let sortie = icebreaker_caverne::presets::retirer_entree(texte, nom)?;
    if Formes::parse(&sortie)?.tirer(0, None).is_none() {
        return Err("il faut au moins un preset de poids non nul (tiré par la graine)".into());
    }
    Ok(if texte.contains("\r\n") { sortie.replace('\n', "\r\n") } else { sortie })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_profiles_are_valid_and_change_the_settings() {
        let formes = Formes::load().unwrap_or_else(|e| panic!("{e}"));
        assert!(!formes.profils.is_empty());
        let settings = Settings::load().unwrap();
        for (nom, profil) in &formes.profils {
            let changed = profil.forme.appliquer(settings.clone());
            if let Some(levels) = profil.forme.levels {
                assert!(changed.sizes.values().all(|s| s.levels == levels), "{nom}");
            }
            if let Some(p) = profil.forme.profondeur {
                assert_eq!(changed.placement.profondeur, p, "{nom}");
            }
        }
        // Asked for: that one; otherwise drawn among those of positive weight.
        let (premier, _) = formes.profils.iter().next().unwrap();
        assert_eq!(formes.tirer(7, Some(premier)).unwrap().0, premier);
        for graine in 0..50 {
            if let Some((nom, _)) = formes.tirer(graine, None) {
                assert!(formes.profils[nom].poids > 0.0, "{nom} tiré sans poids");
            }
        }
    }

    /// Icebreaker F.34: the workshop adds a preset to the real file, replaces it, and removing it gives the file back.
    #[test]
    fn the_real_file_takes_a_preset() {
        let texte = std::fs::read_to_string(super::super::data_path(FICHIER)).unwrap();
        let forme = Forme { levels: Some(Range(2, 2)), profondeur: Some(30), ..Forme::default() };
        let plus = poser_profil(&texte, "essai", 0.5, &forme).unwrap();
        let formes = Formes::parse(&plus).unwrap();
        assert_eq!(formes.profils["essai"].forme, forme);
        assert_eq!(formes.profils.len(), Formes::parse(&texte).unwrap().profils.len() + 1);
        let autre = Forme { loops: Some(Range(0, 0)), ..Forme::default() };
        let remplace = poser_profil(&plus, "essai", 0.0, &autre).unwrap();
        assert_eq!(Formes::parse(&remplace).unwrap().profils["essai"].forme, autre);
        assert_eq!(retirer_profil(&remplace, "essai").unwrap(), texte);
        assert!(poser_profil(&texte, "Pas Bon", 1.0, &forme).is_err());
        assert!(poser_profil(&texte, "x", 1.0, &Forme { levels: Some(Range(3, 1)), ..Forme::default() }).is_err());
    }

    #[test]
    fn a_bad_profile_is_refused() {
        assert!(Formes::parse("(profils: {\"x\": (poids: 1.0, forme: (levels: Some((3, 1))))})").is_err());
        assert!(Formes::parse("(profils: {\"x\": (poids: 1.0, forme: (etages: Some((1, 1))))})").is_err());
        let ok = Formes::parse("(profils: {\"x\": (poids: 0.0, forme: (levels: Some((1, 1))))})").unwrap();
        assert!(ok.tirer(1, None).is_none());
    }
}
