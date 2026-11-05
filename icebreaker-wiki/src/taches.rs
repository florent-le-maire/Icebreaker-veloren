//! Les tâches des plans : lignes `- [x] **X.n Titre** — détail … <hash>` de `PLAN.md`, `HISTORIQUE.md` et des
//! `PLAN-*.md`, et le tableau des jalons du `README.md`.

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Tache {
    /// Identifiant `X.n` ; absent pour les premières tâches du prototype.
    pub id: Option<String>,
    /// Lettre du jalon (`T` pour `T.1`), ou `Prototype` sans identifiant.
    pub jalon: String,
    pub titre: String,
    /// Le reste de la ligne, hashes retirés, en Markdown.
    pub detail: String,
    pub fait: bool,
    pub hashes: Vec<String>,
    /// Fichier d'origine, relatif à `docs/icebreaker/`.
    pub fichier: String,
    /// Titre de la section `##` qui contient la tâche.
    pub section: String,
    /// Qui l'a prise (`— *en cours : <worktree>, <date>*`).
    pub en_cours: Option<String>,
    /// Vrai quand le détail dit que ce n'est pas vérifié dans le vrai client.
    pub a_voir_en_jeu: bool,
}

/// Une ligne du tableau des jalons du README.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Jalon {
    /// Les lettres de la ligne (`S, D` en donne deux).
    pub lettres: Vec<String>,
    pub titre: String,
    pub plan: String,
    pub etat: String,
    pub worktree: String,
}

static LIGNE_TACHE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^- \[([ x])\] (.*)$").unwrap());
static ID_TITRE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\*\*([A-Z]+\.\d+)\s+([^*]+?)\*\*\s*(.*)$").unwrap());
static HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[0-9a-f]{10}\b").unwrap());
static EN_COURS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"—?\s*\*en cours : ([^*]+)\*\s*—?").unwrap());
static A_VOIR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)pas (encore )?(vus?|vérifiée?s?|parcourue?|descendue?|joué|testée?) |à (vérifier|voir) en jeu|pas vus? (dans|à l')")
        .unwrap()
});

/// Lit les tâches d'un fichier Markdown ; `fichier` est son chemin relatif, gardé dans chaque tâche.
pub fn lire(texte: &str, fichier: &str) -> Vec<Tache> {
    let mut taches = Vec::new();
    let mut section = String::new();
    // La ligne de tâche en cours de lecture (les lignes indentées qui suivent la continuent).
    let mut courante: Option<(bool, String)> = None;
    let terminer = |courante: &mut Option<(bool, String)>, taches: &mut Vec<Tache>, section: &str| {
        if let Some((fait, ligne)) = courante.take() {
            taches.push(construire(fait, &ligne, fichier, section));
        }
    };
    for ligne in texte.lines() {
        let ligne = ligne.trim_end_matches('\r');
        if let Some(titre) = ligne.strip_prefix("## ") {
            terminer(&mut courante, &mut taches, &section);
            section = titre.trim().to_string();
        } else if let Some(cap) = LIGNE_TACHE.captures(ligne) {
            terminer(&mut courante, &mut taches, &section);
            courante = Some((&cap[1] == "x", cap[2].to_string()));
        } else if ligne.starts_with("  ") && !ligne.trim_start().starts_with("- ") {
            if let Some((_, texte)) = courante.as_mut() {
                texte.push(' ');
                texte.push_str(ligne.trim());
            }
        } else {
            terminer(&mut courante, &mut taches, &section);
        }
    }
    terminer(&mut courante, &mut taches, &section);
    taches
}

fn construire(fait: bool, ligne: &str, fichier: &str, section: &str) -> Tache {
    let hashes: Vec<String> = HASH.find_iter(ligne).map(|m| m.as_str().to_string()).collect();
    let sans_hash = HASH.replace_all(ligne, "");
    let en_cours = EN_COURS.captures(&sans_hash).map(|c| c[1].trim().to_string());
    let sans_en_cours = EN_COURS.replace(&sans_hash, " ");
    let texte = nettoyer(&sans_en_cours);
    let (id, titre, detail) = match ID_TITRE.captures(&texte) {
        Some(cap) => (Some(cap[1].to_string()), cap[2].trim().to_string(), nettoyer(&cap[3])),
        None => match texte.split_once(" — ") {
            Some((titre, detail)) => (None, titre.trim().to_string(), nettoyer(detail)),
            None => (None, texte.clone(), String::new()),
        },
    };
    let jalon = id
        .as_deref()
        .and_then(|id| id.split('.').next())
        .map(str::to_string)
        .unwrap_or_else(|| "Prototype".to_string());
    Tache {
        a_voir_en_jeu: fait && A_VOIR.is_match(&detail),
        id,
        jalon,
        titre,
        detail,
        fait,
        hashes,
        fichier: fichier.to_string(),
        section: section.to_string(),
        en_cours,
    }
}

/// Retire les tirets et espaces qui restent en bordure une fois les hashes et marqueurs enlevés.
fn nettoyer(texte: &str) -> String {
    let texte = texte.split_whitespace().collect::<Vec<_>>().join(" ");
    let texte = texte
        .trim_matches(|c: char| c.is_whitespace() || c == '—' || c == ',' || c == '.' && false)
        .to_string();
    let texte = texte.replace(" , ", ", ").replace(" .", ".");
    texte.trim_end_matches(" —").trim_start_matches("— ").trim().to_string()
}

/// Lit le tableau « Les jalons et leurs lettres » du README : lignes `| Lettre | Jalon | Plan | État | Worktree |`.
pub fn lire_jalons(readme: &str) -> Vec<Jalon> {
    let mut jalons = Vec::new();
    let mut dans_tableau = false;
    for ligne in readme.lines() {
        let ligne = ligne.trim_end_matches('\r').trim();
        if !ligne.starts_with('|') {
            dans_tableau = false;
            continue;
        }
        let cellules: Vec<&str> = ligne
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().trim_matches('`'))
            .collect();
        if cellules.len() < 5 {
            continue;
        }
        if cellules[0] == "Lettre" {
            dans_tableau = true;
            continue;
        }
        if !dans_tableau || cellules[0].starts_with("---") {
            continue;
        }
        jalons.push(Jalon {
            lettres: cellules[0].split(',').map(|l| l.trim().to_string()).collect(),
            titre: cellules[1].to_string(),
            plan: cellules[2].to_string(),
            etat: cellules[3].to_string(),
            worktree: cellules[4].to_string(),
        });
    }
    jalons
}

/// Le texte d'une section `## …` (sans son titre), jusqu'au `## ` suivant. `titre` est la ligne complète
/// (`## À vérifier en jeu`) ou son début.
pub fn extraire_section<'a>(texte: &'a str, titre: &str) -> Option<&'a str> {
    let mut debut = None;
    let mut position = 0;
    for ligne in texte.split_inclusive('\n') {
        let contenu = ligne.trim_end_matches(['\r', '\n']);
        if let Some(d) = debut {
            if contenu.starts_with("## ") {
                return Some(&texte[d..position]);
            }
        } else if contenu.starts_with(titre) {
            debut = Some(position + ligne.len());
        }
        position += ligne.len();
    }
    debut.map(|d| &texte[d..])
}

/// Rang d'un identifiant `X.n` pour trier les tâches d'un jalon ; les tâches sans identifiant passent en premier.
pub fn rang(id: Option<&str>) -> u32 {
    id.and_then(|id| id.split('.').nth(1))
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAN: &str = "## Jalon T — À terre (Flo, 2026-11-04)\n\nIntro.\n\n\
- [x] **T.1 Relever à 25 % et mort après 30 s à terre** — client texte à deux : relevé à 25/100 PV, mort ~30 s\n  \
après une seconde chute. 63f57ce830\n\
- [x] **T.2 Visuel du temps qui reste à terre** — barre et « Mort dans N s ». Client\n  compilé, pas vu à l'écran. bab7a8345d\n\
- [ ] **T.6 Suite** — *en cours : Veloren-mort, 2026-11-05* — à faire.\n\n## Autre\n\n\
- [x] Touches AZERTY par défaut — 2788a2682b\n\
- [x] **I.1 Deux hashes** — ni faune. Pas encore vérifié 5 min en jeu. — 13e0816046, 34dcab4f61\n";

    #[test]
    fn lit_les_taches_sur_plusieurs_lignes() {
        let taches = lire(PLAN, "PLAN.md");
        assert_eq!(taches.len(), 5);
        let t1 = &taches[0];
        assert_eq!(t1.id.as_deref(), Some("T.1"));
        assert_eq!(t1.jalon, "T");
        assert_eq!(t1.titre, "Relever à 25 % et mort après 30 s à terre");
        assert_eq!(t1.detail, "client texte à deux : relevé à 25/100 PV, mort ~30 s après une seconde chute.");
        assert_eq!(t1.hashes, vec!["63f57ce830"]);
        assert!(t1.fait);
        assert_eq!(t1.section, "Jalon T — À terre (Flo, 2026-11-04)");
        assert!(!t1.a_voir_en_jeu);
        assert!(taches[1].a_voir_en_jeu);
        let t6 = &taches[2];
        assert!(!t6.fait);
        assert_eq!(t6.en_cours.as_deref(), Some("Veloren-mort, 2026-11-05"));
        assert_eq!(t6.detail, "à faire.");
        let proto = &taches[3];
        assert_eq!(proto.id, None);
        assert_eq!(proto.jalon, "Prototype");
        assert_eq!(proto.titre, "Touches AZERTY par défaut");
        assert_eq!(proto.hashes, vec!["2788a2682b"]);
        let i1 = &taches[4];
        assert_eq!(i1.hashes, vec!["13e0816046", "34dcab4f61"]);
        assert_eq!(i1.detail, "ni faune. Pas encore vérifié 5 min en jeu.");
        assert!(i1.a_voir_en_jeu);
    }

    #[test]
    fn lit_le_tableau_des_jalons() {
        let readme = "## Les jalons\n\n| Lettre | Jalon | Plan | État (2026-11-04) | Worktree |\n|---|---|---|---|---|\n\
| I | Instances propres | `HISTORIQUE.md` | fait | — |\n| S, D | Démo | `PLAN.md` | S.1 fait | `Veloren-serveur` |\n\nTexte.\n\
| a | b | c | d | e |\n";
        let jalons = lire_jalons(readme);
        assert_eq!(jalons.len(), 2);
        assert_eq!(jalons[0].lettres, vec!["I"]);
        assert_eq!(jalons[0].plan, "HISTORIQUE.md");
        assert_eq!(jalons[1].lettres, vec!["S", "D"]);
        assert_eq!(jalons[1].worktree, "Veloren-serveur");
    }

    #[test]
    fn extrait_une_section() {
        let texte = "# T\n\n## A\n\nun\n\n## B — suite\n\ndeux\n- x\n\n## C\n";
        assert_eq!(extraire_section(texte, "## B"), Some("\ndeux\n- x\n\n"));
        assert_eq!(extraire_section(texte, "## C"), Some(""));
        assert_eq!(extraire_section(texte, "## Z"), None);
    }

    #[test]
    fn rang_des_identifiants() {
        assert_eq!(rang(Some("T.12")), 12);
        assert_eq!(rang(None), 0);
    }
}
