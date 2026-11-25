//! Icebreaker : la vue d'ensemble de l'équilibrage en texte — les deux matrices de la Synthèse, l'expédition
//! type, les achats. Elle fait le corps de `docs/icebreaker/equilibrage/synthese.md`, et `--resume [filtre]`
//! l'affiche, avec les lignes de Combat dont le nom contient le filtre.

use crate::{
    combat::{Ecrite, EnnemiEcrit},
    donnees::{Donnees, Sorte},
    economie, expeditions,
    hyp::Hyp,
    texte::tableau,
};
use std::fmt::Write as _;

fn nombre(x: f64) -> String { if x.is_finite() { format!("{x:.0}") } else { "—".into() } }

/// La vue d'ensemble, en Markdown.
pub fn synthese(
    d: &Donnees,
    hyp: &Hyp,
    competences: &[Ecrite],
    ennemis: &[EnnemiEcrit],
    expedition: &expeditions::Ecrit,
    difficultes: &[crate::difficulte::Ligne],
) -> String {
    let mut s = String::new();
    let principales: Vec<&Ecrite> =
        competences.iter().filter(|e| e.sorte == Sorte::Base && e.nom == "Attaque principale").collect();
    let frappes: Vec<&EnnemiEcrit> = ennemis.iter().filter(|e| e.principale).collect();
    let ordinaires = d.ennemis.iter().filter(|e| e.role != "chef").count();
    let dans = |valeurs: &[f64], mini: f64, maxi: f64| valeurs.iter().filter(|v| **v >= mini && **v <= maxi).count();

    // Tuer.
    let (mini, maxi) = (hyp.v("cible.tuer_min"), hyp.v("cible.tuer_max"));
    let valeurs: Vec<f64> = principales.iter().flat_map(|e| e.contre.iter().take(ordinaires).map(|c| c.0)).collect();
    let _ = writeln!(
        s,
        "## Tuer : touches à l'attaque principale\n\nCible contre un ennemi ordinaire (pas le chef) : {mini} à {maxi} \
         touches. Dans la cible : {} cases sur {} ; moyenne {:.1}.\n",
        dans(&valeurs, mini, maxi),
        valeurs.len(),
        valeurs.iter().sum::<f64>() / valeurs.len().max(1) as f64
    );
    let mut colonnes = vec!["Jeu d'armes".to_string()];
    colonnes.extend(d.ennemis.iter().map(|e| e.role.clone()));
    colonnes.push("Dégâts par seconde".into());
    let lignes: Vec<Vec<String>> = principales
        .iter()
        .map(|e| {
            let nom = d.jeux.iter().find(|j| j.id == e.jeu).map_or(e.jeu.clone(), |j| j.nom.clone());
            let mut ligne = vec![nom];
            ligne.extend(e.contre.iter().map(|c| nombre(c.0)));
            ligne.push(format!("{:.1}", e.dps));
            ligne
        })
        .collect();
    s.push_str(&tableau(&colonnes, &lignes));

    // Mourir.
    let (mini, maxi) = (hyp.v("cible.mourir_min"), hyp.v("cible.mourir_max"));
    let valeurs: Vec<f64> = frappes.iter().take(ordinaires).flat_map(|e| e.contre.iter().map(|c| c.0)).collect();
    let _ = writeln!(
        s,
        "\n## Mourir : touches d'un ennemi, attaque principale\n\nCible pour un ennemi ordinaire : {mini} à {maxi} \
         touches. Dans la cible : {} cases sur {} ; moyenne {:.1}.\n",
        dans(&valeurs, mini, maxi),
        valeurs.len(),
        valeurs.iter().sum::<f64>() / valeurs.len().max(1) as f64
    );
    let mut colonnes = vec!["Ennemi".to_string()];
    colonnes.extend(d.classes.iter().map(|c| c.nom.clone()));
    let lignes: Vec<Vec<String>> = frappes
        .iter()
        .map(|e| std::iter::once(e.role.clone()).chain(e.contre.iter().map(|c| nombre(c.0))).collect())
        .collect();
    s.push_str(&tableau(&colonnes, &lignes));

    // L'expédition type et les achats.
    let (creatures, pv, or, minutes, points) = expedition.valeurs;
    let premier = d.ennemis.first().map_or(String::new(), |e| e.role.clone());
    let _ = writeln!(
        s,
        "\n## L'expédition type\n\nDonjon moyen, difficulté et densité par défaut : {creatures:.0} créatures, {pv:.0} PV à \
         abattre, {or:.0} or, {minutes:.1} minutes de frappe du groupe, {points:.0} points d'arme gagnés seul. \
         Premier point d'arme : {:.1} ennemis « {premier} » à tuer seul.\n\n## Les achats\n",
        hyp.v("salle.premier_point")
            / (hyp.v(&format!("ennemi.{premier}.experience")) * hyp.v("salle.maitrise_par_experience")),
    );
    let lignes: Vec<Vec<String>> = economie::ACHATS
        .iter()
        .map(|(cle, nom)| vec![nom.to_string(), format!("{:.0}", hyp.v(cle)), format!("{:.2}", hyp.v(cle) / or)])
        .collect();
    s.push_str(&tableau(&["Achat".into(), "Prix (or)".into(), "Expéditions type".into()], &lignes));
    s.push_str(&crate::difficulte::resume(difficultes));
    s
}

/// Les lignes de Combat dont le jeu d'armes ou le nom contient `filtre`.
pub fn lignes(d: &Donnees, competences: &[Ecrite], filtre: &str) -> String {
    let filtre = filtre.to_lowercase();
    let roles: Vec<&str> = d.ennemis.iter().map(|e| e.role.as_str()).collect();
    let mut s = format!("Combat, lignes contenant « {filtre} » (touches / secondes pour tuer : {}) :\n", roles.join(", "));
    for e in competences.iter().filter(|e| format!("{} {}", e.jeu, e.nom).to_lowercase().contains(&filtre)) {
        let niveau = if e.niveau == 0 { "base".to_string() } else { format!("niv {}", e.niveau) };
        let contre: String = e.contre.iter().map(|c| format!("{:>5} /{:>6.1} s", nombre(c.0), c.1)).collect();
        let _ = writeln!(
            s,
            "  {:<14}{:<22}{niveau:<7} dégâts {:>6.1}  par seconde {:>6.1} |{contre}",
            e.jeu, e.nom, e.degats, e.dps
        );
    }
    s
}
