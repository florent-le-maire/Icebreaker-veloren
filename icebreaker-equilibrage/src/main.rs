//! Icebreaker : écrit `docs/icebreaker/equilibrage.xlsx`, la vue d'ensemble de l'équilibrage (jalon EQ,
//! `docs/icebreaker/PLAN-PLAYTEST.md`, § 8). Les chiffres sont lus dans `assets/` avec les structures du jeu ;
//! les calculs sont des formules Excel branchées sur l'onglet « Hypothèses », qu'on peut modifier pour essayer.
//! Lancer depuis la racine du dépôt : `cargo run -p icebreaker-equilibrage [sortie.xlsx] [--resume [filtre]]`.
//! À côté du classeur, le dossier `equilibrage/` reçoit les mêmes tableaux en Markdown (`texte.rs`).
//! `--resume` affiche aussi la vue d'ensemble (`synthese.md`), et les lignes de Combat dont le jeu d'armes ou
//! le nom contient `filtre`.

mod butin;
mod calcul;
mod capacite;
mod combat;
mod donjon;
mod donnees;
mod economie;
mod expeditions;
mod hyp;
mod progression;
mod reglages;
mod resume;
mod style;
mod synthese;
mod texte;

use anyhow::{Context, Result};
use rust_xlsxwriter::{DocProperties, ExcelDateTime, Workbook, Worksheet};

/// La version du jeu (`assets/icebreaker/version.ron`).
fn version() -> String {
    std::fs::read_to_string("assets/icebreaker/version.ron")
        .ok()
        .and_then(|texte| texte.split('"').nth(1).map(str::to_string))
        .unwrap_or_else(|| "inconnue".into())
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let resume = arguments.iter().position(|a| a == "--resume");
    let filtre = resume.and_then(|p| arguments.get(p + 1)).filter(|a| !a.starts_with("--"));
    // La sortie : le premier argument qui n'est ni une option ni le filtre du résumé.
    let sortie = arguments
        .iter()
        .enumerate()
        .find(|(i, a)| !a.starts_with("--") && resume.is_none_or(|p| *i != p + 1))
        .map_or_else(|| "docs/icebreaker/equilibrage.xlsx".to_string(), |(_, a)| a.clone());
    let d = donnees::Donnees::lire()?;
    // Toutes les hypothèses d'abord : les onglets y pointent par leur ligne.
    let mut hyp = hyp::Hyp::new();
    combat::hypotheses(&mut hyp, &d);
    progression::hypotheses(&mut hyp, &d);
    economie::hypotheses(&mut hyp, &d);
    expeditions::hypotheses(&mut hyp, &d);

    let mut feuilles: Vec<Worksheet> = (0..9).map(|_| Worksheet::new()).collect();
    let [f_synthese, f_hyp, f_combat, f_ennemis, f_progression, f_economie, f_expeditions, f_reglages, f_lisez_moi] =
        &mut feuilles[..]
    else {
        unreachable!()
    };
    let competences = combat::ecrire_combat(f_combat, &d, &hyp)?;
    let ennemis = combat::ecrire_ennemis(f_ennemis, &d, &hyp)?;
    let premier_point = progression::ecrire(f_progression, &d, &hyp, &competences)?;
    let expedition = expeditions::ecrire(f_expeditions, &d, &hyp)?;
    let achats = economie::ecrire(f_economie, &d, &hyp, &expedition)?;
    hyp.ecrire(f_hyp)?;
    let reglages = reglages::ecrire(f_reglages)?;
    reglages::ecrire_lisez_moi(f_lisez_moi, &d.alertes)?;
    synthese::ecrire(f_synthese, &d, &hyp, &synthese::Sources {
        combat: &competences,
        ennemis: &ennemis,
        expedition: &expedition,
        achats: &achats,
        premier_point,
        version: version(),
    })?;

    let mut classeur = Workbook::new();
    // Une date fixe : deux générations sur les mêmes données donnent le même fichier (pas de bruit dans git).
    classeur.set_properties(
        &DocProperties::new()
            .set_title("Icebreaker : équilibrage")
            .set_author("icebreaker-equilibrage")
            .set_creation_datetime(&ExcelDateTime::from_ymd(2026, 1, 1)?),
    );
    for feuille in feuilles {
        classeur.push_worksheet(feuille);
    }
    classeur.save(&sortie).with_context(|| format!("écriture de {sortie} (fermer le classeur dans Excel ?)"))?;
    // Les mêmes tableaux en texte, dans le dossier du même nom que le classeur.
    let vue = resume::synthese(&d, &hyp, &competences, &ennemis, &expedition);
    let dossier = std::path::Path::new(&sortie).with_extension("");
    let fichiers = texte::ecrire(&dossier, &vue)?;
    println!("{} : {fichiers} fichiers Markdown", dossier.display());
    println!(
        "{sortie} : {} compétences et niveaux, {} attaques d'ennemis, {} cas de donjon, {reglages} réglages",
        competences.len(),
        ennemis.len(),
        d.donjon.cas.len()
    );
    for alerte in &d.alertes {
        println!("lecture partielle : {alerte}");
    }
    if resume.is_some() {
        println!("
{vue}");
        if let Some(filtre) = filtre {
            println!("{}", resume::lignes(&d, &competences, filtre));
        }
    }
    Ok(())
}
