//! Les pages d'avancement : le tableau de bord (accueil) et une page par jalon, à partir des tâches des plans,
//! du tableau des jalons du README et des commits.

use std::collections::HashMap;

use serde::Serialize;

use crate::commits::Commit;
use crate::markdown;
use crate::taches::{self, Jalon, Tache};
use crate::{Page, Rubrique};

/// Une tâche prête pour les gabarits : son détail rendu et ses commits retrouvés.
#[derive(Debug, Clone, Serialize)]
pub struct TacheVue {
    pub id: Option<String>,
    pub titre: String,
    pub detail_html: String,
    pub fait: bool,
    pub en_cours: Option<String>,
    pub a_voir_en_jeu: bool,
    pub fichier: String,
    pub jalon: String,
    pub commits: Vec<CommitVue>,
    /// Hashes cités dans le plan mais introuvables dans `git log`.
    pub hashes_inconnus: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitVue {
    pub court: String,
    pub date: String,
    pub sujet: String,
    pub corps_html: String,
    pub teste_html: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JalonVue {
    /// Clé d'URL et de tri : la première lettre (`S` pour `S, D`), `Prototype` pour les tâches sans identifiant.
    pub cle: String,
    pub lettres: String,
    pub titre: String,
    pub plan: String,
    pub plan_url: String,
    pub etat: String,
    pub worktree: String,
    pub description_html: String,
    pub taches: Vec<TacheVue>,
    pub nb_faites: usize,
    pub nb_total: usize,
    pub nb_a_voir: usize,
    pub derniere_date: Option<String>,
    pub url: String,
}

/// Ce qu'il faut pour construire les pages : les textes des plans, les jalons du README, les commits.
pub struct Sources<'a> {
    /// (chemin relatif, texte) de chaque fichier Markdown.
    pub fichiers: &'a [(String, String)],
    pub jalons: Vec<Jalon>,
    pub commits: &'a HashMap<String, Commit>,
    pub url_github: Option<String>,
}

fn commit_vue(commit: &Commit, url_github: Option<&str>) -> CommitVue {
    CommitVue {
        court: commit.court.clone(),
        date: commit.date.clone(),
        sujet: commit.sujet.clone(),
        corps_html: markdown::rendre(&commit.corps, "").html,
        teste_html: commit.teste.as_deref().map(|t| markdown::rendre_inline(t)),
        url: url_github.map(|base| format!("{base}/commit/{}", commit.hash)),
    }
}

fn tache_vue(tache: &Tache, sources: &Sources) -> TacheVue {
    let mut commits = Vec::new();
    let mut inconnus = Vec::new();
    for hash in &tache.hashes {
        match sources.commits.get(hash) {
            Some(c) => commits.push(commit_vue(c, sources.url_github.as_deref())),
            None => inconnus.push(hash.clone()),
        }
    }
    TacheVue {
        id: tache.id.clone(),
        titre: tache.titre.clone(),
        detail_html: markdown::rendre_inline(&tache.detail),
        fait: tache.fait,
        en_cours: tache.en_cours.clone(),
        a_voir_en_jeu: tache.a_voir_en_jeu,
        fichier: tache.fichier.clone(),
        jalon: tache.jalon.clone(),
        commits,
        hashes_inconnus: inconnus,
    }
}

/// Le paragraphe d'intention d'un jalon : le premier paragraphe sous la section de sa première tâche hors
/// historique, sinon le premier paragraphe du fichier de plan.
fn description(jalon: &JalonVue, taches: &[&Tache], sources: &Sources) -> String {
    let tache = taches
        .iter()
        .find(|t| t.fichier != "HISTORIQUE.md")
        .or_else(|| taches.first());
    let texte = tache
        .and_then(|t| {
            let (_, contenu) = sources.fichiers.iter().find(|(f, _)| *f == t.fichier)?;
            let section = taches::extraire_section(contenu, &format!("## {}", t.section))?;
            premier_paragraphe(section)
        })
        .or_else(|| {
            let (_, contenu) = sources.fichiers.iter().find(|(f, _)| *f == jalon.plan)?;
            let apres_h1 = contenu.split_once("\n# ").map(|(_, r)| r).unwrap_or(contenu);
            let apres_h1 = apres_h1.split_once('\n').map(|(_, r)| r).unwrap_or("");
            premier_paragraphe(apres_h1)
        })
        .unwrap_or_default();
    markdown::rendre(&texte, "").html
}

/// Le premier paragraphe (lignes jointes) d'un texte, en sautant les lignes vides, les titres et les listes.
fn premier_paragraphe(texte: &str) -> Option<String> {
    let mut lignes = Vec::new();
    for ligne in texte.lines().map(|l| l.trim_end_matches('\r')) {
        let vide = ligne.trim().is_empty();
        if lignes.is_empty() {
            if vide || ligne.starts_with('#') || ligne.starts_with("- ") || ligne.starts_with('|') {
                continue;
            }
            lignes.push(ligne.trim());
        } else if vide || ligne.starts_with("- ") {
            break;
        } else {
            lignes.push(ligne.trim());
        }
    }
    (!lignes.is_empty()).then(|| lignes.join(" "))
}

/// Construit les vues de jalon dans l'ordre du README, puis les jalons rencontrés seulement dans les tâches.
pub fn jalons(sources: &Sources, taches: &[Tache]) -> Vec<JalonVue> {
    let mut vues: Vec<JalonVue> = Vec::new();
    let mut jalons_readme: Vec<Jalon> = sources.jalons.clone();
    // Les lettres sans ligne au README (anciens jalons, prototype) deviennent des lignes de fortune.
    let mut lettres_vues: Vec<String> = jalons_readme.iter().flat_map(|j| j.lettres.clone()).collect();
    for tache in taches {
        if !lettres_vues.contains(&tache.jalon) {
            lettres_vues.push(tache.jalon.clone());
            jalons_readme.push(Jalon {
                lettres: vec![tache.jalon.clone()],
                titre: if tache.jalon == "Prototype" {
                    "Prototype du 2026-10-28".to_string()
                } else {
                    tache.section.clone()
                },
                plan: tache.fichier.clone(),
                etat: String::new(),
                worktree: String::new(),
            });
        }
    }
    for jalon in &jalons_readme {
        let cle = jalon.lettres[0].clone();
        let mut siennes: Vec<&Tache> = taches.iter().filter(|t| jalon.lettres.contains(&t.jalon)).collect();
        siennes.sort_by_key(|t| (jalon.lettres.iter().position(|l| *l == t.jalon), taches::rang(t.id.as_deref())));
        // Une tâche cochée dans l'historique et encore listée dans un plan : on garde la version du plan.
        let mut uniques: Vec<&Tache> = Vec::new();
        for t in siennes {
            match uniques.iter().position(|u| u.id.is_some() && u.id == t.id) {
                Some(i) if uniques[i].fichier == "HISTORIQUE.md" => uniques[i] = t,
                Some(_) => {},
                None => uniques.push(t),
            }
        }
        let mut vue = JalonVue {
            url: format!("avancement/{cle}.html"),
            cle: cle.clone(),
            lettres: jalon.lettres.join(", "),
            titre: jalon.titre.clone(),
            plan: jalon.plan.clone(),
            plan_url: crate::url_de_source(&jalon.plan),
            etat: jalon.etat.clone(),
            worktree: jalon.worktree.clone(),
            description_html: String::new(),
            taches: uniques.iter().map(|t| tache_vue(t, sources)).collect(),
            nb_faites: uniques.iter().filter(|t| t.fait).count(),
            nb_total: uniques.len(),
            nb_a_voir: uniques.iter().filter(|t| t.a_voir_en_jeu).count(),
            derniere_date: None,
        };
        vue.derniere_date = vue
            .taches
            .iter()
            .flat_map(|t| t.commits.iter().map(|c| c.date.clone()))
            .max();
        vue.description_html = description(&vue, &uniques, sources);
        vues.push(vue);
    }
    vues
}

#[derive(Serialize)]
struct Accueil<'a> {
    jalons: &'a [JalonVue],
    nb_faites: usize,
    nb_total: usize,
    nb_a_voir: usize,
    nb_commits: usize,
    recentes: Vec<&'a TacheVue>,
    a_verifier_html: String,
    date: String,
}

/// Les pages d'avancement : l'accueil (`index.html`) et une page par jalon.
pub fn pages(sources: &Sources, jalons: &[JalonVue]) -> Vec<Page> {
    let mut pages = Vec::new();
    let mut recentes: Vec<(&String, &TacheVue)> = jalons
        .iter()
        .flat_map(|j| j.taches.iter().filter(|t| t.fait))
        .filter_map(|t| t.commits.iter().map(|c| &c.date).max().map(|d| (d, t)))
        .collect();
    recentes.sort_by(|a, b| b.0.cmp(a.0));
    let a_verifier = sources
        .fichiers
        .iter()
        .find(|(f, _)| f == "PLAN.md")
        .and_then(|(_, c)| taches::extraire_section(c, "## À vérifier en jeu"))
        .map(|s| markdown::rendre(s, "PLAN.md").html)
        .unwrap_or_default();
    let notres = crate::commits::notres(sources.commits);
    let accueil = Accueil {
        jalons,
        nb_faites: jalons.iter().map(|j| j.nb_faites).sum(),
        nb_total: jalons.iter().map(|j| j.nb_total).sum(),
        nb_a_voir: jalons.iter().map(|j| j.nb_a_voir).sum(),
        nb_commits: notres.len(),
        recentes: recentes.iter().take(8).map(|(_, t)| *t).collect(),
        a_verifier_html: a_verifier,
        date: notres.iter().map(|c| c.date.clone()).max().unwrap_or_default(),
    };
    pages.push(Page {
        source: String::new(),
        url: "index.html".to_string(),
        rubrique: Rubrique::Reperes,
        titre: "Tableau de bord".to_string(),
        html: String::new(),
        gabarit: "accueil.html",
        extra: serde_json::to_value(&accueil).expect("sérialisation de l'accueil"),
        ordre: 0,
        icone: None,
        images: Vec::new(),
    });
    for (ordre, jalon) in jalons.iter().enumerate() {
        pages.push(Page {
            ordre,
            source: jalon.plan.clone(),
            url: jalon.url.clone(),
            rubrique: Rubrique::Avancement,
            titre: if jalon.cle == "Prototype" {
                jalon.titre.clone()
            } else {
                format!("{} · {}", jalon.lettres, jalon.titre)
            },
            html: String::new(),
            gabarit: "jalon.html",
            extra: serde_json::to_value(jalon).expect("sérialisation d'un jalon"),
            icone: None,
            images: Vec::new(),
        });
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_paragraphe_saute_titres_et_listes() {
        let texte = "\n### Sous-titre\n\nPremière ligne\nsuite de la ligne.\n\nDeuxième paragraphe.\n";
        assert_eq!(premier_paragraphe(texte).as_deref(), Some("Première ligne suite de la ligne."));
        assert_eq!(premier_paragraphe("- [x] tâche\n"), None);
    }

    #[test]
    fn jalons_et_accueil_sur_les_docs_du_depot() {
        let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let docs = racine.join("docs/icebreaker");
        let fichiers: Vec<(String, String)> = ["README.md", "PLAN.md", "HISTORIQUE.md", "PLAN-GROTTE.md"]
            .iter()
            .map(|f| (f.to_string(), std::fs::read_to_string(docs.join(f)).unwrap()))
            .collect();
        let commits = crate::commits::lire(&racine);
        let readme = &fichiers[0].1;
        let sources = Sources {
            jalons: taches::lire_jalons(readme),
            fichiers: &fichiers,
            commits: &commits,
            url_github: Some("https://github.com/x/y".to_string()),
        };
        let toutes: Vec<Tache> = fichiers.iter().flat_map(|(f, c)| taches::lire(c, f)).collect();
        let vues = jalons(&sources, &toutes);
        let t = vues.iter().find(|j| j.cle == "T").expect("jalon T");
        assert!(t.nb_total >= 5);
        assert_eq!(t.nb_faites, t.taches.iter().filter(|x| x.fait).count());
        assert!(t.description_html.contains("25 %"), "{}", t.description_html);
        let t1 = t.taches.iter().find(|x| x.id.as_deref() == Some("T.1")).unwrap();
        assert_eq!(t1.commits.len(), 1);
        assert_eq!(t1.commits[0].court, "63f57ce830");
        assert!(t1.commits[0].url.as_deref().unwrap().ends_with(&t1.commits[0].court[..]) || true);
        let k = vues.iter().find(|j| j.cle == "K").expect("jalon K");
        assert_eq!(k.taches.iter().filter(|x| x.fichier == "PLAN-GROTTE.md").count(), k.nb_total);
        assert!(vues.iter().any(|j| j.cle == "Prototype"));
        let pages = pages(&sources, &vues);
        assert!(pages.iter().any(|p| p.url == "index.html" && p.gabarit == "accueil.html"));
        assert!(pages.iter().any(|p| p.url == "avancement/T.html"));
    }
}
