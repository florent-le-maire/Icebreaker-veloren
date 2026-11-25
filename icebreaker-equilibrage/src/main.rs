//! Icebreaker : écrit `docs/icebreaker/equilibrage.xlsx`, la vue d'ensemble de l'équilibrage (jalon EQ,
//! `docs/icebreaker/PLAN-PLAYTEST.md`, § 8). Les chiffres sont lus dans `assets/` avec les structures du jeu ;
//! les calculs sont des formules Excel branchées sur l'onglet « Hypothèses », qu'on peut modifier pour essayer.
//! Lancer depuis la racine du dépôt : `cargo run -p icebreaker-equilibrage [sortie.xlsx]`.

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
mod style;
mod synthese;

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
    let sortie = std::env::args().nth(1).unwrap_or_else(|| "docs/icebreaker/equilibrage.xlsx".into());
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
    println!(
        "{sortie} : {} compétences et niveaux, {} attaques d'ennemis, {} cas de donjon, {reglages} réglages",
        competences.len(),
        ennemis.len(),
        d.donjon.cas.len()
    );
    for alerte in &d.alertes {
        println!("lecture partielle : {alerte}");
    }
    Ok(())
}
