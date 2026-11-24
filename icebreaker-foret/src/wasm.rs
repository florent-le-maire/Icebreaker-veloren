//! What the workshop page (`web/index.html`) calls: raw exports, no binding
//! generator. The page writes the settings in `foret_entree`, calls
//! `foret_generer`, and reads the outputs as bytes. (The names start with
//! `foret_`: `icebreaker-caverne`, which this crate uses, exports its own.)

use crate::{CHAMPS, Forme, Formes, Genre, generer, maillage::mailler};
use std::sync::Mutex;

static FORMES: &str = include_str!("../../assets/icebreaker/forets/formes.ron");

struct Etat {
    entree: Vec<f32>,
    /// The text of `formes.ron` the page got from the server of the workshop
    /// (empty: the one built in, when the page is a file).
    formes: Vec<u8>,
    /// 0: vertices (i16), 1: colours (u8), 2: triangles (u32), 3: marks (i16:
    /// x, y, z, kind), 4: summary, 5: the text asked by `foret_texte`.
    sorties: [Vec<u8>; 6],
    sans_feuilles: u32,
}

static ETAT: Mutex<Etat> = Mutex::new(Etat {
    entree: Vec::new(),
    formes: Vec::new(),
    sorties: [const { Vec::new() }; 6],
    sans_feuilles: 0,
});

/// Room for `n` settings (in the order of `CHAMPS`): where to write them.
#[unsafe(no_mangle)]
pub extern "C" fn foret_entree(n: usize) -> *mut f32 {
    let mut etat = ETAT.lock().unwrap();
    etat.entree = vec![0.0; n];
    etat.entree.as_mut_ptr()
}

/// Room for the `n` bytes of `formes.ron`, as the server of the workshop gives it.
#[unsafe(no_mangle)]
pub extern "C" fn foret_formes(n: usize) -> *mut u8 {
    let mut etat = ETAT.lock().unwrap();
    etat.formes = vec![0; n];
    etat.formes.as_mut_ptr()
}

/// Generates the forest of the settings written in `foret_entree`; 1 if one
/// walks it whole.
#[unsafe(no_mangle)]
pub extern "C" fn foret_generer(graine: f64) -> u32 {
    let mut etat = ETAT.lock().unwrap();
    let foret = generer(&Forme::depuis_valeurs(&etat.entree), graine as u64, &|_, _| 0);
    let m = mailler(&foret);
    etat.sorties[0] = m.sommets.iter().flat_map(|v| v.to_le_bytes()).collect();
    etat.sorties[1] = m.couleurs;
    etat.sorties[2] = m.indices.iter().flat_map(|v| v.to_le_bytes()).collect();
    etat.sans_feuilles = m.sans_feuilles as u32;
    // The marks: 0 arrival, 1 camp, 2 nest, 3 end, 4 way up, 5 platform not reached.
    let mut marques: Vec<[i32; 4]> = foret
        .clairieres
        .iter()
        .map(|c| {
            let genre = match c.genre {
                Genre::Arrivee => 0,
                Genre::Camp => 1,
                Genre::Nid => 2,
                Genre::Fond => 3,
            };
            [c.centre[0], c.centre[1], c.centre[2], genre]
        })
        .collect();
    for arbre in &foret.arbres {
        if arbre.montee {
            marques.push([arbre.pied[0], arbre.pied[1], arbre.niveaux[0], 4]);
        }
        for (z, atteint) in arbre.niveaux.iter().zip(&arbre.atteints) {
            if !atteint {
                marques.push([arbre.pied[0], arbre.pied[1], *z + 4, 5]);
            }
        }
    }
    etat.sorties[3] = marques.iter().flatten().flat_map(|v| (*v as i16).to_le_bytes()).collect();
    etat.sorties[4] = foret.resume.into_bytes();
    foret.praticable as u32
}

/// How many triangle indices of the last forest come before the leaves.
#[unsafe(no_mangle)]
pub extern "C" fn foret_sans_feuilles() -> u32 { ETAT.lock().unwrap().sans_feuilles }

/// Writes a text in output 5. 0: the settings (one per line: key, group,
/// label, min, max, step, default); 1: the profiles (name, weight, values);
/// 2: the settings of `foret_entree` as a block of `formes.ron`; 3: the name
/// of the profile the seed draws.
#[unsafe(no_mangle)]
pub extern "C" fn foret_texte(quoi: u32, graine: f64) {
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
pub extern "C" fn foret_sortie_ptr(i: usize) -> *const u8 { ETAT.lock().unwrap().sorties[i.min(5)].as_ptr() }

#[unsafe(no_mangle)]
pub extern "C" fn foret_sortie_len(i: usize) -> usize { ETAT.lock().unwrap().sorties[i.min(5)].len() }
