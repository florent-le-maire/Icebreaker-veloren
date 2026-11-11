//! Icebreaker: travelling between the places of the campaign (V.8). A server
//! tells the client to go to another place (`IcebreakerTravel`): the session
//! notes where and with which character, and goes back to the main menu,
//! which connects there; the character selection then plays that character.

use common::character::CharacterId;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

struct Travel {
    /// Where to connect, `host:port`; taken by the main menu.
    address: Option<String>,
    /// Who to play there; taken by the character selection.
    character: Option<CharacterId>,
}

static TRAVEL: Mutex<Option<Travel>> = Mutex::new(None);

/// Icebreaker: the character of the registry to play once on the character
/// screen (jalon R); none to come back as a ghost.
static JOUER: Mutex<Option<CharacterId>> = Mutex::new(None);

/// Plays the character `id` of the registry (none: a ghost) once back on the
/// character screen.
pub fn jouer(id: Option<CharacterId>) { *JOUER.lock().unwrap_or_else(|e| e.into_inner()) = id; }

/// The character of the registry to play, if any.
pub fn take_jouer() -> Option<CharacterId> { JOUER.lock().unwrap_or_else(|e| e.into_inner()).take() }

/// Icebreaker N.8 : le bilan de retour que le joueur n'a pas encore fermé. Quand l'expédition a
/// péri, il arrive juste avant le retour en fantôme, qui repasse par l'écran des personnages et
/// refait le HUD : il attend ici le HUD suivant.
static BILAN: Mutex<Option<common::icebreaker_panneau::Panneau>> = Mutex::new(None);

pub fn garder_bilan(panneau: common::icebreaker_panneau::Panneau) {
    *BILAN.lock().unwrap_or_else(|e| e.into_inner()) = Some(panneau);
}

/// Le bilan pas encore fermé, s'il y en a un.
pub fn bilan_garde() -> Option<common::icebreaker_panneau::Panneau> {
    BILAN.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Le joueur a fermé le bilan.
pub fn bilan_vu() { *BILAN.lock().unwrap_or_else(|e| e.into_inner()) = None; }

fn travel() -> std::sync::MutexGuard<'static, Option<Travel>> {
    TRAVEL.lock().unwrap_or_else(|e| e.into_inner())
}

/// Goes to the place listening on `port`, on the host of `current_server`
/// (`host:port`), with `character`.
pub fn go(current_server: &str, port: u16, character: Option<CharacterId>) {
    let host = current_server
        .rsplit_once(':')
        .map_or(current_server, |(host, _)| host);
    let host = if host.is_empty() { "localhost" } else { host };
    *travel() = Some(Travel {
        address: Some(format!("{host}:{port}")),
        character,
    });
}

/// Where to connect, if travelling.
pub fn take_address() -> Option<String> { travel().as_mut()?.address.take() }

/// Whether a character is to be played after a travel (the ghost of the
/// registry waits for it).
pub fn travelling() -> bool { travel().as_ref().is_some_and(|t| t.character.is_some()) }

/// Who to play, if travelling.
pub fn take_character() -> Option<CharacterId> { travel().take()?.character }

/// Icebreaker: a travel that failed (no connection) is over, so that the next
/// connection by hand starts clean (X.7).
pub fn annuler() { *travel() = None; }

/// Icebreaker: whether the player connected to a place of the campaign (X.7):
/// the character screen then stays a plain loading screen until the server
/// says whether to enter as a ghost.
static CAMPAGNE: AtomicBool = AtomicBool::new(false);

pub fn campagne_attendue() { CAMPAGNE.store(true, Ordering::Relaxed); }

pub fn campagne_fini() { CAMPAGNE.store(false, Ordering::Relaxed); }

pub fn en_campagne() -> bool { CAMPAGNE.load(Ordering::Relaxed) }

// Icebreaker N.6 : les étapes du lancement, affichées sur l'écran de connexion et
// sur l'écran d'attente des personnages.

/// Ce que le menu a lancé : la partie (ou le lieu distant) pour le titre du panneau.
#[derive(Clone, Debug, Default)]
pub struct Lancement {
    /// Nom de la partie, ou du lieu pour un serveur distant.
    pub titre: String,
    /// Nom du lieu pour une partie locale (vide pour un serveur distant).
    pub sous_titre: String,
    /// Serveur distant : l'étape « Serveur lancé » n'existe pas.
    pub distant: bool,
}

static LANCEMENT: Mutex<Option<Lancement>> = Mutex::new(None);

/// Note ce que le menu vient de lancer.
pub fn noter_lancement(lancement: Lancement) {
    // Un bilan de retour resté ouvert dans une partie d'avant ne revient pas dans celle-ci (N.8).
    bilan_vu();
    *LANCEMENT.lock().unwrap_or_else(|e| e.into_inner()) = Some(lancement);
}

/// Le dernier lancement noté (un voyage entre lieux garde celui du départ).
pub fn lancement() -> Lancement {
    LANCEMENT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Etape {
    Serveur,
    Monde,
    Connexion,
    Village,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EtatEtape {
    Fait,
    EnCours,
    AVenir,
}

impl Etape {
    /// Clé `.ftl` du nom de l'étape.
    pub fn cle(self) -> &'static str {
        match self {
            Etape::Serveur => "icebreaker-etape-serveur",
            Etape::Monde => "icebreaker-etape-monde",
            Etape::Connexion => "icebreaker-etape-connexion",
            Etape::Village => "icebreaker-etape-village",
        }
    }
}

impl EtatEtape {
    /// Clé `.ftl` du mot qui dit l'état.
    pub fn cle(self) -> &'static str {
        match self {
            EtatEtape::Fait => "icebreaker-etat-fait",
            EtatEtape::EnCours => "icebreaker-etat-en-cours",
            EtatEtape::AVenir => "icebreaker-etat-a-venir",
        }
    }
}

/// L'étape en cours d'après ce que le client sait déjà. Tant que la connexion TCP
/// n'a pas abouti (`ConnectionEstablish`, ou rien encore), le serveur démarre ou
/// génère son monde et le client réessaie toutes les 5 s : « Préparation du
/// monde ». Dès que le serveur répond, c'est « Connexion » ; une fois dans
/// l'écran des personnages, « Arrivée au village ». Le serveur est considéré comme
/// lancé dès qu'on est là (le menu l'a démarré avant de se connecter).
pub fn etape_en_cours(
    stage: &crate::menu::main::DetailedInitializationStage,
    village: bool,
) -> Etape {
    use crate::menu::main::DetailedInitializationStage as Stage;
    use client::ClientInitStage;
    if village {
        Etape::Village
    } else if matches!(stage, Stage::Client(s) if !matches!(s, ClientInitStage::ConnectionEstablish))
        || matches!(stage, Stage::CreatingRenderPipeline(..))
    {
        Etape::Connexion
    } else {
        Etape::Monde
    }
}

/// Les étapes à montrer avec leur état ; « Serveur lancé » est sautée pour un
/// serveur distant.
pub fn etats_etapes(distant: bool, courante: Etape) -> Vec<(Etape, EtatEtape)> {
    [Etape::Serveur, Etape::Monde, Etape::Connexion, Etape::Village]
        .into_iter()
        .filter(|e| !(distant && *e == Etape::Serveur))
        .map(|e| {
            let etat = match e.cmp(&courante) {
                std::cmp::Ordering::Less => EtatEtape::Fait,
                std::cmp::Ordering::Equal => EtatEtape::EnCours,
                std::cmp::Ordering::Greater => EtatEtape::AVenir,
            };
            (e, etat)
        })
        .collect()
}

/// Le panneau du lancement : le nom de la partie et les étapes, l'étape en cours
/// mise en avant.
pub fn panneau_lancement<'a, M: 'a>(
    fonts: &crate::ui::fonts::IcedFonts,
    i18n: &i18n::Localization,
    lancement: &Lancement,
    courante: Etape,
) -> crate::ui::ice::Element<'a, M> {
    use crate::ui::ice::style;
    use iced::{Align, Color, Column, Container, Length, Row, Space, Text};

    let clair = Color::from_rgb(1.0, 1.0, 1.0);
    let fait = Color::from_rgb(0.55, 0.75, 0.45);
    let pale = Color::from_rgba(1.0, 1.0, 1.0, 0.4);

    let mut colonne = vec![
        Text::new(lancement.titre.clone())
            .font(fonts.alkhemi.id)
            .size(fonts.alkhemi.scale(32))
            .into(),
    ];
    if !lancement.sous_titre.is_empty() {
        colonne.push(
            Text::new(lancement.sous_titre.clone())
                .size(fonts.cyri.scale(16))
                .color(pale)
                .into(),
        );
    }
    colonne.push(Space::new(Length::Fill, Length::Units(10)).into());
    for (etape, etat) in etats_etapes(lancement.distant, courante) {
        let (couleur, taille) = match etat {
            EtatEtape::Fait => (fait, 18),
            EtatEtape::EnCours => (clair, 24),
            EtatEtape::AVenir => (pale, 18),
        };
        colonne.push(
            Row::with_children(vec![
                Text::new(i18n.get_msg(etape.cle()).into_owned())
                    .size(fonts.cyri.scale(taille))
                    .color(couleur)
                    .width(Length::Fill)
                    .into(),
                Text::new(i18n.get_msg(etat.cle()).into_owned())
                    .size(fonts.cyri.scale(16))
                    .color(couleur)
                    .into(),
            ])
            .spacing(20)
            .align_items(Align::Center)
            .width(Length::Fill)
            .into(),
        );
    }

    let cadre = Container::new(
        Column::with_children(colonne)
            .spacing(8)
            .width(Length::Fill),
    )
    .style(
        style::container::Style::color_with_double_cornerless_border(
            (22, 18, 16, 255).into(),
            (11, 11, 11, 255).into(),
            (54, 46, 38, 255).into(),
        ),
    )
    .width(Length::Units(340))
    .padding(20);

    Container::new(cadre)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::main::DetailedInitializationStage as Stage;
    use client::ClientInitStage;

    #[test]
    fn etape_selon_le_client() {
        let monde = |s| etape_en_cours(&s, false);
        assert_eq!(monde(Stage::StartingMultiplayer), Etape::Monde);
        assert_eq!(
            monde(Stage::Client(ClientInitStage::ConnectionEstablish)),
            Etape::Monde
        );
        assert_eq!(
            monde(Stage::Client(ClientInitStage::WatingForServerVersion)),
            Etape::Connexion
        );
        assert_eq!(
            monde(Stage::Client(ClientInitStage::LoadingInitData)),
            Etape::Connexion
        );
        assert_eq!(monde(Stage::CreatingRenderPipeline(1, 3)), Etape::Connexion);
        // L'écran des personnages en attente : le village, quoi que dise le client.
        assert_eq!(
            etape_en_cours(&Stage::StartingMultiplayer, true),
            Etape::Village
        );
    }

    #[test]
    fn etats_des_etapes() {
        use EtatEtape::*;
        assert_eq!(etats_etapes(false, Etape::Monde), [
            (Etape::Serveur, Fait),
            (Etape::Monde, EnCours),
            (Etape::Connexion, AVenir),
            (Etape::Village, AVenir),
        ]);
        // Serveur distant : pas d'étape « Serveur lancé ».
        assert_eq!(etats_etapes(true, Etape::Village), [
            (Etape::Monde, Fait),
            (Etape::Connexion, Fait),
            (Etape::Village, EnCours),
        ]);
    }
}
