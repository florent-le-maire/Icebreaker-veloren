//! Icebreaker : l'onglet « Économie » — ce que donne chaque butin, face aux prix.

use crate::{
    butin::SORTES,
    donnees::Donnees,
    hyp::Hyp,
    style::{self, Nb, Val, a1, accent, col},
};
use anyhow::Result;
use rust_xlsxwriter::{Color, ConditionalFormatDataBar, Worksheet};

pub const ONGLET: &str = "Économie";

/// Les achats : (clé de l'hypothèse, ce qu'on lit).
pub const ACHATS: [(&str, &str); 5] = [
    ("prix.maison", "Une maison du village"),
    ("prix.jeu_armes", "Un jeu d'armes"),
    ("prix.ultime", "Un ultime"),
    ("prix.passif", "Un passif"),
    ("prix.soin", "Un soin"),
];

pub fn hypotheses(hyp: &mut Hyp, d: &Donnees) {
    let g = "Prix (or)";
    let salle = "assets/icebreaker/salle_armes.ron";
    hyp.ajoute(g, "prix.maison", "Une maison du village", f64::from(d.village.prix_maison), Nb::Entier, "assets/icebreaker/village.ron : prix_maison");
    hyp.ajoute(g, "prix.jeu_armes", "Apprendre un jeu d'armes", f64::from(d.salle.prix_jeu_armes), Nb::Entier, &format!("{salle} : prix_jeu_armes"));
    hyp.ajoute(g, "prix.ultime", "Apprendre un ultime", f64::from(d.salle.prix_ultime), Nb::Entier, &format!("{salle} : prix_ultime"));
    hyp.ajoute(g, "prix.passif", "Apprendre un passif", f64::from(d.salle.prix_passif), Nb::Entier, &format!("{salle} : prix_passif"));
    hyp.ajoute(g, "prix.soin", "Un soin au village", d.prix_soin.unwrap_or(0.0), Nb::Entier, "server/src/settings/mod.rs : heal_price (réglage du serveur, pas encore dans assets/)");

    let g = "Or moyen par butin";
    for table in &d.tables {
        hyp.ajoute(g, &format!("butin.{}.or", table.cle), &format!("{} : or moyen", table.role), table.or_moyen, Nb::Dec1, &format!("assets/{}.ron (nommée dans assets/icebreaker/butin.ron)", table.id.replace('.', "/")));
    }
}

/// Où sont les lignes des achats, pour y ajouter des colonnes (expéditions).
pub struct Ecrit {
    /// Ligne de chaque achat, dans l'ordre de `ACHATS`.
    pub achats: Vec<u32>,
    /// Première colonne libre à droite du tableau des achats.
    pub colonne_libre: u16,
    /// Ligne de l'en-tête du tableau des achats.
    pub entete_achats: u32,
}

pub fn ecrire(feuille: &mut Worksheet, d: &Donnees, hyp: &Hyp) -> Result<Ecrit> {
    feuille.set_name(ONGLET)?;
    let largeur = 13.0;
    let mut ligne = style::bandeau(
        feuille,
        "Économie : le butin face aux prix",
        "Ce qu'un coffre, une créature ou un boss donne en moyenne (toutes les branches de la table, pondérées), et \
         combien il en faut pour chaque achat. L'or moyen se change dans Hypothèses.",
        accent::ECONOMIE,
        12,
    )?;
    let titre = style::intertitre(accent::ECONOMIE);

    feuille.merge_range(ligne, 0, ligne, 10, "Ce que donne chaque butin, en moyenne", &titre)?;
    ligne += 1;
    let mut colonnes = vec![
        col("Butin", 26.0, Nb::Texte),
        col("Or au moins", largeur, Nb::Entier),
        col("Or moyen", largeur, Nb::Dec1),
        col("Or au plus", largeur, Nb::Entier),
    ];
    colonnes.extend(SORTES.iter().map(|s| col(s, largeur, Nb::Dec2)));
    colonnes.push(col("Objets en tout", largeur, Nb::Dec2));
    colonnes.push(col("Table (fichier)", 52.0, Nb::Texte));
    let groupes = [(1, 3, "Or".to_string()), (4, 4 + SORTES.len() as u16, "Objets par butin".to_string())];
    let lignes: Vec<Vec<Val>> = d
        .tables
        .iter()
        .map(|t| {
            let cle = format!("butin.{}.or", t.cle);
            let mut valeurs = vec![
                Val::t(&t.role),
                Val::N(t.or_min),
                Val::F(format!("={}", hyp.r(&cle)), hyp.v(&cle)),
                Val::N(t.or_max),
            ];
            valeurs.extend(t.par_sorte.iter().map(|n| if *n > 0.0 { Val::N(*n) } else { Val::Vide }));
            valeurs.push(Val::N((t.par_sorte.iter().sum::<f64>() * 100.0).round() / 100.0));
            valeurs.push(Val::t(format!("assets/{}.ron", t.id.replace('.', "/"))));
            valeurs
        })
        .collect();
    let (_, debut, fin) = style::tableau(feuille, ligne, 0, accent::ECONOMIE, &groupes, &colonnes, &lignes, |i| {
        d.tables[i].cle.split('_').next().unwrap_or_default().to_string()
    })?;
    let barres = ConditionalFormatDataBar::new().set_fill_color(Color::RGB(0xEBCB6B)).set_solid_fill(true);
    feuille.add_conditional_format(debut, 2, fin, 2, &barres)?;

    ligne = fin + 3;
    feuille.merge_range(ligne, 0, ligne, 10, "Combien de butins pour s'offrir…", &titre)?;
    ligne += 1;
    let mut colonnes = vec![col("Achat", 26.0, Nb::Texte), col("Prix (or)", largeur, Nb::Entier)];
    colonnes.extend(d.tables.iter().map(|t| col(&t.role, largeur, Nb::Dec1)));
    let entete_achats = ligne;
    let lignes: Vec<Vec<Val>> = ACHATS
        .iter()
        .enumerate()
        .map(|(i, (cle, nom))| {
            let l = ligne + 1 + i as u32;
            let mut valeurs = vec![Val::t(*nom), Val::F(format!("={}", hyp.r(cle)), hyp.v(cle))];
            for t in &d.tables {
                let or = format!("butin.{}.or", t.cle);
                valeurs.push(Val::F(format!("={}/{}", a1(l, 1), hyp.r(&or)), hyp.v(cle) / hyp.v(&or)));
            }
            valeurs
        })
        .collect();
    let (_, debut, _) = style::tableau(feuille, ligne, 0, accent::ECONOMIE, &[], &colonnes, &lignes, |i| i.to_string())?;
    feuille.set_freeze_panes(3, 1)?;
    Ok(Ecrit {
        achats: (0..ACHATS.len() as u32).map(|i| debut + i).collect(),
        colonne_libre: colonnes.len() as u16,
        entete_achats,
    })
}
