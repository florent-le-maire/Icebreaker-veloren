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
const STYLE: &str = include_str!("../static/style.css");

#[derive(Serialize)]
struct LienNav {
    titre: String,
    url: String,
    actif: bool,
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
        Rubrique::Decisions | Rubrique::Avancement | Rubrique::Reglages | Rubrique::Mecaniques => titre,
        _ => titre.split(" — ").next().unwrap_or(titre),
    };
    let titre = match titre.find(" (") {
        Some(i) => &titre[..i],
        None => titre,
    };
    titre.trim().to_string()
}

/// Écrit toutes les pages et la feuille de style dans `sortie` ; renvoie le nombre de pages.
pub fn ecrire(sortie: &Path, pages: &[Page]) -> Result<usize> {
    let env = environnement()?;
    std::fs::create_dir_all(sortie)?;
    std::fs::write(sortie.join("style.css"), STYLE)?;
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
            })
            .with_context(|| format!("rendu de {}", page.source))?;
        let chemin = sortie.join(&page.url);
        if let Some(parent) = chemin.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&chemin, html).with_context(|| format!("écriture de {}", chemin.display()))?;
    }
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
        let nb = ecrire(&dossier, &pages).unwrap();
        assert_eq!(nb, 2);
        let combat = std::fs::read_to_string(dossier.join("design/combat.html")).unwrap();
        assert!(combat.contains("<p>règles</p>"));
        assert!(combat.contains("href=\"../style.css\""));
        assert!(combat.contains("href=\"../index.html\""));
        assert!(combat.contains("href=\"../readme.html\""));
        assert!(combat.contains("Lire les docs"));
        assert!(dossier.join("style.css").is_file());
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
