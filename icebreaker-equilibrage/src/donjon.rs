//! Icebreaker : ce qu'il y a dans un donjon généré, compté sur de vrais donjons (le générateur du jeu, plusieurs
//! graines par cas) plutôt qu'estimé d'après les fourchettes de `generation.ron`.

use anyhow::{Result, anyhow};
use world::icebreaker::dungeon::{self, Mission, Settings, populate};

/// Les sortes de créatures d'un plan, dans l'ordre des colonnes : (clé du générateur, ce qu'on lit).
pub const SORTES: [(&str, &str); 6] = [
    ("brute", "Brutes"),
    ("archer", "Tireurs"),
    ("lookout", "Guetteurs"),
    ("beast", "Bêtes"),
    ("chief", "Chefs"),
    ("boss", "Boss"),
];

/// Un cas : une taille, une menace, une densité ; les nombres sont des moyennes sur les graines.
#[derive(Clone, Debug)]
pub struct Cas {
    pub taille: String,
    pub menace: i32,
    pub densite: String,
    /// L'expédition type : taille moyenne, menace et densité par défaut.
    pub reference: bool,
    /// Donjons générés pour ce cas.
    pub graines: usize,
    pub salles: f64,
    /// Créatures par sorte, dans l'ordre de `SORTES`.
    pub par_sorte: Vec<f64>,
    /// Créatures ordinaires (ni chef ni boss) par palier de butin.
    pub creatures_palier: Vec<f64>,
    pub coffres_palier: Vec<f64>,
}

pub struct Donjon {
    pub cas: Vec<Cas>,
    /// Qui est chaque sorte dans la faction par défaut : les fiches d'entité, dans l'ordre de `SORTES`.
    pub habitants: Vec<Vec<String>>,
    pub faction: String,
    pub paliers: usize,
}

fn un_cas(settings: &Settings, taille: &str, menace: i32, densite: &str, graines: u64, paliers: usize) -> Cas {
    let mut cas = Cas {
        taille: taille.to_string(),
        menace,
        densite: densite.to_string(),
        reference: false,
        graines: 0,
        salles: 0.0,
        par_sorte: vec![0.0; SORTES.len()],
        creatures_palier: vec![0.0; paliers],
        coffres_palier: vec![0.0; paliers],
    };
    for graine in 1..=graines {
        let mission = Mission {
            seed: graine,
            size: taille.to_string(),
            threat: Some(menace),
            density: Some(densite.to_string()),
            ..Mission::default()
        };
        let Ok(donjon) = dungeon::generate(settings, &mission) else { continue };
        cas.graines += 1;
        cas.salles += donjon.salles.len() as f64;
        for (case, marque) in &donjon.plan.cells {
            let palier = donjon.paliers.get(case).copied().unwrap_or(1).clamp(1, paliers) - 1;
            if *marque == '$' {
                cas.coffres_palier[palier] += 1.0;
            } else if let Some(sorte) = populate::kind_of(*marque) {
                if let Some(i) = SORTES.iter().position(|(cle, _)| *cle == sorte) {
                    cas.par_sorte[i] += 1.0;
                }
                if sorte != "chief" && sorte != "boss" {
                    cas.creatures_palier[palier] += 1.0;
                }
            }
        }
    }
    let n = cas.graines.max(1) as f64;
    let moyenne = |x: &mut f64| *x = (*x / n * 10.0).round() / 10.0;
    moyenne(&mut cas.salles);
    cas.par_sorte.iter_mut().for_each(moyenne);
    cas.creatures_palier.iter_mut().for_each(moyenne);
    cas.coffres_palier.iter_mut().for_each(moyenne);
    cas
}

/// Génère `graines` donjons par cas : chaque taille à chaque menace (densité par défaut), puis chaque taille à
/// chaque autre densité (menace par défaut).
pub fn lire(graines: u64) -> Result<Donjon> {
    let settings = Settings::load().map_err(|e| anyhow!("generation.ron : {e}"))?;
    let paliers = world::icebreaker::butin::Butin::load().map_err(anyhow::Error::msg)?.paliers();
    let parametres = &settings.parameters;
    let mut tailles: Vec<&String> = settings.sizes.keys().collect();
    tailles.sort_by_key(|t| settings.sizes[*t].main_path.0);
    let mut menaces: Vec<i32> = parametres.threats.keys().filter_map(|m| m.parse().ok()).collect();
    menaces.sort_unstable();
    let mut densites: Vec<&String> = parametres.densities.keys().collect();
    densites.sort_by(|a, b| {
        parametres.densities[*a].creature_factor.total_cmp(&parametres.densities[*b].creature_factor)
    });
    let mut cas = Vec::new();
    for taille in &tailles {
        for menace in &menaces {
            cas.push(un_cas(&settings, taille, *menace, &parametres.default_density, graines, paliers));
        }
        for densite in densites.iter().filter(|d| ***d != parametres.default_density) {
            cas.push(un_cas(&settings, taille, parametres.default_threat, densite, graines, paliers));
        }
    }
    for c in &mut cas {
        c.reference =
            c.taille == "medium" && c.menace == parametres.default_threat && c.densite == parametres.default_density;
    }
    let faction = settings
        .factions
        .get(&settings.default_faction)
        .ok_or_else(|| anyhow!("faction par défaut inconnue : {}", settings.default_faction))?;
    let habitants = SORTES
        .iter()
        .map(|(cle, _)| match faction.creatures.get(*cle) {
            Some(fiches) => fiches.clone(),
            // Une faction sans bête : ses nids sont peuplés par les monstres solitaires.
            None if *cle == "beast" => settings.solitaires.clone(),
            None => Vec::new(),
        })
        .collect();
    Ok(Donjon {
        cas,
        habitants,
        faction: settings.default_faction.clone(),
        paliers,
    })
}

pub fn taille_en_clair(taille: &str) -> &str {
    match taille {
        "small" => "Petit",
        "medium" => "Moyen",
        "large" => "Grand",
        autre => autre,
    }
}

pub fn densite_en_clair(densite: &str) -> &str {
    match densite {
        "low" => "Faible",
        "medium" => "Moyenne",
        "high" => "Forte",
        autre => autre,
    }
}
