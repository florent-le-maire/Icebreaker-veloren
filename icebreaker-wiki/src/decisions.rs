//! Les questions pour l'équipe (`QUESTIONS.md` : ouvertes par thème, puis « Tranché ») et l'index des notes de
//! décision (`decisions/NNNN-*.md` : date, statut).

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::{Page, Rubrique, markdown};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Question {
    pub numero: String,
    pub titre: String,
    pub texte_html: String,
    pub section: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Tranchee {
    pub date: String,
    pub contexte_html: String,
    pub texte_html: String,
}

static QUESTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\*\*(Q\d+) ([^*]+?)\*\*\s*:?\s*(.*)$").unwrap());
static TRANCHEE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\*\*(\d{4}-\d{2}-\d{2})\s*(?:\(([^)]*)\))?\*\*\s*:?\s*(.*)$").unwrap());

/// Les lignes `- …` d'un texte, continuations jointes, avec le titre de leur section `##`.
pub fn lignes_de_liste(texte: &str) -> Vec<(String, String)> {
    let mut sortie = Vec::new();
    let mut section = String::new();
    let mut courante: Option<String> = None;
    for ligne in texte.lines().map(|l| l.trim_end_matches('\r')) {
        if let Some(titre) = ligne.strip_prefix("## ") {
            if let Some(c) = courante.take() {
                sortie.push((section.clone(), c));
            }
            section = titre.trim().to_string();
        } else if let Some(reste) = ligne.strip_prefix("- ") {
            if let Some(c) = courante.take() {
                sortie.push((section.clone(), c));
            }
            courante = Some(reste.trim().to_string());
        } else if ligne.starts_with("  ") && !ligne.trim_start().starts_with("- ") {
            if let Some(c) = courante.as_mut() {
                c.push(' ');
                c.push_str(ligne.trim());
            }
        } else if let Some(c) = courante.take() {
            sortie.push((section.clone(), c));
        }
    }
    if let Some(c) = courante.take() {
        sortie.push((section, c));
    }
    sortie
}

/// Les questions ouvertes et les petites décisions tranchées de `QUESTIONS.md`.
pub fn lire_questions(texte: &str) -> (Vec<Question>, Vec<Tranchee>) {
    let mut questions = Vec::new();
    let mut tranchees = Vec::new();
    for (section, ligne) in lignes_de_liste(texte) {
        if section.starts_with("Tranché") {
            if let Some(cap) = TRANCHEE.captures(&ligne) {
                tranchees.push(Tranchee {
                    date: cap[1].to_string(),
                    contexte_html: markdown::rendre_inline(cap.get(2).map_or("", |m| m.as_str())),
                    texte_html: markdown::rendre_inline(&cap[3]),
                });
            }
        } else if let Some(cap) = QUESTION.captures(&ligne) {
            questions.push(Question {
                numero: cap[1].to_string(),
                titre: cap[2].trim().trim_end_matches(['.', ':']).trim().to_string(),
                texte_html: markdown::rendre_inline(&cap[3]),
                section: section.clone(),
            });
        }
    }
    (questions, tranchees)
}

#[derive(Serialize)]
struct SectionVue {
    nom: String,
    questions: Vec<Question>,
}

#[derive(Serialize)]
struct QuestionsVue {
    intro_html: String,
    nb_ouvertes: usize,
    nb_tranchees: usize,
    sections: Vec<SectionVue>,
    tranchees: Vec<Tranchee>,
}

/// La page des questions, à la place du rendu brut de `QUESTIONS.md`.
pub fn page_questions(texte: &str) -> Page {
    let (questions, tranchees) = lire_questions(texte);
    let intro = texte
        .split("\n## ")
        .next()
        .map(|t| t.split_once('\n').map(|(_, r)| r).unwrap_or(""))
        .unwrap_or("");
    let mut sections: Vec<SectionVue> = Vec::new();
    for q in questions {
        match sections.iter_mut().find(|s| s.nom == q.section) {
            Some(s) => s.questions.push(q),
            None => sections.push(SectionVue { nom: q.section.clone(), questions: vec![q] }),
        }
    }
    let vue = QuestionsVue {
        intro_html: markdown::rendre(intro, "QUESTIONS.md").html,
        nb_ouvertes: sections.iter().map(|s| s.questions.len()).sum(),
        nb_tranchees: tranchees.len(),
        sections,
        tranchees: tranchees.into_iter().rev().collect(),
    };
    Page {
        source: "QUESTIONS.md".to_string(),
        url: "QUESTIONS.html".to_string(),
        rubrique: Rubrique::Reperes,
        titre: "Questions pour l'équipe".to_string(),
        html: String::new(),
        gabarit: "questions.html",
        extra: serde_json::to_value(&vue).expect("sérialisation des questions"),
        ordre: 0,
        icone: None,
        images: Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Decision {
    pub numero: String,
    pub titre: String,
    pub date: String,
    pub statut_html: String,
    pub url: String,
}

/// L'en-tête d'une note de décision : `# NNNN — Titre`, `- **Date** : …`, `- **Statut** : …`.
pub fn lire_decision(texte: &str, source: &str) -> Option<Decision> {
    let h1 = texte.lines().find_map(|l| l.trim_end_matches('\r').strip_prefix("# "))?;
    let (numero, titre) = h1.split_once(" — ").unwrap_or((h1, ""));
    let champ = |nom: &str| {
        lignes_de_liste(texte)
            .into_iter()
            .find_map(|(_, l)| l.strip_prefix(&format!("**{nom}** :")).map(|r| r.trim().to_string()))
            .unwrap_or_default()
    };
    Some(Decision {
        numero: numero.trim().to_string(),
        titre: titre.trim().to_string(),
        date: champ("Date"),
        statut_html: markdown::rendre_inline(&champ("Statut")),
        url: crate::url_de_source(source),
    })
}

/// L'index des décisions, en tête de la rubrique.
pub fn page_index(fichiers: &[(String, String)]) -> Page {
    let decisions: Vec<Decision> = fichiers
        .iter()
        .filter(|(f, _)| f.starts_with("decisions/"))
        .filter_map(|(f, t)| lire_decision(t, f))
        .collect();
    Page {
        source: "decisions/".to_string(),
        url: "decisions/index.html".to_string(),
        rubrique: Rubrique::Decisions,
        titre: "Les décisions".to_string(),
        html: String::new(),
        gabarit: "decisions.html",
        extra: serde_json::json!({ "decisions": decisions }),
        ordre: 0,
        icone: None,
        images: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn questions_et_tranchees() {
        let texte = "# Questions\n\nIntro.\n\n## 1. Stratégie\n\n- **Q1 Contenu de la démo.** Le scénario suppose\n  la route. *Qui* : Flo.\n\
- **Q2 Le moteur après la démo** : Veloren ou Godot. *Qui* : équipe.\n\n## Tranché\n\nTexte.\n\n\
- **2026-10-19 (donjon, `design/donjon.md`)** : mort = cadavre.\n- **2026-11-03 (`decisions/0017`)** : charrette.\n";
        let (q, t) = lire_questions(texte);
        assert_eq!(q.len(), 2);
        assert_eq!(q[0].numero, "Q1");
        assert_eq!(q[0].titre, "Contenu de la démo");
        assert!(q[0].texte_html.contains("Le scénario suppose la route."));
        assert_eq!(q[0].section, "1. Stratégie");
        assert_eq!(q[1].titre, "Le moteur après la démo");
        assert_eq!(q[1].texte_html, "Veloren ou Godot. <em>Qui</em> : équipe.");
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].date, "2026-10-19");
        assert!(t[0].contexte_html.contains("<code>design/donjon.md</code>"));
        assert_eq!(t[1].texte_html, "charrette.");
        let page = page_questions(texte);
        assert_eq!(page.extra["nb_ouvertes"], 2);
        assert!(page.extra["intro_html"].as_str().unwrap().contains("Intro."));
    }

    #[test]
    fn en_tete_de_decision() {
        let texte = "# 0017 — La charrette et le registre commun\n\n- **Date** : 2026-11-03\n- **Statut** : choisi par Flo le 2026-11-03 ;\n  remplace la Décision 1\n- **Règles** : x\n\n## Problème\n";
        let d = lire_decision(texte, "decisions/0017-charrette.md").unwrap();
        assert_eq!(d.numero, "0017");
        assert_eq!(d.titre, "La charrette et le registre commun");
        assert_eq!(d.date, "2026-11-03");
        assert_eq!(d.statut_html, "choisi par Flo le 2026-11-03 ; remplace la Décision 1");
        assert_eq!(d.url, "decisions/0017-charrette.html");
    }

    #[test]
    fn les_vraies_questions() {
        let texte = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/icebreaker/QUESTIONS.md"),
        )
        .unwrap();
        let (q, t) = lire_questions(&texte);
        assert!(q.len() >= 20, "{} questions", q.len());
        assert!(t.len() >= 5, "{} tranchées", t.len());
        assert!(q.iter().any(|x| x.numero == "Q1"));
    }
}
