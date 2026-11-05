//! Les fiches mécaniques : `docs/icebreaker/wiki/<domaine>.md`, un texte de vulgarisation avec un en-tête TOML qui
//! le relie aux règles (`design/`), aux réglages (`assets/icebreaker/*.ron`), aux tâches des plans et aux fichiers
//! de code. La page a trois étages : la règle, le détail, sous le capot.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::avancement::{JalonVue, TacheVue};
use crate::reglages::{self, Fichier};
use crate::{Page, Rubrique, markdown};

#[derive(Debug, Deserialize)]
pub struct Entete {
    pub titre: String,
    /// Icône : chemin d'une image sous `assets/` (ex. `voxygen/element/ui/generic/icons/skull_2.png`).
    #[serde(default)]
    pub icone: String,
    #[serde(default)]
    pub resume: String,
    /// Fichiers de `design/` (sans le dossier).
    #[serde(default)]
    pub design: Vec<String>,
    /// Fichiers de `assets/icebreaker/` (chemin relatif).
    #[serde(default)]
    pub reglages: Vec<String>,
    /// Lettres de jalon (`T`) ou identifiants de tâche (`C.3`).
    #[serde(default)]
    pub taches: Vec<String>,
    /// Fichiers de code, relatifs à la racine du dépôt.
    #[serde(default)]
    pub systemes: Vec<String>,
    /// Chiffres de l'encart ; vides, on prend les nombres de premier niveau du premier fichier de réglages.
    #[serde(default)]
    pub chiffre: Vec<Chiffre>,
    /// Rang dans la barre latérale (les fiches sans rang passent après, par nom).
    #[serde(default = "rang_par_defaut")]
    pub ordre: usize,
    /// Schéma dessiné depuis les réglages (`schemas::NOMS`), affiché en tête de la règle.
    #[serde(default)]
    pub schema: String,
}

fn rang_par_defaut() -> usize {
    1000
}

#[derive(Debug, Deserialize)]
pub struct Chiffre {
    pub cle: String,
    pub fichier: Option<String>,
    pub nom: Option<String>,
    #[serde(default)]
    pub unite: String,
    #[serde(default)]
    pub pourcent: bool,
}

#[derive(Serialize)]
struct ChiffreVue {
    nom: String,
    valeur: String,
    cle: String,
    fichier: String,
    commentaire: String,
}

#[derive(Serialize)]
struct DesignVue {
    nom: String,
    titre: String,
    url: String,
    html: String,
}

#[derive(Serialize)]
struct SystemeVue {
    chemin: String,
    doc_html: String,
    url: Option<String>,
}

#[derive(Serialize)]
struct JalonLie {
    cle: String,
    lettres: String,
    titre: String,
    url: String,
    nb_faites: usize,
    nb_total: usize,
}

#[derive(Serialize)]
struct FicheVue {
    titre: String,
    /// URL de l'icône relative à la racine du site, si le fichier existe.
    icone_url: Option<String>,
    resume: String,
    schema_svg: Option<String>,
    schema_legende: Option<String>,
    regle_html: String,
    chiffres: Vec<ChiffreVue>,
    designs: Vec<DesignVue>,
    reglages: Vec<reglages::FichierVue>,
    systemes: Vec<SystemeVue>,
    jalons: Vec<JalonLie>,
    taches: Vec<TacheVue>,
    source: String,
}

/// Les lignes `//!` en tête d'un fichier Rust, rendues en Markdown.
pub fn doc_de_module(texte: &str) -> String {
    let mut lignes = Vec::new();
    for ligne in texte.lines() {
        let ligne = ligne.trim_end_matches('\r');
        if let Some(reste) = ligne.strip_prefix("//!") {
            lignes.push(reste.strip_prefix(' ').unwrap_or(reste).to_string());
        } else if lignes.is_empty() && ligne.trim().is_empty() {
            continue;
        } else {
            break;
        }
    }
    lignes.join("\n")
}

/// Cherche une clé dans un fichier de réglages, au premier niveau puis en profondeur.
fn chercher<'a>(entrees: &'a [reglages::Entree], cle: &str) -> Option<&'a reglages::Entree> {
    entrees
        .iter()
        .find(|e| e.cle == cle)
        .or_else(|| entrees.iter().find_map(|e| chercher(&e.enfants, cle)))
}

/// `30.0` → `30`, `0.25` en pourcent → `25 %`, avec l'unité.
pub fn formater(valeur: &str, unite: &str, pourcent: bool) -> String {
    let nombre = valeur.parse::<f64>().ok();
    let texte = match (nombre, pourcent) {
        (Some(n), true) => format!("{} %", arrondir(n * 100.0)),
        (Some(n), false) => arrondir(n),
        (None, _) => valeur.to_string(),
    };
    if unite.is_empty() || pourcent {
        texte
    } else {
        format!("{texte} {unite}")
    }
}

fn arrondir(n: f64) -> String {
    if (n - n.round()).abs() < 1e-9 {
        format!("{}", n.round() as i64)
    } else {
        format!("{n}")
    }
}

fn chiffres(entete: &Entete, reglages: &[Fichier]) -> Vec<ChiffreVue> {
    let premier = entete.reglages.first();
    if entete.chiffre.is_empty() {
        return premier
            .and_then(|p| reglages.iter().find(|f| &f.chemin == p))
            .map(|f| {
                reglages::chiffres_cles(f)
                    .into_iter()
                    .map(|e| ChiffreVue {
                        nom: e.cle.replace('_', " "),
                        valeur: formater(&e.valeur, "", false),
                        cle: e.cle.clone(),
                        fichier: f.chemin.clone(),
                        commentaire: e.commentaire.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
    }
    entete
        .chiffre
        .iter()
        .filter_map(|c| {
            let chemin = c.fichier.as_deref().or(premier.map(String::as_str))?;
            let fichier = reglages.iter().find(|f| f.chemin == chemin)?;
            let entree = chercher(&fichier.entrees, &c.cle)?;
            Some(ChiffreVue {
                nom: c.nom.clone().unwrap_or_else(|| c.cle.replace('_', " ")),
                valeur: formater(&entree.valeur, &c.unite, c.pourcent),
                cle: c.cle.clone(),
                fichier: chemin.to_string(),
                commentaire: entree.commentaire.clone(),
            })
        })
        .collect()
}

fn taches_liees(specs: &[String], jalons: &[JalonVue]) -> (Vec<JalonLie>, Vec<TacheVue>) {
    let mut lies = Vec::new();
    let mut taches = Vec::new();
    for spec in specs {
        if spec.contains('.') {
            if let Some(t) = jalons.iter().flat_map(|j| j.taches.iter()).find(|t| t.id.as_deref() == Some(spec)) {
                taches.push(t.clone());
            }
        } else if let Some(j) = jalons.iter().find(|j| j.lettres.split(", ").any(|l| l == spec)) {
            lies.push(JalonLie {
                cle: j.cle.clone(),
                lettres: j.lettres.clone(),
                titre: j.titre.clone(),
                url: j.url.clone(),
                nb_faites: j.nb_faites,
                nb_total: j.nb_total,
            });
            taches.extend(j.taches.iter().filter(|t| t.jalon == *spec).cloned());
        }
    }
    (lies, taches)
}

/// L'URL d'une icône sous `assets/`, relative à la racine du site ; `None` (avec un avertissement) si le fichier
/// manque.
fn icone_url(racine: &Path, icone: &str, source: &str) -> Option<String> {
    if icone.is_empty() {
        return None;
    }
    if racine.join("assets").join(icone).is_file() {
        Some(format!("assets/{icone}"))
    } else {
        eprintln!("{source} : icône introuvable, assets/{icone}");
        None
    }
}

/// Construit la page d'une fiche à partir de son texte.
pub fn page(
    racine: &Path,
    source: &str,
    texte: &str,
    fichiers_md: &[(String, String)],
    jalons: &[JalonVue],
    reglages: &[Fichier],
    url_github: Option<&str>,
) -> Result<Page> {
    let (entete, corps) = markdown::separer_entete(texte);
    let entete: Entete = toml::from_str(entete.unwrap_or_default()).with_context(|| format!("en-tête de {source}"))?;
    let designs = entete
        .design
        .iter()
        .filter_map(|nom| {
            let chemin = format!("design/{nom}");
            let (_, texte) = fichiers_md.iter().find(|(f, _)| *f == chemin)?;
            let rendu = markdown::rendre(texte, &chemin);
            Some(DesignVue {
                nom: nom.clone(),
                titre: rendu
                    .titre
                    .unwrap_or_else(|| nom.clone())
                    .trim_start_matches("Game design — ")
                    .to_string(),
                url: crate::url_de_source(&chemin),
                html: rendu.html,
            })
        })
        .collect();
    let systemes = entete
        .systemes
        .iter()
        .map(|chemin| {
            let doc = std::fs::read_to_string(racine.join(chemin))
                .map(|t| doc_de_module(&t))
                .unwrap_or_else(|_| "*(fichier introuvable)*".to_string());
            SystemeVue {
                chemin: chemin.clone(),
                doc_html: markdown::rendre(&doc, "").html,
                url: url_github.map(|u| format!("{u}/blob/master/{chemin}")),
            }
        })
        .collect();
    let (jalons_lies, taches) = taches_liees(&entete.taches, jalons);
    let icone_vue = icone_url(racine, &entete.icone, source);
    let schema = if entete.schema.is_empty() {
        None
    } else {
        let s = crate::schemas::dessiner(&entete.schema, reglages, "../");
        if s.is_none() {
            eprintln!(
                "{source} : schéma inconnu ou réglages manquants, {} (connus : {})",
                entete.schema,
                crate::schemas::NOMS.join(", ")
            );
        }
        s
    };
    let regle = markdown::rendre_images(corps, source, Some(&racine.join("docs").join("icebreaker").join("wiki")));
    let nom = source.trim_start_matches("wiki/").trim_end_matches(".md");
    let url = format!("fiches/{nom}.html");
    let mut images = crate::images_de_page(source, &url, &regle.images);
    if let Some(s) = &schema {
        images.extend(s.images.iter().map(|i| (format!("assets/{i}"), format!("assets/{i}"))));
    }
    let vue = FicheVue {
        titre: entete.titre.clone(),
        icone_url: icone_vue.clone(),
        resume: entete.resume.clone(),
        schema_svg: schema.as_ref().map(|s| s.svg.clone()),
        schema_legende: schema.as_ref().map(|s| s.legende.clone()),
        regle_html: regle.html,
        chiffres: chiffres(&entete, reglages),
        designs,
        reglages: entete
            .reglages
            .iter()
            .filter_map(|c| reglages.iter().find(|f| &f.chemin == c))
            .map(reglages::vue)
            .collect(),
        systemes,
        jalons: jalons_lies,
        taches,
        source: source.to_string(),
    };
    Ok(Page {
        source: source.to_string(),
        url,
        rubrique: Rubrique::Mecaniques,
        titre: entete.titre,
        html: String::new(),
        gabarit: "fiche.html",
        extra: serde_json::to_value(&vue).expect("sérialisation d'une fiche"),
        ordre: entete.ordre,
        icone: icone_vue.clone().map(|_| entete.icone.clone()),
        images,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_de_module_prend_les_lignes_de_tete() {
        let texte = "//! Icebreaker : à terre (jalon T).\n//! Deuxième ligne.\n\nuse x;\n//! pas celle-ci\n";
        assert_eq!(doc_de_module(texte), "Icebreaker : à terre (jalon T).\nDeuxième ligne.");
        assert_eq!(doc_de_module("use x;\n"), "");
    }

    #[test]
    fn formatage_des_chiffres() {
        assert_eq!(formater("30.0", "s", false), "30 s");
        assert_eq!(formater("0.25", "", true), "25 %");
        assert_eq!(formater("5.0", "×", false), "5 ×");
        assert_eq!(formater("(2, 3)", "", false), "(2, 3)");
        assert_eq!(formater("0.18", "", false), "0.18");
    }

    #[test]
    fn en_tete_et_liaisons() {
        let texte = "+++\ntitre = \"Mort\"\nicone = \"voxygen/element/ui/generic/icons/skull_2.png\"\nreglages = [\"mort.ron\"]\ntaches = [\"T\", \"L.1\"]\n\
systemes = [\"icebreaker-wiki/src/fiches.rs\"]\n[[chiffre]]\ncle = \"pv_releve\"\nnom = \"PV rendus\"\npourcent = true\n+++\n\n## La règle\n\nTexte.\n";
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mort = reglages::analyser("(\n pv_releve: 0.25, // part\n delai: 30.0,\n)\n", "mort.ron");
        let jalon = JalonVue {
            cle: "T".into(),
            lettres: "T".into(),
            titre: "À terre".into(),
            plan: "PLAN.md".into(),
            plan_url: "PLAN.html".into(),
            etat: String::new(),
            worktree: String::new(),
            description_html: String::new(),
            taches: vec![TacheVue {
                id: Some("T.1".into()),
                titre: "Relever".into(),
                detail_html: String::new(),
                fait: true,
                en_cours: None,
                a_voir_en_jeu: false,
                fichier: "PLAN.md".into(),
                jalon: "T".into(),
                commits: vec![],
                hashes_inconnus: vec![],
            }],
            nb_faites: 1,
            nb_total: 1,
            nb_a_voir: 0,
            derniere_date: None,
            url: "avancement/T.html".into(),
        };
        let page = page(&racine, "wiki/mort.md", texte, &[], &[jalon], &[mort], Some("https://g/x")).unwrap();
        assert_eq!(page.url, "fiches/mort.html");
        assert_eq!(page.titre, "Mort");
        assert_eq!(page.icone.as_deref(), Some("voxygen/element/ui/generic/icons/skull_2.png"));
        assert_eq!(page.extra["icone_url"], "assets/voxygen/element/ui/generic/icons/skull_2.png");
        let extra = &page.extra;
        assert_eq!(extra["chiffres"][0]["valeur"], "25 %");
        assert_eq!(extra["chiffres"][0]["nom"], "PV rendus");
        assert_eq!(extra["taches"][0]["id"], "T.1");
        assert_eq!(extra["jalons"][0]["cle"], "T");
        assert!(extra["systemes"][0]["doc_html"].as_str().unwrap().contains("fiches mécaniques"));
        assert!(extra["regle_html"].as_str().unwrap().contains("<h2>La règle</h2>"));
    }
}
