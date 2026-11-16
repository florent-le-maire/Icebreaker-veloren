//! Icebreaker : la page « Feuille de route » (W.18), rendue depuis `PLAN-PLAYTEST.md` : les chantiers d'avant
//! playtest (tableau « Vue d'ensemble »), l'avancement de chacun d'après ses tâches, et ce qui le retient.

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::avancement::{JalonVue, TacheVue};
use crate::taches::{self, Tache};
use crate::{Page, Rubrique, markdown};

pub const SOURCE: &str = "PLAN-PLAYTEST.md";

/// Une ligne du tableau « Vue d'ensemble » : `| # | Chantier | Tâches | Worktree | État |`.
#[derive(Debug, Clone, PartialEq)]
pub struct Chantier {
    pub numero: u32,
    pub titre: String,
    pub worktree: String,
    pub etat: String,
}

#[derive(Serialize)]
struct TacheDeChantier<'a> {
    /// Page du jalon de la tâche, avec son ancre.
    url: String,
    tache: &'a TacheVue,
}

#[derive(Serialize)]
struct ChantierVue<'a> {
    numero: u32,
    titre: String,
    worktree: String,
    etat_html: String,
    /// `fini`, `en-cours`, `a-faire`, ou `sans-tache` (chantier décrit sans tâche).
    statut: &'static str,
    statut_nom: &'static str,
    nb_faites: usize,
    nb_total: usize,
    taches: Vec<TacheDeChantier<'a>>,
    /// Ce que dit le plan avant les tâches (la demande, ce qui existe).
    contexte_html: String,
}

static QUESTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bQ(\d+)\b").unwrap());

/// Lit le tableau « Vue d'ensemble ».
pub fn lire_chantiers(texte: &str) -> Vec<Chantier> {
    let Some(section) = taches::extraire_section(texte, "## Vue d'ensemble") else {
        return Vec::new();
    };
    section
        .lines()
        .filter_map(|ligne| {
            let cellules: Vec<&str> = ligne.trim().trim_matches('|').split('|').map(str::trim).collect();
            let numero = cellules.first()?.parse().ok()?;
            (cellules.len() >= 5).then(|| Chantier {
                numero,
                titre: cellules[1].to_string(),
                worktree: cellules[3].trim_matches('`').to_string(),
                etat: cellules[4].to_string(),
            })
        })
        .collect()
}

/// Le texte de la section `## <numero>. …` avant sa première tâche.
fn contexte(texte: &str, numero: u32) -> String {
    let section = taches::extraire_section(texte, &format!("## {numero}. ")).unwrap_or("");
    let mut lignes = Vec::new();
    for ligne in section.lines() {
        if ligne.starts_with("- [") {
            break;
        }
        lignes.push(ligne.trim_end_matches('\r'));
    }
    lignes.join("\n")
}

/// L'état d'un chantier en HTML, ses questions liées à la page des questions.
fn etat_html(etat: &str) -> String {
    let html = markdown::rendre_inline(etat);
    QUESTION.replace_all(&html, r#"<a href="QUESTIONS.html#Q$1">Q$1</a>"#).into_owned()
}

/// La page « Feuille de route » ; `jalons` donne les tâches déjà rendues (commits, à voir en jeu).
pub fn page(texte: &str, jalons: &[JalonVue]) -> Page {
    let texte = &texte.replace("\r\n", "\n");
    let toutes: Vec<Tache> = taches::lire(texte, SOURCE);
    let vues: Vec<ChantierVue> = lire_chantiers(texte)
        .into_iter()
        .map(|c| {
            let prefixe = format!("{}. ", c.numero);
            let siennes: Vec<TacheDeChantier> = toutes
                .iter()
                .filter(|t| t.section.starts_with(&prefixe))
                .filter_map(|t| {
                    let id = t.id.as_deref()?;
                    jalons.iter().find_map(|j| {
                        let tache = j.taches.iter().find(|v| v.id.as_deref() == Some(id) && v.fichier == SOURCE)?;
                        Some(TacheDeChantier { url: format!("{}#{id}", j.url), tache })
                    })
                })
                .collect();
            let nb_faites = siennes.iter().filter(|t| t.tache.fait).count();
            let nb_total = siennes.len();
            let commence = nb_faites > 0 || siennes.iter().any(|t| t.tache.en_cours.is_some());
            let (statut, statut_nom) = match (nb_total, nb_faites) {
                (0, _) => ("sans-tache", "sans tâche"),
                (n, f) if n == f => ("fini", "fini"),
                _ if commence => ("en-cours", "en cours"),
                _ => ("a-faire", "à faire"),
            };
            ChantierVue {
                numero: c.numero,
                etat_html: etat_html(&c.etat),
                contexte_html: markdown::rendre(&contexte(texte, c.numero), SOURCE).html,
                titre: c.titre,
                worktree: c.worktree,
                statut,
                statut_nom,
                nb_faites,
                nb_total,
                taches: siennes,
            }
        })
        .collect();
    let ordre = taches::extraire_section(texte, "## Vue d'ensemble")
        .and_then(|s| s.split("\n\n").map(|p| p.trim()).find(|p| p.starts_with("**Ordre")).map(str::to_string))
        .unwrap_or_default();
    let extra = serde_json::json!({
        "nb_faites": vues.iter().map(|c| c.nb_faites).sum::<usize>(),
        "nb_total": vues.iter().map(|c| c.nb_total).sum::<usize>(),
        "nb_finis": vues.iter().filter(|c| c.statut == "fini").count(),
        "nb_en_cours": vues.iter().filter(|c| c.statut == "en-cours").count(),
        "ordre_html": markdown::rendre(&ordre, SOURCE).html,
        "plan_url": crate::url_de_source(SOURCE),
        "chantiers": vues,
    });
    Page {
        source: "feuille-de-route".to_string(),
        url: "feuille-de-route.html".to_string(),
        rubrique: Rubrique::Reperes,
        titre: "Feuille de route".to_string(),
        html: String::new(),
        gabarit: "feuille-route.html",
        extra,
        ordre: 0,
        icone: None,
        images: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAN: &str = "# Plan\n\n## Vue d'ensemble\n\n| # | Chantier | Tâches | Worktree | État |\n|---|---|---|---|---|\n\
        | 1 | Donjon | F.10 à F.11 | `Veloren-donjon` | tranché |\n| 2 | Classes | — | — | attend (Q61) |\n\n\
        **Ordre proposé** : le 1 d'abord.\n\n## 1. Donjon : des pièces\n\n**Ce que Flo demande** : des pièces.\n\n\
        - [x] **F.10 Catalogue** — fait. 0123456789\n- [ ] **F.11 Fonctions** — à faire.\n\n## 2. Classes\n\nPlus tard.\n";

    #[test]
    fn lit_le_tableau() {
        let c = lire_chantiers(PLAN);
        assert_eq!(c.len(), 2);
        assert_eq!((c[0].numero, c[0].titre.as_str(), c[0].worktree.as_str()), (1, "Donjon", "Veloren-donjon"));
        assert_eq!(c[1].etat, "attend (Q61)");
        assert!(etat_html(&c[1].etat).contains(r#"<a href="QUESTIONS.html#Q61">Q61</a>"#));
        assert!(contexte(PLAN, 1).contains("Ce que Flo demande"));
        assert!(!contexte(PLAN, 1).contains("F.10"));
    }

    #[test]
    fn page_du_vrai_depot() {
        let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let pages = crate::collecter(&racine, &racine.join("docs/icebreaker")).unwrap();
        let page = pages.iter().find(|p| p.url == "feuille-de-route.html").expect("page de la feuille de route");
        let chantiers = page.extra["chantiers"].as_array().unwrap();
        assert!(chantiers.len() >= 9, "{} chantiers", chantiers.len());
        // Chaque tâche du plan est retrouvée dans un jalon, donc comptée.
        let nb = taches::lire(&std::fs::read_to_string(racine.join("docs/icebreaker").join(SOURCE)).unwrap(), SOURCE)
            .len();
        assert_eq!(page.extra["nb_total"], nb);
        assert!(chantiers[0]["taches"][0]["url"].as_str().unwrap().starts_with("avancement/"));
        assert!(!page.extra["ordre_html"].as_str().unwrap().is_empty());
        let accueil = pages.iter().find(|p| p.url == "index.html").unwrap();
        assert_eq!(accueil.extra["playtest"]["nb_total"], nb);
    }
}
