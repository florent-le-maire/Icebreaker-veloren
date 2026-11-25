//! Icebreaker : écrit `docs/icebreaker/equilibrage.xlsx`, la vue d'ensemble de l'équilibrage (jalon EQ,
//! `docs/icebreaker/PLAN-PLAYTEST.md`, § 8). Les chiffres sont lus dans `assets/` avec les structures du jeu ;
//! les calculs sont des formules Excel branchées sur l'onglet « Hypothèses », qu'on peut modifier pour essayer.
//! Lancer depuis la racine du dépôt : `cargo run -p icebreaker-equilibrage [sortie.xlsx]`.

mod butin;
mod calcul;
mod combat;
mod donnees;
mod economie;
mod hyp;
mod progression;
mod style;

use anyhow::{Context, Result};
use rust_xlsxwriter::{DocProperties, ExcelDateTime, Workbook, Worksheet};

fn main() -> Result<()> {
    let sortie = std::env::args().nth(1).unwrap_or_else(|| "docs/icebreaker/equilibrage.xlsx".into());
    let d = donnees::Donnees::lire()?;
    let mut hyp = hyp::Hyp::new();
    combat::hypotheses(&mut hyp, &d);
    progression::hypotheses(&mut hyp, &d);
    economie::hypotheses(&mut hyp, &d);

    let mut f_hyp = Worksheet::new();
    let mut f_combat = Worksheet::new();
    let mut f_ennemis = Worksheet::new();
    let mut f_progression = Worksheet::new();
    let mut f_economie = Worksheet::new();
    let competences = combat::ecrire_combat(&mut f_combat, &d, &hyp)?;
    let ennemis = combat::ecrire_ennemis(&mut f_ennemis, &d, &hyp)?;
    progression::ecrire(&mut f_progression, &d, &hyp, &competences)?;
    let _achats = economie::ecrire(&mut f_economie, &d, &hyp)?;
    hyp.ecrire(&mut f_hyp)?;

    let mut classeur = Workbook::new();
    // Une date fixe : deux générations sur les mêmes données donnent le même fichier (pas de bruit dans git).
    classeur.set_properties(
        &DocProperties::new()
            .set_title("Icebreaker : équilibrage")
            .set_author("icebreaker-equilibrage")
            .set_creation_datetime(&ExcelDateTime::from_ymd(2026, 1, 1)?),
    );
    for feuille in [f_hyp, f_combat, f_ennemis, f_progression, f_economie] {
        classeur.push_worksheet(feuille);
    }
    classeur.save(&sortie).with_context(|| format!("écriture de {sortie} (fermer le classeur dans Excel ?)"))?;
    println!("{} compétences et niveaux, {} attaques d'ennemis dans {sortie}", competences.len(), ennemis.len());
    for alerte in &d.alertes {
        println!("lecture partielle : {alerte}");
    }
    Ok(())
}
