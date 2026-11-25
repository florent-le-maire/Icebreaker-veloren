//! Icebreaker : le classeur en texte. À côté de `equilibrage.xlsx`, un dossier `equilibrage/` avec un fichier
//! Markdown par onglet (les mêmes tableaux, mêmes chiffres) et `synthese.md` pour la vue d'ensemble : lisible
//! sans tableur, cherchable, et les changements d'équilibrage se lisent dans les diffs de git.

use crate::style::{self, Onglet, Tableau};
use anyhow::{Context, Result};
use std::{fmt::Write as _, path::Path};

/// Onglets sans version texte : la Synthèse a son fichier à part, les Réglages recopient les `.ron`.
const SANS_FICHIER: [&str; 3] = ["Synthèse", "Réglages", "Lisez-moi"];
const AVIS: &str = "> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\\icebreaker.ps1 equilibrage`) \
                    depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que \
                    `docs/icebreaker/equilibrage.xlsx`.";

/// Le nom de fichier d'un onglet : « Économie » → `economie`.
fn nom_de_fichier(onglet: &str) -> String {
    onglet
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'é' | 'è' | 'ê' => 'e',
            'à' | 'â' => 'a',
            'a'..='z' | '0'..='9' => c,
            _ => '-',
        })
        .collect()
}

fn cellule(texte: &str) -> String { texte.replace('|', "/").replace(['\r', '\n'], " ") }

/// Un tableau Markdown.
pub fn tableau(colonnes: &[String], lignes: &[Vec<String>]) -> String {
    let mut sortie = String::new();
    let ligne = |cases: &[String]| format!("| {} |\n", cases.iter().map(|c| cellule(c)).collect::<Vec<_>>().join(" | "));
    sortie.push_str(&ligne(colonnes));
    sortie.push_str(&format!("|{}\n", "---|".repeat(colonnes.len())));
    lignes.iter().for_each(|l| sortie.push_str(&ligne(l)));
    sortie
}

fn fichier_d_onglet(onglet: &Onglet, tableaux: &[&Tableau]) -> String {
    let mut texte = format!("# {}\n\n{AVIS}\n\n{}\n", onglet.titre, onglet.explication);
    for t in tableaux {
        if !t.titre.is_empty() {
            let _ = write!(texte, "\n## {}\n", t.titre);
        }
        let _ = write!(texte, "\n{}", tableau(&t.colonnes, &t.lignes));
    }
    texte
}

/// Écrit le dossier ; `synthese` est le corps de `synthese.md`. Renvoie le nombre de fichiers écrits.
pub fn ecrire(dossier: &Path, synthese: &str) -> Result<usize> {
    let (onglets, tableaux) = style::ecrits();
    std::fs::create_dir_all(dossier).with_context(|| format!("création de {}", dossier.display()))?;
    // Un onglet disparu ne laisse pas son fichier derrière lui.
    for entree in std::fs::read_dir(dossier)? {
        let chemin = entree?.path();
        if chemin.extension().is_some_and(|e| e == "md") {
            std::fs::remove_file(&chemin)?;
        }
    }
    let mut sommaire = String::new();
    let mut ecrits = 1;
    for onglet in onglets.iter().filter(|o| !SANS_FICHIER.contains(&o.nom.as_str())) {
        let siens: Vec<&Tableau> = tableaux.iter().filter(|t| t.onglet == onglet.nom).collect();
        let fichier = format!("{}.md", nom_de_fichier(&onglet.nom));
        std::fs::write(dossier.join(&fichier), fichier_d_onglet(onglet, &siens))?;
        let lignes: usize = siens.iter().map(|t| t.lignes.len()).sum();
        let _ = writeln!(sommaire, "- [`{fichier}`]({fichier}) ({lignes} lignes) — {}", onglet.explication);
        ecrits += 1;
    }
    let texte = format!(
        "# Équilibrage d'Icebreaker : vue d'ensemble\n\n{AVIS}\n\n{synthese}\n## Le détail, un fichier par onglet\n\n\
         {sommaire}\nCe que ces chiffres supposent, et comment s'en servir pour régler le jeu : skill `equilibrage` \
         (`.claude/skills/equilibrage/SKILL.md`).\n"
    );
    std::fs::write(dossier.join("synthese.md"), texte)?;
    Ok(ecrits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noms_et_tableau() {
        assert_eq!(nom_de_fichier("Économie"), "economie");
        assert_eq!(nom_de_fichier("Hypothèses"), "hypotheses");
        let t = tableau(&["a".into(), "b".into()], &[vec!["1".into(), "x | y".into()]]);
        assert_eq!(t, "| a | b |\n|---|---|\n| 1 | x / y |\n");
    }
}
