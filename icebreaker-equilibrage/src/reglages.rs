//! Icebreaker : les onglets « Réglages » (tous les nombres de `assets/icebreaker/`, tels qu'écrits, avec leur
//! commentaire) et « Lisez-moi ».

use crate::style::{self, Nb, Val, accent, col, col_longue};
use anyhow::{Context, Result};
use rust_xlsxwriter::{Color, Worksheet};
use std::path::Path;

pub const ONGLET: &str = "Réglages";
pub const ONGLET_LISEZ_MOI: &str = "Lisez-moi";

/// Fichiers sans chiffre d'équilibrage : plans, couleurs, noms, musique.
const HORS_SUJET: [&str; 7] = [
    "village_plan.ron",
    "vue_village.ron",
    "donjons/materiaux.ron",
    "donjons/pieces.ron",
    "musique.ron",
    "version.ron",
    "lieux.ron",
];

/// Un réglage lu dans un fichier.
#[derive(Clone, Debug, PartialEq)]
pub struct Reglage {
    pub fichier: String,
    /// Les groupes qui le contiennent : « population › guardRoom ».
    pub chemin: String,
    pub cle: String,
    pub valeur: String,
    pub commentaire: String,
}

/// Le texte sans le contenu de ses chaînes, pour compter les crochets et trouver `//` et `:`.
fn masque(code: &str) -> String {
    let mut dans = false;
    code.chars()
        .map(|c| match (dans, c) {
            (_, '"') => {
                dans = !dans;
                '"'
            },
            (true, _) => '_',
            (false, c) => c,
        })
        .collect()
}

/// Les nombres d'un fichier `.ron` commenté, ligne à ligne : une lecture simple, qui garde les commentaires
/// (ce que le lecteur RON du jeu jette). Une ligne = une clé et sa valeur, ou l'ouverture d'un groupe.
pub fn lire(texte: &str, fichier: &str) -> Vec<Reglage> {
    let mut reglages = Vec::new();
    let mut groupes: Vec<String> = Vec::new();
    let mut attente: Vec<String> = Vec::new();
    for brute in texte.lines() {
        let ligne = brute.trim();
        if ligne.is_empty() || ligne.starts_with("#!") {
            attente.clear();
            continue;
        }
        if let Some(commentaire) = ligne.strip_prefix("//") {
            attente.push(commentaire.trim().to_string());
            continue;
        }
        let vu = masque(ligne);
        let (code, vu, fin) = match vu.find("//") {
            Some(i) => (ligne[..i].trim(), vu[..i].trim().to_string(), Some(ligne[i + 2..].trim())),
            None => (ligne, vu.clone(), None),
        };
        let mut commentaire = std::mem::take(&mut attente).join(" ");
        if let Some(fin) = fin {
            commentaire = format!("{commentaire} {fin}").trim().to_string();
        }
        let ouverts = vu.chars().filter(|c| "([{".contains(*c)).count() as i32;
        let fermes = vu.chars().filter(|c| ")]}".contains(*c)).count() as i32;
        // La clé : ce qui précède le premier `:` hors crochets ; sinon la première chaîne (`("trait_lourd", (`).
        let mut profondeur = 0;
        let deux_points = vu.char_indices().find_map(|(i, c)| {
            match c {
                '(' | '[' | '{' => profondeur += 1,
                ')' | ']' | '}' => profondeur -= 1,
                ':' if profondeur == 0 => return Some(i),
                _ => {},
            }
            None
        });
        let (cle, valeur) = match deux_points {
            Some(i) => (code[..i].trim().trim_matches('"').to_string(), code[i + 1..].trim().trim_end_matches(',').to_string()),
            None => (code.split('"').nth(1).unwrap_or_default().to_string(), String::new()),
        };
        if ouverts > fermes {
            // Un groupe s'ouvre : il porte la clé (ou rien), une fois par crochet resté ouvert.
            for k in 0..(ouverts - fermes) {
                groupes.push(if k == 0 { cle.clone() } else { String::new() });
            }
        } else {
            let chiffre = valeur.chars().any(|c| c.is_ascii_digit());
            let texte_seul = valeur.starts_with('"') && valeur.ends_with('"');
            if !cle.is_empty() && chiffre && !texte_seul && valeur.len() <= 80 {
                reglages.push(Reglage {
                    fichier: fichier.to_string(),
                    chemin: groupes.iter().filter(|g| !g.is_empty()).cloned().collect::<Vec<_>>().join(" › "),
                    cle,
                    valeur,
                    commentaire,
                });
            }
            for _ in 0..(fermes - ouverts) {
                groupes.pop();
            }
        }
    }
    reglages
}

fn fichiers(dossier: &Path, racine: &Path, sortie: &mut Vec<(String, String)>) -> Result<()> {
    let mut entrees: Vec<_> = std::fs::read_dir(dossier)
        .with_context(|| format!("{} (lancer depuis la racine du dépôt)", dossier.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entrees.sort();
    for chemin in entrees {
        if chemin.is_dir() {
            fichiers(&chemin, racine, sortie)?;
        } else if chemin.extension().is_some_and(|e| e == "ron") {
            let relatif = chemin.strip_prefix(racine).unwrap_or(&chemin).to_string_lossy().replace('\\', "/");
            if !HORS_SUJET.contains(&relatif.as_str()) {
                sortie.push((relatif, std::fs::read_to_string(&chemin)?));
            }
        }
    }
    Ok(())
}

pub fn ecrire(feuille: &mut Worksheet) -> Result<usize> {
    feuille.set_name(ONGLET)?;
    let racine = Path::new("assets/icebreaker");
    let mut lus = Vec::new();
    fichiers(racine, racine, &mut lus)?;
    // Les réglages des systèmes (à la racine) d'abord, puis les dossiers.
    lus.sort_by_key(|(fichier, _)| (fichier.contains('/'), fichier.clone()));
    let reglages: Vec<Reglage> = lus.iter().flat_map(|(fichier, texte)| lire(texte, fichier)).collect();
    let colonnes = [
        col("Fichier (assets/icebreaker/)", 44.0, Nb::Texte),
        col("Groupe", 40.0, Nb::Texte),
        col("Réglage", 28.0, Nb::Texte),
        col("Valeur", 24.0, Nb::Dec2),
        col_longue("Commentaire du fichier", 110.0),
    ];
    let depart = style::bandeau(
        feuille,
        "Réglages : tous les nombres des fichiers du jeu",
        "Chaque nombre de assets/icebreaker/ tel qu'il est écrit, avec le commentaire qui l'accompagne. Filtrer par \
         fichier pour retrouver un réglage ; les plans, couleurs et noms sont laissés de côté.",
        accent::REGLAGES,
        colonnes.len() as u16,
    )?;
    let lignes: Vec<Vec<Val>> = reglages
        .iter()
        .map(|r| {
            let valeur = match r.valeur.parse::<f64>() {
                Ok(n) => Val::N(n),
                Err(_) => Val::t(&r.valeur),
            };
            vec![Val::t(&r.fichier), Val::t(&r.chemin), Val::t(&r.cle), valeur, Val::t(&r.commentaire)]
        })
        .collect();
    let (entete, debut, fin) =
        style::tableau(feuille, depart, 0, accent::REGLAGES, &[], &colonnes, &lignes, |i| reglages[i].fichier.clone())?;
    feuille.autofilter(entete, 0, fin, colonnes.len() as u16 - 1)?;
    feuille.set_freeze_panes(debut, 1)?;
    Ok(reglages.len())
}

pub fn ecrire_lisez_moi(feuille: &mut Worksheet, alertes: &[String]) -> Result<()> {
    feuille.set_name(ONGLET_LISEZ_MOI)?;
    feuille.set_screen_gridlines(false);
    feuille.set_column_width(0, 2)?;
    feuille.set_column_width(1, 26)?;
    feuille.set_column_width(2, 120)?;
    style::bandeau(
        feuille,
        "Lisez-moi",
        "À quoi sert ce classeur, comment le lire, comment le refaire, et ce qu'il ne compte pas.",
        accent::LISEZ_MOI,
        3,
    )?;
    let titre = style::intertitre(accent::LISEZ_MOI);
    let gras = style::texte().set_bold();
    let texte = style::texte();
    let mut ligne = 3;
    let mut section = |feuille: &mut Worksheet, nom: &str, lignes: &[(&str, &str)]| -> Result<()> {
        feuille.merge_range(ligne, 1, ligne, 2, nom, &titre)?;
        ligne += 1;
        for (quoi, comment) in lignes {
            feuille.write_string_with_format(ligne, 1, *quoi, &gras)?;
            feuille.write_string_with_format(ligne, 2, *comment, &texte)?;
            feuille.set_row_height(ligne, 15.0 * (1 + comment.chars().count() / 125) as f64)?;
            ligne += 1;
        }
        ligne += 1;
        Ok(())
    };
    section(feuille, "À quoi il sert", &[
        ("Le but", "Voir d'un coup l'équilibre du jeu : combien de coups pour tuer et pour mourir, ce que rapporte et demande une expédition, ce que coûtent les achats et la progression."),
        ("La source", "Les chiffres sont lus dans les fichiers du jeu (assets/), avec les structures du jeu. Ce classeur n'est qu'une vue : rien de ce qu'on y change ne change le jeu."),
        ("Essayer une valeur", "Changer une case jaune de l'onglet Hypothèses : tout le classeur se recalcule, et la colonne « Écart » dit ce qui a été modifié. La valeur retenue se reporte ensuite à la main dans le fichier indiqué dans « Où la régler »."),
        ("Le refaire", "Depuis la racine du dépôt : .\\icebreaker.ps1 equilibrage (ou cargo run -p icebreaker-equilibrage). Le fichier docs/icebreaker/equilibrage.xlsx est réécrit avec les valeurs du jeu : les essais sont perdus, le fermer avant."),
    ])?;
    section(feuille, "Les onglets", &[
        ("Synthèse", "Le tableau de bord : touches pour tuer et pour mourir face aux cibles, expédition type, prix en expéditions."),
        ("Hypothèses", "Les cases « et si » : chaque chiffre du jeu qui entre dans un calcul, et le fichier où il se règle."),
        ("Combat", "Une ligne par compétence et par niveau : dégâts, énergie, recharge, dégâts par seconde, touches et secondes pour tuer chaque ennemi."),
        ("Ennemis", "Les attaques de base des ennemis face à chaque classe, et les PV et l'armure des classes."),
        ("Progression", "La maîtrise qu'il faut par point d'arme, le coût d'une compétence, ce qu'un niveau apporte, les passifs."),
        ("Économie", "L'or et les objets moyens de chaque table de butin, et ce qu'il faut pour chaque achat."),
        ("Expéditions", "Le contenu de vrais donjons générés (par taille, difficulté et densité), ce qu'ils rapportent et demandent ; les minerais de la grotte."),
        ("Difficulté", "Les six difficultés d'une expédition : leurs réglages, les ennemis comptés dans les trois lieux, et ce que donnent leurs effets multipliés."),
        ("Réglages", "Tous les nombres de assets/icebreaker/, avec leur commentaire, à filtrer."),
    ])?;
    section(feuille, "Les couleurs", &[
        ("Jaune, chiffre bleu", "Une case à modifier (onglet Hypothèses)."),
        ("Vert, orange, rouge", "Dans la Synthèse : dans la cible, à côté, loin (plus de deux coups au-delà). Les cibles sont des hypothèses."),
        ("Dégradé vert à rouge", "Dans Combat : du plus rapide au plus lent à tuer. Dans Ennemis, le vert est du côté où le joueur tient le plus longtemps."),
        ("Barres", "La longueur compare les valeurs d'une même colonne."),
        ("Tiret", "Sans objet : une compétence qui ne blesse pas, une énergie nulle."),
    ])?;
    section(feuille, "Ce que le calcul ne compte pas", &[
        ("Tous les coups portent", "Une cible à la fois, immobile, qui ne pare pas et n'esquive pas. Les attaques de zone ne comptent qu'une cible."),
        ("Pas d'altérations", "Ni brûlure, ni saignement (la colonne Notes dit ce que la compétence en porte), ni vol de vie, ni ralentissement, ni déséquilibre ; pas de coups précis ni de combos."),
        ("Le joueur", "Arme et armure de départ de sa classe, sans passif ; son énergie n'est pas limitée (voir « Dégâts par point d'énergie »)."),
        ("Les ennemis", "Ils n'utilisent que les deux attaques de base de leur arme, sans pause : leur intelligence n'est pas comptée. Les PV plus hauts à plusieurs joueurs non plus."),
        ("Un usage", "Un geste entier, lancé avec l'arme de départ comme le fait le jeu (sa vitesse allonge les gestes et espace les touches d'un rayon). Un rayon ou un tir qu'on tient est compté par seconde ou par tir ; la colonne Notes le dit."),
        ("Secondes pour tuer", "Le temps de placer les touches qu'il faut : la préparation, puis la part du geste nécessaire. Si une utilisation d'une compétence ne suffit pas, on finit à l'attaque principale, sans attendre la recharge."),
        ("Cas particuliers", "Charge et élan comptés à fond, plongeon frappé depuis le sol, riposte comptée comme réussie, projectiles en éventail comptés pour un seul, pluie de flèches estimée : la colonne Notes de Combat le dit ligne par ligne."),
        ("Expéditions", "Moyennes de donjons générés, objectif « tuer le chef », faction par défaut. L'or ne compte ni les caisses du décor ni la revente des objets. Les minutes de frappe ne comptent ni la marche, ni la fouille, ni les morts. Grotte et forêt : seuls leurs ennemis sont comptés, par difficulté (onglet Difficulté)."),
        ("Difficulté", "Les facteurs sont ceux du fichier ; l'habileté (le rang des compétences des ennemis) n'entre dans aucun calcul. PV à abattre et dégâts reçus supposent que tous les ennemis du lieu sont affrontés."),
        ("Expérience", "Celle que Veloren donne pour un ennemi, entière : à plusieurs, elle se partage selon les dégâts infligés."),
    ])?;
    if !alertes.is_empty() {
        let lignes: Vec<(&str, &str)> = alertes.iter().map(|a| ("Lecture partielle", a.as_str())).collect();
        section(feuille, "Ce qui n'a pas pu être lu", &lignes)?;
    }
    feuille.set_tab_color(Color::RGB(accent::LISEZ_MOI));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_les_nombres_et_leurs_commentaires() {
        let texte = "// En-tête.\n(\n    // Maîtrise du premier point.\n    premier_point: 100,\n    nom: \"salle 2\",\n    \
                     population: {\n        \"guardRoom\": (creatures: {\"brute\": (2, 3)}), // gardes\n    },\n    \
                     cout_niveaux: [1, 1, 2, 2],\n)\n";
        let lus = lire(texte, "essai.ron");
        let resume: Vec<(&str, &str, &str)> =
            lus.iter().map(|r| (r.chemin.as_str(), r.cle.as_str(), r.valeur.as_str())).collect();
        assert_eq!(resume, vec![
            ("", "premier_point", "100"),
            ("population", "guardRoom", "(creatures: {\"brute\": (2, 3)})"),
            ("", "cout_niveaux", "[1, 1, 2, 2]"),
        ]);
        assert_eq!(lus[0].commentaire, "Maîtrise du premier point.");
        assert_eq!(lus[1].commentaire, "gardes");
    }
}
