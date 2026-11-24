//! The profiles of `assets/icebreaker/forets/formes.ron` edited as text, for
//! the workshop to save a preset or remove one without losing the comments of
//! the file (the text is handled by `icebreaker_caverne::presets`, as for the
//! caves and the dungeons). Pure functions: the server of the workshop
//! (`bin/atelier.rs`) writes the result and commits it.

use crate::{Forme, Formes};
use icebreaker_caverne::presets::{nom_valide, poser_entree, retirer_entree};

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

/// `formes.ron` with the profile `nom` added, or replaced if it is there.
pub fn poser_profil(texte: &str, nom: &str, poids: f32, forme: &Forme) -> Result<String, String> {
    nom_valide(nom)?;
    if !(0.0..=100.0).contains(&poids) {
        return Err(format!("poids refusé : {poids} (0 à 100)"));
    }
    let sortie = poser_entree(texte, nom, &bloc(nom, poids, &forme.bornee()))?;
    match Formes::lire(&sortie)?.profils.get(nom) {
        Some(p) if p.forme == forme.bornee() => rendre(texte, sortie),
        _ => Err(format!("le preset « {nom} » n'a pas été écrit comme prévu")),
    }
}

/// `formes.ron` without the profile `nom` (nor the comment over it).
pub fn retirer_profil(texte: &str, nom: &str) -> Result<String, String> { rendre(texte, retirer_entree(texte, nom)?) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_real_file_takes_a_preset() {
        let texte = include_str!("../../assets/icebreaker/forets/formes.ron");
        let forme = Forme { ponts: 0.9, ..Forme::default() };
        let plus = poser_profil(texte, "essai", 1.0, &forme).unwrap();
        assert_eq!(Formes::lire(&plus).unwrap().profils["essai"].forme, forme);
        assert_eq!(retirer_profil(&plus, "essai").unwrap().replace("\r\n", "\n"), texte.replace("\r\n", "\n"));
        assert!(poser_profil(texte, "Grande Forêt", 1.0, &forme).is_err());
        assert!(retirer_profil(texte, "essai").is_err());
    }
}
