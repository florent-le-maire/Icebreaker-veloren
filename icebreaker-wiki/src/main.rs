//! Icebreaker : générateur du wiki de l'équipe (`docs/icebreaker/PLAN-WIKI.md`).
//!
//! Lit `docs/icebreaker/**/*.md` et `assets/icebreaker/`, écrit un site statique dans `site/`.
//! Lancer depuis la racine du dépôt : `cargo run -p icebreaker-wiki` (ou passer la racine en argument).

mod markdown;
mod site;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

/// Une page du site, issue d'un fichier Markdown.
#[derive(Debug, Clone)]
pub struct Page {
    /// Chemin relatif à `docs/icebreaker/`, avec des `/` (ex. `design/combat.md`).
    pub source: String,
    /// Chemin relatif de la page écrite (ex. `design/combat.html`, `index.html` pour le README).
    pub url: String,
    /// Rubrique de la barre latérale.
    pub rubrique: Rubrique,
    pub titre: String,
    /// Corps rendu en HTML.
    pub html: String,
}

/// Les rubriques de la barre latérale, dans l'ordre d'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rubrique {
    Reperes,
    Plans,
    Regles,
    Decisions,
}

impl Rubrique {
    pub fn nom(self) -> &'static str {
        match self {
            Rubrique::Reperes => "Repères",
            Rubrique::Plans => "Plans",
            Rubrique::Regles => "Règles du jeu",
            Rubrique::Decisions => "Décisions",
        }
    }

    pub fn toutes() -> [Rubrique; 4] {
        [Rubrique::Reperes, Rubrique::Plans, Rubrique::Regles, Rubrique::Decisions]
    }

    /// La rubrique d'un fichier d'après son chemin relatif à `docs/icebreaker/`.
    pub fn de_source(source: &str) -> Rubrique {
        if source.starts_with("design/") {
            Rubrique::Regles
        } else if source.starts_with("decisions/") {
            Rubrique::Decisions
        } else if source.starts_with("PLAN-") {
            Rubrique::Plans
        } else {
            Rubrique::Reperes
        }
    }
}

/// Les fichiers repères, dans l'ordre de la barre latérale, avec leur nom court.
const REPERES: [(&str, &str); 7] = [
    ("README.md", "Accueil"),
    ("PLAN.md", "Plan"),
    ("HISTORIQUE.md", "Historique"),
    ("QUESTIONS.md", "Questions"),
    ("IDEES.md", "Idées"),
    ("VISION.md", "Vision"),
    ("GAME_DESIGN.md", "Index des règles"),
];

/// Nom court d'un fichier repère, s'il en est un.
pub fn nom_repere(source: &str) -> Option<&'static str> {
    REPERES.iter().find(|(s, _)| *s == source).map(|(_, n)| *n)
}

/// Clé de tri dans la barre latérale : les repères dans l'ordre de `REPERES`, le reste par chemin.
fn cle_de_tri(page: &Page) -> (Rubrique, usize, String) {
    let rang = REPERES
        .iter()
        .position(|(s, _)| *s == page.source)
        .unwrap_or(REPERES.len());
    (page.rubrique, rang, page.source.clone())
}

/// L'URL d'une page d'après son fichier source : `README.md` devient l'accueil, les autres gardent leur chemin.
pub fn url_de_source(source: &str) -> String {
    if source == "README.md" {
        "index.html".to_string()
    } else {
        format!("{}.html", source.trim_end_matches(".md"))
    }
}

/// Lit tous les fichiers Markdown de `docs` et les rend en pages (sans les écrire).
pub fn collecter(docs: &Path) -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    for entree in WalkDir::new(docs).sort_by_file_name() {
        let entree = entree?;
        let chemin = entree.path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let source = chemin
            .strip_prefix(docs)?
            .to_string_lossy()
            .replace('\\', "/");
        let texte = std::fs::read_to_string(chemin).with_context(|| format!("lecture de {source}"))?;
        let (_entete, corps) = markdown::separer_entete(&texte);
        let rendu = markdown::rendre(corps, &source);
        let titre = rendu
            .titre
            .unwrap_or_else(|| source.trim_end_matches(".md").to_string());
        pages.push(Page {
            url: url_de_source(&source),
            rubrique: Rubrique::de_source(&source),
            titre,
            html: rendu.html,
            source,
        });
    }
    pages.sort_by_key(cle_de_tri);
    Ok(pages)
}

fn racine_depot() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn main() -> Result<()> {
    let racine = racine_depot();
    let docs = racine.join("docs").join("icebreaker");
    anyhow::ensure!(
        docs.is_dir(),
        "dossier {} introuvable : lancer depuis la racine du dépôt ou passer la racine en argument",
        docs.display()
    );
    let sortie = racine.join("site");
    let pages = collecter(&docs)?;
    let nb = site::ecrire(&sortie, &pages)?;
    println!("{nb} pages écrites dans {}", sortie.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docs_du_depot() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/icebreaker")
    }

    #[test]
    fn chaque_fichier_markdown_donne_une_page() {
        let docs = docs_du_depot();
        let pages = collecter(&docs).unwrap();
        let nb_md = WalkDir::new(&docs)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            .count();
        assert_eq!(pages.len(), nb_md);
        assert!(pages.iter().any(|p| p.url == "index.html" && p.source == "README.md"));
        assert!(pages.iter().any(|p| p.url == "design/combat.html" && p.rubrique == Rubrique::Regles));
        assert!(pages.iter().all(|p| !p.titre.is_empty() && !p.html.is_empty()));
    }

    #[test]
    fn rubriques_et_urls() {
        assert_eq!(Rubrique::de_source("PLAN.md"), Rubrique::Reperes);
        assert_eq!(Rubrique::de_source("PLAN-MOD.md"), Rubrique::Plans);
        assert_eq!(Rubrique::de_source("design/ames.md"), Rubrique::Regles);
        assert_eq!(Rubrique::de_source("decisions/0016-x.md"), Rubrique::Decisions);
        assert_eq!(url_de_source("README.md"), "index.html");
        assert_eq!(url_de_source("decisions/0016-x.md"), "decisions/0016-x.html");
    }
}
