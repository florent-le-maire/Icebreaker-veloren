//! Les réglages du jeu : les fichiers `.ron` de `assets/icebreaker/`, lus ligne à ligne pour garder les
//! commentaires (clé, valeur, commentaire, sous-groupes), sans dépendre des types Rust qui les chargent.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Entree {
    /// Nom de la clé ; vide pour un élément de liste.
    pub cle: String,
    /// Texte brut de la valeur ; vide pour un groupe.
    pub valeur: String,
    /// Commentaires au-dessus et en fin de ligne, sans les `//`.
    pub commentaire: String,
    pub enfants: Vec<Entree>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Fichier {
    /// Chemin relatif à `assets/icebreaker/`, avec des `/`.
    pub chemin: String,
    /// Les commentaires de tête du fichier, avant la première parenthèse.
    pub entete: String,
    pub entrees: Vec<Entree>,
}

/// Une ligne de tableau, pour les gabarits.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Ligne {
    pub profondeur: usize,
    pub cle: String,
    pub valeur: String,
    pub commentaire: String,
    pub groupe: bool,
}

/// Lit tous les `.ron` d'un dossier (récursif), triés par chemin.
pub fn lire_dossier(dossier: &Path) -> Result<Vec<Fichier>> {
    let mut fichiers = Vec::new();
    for entree in WalkDir::new(dossier).sort_by_file_name() {
        let entree = entree?;
        let chemin = entree.path();
        if chemin.extension().and_then(|e| e.to_str()) != Some("ron") {
            continue;
        }
        let relatif = chemin
            .strip_prefix(dossier)?
            .to_string_lossy()
            .replace('\\', "/");
        let texte = std::fs::read_to_string(chemin).with_context(|| format!("lecture de {relatif}"))?;
        fichiers.push(analyser(&texte, &relatif));
    }
    Ok(fichiers)
}

/// Enlève le contenu des chaînes `"…"`, pour compter les crochets et trouver les `//` et `:` hors texte.
fn sans_chaines(code: &str) -> String {
    let mut sortie = String::with_capacity(code.len());
    let mut dans_chaine = false;
    let mut echappe = false;
    for c in code.chars() {
        match (dans_chaine, c) {
            (true, '\\') if !echappe => echappe = true,
            (true, '"') if !echappe => {
                dans_chaine = false;
                sortie.push('"');
            },
            (true, _) => {
                echappe = false;
                sortie.push('_');
            },
            (false, '"') => {
                dans_chaine = true;
                sortie.push('"');
            },
            (false, _) => sortie.push(c),
        }
    }
    sortie
}

/// Sépare le code du commentaire de fin de ligne (`//` hors chaîne).
fn separer_commentaire(ligne: &str) -> (&str, Option<&str>) {
    let masque = sans_chaines(ligne);
    match masque.find("//") {
        Some(i) => (ligne[..i].trim_end(), Some(ligne[i + 2..].trim())),
        None => (ligne.trim_end(), None),
    }
}

/// Sépare `cle: valeur` quand le `:` est au niveau 0 des crochets (hors chaîne).
fn separer_cle(code: &str) -> (String, String) {
    let masque = sans_chaines(code);
    let mut profondeur = 0i32;
    for (i, c) in masque.char_indices() {
        match c {
            '(' | '[' | '{' => profondeur += 1,
            ')' | ']' | '}' => profondeur -= 1,
            ':' if profondeur == 0 => {
                let cle = code[..i].trim().trim_matches('"').to_string();
                return (cle, code[i + 1..].trim().to_string());
            },
            _ => {},
        }
    }
    (String::new(), code.trim().to_string())
}

fn est_ouvrant(c: char) -> bool {
    matches!(c, '(' | '[' | '{')
}

fn est_fermant(c: char) -> bool {
    matches!(c, ')' | ']' | '}')
}

/// Un groupe en cours de lecture : l'entrée et le nombre de crochets qu'il reste à fermer.
struct Cadre {
    entree: Entree,
    ouverts: i32,
}

/// Termine un groupe : une liste de valeurs sans clé devient une seule valeur.
fn fermer(cadre: Cadre) -> Entree {
    let mut entree = cadre.entree;
    if !entree.enfants.is_empty() && entree.enfants.iter().all(|e| e.cle.is_empty() && e.enfants.is_empty()) {
        let valeurs: Vec<&str> = entree.enfants.iter().map(|e| e.valeur.as_str()).collect();
        let ouvrant = entree.valeur.chars().next().unwrap_or('[');
        let fermant = match ouvrant {
            '(' => ')',
            '{' => '}',
            _ => ']',
        };
        entree.valeur = format!("{ouvrant}{}{fermant}", valeurs.join(", "));
        entree.enfants.clear();
    } else if entree.valeur.chars().all(est_ouvrant) {
        // Un simple `(`, `[` ou `{` : les enfants disent tout. Sinon on garde les premiers champs écrits sur la
        // ligne d'ouverture (`(id: "guerrier", nom: "guerrier",`).
        entree.valeur = String::new();
    }
    entree
}

/// Analyse un fichier `.ron` commenté.
pub fn analyser(texte: &str, chemin: &str) -> Fichier {
    let mut entete: Vec<String> = Vec::new();
    let mut racine_ouverte = false;
    let mut pile: Vec<Cadre> = vec![Cadre { entree: Entree::default(), ouverts: 0 }];
    let mut en_attente: Vec<String> = Vec::new();

    for brute in texte.lines() {
        let ligne = brute.trim_end_matches('\r').trim();
        if ligne.is_empty() {
            if !en_attente.is_empty() && racine_ouverte {
                en_attente.clear();
            }
            continue;
        }
        if ligne.starts_with("#!") {
            continue;
        }
        if let Some(commentaire) = ligne.strip_prefix("//") {
            let commentaire = commentaire.trim();
            if racine_ouverte {
                en_attente.push(commentaire.to_string());
            } else {
                entete.push(commentaire.to_string());
            }
            continue;
        }
        let (code, fin_de_ligne) = separer_commentaire(ligne);
        let mut code = code.trim().trim_end_matches(',').trim();
        if !racine_ouverte {
            if code.chars().all(est_ouvrant) && !code.is_empty() {
                racine_ouverte = true;
                pile[0].ouverts = code.len() as i32;
            }
            continue;
        }
        // Les crochets fermants en tête de ligne ferment les groupes ouverts.
        while let Some(c) = code.chars().next().filter(|c| est_fermant(*c)) {
            let _ = c;
            code = code[1..].trim_start().trim_start_matches(',').trim_start();
            if let Some(cadre) = pile.last_mut() {
                cadre.ouverts -= 1;
                if cadre.ouverts == 0 && pile.len() > 1 {
                    let fini = fermer(pile.pop().unwrap());
                    pile.last_mut().unwrap().entree.enfants.push(fini);
                }
            }
        }
        if code.is_empty() {
            continue;
        }
        let masque = sans_chaines(code);
        let ouvertures = masque.chars().filter(|c| est_ouvrant(*c)).count() as i32;
        let fermetures = masque.chars().filter(|c| est_fermant(*c)).count() as i32;
        let net = ouvertures - fermetures;
        let mut commentaire = en_attente.join(" ");
        en_attente.clear();
        if let Some(fin) = fin_de_ligne {
            if !commentaire.is_empty() {
                commentaire.push(' ');
            }
            commentaire.push_str(fin);
        }
        let (cle, valeur) = separer_cle(code);
        if net > 0 {
            pile.push(Cadre {
                entree: Entree { cle, valeur, commentaire, enfants: Vec::new() },
                ouverts: net,
            });
        } else {
            // Une valeur complète, qui peut en plus fermer des groupes (`touches: [...]),`).
            let mut valeur = valeur;
            for _ in 0..(-net) {
                valeur = valeur.trim_end_matches(',').trim_end().to_string();
                valeur.pop();
            }
            pile.last_mut()
                .unwrap()
                .entree
                .enfants
                .push(Entree { cle, valeur: valeur.trim_end_matches(',').trim().to_string(), commentaire, enfants: Vec::new() });
            for _ in 0..(-net) {
                let cadre = pile.last_mut().unwrap();
                cadre.ouverts -= 1;
                if cadre.ouverts == 0 && pile.len() > 1 {
                    let fini = fermer(pile.pop().unwrap());
                    pile.last_mut().unwrap().entree.enfants.push(fini);
                }
            }
        }
    }
    while pile.len() > 1 {
        let fini = fermer(pile.pop().unwrap());
        pile.last_mut().unwrap().entree.enfants.push(fini);
    }
    let racine = pile.pop().unwrap().entree;
    Fichier {
        chemin: chemin.to_string(),
        entete: entete.join("\n"),
        entrees: racine.enfants,
    }
}

/// Les entrées mises à plat pour un tableau, avec leur profondeur.
pub fn aplatir(entrees: &[Entree]) -> Vec<Ligne> {
    fn descendre(entrees: &[Entree], profondeur: usize, sortie: &mut Vec<Ligne>) {
        for e in entrees {
            sortie.push(Ligne {
                profondeur,
                cle: e.cle.clone(),
                valeur: e.valeur.clone(),
                commentaire: e.commentaire.clone(),
                groupe: !e.enfants.is_empty(),
            });
            descendre(&e.enfants, profondeur + 1, sortie);
        }
    }
    let mut sortie = Vec::new();
    descendre(entrees, 0, &mut sortie);
    sortie
}

/// Les chiffres clés d'un fichier : ses valeurs de premier niveau qui sont des nombres.
pub fn chiffres_cles(fichier: &Fichier) -> Vec<&Entree> {
    fichier
        .entrees
        .iter()
        .filter(|e| e.enfants.is_empty() && !e.cle.is_empty() && e.valeur.parse::<f64>().is_ok())
        .collect()
}

#[derive(Serialize)]
pub struct LigneVue {
    pub profondeur: usize,
    pub cle: String,
    pub valeur: String,
    pub commentaire_html: String,
    pub groupe: bool,
    /// Icône de Veloren (chemin sous `assets/`) quand la clé ou la valeur parle d'une chose connue.
    pub icone: Option<String>,
}

#[derive(Serialize)]
pub struct ChiffreVue {
    pub cle: String,
    pub valeur: String,
    pub commentaire: String,
}

/// Un fichier de réglages prêt pour les gabarits (page de réglages, bloc d'une fiche).
#[derive(Serialize)]
pub struct FichierVue {
    pub chemin: String,
    pub entete_html: String,
    pub chiffres: Vec<ChiffreVue>,
    pub lignes: Vec<LigneVue>,
}

/// L'URL de la page d'un fichier de réglages (`mort.ron` → `reglages/mort.html`).
pub fn url(chemin: &str) -> String {
    format!("reglages/{}.html", chemin.trim_end_matches(".ron"))
}

pub fn vue(f: &Fichier) -> FichierVue {
    FichierVue {
        chemin: f.chemin.clone(),
        entete_html: crate::markdown::rendre(&f.entete, "").html,
        chiffres: chiffres_cles(f)
            .into_iter()
            .map(|e| ChiffreVue {
                cle: e.cle.clone(),
                valeur: e.valeur.clone(),
                commentaire: e.commentaire.clone(),
            })
            .collect(),
        lignes: aplatir(&f.entrees)
            .into_iter()
            .map(|l| LigneVue {
                icone: crate::icones::pour_texte(&format!("{} {}", l.cle, l.valeur)).map(str::to_string),
                profondeur: l.profondeur,
                cle: l.cle,
                valeur: l.valeur,
                commentaire_html: crate::markdown::rendre_inline(&l.commentaire),
                groupe: l.groupe,
            })
            .collect(),
    }
}

/// Les icônes d'une vue, à copier dans le site.
pub fn images(vue: &FichierVue) -> Vec<(String, String)> {
    let mut images: Vec<(String, String)> = Vec::new();
    for i in vue.lignes.iter().filter_map(|l| l.icone.as_ref()) {
        let paire = (format!("assets/{i}"), format!("assets/{i}"));
        if !images.contains(&paire) {
            images.push(paire);
        }
    }
    images
}

/// Une page par fichier de réglages, dans la rubrique « Réglages ».
pub fn pages(fichiers: &[Fichier]) -> Vec<crate::Page> {
    fichiers
        .iter()
        .map(|f| {
            let vue = vue(f);
            let images = images(&vue);
            crate::Page {
                source: format!("assets/icebreaker/{}", f.chemin),
                url: url(&f.chemin),
                rubrique: crate::Rubrique::Reglages,
                titre: f.chemin.trim_end_matches(".ron").to_string(),
                html: String::new(),
                gabarit: "reglages.html",
                extra: serde_json::to_value(&vue).expect("sérialisation des réglages"),
                ordre: 0,
                icone: None,
                images,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MORT: &str = "// Icebreaker : à terre, relève et mort (jalon T).\n// Deuxième ligne.\n(\n    delai_a_terre: 30.0,\n    pv_releve: 0.25, // part des PV\n    duree_fouille: 3.0,\n)\n";

    #[test]
    fn fichier_plat() {
        let f = analyser(MORT, "mort.ron");
        assert_eq!(f.entete, "Icebreaker : à terre, relève et mort (jalon T).\nDeuxième ligne.");
        assert_eq!(f.entrees.len(), 3);
        assert_eq!(f.entrees[0].cle, "delai_a_terre");
        assert_eq!(f.entrees[0].valeur, "30.0");
        assert_eq!(f.entrees[1].commentaire, "part des PV");
        assert_eq!(chiffres_cles(&f).len(), 3);
    }

    #[test]
    fn groupes_commentaires_et_listes() {
        let texte = "// En-tête\n(\n    // Maisons bâties au début.\n    maisons_au_depart: 8,\n    // Enseignes de chaque bâtiment :\n    // deux lignes.\n    enseignes: {\n        \"entrepot\": [\"Crate\", \"Barrel\"],\n        \"caisse\": [\"Chest\"],\n    },\n    noms: [\n        \"Brunhild\", \"Aldric\",\n        \"Maelis\",\n    ],\n    reseau: (\n        poches: (6, 10),\n        rayon: 2.0,\n    ),\n    classes: [\n        (id: \"guerrier\", nom: \"guerrier\",\n            competences: \"x\",\n            touches: [Sword(DefensiveRiposte), Sword(HeavySweep)]),\n        (id: \"mage\", nom: \"mage\"),\n    ],\n    filons: (chance: 0.06, bord: 0.55),\n)\n";
        let f = analyser(texte, "village.ron");
        let cles: Vec<&str> = f.entrees.iter().map(|e| e.cle.as_str()).collect();
        assert_eq!(cles, vec!["maisons_au_depart", "enseignes", "noms", "reseau", "classes", "filons"]);
        assert_eq!(f.entrees[0].commentaire, "Maisons bâties au début.");
        let enseignes = &f.entrees[1];
        assert_eq!(enseignes.commentaire, "Enseignes de chaque bâtiment : deux lignes.");
        assert_eq!(enseignes.enfants.len(), 2);
        assert_eq!(enseignes.enfants[0].cle, "entrepot");
        assert_eq!(enseignes.enfants[0].valeur, "[\"Crate\", \"Barrel\"]");
        let noms = &f.entrees[2];
        assert!(noms.enfants.is_empty());
        assert_eq!(noms.valeur, "[\"Brunhild\", \"Aldric\", \"Maelis\"]");
        let reseau = &f.entrees[3];
        assert_eq!(reseau.enfants.len(), 2);
        assert_eq!(reseau.enfants[0].valeur, "(6, 10)");
        let classes = &f.entrees[4];
        assert_eq!(classes.enfants.len(), 2);
        let guerrier = &classes.enfants[0];
        assert_eq!(guerrier.enfants.len(), 2);
        assert_eq!(guerrier.valeur, "(id: \"guerrier\", nom: \"guerrier\"");
        assert_eq!(guerrier.enfants[1].cle, "touches");
        assert_eq!(guerrier.enfants[1].valeur, "[Sword(DefensiveRiposte), Sword(HeavySweep)]");
        assert_eq!(classes.enfants[1].valeur, "(id: \"mage\", nom: \"mage\")");
        assert_eq!(f.entrees[5].valeur, "(chance: 0.06, bord: 0.55)");
        let lignes = aplatir(&f.entrees);
        assert!(lignes.iter().any(|l| l.cle == "poches" && l.profondeur == 1));
        assert!(lignes.iter().find(|l| l.cle == "reseau").unwrap().groupe);
    }

    #[test]
    fn racine_liste_et_attribut() {
        let texte = "// Lieux.\n#![enable(implicit_some)]\n[\n    (nom: \"Campagne\", port: 14100),\n    (nom: \"Donjon\", port: 14110),\n]\n";
        let f = analyser(texte, "lieux.ron");
        assert_eq!(f.entrees.len(), 2);
        assert_eq!(f.entrees[1].valeur, "(nom: \"Donjon\", port: 14110)");
        assert!(f.entrees[1].cle.is_empty());
    }

    #[test]
    fn les_vrais_fichiers() {
        let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/icebreaker");
        let fichiers = lire_dossier(&dossier).unwrap();
        assert!(fichiers.len() >= 10);
        let mort = fichiers.iter().find(|f| f.chemin == "mort.ron").unwrap();
        assert_eq!(chiffres_cles(mort).iter().map(|e| e.cle.as_str()).collect::<Vec<_>>(), vec![
            "delai_a_terre",
            "pv_a_terre",
            "pv_releve",
            "duree_fouille"
        ]);
        let grotte = fichiers.iter().find(|f| f.chemin == "grottes/grotte.ron").unwrap();
        let ambiances = grotte.entrees.iter().find(|e| e.cle == "ambiances").unwrap();
        assert_eq!(ambiances.enfants.len(), 3);
        assert_eq!(ambiances.enfants[0].cle, "champignons");
        assert!(ambiances.enfants[0].enfants.iter().any(|e| e.cle == "solitaires"));
        let registre = fichiers.iter().find(|f| f.chemin == "registre.ron").unwrap();
        let noms = registre.entrees.iter().find(|e| e.cle == "noms").unwrap();
        assert!(noms.enfants.is_empty() && noms.valeur.contains("\"Ulric\""));
    }
}
