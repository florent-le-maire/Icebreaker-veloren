//! Icebreaker CA : l'affichage de la carte 3D de l'expédition. La logique (ce qu'on a exploré, l'image)
//! est dans `common::icebreaker_carte` ; la session tient la carte et la redessine, ce module montre
//! l'image dans un cadre au centre de l'écran, avec le titre, les objectifs restants et l'aide.

use super::icebreaker::{BORDURE, DISCRET, FOND, OR, TEXTE, cadre};
use crate::{
    hud::{Graphic, Ui},
    ui::fonts::Fonts,
};
use common::icebreaker_carte::Image as ImageCarte;
use conrod_core::{
    Colorable, Positionable, Sizeable, UiCell, Widget, image,
    widget::{self, Image, Text},
    widget_ids,
};
use i18n::Localization;
use std::sync::Arc;
use vek::Rgba;

/// Marge du cadre autour de l'image, et hauteur des bandes de texte au-dessus et en dessous.
const MARGE: f64 = 12.0;
const BANDE: f64 = 34.0;
/// Hauteur d'une ligne de texte sous l'image.
const LIGNE: f64 = 24.0;

/// La carte ouverte, telle que la session la donne au HUD.
#[derive(Clone, Debug)]
pub struct Ouverte {
    pub image: Arc<ImageCarte>,
    /// Change à chaque nouveau dessin.
    pub version: u64,
    /// Objectifs du lieu, et ceux qui restent à remplir.
    pub objectifs: usize,
    pub restants: usize,
}

widget_ids! {
    struct Ids {
        fond,
        image,
        titre,
        objectifs,
        aide,
    }
}

pub struct Carte {
    ids: Ids,
    /// L'image donnée à l'interface, et la version qu'elle montre.
    graphique: Option<(image::Id, u64)>,
}

impl Carte {
    pub fn new(generateur: widget::id::Generator) -> Self {
        Self {
            ids: Ids::new(generateur),
            graphique: None,
        }
    }

    /// Donne à l'interface le dernier dessin de la carte (avant de poser les widgets).
    pub fn maintenir(&mut self, ui: &mut Ui, ouverte: Option<&Ouverte>) {
        let Some(ouverte) = ouverte else {
            return;
        };
        if self.graphique.is_some_and(|(_, version)| version == ouverte.version) {
            return;
        }
        let cote = ouverte.image.cote;
        let Some(pixels) = ::image::RgbaImage::from_raw(cote, cote, ouverte.image.rgba.clone()) else {
            return;
        };
        let graphique = Graphic::Image(
            Arc::new(::image::DynamicImage::ImageRgba8(pixels)),
            Some(Rgba::from([0.0, 0.0, 0.0, 0.0])),
        );
        let id = match self.graphique {
            Some((id, _)) => {
                ui.replace_graphic(id, graphique);
                id
            },
            None => ui.add_graphic(graphique),
        };
        self.graphique = Some((id, ouverte.version));
    }

    pub fn dessiner(&mut self, ui: &mut UiCell, ouverte: &Ouverte, fonts: &Fonts, i18n: &Localization) {
        let Some((image, _)) = self.graphique else {
            return;
        };
        let ids = &self.ids;
        let cote = ouverte.image.cote as f64;
        let hauteur = cote + 2.0 * MARGE + BANDE + 2.0 * LIGNE;
        cadre([cote + 2.0 * MARGE, hauteur], FOND, BORDURE, 2.0)
            .x_y(0.0, 0.0)
            .set(ids.fond, ui);
        Text::new(&i18n.get_msg("icebreaker-carte-titre"))
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(20))
            .color(OR)
            .mid_top_with_margin_on(ids.fond, MARGE)
            .set(ids.titre, ui);
        Image::new(image)
            .w_h(cote, cote)
            .mid_top_with_margin_on(ids.fond, MARGE + BANDE)
            .set(ids.image, ui);
        let (texte, couleur) = if ouverte.objectifs == 0 {
            (i18n.get_msg("icebreaker-carte-sans-objectif"), DISCRET)
        } else if ouverte.restants == 0 {
            (i18n.get_msg("icebreaker-carte-objectifs-faits"), DISCRET)
        } else {
            (
                i18n.get_msg_ctx("icebreaker-carte-objectifs", &i18n::fluent_args! {
                    "restants" => ouverte.restants as u64,
                }),
                TEXTE,
            )
        };
        Text::new(&texte)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(15))
            .color(couleur)
            .mid_bottom_with_margin_on(ids.fond, MARGE + LIGNE)
            .set(ids.objectifs, ui);
        Text::new(&i18n.get_msg("icebreaker-carte-aide"))
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(13))
            .color(DISCRET)
            .mid_bottom_with_margin_on(ids.fond, MARGE)
            .set(ids.aide, ui);
    }
}
