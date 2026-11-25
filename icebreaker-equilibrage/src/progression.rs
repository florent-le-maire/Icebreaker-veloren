//! Icebreaker : l'onglet « Progression » — gagner des points d'arme, monter une compétence, ce qu'un niveau
//! apporte, les passifs.

use crate::{
    calcul,
    combat::{C_DEG, C_DPS_SOUTENU, Ecrite, ONGLET_COMBAT},
    capacite::net,
    donnees::{Donnees, Sorte, effet_en_clair},
    hyp::Hyp,
    style::{self, Nb, Val, a1, accent, col, col_longue},
};
use anyhow::Result;
use common::icebreaker_competences::registre;
use rust_xlsxwriter::Worksheet;

pub const ONGLET: &str = "Progression";
/// Points d'arme montrés dans le premier tableau.
const POINTS: u32 = 12;
// Largeurs communes aux tableaux empilés.
const L: [f64; 4] = [18.0, 18.0, 34.0, 13.0];

pub fn hypotheses(hyp: &mut Hyp, d: &Donnees) {
    let g = "Progression (salle d'armes)";
    let source = "assets/icebreaker/salle_armes.ron";
    let s = &d.salle;
    hyp.ajoute(g, "salle.premier_point", "Maîtrise pour le premier point d'arme", f64::from(s.premier_point), Nb::Entier, &format!("{source} : premier_point"));
    hyp.ajoute(g, "salle.pas_point", "Maîtrise en plus pour chaque point suivant", f64::from(s.pas_point), Nb::Entier, &format!("{source} : pas_point"));
    hyp.ajoute(g, "salle.maitrise_par_experience", "Maîtrise gagnée par point d'expérience", net(s.maitrise_par_experience), Nb::Fois, &format!("{source} : maitrise_par_experience"));
    hyp.ajoute(g, "salle.cout_1", "Points d'arme pour débloquer une compétence", f64::from(s.cout_debloquer), Nb::Entier, &format!("{source} : cout_debloquer"));
    for (i, cout) in s.cout_niveaux.iter().enumerate() {
        hyp.ajoute(g, &format!("salle.cout_{}", i + 2), &format!("Points d'arme pour passer au niveau {}", i + 2), f64::from(*cout), Nb::Entier, &format!("{source} : cout_niveaux"));
    }
}

/// Renvoie la ligne du premier point d'arme (pour la Synthèse).
pub fn ecrire(feuille: &mut Worksheet, d: &Donnees, hyp: &Hyp, combat: &[Ecrite]) -> Result<u32> {
    feuille.set_name(ONGLET)?;
    let mut ligne = style::bandeau(
        feuille,
        "Progression : points d'arme, niveaux et passifs",
        "Jouer un jeu d'armes en expédition donne de la maîtrise (l'expérience des ennemis tués, partagée entre ceux \
         qui les ont blessés) ; la maîtrise donne des points d'arme, dépensés à la salle d'armes.",
        accent::PROGRESSION,
        14,
    )?;

    // 1. Gagner des points d'arme.
    style::section(feuille, ligne, 7, "Gagner des points d'arme", accent::PROGRESSION)?;
    ligne += 1;
    let premier_point = ligne + 1;
    let mut colonnes = vec![
        col("Point d'arme n°", L[0], Nb::Entier),
        col("Maîtrise pour ce point", L[1], Nb::Entier),
        col("Maîtrise cumulée", L[2], Nb::Entier),
    ];
    for ennemi in &d.ennemis {
        colonnes.push(col(&format!("Ennemis « {} » à tuer seul", ennemi.role), L[3], Nb::Dec1));
    }
    let (premier, pas, par_xp) =
        (hyp.r("salle.premier_point"), hyp.r("salle.pas_point"), hyp.r("salle.maitrise_par_experience"));
    let (premier_v, pas_v, par_xp_v) =
        (hyp.v("salle.premier_point"), hyp.v("salle.pas_point"), hyp.v("salle.maitrise_par_experience"));
    let lignes: Vec<Vec<Val>> = (1..=POINTS)
        .map(|n| {
            let l = ligne + n;
            let n_f = f64::from(n);
            let cumul = calcul::maitrise_pour(n_f, premier_v, pas_v);
            let (a, c) = (a1(l, 0), a1(l, 2));
            let mut valeurs = vec![
                Val::N(n_f),
                Val::F(format!("={premier}+({a}-1)*{pas}"), premier_v + (n_f - 1.0) * pas_v),
                Val::F(format!("={a}*{premier}+{pas}*{a}*({a}-1)/2"), cumul),
            ];
            for ennemi in &d.ennemis {
                let xp = format!("ennemi.{}.experience", ennemi.role);
                valeurs.push(Val::F(format!("={c}/({}*{par_xp})", hyp.r(&xp)), cumul / (hyp.v(&xp) * par_xp_v)));
            }
            valeurs
        })
        .collect();
    let (_, _, fin) = style::tableau(feuille, ligne, 0, accent::PROGRESSION, &[], &colonnes, &lignes, |i| i.to_string())?;

    // 2. Monter une compétence.
    ligne = fin + 3;
    style::section(feuille, ligne, 7, "Monter une compétence", accent::PROGRESSION)?;
    ligne += 1;
    let colonnes = [
        col("Étape", L[0], Nb::Texte),
        col("Points d'arme", L[1], Nb::Entier),
        col("Points d'arme cumulés", L[2], Nb::Entier),
        col("Maîtrise cumulée, si on ne monte qu'elle", L[3], Nb::Entier),
    ];
    let niveaux = d.salle.cout_niveaux.len() + 1;
    let mut lignes = Vec::new();
    let mut cumul_v = 0.0;
    for n in 1..=niveaux {
        let l = ligne + n as u32;
        let cle = format!("salle.cout_{n}");
        cumul_v += hyp.v(&cle);
        let cumul = if n == 1 { format!("={}", a1(l, 1)) } else { format!("={}+{}", a1(l - 1, 2), a1(l, 1)) };
        let c = a1(l, 2);
        lignes.push(vec![
            Val::t(if n == 1 { "Débloquer (niveau 1)".to_string() } else { format!("Niveau {n}") }),
            Val::F(format!("={}", hyp.r(&cle)), hyp.v(&cle)),
            Val::F(cumul, cumul_v),
            Val::F(format!("={c}*{premier}+{pas}*{c}*({c}-1)/2"), calcul::maitrise_pour(cumul_v, premier_v, pas_v)),
        ]);
    }
    // Un jeu d'armes entier : toutes ses compétences au dernier niveau.
    let par_jeu = d.jeux.first().and_then(|j| registre().jeu(&j.id)).map_or(4, |j| j.competences.len()) as f64;
    let l = ligne + niveaux as u32 + 1;
    let dernier = a1(l - 1, 2);
    let c = a1(l, 2);
    lignes.push(vec![
        Val::t(format!("Un jeu d'armes entier ({par_jeu} compétences)")),
        Val::Vide,
        Val::F(format!("={par_jeu}*{dernier}"), par_jeu * cumul_v),
        Val::F(format!("={c}*{premier}+{pas}*{c}*({c}-1)/2"), calcul::maitrise_pour(par_jeu * cumul_v, premier_v, pas_v)),
    ]);
    let (_, _, fin) = style::tableau(feuille, ligne, 0, accent::PROGRESSION, &[], &colonnes, &lignes, |i| i.to_string())?;

    // 3. Ce qu'un niveau apporte.
    ligne = fin + 3;
    style::section(feuille, ligne, 13, "Ce qu'un niveau apporte (du niveau 1 au dernier)", accent::PROGRESSION)?;
    ligne += 1;
    let colonnes = [
        col("Classe", L[0], Nb::Texte),
        col("Jeu d'armes", L[1], Nb::Texte),
        col("Compétence", L[2], Nb::Texte),
        col("Dernier niveau", L[3], Nb::Entier),
        col("Puissance au dernier niveau", 13.0, Nb::Fois),
        col("Recharge au niveau 1", 13.0, Nb::Secondes),
        col("Recharge au dernier niveau", 13.0, Nb::Secondes),
        col("Dégâts par usage au niveau 1", 13.0, Nb::Dec1),
        col("Dégâts par usage au dernier niveau", 13.0, Nb::Dec1),
        col("Gain de dégâts", 13.0, Nb::Pct),
        col("Dégâts par seconde (soutenu) au niveau 1", 13.0, Nb::Dec1),
        col("Dégâts par seconde (soutenu) au dernier niveau", 13.0, Nb::Dec1),
        col("Gain en dégâts par seconde", 13.0, Nb::Pct),
    ];
    let mut lignes = Vec::new();
    let mut paquets = Vec::new();
    let premieres: Vec<&Ecrite> = combat.iter().filter(|e| e.sorte == Sorte::Competence && e.niveau == 1).collect();
    for (i, un) in premieres.iter().enumerate() {
        let Some(dernier) = combat
            .iter()
            .filter(|e| e.sorte == Sorte::Competence && e.jeu == un.jeu && e.nom == un.nom)
            .max_by_key(|e| e.niveau)
        else {
            continue;
        };
        let l = ligne + 1 + i as u32;
        let source = d.lignes.iter().find(|x| x.jeu == un.jeu && x.nom == un.nom && x.niveau == dernier.niveau);
        let source_un = d.lignes.iter().find(|x| x.jeu == un.jeu && x.nom == un.nom && x.niveau == 1);
        let (Some(source), Some(source_un)) = (source, source_un) else { continue };
        let r = |e: &Ecrite, colonne: u16| format!("='{ONGLET_COMBAT}'!{}", a1(e.ligne, colonne));
        let mut valeurs = vec![
            Val::t(&un.classe),
            Val::t(&source.jeu_nom),
            Val::t(&un.nom),
            Val::N(f64::from(dernier.niveau)),
            Val::N(source.puissance),
            Val::N(source_un.recharge),
            Val::N(source.recharge),
        ];
        if un.degats.is_finite() && dernier.degats.is_finite() {
            valeurs.extend([
                Val::F(r(un, C_DEG), un.degats),
                Val::F(r(dernier, C_DEG), dernier.degats),
                Val::F(format!("={}/{}-1", a1(l, 8), a1(l, 7)), dernier.degats / un.degats - 1.0),
                Val::F(r(un, C_DPS_SOUTENU), un.dps_soutenu),
                Val::F(r(dernier, C_DPS_SOUTENU), dernier.dps_soutenu),
                Val::F(format!("={}/{}-1", a1(l, 11), a1(l, 10)), dernier.dps_soutenu / un.dps_soutenu - 1.0),
            ]);
        } else {
            valeurs.extend(std::iter::repeat_n(Val::Vide, 6));
        }
        lignes.push(valeurs);
        paquets.push(un.jeu.clone());
    }
    let (_, debut, fin) =
        style::tableau(feuille, ligne, 0, accent::PROGRESSION, &[], &colonnes, &lignes, |i| paquets[i].clone())?;
    for colonne in [9, 12] {
        let regle = rust_xlsxwriter::ConditionalFormatDataBar::new()
            .set_fill_color(rust_xlsxwriter::Color::RGB(0xA9C4F5))
            .set_solid_fill(true);
        feuille.add_conditional_format(debut, colonne, fin, colonne, &regle)?;
    }

    // 4. Passifs et inné.
    ligne = fin + 3;
    style::section(feuille, ligne, 7, "Passifs et inné des classes", accent::PROGRESSION)?;
    ligne += 1;
    let colonnes = [
        col("Classe", L[0], Nb::Texte),
        col("Passif", L[1], Nb::Texte),
        col_longue("Effets", L[2]),
        col("Prix (or)", L[3], Nb::Entier),
    ];
    let mut lignes = Vec::new();
    let mut paquets = Vec::new();
    for classe in &d.classes {
        let Some(passifs) = registre().passifs_de(&classe.id) else { continue };
        let en_clair = |effets: &[common::comp::buff::BuffEffect]| {
            effets.iter().map(effet_en_clair).collect::<Vec<_>>().join(" ; ")
        };
        lignes.push(vec![Val::t(&classe.nom), Val::t("Inné (toujours actif)"), Val::T(en_clair(&passifs.inne)), Val::Vide]);
        paquets.push(classe.id.clone());
        for (_, passif) in &passifs.passifs {
            lignes.push(vec![
                Val::t(&classe.nom),
                Val::t(&passif.nom),
                Val::T(en_clair(&passif.effets)),
                Val::F(format!("={}", hyp.r("prix.passif")), hyp.v("prix.passif")),
            ]);
            paquets.push(classe.id.clone());
        }
    }
    style::tableau(feuille, ligne, 0, accent::PROGRESSION, &[], &colonnes, &lignes, |i| paquets[i].clone())?;
    feuille.set_freeze_panes(3, 0)?;
    Ok(premier_point)
}
