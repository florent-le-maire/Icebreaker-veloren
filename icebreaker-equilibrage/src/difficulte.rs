//! Icebreaker : l'onglet « Difficulté » (jalon DF) — les six difficultés d'une expédition
//! (`assets/icebreaker/difficulte.ron`), et ce qu'elles donnent une fois leurs effets multipliés : les ennemis
//! comptés dans de vrais lieux générés, les PV à abattre et les dégâts reçus par rapport à la difficulté par défaut.

use crate::{
    donnees::Donnees,
    style::{self, Nb, Val, a1, col},
};
use anyhow::{Result, anyhow};
use common::icebreaker_difficulte::IcebreakerDifficulte;
use rust_xlsxwriter::{Color, ConditionalFormatDataBar, Format, Worksheet};
use vek::Vec3;
use world::icebreaker::{foret, grotte};

pub const ONGLET: &str = "Difficulté";
const ACCENT: u32 = 0xBE185D;
/// Grottes et forêts générées par difficulté (le donjon : les cas de l'onglet Expéditions).
const GRAINES: u64 = 6;

/// Une difficulté, ses réglages et ce qu'on a compté.
#[derive(Clone, Debug)]
pub struct Ligne {
    pub numero: u8,
    pub defaut: bool,
    pub nombre: f64,
    pub pv: f64,
    pub degats: f64,
    pub habilete: String,
    /// Poids guerrier / archer / mage, ou « tirage du lieu ».
    pub classes: String,
    /// Ennemis en moyenne : donjon moyen, grotte, forêt.
    pub donjon: f64,
    pub grotte: f64,
    pub foret: f64,
}

impl Ligne {
    /// Les ennemis des trois lieux, ensemble.
    fn ennemis(&self) -> f64 { self.donjon + self.grotte + self.foret }
}

/// Les six lignes : les réglages lus, les lieux générés à chaque difficulté.
pub fn lire(d: &Donnees) -> Result<Vec<Ligne>> {
    let difficultes = IcebreakerDifficulte::actuel();
    let defaut = difficultes.borne(None);
    let mut grottes = grotte::Settings::load().map_err(|e| anyhow!("grotte.ron : {e}"))?;
    let mut forets = foret::Settings::load().map_err(|e| anyhow!("foret.ron : {e}"))?;
    let bouche = Vec3::new(0, 0, 200);
    let moyenne = |total: usize| (total as f64 / GRAINES as f64 * 10.0).round() / 10.0;
    (1..=difficultes.max())
        .map(|numero| {
            let niveau = difficultes.niveau(Some(numero));
            grottes.difficulte = niveau.clone();
            forets.difficulte = niveau.clone();
            let grotte: usize = (1..=GRAINES)
                .map(|graine| grotte::generate(&grottes, graine, bouche, None, None, &|_| bouche.z).inhabitants.len())
                .sum();
            let mut foret = 0;
            for graine in 1..=GRAINES {
                let lieu = foret::generate(&forets, graine, Vec3::new(2048, 2048, 200), None).map_err(|e| anyhow!("forêt : {e}"))?;
                foret += lieu.inhabitants.len();
            }
            let donjon = d
                .donjon
                .cas
                .iter()
                .find(|c| c.taille == "medium" && c.menace == i32::from(numero) && c.densite == d.donjon.densite_defaut)
                .map_or(0.0, |c| c.par_sorte.iter().sum());
            let poids = |role: &str| niveau.classes.get(role).copied().unwrap_or(1.0);
            Ok(Ligne {
                numero,
                defaut: numero == defaut,
                nombre: f64::from(niveau.nombre),
                pv: f64::from(niveau.pv),
                degats: f64::from(niveau.degats),
                habilete: match niveau.competences.get("guerrier").and_then(|c| c.split('.').find(|m| m.starts_with("rank"))) {
                    Some(rang) => format!("{} (rang {})", niveau.ia, rang.trim_start_matches("rank")),
                    None => format!("{} (celle des fiches)", niveau.ia),
                },
                classes: if niveau.classes.is_empty() {
                    "tirage du lieu".to_string()
                } else {
                    format!("{} / {} / {}", poids("guerrier"), poids("archer"), poids("mage"))
                },
                donjon,
                grotte: moyenne(grotte),
                foret: moyenne(foret),
            })
        })
        .collect()
}

/// Par rapport à la difficulté par défaut : (ennemis, PV à abattre, dégâts reçus par seconde).
pub fn relatif(ligne: &Ligne, lignes: &[Ligne]) -> (f64, f64, f64) {
    let reference = lignes.iter().find(|l| l.defaut).map_or(1.0, Ligne::ennemis).max(1.0);
    let ennemis = ligne.ennemis() / reference;
    (ennemis, ennemis * ligne.pv, ennemis * ligne.degats)
}

pub fn ecrire(feuille: &mut Worksheet, lignes: &[Ligne]) -> Result<()> {
    feuille.set_name(ONGLET)?;
    let colonnes = [
        col("Difficulté", 10.0, Nb::Entier),
        col("Repère", 11.0, Nb::Texte),
        col("Nombre d'ennemis", 10.0, Nb::Fois),
        col("PV des ennemis", 10.0, Nb::Fois),
        col("Dégâts des ennemis", 10.0, Nb::Fois),
        col("Habileté", 22.0, Nb::Texte),
        col("Poids guerrier / archer / mage", 16.0, Nb::Texte),
        col("Ennemis, donjon moyen", 10.0, Nb::Dec1),
        col("Ennemis, grotte", 10.0, Nb::Dec1),
        col("Ennemis, forêt", 10.0, Nb::Dec1),
        col("Ennemis à affronter", 11.0, Nb::Fois),
        col("PV à abattre", 11.0, Nb::Fois),
        col("Dégâts reçus par seconde", 11.0, Nb::Fois),
        col("Coups pour tuer un ennemi", 11.0, Nb::Fois),
        col("Coups encaissés avant de mourir", 11.0, Nb::Fois),
    ];
    let ligne = style::bandeau(
        feuille,
        "Difficulté : les six difficultés d'une expédition",
        &format!(
            "Réglages lus dans assets/icebreaker/difficulte.ron. Ennemis comptés dans de vrais lieux générés (donjon \
             moyen : les cas de l'onglet Expéditions ; grotte et forêt : {GRAINES} graines). Les colonnes de droite \
             multiplient les effets, par rapport à la difficulté par défaut ; l'habileté n'y est pas comptée."
        ),
        ACCENT,
        colonnes.len() as u16,
    )?;
    style::section(feuille, ligne, colonnes.len() as u16 - 1, "Les six difficultés", ACCENT)?;
    let groupes = [
        (0, 1, "L'expédition".to_string()),
        (2, 6, "Réglages (difficulte.ron)".to_string()),
        (7, 9, "Ennemis comptés".to_string()),
        (10, 14, "Par rapport à la difficulté par défaut".to_string()),
    ];
    // Les lignes de données commencent après l'intertitre, les groupes et l'en-tête.
    let premiere = ligne + 3;
    let reference = premiere + lignes.iter().position(|l| l.defaut).unwrap_or(0) as u32;
    let total = |l: u32| format!("({}+{}+{})", a1(l, 7), a1(l, 8), a1(l, 9));
    let valeurs: Vec<Vec<Val>> = lignes
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let l = premiere + i as u32;
            let (ennemis, pv, degats) = relatif(x, lignes);
            vec![
                Val::N(f64::from(x.numero)),
                if x.defaut { Val::t("par défaut") } else { Val::Vide },
                Val::N(x.nombre),
                Val::N(x.pv),
                Val::N(x.degats),
                Val::t(x.habilete.clone()),
                Val::t(x.classes.clone()),
                Val::N(x.donjon),
                Val::N(x.grotte),
                Val::N(x.foret),
                Val::F(format!("={}/{}", total(l), total(reference)), ennemis),
                Val::F(format!("={}*{}", a1(l, 10), a1(l, 3)), pv),
                Val::F(format!("={}*{}", a1(l, 10), a1(l, 4)), degats),
                Val::F(format!("={}/{}", a1(l, 3), a1(reference, 3)), x.pv),
                Val::F(format!("={}/{}", a1(reference, 4), a1(l, 4)), 1.0 / x.degats),
            ]
        })
        .collect();
    let (_, debut, fin) = style::tableau(feuille, ligne + 1, 0, ACCENT, &groupes, &colonnes, &valeurs, |i| i.to_string())?;
    debug_assert_eq!(debut, premiere);
    for (colonne, couleur) in [(10, 0x9FD8B0), (11, 0xF2A7A7), (12, 0xF5B58A)] {
        let barres = ConditionalFormatDataBar::new().set_fill_color(Color::RGB(couleur)).set_solid_fill(true);
        feuille.add_conditional_format(debut, colonne, fin, colonne, &barres)?;
    }
    let gras = Format::new().set_bold();
    let defaut = rust_xlsxwriter::ConditionalFormatFormula::new()
        .set_rule(format!("=${}=\"par défaut\"", a1(debut, 1)).as_str())
        .set_format(&gras);
    feuille.add_conditional_format(debut, 0, fin, colonnes.len() as u16 - 1, &defaut)?;
    Ok(())
}

/// Le tableau de la vue d'ensemble, en Markdown.
pub fn resume(lignes: &[Ligne]) -> String {
    let colonnes: Vec<String> = [
        "Difficulté",
        "Nombre",
        "PV",
        "Dégâts",
        "Habileté",
        "Guerrier / archer / mage",
        "Ennemis (donjon moyen, grotte, forêt)",
        "PV à abattre",
        "Dégâts reçus par seconde",
    ]
    .iter()
    .map(|c| c.to_string())
    .collect();
    let corps: Vec<Vec<String>> = lignes
        .iter()
        .map(|x| {
            let (_, pv, degats) = relatif(x, lignes);
            vec![
                if x.defaut { format!("{} (par défaut)", x.numero) } else { x.numero.to_string() },
                format!("× {:.2}", x.nombre),
                format!("× {:.2}", x.pv),
                format!("× {:.2}", x.degats),
                x.habilete.clone(),
                x.classes.clone(),
                format!("{:.0}, {:.0}, {:.0}", x.donjon, x.grotte, x.foret),
                format!("× {pv:.2}"),
                format!("× {degats:.2}"),
            ]
        })
        .collect();
    format!(
        "\n## Les six difficultés\n\nRéglages de `assets/icebreaker/difficulte.ron` ; ennemis comptés dans de vrais lieux \
         générés ; les deux dernières colonnes multiplient les effets, par rapport à la difficulté par défaut \
         (l'habileté n'y est pas comptée).\n\n{}",
        crate::texte::tableau(&colonnes, &corps)
    )
}
