//! Icebreaker F.33: the shapes of the generated dungeon (`assets/icebreaker/donjons/formes.ron`), as the
//! profiles of `grottes/formes.ron` for the cave. A profile changes some settings of `generation.ron` (the
//! graph of every size, the spacing of the rooms, the depth); those it leaves out keep their value. The seed
//! draws one by weight, or the party chooses one (`/expedition N <profil>`, `-Profil` of the launcher); the
//! dungeon workshop (`.\icebreaker.ps1 donjon-atelier`) writes them.

use super::{
    settings::{Range, Settings},
    silhouette::Silhouette,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Icebreaker F.45: how a setting of a profile is written and shown.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Genre {
    /// `(min, max)`, drawn by the seed.
    Plage,
    Nombre,
    /// One of the names of `Champ::choix`.
    Choix,
}

/// Icebreaker F.45: a setting of a profile: its name in `formes.ron`, what the workshop calls it and the group it
/// shows it in, the values it may take (a range: both ends within them).
#[derive(Copy, Clone, Debug, Serialize)]
pub struct Champ {
    pub cle: &'static str,
    pub libelle: &'static str,
    pub groupe: &'static str,
    pub min: i32,
    pub max: i32,
    pub genre: Genre,
    /// Icebreaker F.46 : the names to choose from (`Genre::Choix`).
    pub choix: &'static [&'static str],
}

/// The value of a setting a profile fixes.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Valeur {
    Plage(Range),
    Nombre(i32),
    Choix(&'static str),
}

/// What a setting may be.
trait Reglage: Copy {
    const CHOIX: &'static [&'static str] = &[];
    fn valeur(self) -> Valeur;
}

impl Reglage for Range {
    fn valeur(self) -> Valeur { Valeur::Plage(self) }
}

impl Reglage for i32 {
    fn valeur(self) -> Valeur { Valeur::Nombre(self) }
}

impl Reglage for Silhouette {
    const CHOIX: &'static [&'static str] = Silhouette::NOMS;

    fn valeur(self) -> Valeur { Valeur::Choix(self.nom()) }
}

/// Icebreaker F.45: the settings of a profile, said once: the fields of `Forme`, the table `CHAMPS` the checks,
/// the writing of `formes.ron` and the sliders of the workshop come from (as `forme!` of `icebreaker-caverne`).
/// What a setting changes is in `Forme::appliquer` and `Forme::lire`.
macro_rules! forme {
    ($( $(#[$doc:meta])* $cle:ident : $genre:ident($ty:ty) [$min:expr, $max:expr] $groupe:literal $libelle:literal; )*) => {
        /// What a profile changes (`None`: the value of `generation.ron`).
        #[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct Forme {
            $( $(#[$doc])* pub $cle: Option<$ty>, )*
        }

        /// The settings of a profile, in the order of the workshop.
        pub const CHAMPS: &[Champ] = &[
            $( Champ {
                cle: stringify!($cle),
                libelle: $libelle,
                groupe: $groupe,
                min: $min,
                max: $max,
                genre: Genre::$genre,
                choix: <$ty as Reglage>::CHOIX,
            }, )*
        ];

        impl Forme {
            /// What the profile fixes, in the order of `CHAMPS`.
            fn valeurs(&self) -> Vec<Option<Valeur>> { vec![ $( self.$cle.map(Reglage::valeur), )* ] }
        }
    };
}

forme! {
    /// The graph, for every size (`sizes` of `generation.ron`).
    main_path: Plage(Range) [2, 20] "Graphe" "Salles du chemin principal";
    levels: Plage(Range) [1, 8] "Graphe" "Étages";
    branches: Plage(Range) [0, 14] "Graphe" "Branches (culs-de-sac)";
    branch_length: Plage(Range) [1, 5] "Graphe" "Longueur des branches";
    locks: Plage(Range) [0, 4] "Graphe" "Portes à clé";
    loops: Plage(Range) [0, 5] "Graphe" "Boucles";
    drops: Plage(Range) [0, 4] "Graphe" "Chutes (sens unique)";
    /// Cells between a door and the next room (`assembly.spacing`).
    spacing: Plage(Range) [1, 8] "Salles" "Cases entre deux salles";
    /// Icebreaker F.48 : rooms against each other (`assembly.collees`), and the generated rooms of varied shapes:
    /// their share among the rooms of a role and their size, walls included (`variety.shapes`).
    collees: Nombre(i32) [0, 100] "Salles" "Salles collées, sans couloir (%)";
    formes_part: Nombre(i32) [0, 90] "Salles" "Salles à forme tirée (%)";
    salle_largeur: Plage(Range) [6, 16] "Salles" "Largeur des salles tirées (cases)";
    salle_profondeur: Plage(Range) [6, 16] "Salles" "Profondeur des salles tirées (cases)";
    /// Icebreaker F.46 : the outline of the plan and how far it reaches (`assembly.silhouette`, `etendue`).
    silhouette: Choix(Silhouette) [0, 0] "Silhouette" "Forme du plan";
    etendue: Nombre(i32) [12, 56] "Silhouette" "Étendue (cases du milieu au bord)";
    /// Icebreaker F.47 : how the corridors go (`assembly.sinuosite`, `impasses`).
    sinuosite: Nombre(i32) [0, 100] "Couloirs" "Sinuosité (0 droit, 100 labyrinthe)";
    impasses: Plage(Range) [0, 8] "Couloirs" "Impasses (couloirs sans issue)";
    /// Blocks of rock at least above the dungeon (`placement.profondeur`).
    profondeur: Nombre(i32) [1, 60] "Sous le sol" "Roche au-dessus (blocs)";
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
        if let Some(collees) = self.collees {
            settings.assembly.collees = collees;
        }
        if let Some(part) = self.formes_part {
            settings.variety.shapes.share = part as f64 / 100.0;
        }
        if let Some(largeur) = self.salle_largeur {
            settings.variety.shapes.width = largeur;
        }
        if let Some(profondeur) = self.salle_profondeur {
            settings.variety.shapes.depth = profondeur;
        }
        if let Some(silhouette) = self.silhouette {
            settings.assembly.silhouette = silhouette;
        }
        if let Some(etendue) = self.etendue {
            settings.assembly.etendue = etendue;
        }
        if let Some(sinuosite) = self.sinuosite {
            settings.assembly.sinuosite = sinuosite;
        }
        if let Some(impasses) = self.impasses {
            settings.assembly.impasses = impasses;
        }
        if let Some(profondeur) = self.profondeur {
            settings.placement.profondeur = profondeur;
        }
        settings
    }

    /// Icebreaker F.45: what `settings` give for a dungeon of size `taille` without a profile: every setting
    /// fixed at its value there (the unticked sliders of the workshop).
    pub fn lire(settings: &Settings, taille: &str) -> Option<Self> {
        let shape = settings.sizes.get(taille)?;
        Some(Self {
            main_path: Some(shape.main_path),
            levels: Some(shape.levels),
            branches: Some(shape.branches),
            branch_length: Some(shape.branch_length),
            locks: Some(shape.locks),
            loops: Some(shape.loops),
            drops: Some(shape.drops),
            spacing: Some(settings.assembly.spacing),
            collees: Some(settings.assembly.collees),
            formes_part: Some((settings.variety.shapes.share * 100.0).round() as i32),
            salle_largeur: Some(settings.variety.shapes.width),
            salle_profondeur: Some(settings.variety.shapes.depth),
            silhouette: Some(settings.assembly.silhouette),
            etendue: Some(settings.assembly.etendue),
            sinuosite: Some(settings.assembly.sinuosite),
            impasses: Some(settings.assembly.impasses),
            profondeur: Some(settings.placement.profondeur),
        })
    }

    /// What is wrong with the profile (empty: nothing).
    pub fn erreurs(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (champ, valeur) in CHAMPS.iter().zip(self.valeurs()) {
            let Champ { cle, min, max, .. } = champ;
            let dedans = |n: i32| (*min..=*max).contains(&n);
            match valeur {
                Some(Valeur::Plage(r)) if !(dedans(r.0) && dedans(r.1) && r.0 <= r.1) => {
                    errors.push(format!("{cle} : plage invalide (de {min} à {max}, min ≤ max)"));
                },
                Some(Valeur::Nombre(n)) if !dedans(n) => errors.push(format!("{cle} : de {min} à {max}")),
                _ => {},
            }
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
        let champs: Vec<String> = CHAMPS
            .iter()
            .zip(self.valeurs())
            .filter_map(|(champ, valeur)| {
                Some(match valeur? {
                    Valeur::Plage(r) => format!("{}: ({}, {})", champ.cle, r.0, r.1),
                    Valeur::Nombre(n) => format!("{}: {n}", champ.cle),
                    Valeur::Choix(nom) => format!("{}: {nom}", champ.cle),
                })
            })
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
        let forme = Forme {
            levels: Some(Range(2, 2)),
            silhouette: Some(Silhouette::Croix),
            profondeur: Some(30),
            ..Forme::default()
        };
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

    /// Icebreaker F.45: the table of the settings follows the fields of `Forme`, and what `generation.ron` gives
    /// is within the bounds of the sliders.
    #[test]
    fn the_settings_are_said_once() {
        let settings = Settings::load().unwrap();
        for taille in settings.sizes.keys() {
            let base = Forme::lire(&settings, taille).unwrap();
            assert!(base.valeurs().iter().all(Option::is_some), "{taille} : un réglage sans valeur de base");
            assert_eq!(base.erreurs(), Vec::<String>::new(), "{taille}");
            // Fixing every setting at its own value changes nothing.
            let same = base.appliquer(settings.clone());
            assert_eq!(Forme::lire(&same, taille).unwrap(), base);
        }
        assert!(Forme { profondeur: Some(61), ..Forme::default() }.erreurs().len() == 1);
        assert!(Forme { levels: Some(Range(1, 9)), ..Forme::default() }.erreurs().len() == 1);
    }

    #[test]
    fn a_bad_profile_is_refused() {
        assert!(Formes::parse("(profils: {\"x\": (poids: 1.0, forme: (levels: Some((3, 1))))})").is_err());
        assert!(Formes::parse("(profils: {\"x\": (poids: 1.0, forme: (etages: Some((1, 1))))})").is_err());
        let ok = Formes::parse("(profils: {\"x\": (poids: 0.0, forme: (levels: Some((1, 1))))})").unwrap();
        assert!(ok.tirer(1, None).is_none());
    }
}
