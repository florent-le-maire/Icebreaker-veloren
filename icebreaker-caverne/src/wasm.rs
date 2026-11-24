//! What the workshop page (`web/index.html`) calls: raw exports, no binding
//! generator. The page writes the settings in `entree`, calls `generer`, and
//! reads the outputs as bytes.

use crate::{CHAMPS, Forme, Formes, Genre, filon, generer as generer_caverne, maillage::mailler_avec};
use std::sync::Mutex;

static FORMES: &str = include_str!("../../assets/icebreaker/grottes/formes.ron");
// Icebreaker U.21: the ores and how many veins, to show them.
static MINERAIS: &str = include_str!("../../assets/icebreaker/grottes/minerais.ron");
static GROTTE: &str = include_str!("../../assets/icebreaker/grottes/grotte.ron");

struct Etat {
    entree: Vec<f32>,
    /// Icebreaker U.10: the text of `formes.ron` the page got from the server
    /// of the workshop (empty: the one built in, when the page is a file).
    formes: Vec<u8>,
    /// 0: vertices (i16), 1: colours (u8), 2: triangles (u32), 3: marks (i16:
    /// x, y, z, kind), 4: summary, 5: the text asked by `texte`.
    sorties: [Vec<u8>; 6],
}

static ETAT: Mutex<Etat> = Mutex::new(Etat {
    entree: Vec::new(),
    formes: Vec::new(),
    sorties: [const { Vec::new() }; 6],
});

/// Room for `n` settings (in the order of `CHAMPS`): where to write them.
#[unsafe(no_mangle)]
pub extern "C" fn entree(n: usize) -> *mut f32 {
    let mut etat = ETAT.lock().unwrap();
    etat.entree = vec![0.0; n];
    etat.entree.as_mut_ptr()
}

/// Room for the `n` bytes of `formes.ron`, as the server of the workshop
/// gives it: the profiles `texte` lists are then the ones of that text.
#[unsafe(no_mangle)]
pub extern "C" fn formes(n: usize) -> *mut u8 {
    let mut etat = ETAT.lock().unwrap();
    etat.formes = vec![0; n];
    etat.formes.as_mut_ptr()
}

/// Generates the cave of the settings written in `entree`; 1 if its trail is whole.
#[unsafe(no_mangle)]
pub extern "C" fn generer(graine: f64) -> u32 {
    let mut etat = ETAT.lock().unwrap();
    let caverne = generer_caverne(&Forme::depuis_valeurs(&etat.entree), graine as u64, &|_, _| 0, i32::MIN);
    // Icebreaker U.21: the veins, placed by the function and the settings of
    // the game (minerais.ron, grotte.ron), as cubes in the colour of their ore;
    // not at the places of the game (no ground of the world, no inhabitants).
    let mut ores = std::collections::HashMap::new();
    let mut veines = 0;
    if let Ok(r) = filon::lire_reglages(MINERAIS, GROTTE) {
        let niveaux = filon::niveaux_des_zones(&caverne, r.niveaux);
        let poses = filon::poser(&caverne, &r.gisements, &niveaux, &r.par_zone, r.salle, &|_| false, graine as u64);
        veines = poses.len();
        for pose in &poses {
            ores.extend(pose.filon.blocs().map(|p| (*p, r.couleurs[pose.minerai])));
        }
    }
    let m = mailler_avec(&caverne, &|p| ores.get(&p).copied());
    etat.sorties[0] = m.sommets.iter().flat_map(|v| v.to_le_bytes()).collect();
    etat.sorties[1] = m.couleurs;
    etat.sorties[2] = m.indices.iter().flat_map(|v| v.to_le_bytes()).collect();
    let fond = caverne.fond();
    let mut marks: Vec<[i32; 4]> = vec![[caverne.depart[0], caverne.depart[1], caverne.depart[2], 0]];
    for (i, zone) in caverne.zones.iter().enumerate() {
        let kind = match zone.genre {
            _ if i == fond => 4,
            Genre::Plaine => 1,
            Genre::Chaos => 2,
            Genre::Niche => 3,
            // Icebreaker U.15: the ore room.
            Genre::Minerai => 6,
            // The safe room is the first mark: where one arrives.
            Genre::Abri => continue,
        };
        marks.push([zone.ancre[0], zone.ancre[1], zone.ancre[2], kind]);
    }
    // Icebreaker U.13: the walls to break.
    marks.extend(caverne.liens.iter().filter_map(|l| l.mur).map(|m| [m[0], m[1], m[2], 5]));
    etat.sorties[3] = marks.iter().flatten().flat_map(|v| (*v as i16).to_le_bytes()).collect();
    etat.sorties[4] = format!("{} ; {veines} filon(s), {} blocs de minerai", caverne.resume, ores.len()).into_bytes();
    caverne.praticable as u32
}

/// Writes a text in output 5. 0: the settings (one per line: key, group,
/// label, min, max, step, default); 1: the profiles (name, weight, values);
/// 2: the settings of `entree` as a block of `formes.ron`; 3: the name of the
/// profile the seed draws.
#[unsafe(no_mangle)]
pub extern "C" fn texte(quoi: u32, graine: f64) {
    let mut etat = ETAT.lock().unwrap();
    let formes = Formes::lire(if etat.formes.is_empty() { FORMES } else { std::str::from_utf8(&etat.formes).unwrap_or("") });
    let text = match (quoi, formes) {
        (0, _) => CHAMPS
            .iter()
            .map(|c| format!("{}\t{}\t{}\t{}\t{}\t{}\t{}", c.cle, c.groupe, c.libelle, c.min, c.max, c.pas, c.defaut))
            .collect::<Vec<_>>()
            .join("\n"),
        (1, Ok(formes)) => formes
            .profils
            .iter()
            .map(|(nom, p)| {
                let valeurs: Vec<String> = p.forme.valeurs().iter().map(|v| v.to_string()).collect();
                format!("{nom}\t{}\t{}", p.poids, valeurs.join("\t"))
            })
            .collect::<Vec<_>>()
            .join("\n"),
        (2, _) => Forme::depuis_valeurs(&etat.entree).bornee().vers_ron(),
        (3, Ok(formes)) => formes.tirer(graine as u64, None).0.to_string(),
        (_, Err(e)) => e,
        _ => String::new(),
    };
    etat.sorties[5] = text.into_bytes();
}

#[unsafe(no_mangle)]
pub extern "C" fn sortie_ptr(i: usize) -> *const u8 { ETAT.lock().unwrap().sorties[i.min(5)].as_ptr() }

#[unsafe(no_mangle)]
pub extern "C" fn sortie_len(i: usize) -> usize { ETAT.lock().unwrap().sorties[i.min(5)].len() }
