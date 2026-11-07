//! Icebreaker : ce que le wiki local (`serve`) écrit dans la doc : réponses aux questions de `QUESTIONS.md` (W.12)
//! et todos de `A-FAIRE.md` (W.13).
//!
//! Fonctions pures sur le texte du fichier ; ses fins de ligne (CRLF ou LF) sont gardées et les lignes ajoutées sont
//! coupées à la largeur du reste de la doc.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};

use crate::decisions::QUESTION;

/// Largeur des lignes ajoutées, comme dans le reste de `docs/icebreaker/`.
const LARGEUR: usize = 118;

/// Les lignes d'un texte, sans fin de ligne, et la fin de ligne du fichier.
fn lignes(texte: &str) -> (Vec<String>, &'static str) {
    let fin = if texte.contains("\r\n") { "\r\n" } else { "\n" };
    (texte.lines().map(|l| l.trim_end_matches('\r').to_string()).collect(), fin)
}

fn joindre(lignes: &[String], fin: &str) -> String {
    let mut texte = lignes.join(fin);
    texte.push_str(fin);
    texte
}

/// Coupe `contenu` en lignes de `LARGEUR` caractères au plus : `retrait1` devant la première, `retrait` devant les
/// suivantes. Les retours à la ligne de `contenu` deviennent des espaces.
fn couper(contenu: &str, retrait1: &str, retrait: &str) -> Vec<String> {
    let mut sortie = Vec::new();
    let mut courante = String::new();
    for mot in contenu.split_whitespace() {
        if courante.is_empty() {
            courante = format!("{retrait1}{mot}");
        } else if courante.chars().count() + 1 + mot.chars().count() > LARGEUR {
            sortie.push(std::mem::take(&mut courante));
            courante = format!("{retrait}{mot}");
        } else {
            courante.push(' ');
            courante.push_str(mot);
        }
    }
    if !courante.is_empty() {
        sortie.push(courante);
    }
    sortie
}

/// Un texte saisi, sur une ligne : espaces repliés, vide refusé.
fn une_ligne(texte: &str, quoi: &str) -> Result<String> {
    let texte = texte.split_whitespace().collect::<Vec<_>>().join(" ");
    if texte.is_empty() {
        bail!("{quoi} vide");
    }
    Ok(texte)
}

/// Retire les lignes `[debut, fin)` sans laisser deux lignes vides de suite.
fn retirer(l: &mut Vec<String>, debut: usize, fin: usize) {
    l.drain(debut..fin);
    if debut > 0 && debut < l.len() && l[debut - 1].trim().is_empty() && l[debut].trim().is_empty() {
        l.remove(debut);
    }
}

/// Les lignes `[debut, fin)` de la question `numero` : sa ligne `- **Qn …` et ses continuations en retrait.
fn bloc_question(lignes: &[String], numero: &str) -> Result<(usize, usize)> {
    if !numero.starts_with('Q') || numero.len() < 2 || !numero[1..].bytes().all(|b| b.is_ascii_digit()) {
        bail!("numéro de question invalide : {numero}");
    }
    let tete = format!("- **{numero} ");
    let Some(debut) = lignes.iter().position(|l| l.starts_with(&tete)) else {
        bail!("{numero} introuvable dans QUESTIONS.md (déjà traitée ?)");
    };
    let mut fin = debut + 1;
    while fin < lignes.len() && lignes[fin].starts_with("  ") {
        fin += 1;
    }
    Ok((debut, fin))
}

/// Ajoute sous la question `numero` la ligne `*Réponse de <nom> (<date>)* : <reponse>`.
pub fn repondre(texte: &str, numero: &str, reponse: &str, nom: &str, date: &str) -> Result<String> {
    let reponse = une_ligne(reponse, "réponse")?;
    let (mut l, fin) = lignes(texte);
    let (_, apres) = bloc_question(&l, numero)?;
    let nouvelles = couper(&format!("*Réponse de {nom} ({date})* : {reponse}"), "  ", "  ");
    l.splice(apres..apres, nouvelles);
    Ok(joindre(&l, fin))
}

/// Retire la question `numero` du fichier.
pub fn supprimer_question(texte: &str, numero: &str) -> Result<String> {
    let (mut l, fin) = lignes(texte);
    let (debut, apres) = bloc_question(&l, numero)?;
    retirer(&mut l, debut, apres);
    Ok(joindre(&l, fin))
}

/// Retire la question `numero` et note dans « Tranché » qu'elle est obsolète.
pub fn rendre_obsolete(texte: &str, numero: &str, nom: &str, date: &str) -> Result<String> {
    let (mut l, fin) = lignes(texte);
    let (debut, apres) = bloc_question(&l, numero)?;
    let bloc: Vec<&str> = l[debut..apres].iter().map(|x| x.trim()).collect();
    let bloc = bloc.join(" ");
    let titre = QUESTION
        .captures(bloc.trim_start_matches("- "))
        .map(|c| c[2].trim().trim_end_matches(['.', ':']).trim().to_string())
        .unwrap_or_default();
    retirer(&mut l, debut, apres);
    let ligne = couper(&format!("- **{date} ({nom})** : {numero} *{titre}* : obsolète."), "", "  ");
    ajouter_en_fin_de_section(&mut l, "## Tranché", ligne)?;
    Ok(joindre(&l, fin))
}

/// La section `## <titre>…` : (ligne du titre, fin de la section, sans les lignes vides du bas).
fn section(l: &[String], titre: &str) -> Result<(usize, usize)> {
    let Some(tete) = l.iter().position(|x| x.starts_with(titre)) else {
        bail!("section « {} » introuvable", titre.trim_start_matches("## "));
    };
    let mut fin = l[tete + 1..]
        .iter()
        .position(|x| x.starts_with("## "))
        .map_or(l.len(), |i| tete + 1 + i);
    while fin > tete + 1 && l[fin - 1].trim().is_empty() {
        fin -= 1;
    }
    Ok((tete, fin))
}

/// Ajoute `nouvelles` à la fin de la section `titre`, avec une ligne vide après le titre et avant la section
/// suivante.
fn ajouter_en_fin_de_section(l: &mut Vec<String>, titre: &str, mut nouvelles: Vec<String>) -> Result<()> {
    let (tete, fin) = section(l, titre)?;
    if fin == tete + 1 {
        nouvelles.insert(0, String::new());
    }
    if l.get(fin).is_some_and(|x| x.starts_with("## ")) {
        nouvelles.push(String::new());
    }
    l.splice(fin..fin, nouvelles);
    Ok(())
}

// Icebreaker : les todos de `A-FAIRE.md` (W.13) : `- [ ] **2026-11-07 (Flo)** : texte` dans « Ouvert », puis
// `- [x] … → où c'est parti` dans « Pris en compte ».

const OUVERT: &str = "## Ouvert";
const PRIS: &str = "## Pris en compte";

/// Les todos ouverts : (début, fin) de chaque entrée, continuations comprises.
fn blocs_ouverts(l: &[String]) -> Result<Vec<(usize, usize)>> {
    let (tete, fin) = section(l, OUVERT)?;
    let mut blocs = Vec::new();
    let mut i = tete + 1;
    while i < fin {
        if l[i].starts_with("- ") {
            let mut j = i + 1;
            while j < fin && l[j].starts_with("  ") {
                j += 1;
            }
            blocs.push((i, j));
            i = j;
        } else {
            i += 1;
        }
    }
    Ok(blocs)
}

/// Le texte d'une entrée sur une ligne, sans le `- ` (ce que la page envoie pour vérifier qu'on touche la bonne).
pub fn texte_d_entree(lignes: &[String]) -> String {
    let joint: Vec<&str> = lignes.iter().map(|x| x.trim()).collect();
    joint.join(" ").trim_start_matches("- ").to_string()
}

/// Le bloc du todo ouvert n° `index`, s'il a toujours le texte `attendu`.
fn todo_verifie(l: &[String], index: usize, attendu: &str) -> Result<(usize, usize)> {
    let blocs = blocs_ouverts(l)?;
    let attendu = attendu.split_whitespace().collect::<Vec<_>>().join(" ");
    match blocs.get(index) {
        Some(&(debut, fin)) if texte_d_entree(&l[debut..fin]) == attendu => Ok((debut, fin)),
        _ => bail!("A-FAIRE.md a changé depuis l'affichage de la page : recharger et recommencer"),
    }
}

/// Ajoute un todo à la fin de « Ouvert ».
pub fn ajouter_todo(texte: &str, contenu: &str, nom: &str, date: &str) -> Result<String> {
    let contenu = une_ligne(contenu, "todo")?;
    let (mut l, fin) = lignes(texte);
    let nouvelles = couper(&format!("- [ ] **{date} ({nom})** : {contenu}"), "", "  ");
    ajouter_en_fin_de_section(&mut l, OUVERT, nouvelles)?;
    Ok(joindre(&l, fin))
}

/// Coche le todo ouvert n° `index` : il passe dans « Pris en compte » avec « → fait (nom, date) ».
pub fn cocher_todo(texte: &str, index: usize, attendu: &str, nom: &str, date: &str) -> Result<String> {
    let (mut l, fin) = lignes(texte);
    let (debut, apres) = todo_verifie(&l, index, attendu)?;
    let entree = texte_d_entree(&l[debut..apres]);
    retirer(&mut l, debut, apres);
    let entree = entree.replacen("[ ]", "[x]", 1);
    let nouvelles = couper(&format!("- {entree} → fait ({nom}, {date})"), "", "  ");
    ajouter_en_fin_de_section(&mut l, PRIS, nouvelles)?;
    Ok(joindre(&l, fin))
}

/// Retire le todo ouvert n° `index`.
pub fn supprimer_todo(texte: &str, index: usize, attendu: &str) -> Result<String> {
    let (mut l, fin) = lignes(texte);
    let (debut, apres) = todo_verifie(&l, index, attendu)?;
    retirer(&mut l, debut, apres);
    Ok(joindre(&l, fin))
}

// Icebreaker : ce qui est vu dans le vrai client (W.16).

/// Ajoute « Vu en jeu (nom, date) » (avec le commentaire s'il y en a un) à la fin de la tâche cochée `id` d'un plan ;
/// la tâche ne compte plus dans « à voir dans le vrai client ».
pub fn marquer_vu(texte: &str, id: &str, commentaire: &str, nom: &str, date: &str) -> Result<String> {
    let valide = id.split_once('.').is_some_and(|(lettres, n)| {
        !lettres.is_empty()
            && lettres.bytes().all(|b| b.is_ascii_uppercase())
            && !n.is_empty()
            && n.bytes().all(|b| b.is_ascii_digit())
    });
    if !valide {
        bail!("identifiant de tâche invalide : {id}");
    }
    let (mut l, fin) = lignes(texte);
    let tete = format!("- [x] **{id} ");
    let Some(debut) = l.iter().position(|x| x.starts_with(&tete)) else {
        bail!("tâche {id} introuvable (ou pas cochée) dans ce fichier");
    };
    let mut apres = debut + 1;
    while apres < l.len() && l[apres].starts_with("  ") && !l[apres].trim_start().starts_with("- ") {
        apres += 1;
    }
    let commentaire = commentaire.split_whitespace().collect::<Vec<_>>().join(" ");
    let note = if commentaire.is_empty() {
        format!("Vu en jeu ({nom}, {date}).")
    } else {
        format!("Vu en jeu ({nom}, {date}) : {}.", commentaire.trim_end_matches('.'))
    };
    let derniere = apres - 1;
    let (retrait1, contenu) = if derniere == debut {
        ("", l[derniere].clone())
    } else {
        ("  ", l[derniere].trim().to_string())
    };
    let nouvelles = couper(&format!("{contenu} {note}"), retrait1, "  ");
    l.splice(derniere..apres, nouvelles);
    Ok(joindre(&l, fin))
}

/// La date du jour (UTC) au format `AAAA-MM-JJ`.
pub fn aujourd_hui() -> String {
    let secondes = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as i64;
    // Jours depuis 1970 → date du calendrier (algorithme « civil_from_days » de H. Hinnant).
    let z = secondes.div_euclid(86_400) + 719_468;
    let ere = z.div_euclid(146_097);
    let jour_ere = z - ere * 146_097;
    let annee_ere = (jour_ere - jour_ere / 1460 + jour_ere / 36_524 - jour_ere / 146_096) / 365;
    let jour_annee = jour_ere - (365 * annee_ere + annee_ere / 4 - annee_ere / 100);
    let mp = (5 * jour_annee + 2) / 153;
    let jour = jour_annee - (153 * mp + 2) / 5 + 1;
    let mois = if mp < 10 { mp + 3 } else { mp - 9 };
    let annee = annee_ere + ere * 400 + i64::from(mois <= 2);
    format!("{annee:04}-{mois:02}-{jour:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decisions::lire_questions;

    const TEXTE: &str = "# Questions\r\n\r\n## 1. Stratégie\r\n\r\n- **Q1 Contenu de la démo.** Le scénario suppose\r\n  la route. *Qui* : Flo.\r\n- **Q12 Le moteur** : Veloren ou Godot.\r\n\r\n## Tranché\r\n\r\nTexte.\r\n\r\n- **2026-10-19 (donjon)** : mort = cadavre.\r\n";

    #[test]
    fn repondre_garde_le_fichier() {
        let t = repondre(TEXTE, "Q1", "Une démo\nsans route.", "Flo", "2026-11-07").unwrap();
        assert!(t.contains("  la route. *Qui* : Flo.\r\n  *Réponse de Flo (2026-11-07)* : Une démo sans route.\r\n- **Q12"));
        assert!(!t.contains("\n\n") && t.matches('\n').count() == t.matches("\r\n").count());
        let (q, _) = lire_questions(&t);
        assert_eq!(q.len(), 2);
        assert_eq!(q[0].reponses[0].texte_html, "Une démo sans route.");
        assert!(q[1].reponses.is_empty());
        // Q1 ne doit pas attraper Q12, ni l'inverse.
        let t = repondre(TEXTE, "Q12", "Veloren", "Flo", "2026-11-07").unwrap();
        assert_eq!(lire_questions(&t).0[1].reponses.len(), 1);
    }

    #[test]
    fn longue_reponse_coupee() {
        let longue = "mot ".repeat(80);
        let t = repondre(TEXTE, "Q12", &longue, "Flo", "2026-11-07").unwrap();
        assert!(t.lines().all(|l| l.trim_end_matches('\r').chars().count() <= LARGEUR));
        assert_eq!(lire_questions(&t).0[1].reponses.len(), 1);
    }

    #[test]
    fn supprimer_et_obsolete() {
        let t = supprimer_question(TEXTE, "Q1").unwrap();
        let (q, tr) = lire_questions(&t);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].numero, "Q12");
        assert_eq!(tr.len(), 1);
        let t = rendre_obsolete(TEXTE, "Q1", "Flo", "2026-11-07").unwrap();
        let (q, tr) = lire_questions(&t);
        assert_eq!(q.len(), 1);
        assert_eq!(tr.len(), 2);
        assert_eq!(tr[1].date, "2026-11-07");
        assert_eq!(tr[1].texte_html, "Q1 <em>Contenu de la démo</em> : obsolète.");
        assert!(t.ends_with("cadavre.\r\n- **2026-11-07 (Flo)** : Q1 *Contenu de la démo* : obsolète.\r\n"));
    }

    #[test]
    fn erreurs() {
        assert!(repondre(TEXTE, "Q3", "x", "Flo", "2026-11-07").is_err());
        assert!(repondre(TEXTE, "Q1", "  \n ", "Flo", "2026-11-07").is_err());
        assert!(supprimer_question(TEXTE, "Q1 ").is_err());
        assert!(supprimer_question(TEXTE, "").is_err());
    }

    #[test]
    fn sur_le_vrai_fichier() {
        let texte = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/icebreaker/QUESTIONS.md"),
        )
        .unwrap();
        let (avant, tranchees) = lire_questions(&texte);
        let premiere = &avant[0].numero;
        let t = rendre_obsolete(&texte, premiere, "Flo", "2026-11-07").unwrap();
        let (apres, tranchees_apres) = lire_questions(&t);
        assert_eq!(apres.len(), avant.len() - 1);
        assert_eq!(tranchees_apres.len(), tranchees.len() + 1);
        let t = repondre(&texte, premiere, "oui", "Flo", "2026-11-07").unwrap();
        assert_eq!(lire_questions(&t).0.len(), avant.len());
    }

    const TODO: &str = "# À faire\n\nIntro.\n\n## Ouvert\n\n## Pris en compte\n";

    #[test]
    fn todos() {
        let t = ajouter_todo(TODO, "Revoir\nla grotte", "Flo", "2026-11-07").unwrap();
        assert_eq!(t, "# À faire\n\nIntro.\n\n## Ouvert\n\n- [ ] **2026-11-07 (Flo)** : Revoir la grotte\n\n## Pris en compte\n");
        let t = ajouter_todo(&t, "Captures du donjon", "Flo", "2026-11-08").unwrap();
        let (ouverts, pris) = crate::todo::lire(&t);
        assert_eq!(ouverts.len(), 2);
        assert_eq!(ouverts[1].texte, "[ ] **2026-11-08 (Flo)** : Captures du donjon");
        assert!(pris.is_empty());
        // Mauvais texte attendu : refusé, rien ne bouge.
        assert!(cocher_todo(&t, 0, "autre chose", "Flo", "2026-11-09").is_err());
        assert!(supprimer_todo(&t, 5, "x").is_err());
        let c = cocher_todo(&t, 0, &ouverts[0].texte, "Flo", "2026-11-09").unwrap();
        assert!(c.ends_with(
            "## Pris en compte\n\n- [x] **2026-11-07 (Flo)** : Revoir la grotte → fait (Flo, 2026-11-09)\n"
        ));
        let (ouverts, pris) = crate::todo::lire(&c);
        assert_eq!((ouverts.len(), pris.len()), (1, 1));
        let s = supprimer_todo(&c, 0, &ouverts[0].texte).unwrap();
        let (ouverts, pris) = crate::todo::lire(&s);
        assert_eq!((ouverts.len(), pris.len()), (0, 1));
        assert!(s.contains("## Ouvert\n\n## Pris en compte"));
        assert!(ajouter_todo(TODO, " ", "Flo", "2026-11-07").is_err());
    }

    #[test]
    fn le_vrai_todo() {
        let texte = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/icebreaker/A-FAIRE.md"),
        )
        .unwrap();
        let t = ajouter_todo(&texte, "essai", "Flo", "2026-11-07").unwrap();
        assert_eq!(crate::todo::lire(&t).0.len(), crate::todo::lire(&texte).0.len() + 1);
    }

    #[test]
    fn vu_en_jeu() {
        let plan = "## Jalon X\r\n\r\n- [x] **X.1 Vue U** — fermée au départ ; pas vu en jeu.\r\n  Suite du détail. 6107c3c00d\r\n- [x] **X.11 Plan** — pas vu en jeu. c9d929adf2\r\n- [ ] **X.2 Reste** — à faire.\r\n";
        let t = marquer_vu(plan, "X.1", "", "Flo", "2026-11-07").unwrap();
        assert!(t.contains("  Suite du détail. 6107c3c00d Vu en jeu (Flo, 2026-11-07).\r\n- [x] **X.11"));
        let taches = crate::taches::lire(&t, "PLAN.md");
        assert!(!taches[0].a_voir_en_jeu);
        assert!(taches[1].a_voir_en_jeu);
        let t = marquer_vu(plan, "X.11", "marche à deux.", "Flo", "2026-11-07").unwrap();
        assert!(t.contains("c9d929adf2 Vu en jeu (Flo, 2026-11-07) : marche à deux.\r\n"));
        assert!(!crate::taches::lire(&t, "PLAN.md")[1].a_voir_en_jeu);
        let longue = marquer_vu(plan, "X.11", &"mot ".repeat(40), "Flo", "2026-11-07").unwrap();
        assert!(longue.lines().all(|x| x.trim_end_matches('\r').chars().count() <= LARGEUR));
        assert_eq!(crate::taches::lire(&longue, "PLAN.md").len(), 3);
        assert!(marquer_vu(plan, "X.2", "", "Flo", "2026-11-07").is_err()); // pas cochée
        assert!(marquer_vu(plan, "X.9", "", "Flo", "2026-11-07").is_err());
        assert!(marquer_vu(plan, "x.1", "", "Flo", "2026-11-07").is_err());
    }

    #[test]
    fn date_du_jour() {
        let d = aujourd_hui();
        assert_eq!(d.len(), 10);
        assert!(d.as_str() >= "2026-01-01");
    }
}
