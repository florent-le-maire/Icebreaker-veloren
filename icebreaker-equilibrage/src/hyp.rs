//! Icebreaker : l'onglet « Hypothèses », les cases « et si » du classeur. Chaque chiffre du jeu qui entre dans
//! un calcul y a sa case ; les formules des autres onglets pointent dessus.

use crate::style::{self, Nb, accent};
use anyhow::Result;
use rust_xlsxwriter::{Formula, Worksheet};
use std::collections::HashMap;

pub const ONGLET: &str = "Hypothèses";
const COL_LIBELLE: u16 = 0;
const COL_VALEUR: u16 = 1;
const COL_JEU: u16 = 2;
const COL_ECART: u16 = 3;
const COL_SOURCE: u16 = 4;

struct Param {
    groupe: String,
    libelle: String,
    valeur: f64,
    nb: Nb,
    source: String,
    ligne: u32,
}

pub struct Hyp {
    params: Vec<Param>,
    par_cle: HashMap<String, usize>,
    prochaine: u32,
}

impl Hyp {
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
            par_cle: HashMap::new(),
            prochaine: 4,
        }
    }

    /// Ajoute une case ; `source` dit dans quel fichier la valeur se règle pour de bon.
    pub fn ajoute(&mut self, groupe: &str, cle: &str, libelle: &str, valeur: f64, nb: Nb, source: &str) {
        if self.params.last().is_none_or(|p| p.groupe != groupe) {
            self.prochaine += 2;
        }
        self.par_cle.insert(cle.to_string(), self.params.len());
        self.params.push(Param {
            groupe: groupe.to_string(),
            libelle: libelle.to_string(),
            valeur,
            nb,
            source: source.to_string(),
            ligne: self.prochaine,
        });
        self.prochaine += 1;
    }

    fn param(&self, cle: &str) -> &Param {
        &self.params[*self.par_cle.get(cle).unwrap_or_else(|| panic!("hypothèse inconnue : {cle}"))]
    }

    /// La référence de la case, à mettre dans une formule.
    pub fn r(&self, cle: &str) -> String { format!("'{ONGLET}'!$B${}", self.param(cle).ligne + 1) }

    /// La valeur du jeu.
    pub fn v(&self, cle: &str) -> f64 { self.param(cle).valeur }

    pub fn ecrire(&self, feuille: &mut Worksheet) -> Result<()> {
        feuille.set_name(ONGLET)?;
        style::bandeau(
            feuille,
            "Hypothèses : les cases « et si »",
            "Changer une valeur de la colonne jaune recalcule tout le classeur. La vraie valeur se règle dans le \
             fichier indiqué ; régénérer le classeur (icebreaker.ps1 equilibrage) remet les valeurs du jeu.",
            accent::HYPOTHESES,
            5,
        )?;
        for (colonne, largeur) in
            [(COL_LIBELLE, 52.0), (COL_VALEUR, 14.0), (COL_JEU, 14.0), (COL_ECART, 12.0), (COL_SOURCE, 70.0)]
        {
            feuille.set_column_width(colonne, largeur)?;
        }
        let entete = style::entete(accent::HYPOTHESES);
        for (colonne, titre) in [
            (COL_LIBELLE, "Réglage"),
            (COL_VALEUR, "Valeur essayée"),
            (COL_JEU, "Valeur du jeu"),
            (COL_ECART, "Écart"),
            (COL_SOURCE, "Où la régler"),
        ] {
            feuille.write_string_with_format(3, colonne, titre, &entete)?;
        }
        feuille.set_row_height(3, 24)?;
        feuille.set_freeze_panes(4, 1)?;
        let mut groupe = "";
        for p in &self.params {
            if p.groupe != groupe {
                groupe = &p.groupe;
                feuille.merge_range(p.ligne - 1, 0, p.ligne - 1, COL_SOURCE, groupe, &style::intertitre(accent::HYPOTHESES))?;
                feuille.set_row_height(p.ligne - 1, 22)?;
            }
            feuille.write_string_with_format(p.ligne, COL_LIBELLE, &p.libelle, &style::cellule(Nb::Texte, false))?;
            feuille.write_number_with_format(p.ligne, COL_VALEUR, p.valeur, &style::saisie(p.nb))?;
            feuille.write_number_with_format(p.ligne, COL_JEU, p.valeur, &style::cellule(p.nb, false))?;
            let l = p.ligne + 1;
            feuille.write_formula_with_format(
                p.ligne,
                COL_ECART,
                Formula::new(format!("=IF(B{l}<>C{l},\"modifié\",\"—\")")).set_result("—"),
                &style::cellule(Nb::Texte, false).set_bold().set_font_color(rust_xlsxwriter::Color::RGB(accent::HYPOTHESES)),
            )?;
            feuille.write_string_with_format(p.ligne, COL_SOURCE, &p.source, &style::texte_long(false))?;
        }
        // La version texte : un tableau par groupe.
        let colonnes = || vec!["Réglage".to_string(), "Valeur du jeu".to_string(), "Où la régler".to_string()];
        let mut groupes: Vec<(&str, Vec<Vec<String>>)> = Vec::new();
        for p in &self.params {
            if groupes.last().is_none_or(|(g, _)| *g != p.groupe) {
                groupes.push((&p.groupe, Vec::new()));
            }
            let ligne = vec![p.libelle.clone(), style::nombre_en_texte(p.valeur, p.nb), p.source.clone()];
            groupes.last_mut().expect("un groupe vient d'être ajouté").1.push(ligne);
        }
        for (groupe, lignes) in groupes {
            style::noter(ONGLET, groupe, colonnes(), lignes);
        }
        Ok(())
    }
}
