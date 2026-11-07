//! Icebreaker : la page « À faire » (W.13), rendue depuis `A-FAIRE.md` : todos ouverts, puis ceux pris en compte.

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::{Page, Rubrique, decisions, markdown};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Todo {
    /// Rang parmi les todos de sa section (ce que la page renvoie avec `texte` pour cocher ou supprimer).
    pub index: usize,
    /// L'entrée sur une ligne, sans le `- ` (voir `edition::texte_d_entree`).
    pub texte: String,
    pub date: String,
    pub nom: String,
    pub texte_html: String,
}

static ENTREE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[[ xX]\]\s*\*\*(\d{4}-\d{2}-\d{2})\s*(?:\(([^)]*)\))?\*\*\s*:?\s*(.*)$").unwrap());

/// Les todos ouverts et ceux pris en compte.
pub fn lire(texte: &str) -> (Vec<Todo>, Vec<Todo>) {
    let mut ouverts = Vec::new();
    let mut pris = Vec::new();
    for (section, entree) in decisions::lignes_de_liste(texte) {
        let liste = match section.as_str() {
            "Ouvert" => &mut ouverts,
            "Pris en compte" => &mut pris,
            _ => continue,
        };
        let (date, nom, corps) = match ENTREE.captures(&entree) {
            Some(c) => (
                c[1].to_string(),
                c.get(2).map_or("", |m| m.as_str()).to_string(),
                c[3].to_string(),
            ),
            None => (String::new(), String::new(), entree.trim_start_matches(['[', ' ', 'x', ']']).to_string()),
        };
        liste.push(Todo {
            index: liste.len(),
            texte: entree,
            date,
            nom,
            texte_html: markdown::rendre_inline(&corps),
        });
    }
    (ouverts, pris)
}

#[derive(Serialize)]
struct TodoVue {
    intro_html: String,
    ouverts: Vec<Todo>,
    pris: Vec<Todo>,
}

/// La page « À faire », à la place du rendu brut de `A-FAIRE.md`.
pub fn page(texte: &str) -> Page {
    let (ouverts, pris) = lire(texte);
    let intro = texte
        .split("\n## ")
        .next()
        .map(|t| t.split_once('\n').map(|(_, r)| r).unwrap_or(""))
        .unwrap_or("");
    let vue = TodoVue {
        intro_html: markdown::rendre(intro, "A-FAIRE.md").html,
        ouverts,
        pris: pris.into_iter().rev().collect(),
    };
    Page {
        source: "A-FAIRE.md".to_string(),
        url: "A-FAIRE.html".to_string(),
        rubrique: Rubrique::Reperes,
        titre: "À faire".to_string(),
        html: String::new(),
        gabarit: "todo.html",
        extra: serde_json::to_value(&vue).expect("sérialisation des todos"),
        ordre: 0,
        icone: None,
        images: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture() {
        let texte = "# À faire\n\nIntro.\n\n## Ouvert\n\n- [ ] **2026-11-07 (Flo)** : Revoir `grotte.ron`\n  et la pioche.\n- noté à la main\n\n## Pris en compte\n\n- [x] **2026-11-06 (Sayanel)** : x → PLAN.md, X.14\n";
        let (o, p) = lire(texte);
        assert_eq!(o.len(), 2);
        assert_eq!(o[0].date, "2026-11-07");
        assert_eq!(o[0].nom, "Flo");
        assert_eq!(o[0].texte, "[ ] **2026-11-07 (Flo)** : Revoir `grotte.ron` et la pioche.");
        assert_eq!(o[0].texte_html, "Revoir <code>grotte.ron</code> et la pioche.");
        assert_eq!(o[1].index, 1);
        assert_eq!(o[1].texte_html, "noté à la main");
        assert_eq!(p[0].nom, "Sayanel");
        let page = page(texte);
        assert!(page.extra["intro_html"].as_str().unwrap().contains("Intro."));
    }
}
