//! Icebreaker RO : le dessin de la roue des objets (Tab tenu). La logique (directions, objets, croix
//! rouge) est dans `common::icebreaker_roue` ; la session dit ce qui est ouvert et visé, ce module
//! dessine : un disque au centre de l'écran, huit emplacements sur un cercle, le secteur visé en or,
//! une croix rouge sur ce qui manque dans le sac.

use super::{
    icebreaker::{BORDURE, CASE, DISCRET, FOND, IMAGE_GRISE, MAUVAIS, OR, TEXTE, TRAIT, cadre},
    img_ids::Imgs,
    item_imgs::{ItemImgs, animate_by_pulse},
};
use crate::ui::fonts::Fonts;
use common::{
    comp::ItemKey,
    icebreaker_roue::{Action, Direction8, Etat, Secteur},
};
use conrod_core::{
    Colorable, Positionable, Sizeable, UiCell, Widget, image,
    widget::{self, Image, Text},
    widget_ids,
};
use i18n::Localization;

/// Distance du centre de l'écran à un emplacement.
const RAYON: f64 = 120.0;
/// Côté d'un emplacement.
const CASE_COTE: f64 = 64.0;
/// Demi-longueur d'une branche de la croix rouge.
const CROIX: f64 = 20.0;

/// La roue ouverte, telle que la session la donne au HUD.
#[derive(Clone, Debug)]
pub struct Ouverte {
    pub secteurs: Vec<Secteur>,
    /// Pour chaque secteur : l'image de l'objet qu'il prendra (potion, plat, arme de rechange) et sa
    /// quantité.
    pub objets: Vec<Option<(ItemKey, u32)>>,
    /// La direction visée et la part du seuil parcourue (0 à 1).
    pub vise: Option<(Direction8, f32)>,
}

widget_ids! {
    struct Ids {
        disque,
        anneau,
        centre,
        point,
        nom,
        fonds[],
        images[],
        nombres[],
        croix_a[],
        croix_b[],
        vides[],
    }
}

pub struct Roue {
    ids: Ids,
}

impl Roue {
    pub fn new(generateur: widget::id::Generator) -> Self {
        Self {
            ids: Ids::new(generateur),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn dessiner(
        &mut self,
        ui: &mut UiCell,
        ouverte: &Ouverte,
        imgs: &Imgs,
        item_imgs: &ItemImgs,
        fonts: &Fonts,
        i18n: &Localization,
        pulse: f32,
    ) {
        let ids = &mut self.ids;
        for liste in [
            &mut ids.fonds,
            &mut ids.images,
            &mut ids.nombres,
            &mut ids.croix_a,
            &mut ids.croix_b,
            &mut ids.vides,
        ] {
            if liste.len() < 8 {
                liste.resize(8, &mut ui.widget_id_generator());
            }
        }

        widget::Circle::fill_with(RAYON + CASE_COTE * 0.75, FOND.alpha(0.55))
            .x_y(0.0, 0.0)
            .set(ids.disque, ui);
        widget::Circle::outline_styled(RAYON, widget::primitive::line::Style::solid().thickness(2.0))
            .color(BORDURE)
            .x_y(0.0, 0.0)
            .set(ids.anneau, ui);
        widget::Circle::fill_with(26.0, CASE)
            .x_y(0.0, 0.0)
            .set(ids.centre, ui);
        // Le repère du centre suit la souris jusqu'au bord du disque central.
        let (vx, vy) = ouverte.vise.map_or((0.0, 0.0), |(d, part)| {
            let v = d.vecteur();
            (v.x as f64 * part as f64 * 22.0, -v.y as f64 * part as f64 * 22.0)
        });
        widget::Circle::fill_with(7.0, OR).x_y(vx, vy).set(ids.point, ui);

        let vise = ouverte.vise.filter(|(_, part)| *part > 0.15).map(|(d, _)| d);
        for (i, direction) in Direction8::TOUTES.into_iter().enumerate() {
            let v = direction.vecteur();
            let (x, y) = (v.x as f64 * RAYON, -v.y as f64 * RAYON);
            let Some(k) = ouverte.secteurs.iter().position(|s| s.direction == direction) else {
                widget::Circle::fill_with(5.0, TRAIT)
                    .x_y(x, y)
                    .set(ids.vides[i], ui);
                continue;
            };
            let secteur = &ouverte.secteurs[k];
            let bord = if vise == Some(direction) { OR } else { BORDURE };
            cadre([CASE_COTE, CASE_COTE], CASE, bord, 2.0)
                .x_y(x, y)
                .set(ids.fonds[i], ui);

            let objet = ouverte.objets.get(k).cloned().flatten();
            let image = match &objet {
                Some((cle, _)) => animate_by_pulse(&item_imgs.img_ids_or_not_found_img(cle.clone()), pulse),
                None => icone(imgs, secteur.action),
            };
            let teinte = if secteur.etat == Etat::Pret { TEXTE } else { IMAGE_GRISE };
            Image::new(image)
                .w_h(CASE_COTE - 16.0, CASE_COTE - 16.0)
                .x_y(x, y)
                .color(Some(teinte))
                .set(ids.images[i], ui);
            if let Some((_, nombre)) = objet
                && matches!(secteur.action, Action::Potion | Action::Nourriture)
            {
                Text::new(&nombre.to_string())
                    .font_id(fonts.cyri.conrod_id)
                    .font_size(fonts.cyri.scale(14))
                    .color(TEXTE)
                    .bottom_right_with_margins_on(ids.fonds[i], 3.0, 5.0)
                    .set(ids.nombres[i], ui);
            }
            match secteur.etat {
                Etat::Manque => {
                    for (id, (a, b)) in [
                        (ids.croix_a[i], ([-CROIX, -CROIX], [CROIX, CROIX])),
                        (ids.croix_b[i], ([-CROIX, CROIX], [CROIX, -CROIX])),
                    ] {
                        widget::Line::abs([x + a[0], y + a[1]], [x + b[0], y + b[1]])
                            .thickness(5.0)
                            .color(MAUVAIS)
                            .set(id, ui);
                    }
                },
                Etat::Pret => {},
            }
        }

        // Le nom du secteur visé, sous la roue.
        if let Some(secteur) = vise.and_then(|d| ouverte.secteurs.iter().find(|s| s.direction == d)) {
            Text::new(&i18n.get_msg(cle_nom(secteur.action)))
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(18))
                .color(if secteur.etat == Etat::Pret { OR } else { DISCRET })
                .x_y(0.0, -(RAYON + CASE_COTE))
                .set(ids.nom, ui);
        }
    }
}

/// L'icône d'un secteur sans objet à montrer.
fn icone(imgs: &Imgs, action: Action) -> image::Id {
    match action {
        Action::Carte => imgs.map_icon,
        Action::ChangerArme => imgs.swap_equipped_weapons_btn,
        Action::Potion => imgs.icon_potion,
        Action::Nourriture => imgs.icon_food,
    }
}

fn cle_nom(action: Action) -> &'static str {
    match action {
        Action::Carte => "icebreaker-roue-carte",
        Action::ChangerArme => "icebreaker-roue-arme",
        Action::Potion => "icebreaker-roue-potion",
        Action::Nourriture => "icebreaker-roue-nourriture",
    }
}
