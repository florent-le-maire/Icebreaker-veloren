//! Icebreaker : l'onglet « Synthèse », le tableau de bord. Tout y est formule vers les autres onglets : changer
//! une hypothèse le met à jour.

use crate::{
    combat::{C_DPS, C_ENNEMIS, E_CLASSES, Ecrite, EnnemiEcrit, ONGLET_COMBAT, ONGLET_ENNEMIS},
    donnees::{Donnees, Sorte},
    economie, expeditions,
    hyp::Hyp,
    progression,
    style::{self, Nb, Val, a1, accent},
};
use anyhow::Result;
use rust_xlsxwriter::{
    Chart, ChartDataLabel, ChartFont, ChartFormat, ChartSolidFill, ChartType, Color, ConditionalFormatCell,
    ConditionalFormatCellRule, Format, FormatAlign, FormatBorder, Formula, Worksheet,
};

pub const ONGLET: &str = "Synthèse";
/// Première colonne de la grille (la colonne A sert de marge), et largeur de ses colonnes.
const G: u16 = 1;
const LARGEUR: f64 = 13.0;
const COLONNES: u16 = 12;

pub struct Sources<'a> {
    pub combat: &'a [Ecrite],
    pub ennemis: &'a [EnnemiEcrit],
    pub expedition: &'a expeditions::Ecrit,
    pub achats: &'a economie::Ecrit,
    /// Ligne du premier point d'arme dans Progression.
    pub premier_point: u32,
    pub version: String,
}

fn police() -> Format { Format::new().set_font_name(style::POLICE).set_font_color(Color::RGB(style::ENCRE)) }

/// Une carte : un libellé, un grand nombre (formule), une précision. Deux colonnes de large, trois lignes.
fn carte(
    feuille: &mut Worksheet,
    ligne: u32,
    colonne: u16,
    libelle: &str,
    valeur: &Val,
    nb: Nb,
    precision: &str,
) -> Result<()> {
    let cadre = Color::RGB(style::TRAIT);
    let haut = police()
        .set_font_size(9)
        .set_font_color(Color::RGB(style::GRIS))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_background_color(Color::RGB(style::BANDE))
        .set_border_top(FormatBorder::Thin)
        .set_border_left(FormatBorder::Thin)
        .set_border_right(FormatBorder::Thin)
        .set_border_color(cadre);
    let milieu = style::cellule(nb, false)
        .set_bold()
        .set_font_size(22)
        .set_font_color(Color::RGB(accent::SYNTHESE))
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(style::BANDE))
        .set_border_bottom(FormatBorder::None)
        .set_border_left(FormatBorder::Thin)
        .set_border_right(FormatBorder::Thin)
        .set_border_color(cadre);
    let bas = police()
        .set_font_size(9)
        .set_italic()
        .set_font_color(Color::RGB(style::GRIS))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_background_color(Color::RGB(style::BANDE))
        .set_border_bottom(FormatBorder::Thin)
        .set_border_left(FormatBorder::Thin)
        .set_border_right(FormatBorder::Thin)
        .set_border_color(cadre);
    feuille.merge_range(ligne, colonne, ligne, colonne + 1, libelle, &haut)?;
    feuille.merge_range(ligne + 1, colonne, ligne + 1, colonne + 1, "", &milieu)?;
    style::ecrire(feuille, ligne + 1, colonne, valeur, &milieu)?;
    feuille.merge_range(ligne + 2, colonne, ligne + 2, colonne + 1, precision, &bas)?;
    Ok(())
}

/// Colore une plage selon une cible : vert dans la fourchette, rouge deux coups au-delà, orange sinon.
fn cible(feuille: &mut Worksheet, plage: (u32, u16, u32, u16), mini: &str, maxi: &str) -> Result<()> {
    let (l1, c1, l2, c2) = plage;
    let vert = ConditionalFormatCell::new()
        .set_rule(ConditionalFormatCellRule::Between(Formula::new(format!("={mini}")), Formula::new(format!("={maxi}"))))
        .set_format(style::pastille(style::VERT, style::VERT_ENCRE));
    let rouge = ConditionalFormatCell::new()
        .set_rule(ConditionalFormatCellRule::GreaterThan(Formula::new(format!("={maxi}+2"))))
        .set_format(style::pastille(style::ROUGE, style::ROUGE_ENCRE));
    let orange = ConditionalFormatCell::new()
        .set_rule(ConditionalFormatCellRule::NotBetween(Formula::new(format!("={mini}")), Formula::new(format!("={maxi}"))))
        .set_format(style::pastille(style::ORANGE, style::ORANGE_ENCRE));
    feuille.add_conditional_format(l1, c1, l2, c2, &vert)?;
    feuille.add_conditional_format(l1, c1, l2, c2, &rouge)?;
    feuille.add_conditional_format(l1, c1, l2, c2, &orange)?;
    Ok(())
}

fn graphique(titre: &str, categories: (u32, u16, u32, u16), valeurs: (u32, u16, u32, u16), couleur: u32) -> Chart {
    let mut graphique = Chart::new(ChartType::Bar);
    graphique
        .add_series()
        .set_categories((ONGLET, categories.0, categories.1, categories.2, categories.3))
        .set_values((ONGLET, valeurs.0, valeurs.1, valeurs.2, valeurs.3))
        .set_format(ChartFormat::new().set_solid_fill(ChartSolidFill::new().set_color(Color::RGB(couleur))))
        .set_data_label(ChartDataLabel::new().show_value().set_num_format("0.0"))
        .set_gap(60);
    graphique.title().set_name(titre).set_font(ChartFont::new().set_name(style::POLICE).set_size(11).set_bold());
    graphique.legend().set_hidden();
    graphique.x_axis().set_reverse().set_font(ChartFont::new().set_name(style::POLICE).set_size(9));
    graphique.y_axis().set_font(ChartFont::new().set_name(style::POLICE).set_size(9));
    graphique.set_width(760).set_height(300);
    graphique
}

pub fn ecrire(feuille: &mut Worksheet, d: &Donnees, hyp: &Hyp, s: &Sources) -> Result<()> {
    feuille.set_name(ONGLET)?;
    feuille.set_screen_gridlines(false);
    feuille.set_column_width(0, 2)?;
    for c in 0..=COLONNES {
        feuille.set_column_width(G + c, LARGEUR)?;
    }
    // Le bandeau commence en colonne A, sur toute la largeur.
    style::bandeau(
        feuille,
        "Icebreaker : équilibrage, vue d'ensemble",
        &format!(
            "Version {} du jeu. Tout se recalcule depuis l'onglet Hypothèses (cases jaunes). Vert : dans la cible ; \
             orange : à côté ; rouge : loin. Détail dans les onglets suivants ; mode d'emploi dans Lisez-moi.",
            s.version
        ),
        accent::SYNTHESE,
        G + COLONNES,
    )?;
    let titre = style::intertitre(accent::SYNTHESE);
    let entete = style::entete(accent::SYNTHESE);
    let etiquette = style::cellule(Nb::Texte, false).set_bold();

    // Les lignes d'attaque principale des jeux d'armes, et des ennemis.
    let principales: Vec<&Ecrite> =
        s.combat.iter().filter(|e| e.sorte == Sorte::Base && e.nom == "Attaque principale").collect();
    let frappes: Vec<&EnnemiEcrit> = s.ennemis.iter().filter(|e| e.principale).collect();
    let ordinaires = d.ennemis.iter().filter(|e| e.role != "chef").count() as u16;

    // 2. Tuer : la matrice des jeux d'armes contre les ennemis.
    let l_matrices = 9;
    let (c_tuer, c_mourir) = (G, G + 7);
    feuille.merge_range(l_matrices, c_tuer, l_matrices, c_tuer + 5, "Tuer : touches à l'attaque principale", &titre)?;
    feuille.merge_range(l_matrices + 1, c_tuer, l_matrices + 1, c_tuer + 1, "Jeu d'armes", &entete)?;
    for (i, ennemi) in d.ennemis.iter().enumerate() {
        feuille.write_string_with_format(l_matrices + 1, c_tuer + 2 + i as u16, &ennemi.role, &entete)?;
    }
    let tuer_debut = l_matrices + 2;
    for (i, e) in principales.iter().enumerate() {
        let l = tuer_debut + i as u32;
        let nom = d.jeux.iter().find(|j| j.id == e.jeu).map_or(e.jeu.as_str(), |j| j.nom.as_str());
        feuille.merge_range(l, c_tuer, l, c_tuer + 1, nom, &etiquette)?;
        for (k, (touches, _)) in e.contre.iter().enumerate() {
            let source = format!("='{ONGLET_COMBAT}'!{}", a1(e.ligne, C_ENNEMIS + 2 * k as u16));
            let format = style::cellule(Nb::Entier, false).set_align(FormatAlign::Center);
            style::ecrire(feuille, l, c_tuer + 2 + k as u16, &Val::F(source, *touches), &format)?;
        }
    }
    let tuer_fin = tuer_debut + principales.len().max(1) as u32 - 1;
    let plage_tuer = (tuer_debut, c_tuer + 2, tuer_fin, c_tuer + 1 + ordinaires);
    cible(feuille, plage_tuer, &hyp.r("cible.tuer_min"), &hyp.r("cible.tuer_max"))?;

    // 3. Mourir : la matrice des ennemis contre les classes.
    feuille.merge_range(l_matrices, c_mourir, l_matrices, c_mourir + 4, "Mourir : touches d'un ennemi, attaque principale", &titre)?;
    feuille.merge_range(l_matrices + 1, c_mourir, l_matrices + 1, c_mourir + 1, "Ennemi", &entete)?;
    for (i, classe) in d.classes.iter().enumerate() {
        feuille.write_string_with_format(l_matrices + 1, c_mourir + 2 + i as u16, &classe.nom, &entete)?;
    }
    let mourir_debut = l_matrices + 2;
    for (i, e) in frappes.iter().enumerate() {
        let l = mourir_debut + i as u32;
        feuille.merge_range(l, c_mourir, l, c_mourir + 1, &e.role, &etiquette)?;
        for (k, (touches, _)) in e.contre.iter().enumerate() {
            let source = format!("='{ONGLET_ENNEMIS}'!{}", a1(e.ligne, E_CLASSES + 2 * k as u16));
            let format = style::cellule(Nb::Entier, false).set_align(FormatAlign::Center);
            style::ecrire(feuille, l, c_mourir + 2 + k as u16, &Val::F(source, *touches), &format)?;
        }
    }
    let mourir_fin = mourir_debut + u32::from(ordinaires).max(1) - 1;
    let plage_mourir = (mourir_debut, c_mourir + 2, mourir_fin, c_mourir + 1 + d.classes.len() as u16);
    cible(feuille, plage_mourir, &hyp.r("cible.mourir_min"), &hyp.r("cible.mourir_max"))?;

    // Sous chaque matrice : combien de cases sont dans la cible.
    let plage = |p: (u32, u16, u32, u16)| format!("{}:{}", a1(p.0, p.1), a1(p.2, p.3));
    let dans_la_cible = |p: (u32, u16, u32, u16), mini: &str, maxi: &str| {
        format!("=SUMPRODUCT(({pl}>={mini})*({pl}<={maxi}))", pl = plage(p))
    };
    let compte = |valeurs: Vec<f64>, mini: f64, maxi: f64| valeurs.iter().filter(|v| **v >= mini && **v <= maxi).count() as f64;
    let note = style::note();
    let gras = style::cellule(Nb::Entier, false).set_bold().set_align(FormatAlign::Center);
    let tuer_valeurs: Vec<f64> =
        principales.iter().flat_map(|e| e.contre.iter().take(usize::from(ordinaires)).map(|c| c.0)).collect();
    let mourir_valeurs: Vec<f64> =
        frappes.iter().take(usize::from(ordinaires)).flat_map(|e| e.contre.iter().map(|c| c.0)).collect();
    let l_bilan = tuer_fin + 1;
    for (colonne, p, mini, maxi, valeurs, phrase) in [
        (c_tuer, plage_tuer, "cible.tuer_min", "cible.tuer_max", &tuer_valeurs, "cases dans la cible, sur"),
        (c_mourir, plage_mourir, "cible.mourir_min", "cible.mourir_max", &mourir_valeurs, "cases dans la cible, sur"),
    ] {
        style::ecrire(
            feuille,
            l_bilan,
            colonne,
            &Val::F(dans_la_cible(p, &hyp.r(mini), &hyp.r(maxi)), compte(valeurs.clone(), hyp.v(mini), hyp.v(maxi))),
            &gras,
        )?;
        feuille.merge_range(l_bilan, colonne + 1, l_bilan, colonne + 4, &format!("{phrase} {}", valeurs.len()), &note)?;
        feuille.merge_range(
            l_bilan + 1,
            colonne,
            l_bilan + 1,
            colonne + 4,
            "Cible : ennemis ordinaires (le chef n'est pas coloré). Elle se change dans Hypothèses.",
            &note,
        )?;
    }

    // 1. Les cartes (écrites après les matrices : elles en font la moyenne).
    feuille.merge_range(3, G, 3, G + COLONNES - 1, "En un coup d'œil", &titre)?;
    for l in 4..7 {
        feuille.set_row_height(l, [30.0, 36.0, 30.0][(l - 4) as usize])?;
    }
    let moyenne = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let exp = s.expedition;
    let autre = |onglet: &str, ligne: u32, colonne: u16| format!("='{onglet}'!{}", a1(ligne, colonne));
    let premier_ennemi = d.ennemis.first().map_or("", |e| e.role.as_str());
    let cartes: Vec<(String, Val, Nb, String)> = vec![
        (
            "Touches de base pour tuer un ennemi ordinaire".into(),
            Val::F(format!("=AVERAGE({})", plage(plage_tuer)), moyenne(&tuer_valeurs)),
            Nb::Dec1,
            format!("cible : {:.0} ; moyenne des jeux d'armes", hyp.v("cible.tuer_max")),
        ),
        (
            "Touches d'un ennemi ordinaire pour tuer un joueur".into(),
            Val::F(format!("=AVERAGE({})", plage(plage_mourir)), moyenne(&mourir_valeurs)),
            Nb::Dec1,
            format!("cible : {:.0} à {:.0} ; moyenne des classes", hyp.v("cible.mourir_min"), hyp.v("cible.mourir_max")),
        ),
        (
            "Créatures d'une expédition type".into(),
            Val::F(autre(expeditions::ONGLET, exp.reference, exp.c_creatures), exp.valeurs.0),
            Nb::Entier,
            "donjon moyen, difficulté et densité par défaut".into(),
        ),
        (
            "Minutes de frappe d'une expédition type".into(),
            Val::F(autre(expeditions::ONGLET, exp.reference, exp.c_minutes), exp.valeurs.3),
            Nb::Dec1,
            "temps passé à frapper, armures comprises".into(),
        ),
        (
            "Or d'une expédition type".into(),
            Val::F(autre(expeditions::ONGLET, exp.reference, exp.c_or), exp.valeurs.2),
            Nb::Entier,
            "coffres, créatures et chef".into(),
        ),
        (
            "Ennemis à tuer pour le premier point d'arme".into(),
            Val::F(
                autre(progression::ONGLET, s.premier_point, 3),
                hyp.v("salle.premier_point")
                    / (hyp.v(&format!("ennemi.{premier_ennemi}.experience")) * hyp.v("salle.maitrise_par_experience")),
            ),
            Nb::Dec1,
            format!("ennemis « {premier_ennemi} », seul"),
        ),
    ];
    for (i, (libelle, valeur, nb, precision)) in cartes.iter().enumerate() {
        carte(feuille, 4, G + 2 * i as u16, libelle, valeur, *nb, precision)?;
    }

    // 4. Les dégâts par seconde des attaques principales.
    let l_dps = l_bilan + 4;
    feuille.merge_range(l_dps, G, l_dps, G + COLONNES - 1, "Frapper : dégâts par seconde de l'attaque principale", &titre)?;
    feuille.merge_range(l_dps + 1, G, l_dps + 1, G + 1, "Jeu d'armes", &entete)?;
    feuille.write_string_with_format(l_dps + 1, G + 2, "Dégâts par seconde", &entete)?;
    for (i, e) in principales.iter().enumerate() {
        let l = l_dps + 2 + i as u32;
        let nom = d.jeux.iter().find(|j| j.id == e.jeu).map_or(e.jeu.as_str(), |j| j.nom.as_str());
        feuille.merge_range(l, G, l, G + 1, nom, &etiquette)?;
        style::ecrire(
            feuille,
            l,
            G + 2,
            &Val::F(format!("='{ONGLET_COMBAT}'!{}", a1(e.ligne, C_DPS)), e.dps),
            &style::cellule(Nb::Dec1, false),
        )?;
    }
    let n = principales.len().max(1) as u32;
    let dps = graphique(
        "Dégâts par seconde de l'attaque principale, sur une créature sans armure",
        (l_dps + 2, G, l_dps + 1 + n, G),
        (l_dps + 2, G + 2, l_dps + 1 + n, G + 2),
        accent::COMBAT,
    );
    feuille.insert_chart(l_dps + 1, G + 4, &dps)?;

    // 5. Ce que coûtent les achats, en expéditions.
    let l_achats = l_dps + 18;
    feuille.merge_range(l_achats, G, l_achats, G + COLONNES - 1, "S'offrir quelque chose : expéditions type qu'il faut", &titre)?;
    feuille.merge_range(l_achats + 1, G, l_achats + 1, G + 1, "Achat", &entete)?;
    feuille.write_string_with_format(l_achats + 1, G + 2, "Prix (or)", &entete)?;
    feuille.write_string_with_format(l_achats + 1, G + 3, "Expéditions type", &entete)?;
    for (i, (cle, nom)) in economie::ACHATS.iter().enumerate() {
        let l = l_achats + 2 + i as u32;
        feuille.merge_range(l, G, l, G + 1, nom, &etiquette)?;
        style::ecrire(feuille, l, G + 2, &Val::F(format!("={}", hyp.r(cle)), hyp.v(cle)), &style::cellule(Nb::Entier, false))?;
        style::ecrire(
            feuille,
            l,
            G + 3,
            &Val::F(autre(economie::ONGLET, s.achats.achats[i], s.achats.c_expeditions), hyp.v(cle) / exp.valeurs.2),
            &style::cellule(Nb::Dec2, false),
        )?;
    }
    let n = economie::ACHATS.len() as u32;
    let mut achats = graphique(
        "Expéditions type pour chaque achat (tout l'or du groupe)",
        (l_achats + 2, G, l_achats + 1 + n, G),
        (l_achats + 2, G + 3, l_achats + 1 + n, G + 3),
        accent::ECONOMIE,
    );
    achats.set_width(660).set_height(260);
    feuille.insert_chart(l_achats + 1, G + 5, &achats)?;
    Ok(())
}
