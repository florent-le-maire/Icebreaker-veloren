//! Écriture du site : gabarits minijinja, barre latérale, feuille de style.

use std::path::Path;

use anyhow::{Context, Result};
use minijinja::{Environment, context};
use serde::Serialize;

use crate::{Page, Rubrique};

const GABARIT_BASE: &str = include_str!("../gabarits/base.html");
const GABARIT_PAGE: &str = include_str!("../gabarits/page.html");
const GABARIT_ACCUEIL: &str = include_str!("../gabarits/accueil.html");
const GABARIT_JALON: &str = include_str!("../gabarits/jalon.html");
const GABARIT_REGLAGES: &str = include_str!("../gabarits/reglages.html");
const GABARIT_FICHE: &str = include_str!("../gabarits/fiche.html");
const GABARIT_QUESTIONS: &str = include_str!("../gabarits/questions.html");
const GABARIT_DECISIONS: &str = include_str!("../gabarits/decisions.html");
const GABARIT_TODO: &str = include_str!("../gabarits/todo.html");
const GABARIT_CREATURES: &str = include_str!("../gabarits/creatures.html"); // Icebreaker : W.15
const STYLE: &str = include_str!("../static/style.css");
const SCRIPT: &str = include_str!("../static/wiki.js");
const LOGO: &str = include_str!("../static/logo.svg");

#[derive(Serialize)]
struct LienNav {
    titre: String,
    url: String,
    actif: bool,
    /// URL de l'icône de la page, depuis la page courante.
    icone: Option<String>,
}

#[derive(Serialize)]
struct RubriqueNav {
    nom: &'static str,
    liens: Vec<LienNav>,
}

/// Préfixe à mettre devant une URL relative à la racine du site, depuis la page `url`.
pub fn prefixe_racine(url: &str) -> String {
    "../".repeat(url.matches('/').count())
}

fn environnement() -> Result<Environment<'static>> {
    let mut env = Environment::new();
    env.add_template("base.html", GABARIT_BASE)?;
    env.add_template("page.html", GABARIT_PAGE)?;
    env.add_template("accueil.html", GABARIT_ACCUEIL)?;
    env.add_template("jalon.html", GABARIT_JALON)?;
    env.add_template("reglages.html", GABARIT_REGLAGES)?;
    env.add_template("fiche.html", GABARIT_FICHE)?;
    env.add_template("questions.html", GABARIT_QUESTIONS)?;
    env.add_template("decisions.html", GABARIT_DECISIONS)?;
    env.add_template("todo.html", GABARIT_TODO)?;
    env.add_template("creatures.html", GABARIT_CREATURES)?;
    Ok(env)
}

fn navigation(pages: &[Page], courante: &Page, racine: &str) -> Vec<RubriqueNav> {
    Rubrique::toutes()
        .into_iter()
        .map(|rubrique| RubriqueNav {
            nom: rubrique.nom(),
            liens: pages
                .iter()
                .filter(|p| p.rubrique == rubrique)
                .map(|p| LienNav {
                    titre: titre_court(p),
                    url: format!("{racine}{}", p.url),
                    actif: p.url == courante.url,
                    icone: p.icone.as_ref().map(|i| format!("{racine}assets/{i}")),
                })
                .collect(),
        })
        .filter(|r| !r.liens.is_empty())
        .collect()
}

/// Titre pour la barre latérale : nom fixe pour les repères, sinon le titre débarrassé du préfixe
/// « Game design — », du sous-titre après le tiret et de la parenthèse finale.
pub fn titre_court(page: &Page) -> String {
    if page.rubrique == Rubrique::Reperes {
        if let Some(nom) = crate::nom_repere(&page.source) {
            return nom.to_string();
        }
    }
    let titre = page.titre.trim_start_matches("Game design — ");
    let titre = match page.rubrique {
        Rubrique::Decisions | Rubrique::Avancement | Rubrique::Reglages | Rubrique::Mecaniques | Rubrique::Creatures => titre,
        _ => titre.split(" — ").next().unwrap_or(titre),
    };
    let titre = match titre.find(" (") {
        Some(i) => &titre[..i],
        None => titre,
    };
    titre.trim().to_string()
}

/// Le texte brut du contenu principal d'une page rendue (entre `<main` et `</main>`), balises enlevées,
/// entités courantes décodées, espaces repliés, coupé à `max` caractères.
pub fn texte_brut(html: &str, max: usize) -> String {
    let debut = html.find("<main").map(|i| i + html[i..].find('>').map_or(0, |j| j + 1)).unwrap_or(0);
    let fin = html.find("</main>").unwrap_or(html.len());
    let mut texte = String::with_capacity(fin.saturating_sub(debut));
    let mut dans_balise = false;
    for c in html[debut..fin].chars() {
        match c {
            '<' => {
                dans_balise = true;
                texte.push(' ');
            },
            '>' if dans_balise => dans_balise = false,
            _ if !dans_balise => texte.push(c),
            _ => {},
        }
    }
    let texte = texte
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'");
    let mots: Vec<&str> = texte.split_whitespace().collect();
    let mut sortie = mots.join(" ");
    if sortie.chars().count() > max {
        sortie = sortie.chars().take(max).collect();
    }
    sortie
}

#[derive(Serialize)]
struct EntreeIndex<'a> {
    titre: &'a str,
    url: &'a str,
    rubrique: &'static str,
    texte: String,
}

/// Copie dans le site les fichiers du dépôt dont les pages ont besoin : (chemin depuis `racine`, chemin dans `sortie`).
pub fn copier_images<'a>(racine: &Path, sortie: &Path, fichiers: impl Iterator<Item = (&'a str, &'a str)>) -> Result<()> {
    for (source, cible) in fichiers {
        let cible = sortie.join(cible);
        if let Some(parent) = cible.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(racine.join(source), &cible).with_context(|| format!("copie de {source}"))?;
    }
    Ok(())
}

/// Écrit toutes les pages, la feuille de style, le script, le logo, les icônes et l'index de recherche dans
/// `sortie` (`racine` est la racine du dépôt, pour copier les images) ; renvoie le nombre de pages. `edition` :
/// formulaires de saisie (wiki local, `serve`).
pub fn ecrire(sortie: &Path, pages: &[Page], racine: &Path, edition: bool) -> Result<usize> {
    let env = environnement()?;
    std::fs::create_dir_all(sortie)?;
    std::fs::write(sortie.join("style.css"), STYLE)?;
    std::fs::write(sortie.join("wiki.js"), SCRIPT)?;
    std::fs::write(sortie.join("logo.svg"), LOGO)?;
    let icones: Vec<(String, String)> = pages
        .iter()
        .filter_map(|p| p.icone.as_deref())
        .map(|i| (format!("assets/{i}"), format!("assets/{i}")))
        .collect();
    let images = icones.iter().chain(pages.iter().flat_map(|p| p.images.iter()));
    copier_images(racine, sortie, images.map(|(s, c)| (s.as_str(), c.as_str())))?;
    let mut index: Vec<EntreeIndex> = Vec::with_capacity(pages.len());
    for page in pages {
        let racine = prefixe_racine(&page.url);
        let html = env
            .get_template(page.gabarit)?
            .render(context! {
                titre => page.titre,
                contenu => page.html,
                source => page.source,
                racine => racine,
                nav => navigation(pages, page, &racine),
                extra => page.extra,
                edition => edition,
            })
            .with_context(|| format!("rendu de {}", page.source))?;
        let chemin = sortie.join(&page.url);
        if let Some(parent) = chemin.parent() {
            std::fs::create_dir_all(parent)?;
        }
        index.push(EntreeIndex {
            titre: &page.titre,
            url: &page.url,
            rubrique: page.rubrique.nom(),
            texte: texte_brut(&html, 4000),
        });
        std::fs::write(&chemin, html).with_context(|| format!("écriture de {}", chemin.display()))?;
    }
    // Un fichier de script plutôt qu'un JSON : `fetch` est refusé sur `file://`, pas un `<script>`.
    let json = serde_json::to_string(&index)?;
    std::fs::write(
        sortie.join("recherche-index.js"),
        format!("// Icebreaker : index de recherche généré par icebreaker-wiki.\nwindow.ICEBREAKER_INDEX = {json};\n"),
    )?;
    Ok(pages.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixe_selon_la_profondeur() {
        assert_eq!(prefixe_racine("index.html"), "");
        assert_eq!(prefixe_racine("design/combat.html"), "../");
        assert_eq!(prefixe_racine("a/b/c.html"), "../../");
    }

    fn page(source: &str, rubrique: Rubrique, titre: &str) -> Page {
        Page {
            source: source.into(),
            url: crate::url_de_source(source),
            rubrique,
            titre: titre.into(),
            html: String::new(),
            gabarit: "page.html",
            extra: serde_json::Value::Null,
            ordre: 0,
            icone: None,
            images: Vec::new(),
        }
    }

    #[test]
    fn titres_courts() {
        assert_eq!(titre_court(&page("README.md", Rubrique::Reperes, "Docs — point d'entrée")), "Lire les docs");
        assert_eq!(titre_court(&page("GAME_DESIGN.md", Rubrique::Reperes, "Game design")), "Index des règles");
        assert_eq!(titre_court(&page("design/combat.md", Rubrique::Regles, "Game design — Combat")), "Combat");
        assert_eq!(
            titre_court(&page("design/ames.md", Rubrique::Regles, "Game design — Âmes (idée de Sayanel)")),
            "Âmes"
        );
        assert_eq!(
            titre_court(&page("PLAN-GROTTE.md", Rubrique::Plans, "La grotte — un lieu façon DRG (jalon K)")),
            "La grotte"
        );
        assert_eq!(
            titre_court(&page("decisions/0017-c.md", Rubrique::Decisions, "0017 — La charrette et le registre")),
            "0017 — La charrette et le registre"
        );
    }

    #[test]
    fn rendu_d_une_page() {
        let mut pages = vec![
            page("README.md", Rubrique::Reperes, "Docs Icebreaker — point d'entrée"),
            page("design/combat.md", Rubrique::Regles, "Combat"),
        ];
        pages[0].html = "<p>bonjour</p>".into();
        pages[1].html = "<p>règles</p>".into();
        let dossier = std::env::temp_dir().join(format!("icebreaker-wiki-test-{}", std::process::id()));
        let nb = ecrire(&dossier, &pages, &dossier, false).unwrap();
        assert_eq!(nb, 2);
        let combat = std::fs::read_to_string(dossier.join("design/combat.html")).unwrap();
        assert!(combat.contains("<p>règles</p>"));
        assert!(combat.contains("href=\"../style.css\""));
        assert!(combat.contains("href=\"../index.html\""));
        assert!(combat.contains("href=\"../readme.html\""));
        assert!(combat.contains("Lire les docs"));
        assert!(dossier.join("style.css").is_file());
        let index = std::fs::read_to_string(dossier.join("recherche-index.js")).unwrap();
        assert!(index.starts_with("// Icebreaker"));
        assert!(index.contains("\"url\":\"design/combat.html\""));
        assert!(index.contains("règles"));
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn texte_brut_sans_balises() {
        let html = "<html><nav>menu</nav><main class=\"c\"><h1>Titre</h1><p>a &amp; <code>b</code>   c</p></main></html>";
        assert_eq!(texte_brut(html, 100), "Titre a & b c");
        assert_eq!(texte_brut(html, 5), "Titre");
    }
}
