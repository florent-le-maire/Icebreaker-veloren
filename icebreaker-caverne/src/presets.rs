//! Icebreaker U.10: the profiles of `formes.ron` edited as text, for the
//! workshop to save a preset or remove one without losing the comments of
//! the file. Pure functions: the server of the workshop (`bin/atelier.rs`)
//! writes the result and commits it.

use crate::{Forme, Formes};

/// A name one can type after `/expedition` and read on a button.
pub fn nom_valide(nom: &str) -> Result<(), String> {
    let ok = !nom.is_empty()
        && nom.len() <= 24
        && nom.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_');
    if ok { Ok(()) } else { Err(format!("nom de preset refusé : « {nom} » (1 à 24 caractères parmi a-z, 0-9, - et _)")) }
}

/// Where the bracket opened at `open` closes (comments and strings skipped).
fn fermeture(t: &[u8], open: usize) -> Option<usize> {
    let (mut depth, mut i) = (0, open);
    while i < t.len() {
        match t[i] {
            b'/' if t.get(i + 1) == Some(&b'/') => {
                while i < t.len() && t[i] != b'\n' {
                    i += 1;
                }
                continue;
            },
            b'"' => {
                i += 1;
                while i < t.len() && t[i] != b'"' {
                    i += 1 + (t[i] == b'\\') as usize;
                }
            },
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            },
            _ => {},
        }
        i += 1;
    }
    None
}

/// The start of the line `at` is in.
fn debut_ligne(t: &str, at: usize) -> usize { t[..at].rfind('\n').map_or(0, |i| i + 1) }

/// The lines of the profile `nom` (`"nom": ( … ),`), in bytes of `t`.
fn entree(t: &str, nom: &str) -> Option<(usize, usize)> {
    let cle = format!("\"{nom}\":");
    let mut debut = 0;
    for ligne in t.split_inclusive('\n') {
        if ligne.trim_start().starts_with(&cle) {
            let open = debut + ligne.find('(')?;
            let mut fin = fermeture(t.as_bytes(), open)? + 1;
            fin += t[fin..].starts_with(',') as usize;
            fin += t[fin..].find('\n').map_or(t.len() - fin, |i| i + 1);
            return Some((debut, fin));
        }
        debut += ligne.len();
    }
    None
}

fn bloc(nom: &str, poids: f32, forme: &Forme) -> String {
    let ron = forme.vers_ron();
    let lignes: Vec<&str> = ron.lines().collect();
    let mut text = format!("        \"{nom}\": (\n            poids: {poids:?},\n            forme: (\n");
    for ligne in &lignes[1..lignes.len() - 1] {
        text += &format!("                {}\n", ligne.trim());
    }
    text + "            ),\n        ),\n"
}

fn rendre(texte: &str, sortie: String) -> Result<String, String> {
    Formes::lire(&sortie)?;
    Ok(if texte.contains("\r\n") { sortie.replace('\n', "\r\n") } else { sortie })
}

/// `formes.ron` with the profile `nom` added, or replaced if it is there
/// (the comment over it is kept).
pub fn poser_profil(texte: &str, nom: &str, poids: f32, forme: &Forme) -> Result<String, String> {
    nom_valide(nom)?;
    if !(0.0..=100.0).contains(&poids) {
        return Err(format!("poids refusé : {poids} (0 à 100)"));
    }
    let sortie = poser_entree(texte, nom, &bloc(nom, poids, &forme.bornee()))?.replace("\r\n", "\n");
    let formes = rendre(texte, sortie.clone())?;
    match Formes::lire(&sortie)?.profils.get(nom) {
        Some(p) if p.forme == forme.bornee() => Ok(formes),
        _ => Err(format!("le preset « {nom} » n'a pas été écrit comme prévu")),
    }
}

/// `formes.ron` without the profile `nom` (nor the comment over it).
pub fn retirer_profil(texte: &str, nom: &str) -> Result<String, String> {
    rendre(texte, retirer_entree(texte, nom)?)
}

/// Icebreaker F.34: `texte` (a RON file of `profils: { … }`, as a `formes.ron`)
/// with the entry `nom` replaced by `bloc` (its lines), or `bloc` added at the
/// end of the profiles; `\n` line ends, unchecked (the dungeon workshop uses it
/// too, with its own file).
pub fn poser_entree(texte: &str, nom: &str, bloc: &str) -> Result<String, String> {
    let t = texte.replace("\r\n", "\n");
    let (debut, fin) = match entree(&t, nom) {
        Some(range) => range,
        None => {
            let open = t.find("profils:").and_then(|i| t[i..].find('{').map(|j| i + j));
            let close = open.and_then(|o| fermeture(t.as_bytes(), o)).ok_or("formes.ron : pas de `profils: { … }`")?;
            let at = debut_ligne(&t, close);
            (at, at)
        },
    };
    Ok(format!("{}{bloc}{}", &t[..debut], &t[fin..]))
}

/// Icebreaker F.34: `texte` without the entry `nom` (nor the comment over it); `\n` line ends, unchecked.
pub fn retirer_entree(texte: &str, nom: &str) -> Result<String, String> {
    let t = texte.replace("\r\n", "\n");
    let (mut debut, fin) = entree(&t, nom).ok_or(format!("pas de preset « {nom} »"))?;
    while debut > 0 {
        let avant = debut_ligne(&t, debut - 1);
        if !t[avant..debut].trim_start().starts_with("//") {
            break;
        }
        debut = avant;
    }
    Ok(format!("{}{}", &t[..debut], &t[fin..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXTE: &str = "// En-tête.\n(\n    profils: {\n        // Le premier (avec une parenthèse.\n        \"un\": (\n            \
                         poids: 1.0,\n            forme: (zones: 6.0, verticalite: 0.9),\n        ),\n        // Le second.\n        \
                         \"deux\": (poids: 2.0, forme: (zones: 9.0)),\n    },\n)\n";

    #[test]
    fn a_preset_is_added_replaced_and_removed() {
        let forme = Forme { zones: 4.0, etendue_y: 0.35, ..Forme::default() };
        // Added at the end, the rest untouched.
        let plus = poser_profil(TEXTE, "boyau", 3.0, &forme).unwrap();
        assert!(plus.starts_with(&TEXTE[..TEXTE.find("    },").unwrap()]));
        let formes = Formes::lire(&plus).unwrap();
        assert_eq!(formes.profils.len(), 3);
        assert_eq!(formes.profils["boyau"].forme, forme);
        assert_eq!(formes.profils["boyau"].poids, 3.0);
        // Replaced: its comment and the others stay.
        let autre = poser_profil(&plus, "un", 0.5, &forme).unwrap();
        assert!(autre.contains("// Le premier (avec une parenthèse.\n        \"un\": (\n            poids: 0.5,"));
        assert!(autre.contains("// Le second.\n        \"deux\": (poids: 2.0, forme: (zones: 9.0)),"));
        assert_eq!(Formes::lire(&autre).unwrap().profils["un"].forme, forme);
        // Removed, with its comment; adding then removing gives the file back.
        assert_eq!(retirer_profil(&plus, "boyau").unwrap(), TEXTE);
        let moins = retirer_profil(TEXTE, "deux").unwrap();
        assert!(!moins.contains("second") && Formes::lire(&moins).unwrap().profils.len() == 1);
    }

    #[test]
    fn what_would_break_the_file_is_refused() {
        let forme = Forme::default();
        assert!(poser_profil(TEXTE, "Grand Trou", 1.0, &forme).is_err());
        assert!(poser_profil(TEXTE, "", 1.0, &forme).is_err());
        assert!(poser_profil(TEXTE, "x", -1.0, &forme).is_err());
        assert!(retirer_profil(TEXTE, "trois").is_err());
        // Not the last profile the seed may draw.
        let seul = retirer_profil(TEXTE, "deux").unwrap();
        assert!(retirer_profil(&seul, "un").is_err());
        assert!(poser_profil(&seul, "un", 0.0, &forme).is_err());
        // The line endings of the file are kept.
        let crlf = TEXTE.replace('\n', "\r\n");
        let plus = poser_profil(&crlf, "boyau", 1.0, &forme).unwrap();
        assert!(!plus.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn the_real_file_takes_a_preset() {
        let texte = include_str!("../../assets/icebreaker/grottes/formes.ron");
        let plus = poser_profil(texte, "essai", 1.0, &Forme::default()).unwrap();
        assert_eq!(retirer_profil(&plus, "essai").unwrap(), texte);
    }
}
