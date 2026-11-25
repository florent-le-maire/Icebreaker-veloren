//! Icebreaker : l'allure commune des onglets du classeur d'équilibrage, et de quoi écrire un tableau.

use anyhow::Result;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Formula, Worksheet};
use std::cell::RefCell;

pub const POLICE: &str = "Arial";
pub const ENCRE: u32 = 0x1F2937;
pub const GRIS: u32 = 0x6B7280;
pub const TRAIT: u32 = 0xD9DEE5;
pub const BANDE: u32 = 0xF4F6F9;
/// Une case à modifier : bleu sur jaune pâle.
pub const SAISIE_FOND: u32 = 0xFFF7D6;
pub const SAISIE_ENCRE: u32 = 0x0B3FC4;
pub const VERT: u32 = 0xCDEBD3;
pub const VERT_ENCRE: u32 = 0x14532D;
pub const ORANGE: u32 = 0xFDE3B8;
pub const ORANGE_ENCRE: u32 = 0x7C3A00;
pub const ROUGE: u32 = 0xF8C9C9;
pub const ROUGE_ENCRE: u32 = 0x7F1D1D;

/// La couleur d'accent de chaque onglet.
pub mod accent {
    pub const SYNTHESE: u32 = 0x0F766E;
    pub const HYPOTHESES: u32 = 0xB45309;
    pub const COMBAT: u32 = 0xB91C1C;
    pub const ENNEMIS: u32 = 0x6D28D9;
    pub const PROGRESSION: u32 = 0x1D4ED8;
    pub const ECONOMIE: u32 = 0xA16207;
    pub const EXPEDITIONS: u32 = 0x15803D;
    pub const REGLAGES: u32 = 0x475569;
    pub const LISEZ_MOI: u32 = 0x334155;
}

/// Comment un nombre s'affiche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nb {
    Texte,
    Entier,
    Dec1,
    Dec2,
    Pct,
    /// Un multiplicateur : « × 1,25 ».
    Fois,
    Secondes,
}

impl Nb {
    fn format(self) -> Option<&'static str> {
        match self {
            Nb::Texte => None,
            Nb::Entier => Some("#,##0"),
            Nb::Dec1 => Some("#,##0.0"),
            Nb::Dec2 => Some("0.00"),
            Nb::Pct => Some("0%"),
            Nb::Fois => Some("\"× \"0.00"),
            Nb::Secondes => Some("0.0\" s\""),
        }
    }
}

fn base() -> Format { Format::new().set_font_name(POLICE).set_font_size(10).set_font_color(Color::RGB(ENCRE)) }

fn avec_nombre(format: Format, nb: Nb) -> Format {
    match nb.format() {
        Some(f) => format.set_num_format(f).set_align(FormatAlign::Right),
        None => format.set_align(FormatAlign::Left),
    }
}

/// Une cellule de tableau ; `bande` : ligne teintée.
pub fn cellule(nb: Nb, bande: bool) -> Format {
    let f = avec_nombre(base(), nb)
        .set_align(FormatAlign::VerticalCenter)
        .set_border_bottom(FormatBorder::Thin)
        .set_border_bottom_color(Color::RGB(TRAIT));
    if bande { f.set_background_color(Color::RGB(BANDE)) } else { f }
}

/// Une cellule de texte long, qui passe à la ligne.
pub fn texte_long(bande: bool) -> Format { cellule(Nb::Texte, bande).set_text_wrap().set_font_color(Color::RGB(GRIS)) }

/// Une case à modifier.
pub fn saisie(nb: Nb) -> Format {
    avec_nombre(base(), nb)
        .set_bold()
        .set_font_color(Color::RGB(SAISIE_ENCRE))
        .set_background_color(Color::RGB(SAISIE_FOND))
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(0xE5C76B))
}

pub fn entete(accent: u32) -> Format {
    base()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(accent))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::White)
}

/// Le titre d'un groupe de colonnes, au-dessus de l'en-tête.
pub fn groupe() -> Format {
    base()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(ENCRE))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::White)
}

/// Un intertitre dans un onglet.
pub fn intertitre(accent: u32) -> Format {
    base()
        .set_bold()
        .set_font_size(12)
        .set_font_color(Color::RGB(accent))
        .set_border_bottom(FormatBorder::Medium)
        .set_border_bottom_color(Color::RGB(accent))
}

pub fn note() -> Format { base().set_italic().set_font_color(Color::RGB(GRIS)).set_text_wrap().set_align(FormatAlign::Top) }

pub fn texte() -> Format { base().set_text_wrap().set_align(FormatAlign::Top) }

/// Une pastille de couleur (mise en forme conditionnelle).
pub fn pastille(fond: u32, encre: u32) -> Format {
    Format::new().set_background_color(Color::RGB(fond)).set_font_color(Color::RGB(encre))
}

/// Le bandeau de titre d'un onglet : titre, puis une ligne d'explication ; renvoie la première ligne libre.
pub fn bandeau(feuille: &mut Worksheet, titre: &str, explication: &str, accent: u32, colonnes: u16) -> Result<u32> {
    let titre_format = base()
        .set_bold()
        .set_font_size(18)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(accent))
        .set_align(FormatAlign::VerticalCenter)
        .set_indent(1);
    let explication_format = base()
        .set_italic()
        .set_font_color(Color::RGB(GRIS))
        .set_background_color(Color::RGB(BANDE))
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_indent(1);
    let fin = colonnes.max(2) - 1;
    feuille.merge_range(0, 0, 0, fin, titre, &titre_format)?;
    feuille.set_row_height(0, 34)?;
    feuille.merge_range(1, 0, 1, fin, explication, &explication_format)?;
    feuille.set_row_height(1, 32)?;
    feuille.set_tab_color(Color::RGB(accent));
    ONGLETS.with_borrow_mut(|o| o.push(Onglet { nom: feuille.name(), titre: titre.to_string(), explication: explication.to_string() }));
    Ok(3)
}

/// Une valeur de tableau.
#[derive(Clone, Debug)]
pub enum Val {
    Vide,
    T(String),
    N(f64),
    /// Une formule et son résultat calculé ici, pour un lecteur qui ne recalcule pas.
    F(String, f64),
    /// Une formule dont le résultat est vide (« sans objet »).
    FVide(String),
}

impl Val {
    pub fn t(texte: impl Into<String>) -> Self { Val::T(texte.into()) }
}

pub struct Col {
    pub titre: String,
    pub largeur: f64,
    pub nb: Nb,
    /// Texte long qui passe à la ligne.
    pub long: bool,
}

pub fn col(titre: &str, largeur: f64, nb: Nb) -> Col { Col { titre: titre.to_string(), largeur, nb, long: false } }

pub fn col_longue(titre: &str, largeur: f64) -> Col { Col { titre: titre.to_string(), largeur, nb: Nb::Texte, long: true } }

/// Écrit une valeur.
pub fn ecrire(feuille: &mut Worksheet, ligne: u32, colonne: u16, valeur: &Val, format: &Format) -> Result<()> {
    match valeur {
        Val::Vide => {
            feuille.write_blank(ligne, colonne, format)?;
        },
        Val::T(texte) => {
            feuille.write_string_with_format(ligne, colonne, texte, format)?;
        },
        Val::N(n) => {
            feuille.write_number_with_format(ligne, colonne, *n, format)?;
        },
        Val::F(formule, resultat) => {
            let resultat = if resultat.is_finite() { resultat.to_string() } else { String::new() };
            feuille.write_formula_with_format(ligne, colonne, Formula::new(formule).set_result(resultat), format)?;
        },
        Val::FVide(formule) => {
            feuille.write_formula_with_format(ligne, colonne, Formula::new(formule).set_result("—"), format)?;
        },
    }
    Ok(())
}

/// Un tableau : titres de groupes facultatifs (première colonne, dernière, titre), en-tête, lignes. Les lignes
/// sont teintées par paquets : la teinte change quand `paquet` change. Renvoie (ligne de l'en-tête, première
/// ligne de données, dernière ligne de données).
pub fn tableau(
    feuille: &mut Worksheet,
    depart: u32,
    premiere_colonne: u16,
    accent: u32,
    groupes: &[(u16, u16, String)],
    colonnes: &[Col],
    lignes: &[Vec<Val>],
    paquet: impl Fn(usize) -> String,
) -> Result<(u32, u32, u32)> {
    let mut ligne = depart;
    if !groupes.is_empty() {
        for (debut, fin, titre) in groupes {
            if debut == fin {
                feuille.write_string_with_format(ligne, premiere_colonne + debut, titre, &groupe())?;
            } else {
                feuille.merge_range(ligne, premiere_colonne + debut, ligne, premiere_colonne + fin, titre, &groupe())?;
            }
        }
        feuille.set_row_height(ligne, 20)?;
        ligne += 1;
    }
    let ligne_entete = ligne;
    let entete = entete(accent);
    for (i, colonne) in colonnes.iter().enumerate() {
        let c = premiere_colonne + i as u16;
        feuille.write_string_with_format(ligne, c, &colonne.titre, &entete)?;
        feuille.set_column_width(c, colonne.largeur)?;
    }
    feuille.set_row_height(ligne, 44)?;
    ligne += 1;
    let premiere = ligne;
    let mut bande = false;
    let mut precedent = None;
    for (i, valeurs) in lignes.iter().enumerate() {
        let cle = paquet(i);
        if precedent.as_ref().is_some_and(|p| *p != cle) {
            bande = !bande;
        }
        precedent = Some(cle);
        for (j, valeur) in valeurs.iter().enumerate() {
            let colonne = &colonnes[j];
            let format = if colonne.long { texte_long(bande) } else { cellule(colonne.nb, bande) };
            ecrire(feuille, ligne, premiere_colonne + j as u16, valeur, &format)?;
        }
        ligne += 1;
    }
    // La même chose en texte, pour `texte.rs` : un titre répété (« Touches pour tuer ») prend celui de son groupe.
    let titres: Vec<String> = colonnes
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let double = colonnes.iter().filter(|autre| autre.titre == c.titre).count() > 1;
            match groupes.iter().find(|(debut, fin, _)| double && (*debut..=*fin).contains(&(i as u16))) {
                Some((_, _, groupe)) => format!("{groupe} — {}", c.titre),
                None => c.titre.clone(),
            }
        })
        .collect();
    let en_texte = lignes.iter().map(|l| l.iter().zip(colonnes).map(|(v, c)| texte_de(v, c.nb)).collect()).collect();
    noter(&feuille.name(), &SECTION.take(), titres, en_texte);
    Ok((ligne_entete, premiere, ligne.saturating_sub(1).max(premiere)))
}

/// `A1` d'une case.
pub fn a1(ligne: u32, colonne: u16) -> String { rust_xlsxwriter::utility::row_col_to_cell(ligne, colonne) }

/// Un onglet écrit : son nom, le titre et l'explication de son bandeau.
pub struct Onglet {
    pub nom: String,
    pub titre: String,
    pub explication: String,
}

/// Un tableau écrit, en texte : ce que `texte.rs` range dans les fichiers lisibles sans tableur.
pub struct Tableau {
    pub onglet: String,
    /// L'intertitre qui le précède (vide : le seul tableau de l'onglet).
    pub titre: String,
    pub colonnes: Vec<String>,
    pub lignes: Vec<Vec<String>>,
}

thread_local! {
    static ONGLETS: RefCell<Vec<Onglet>> = const { RefCell::new(Vec::new()) };
    static TABLEAUX: RefCell<Vec<Tableau>> = const { RefCell::new(Vec::new()) };
    static SECTION: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Un intertitre au-dessus d'un tableau ; il sera aussi le titre du tableau dans la version texte.
pub fn section(feuille: &mut Worksheet, ligne: u32, derniere_colonne: u16, titre: &str, accent: u32) -> Result<()> {
    feuille.merge_range(ligne, 0, ligne, derniere_colonne, titre, &intertitre(accent))?;
    SECTION.set(titre.to_string());
    Ok(())
}

/// Garde un tableau pour la version texte.
pub fn noter(onglet: &str, titre: &str, colonnes: Vec<String>, lignes: Vec<Vec<String>>) {
    TABLEAUX.with_borrow_mut(|t| {
        t.push(Tableau { onglet: onglet.to_string(), titre: titre.to_string(), colonnes, lignes })
    });
}

/// Tout ce qui a été écrit, dans l'ordre.
pub fn ecrits() -> (Vec<Onglet>, Vec<Tableau>) { (ONGLETS.take(), TABLEAUX.take()) }

/// Un nombre comme on le lit dans le classeur, avec un point.
pub fn nombre_en_texte(x: f64, nb: Nb) -> String {
    if !x.is_finite() {
        return "—".into();
    }
    match nb {
        Nb::Entier => format!("{x:.0}"),
        Nb::Dec1 | Nb::Secondes => format!("{x:.1}"),
        Nb::Pct => format!("{:.0} %", x * 100.0),
        Nb::Dec2 | Nb::Fois | Nb::Texte => format!("{}", (x * 100.0).round() / 100.0),
    }
}

fn texte_de(valeur: &Val, nb: Nb) -> String {
    match valeur {
        Val::Vide => String::new(),
        Val::T(texte) => texte.clone(),
        Val::N(n) | Val::F(_, n) => nombre_en_texte(*n, nb),
        Val::FVide(_) => "—".into(),
    }
}
