//! Icebreaker : l'interface ajoutée au HUD (jalon N, décision 0019,
//! `docs/icebreaker/PLAN-INTERFACE.md`).
//!
//! - La fenêtre du village : le plan à gauche, le panneau typé du bâtiment
//!   cliqué à droite, dans un seul cadre au centre de l'écran.
//! - Les bandeaux, en haut au centre : l'état de fantôme, l'expédition en
//!   préparation.
//! - Les messages, à droite sous la mini-carte : annonces, refus, infos du
//!   serveur, quand aucun panneau n'est ouvert pour les montrer.
//! - Le panneau libre : le choix de la couleur de la balise.
//!
//! Tout a une taille fixe : rien ne dépend du texte de l'image précédente, les
//! listes défilent. Les textes sont préparés à la réception (une fois), pas à
//! chaque image. Le serveur envoie ce qui change : rien n'est redemandé.
//!
//! Tout se dessine avec des rectangles pleins, du texte et les images déjà dans
//! le jeu (boutons, icônes) : aucune nouvelle texture. Les contours sont des
//! rectangles à bordure (`BorderedRectangle`), pas `Rectangle::outline_styled` :
//! dans la version de conrod de Veloren, les contours tracés s'accumulent d'un
//! rectangle à l'autre et prennent tous la couleur du dernier.

use super::{
    Event,
    img_ids::Imgs,
    item_imgs::{ItemImgs, animate_by_pulse},
};
use crate::{
    game_input::GameInput,
    session::{icebreaker_lock::Avis, icebreaker_vue::Reglages},
    settings::ControlSettings,
    ui::fonts::Fonts,
};
use common::{
    comp::{self, ChatType, Content, ItemKey},
    icebreaker_panneau::{Action, Batiment, Demande, Expedition, Ligne, Panneau, Sorte, Ton},
    interaction::InteractionKind,
    states::{interact::InteractKind, utils::StageSection},
};
use conrod_core::{
    Borderable, Color, Colorable, Labelable, Positionable, Sizeable, UiCell, Widget, color, image,
    position::Relative,
    widget::{self, BorderedRectangle, Button, Image, Rectangle, Scrollbar, Text},
    widget_ids,
};
use i18n::Localization;
use std::time::{Duration, Instant};
use vek::*;

// La charte (`PLAN-INTERFACE.md`) : les couleurs de Veloren, un seul accent à nous, l'or de
// l'anneau de verrouillage.
const FOND: Color = Color::Rgba(0.047, 0.055, 0.063, 1.0);
const FOND_PLAN: Color = Color::Rgba(0.063, 0.078, 0.059, 1.0);
const BANDEAU: Color = Color::Rgba(0.118, 0.141, 0.149, 1.0);
const CASE: Color = Color::Rgba(0.078, 0.094, 0.102, 1.0);
const BORDURE: Color = Color::Rgba(0.357, 0.400, 0.416, 1.0);
const TRAIT: Color = Color::Rgba(0.165, 0.188, 0.200, 1.0);
const TEXTE: Color = Color::Rgba(1.0, 1.0, 1.0, 1.0);
const CLAIR: Color = Color::Rgba(0.788, 0.831, 0.831, 1.0);
const DISCRET: Color = Color::Rgba(0.612, 0.702, 0.702, 1.0);
const GRIS: Color = Color::Rgba(0.557, 0.604, 0.604, 1.0);
const IMAGE_GRISE: Color = Color::Rgba(0.45, 0.45, 0.45, 1.0);
const OR: Color = Color::Rgba(1.0, 0.82, 0.4, 1.0);
const BON: Color = Color::Rgba(0.608, 0.847, 0.290, 1.0);
const MAUVAIS: Color = Color::Rgba(0.949, 0.439, 0.416, 1.0);
const INFO: Color = Color::Rgba(0.486, 0.769, 0.878, 1.0);

/// La couleur d'un chiffre, d'une pastille ou d'un message.
fn teinte(ton: Ton) -> Color {
    match ton {
        Ton::Normal => TEXTE,
        Ton::Or => OR,
        Ton::Bon => BON,
        Ton::Mauvais => MAUVAIS,
        Ton::Info => INFO,
        Ton::Discret => DISCRET,
    }
}

/// La couleur de remplissage d'une jauge.
fn remplissage(ton: Ton) -> Color {
    match ton {
        Ton::Bon => Color::Rgba(0.33, 0.63, 0.0, 1.0),
        Ton::Mauvais => Color::Rgba(0.79, 0.19, 0.17, 1.0),
        Ton::Or => OR,
        _ => Color::Rgba(0.29, 0.62, 0.75, 1.0),
    }
}

/// `part` de `couleur` sur `fond`, sans transparence.
fn melange(couleur: (f32, f32, f32), part: f32, fond: Color) -> Color {
    let fond = fond.to_rgb();
    let vers = |de: f32, a: f32| de + (a - de) * part;
    Color::Rgba(
        vers(fond.0, couleur.0),
        vers(fond.1, couleur.1),
        vers(fond.2, couleur.2),
        1.0,
    )
}

/// Un rectangle plein avec sa bordure.
fn cadre(dim: [f64; 2], fond: Color, bord: Color, epaisseur: f64) -> BorderedRectangle {
    BorderedRectangle::new(dim)
        .color(fond)
        .border(epaisseur)
        .border_color(bord)
}

// La fenêtre, en unités de l'interface (les mêmes que le reste du HUD).
const FEN_W: f64 = 1080.0;
const FEN_H: f64 = 600.0;
/// Du haut de l'écran au haut de la fenêtre : sous les bandeaux.
const FEN_HAUT: f64 = 110.0;
const BANDEAU_H: f64 = 48.0;
const PLAN_W: f64 = 560.0;
const PAN_W: f64 = FEN_W - 4.0 - PLAN_W - 2.0;
const CORPS_H: f64 = FEN_H - 4.0 - BANDEAU_H - 2.0;
// Le plan : une grille de cases, chaque bâtiment dans la case la plus proche de sa place.
const COLONNES: usize = 5;
const RANGS: usize = 6;
const CASE_W: f64 = 104.0;
const CASE_H: f64 = 84.0;
const PAS_X: f64 = CASE_W + 4.0;
const PAS_Y: f64 = CASE_H + 4.0;

// Le panneau : chaque bloc a sa hauteur.
const MARGE: f64 = 16.0;
const LARGEUR: f64 = PAN_W - 2.0 * MARGE;
const EN_TETE_H: f64 = 76.0;
const ONGLET_H: f64 = 36.0;
const CHIFFRE_H: f64 = 76.0;
const JAUGE_H: f64 = 44.0;
const OBJET: f64 = 54.0;
const OBJETS_PAR_RANG: usize = 8;
const LIGNE_H: f64 = 52.0;
const ACTION_H: f64 = 44.0;
/// Deux lignes de texte.
const MESSAGE_H: f64 = 44.0;
const ESPACE: f64 = 10.0;

// Les bandeaux et les messages.
const BANDEAU_W: f64 = 760.0;
const BANDEAU_HAUT: f64 = 44.0;
const ANNONCE_W: f64 = 400.0;
const ANNONCE_H: f64 = 70.0;
/// Du haut de l'écran à la première annonce : sous la mini-carte.
const ANNONCES_HAUT: f64 = 310.0;
const ANNONCES_MAX: usize = 4;

/// Combien de temps la réponse du serveur à une action reste dans la ligne de message.
const DUREE_REPONSE: Duration = Duration::from_secs(6);
/// Combien de temps un bouton cliqué reste « en cours » si rien ne revient.
const DUREE_ATTENTE: Duration = Duration::from_millis(1500);
/// Combien de temps une annonce reste à l'écran.
const DUREE_ANNONCE: Duration = Duration::from_secs(7);

widget_ids! {
    pub struct Ids {
        fond,
        bandeau,
        bandeau_trait,
        titre,
        aide,
        fermer,
        plan,
        plan_cases[],
        plan_icones[],
        plan_noms[],
        plan_etats[],
        plan_joueur,
        plan_vous,
        separation,
        pan,
        pan_vide,
        p_icone,
        p_titre,
        p_desc,
        onglets[],
        chiffres_fond[],
        chiffres_valeur[],
        chiffres_legende[],
        jauge_legende,
        jauge_texte,
        jauge_fond,
        jauge_part,
        objets_fond[],
        objets_image[],
        objets_nombre[],
        liste,
        liste_barre,
        lignes_fond[],
        lignes_icone[],
        lignes_titre[],
        lignes_sous[],
        lignes_pastille[],
        lignes_mot[],
        lignes_cadenas[],
        lignes_bouton[],
        actions_cadre[],
        actions[],
        message,
        libre_fond,
        libre_titre,
        libre_desc,
        libre_cases[],
        libre_couleurs[],
        libre_message,
        fantome_fond,
        fantome_pastille,
        fantome_mot,
        fantome_aide,
        fantome_touche,
        expedition_fond,
        expedition_icone,
        expedition_titre,
        expedition_depart,
        expedition_pastilles[],
        expedition_mots[],
        annonces_fond[],
        annonces_mot[],
        annonces_texte[],
        action_fond,
        action_texte,
        action_jauge_fond,
        action_jauge_part,
        // À terre (bloc de la zone « action en cours »), mort, allié regardé.
        terre_titre,
        terre_reste,
        terre_jauge_fond,
        terre_jauge_part,
        terre_aide,
        terre_abandon,
        mort_fond,
        mort_zone,
        mort_titre,
        mort_nom,
        mort_perdu_fond,
        mort_perdu,
        mort_sac,
        mort_cartes_fond[],
        mort_cartes_touche_fond[],
        mort_cartes_touche[],
        mort_cartes_titre[],
        mort_cartes_sous[],
        regarde_fond,
        regarde_pastille,
        regarde_mot,
        regarde_nom,
        regarde_vie,
        regarde_jauge_fond,
        regarde_jauge_part,
        regarde_aide,
    }
}

// À terre, mort, allié regardé : en unités de l'interface (maquettes `ATerre`, `Mort`,
// `Spectateur`, en 1280 x 720, multipliées par 1,4).
const TERRE_W: f64 = 620.0;
const TERRE_H: f64 = 214.0;
const TERRE_JAUGE_H: f64 = 22.0;
const MORT_W: f64 = 950.0;
const MORT_HAUT: f64 = 170.0;
const MORT_PERDU_H: f64 = 132.0;
const MORT_CARTE_H: f64 = 158.0;
const MORT_ESPACE: f64 = 22.0;
const REGARDE_H: f64 = 88.0;
const REGARDE_JAUGE_W: f64 = 200.0;
/// Le rouge de la bordure et des barres du « à terre » (`remplissage(Ton::Mauvais)`).
const ROUGE_BORD: Color = Color::Rgba(0.79, 0.19, 0.17, 1.0);
/// Le fond des cadres d'action de l'écran de mort.
const FOND_CARTE: Color = Color::Rgba(0.353, 0.165, 0.102, 1.0);
const FOND_TOUCHE: Color = Color::Rgba(0.047, 0.055, 0.063, 1.0);

/// Ce que le HUD montre du joueur à terre ou mort (en campagne seulement).
#[derive(Clone, Debug, PartialEq)]
pub enum Etat {
    Rien,
    /// À terre : secondes qui restent (arrondies au-dessus) et part du délai qui reste (0 à 1).
    ATerre { secondes: u32, part: f32 },
    /// Mort, ne regarde personne : son nom, et le nom de l'allié que le clic gauche regarderait.
    Mort { nom: String, allie: Option<String> },
    /// Mort, regarde un allié : son nom, sa vie (courante, maximale).
    Regarde { nom: String, vie: u32, max: u32 },
}

/// L'état du joueur. `regarde` : le point de vue n'est plus le personnage du joueur. Les noms ne
/// sont cherchés que dans l'état qui les montre.
pub fn etat_joueur(
    registre: bool,
    mort: bool,
    regarde: bool,
    a_terre: Option<(f64, f32)>,
    nom: impl FnOnce() -> String,
    allie: impl FnOnce() -> Option<String>,
    regarde_de: impl FnOnce() -> (String, u32, u32),
) -> Etat {
    if !registre {
        return Etat::Rien;
    }
    if mort {
        return if regarde {
            let (nom, vie, max) = regarde_de();
            Etat::Regarde { nom, vie, max }
        } else {
            Etat::Mort { nom: nom(), allie: allie() }
        };
    }
    match a_terre {
        Some((reste, part)) => Etat::ATerre {
            secondes: reste.max(0.0).ceil() as u32,
            part: part.clamp(0.0, 1.0),
        },
        None => Etat::Rien,
    }
}

/// Le texte gardé de `cle`, refait par `faire` seulement quand `cle` change.
fn garde<'a, K: PartialEq>(
    cache: &'a mut Option<(K, String)>,
    cle: K,
    faire: impl FnOnce() -> String,
) -> &'a str {
    if cache.as_ref().is_none_or(|(ancienne, _)| *ancienne != cle) {
        *cache = Some((cle, faire()));
    }
    cache.as_ref().map_or("", |(_, texte)| texte.as_str())
}

/// Comme `garde`, pour un texte qui dépend d'un nom, sans recopier le nom à chaque image.
fn garde_nom<'a>(
    cache: &'a mut Option<(String, String)>,
    nom: &str,
    faire: impl FnOnce(&str) -> String,
) -> &'a str {
    if cache.as_ref().is_none_or(|(ancien, _)| ancien != nom) {
        *cache = Some((nom.to_string(), faire(nom)));
    }
    cache.as_ref().map_or("", |(_, texte)| texte.as_str())
}

/// Les textes fixes du « à terre », de l'écran de mort et de l'allié regardé, dans la langue du
/// joueur et avec ses touches.
fn textes_vie(i18n: &Localization, controls: &ControlSettings) -> TextesVie {
    let touche = |entree: GameInput| {
        controls
            .get_binding(entree)
            .map_or_else(String::new, |touche| touche.display_string())
    };
    let retour = touche(GameInput::Respawn);
    let dire = |cle: &str| i18n.get_msg(cle).into_owned();
    TextesVie {
        terre: dire("icebreaker-a-terre"),
        terre_aide: dire("icebreaker-a-terre-aide"),
        abandon: i18n
            .get_msg_ctx("icebreaker-a-terre-abandon", &i18n::fluent_args! {
                "key" => touche(GameInput::GiveUp),
            })
            .into_owned(),
        mot_mort: dire("icebreaker-mort-mot"),
        mort_titre: dire("icebreaker-mort-titre"),
        perdu: dire("icebreaker-mort-perdu"),
        sac: dire("icebreaker-mort-sac"),
        retour: dire("icebreaker-mort-retour"),
        retour_sous: dire("icebreaker-mort-retour-sous"),
        clic: dire("icebreaker-mort-clic"),
        regarder: dire("icebreaker-mort-regarder-allie"),
        aucun: dire("icebreaker-mort-aucun"),
        aide_regarde: i18n
            .get_msg_ctx("icebreaker-regarde-aide", &i18n::fluent_args! {
                "key" => retour.clone(),
            })
            .into_owned(),
        touche_retour: retour,
    }
}

/// Les textes fixes du « à terre », de l'écran de mort et de l'allié regardé, préparés une fois.
struct TextesVie {
    terre: String,
    terre_aide: String,
    abandon: String,
    mot_mort: String,
    mort_titre: String,
    perdu: String,
    sac: String,
    touche_retour: String,
    retour: String,
    retour_sous: String,
    clic: String,
    regarder: String,
    aucun: String,
    aide_regarde: String,
}

/// Les textes qui changent avec l'état, gardés tant que leur donnée ne change pas.
#[derive(Default)]
struct TextesVivants {
    reste: Option<(u32, String)>,
    allie: Option<(String, String)>,
    regarde: Option<(String, String)>,
    vie: Option<((u32, u32), String)>,
}

// La zone « action en cours », en bas au centre, au-dessus de la barre de raccourcis.
const ACTION_BAS: f64 = 180.0;
const ACTION_W: f64 = 540.0;
const ACTION_BANDEAU_H: f64 = 38.0;
const ACTION_JAUGE_W: f64 = 300.0;
/// Hauteur du panneau libre (balise), qui occupe le même bas d'écran.
const LIBRE_H: f64 = 150.0;

/// Ce que le joueur est en train de faire, pour la zone « action en cours ».
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnCours {
    /// Le verrouillage de cible : rappel des touches, ou « Cible perdue ».
    pub verrou: Avis,
    /// La part de la fouille d'un sac de mort faite (0 à 1), si le joueur fouille.
    pub fouille: Option<f32>,
}

/// Ce que la zone montre : une seule chose à la fois, la fouille d'abord.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Zone {
    Fouille(f32),
    Perdue,
    Rappel,
}

impl EnCours {
    fn zone(&self) -> Option<Zone> {
        if let Some(part) = self.fouille {
            return Some(Zone::Fouille(part.clamp(0.0, 1.0)));
        }
        match self.verrou {
            Avis::Perdue => Some(Zone::Perdue),
            Avis::Rappel => Some(Zone::Rappel),
            Avis::Rien => None,
        }
    }
}

/// La part de fouille faite, `timer` écoulé sur `duree`.
pub fn part_fouille(timer: std::time::Duration, duree: std::time::Duration) -> f32 {
    if duree.is_zero() {
        1.0
    } else {
        (timer.as_secs_f32() / duree.as_secs_f32()).clamp(0.0, 1.0)
    }
}

/// La part de fouille faite, si l'état du personnage est la fouille d'un sac de mort.
pub fn fouille_de(etat: &comp::CharacterState) -> Option<f32> {
    let comp::CharacterState::Interact(interaction) = etat else {
        return None;
    };
    let donnees = &interaction.static_data;
    if !matches!(donnees.interact, InteractKind::Entity {
        kind: InteractionKind::IcebreakerFouillerSac,
        ..
    }) {
        return None;
    }
    Some(match (interaction.stage_section, donnees.use_duration) {
        (StageSection::Buildup, _) => 0.0,
        (StageSection::Action, Some(duree)) => part_fouille(interaction.timer, duree),
        _ => 1.0,
    })
}

/// L'icône du HUD appelée `nom` dans les données (`vue_village.ron`, `classes.ron`).
pub fn icone(imgs: &Imgs, nom: &str) -> Option<image::Id> {
    Some(match nom {
        "utilitaire" => imgs.icon_utility,
        "potion" => imgs.icon_potion,
        "vivres" => imgs.icon_food,
        "globe" => imgs.icon_globe,
        "outils" => imgs.icon_tools,
        "armes" => imgs.icon_weapon,
        "armure" => imgs.icon_armor,
        "buche" => imgs.icon_log,
        "lingot" => imgs.icon_ingot,
        "planeur" => imgs.icon_glider,
        // Les classes (`classes.ron`, champ `icone`).
        "epees" => imgs.swords_crossed,
        "epee" => imgs.sword,
        "baton" => imgs.staff,
        "arc" => imgs.bow,
        "hache" => imgs.axe,
        "marteau" => imgs.hammer,
        "sceptre" => imgs.sceptre,
        _ => return None,
    })
}

/// Du monde à la grille du plan : l'étendue des centres des bâtiments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Repere {
    min: Vec2<f32>,
    etendue: Vec2<f32>,
}

impl Repere {
    pub fn de(centres: &[Vec2<f32>]) -> Option<Self> {
        let premier = *centres.first()?;
        let (min, max) = centres.iter().fold((premier, premier), |(min, max), c| {
            (Vec2::partial_min(min, *c), Vec2::partial_max(max, *c))
        });
        Some(Self {
            min,
            etendue: (max - min).map(|d| d.max(1.0)),
        })
    }

    /// La place d'un point du monde sur une grille de `colonnes` x `rangs`
    /// cases, en cases : (0, 0) est la case en haut à gauche, l'ouest est à
    /// gauche et le nord en haut.
    pub fn place(&self, p: Vec2<f32>, colonnes: usize, rangs: usize) -> Vec2<f32> {
        let part = (p - self.min) / self.etendue;
        Vec2::new(
            part.x * colonnes.saturating_sub(1) as f32,
            (1.0 - part.y) * rangs.saturating_sub(1) as f32,
        )
    }
}

/// Range les bâtiments sur une grille de `colonnes` x `rangs` cases : chacun,
/// dans l'ordre donné, prend la case libre la plus proche de sa place réelle.
/// Deux bâtiments voisins ne se recouvrent donc jamais, et le plan garde la
/// forme du village. Rend (colonne, rang) pour chacun ; au-delà du nombre de
/// cases, les derniers partagent la dernière case.
pub fn ranger(centres: &[Vec2<f32>], colonnes: usize, rangs: usize) -> Vec<(usize, usize)> {
    let Some(repere) = Repere::de(centres) else {
        return Vec::new();
    };
    let cases = (colonnes * rangs).max(1);
    let mut prises = vec![false; cases];
    centres
        .iter()
        .map(|centre| {
            let ideale = repere.place(*centre, colonnes, rangs);
            let ecart = |i: usize| {
                Vec2::new((i % colonnes) as f32, (i / colonnes) as f32).distance_squared(ideale)
            };
            let libre = (0..cases)
                .filter(|i| !prises[*i])
                .min_by(|a, b| ecart(*a).total_cmp(&ecart(*b)));
            let i = libre.unwrap_or(cases - 1);
            prises[i] = true;
            (i % colonnes, i / colonnes)
        })
        .collect()
}

/// Le centre d'une place de la grille (en cases, même fractionnaire), depuis le
/// centre du plan (y vers le haut).
fn centre_case(place: Vec2<f32>) -> Vec2<f64> {
    let largeur = COLONNES as f64 * PAS_X - 4.0;
    let hauteur = RANGS as f64 * PAS_Y - 4.0;
    Vec2::new(
        -largeur / 2.0 + CASE_W / 2.0 + f64::from(place.x) * PAS_X,
        hauteur / 2.0 - CASE_H / 2.0 - f64::from(place.y) * PAS_Y,
    )
}

/// Où chaque bloc du panneau commence, depuis le haut du corps de la fenêtre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Places {
    pub onglets: Option<f64>,
    pub chiffres: Option<f64>,
    pub jauge: Option<f64>,
    pub objets: Option<f64>,
    /// La liste : son haut et sa hauteur (ce qui reste).
    pub liste: Option<(f64, f64)>,
    pub actions: Option<f64>,
    pub message: f64,
}

/// La mise en page d'un panneau : chaque bloc présent a sa hauteur fixe, du
/// haut vers le bas ; les actions et le message sont collés en bas ; la liste
/// prend ce qui reste, et défile.
pub fn places(
    onglets: bool,
    chiffres: bool,
    jauge: bool,
    rangs_objets: usize,
    liste: bool,
    rangs_actions: usize,
) -> Places {
    let mut y = EN_TETE_H;
    let mut bloc = |present: bool, hauteur: f64| {
        present.then(|| {
            let haut = y;
            y += hauteur + ESPACE;
            haut
        })
    };
    let onglets = bloc(onglets, ONGLET_H);
    let chiffres = bloc(chiffres, CHIFFRE_H);
    let jauge = bloc(jauge, JAUGE_H);
    let objets = bloc(
        rangs_objets > 0,
        rangs_objets as f64 * (OBJET + 8.0) - 8.0,
    );
    let message = CORPS_H - MARGE - MESSAGE_H;
    let actions = (rangs_actions > 0)
        .then(|| message - ESPACE - rangs_actions as f64 * (ACTION_H + 8.0) + 8.0);
    let bas = actions.unwrap_or(message) - ESPACE;
    Places {
        onglets,
        chiffres,
        jauge,
        objets,
        liste: liste.then(|| (y, (bas - y).max(LIGNE_H))),
        actions,
        message,
    }
}

/// Un texte au plus long de `max` caractères, coupé d'un « … ».
pub fn couper(texte: &str, max: usize) -> String {
    if texte.chars().count() <= max {
        texte.to_string()
    } else {
        let mut court: String = texte.chars().take(max.saturating_sub(1)).collect();
        court.push('…');
        court
    }
}

/// Un bouton, textes prêts.
struct Bouton {
    libelle: String,
    commande: Vec<String>,
    principale: bool,
    permise: bool,
    couleur: Option<Rgb<u8>>,
}

struct Rangee {
    icone: String,
    titre: String,
    sous_titre: String,
    pastille: Option<(String, Ton)>,
    cadenas: Option<(bool, Vec<String>)>,
    action: Option<Bouton>,
    choisie: bool,
    estompee: bool,
}

/// Un panneau prêt à être dessiné : les textes dans la langue du joueur, une
/// fois pour toutes à la réception.
struct Pret {
    icone: String,
    titre: String,
    description: String,
    onglets: Vec<(String, Vec<Rangee>)>,
    chiffres: Vec<(String, String, Ton)>,
    jauge: Option<(String, String, f32, Ton)>,
    objets: Vec<(ItemKey, String)>,
    lignes: Vec<Rangee>,
    actions: Vec<Bouton>,
    colonnes: usize,
    message: Option<(String, Sorte)>,
    libre: bool,
}

impl Pret {
    fn de(panneau: Panneau, i18n: &Localization) -> Self {
        let dire = |contenu: &Content| i18n.get_content(contenu);
        let bouton = |action: Action| Bouton {
            libelle: dire(&action.libelle),
            commande: action.commande,
            principale: action.principale,
            permise: action.permise,
            couleur: action.couleur,
        };
        let rangee = |ligne: Ligne| Rangee {
            icone: ligne.icone,
            titre: dire(&ligne.titre),
            sous_titre: dire(&ligne.sous_titre),
            pastille: ligne.pastille.map(|(mot, ton)| (dire(&mot), ton)),
            cadenas: ligne.cadenas,
            action: ligne.action.map(bouton),
            choisie: ligne.choisie,
            estompee: ligne.estompee,
        };
        Self {
            icone: panneau.icone,
            titre: dire(&panneau.titre),
            description: dire(&panneau.description),
            onglets: panneau
                .onglets
                .into_iter()
                .map(|onglet| {
                    (
                        dire(&onglet.libelle),
                        onglet.lignes.into_iter().map(rangee).collect(),
                    )
                })
                .collect(),
            chiffres: panneau
                .chiffres
                .into_iter()
                .map(|c| (dire(&c.valeur), dire(&c.legende), c.ton))
                .collect(),
            jauge: panneau
                .jauge
                .map(|j| (dire(&j.legende), dire(&j.texte), j.part.clamp(0.0, 1.0), j.ton)),
            objets: panneau
                .objets
                .into_iter()
                .map(|(cle, n)| (cle, n.to_string()))
                .collect(),
            lignes: panneau.lignes.into_iter().map(rangee).collect(),
            actions: panneau.actions.into_iter().map(bouton).collect(),
            colonnes: usize::from(panneau.colonnes.max(1)),
            message: panneau.message.map(|(texte, sorte)| (dire(&texte), sorte)),
            libre: panneau.libre,
        }
    }
}

/// Une case du plan : un bâtiment, textes prêts, rangé.
struct Case {
    nom: String,
    etat: String,
    sorte: String,
    emprise: Aabr<i32>,
    couleur: (f32, f32, f32),
    icone: String,
    /// Depuis le centre du plan (y vers le haut).
    centre: Vec2<f64>,
}

/// L'expédition en préparation au village, textes prêts.
struct Depart {
    titre: String,
    /// Chaque inscrit : ce qui s'écrit dans sa pastille, et s'il est prêt.
    membres: Vec<(String, bool)>,
    /// Quand elle part, une fois tout le monde prêt : le décompte se fait ici.
    depart: Option<Instant>,
    /// Les secondes affichées en dernier, et leur texte.
    decompte: Option<(u64, String)>,
}

/// La sorte d'une annonce : son mot et sa couleur.
#[derive(Clone, Copy)]
enum Genre {
    Village,
    Refus,
    Info,
}

/// Un message du serveur montré à droite, textes prêts.
struct Annonce {
    genre: Genre,
    texte: String,
    /// Un mot trop long pour une ligne (un chemin de fichier) : couper dans les mots.
    mot_long: bool,
    quand: Instant,
}

/// Ce que la touche Échap a fait.
pub enum Echap {
    /// Rien d'ouvert : la touche est à quelqu'un d'autre.
    Rien,
    /// Un panneau ou le plan s'est fermé ; ce qu'il faut dire au serveur.
    Ferme(Option<Demande>),
}

pub struct Interface {
    ids: Ids,
    reglages: Reglages,
    /// Le plan du village est ouvert.
    ouvert: bool,
    cases: Vec<Case>,
    repere: Option<Repere>,
    /// Le joueur, dans le monde (rien pour un fantôme).
    joueur: Option<Vec2<f32>>,
    /// Le bâtiment cliqué, par son emprise : son panneau est attendu ou montré.
    choisi: Option<Aabr<i32>>,
    panneau: Option<Pret>,
    onglet: usize,
    /// Le bouton cliqué (sa commande), en attendant la suite.
    attente: Option<(Vec<String>, Instant)>,
    /// La dernière réponse du serveur à une action, à préparer.
    reponse_recue: Option<(Content, Sorte)>,
    reponse: Option<(String, Sorte, Instant)>,
    /// Les textes fixes de la fenêtre (titre, aide, « Vous », panneau vide), préparés à l'ouverture.
    textes: Option<[String; 4]>,
    expedition: Option<Depart>,
    /// Les textes du bandeau du fantôme (mot, aide, touche), préparés une fois.
    textes_fantome: Option<[String; 3]>,
    /// Les messages du serveur à montrer à droite : reçus, puis prêts.
    annonces_recues: Vec<(Content, Genre)>,
    annonces: Vec<Annonce>,
    /// Les mots des annonces (village, refus, info), préparés une fois.
    mots: Option<[String; 3]>,
    /// Les textes de la zone « action en cours » (rappel des touches, cible perdue, fouille),
    /// préparés une fois.
    textes_action: Option<[String; 3]>,
    /// Les textes de l'état à terre, mort et allié regardé.
    textes_vie: Option<TextesVie>,
    vivants: TextesVivants,
}

impl Interface {
    pub fn new(generateur: widget::id::Generator) -> Self {
        Self {
            ids: Ids::new(generateur),
            reglages: Reglages::charger(),
            ouvert: false,
            cases: Vec::new(),
            repere: None,
            joueur: None,
            choisi: None,
            panneau: None,
            onglet: 0,
            attente: None,
            reponse_recue: None,
            reponse: None,
            textes: None,
            expedition: None,
            textes_fantome: None,
            annonces_recues: Vec::new(),
            annonces: Vec::new(),
            mots: None,
            textes_action: None,
            textes_vie: None,
            vivants: TextesVivants::default(),
        }
    }

    /// L'expédition en préparation au village (ou aucune) : le serveur l'envoie
    /// quand elle change ; le décompte avant le départ se fait ici.
    pub fn expedition(&mut self, expedition: Option<Expedition>, i18n: &Localization) {
        self.expedition = expedition.map(|expedition| {
            let pret = i18n.get_msg("icebreaker-expedition-pret");
            Depart {
                titre: i18n
                    .get_msg_ctx("icebreaker-expedition-vers", &i18n::fluent_args! {
                        "lieu" => expedition.destination,
                    })
                    .into_owned(),
                membres: expedition
                    .membres
                    .into_iter()
                    .map(|(alias, est_pret)| {
                        if est_pret {
                            (format!("{alias}, {pret}"), true)
                        } else {
                            (alias, false)
                        }
                    })
                    .collect(),
                depart: expedition
                    .depart_dans
                    .map(|secondes| Instant::now() + Duration::from_secs_f32(secondes.max(0.0))),
                decompte: None,
            }
        });
    }

    /// Le plan du village s'ouvre ou se ferme : tout repart de rien.
    pub fn ouvrir(&mut self, ouvert: bool) {
        self.ouvert = ouvert;
        self.cases.clear();
        self.repere = None;
        self.choisi = None;
        self.panneau = None;
        self.attente = None;
        self.reponse = None;
        self.reponse_recue = None;
        self.textes = None;
    }

    /// Le curseur est libre : le plan est ouvert, ou un panneau libre (balise).
    pub fn curseur_libre(&self) -> bool {
        self.ouvert || self.panneau.as_ref().is_some_and(|p| p.libre)
    }

    /// Les bâtiments envoyés par le serveur : leurs textes et leurs cases sont
    /// préparés ici, pas à chaque image.
    pub fn batiments(&mut self, batiments: Vec<Batiment>, i18n: &Localization) {
        let centres: Vec<Vec2<f32>> = batiments
            .iter()
            .map(|b| (b.emprise.min + b.emprise.max).as_::<f32>() / 2.0)
            .collect();
        self.repere = Repere::de(&centres);
        let places = ranger(&centres, COLONNES, RANGS);
        self.cases = batiments
            .into_iter()
            .zip(places)
            .map(|(batiment, (colonne, rang))| {
                let apparence = self.reglages.apparence(&batiment.sorte);
                Case {
                    nom: i18n.get_content(&batiment.nom),
                    etat: i18n.get_content(&batiment.etat),
                    sorte: batiment.sorte,
                    emprise: batiment.emprise,
                    couleur: apparence.couleur,
                    icone: apparence.icone,
                    centre: centre_case(Vec2::new(colonne as f32, rang as f32)),
                }
            })
            .collect();
    }

    /// Un panneau envoyé par le serveur : celui du bâtiment cliqué, ou un
    /// panneau libre (balise). Un panneau arrivé après la fermeture est ignoré.
    pub fn panneau(&mut self, panneau: Panneau, i18n: &Localization) {
        if !panneau.libre && !(self.ouvert && self.choisi.is_some()) {
            return;
        }
        // À l'ouverture, l'onglet que le serveur propose ; ensuite, celui que le joueur a choisi.
        if self.panneau.is_none() {
            self.onglet = usize::from(panneau.onglet);
        }
        self.panneau = Some(Pret::de(panneau, i18n));
        // Le serveur a répondu : le bouton n'est plus « en cours ».
        self.attente = None;
    }

    pub fn joueur(&mut self, position: Option<Vec2<f32>>) { self.joueur = position; }

    /// Choisit le bâtiment `emprise` : son panneau est attendu ; ce qu'il faut
    /// dire au serveur.
    fn choisir(&mut self, emprise: Aabr<i32>) -> Demande {
        // Un autre bâtiment : son panneau remplace l'ancien quand il arrive.
        if self.choisi != Some(emprise) {
            self.panneau = None;
            self.reponse = None;
        }
        self.choisi = Some(emprise);
        Demande::Batiment((emprise.min + emprise.max).as_::<f32>() / 2.0)
    }

    /// Comme un clic sur le bâtiment de cette sorte (essai sans les mains).
    pub fn cliquer(&mut self, sorte: &str) -> Option<Demande> {
        let emprise = self.cases.iter().find(|case| case.sorte == sorte)?.emprise;
        Some(self.choisir(emprise))
    }

    /// Un message du serveur : réponse à une action (refus, confirmation) ou
    /// annonce. Avec un panneau ouvert, il s'affiche dans sa ligne de message,
    /// là où le joueur a cliqué ; sinon à droite de l'écran, quelques secondes.
    /// Le chat le garde dans tous les cas. Hors de la campagne : rien.
    pub fn message_du_serveur(&mut self, message: &comp::ChatMsg) {
        if !crate::icebreaker_mort::registre() {
            return;
        }
        let (sorte, genre) = match message.chat_type {
            ChatType::CommandError => (Sorte::Refus, Genre::Refus),
            ChatType::CommandInfo => (Sorte::Reussite, Genre::Info),
            ChatType::Meta => (Sorte::Reussite, Genre::Village),
            _ => return,
        };
        if self.ouvert && self.panneau.is_some() {
            // Une annonce à tout le village ne chasse pas la réponse faite au joueur lui-même
            // (« Tu es inscrit pour… » avant « X prépare une expédition… »).
            let reponse_fraiche = self.reponse_recue.is_some()
                || self
                    .reponse
                    .as_ref()
                    .is_some_and(|(_, _, quand)| quand.elapsed() < Duration::from_secs(2));
            if !(matches!(genre, Genre::Village) && reponse_fraiche) {
                self.reponse_recue = Some((message.content().clone(), sorte));
            }
        } else {
            self.annonces_recues.push((message.content().clone(), genre));
        }
    }

    /// Échap ferme d'abord le panneau, puis le plan.
    pub fn echap(&mut self) -> Echap {
        match &self.panneau {
            Some(panneau) => {
                let libre = panneau.libre;
                self.fermer_panneau();
                Echap::Ferme((!libre).then_some(Demande::FermerPanneau))
            },
            None if self.ouvert => Echap::Ferme(Some(Demande::Fermer)),
            None => Echap::Rien,
        }
    }

    fn fermer_panneau(&mut self) {
        self.panneau = None;
        self.choisi = None;
        self.attente = None;
        self.reponse = None;
    }

    /// Dessine ce qui est ouvert ; ce que le joueur a fait. `fantome` : le
    /// joueur est en fantôme dans la campagne (son bandeau le lui dit).
    pub fn dessiner(
        &mut self,
        ui: &mut UiCell,
        imgs: &Imgs,
        item_imgs: &ItemImgs,
        fonts: &Fonts,
        i18n: &Localization,
        controls: &ControlSettings,
        pulse: f32,
        fantome: bool,
        en_cours: EnCours,
        etat: &Etat,
    ) -> Vec<Event> {
        let mut evenements = Vec::new();
        // Les textes du « à terre », de la mort et de l'allié regardé, préparés une fois.
        if !matches!(etat, Etat::Rien) && self.textes_vie.is_none() {
            self.textes_vie = Some(textes_vie(i18n, controls));
        }
        if let Some((contenu, sorte)) = self.reponse_recue.take() {
            self.reponse = Some((couper(&i18n.get_content(&contenu), 110), sorte, Instant::now()));
        }
        if self
            .reponse
            .as_ref()
            .is_some_and(|(_, _, quand)| quand.elapsed() > DUREE_REPONSE)
        {
            self.reponse = None;
        }
        if self
            .attente
            .as_ref()
            .is_some_and(|(_, quand)| quand.elapsed() > DUREE_ATTENTE)
        {
            self.attente = None;
        }

        // Les bandeaux, en haut au centre : le fantôme, puis l'expédition.
        let mut haut = 10.0;
        if fantome {
            self.bandeau_fantome(ui, fonts, i18n, controls, haut);
            haut += BANDEAU_HAUT + 6.0;
        }
        if let Etat::Regarde { nom, vie, max } = etat {
            self.bandeau_regarde(ui, fonts, i18n, nom, *vie, *max, haut);
            haut += REGARDE_H + 6.0;
        }
        self.bandeau_expedition(ui, imgs, fonts, i18n, haut);
        self.messages(ui, fonts, i18n);
        match etat {
            Etat::ATerre { secondes, part } => self.bloc_a_terre(ui, fonts, i18n, *secondes, *part),
            Etat::Mort { nom, allie } => self.ecran_mort(ui, fonts, i18n, nom, allie.as_deref()),
            _ => self.action_en_cours(ui, fonts, i18n, controls, en_cours),
        }

        if self.ouvert {
            self.fenetre(ui, imgs, item_imgs, fonts, i18n, controls, pulse, &mut evenements);
        } else if self.panneau.as_ref().is_some_and(|p| p.libre) {
            self.libre(ui, fonts, &mut evenements);
        }
        evenements
    }

    /// Le bandeau du fantôme : ce qu'il est, et quoi faire.
    fn bandeau_fantome(
        &mut self,
        ui: &mut UiCell,
        fonts: &Fonts,
        i18n: &Localization,
        controls: &ControlSettings,
        haut: f64,
    ) {
        let [mot, aide, touche] = self.textes_fantome.get_or_insert_with(|| {
            let touche = controls
                .get_binding(GameInput::IcebreakerTopView)
                .map_or_else(String::new, |touche| touche.display_string());
            [
                i18n.get_msg("icebreaker-fantome").into_owned(),
                i18n.get_msg("icebreaker-fantome-aide").into_owned(),
                i18n.get_msg_ctx("icebreaker-fantome-plan", &i18n::fluent_args! { "key" => touche })
                    .into_owned(),
            ]
        });
        let fond = self.ids.fantome_fond;
        cadre([BANDEAU_W, BANDEAU_HAUT], FOND, BORDURE, 2.0)
            .mid_top_with_margin_on(ui.window, haut)
            .set(fond, ui);
        cadre([104.0, 28.0], FOND, CLAIR, 2.0)
            .mid_left_with_margin_on(fond, 10.0)
            .graphics_for(fond)
            .set(self.ids.fantome_pastille, ui);
        Text::new(mot)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(18))
            .color(CLAIR)
            .middle_of(self.ids.fantome_pastille)
            .graphics_for(fond)
            .set(self.ids.fantome_mot, ui);
        Text::new(aide)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(20))
            .color(TEXTE)
            .mid_left_with_margin_on(fond, 126.0)
            .graphics_for(fond)
            .set(self.ids.fantome_aide, ui);
        Text::new(touche)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(18))
            .color(DISCRET)
            .mid_right_with_margin_on(fond, 12.0)
            .graphics_for(fond)
            .set(self.ids.fantome_touche, ui);
    }

    /// Le bandeau de l'expédition en préparation, en haut au centre, à `haut`
    /// du bord de l'écran : sa destination, ses inscrits (prêts ou non), et le
    /// décompte une fois tout le monde prêt.
    fn bandeau_expedition(
        &mut self,
        ui: &mut UiCell,
        imgs: &Imgs,
        fonts: &Fonts,
        i18n: &Localization,
        haut: f64,
    ) {
        let Some(depart) = &mut self.expedition else {
            return;
        };
        const PASTILLE: f64 = 132.0;
        let fond = self.ids.expedition_fond;
        cadre([BANDEAU_W, BANDEAU_HAUT], FOND, BORDURE, 2.0)
            .mid_top_with_margin_on(ui.window, haut)
            .set(fond, ui);
        Image::new(imgs.icon_globe)
            .w_h(28.0, 28.0)
            .mid_left_with_margin_on(fond, 10.0)
            .graphics_for(fond)
            .set(self.ids.expedition_icone, ui);
        Text::new(&depart.titre)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(20))
            .color(TEXTE)
            .mid_left_with_margin_on(fond, 48.0)
            .graphics_for(fond)
            .set(self.ids.expedition_titre, ui);
        // À droite : le décompte, puis une pastille par inscrit.
        let mut droite = 8.0;
        let reste = depart.depart.map(|quand| {
            quand
                .saturating_duration_since(Instant::now())
                .as_secs_f32()
                .ceil() as u64
        });
        if let Some(secondes) = reste {
            if depart.decompte.as_ref().map(|(n, _)| *n) != Some(secondes) {
                depart.decompte = Some((
                    secondes,
                    i18n.get_msg_ctx("icebreaker-expedition-depart", &i18n::fluent_args! {
                        "secondes" => secondes,
                    })
                    .into_owned(),
                ));
            }
            if let Some((_, texte)) = &depart.decompte {
                Text::new(texte)
                    .font_id(fonts.cyri.conrod_id)
                    .font_size(fonts.cyri.scale(20))
                    .color(OR)
                    .mid_right_with_margin_on(fond, 12.0)
                    .graphics_for(fond)
                    .set(self.ids.expedition_depart, ui);
            }
            droite += 150.0;
        }
        let n = depart.membres.len().min(3);
        assez(&mut self.ids.expedition_pastilles, n, ui);
        assez(&mut self.ids.expedition_mots, n, ui);
        for (i, (mot, pret)) in depart.membres.iter().take(n).enumerate() {
            let couleur = if *pret { BON } else { DISCRET };
            cadre([PASTILLE, 28.0], FOND, couleur, 2.0)
                .mid_right_with_margin_on(fond, droite + i as f64 * (PASTILLE + 6.0))
                .graphics_for(fond)
                .set(self.ids.expedition_pastilles[i], ui);
            Text::new(mot)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(16))
                .color(couleur)
                .middle_of(self.ids.expedition_pastilles[i])
                .graphics_for(fond)
                .set(self.ids.expedition_mots[i], ui);
        }
    }

    /// Les messages du serveur, à droite sous la mini-carte : chacun porte son
    /// mot (village, refus, info) et s'efface seul.
    fn messages(&mut self, ui: &mut UiCell, fonts: &Fonts, i18n: &Localization) {
        for (contenu, genre) in self.annonces_recues.drain(..) {
            // Deux lignes au plus.
            let texte = couper(&i18n.get_content(&contenu), 78);
            self.annonces.push(Annonce {
                genre,
                mot_long: texte.split_whitespace().any(|mot| mot.chars().count() > 34),
                texte,
                quand: Instant::now(),
            });
        }
        self.annonces
            .retain(|annonce| annonce.quand.elapsed() < DUREE_ANNONCE);
        let trop = self.annonces.len().saturating_sub(ANNONCES_MAX);
        self.annonces.drain(..trop);
        if self.annonces.is_empty() {
            return;
        }
        let mots = self.mots.get_or_insert_with(|| {
            [
                i18n.get_msg("icebreaker-message-village").into_owned(),
                i18n.get_msg("icebreaker-message-refus").into_owned(),
                i18n.get_msg("icebreaker-message-info").into_owned(),
            ]
        });
        let n = self.annonces.len();
        for liste in [
            &mut self.ids.annonces_fond,
            &mut self.ids.annonces_mot,
            &mut self.ids.annonces_texte,
        ] {
            assez(liste, n, ui);
        }
        for (i, annonce) in self.annonces.iter().enumerate() {
            let (mot, couleur) = match annonce.genre {
                Genre::Village => (&mots[0], DISCRET),
                Genre::Refus => (&mots[1], MAUVAIS),
                Genre::Info => (&mots[2], INFO),
            };
            let fond = self.ids.annonces_fond[i];
            cadre([ANNONCE_W, ANNONCE_H], FOND, couleur, 2.0)
                .top_right_with_margins_on(ui.window, ANNONCES_HAUT + i as f64 * (ANNONCE_H + 8.0), 10.0)
                .set(fond, ui);
            Text::new(mot)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(16))
                .color(couleur)
                .top_left_with_margins_on(fond, 5.0, 10.0)
                .graphics_for(fond)
                .set(self.ids.annonces_mot[i], ui);
            let texte = Text::new(&annonce.texte)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(18))
                .color(TEXTE)
                .w(ANNONCE_W - 20.0);
            let texte = if annonce.mot_long {
                texte.wrap_by_character()
            } else {
                texte.wrap_by_word()
            };
            texte
                .top_left_with_margins_on(fond, 24.0, 10.0)
                .graphics_for(fond)
                .set(self.ids.annonces_texte[i], ui);
        }
    }

    /// La zone « action en cours », en bas au centre au-dessus de la barre de raccourcis : la
    /// fouille d'un sac (libellé et jauge), « Cible perdue », ou le rappel des touches du
    /// verrouillage. Une seule chose à la fois.
    fn action_en_cours(
        &mut self,
        ui: &mut UiCell,
        fonts: &Fonts,
        i18n: &Localization,
        controls: &ControlSettings,
        en_cours: EnCours,
    ) {
        let Some(zone) = en_cours.zone() else {
            return;
        };
        let [rappel, perdue, fouille] = self.textes_action.get_or_insert_with(|| {
            let touche = controls
                .get_binding(GameInput::IcebreakerLockTarget)
                .map_or_else(String::new, |touche| touche.display_string());
            [
                i18n.get_msg_ctx("icebreaker-verrou-rappel", &i18n::fluent_args! {
                    "key" => touche,
                })
                .into_owned(),
                i18n.get_msg("icebreaker-verrou-perdue").into_owned(),
                i18n.get_msg("icebreaker-fouille-sac").into_owned(),
            ]
        });
        // Le panneau libre de la balise occupe le même bas d'écran : la zone passe au-dessus.
        let bas = if self.panneau.as_ref().is_some_and(|p| p.libre) {
            ACTION_BAS + LIBRE_H + 10.0
        } else {
            ACTION_BAS
        };
        let fond = self.ids.action_fond;
        let (dimensions, bord, texte, couleur) = match zone {
            Zone::Fouille(_) => ([ACTION_JAUGE_W + 40.0, 64.0], BORDURE, &*fouille, TEXTE),
            Zone::Perdue => ([220.0, ACTION_BANDEAU_H], MAUVAIS, &*perdue, MAUVAIS),
            Zone::Rappel => ([ACTION_W, ACTION_BANDEAU_H], BORDURE, &*rappel, CLAIR),
        };
        cadre(dimensions, FOND, bord, 2.0)
            .mid_bottom_with_margin_on(ui.window, bas)
            .set(fond, ui);
        let texte = Text::new(texte)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(if matches!(zone, Zone::Fouille(_)) { 20 } else { 18 }))
            .color(couleur)
            .graphics_for(fond);
        if let Zone::Fouille(part) = zone {
            texte
                .mid_top_with_margin_on(fond, 8.0)
                .set(self.ids.action_texte, ui);
            cadre([ACTION_JAUGE_W, 16.0], CASE, BORDURE, 2.0)
                .mid_bottom_with_margin_on(fond, 10.0)
                .graphics_for(fond)
                .set(self.ids.action_jauge_fond, ui);
            let rempli = (ACTION_JAUGE_W - 4.0) * f64::from(part);
            if rempli >= 1.0 {
                Rectangle::fill_with([rempli, 12.0], remplissage(Ton::Info))
                    .top_left_with_margins_on(self.ids.action_jauge_fond, 2.0, 2.0)
                    .graphics_for(fond)
                    .set(self.ids.action_jauge_part, ui);
            }
        } else {
            texte.middle_of(fond).set(self.ids.action_texte, ui);
        }
    }

    /// Le bas d'écran de la zone « action en cours » : au-dessus du panneau libre de la balise.
    fn bas_action(&self) -> f64 {
        if self.panneau.as_ref().is_some_and(|p| p.libre) {
            ACTION_BAS + LIBRE_H + 10.0
        } else {
            ACTION_BAS
        }
    }

    /// À terre : un seul bloc, qui prend la zone « action en cours » : le temps qui reste avant
    /// la mort, ce qu'on peut attendre, et la touche d'abandon.
    fn bloc_a_terre(
        &mut self,
        ui: &mut UiCell,
        fonts: &Fonts,
        i18n: &Localization,
        secondes: u32,
        part: f32,
    ) {
        let bas = self.bas_action();
        let Some(t) = self.textes_vie.as_ref() else {
            return;
        };
        let reste = garde(&mut self.vivants.reste, secondes, || {
            i18n.get_msg_ctx("icebreaker-a-terre-reste", &i18n::fluent_args! {
                "secondes" => secondes,
            })
            .into_owned()
        });
        let fond = self.ids.action_fond;
        cadre([TERRE_W, TERRE_H], FOND, ROUGE_BORD, 2.0)
            .mid_bottom_with_margin_on(ui.window, bas)
            .set(fond, ui);
        Text::new(&t.terre)
            .font_id(fonts.alkhemi.conrod_id)
            .font_size(fonts.alkhemi.scale(52))
            .color(MAUVAIS)
            .top_left_with_margins_on(fond, 12.0, 20.0)
            .graphics_for(fond)
            .set(self.ids.terre_titre, ui);
        Text::new(reste)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(36))
            .color(TEXTE)
            .top_right_with_margins_on(fond, 24.0, 20.0)
            .graphics_for(fond)
            .set(self.ids.terre_reste, ui);
        let largeur = TERRE_W - 40.0;
        cadre([largeur, TERRE_JAUGE_H], CASE, BORDURE, 2.0)
            .top_left_with_margins_on(fond, 76.0, 20.0)
            .graphics_for(fond)
            .set(self.ids.terre_jauge_fond, ui);
        let rempli = (largeur - 4.0) * f64::from(part.clamp(0.0, 1.0));
        if rempli >= 1.0 {
            Rectangle::fill_with([rempli, TERRE_JAUGE_H - 4.0], ROUGE_BORD)
                .top_left_with_margins_on(self.ids.terre_jauge_fond, 2.0, 2.0)
                .graphics_for(fond)
                .set(self.ids.terre_jauge_part, ui);
        }
        Text::new(&t.terre_aide)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(31))
            .color(TEXTE)
            .top_left_with_margins_on(fond, 114.0, 20.0)
            .graphics_for(fond)
            .set(self.ids.terre_aide, ui);
        Text::new(&t.abandon)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(25))
            .color(DISCRET)
            .bottom_left_with_margins_on(fond, 14.0, 20.0)
            .graphics_for(fond)
            .set(self.ids.terre_abandon, ui);
    }

    /// L'écran de mort, centré sur le fond noir : le titre, le nom, ce qui est perdu, puis les
    /// rappels des deux touches (retour au village, regarder un allié). Ce ne sont pas des
    /// boutons : le curseur reste capturé.
    fn ecran_mort(
        &mut self,
        ui: &mut UiCell,
        fonts: &Fonts,
        i18n: &Localization,
        nom: &str,
        allie: Option<&str>,
    ) {
        let Some(t) = self.textes_vie.as_ref() else {
            return;
        };
        let cyri = fonts.cyri.conrod_id;
        let zone = self.ids.mort_zone;
        // Un fond sombre sous tout le bloc : les noms des entités du monde, dessinés avant, ne se
        // lisent plus à travers le titre.
        const MARGE: f64 = 28.0;
        let hauteur = 180.0 + MORT_PERDU_H + MORT_ESPACE + MORT_CARTE_H;
        Rectangle::fill_with(
            [MORT_W + 2.0 * MARGE, hauteur + 2.0 * MARGE],
            Color::Rgba(0.0, 0.0, 0.0, 0.95),
        )
        .mid_top_with_margin_on(ui.window, MORT_HAUT - MARGE)
        .set(self.ids.mort_fond, ui);
        Rectangle::fill_with([MORT_W, hauteur], color::TRANSPARENT)
            .mid_top_with_margin_on(ui.window, MORT_HAUT)
            .set(zone, ui);
        Text::new(&t.mort_titre)
            .font_id(fonts.alkhemi.conrod_id)
            .font_size(fonts.alkhemi.scale(88))
            .color(MAUVAIS)
            .mid_top_with_margin_on(zone, 0.0)
            .graphics_for(zone)
            .set(self.ids.mort_titre, ui);
        Text::new(nom)
            .font_id(cyri)
            .font_size(fonts.cyri.scale(38))
            .color(TEXTE)
            .mid_top_with_margin_on(zone, 112.0)
            .graphics_for(zone)
            .set(self.ids.mort_nom, ui);
        // Ce qui est perdu, et ce qui reste.
        let perdu = self.ids.mort_perdu_fond;
        cadre([MORT_W, MORT_PERDU_H], CASE, ROUGE_BORD, 2.0)
            .mid_top_with_margin_on(zone, 180.0)
            .graphics_for(zone)
            .set(perdu, ui);
        Text::new(&t.perdu)
            .font_id(cyri)
            .font_size(fonts.cyri.scale(38))
            .color(TEXTE)
            .top_left_with_margins_on(perdu, 20.0, 26.0)
            .graphics_for(perdu)
            .set(self.ids.mort_perdu, ui);
        Text::new(&t.sac)
            .font_id(cyri)
            .font_size(fonts.cyri.scale(28))
            .color(CLAIR)
            .top_left_with_margins_on(perdu, 76.0, 26.0)
            .graphics_for(perdu)
            .set(self.ids.mort_sac, ui);
        // Les deux rappels, côte à côte.
        for liste in [
            &mut self.ids.mort_cartes_fond,
            &mut self.ids.mort_cartes_touche_fond,
            &mut self.ids.mort_cartes_touche,
            &mut self.ids.mort_cartes_titre,
            &mut self.ids.mort_cartes_sous,
        ] {
            assez(liste, 2, ui);
        }
        let sous_allie = allie.map(|allie| {
            garde_nom(&mut self.vivants.allie, allie, |allie| {
                i18n.get_msg_ctx("icebreaker-mort-allie", &i18n::fluent_args! { "nom" => allie })
                    .into_owned()
            })
        });
        let largeur = (MORT_W - MORT_ESPACE) / 2.0;
        let haut = 180.0 + MORT_PERDU_H + MORT_ESPACE;
        // (touche, titre, sous-titre, actif, bordure)
        let cartes = [
            (&t.touche_retour, &t.retour, Some(&*t.retour_sous), true, OR),
            (&t.clic, &t.regarder, Some(sous_allie.unwrap_or(&t.aucun)), allie.is_some(), BORDURE),
        ];
        for (i, (touche, titre, sous, actif, bord)) in cartes.into_iter().enumerate() {
            let fond = self.ids.mort_cartes_fond[i];
            cadre(
                [largeur, MORT_CARTE_H],
                if actif { FOND_CARTE } else { CASE },
                if actif { bord } else { TRAIT },
                2.0,
            )
            .top_left_with_margins_on(zone, haut, i as f64 * (largeur + MORT_ESPACE))
            .graphics_for(zone)
            .set(fond, ui);
            let touche_fond = self.ids.mort_cartes_touche_fond[i];
            // Une lettre fait environ 14 unités de large à cette taille.
            let w_touche = touche.chars().count() as f64 * 14.0 + 24.0;
            cadre([w_touche, 38.0], FOND_TOUCHE, if actif { CLAIR } else { BORDURE }, 2.0)
                .top_left_with_margins_on(fond, 18.0, 20.0)
                .graphics_for(fond)
                .set(touche_fond, ui);
            Text::new(touche)
                .font_id(cyri)
                .font_size(fonts.cyri.scale(25))
                .color(if actif { TEXTE } else { GRIS })
                .middle_of(touche_fond)
                .graphics_for(fond)
                .set(self.ids.mort_cartes_touche[i], ui);
            Text::new(titre)
                .font_id(cyri)
                .font_size(fonts.cyri.scale(38))
                .color(if actif { TEXTE } else { GRIS })
                .top_left_with_margins_on(fond, 66.0, 20.0)
                .graphics_for(fond)
                .set(self.ids.mort_cartes_titre[i], ui);
            Text::new(sous.unwrap_or(""))
                .font_id(cyri)
                .font_size(fonts.cyri.scale(25))
                .color(if actif { CLAIR } else { DISCRET })
                .top_left_with_margins_on(fond, 112.0, 20.0)
                .graphics_for(fond)
                .set(self.ids.mort_cartes_sous[i], ui);
        }
    }

    /// Le bandeau de l'allié regardé par un mort, en haut au centre : son nom et sa vie, puis
    /// les deux touches utiles.
    fn bandeau_regarde(
        &mut self,
        ui: &mut UiCell,
        fonts: &Fonts,
        i18n: &Localization,
        nom: &str,
        vie: u32,
        max: u32,
        haut: f64,
    ) {
        let Some(t) = self.textes_vie.as_ref() else {
            return;
        };
        let cyri = fonts.cyri.conrod_id;
        let regarde = garde_nom(&mut self.vivants.regarde, nom, |nom| {
            couper(
                &i18n
                    .get_msg_ctx("icebreaker-regarde-nom", &i18n::fluent_args! { "nom" => nom }),
                32,
            )
        });
        let texte_vie = garde(&mut self.vivants.vie, (vie, max), || {
            i18n.get_msg_ctx("icebreaker-regarde-vie", &i18n::fluent_args! {
                "vie" => vie,
                "max" => max,
            })
            .into_owned()
        });
        let fond = self.ids.regarde_fond;
        cadre([BANDEAU_W, REGARDE_H], FOND, BORDURE, 2.0)
            .mid_top_with_margin_on(ui.window, haut)
            .set(fond, ui);
        cadre([84.0, 30.0], FOND, MAUVAIS, 2.0)
            .top_left_with_margins_on(fond, 12.0, 14.0)
            .graphics_for(fond)
            .set(self.ids.regarde_pastille, ui);
        Text::new(&t.mot_mort)
            .font_id(cyri)
            .font_size(fonts.cyri.scale(18))
            .color(MAUVAIS)
            .middle_of(self.ids.regarde_pastille)
            .graphics_for(fond)
            .set(self.ids.regarde_mot, ui);
        Text::new(regarde)
            .font_id(cyri)
            .font_size(fonts.cyri.scale(24))
            .color(TEXTE)
            .top_left_with_margins_on(fond, 13.0, 112.0)
            .graphics_for(fond)
            .set(self.ids.regarde_nom, ui);
        cadre([REGARDE_JAUGE_W, 18.0], CASE, BORDURE, 2.0)
            .top_right_with_margins_on(fond, 16.0, 14.0)
            .graphics_for(fond)
            .set(self.ids.regarde_jauge_fond, ui);
        let part = if max == 0 { 0.0 } else { (vie as f32 / max as f32).clamp(0.0, 1.0) };
        let rempli = (REGARDE_JAUGE_W - 4.0) * f64::from(part);
        if rempli >= 1.0 {
            Rectangle::fill_with([rempli, 14.0], remplissage(Ton::Bon))
                .top_left_with_margins_on(self.ids.regarde_jauge_fond, 2.0, 2.0)
                .graphics_for(fond)
                .set(self.ids.regarde_jauge_part, ui);
        }
        Text::new(texte_vie)
            .font_id(cyri)
            .font_size(fonts.cyri.scale(20))
            .color(CLAIR)
            .top_right_with_margins_on(fond, 15.0, 14.0 + REGARDE_JAUGE_W + 12.0)
            .graphics_for(fond)
            .set(self.ids.regarde_vie, ui);
        Text::new(&t.aide_regarde)
            .font_id(cyri)
            .font_size(fonts.cyri.scale(18))
            .color(DISCRET)
            .top_left_with_margins_on(fond, 56.0, 14.0)
            .graphics_for(fond)
            .set(self.ids.regarde_aide, ui);
    }

    /// La fenêtre du village : bandeau de titre, plan à gauche, panneau à droite.
    fn fenetre(
        &mut self,
        ui: &mut UiCell,
        imgs: &Imgs,
        item_imgs: &ItemImgs,
        fonts: &Fonts,
        i18n: &Localization,
        controls: &ControlSettings,
        pulse: f32,
        evenements: &mut Vec<Event>,
    ) {
        let [titre, aide, vous, vide] = self.textes.get_or_insert_with(|| {
            let touche = controls
                .get_binding(GameInput::IcebreakerTopView)
                .map_or_else(String::new, |touche| touche.display_string());
            [
                i18n.get_msg("icebreaker-plan-titre").into_owned(),
                i18n.get_msg_ctx("icebreaker-plan-aide", &i18n::fluent_args! { "key" => touche })
                    .into_owned(),
                i18n.get_msg("icebreaker-plan-vous").into_owned(),
                i18n.get_msg("icebreaker-plan-vide").into_owned(),
            ]
        });

        // Le cadre et le bandeau de titre.
        cadre([FEN_W, FEN_H], FOND, BORDURE, 2.0)
            .mid_top_with_margin_on(ui.window, FEN_HAUT)
            .set(self.ids.fond, ui);
        Rectangle::fill_with([FEN_W - 4.0, BANDEAU_H], BANDEAU)
            .mid_top_with_margin_on(self.ids.fond, 2.0)
            .graphics_for(self.ids.fond)
            .set(self.ids.bandeau, ui);
        Rectangle::fill_with([FEN_W - 4.0, 2.0], BORDURE)
            .mid_top_with_margin_on(self.ids.fond, 2.0 + BANDEAU_H)
            .graphics_for(self.ids.fond)
            .set(self.ids.bandeau_trait, ui);
        Text::new(titre)
            .font_id(fonts.alkhemi.conrod_id)
            .font_size(fonts.alkhemi.scale(34))
            .color(TEXTE)
            .top_left_with_margins_on(self.ids.fond, 5.0, 18.0)
            .graphics_for(self.ids.fond)
            .set(self.ids.titre, ui);
        Text::new(aide)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(18))
            .color(DISCRET)
            .top_right_with_margins_on(self.ids.fond, 15.0, 54.0)
            .graphics_for(self.ids.fond)
            .set(self.ids.aide, ui);
        if Button::image(imgs.close_btn)
            .w_h(24.0, 25.0)
            .hover_image(imgs.close_btn_hover)
            .press_image(imgs.close_btn_press)
            .top_right_with_margins_on(self.ids.fond, 13.0, 14.0)
            .set(self.ids.fermer, ui)
            .was_clicked()
        {
            evenements.push(Event::IcebreakerPlan(Demande::Fermer));
        }

        // Le plan : une case par bâtiment, rangée près de sa place.
        let haut = 2.0 + BANDEAU_H + 2.0;
        Rectangle::fill_with([PLAN_W, CORPS_H], FOND_PLAN)
            .top_left_with_margins_on(self.ids.fond, haut, 2.0)
            .set(self.ids.plan, ui);
        let n = self.cases.len();
        for liste in [
            &mut self.ids.plan_cases,
            &mut self.ids.plan_icones,
            &mut self.ids.plan_noms,
            &mut self.ids.plan_etats,
        ] {
            assez(liste, n, ui);
        }
        let mut clic = None;
        for (i, case) in self.cases.iter().enumerate() {
            let id = self.ids.plan_cases[i];
            let survol = ui.widget_input(id).mouse().is_some();
            let choisi = self.choisi == Some(case.emprise);
            let vif = survol || choisi;
            let (r, g, b) = case.couleur;
            cadre(
                [CASE_W, CASE_H],
                melange(case.couleur, if vif { 0.30 } else { 0.14 }, FOND_PLAN),
                if choisi { OR } else { Color::Rgba(r, g, b, 1.0) },
                if vif { 3.0 } else { 2.0 },
            )
            .x_y_relative_to(self.ids.plan, case.centre.x, case.centre.y)
            .set(id, ui);
            if let Some(image) = icone(imgs, &case.icone) {
                Image::new(image)
                    .w_h(32.0, 32.0)
                    .mid_top_with_margin_on(id, 8.0)
                    .graphics_for(id)
                    .set(self.ids.plan_icones[i], ui);
            }
            Text::new(&case.nom)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(16))
                .color(TEXTE)
                .mid_top_with_margin_on(id, 42.0)
                .graphics_for(id)
                .set(self.ids.plan_noms[i], ui);
            Text::new(&case.etat)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(14))
                .color(DISCRET)
                .mid_top_with_margin_on(id, 61.0)
                .graphics_for(id)
                .set(self.ids.plan_etats[i], ui);
            if ui.widget_input(id).clicks().left().next().is_some() {
                clic = Some(case.emprise);
            }
        }
        if let Some(emprise) = clic {
            // Comme `choisir`, champ par champ : les textes de la fenêtre sont encore empruntés.
            if self.choisi != Some(emprise) {
                self.panneau = None;
                self.reponse = None;
            }
            self.choisi = Some(emprise);
            let centre = (emprise.min + emprise.max).as_::<f32>() / 2.0;
            evenements.push(Event::IcebreakerPlan(Demande::Batiment(centre)));
        }
        // Le joueur : un point (rien pour un fantôme), à sa place entre les cases.
        if let (Some(repere), Some(joueur)) = (self.repere, self.joueur) {
            let bord = Vec2::new(COLONNES as f32 - 0.5, RANGS as f32 - 0.5);
            let place = repere
                .place(joueur, COLONNES, RANGS)
                .clamped(Vec2::broadcast(-0.5), bord);
            let p = centre_case(place);
            let (r, g, b) = self.reglages.couleur_joueur;
            widget::Circle::fill(6.0)
                .color(Color::Rgba(r, g, b, 1.0))
                .x_y_relative_to(self.ids.plan, p.x, p.y)
                .graphics_for(self.ids.plan)
                .set(self.ids.plan_joueur, ui);
            Text::new(vous)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(15))
                .color(TEXTE)
                .x_y_relative_to(self.ids.plan, p.x, p.y - 15.0)
                .graphics_for(self.ids.plan)
                .set(self.ids.plan_vous, ui);
        }

        // Le panneau du bâtiment cliqué, à droite.
        Rectangle::fill_with([2.0, CORPS_H], TRAIT)
            .top_left_with_margins_on(self.ids.fond, haut, 2.0 + PLAN_W)
            .set(self.ids.separation, ui);
        Rectangle::fill_with([PAN_W, CORPS_H], color::TRANSPARENT)
            .top_right_with_margins_on(self.ids.fond, haut, 2.0)
            .set(self.ids.pan, ui);
        let Some(panneau) = self.panneau.take() else {
            Text::new(vide)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(20))
                .color(DISCRET)
                .middle_of(self.ids.pan)
                .set(self.ids.pan_vide, ui);
            return;
        };
        if let Some(commande) = dessiner_panneau(
            &mut self.ids,
            ui,
            &panneau,
            &mut self.onglet,
            self.attente.as_ref().map(|(commande, _)| commande.as_slice()),
            self.reponse.as_ref().map(|(texte, sorte, _)| (texte.as_str(), *sorte)),
            imgs,
            item_imgs,
            fonts,
            pulse,
        ) {
            if let Some((nom, arguments)) = commande.split_first() {
                evenements.push(Event::SendCommand(nom.clone(), arguments.to_vec()));
            }
            // Le bouton passe « en cours » tout de suite, sans attendre le serveur.
            self.attente = Some((commande, Instant::now()));
            self.reponse = None;
        }
        self.panneau = Some(panneau);
    }

    /// Un panneau libre, hors du plan : le choix de la couleur de la balise,
    /// une pastille par couleur, au-dessus de la barre de raccourcis.
    fn libre(&mut self, ui: &mut UiCell, fonts: &Fonts, evenements: &mut Vec<Event>) {
        let Some(panneau) = self.panneau.take() else {
            return;
        };
        const COTE: f64 = 48.0;
        let n = panneau.actions.len();
        // Assez large pour le titre et la couleur actuelle sur une ligne, et pour le message.
        let largeur = (n as f64 * (COTE + 8.0) - 8.0 + 2.0 * MARGE).max(600.0);
        let fond = self.ids.libre_fond;
        cadre([largeur, 150.0], FOND, BORDURE, 2.0)
            .mid_bottom_with_margin_on(ui.window, 180.0)
            .set(fond, ui);
        Text::new(&panneau.titre)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(24))
            .color(TEXTE)
            .top_left_with_margins_on(fond, 10.0, MARGE)
            .graphics_for(fond)
            .set(self.ids.libre_titre, ui);
        Text::new(&panneau.description)
            .font_id(fonts.cyri.conrod_id)
            .font_size(fonts.cyri.scale(18))
            .color(DISCRET)
            .top_right_with_margins_on(fond, 14.0, MARGE)
            .graphics_for(fond)
            .set(self.ids.libre_desc, ui);
        assez(&mut self.ids.libre_cases, n, ui);
        assez(&mut self.ids.libre_couleurs, n, ui);
        let mut choix = None;
        for (i, action) in panneau.actions.iter().enumerate() {
            let id = self.ids.libre_cases[i];
            let survol = ui.widget_input(id).mouse().is_some();
            cadre(
                [COTE, COTE],
                if survol { BANDEAU } else { CASE },
                if action.principale { OR } else { BORDURE },
                if action.principale { 3.0 } else { 2.0 },
            )
            .top_left_with_margins_on(fond, 46.0, MARGE + i as f64 * (COTE + 8.0))
            .set(id, ui);
            let couleur = action
                .couleur
                .unwrap_or(Rgb::broadcast(255))
                .map(|c| f32::from(c) / 255.0);
            Rectangle::fill_with([24.0, 24.0], Color::Rgba(couleur.r, couleur.g, couleur.b, 1.0))
                .middle_of(id)
                .graphics_for(id)
                .set(self.ids.libre_couleurs[i], ui);
            if ui.widget_input(id).clicks().left().next().is_some() {
                choix = Some(action.commande.clone());
            }
        }
        if let Some((message, _)) = &panneau.message {
            Text::new(message)
                .font_id(fonts.cyri.conrod_id)
                .font_size(fonts.cyri.scale(16))
                .color(DISCRET)
                .bottom_left_with_margins_on(fond, 12.0, MARGE)
                .graphics_for(fond)
                .set(self.ids.libre_message, ui);
        }
        match choix {
            // La couleur choisie, le panneau se ferme (J.3).
            Some(commande) => {
                if let Some((nom, arguments)) = commande.split_first() {
                    evenements.push(Event::SendCommand(nom.clone(), arguments.to_vec()));
                }
            },
            None => self.panneau = Some(panneau),
        }
    }
}

/// Assez d'identifiants dans `liste` pour `n` éléments.
fn assez(liste: &mut widget::id::List, n: usize, ui: &mut UiCell) {
    if liste.len() < n {
        liste.resize(n, &mut ui.widget_id_generator());
    }
}

/// Le panneau typé, bloc par bloc, chacun à sa place fixe dans `ids.pan`. Rend
/// la commande du bouton cliqué, s'il y en a un.
fn dessiner_panneau(
    ids: &mut Ids,
    ui: &mut UiCell,
    p: &Pret,
    onglet: &mut usize,
    attente: Option<&[String]>,
    reponse: Option<(&str, Sorte)>,
    imgs: &Imgs,
    item_imgs: &ItemImgs,
    fonts: &Fonts,
    pulse: f32,
) -> Option<Vec<String>> {
    let ancre = ids.pan;
    let cyri = fonts.cyri.conrod_id;
    let taille = |points: u32| fonts.cyri.scale(points);
    *onglet = (*onglet).min(p.onglets.len().saturating_sub(1));
    let lignes = p.onglets.get(*onglet).map_or(&p.lignes, |(_, lignes)| lignes);
    let rangs_objets = p.objets.len().min(2 * OBJETS_PAR_RANG).div_ceil(OBJETS_PAR_RANG);
    let rangs_actions = p.actions.len().div_ceil(p.colonnes);
    let places = places(
        !p.onglets.is_empty(),
        !p.chiffres.is_empty(),
        p.jauge.is_some(),
        rangs_objets,
        !lignes.is_empty(),
        rangs_actions,
    );
    let mut clic = None;

    // L'en-tête : icône, titre, ce que c'est en une ligne.
    let image = icone(imgs, &p.icone);
    if let Some(image) = image {
        Image::new(image)
            .w_h(44.0, 44.0)
            .top_left_with_margins_on(ancre, 14.0, MARGE)
            .set(ids.p_icone, ui);
    }
    let x_titre = if image.is_some() { MARGE + 56.0 } else { MARGE };
    Text::new(&p.titre)
        .font_id(fonts.alkhemi.conrod_id)
        .font_size(fonts.alkhemi.scale(32))
        .color(TEXTE)
        .top_left_with_margins_on(ancre, 8.0, x_titre)
        .set(ids.p_titre, ui);
    Text::new(&p.description)
        .font_id(cyri)
        .font_size(taille(18))
        .color(DISCRET)
        .top_left_with_margins_on(ancre, 46.0, x_titre)
        .set(ids.p_desc, ui);

    // Les onglets : le changement est immédiat, le serveur a déjà tout envoyé.
    if let Some(y) = places.onglets {
        let n = p.onglets.len();
        assez(&mut ids.onglets, n, ui);
        let largeur = (LARGEUR - 6.0 * (n as f64 - 1.0)) / n as f64;
        for (i, (libelle, _)) in p.onglets.iter().enumerate() {
            let actif = i == *onglet;
            if Button::image(if actif { imgs.button_press } else { imgs.button })
                .w_h(largeur, ONGLET_H)
                .hover_image(if actif { imgs.button_press } else { imgs.button_hover })
                .press_image(imgs.button_press)
                .image_color(if actif { TEXTE } else { IMAGE_GRISE })
                .label(libelle)
                .label_font_id(cyri)
                .label_font_size(taille(20))
                .label_color(if actif { OR } else { DISCRET })
                .label_y(Relative::Scalar(2.0))
                .top_left_with_margins_on(ancre, y, MARGE + i as f64 * (largeur + 6.0))
                .set(ids.onglets[i], ui)
                .was_clicked()
            {
                *onglet = i;
            }
        }
    }

    // Les chiffres qui comptent, en grand.
    if let Some(y) = places.chiffres {
        let n = p.chiffres.len();
        for liste in [
            &mut ids.chiffres_fond,
            &mut ids.chiffres_valeur,
            &mut ids.chiffres_legende,
        ] {
            assez(liste, n, ui);
        }
        let largeur = (LARGEUR - ESPACE * (n as f64 - 1.0)) / n as f64;
        for (i, (valeur, legende, ton)) in p.chiffres.iter().enumerate() {
            let fond = ids.chiffres_fond[i];
            cadre([largeur, CHIFFRE_H], CASE, TRAIT, 2.0)
                .top_left_with_margins_on(ancre, y, MARGE + i as f64 * (largeur + ESPACE))
                .set(fond, ui);
            Text::new(valeur)
                .font_id(cyri)
                .font_size(taille(34))
                .color(teinte(*ton))
                .top_left_with_margins_on(fond, 6.0, 12.0)
                .set(ids.chiffres_valeur[i], ui);
            Text::new(legende)
                .font_id(cyri)
                .font_size(taille(16))
                .color(DISCRET)
                .top_left_with_margins_on(fond, 46.0, 12.0)
                .set(ids.chiffres_legende[i], ui);
        }
    }

    // La jauge.
    if let (Some(y), Some((legende, texte, part, ton))) = (places.jauge, &p.jauge) {
        Text::new(legende)
            .font_id(cyri)
            .font_size(taille(18))
            .color(DISCRET)
            .top_left_with_margins_on(ancre, y, MARGE)
            .set(ids.jauge_legende, ui);
        Text::new(texte)
            .font_id(cyri)
            .font_size(taille(18))
            .color(TEXTE)
            .top_right_with_margins_on(ancre, y, MARGE)
            .set(ids.jauge_texte, ui);
        cadre([LARGEUR, 16.0], CASE, BORDURE, 2.0)
            .top_left_with_margins_on(ancre, y + 26.0, MARGE)
            .set(ids.jauge_fond, ui);
        let rempli = (LARGEUR - 4.0) * f64::from(*part);
        if rempli >= 1.0 {
            Rectangle::fill_with([rempli, 12.0], remplissage(*ton))
                .top_left_with_margins_on(ids.jauge_fond, 2.0, 2.0)
                .set(ids.jauge_part, ui);
        }
    }

    // Les objets, en cases.
    if let Some(y) = places.objets {
        let n = p.objets.len().min(2 * OBJETS_PAR_RANG);
        for liste in [
            &mut ids.objets_fond,
            &mut ids.objets_image,
            &mut ids.objets_nombre,
        ] {
            assez(liste, n, ui);
        }
        for (i, (cle, nombre)) in p.objets.iter().take(n).enumerate() {
            let (colonne, rang) = (i % OBJETS_PAR_RANG, i / OBJETS_PAR_RANG);
            let fond = ids.objets_fond[i];
            cadre([OBJET, OBJET], CASE, TRAIT, 2.0)
                .top_left_with_margins_on(
                    ancre,
                    y + rang as f64 * (OBJET + 8.0),
                    MARGE + colonne as f64 * (OBJET + 8.0),
                )
                .set(fond, ui);
            Image::new(animate_by_pulse(
                &item_imgs.img_ids_or_not_found_img(cle.clone()),
                pulse,
            ))
            .w_h(OBJET - 12.0, OBJET - 12.0)
            .middle_of(fond)
            .set(ids.objets_image[i], ui);
            Text::new(nombre)
                .font_id(cyri)
                .font_size(taille(16))
                .color(TEXTE)
                .bottom_right_with_margins_on(fond, 4.0, 6.0)
                .set(ids.objets_nombre[i], ui);
        }
    }

    // La liste : elle prend ce qui reste, et défile.
    if let Some((y, hauteur)) = places.liste {
        let n = lignes.len();
        for liste in [
            &mut ids.lignes_fond,
            &mut ids.lignes_icone,
            &mut ids.lignes_titre,
            &mut ids.lignes_sous,
            &mut ids.lignes_pastille,
            &mut ids.lignes_mot,
            &mut ids.lignes_cadenas,
            &mut ids.lignes_bouton,
        ] {
            assez(liste, n, ui);
        }
        Rectangle::fill_with([LARGEUR, hauteur], color::TRANSPARENT)
            .top_left_with_margins_on(ancre, y, MARGE)
            .scroll_kids_vertically()
            .set(ids.liste, ui);
        Scrollbar::y_axis(ids.liste)
            .thickness(6.0)
            .auto_hide(true)
            .color(BORDURE)
            .set(ids.liste_barre, ui);
        let largeur = LARGEUR - 12.0;
        for (i, ligne) in lignes.iter().enumerate() {
            let fond = ids.lignes_fond[i];
            cadre(
                [largeur, LIGNE_H],
                CASE,
                if ligne.choisie { OR } else { TRAIT },
                2.0,
            )
            .top_left_with_margins_on(ids.liste, i as f64 * (LIGNE_H + 4.0), 0.0)
            .set(fond, ui);
            let image = icone(imgs, &ligne.icone);
            if let Some(image) = image {
                Image::new(image)
                    .w_h(32.0, 32.0)
                    .mid_left_with_margin_on(fond, 10.0)
                    .graphics_for(fond)
                    .set(ids.lignes_icone[i], ui);
            }
            let x_texte = if image.is_some() { 52.0 } else { 12.0 };
            let seul = ligne.sous_titre.is_empty();
            Text::new(&ligne.titre)
                .font_id(cyri)
                .font_size(taille(22))
                .color(if ligne.estompee { GRIS } else { TEXTE })
                .top_left_with_margins_on(fond, if seul { 14.0 } else { 5.0 }, x_texte)
                .graphics_for(fond)
                .set(ids.lignes_titre[i], ui);
            if !seul {
                Text::new(&ligne.sous_titre)
                    .font_id(cyri)
                    .font_size(taille(16))
                    .color(DISCRET)
                    .top_left_with_margins_on(fond, 29.0, x_texte)
                    .graphics_for(fond)
                    .set(ids.lignes_sous[i], ui);
            }
            // À droite, de la droite vers la gauche : le bouton, le cadenas, la pastille.
            let mut droite = 8.0;
            if let Some(action) = &ligne.action {
                if bouton(imgs, action, attente)
                    .w_h(96.0, 36.0)
                    .label_font_id(cyri)
                    .label_font_size(taille(20))
                    .mid_right_with_margin_on(fond, droite)
                    .set(ids.lignes_bouton[i], ui)
                    .was_clicked()
                    && action.permise
                {
                    clic = Some(action.commande.clone());
                }
                droite += 96.0 + 8.0;
            }
            if let Some((ferme, commande)) = &ligne.cadenas {
                if Button::image(imgs.lock)
                    .w_h(30.0, 30.0)
                    .image_color(if *ferme { OR } else { IMAGE_GRISE })
                    .hover_image(imgs.lock)
                    .press_image(imgs.lock)
                    .mid_right_with_margin_on(fond, droite)
                    .set(ids.lignes_cadenas[i], ui)
                    .was_clicked()
                {
                    clic = Some(commande.clone());
                }
                droite += 30.0 + 10.0;
            }
            if let Some((mot, ton)) = &ligne.pastille {
                cadre([92.0, 28.0], CASE, teinte(*ton), 2.0)
                    .mid_right_with_margin_on(fond, droite)
                    .graphics_for(fond)
                    .set(ids.lignes_pastille[i], ui);
                Text::new(mot)
                    .font_id(cyri)
                    .font_size(taille(16))
                    .color(teinte(*ton))
                    .middle_of(ids.lignes_pastille[i])
                    .graphics_for(fond)
                    .set(ids.lignes_mot[i], ui);
            }
        }
    }

    // Les actions : une impossible reste visible, grisée ; la principale a un cadre doré.
    if let Some(y) = places.actions {
        let n = p.actions.len();
        assez(&mut ids.actions_cadre, n, ui);
        assez(&mut ids.actions, n, ui);
        let largeur = (LARGEUR - 8.0 * (p.colonnes as f64 - 1.0)) / p.colonnes as f64;
        for (i, action) in p.actions.iter().enumerate() {
            let (colonne, rang) = (i % p.colonnes, i / p.colonnes);
            let (haut, gauche) = (
                y + rang as f64 * (ACTION_H + 8.0),
                MARGE + colonne as f64 * (largeur + 8.0),
            );
            // Le cadre doré est un rectangle plein derrière le bouton, toujours posé avant lui
            // (transparent pour les autres actions) : l'ordre de dessin ne change jamais.
            let dore = action.principale && action.permise;
            Rectangle::fill_with(
                [largeur + 4.0, ACTION_H + 4.0],
                if dore { OR } else { color::TRANSPARENT },
            )
            .top_left_with_margins_on(ancre, haut - 2.0, gauche - 2.0)
            .set(ids.actions_cadre[i], ui);
            if bouton(imgs, action, attente)
                .w_h(largeur, ACTION_H)
                .label_font_id(cyri)
                .label_font_size(taille(22))
                .top_left_with_margins_on(ancre, haut, gauche)
                .set(ids.actions[i], ui)
                .was_clicked()
                && action.permise
            {
                clic = Some(action.commande.clone());
            }
        }
    }

    // Le message : la réponse du serveur à la dernière action, sinon la raison d'un bouton grisé.
    let message = reponse.or(p.message.as_ref().map(|(texte, sorte)| (texte.as_str(), *sorte)));
    if let Some((texte, sorte)) = message {
        Text::new(texte)
            .font_id(cyri)
            .font_size(taille(18))
            .color(match sorte {
                Sorte::Info => DISCRET,
                Sorte::Reussite => BON,
                Sorte::Refus => MAUVAIS,
            })
            .w(LARGEUR)
            .wrap_by_word()
            .top_left_with_margins_on(ancre, places.message, MARGE)
            .set(ids.message, ui);
    }
    clic
}

/// Un bouton de Veloren : grisé s'il n'est pas permis, enfoncé tant que sa
/// commande est en cours, libellé doré s'il est l'action principale.
fn bouton<'a>(
    imgs: &Imgs,
    action: &'a Bouton,
    attente: Option<&[String]>,
) -> Button<'a, widget::button::Image> {
    let en_cours = attente.is_some_and(|commande| commande == action.commande.as_slice());
    let repos = if en_cours { imgs.button_press } else { imgs.button };
    Button::image(repos)
        .hover_image(if action.permise && !en_cours {
            imgs.button_hover
        } else {
            repos
        })
        .press_image(if action.permise { imgs.button_press } else { repos })
        .image_color(if action.permise { TEXTE } else { IMAGE_GRISE })
        .label(&action.libelle)
        .label_color(if !action.permise || en_cours {
            GRIS
        } else if action.principale {
            OR
        } else {
            TEXTE
        })
        .label_y(Relative::Scalar(2.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn le_plan_garde_la_forme_du_village() {
        // Quatre bâtiments aux quatre coins, un au milieu.
        let centres = [
            Vec2::new(0.0, 100.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 50.0),
        ];
        let places = ranger(&centres, 5, 5);
        // Le nord (y qui monte) est en haut (rang 0), l'ouest à gauche (colonne 0).
        assert_eq!(places, vec![(0, 0), (4, 0), (0, 4), (4, 4), (2, 2)]);
    }

    #[test]
    fn deux_batiments_voisins_n_ont_jamais_la_meme_case() {
        // Un village serré : douze bâtiments dans quelques blocs, plus un au loin.
        let mut centres: Vec<Vec2<f32>> = (0..12)
            .map(|i| Vec2::new((i % 4) as f32 * 3.0, (i / 4) as f32 * 3.0))
            .collect();
        centres.push(Vec2::new(200.0, 150.0));
        let places = ranger(&centres, COLONNES, RANGS);
        assert_eq!(places.len(), centres.len());
        let uniques: HashSet<_> = places.iter().collect();
        assert_eq!(uniques.len(), places.len(), "{places:?}");
        assert!(
            places
                .iter()
                .all(|(colonne, rang)| *colonne < COLONNES && *rang < RANGS)
        );
        // Celui qui est loin au nord-est reste en haut à droite.
        assert_eq!(places[12], (COLONNES - 1, 0));
        assert!(ranger(&[], COLONNES, RANGS).is_empty());
    }

    #[test]
    fn les_cases_tiennent_dans_le_plan() {
        let coin_haut_gauche = centre_case(Vec2::new(0.0, 0.0));
        let coin_bas_droit = centre_case(Vec2::new((COLONNES - 1) as f32, (RANGS - 1) as f32));
        assert!(coin_haut_gauche.x - CASE_W / 2.0 >= -PLAN_W / 2.0);
        assert!(coin_bas_droit.x + CASE_W / 2.0 <= PLAN_W / 2.0);
        assert!(coin_haut_gauche.y + CASE_H / 2.0 <= CORPS_H / 2.0);
        assert!(coin_bas_droit.y - CASE_H / 2.0 >= -CORPS_H / 2.0);
        // Deux cases voisines ne se touchent pas.
        let voisine = centre_case(Vec2::new(1.0, 0.0));
        assert!(voisine.x - coin_haut_gauche.x >= CASE_W);
    }

    #[test]
    fn les_blocs_du_panneau_ne_se_chevauchent_pas() {
        // Tout à la fois : onglets, chiffres, jauge, deux rangs d'objets, liste, deux rangs d'actions.
        let p = places(true, true, true, 2, true, 2);
        let (onglets, chiffres, jauge, objets) =
            (p.onglets.unwrap(), p.chiffres.unwrap(), p.jauge.unwrap(), p.objets.unwrap());
        let (liste, hauteur) = p.liste.unwrap();
        let actions = p.actions.unwrap();
        assert!(onglets >= EN_TETE_H);
        assert!(chiffres >= onglets + ONGLET_H);
        assert!(jauge >= chiffres + CHIFFRE_H);
        assert!(objets >= jauge + JAUGE_H);
        assert!(liste >= objets + 2.0 * OBJET + 8.0);
        assert!(hauteur >= LIGNE_H, "la liste garde au moins une ligne");
        assert!(actions + 2.0 * ACTION_H + 8.0 <= p.message, "les actions au-dessus du message");
        assert!(p.message + MESSAGE_H <= CORPS_H, "le message dans la fenêtre");
    }

    #[test]
    fn la_liste_prend_la_hauteur_qui_reste() {
        // La charrette : onglets, liste, une action.
        let charrette = places(true, false, false, 0, true, 1);
        let (haut, hauteur) = charrette.liste.unwrap();
        assert_eq!(haut, EN_TETE_H + ONGLET_H + ESPACE);
        assert_eq!(haut + hauteur + ESPACE, charrette.actions.unwrap());
        assert!(hauteur >= 5.0 * LIGNE_H, "cinq personnages visibles sans défiler : {hauteur}");
        // Sans action, la liste descend jusqu'au message.
        let sans = places(true, false, false, 0, true, 0);
        assert_eq!(sans.actions, None);
        assert!(sans.liste.unwrap().1 > hauteur);
        // La caisse : rien à faire défiler, quatre boutons sur deux rangs.
        let caisse = places(false, true, true, 0, false, 2);
        assert_eq!(caisse.liste, None);
        assert!(caisse.actions.unwrap() >= caisse.jauge.unwrap() + JAUGE_H);
    }

    #[test]
    fn la_zone_action_montre_une_seule_chose() {
        let rien = EnCours { verrou: Avis::Rien, fouille: None };
        assert_eq!(rien.zone(), None);
        let rappel = EnCours { verrou: Avis::Rappel, fouille: None };
        assert_eq!(rappel.zone(), Some(Zone::Rappel));
        let perdue = EnCours { verrou: Avis::Perdue, fouille: None };
        assert_eq!(perdue.zone(), Some(Zone::Perdue));
        // La fouille l'emporte sur le verrouillage, et sa part reste dans [0, 1].
        let fouille = EnCours { verrou: Avis::Rappel, fouille: Some(1.4) };
        assert_eq!(fouille.zone(), Some(Zone::Fouille(1.0)));
    }

    fn etat(registre: bool, mort: bool, regarde: bool, a_terre: Option<(f64, f32)>, allie: bool) -> Etat {
        etat_joueur(
            registre,
            mort,
            regarde,
            a_terre,
            || "Brunhild".to_string(),
            || allie.then(|| "Sayanel".to_string()),
            || ("Sayanel".to_string(), 64, 100),
        )
    }

    #[test]
    fn l_etat_du_joueur_selon_la_situation() {
        // Vivant, ou hors campagne : rien.
        assert_eq!(etat(true, false, false, None, true), Etat::Rien);
        assert_eq!(etat(false, true, false, Some((10.0, 0.5)), true), Etat::Rien);
        // À terre : les secondes arrondies au-dessus, la part bornée.
        assert_eq!(
            etat(true, false, false, Some((17.2, 1.3)), false),
            Etat::ATerre { secondes: 18, part: 1.0 }
        );
        // Mort sans allié en vie, puis avec.
        assert_eq!(
            etat(true, true, false, None, false),
            Etat::Mort { nom: "Brunhild".into(), allie: None }
        );
        assert_eq!(
            etat(true, true, false, None, true),
            Etat::Mort { nom: "Brunhild".into(), allie: Some("Sayanel".into()) }
        );
        // Mort qui regarde un allié : le bandeau, plus l'écran de mort.
        assert_eq!(
            etat(true, true, true, None, true),
            Etat::Regarde { nom: "Sayanel".into(), vie: 64, max: 100 }
        );
    }

    #[test]
    fn les_noms_ne_sont_cherches_que_si_l_etat_les_montre() {
        use std::cell::Cell;
        let appels = Cell::new(0);
        let compter = || {
            appels.set(appels.get() + 1);
            String::new()
        };
        etat_joueur(true, false, false, None, compter, || None, || (String::new(), 0, 0));
        etat_joueur(true, false, false, Some((5.0, 0.2)), compter, || None, || (String::new(), 0, 0));
        assert_eq!(appels.get(), 0);
        etat_joueur(true, true, false, None, compter, || None, || (String::new(), 0, 0));
        assert_eq!(appels.get(), 1);
    }

    #[test]
    fn un_texte_n_est_refait_que_si_sa_donnee_change() {
        use std::cell::Cell;
        let faits = Cell::new(0);
        let mut cache = None;
        for secondes in [30u32, 30, 30, 29, 29, 28] {
            let texte = garde(&mut cache, secondes, || {
                faits.set(faits.get() + 1);
                format!("Mort dans {secondes} s")
            });
            assert_eq!(texte, format!("Mort dans {secondes} s"));
        }
        assert_eq!(faits.get(), 3);
        // Un nom : refait quand il change, pas à chaque image.
        let mut nom = None;
        let mut refaits = 0;
        for qui in ["Sayanel", "Sayanel", "Flo"] {
            garde_nom(&mut nom, qui, |qui| {
                refaits += 1;
                format!("{qui} est encore en vie")
            });
        }
        assert_eq!(refaits, 2);
        assert_eq!(nom.unwrap().1, "Flo est encore en vie");
    }

    #[test]
    fn la_part_de_fouille() {
        use std::time::Duration;
        let trois = Duration::from_secs(3);
        assert_eq!(part_fouille(Duration::ZERO, trois), 0.0);
        assert!((part_fouille(Duration::from_millis(1500), trois) - 0.5).abs() < 1e-6);
        assert_eq!(part_fouille(Duration::from_secs(5), trois), 1.0);
        assert_eq!(part_fouille(Duration::ZERO, Duration::ZERO), 1.0);
    }

    #[test]
    fn un_texte_trop_long_est_coupe() {
        assert_eq!(couper("court", 10), "court");
        assert_eq!(couper("exactement", 10), "exactement");
        assert_eq!(couper("beaucoup trop long", 10), "beaucoup …");
        assert_eq!(couper("été à l'été", 5).chars().count(), 5);
    }
}
