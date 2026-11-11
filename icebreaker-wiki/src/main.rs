//! Icebreaker : générateur du wiki de l'équipe (`docs/icebreaker/PLAN-WIKI.md`).
//!
//! Lit `docs/icebreaker/**/*.md` et `assets/icebreaker/`, écrit un site statique dans `site/`.
//! Lancer depuis la racine du dépôt : `cargo run -p icebreaker-wiki` (ou passer la racine en argument).

mod avancement;
mod classes; // Icebreaker : A.18
mod commits;
mod competences; // Icebreaker : W.17
mod creatures; // Icebreaker : W.15
mod decisions;
mod edition;
mod fiches;
mod icones;
mod markdown;
mod reglages;
mod schemas;
mod serveur;
mod site;
mod taches;
mod todo;

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
    /// Corps rendu en HTML (pages Markdown).
    pub html: String,
    /// Gabarit minijinja (`page.html` pour une page Markdown).
    pub gabarit: &'static str,
    /// Données propres au gabarit (tableau de bord, jalon), `Null` pour une page Markdown.
    pub extra: serde_json::Value,
    /// Ordre dans la rubrique avant le tri par chemin (les jalons suivent l'ordre du README).
    pub ordre: usize,
    /// Icône de la page : chemin d'une image sous `assets/` (celles de Veloren), copiée dans le site.
    pub icone: Option<String>,
    /// Autres fichiers à copier dans le site : (chemin depuis la racine du dépôt, chemin dans le site).
    pub images: Vec<(String, String)>,
}

/// Les rubriques de la barre latérale, dans l'ordre d'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rubrique {
    Reperes,
    Mecaniques,
    Creatures,
    Competences,
    Avancement,
    Plans,
    Regles,
    Reglages,
    Decisions,
}

impl Rubrique {
    pub fn nom(self) -> &'static str {
        match self {
            Rubrique::Reperes => "Repères",
            Rubrique::Mecaniques => "Mécaniques",
            Rubrique::Creatures => "Créatures",
            Rubrique::Competences => "Compétences",
            Rubrique::Avancement => "Avancement",
            Rubrique::Plans => "Plans",
            Rubrique::Regles => "Règles du jeu",
            Rubrique::Reglages => "Réglages",
            Rubrique::Decisions => "Décisions",
        }
    }

    pub fn toutes() -> [Rubrique; 9] {
        [
            Rubrique::Reperes,
            Rubrique::Mecaniques,
            Rubrique::Creatures,
            Rubrique::Competences,
            Rubrique::Avancement,
            Rubrique::Plans,
            Rubrique::Regles,
            Rubrique::Reglages,
            Rubrique::Decisions,
        ]
    }

    /// La rubrique d'un fichier d'après son chemin relatif à `docs/icebreaker/`.
    pub fn de_source(source: &str) -> Rubrique {
        if source.starts_with("wiki/") {
            Rubrique::Mecaniques
        } else if source.starts_with("design/") {
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
const REPERES: [(&str, &str); 10] = [
    ("", "Tableau de bord"),
    ("README.md", "Lire les docs"),
    ("PLAN.md", "Plan"),
    ("a-voir", "À voir en jeu"),
    ("HISTORIQUE.md", "Historique"),
    ("QUESTIONS.md", "Questions"),
    ("A-FAIRE.md", "À faire"),
    ("IDEES.md", "Idées"),
    ("VISION.md", "Vision"),
    ("GAME_DESIGN.md", "Index des règles"),
];

/// Nom court d'un fichier repère, s'il en est un.
pub fn nom_repere(source: &str) -> Option<&'static str> {
    REPERES.iter().find(|(s, _)| *s == source).map(|(_, n)| *n)
}

/// Clé de tri dans la barre latérale : les repères dans l'ordre de `REPERES`, le reste par chemin.
fn cle_de_tri(page: &Page) -> (Rubrique, usize, usize, String) {
    let rang = match page.rubrique {
        Rubrique::Reperes => REPERES.iter().position(|(s, _)| *s == page.source),
        _ => None,
    }
    .unwrap_or(REPERES.len());
    (page.rubrique, rang, page.ordre, page.source.clone())
}

/// L'URL d'une page d'après son fichier source : `README.md` devient `readme.html` (l'accueil est le tableau
/// de bord), les autres gardent leur chemin.
pub fn url_de_source(source: &str) -> String {
    if source == "README.md" {
        "readme.html".to_string()
    } else {
        format!("{}.html", source.trim_end_matches(".md"))
    }
}

/// Lit tous les fichiers Markdown de `docs` : (chemin relatif avec des `/`, texte).
pub fn lire_fichiers(docs: &Path) -> Result<Vec<(String, String)>> {
    let mut fichiers = Vec::new();
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
        fichiers.push((source, texte));
    }
    Ok(fichiers)
}

/// Le dossier d'un chemin relatif avec des `/`, barre comprise (`design/combat.md` → `design/`, `PLAN.md` → ``).
fn dossier_de(chemin: &str) -> &str {
    match chemin.rfind('/') {
        Some(i) => &chemin[..=i],
        None => "",
    }
}

/// Les images d'une page à copier dans le site : écrites relativement au fichier source, elles restent relatives à
/// la page écrite.
pub fn images_de_page(source: &str, url: &str, images: &[String]) -> Vec<(String, String)> {
    images
        .iter()
        .map(|i| (format!("docs/icebreaker/{}{i}", dossier_de(source)), format!("{}{i}", dossier_de(url))))
        .collect()
}

/// Rend chaque fichier Markdown en page (sans l'écrire) ; `docs` est le dossier `docs/icebreaker/`.
pub fn pages_markdown(fichiers: &[(String, String)], docs: &Path) -> Vec<Page> {
    let mut pages = Vec::new();
    for (source, texte) in fichiers {
        if source.starts_with("wiki/") || source == "QUESTIONS.md" || source == "A-FAIRE.md" {
            continue; // les fiches mécaniques, les questions et les todos ont leur propre gabarit
        }
        let (_entete, corps) = markdown::separer_entete(texte);
        let rendu = markdown::rendre_images(corps, source, Some(&docs.join(dossier_de(source))));
        let titre = rendu
            .titre
            .unwrap_or_else(|| source.trim_end_matches(".md").to_string());
        let url = url_de_source(source);
        let (html, icones) = icones::annoter_tableaux(&rendu.html, &site::prefixe_racine(&url));
        let mut images = images_de_page(source, &url, &rendu.images);
        images.extend(icones.into_iter().map(|i| (format!("assets/{i}"), format!("assets/{i}"))));
        pages.push(Page {
            images,
            url,
            rubrique: Rubrique::de_source(source),
            titre,
            html,
            source: source.clone(),
            gabarit: "page.html",
            extra: serde_json::Value::Null,
            ordre: 0,
            icone: None,
        });
    }
    pages
}

/// Toutes les pages du site, triées pour la barre latérale.
pub fn collecter(racine: &Path, docs: &Path) -> Result<Vec<Page>> {
    let fichiers = lire_fichiers(docs)?;
    let mut pages = pages_markdown(&fichiers, docs);
    let commits = commits::lire(racine);
    let readme = fichiers
        .iter()
        .find(|(f, _)| f == "README.md")
        .map(|(_, t)| t.as_str())
        .unwrap_or("");
    let sources = avancement::Sources {
        jalons: taches::lire_jalons(readme),
        fichiers: &fichiers,
        commits: &commits,
        url_github: commits::url_github(racine),
    };
    let toutes: Vec<taches::Tache> = fichiers
        .iter()
        .filter(|(f, _)| f == "PLAN.md" || f == "HISTORIQUE.md" || f.starts_with("PLAN-"))
        .flat_map(|(f, t)| taches::lire(t, f))
        .collect();
    let jalons = avancement::jalons(&sources, &toutes);
    pages.extend(avancement::pages(&sources, &jalons));
    let fichiers_ron = reglages::lire_dossier(&racine.join("assets").join("icebreaker"))?;
    pages.extend(reglages::pages(&fichiers_ron));
    pages.extend(creatures::pages_du_depot(racine, &fichiers_ron)?);
    pages.extend(competences::pages_du_depot(racine)?); // Icebreaker : W.17
    if let Some((_, texte)) = fichiers.iter().find(|(f, _)| f == "QUESTIONS.md") {
        pages.push(decisions::page_questions(texte));
    }
    if let Some((_, texte)) = fichiers.iter().find(|(f, _)| f == "A-FAIRE.md") {
        pages.push(todo::page(texte));
    }
    pages.push(decisions::page_index(&fichiers));
    for (source, texte) in fichiers.iter().filter(|(f, _)| f.starts_with("wiki/")) {
        pages.push(fiches::page(
            racine,
            source,
            texte,
            &fichiers,
            &jalons,
            &fichiers_ron,
            sources.url_github.as_deref(),
        )?);
    }
    pages.sort_by_key(cle_de_tri);
    // Les cartes des mécaniques sur le tableau de bord, dans l'ordre de la barre latérale.
    let mecaniques: Vec<serde_json::Value> = pages
        .iter()
        .filter(|p| p.gabarit == "fiche.html")
        .map(|p| {
            serde_json::json!({
                "titre": p.titre,
                "url": p.url,
                "resume": p.extra["resume"],
                "icone_url": p.extra["icone_url"],
                "jalons": p.extra["jalons"],
            })
        })
        .collect();
    // Icebreaker : ce qui attend l'agent (W.13) : réponses aux questions et todos ouverts.
    let compter = |gabarit: &str, cle: &str| {
        pages.iter().find(|p| p.gabarit == gabarit).map_or(0, |p| match &p.extra[cle] {
            serde_json::Value::Array(a) => a.len() as u64,
            v => v.as_u64().unwrap_or(0),
        })
    };
    let (nb_reponses, nb_todos) = (compter("questions.html", "nb_repondues"), compter("todo.html", "ouverts"));
    if let Some(accueil) = pages.iter_mut().find(|p| p.url == "index.html") {
        accueil.extra["mecaniques"] = serde_json::Value::Array(mecaniques);
        accueil.extra["nb_reponses"] = nb_reponses.into();
        accueil.extra["nb_todos"] = nb_todos.into();
    }
    Ok(pages)
}

/// `icebreaker-wiki [racine]` écrit le site statique ; `icebreaker-wiki serve [--port N] [--nom X]
/// [--branche-libre] [racine]` le sert en local avec la saisie (Icebreaker : W.12).
fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let serve = args.first().is_some_and(|a| a == "serve");
    let mut options = serveur::Options { port: 8095, nom: "Flo".to_string(), branche_libre: false };
    let mut racine = PathBuf::from(".");
    if serve {
        args.remove(0);
    }
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" if serve => options.port = args.next().context("--port N")?.parse().context("--port N")?,
            "--nom" if serve => options.nom = args.next().context("--nom X")?,
            "--branche-libre" if serve => options.branche_libre = true,
            a if a.starts_with("--") => anyhow::bail!("option inconnue : {a}"),
            _ => racine = PathBuf::from(arg),
        }
    }
    let docs = racine.join("docs").join("icebreaker");
    anyhow::ensure!(
        docs.is_dir(),
        "dossier {} introuvable : lancer depuis la racine du dépôt ou passer la racine en argument",
        docs.display()
    );
    if serve {
        return serveur::lancer(&racine, options);
    }
    let sortie = racine.join("site");
    let pages = collecter(&racine, &docs)?;
    let nb = site::ecrire(&sortie, &pages, &racine, false)?;
    println!("{nb} pages écrites dans {}", sortie.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn racine_du_depot() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    #[test]
    fn chaque_fichier_markdown_donne_une_page() {
        let racine = racine_du_depot();
        let docs = racine.join("docs/icebreaker");
        let pages = collecter(&racine, &docs).unwrap();
        let nb_md = WalkDir::new(&docs)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            .count();
        let markdown: Vec<&Page> = pages.iter().filter(|p| p.gabarit == "page.html").collect();
        let fiches = pages.iter().filter(|p| p.gabarit == "fiche.html").count();
        assert_eq!(markdown.len() + fiches + 2, nb_md); // + QUESTIONS.md et A-FAIRE.md, rendues par leur gabarit
        assert!(pages.iter().any(|p| p.url == "QUESTIONS.html" && p.gabarit == "questions.html"));
        assert!(pages.iter().any(|p| p.url == "A-FAIRE.html" && p.gabarit == "todo.html"));
        assert!(pages.iter().any(|p| p.url == "decisions/index.html"));
        assert!(pages.iter().any(|p| p.url == "fiches/mort.html" && p.rubrique == Rubrique::Mecaniques));
        assert!(pages.iter().any(|p| p.url == "readme.html" && p.source == "README.md"));
        assert_eq!(pages[0].url, "index.html");
        assert!(pages.iter().any(|p| p.url == "design/combat.html" && p.rubrique == Rubrique::Regles));
        assert!(pages.iter().any(|p| p.url == "avancement/T.html" && p.rubrique == Rubrique::Avancement));
        assert!(pages.iter().any(|p| p.url == "reglages/mort.html" && p.rubrique == Rubrique::Reglages));
        assert!(markdown.iter().all(|p| !p.titre.is_empty() && !p.html.is_empty()));
    }

    #[test]
    fn rubriques_et_urls() {
        assert_eq!(Rubrique::de_source("PLAN.md"), Rubrique::Reperes);
        assert_eq!(Rubrique::de_source("PLAN-MOD.md"), Rubrique::Plans);
        assert_eq!(Rubrique::de_source("design/ames.md"), Rubrique::Regles);
        assert_eq!(Rubrique::de_source("decisions/0016-x.md"), Rubrique::Decisions);
        assert_eq!(url_de_source("README.md"), "readme.html");
        assert_eq!(url_de_source("decisions/0016-x.md"), "decisions/0016-x.html");
    }
}
