//! Icebreaker: the expeditions leave the village from the « Départ » house
//! (V.7 to V.9). Its leader offers by dialogue the destinations: the places of
//! the campaign (`assets/icebreaker/lieux.ron`, shared with the main menu)
//! other than the village. Going there (V.8): the village server starts the
//! place's server if needed, and tells the client to go (`IcebreakerTravel`);
//! the client leaves and connects there with the same character.

use crate::{
    client::Client,
    icebreaker_services::say,
    icebreaker_warehouse::{chunk_of, village_npc},
};
use common::{
    assets::{AssetExt, Ron},
    comp::{self, ChatType, Content, Controller, agent::TradingBehavior},
    event::NpcBuilder,
    rtsim::{Dialogue, DialogueId, DialogueKind, Response},
    uid::Uid,
};
use common_net::msg::ServerGeneral;
use serde::Deserialize;
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    process::Command,
    sync::{Mutex, MutexGuard},
    time::Duration,
};
use tracing::{info, warn};
use vek::*;

/// Where the expedition leader stands (none outside the village).
#[derive(Default)]
pub struct IcebreakerExpedition {
    leader_pos: Option<Vec3<f32>>,
    /// When the expedition was last shown to the village (`Time`).
    pub last_shown: f64,
    /// Icebreaker : ce qui a été montré au village en dernier (jalon N).
    pub montree: Option<common::icebreaker_panneau::Expedition>,
}

impl IcebreakerExpedition {
    pub fn new(leader_pos: Option<Vec3<f32>>) -> Self {
        if let Some(pos) = leader_pos {
            info!(?pos, "Icebreaker: village expedition leader");
        }
        Self {
            leader_pos,
            last_shown: 0.0,
            montree: None,
        }
    }

    pub fn leader_pos(&self) -> Option<Vec3<f32>> { self.leader_pos }

    pub fn leader_chunk(&self) -> Option<Vec2<i32>> { self.leader_pos.map(chunk_of) }

    pub fn leader(&self) -> Option<(NpcBuilder, comp::Pos)> {
        let pos = self.leader_pos?;
        let npc = village_npc(
            "Chef d'expédition".to_string(),
            pos,
            TradingBehavior::Departure,
            Vec::new(),
        );
        Some((npc, comp::Pos(pos)))
    }
}

/// The answer to leave nowhere.
pub const NOT_NOW: u16 = u16::MAX;

/// A place of the campaign (fields of `lieux.ron` used here).
#[derive(Clone, Debug, Deserialize)]
pub struct Lieu {
    pub nom: String,
    pub arguments: Vec<String>,
    pub port: u16,
    pub actif: bool,
}

/// The places of the campaign, in the order of `lieux.ron`.
pub fn lieux() -> Vec<Lieu> {
    Ron::<Vec<Lieu>>::load_expect("icebreaker.lieux")
        .read()
        .0
        .clone()
}

impl Lieu {
    /// Whether it is where expeditions leave from: the village, alone or in
    /// the campaign.
    pub fn is_home(&self) -> bool {
        self.arguments
            .first()
            .is_some_and(|lieu| lieu == "village" || lieu == "campagne")
    }

    /// Its site (`-Site`), which chooses the dungeon slot in the campaign.
    pub fn site(&self) -> Option<&str> {
        self.arguments
            .windows(2)
            .find(|w| w[0] == "-Site")
            .map(|w| w[1].as_str())
    }
}

/// The destinations of expeditions: the places other than the village, with
/// their index in `lieux.ron`.
pub fn destinations() -> Vec<(usize, Lieu)> {
    lieux()
        .into_iter()
        .enumerate()
        // Icebreaker O : le lieu d'entraînement se lance seul, on n'y part pas en expédition.
        .filter(|(_, lieu)| {
            !lieu.is_home() && lieu.arguments.first().is_none_or(|lieu| lieu != "entrainement")
        })
        .collect()
}

/// Icebreaker N.8 : le nom affiché du lieu dont le site est `site` (« Genere » : « Donjon
/// généré »), ou le site lui-même s'il n'est pas dans `lieux.ron`.
pub fn nom_du_site(site: &str) -> String {
    lieux()
        .into_iter()
        .find(|lieu| lieu.site() == Some(site))
        .map_or_else(|| site.to_string(), |lieu| lieu.nom)
}

/// The answers to the expedition leader, besides a destination (its index).
pub const JOIN: u16 = 1000;
pub const READY: u16 = 1001;
pub const WAIT: u16 = 1002;
pub const LEAVE: u16 = 1003;

/// How long the expedition waits, once everyone is ready, before leaving.
pub const DEPARTURE_DELAY: f64 = 5.0;

/// The expedition being prepared (V.9): the first player to register chooses
/// the destination, the others join; it leaves `DEPARTURE_DELAY` after all of
/// its members are ready.
#[derive(Clone, Debug, PartialEq)]
pub struct Expedition {
    /// Index in `lieux.ron`.
    pub destination: usize,
    pub members: Vec<Member>,
    /// When it leaves, once everyone is ready.
    pub departure: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub uid: Uid,
    pub alias: String,
    pub ready: bool,
}

impl Expedition {
    fn destination_name(&self) -> String {
        lieux()
            .get(self.destination)
            .map_or_else(|| "?".to_string(), |lieu| lieu.nom.clone())
    }

    fn member(&self, uid: Uid) -> Option<&Member> { self.members.iter().find(|m| m.uid == uid) }

    fn roll(&self) -> String {
        self.members
            .iter()
            .map(|m| {
                if m.ready {
                    format!("{} (prêt)", m.alias)
                } else {
                    m.alias.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Starts the countdown if everyone is ready, stops it otherwise; what to
    /// tell everyone.
    fn update_departure(&mut self, now: f64, announce: &mut Vec<String>) {
        let all_ready = !self.members.is_empty() && self.members.iter().all(|m| m.ready);
        match (all_ready, self.departure) {
            (true, None) => {
                self.departure = Some(now + DEPARTURE_DELAY);
                announce.push(format!(
                    "Tous prêts : départ vers {} dans {DEPARTURE_DELAY} s !",
                    self.destination_name()
                ));
            },
            (false, Some(_)) => {
                self.departure = None;
                announce.push("Départ suspendu : tout le monde n'est pas prêt.".to_string());
            },
            _ => {},
        }
    }
}

static EXPEDITION: Mutex<Option<Expedition>> = Mutex::new(None);

fn expedition() -> MutexGuard<'static, Option<Expedition>> {
    EXPEDITION.lock().unwrap_or_else(|e| e.into_inner())
}

/// On talk: asks `player` where the expedition goes, whether to join it, or
/// whether they are ready.
pub fn offer(controller: &mut Controller, player: Uid) {
    let (msg, responses) = question(expedition().as_ref(), player);
    let id = DialogueId(rand::random());
    say(controller, player, id, DialogueKind::Start);
    say(controller, player, id, DialogueKind::Question {
        tag: 0,
        msg: Content::Plain(msg),
        responses: responses
            .into_iter()
            .map(|(id, text)| (id, Response::from(Content::Plain(text))))
            .collect(),
    });
}

/// The question for `player`, and its answers.
fn question(expedition: Option<&Expedition>, player: Uid) -> (String, Vec<(u16, String)>) {
    let not_now = (NOT_NOW, "Pas maintenant".to_string());
    match expedition {
        None => {
            let mut answers = destinations()
                .into_iter()
                .map(|(index, lieu)| {
                    let text = if lieu.actif {
                        lieu.nom
                    } else {
                        format!("{} (pas encore ouvert)", lieu.nom)
                    };
                    (index as u16, text)
                })
                .collect::<Vec<_>>();
            answers.push(not_now);
            ("Je prépare les expéditions. Où veux-tu partir ?".to_string(), answers)
        },
        Some(exp) => match exp.member(player) {
            None => (
                format!(
                    "Une expédition se prépare vers {} (inscrits : {}). La rejoindre ?",
                    exp.destination_name(),
                    exp.roll()
                ),
                vec![(JOIN, "Rejoindre l'expédition".to_string()), not_now],
            ),
            Some(member) => (
                format!(
                    "Expédition vers {} (inscrits : {}).",
                    exp.destination_name(),
                    exp.roll()
                ),
                vec![
                    if member.ready {
                        (WAIT, "Attendre encore".to_string())
                    } else {
                        (READY, "Je suis prêt".to_string())
                    },
                    (LEAVE, "Me désinscrire".to_string()),
                    (NOT_NOW, "Rien".to_string()),
                ],
            ),
        },
    }
}

/// On the player's answer, the dialogue ends (the dialogue event applied the
/// answer: [`answer`]).
pub fn close(controller: &mut Controller, player: Uid, dialogue: &Dialogue<true>) {
    if matches!(dialogue.kind, DialogueKind::Response { .. }) {
        say(controller, player, dialogue.id, DialogueKind::End);
    }
}

/// What an answer to the expedition leader says: to the player, and to
/// everyone.
#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    pub reply: Option<String>,
    pub announce: Vec<String>,
}

/// The command word of an answer to the expedition leader (`/expedition`), and
/// back.
fn answer_word(id: u16) -> String {
    match id {
        JOIN => "rejoindre".to_string(),
        READY => "pret".to_string(),
        WAIT => "attendre".to_string(),
        LEAVE => "quitter".to_string(),
        index => index.to_string(),
    }
}

fn answer_of_word(word: &str) -> Option<u16> {
    match word {
        "rejoindre" => Some(JOIN),
        "pret" | "prêt" => Some(READY),
        "attendre" => Some(WAIT),
        "quitter" => Some(LEAVE),
        index => index.parse().ok().filter(|index| *index < JOIN),
    }
}

/// Icebreaker : le panneau typé « Départ » du plan (jalon N) pour `player`
/// (`fantome` : sans personnage) : les destinations tant qu'il n'y a pas
/// d'expédition, sinon ses inscrits, qui est prêt, et ce que le joueur peut
/// faire.
pub fn panneau_pour(player: Uid, fantome: bool, now: f64) -> common::icebreaker_panneau::Panneau {
    panneau_de(expedition().as_ref(), player, fantome, now)
}

fn panneau_de(
    expedition: Option<&Expedition>,
    player: Uid,
    fantome: bool,
    now: f64,
) -> common::icebreaker_panneau::Panneau {
    use common::icebreaker_panneau::{Action, Chiffre, Ligne, Panneau, Sorte, Ton};
    let refus = crate::icebreaker_registre::GHOST_REFUSAL;
    let Some(exp) = expedition else {
        let mut panneau = Panneau::nouveau("globe", "Départ", "Où part l'expédition ?");
        panneau.lignes = destinations()
            .into_iter()
            .map(|(index, lieu)| {
                let ligne = Ligne::nouvelle(lieu.nom.clone()).icone("globe").action(
                    Action::nouvelle("Choisir", &["expedition", &answer_word(index as u16)])
                        .principale()
                        .permise(lieu.actif && !fantome),
                );
                if lieu.actif {
                    ligne
                } else {
                    ligne.sous_titre("pas encore ouvert").estompee()
                }
            })
            .collect();
        return if fantome {
            panneau.message(refus, Sorte::Refus)
        } else {
            panneau.message("Personne n'est inscrit pour l'instant.", Sorte::Info)
        };
    };
    let prets = exp.members.iter().filter(|m| m.ready).count();
    let reste = exp.departure.map(|departure| (departure - now).max(0.0).ceil());
    let mut panneau = Panneau::nouveau(
        "globe",
        "Départ",
        format!("Expédition vers : {}.", exp.destination_name()),
    );
    panneau.chiffres = vec![
        Chiffre::nouveau(
            format!("{prets} / {}", exp.members.len()),
            "prêts",
            if prets == exp.members.len() {
                Ton::Bon
            } else {
                Ton::Normal
            },
        ),
        match reste {
            Some(reste) => Chiffre::nouveau(format!("{reste:.0} s"), "avant le départ", Ton::Or),
            None => Chiffre::nouveau("en attente", "avant le départ", Ton::Discret),
        },
    ];
    panneau.lignes = exp
        .members
        .iter()
        .map(|m| {
            let ligne = if m.ready {
                Ligne::nouvelle(m.alias.clone()).pastille("prêt", Ton::Bon)
            } else {
                Ligne::nouvelle(m.alias.clone()).pastille("pas prêt", Ton::Discret)
            };
            if m.uid == player { ligne.choisie() } else { ligne }
        })
        .collect();
    match exp.member(player) {
        None => {
            panneau.actions = vec![
                Action::nouvelle("Rejoindre l'expédition", &["expedition", "rejoindre"])
                    .principale()
                    .permise(!fantome),
            ];
            if fantome {
                return panneau.message(refus, Sorte::Refus);
            }
        },
        Some(member) => {
            panneau.colonnes = 2;
            panneau.actions = vec![
                if member.ready {
                    Action::nouvelle("Attendre encore", &["expedition", "attendre"])
                } else {
                    Action::nouvelle("Je suis prêt", &["expedition", "pret"]).principale()
                },
                Action::nouvelle("Me désinscrire", &["expedition", "quitter"]),
            ];
        },
    }
    match reste {
        Some(reste) => panneau.message(
            format!("Tous prêts : départ dans {reste:.0} s."),
            Sorte::Reussite,
        ),
        None => panneau.message("Le départ se lance quand tout le monde est prêt.", Sorte::Info),
    }
}

/// Icebreaker : l'expédition en préparation, pour le bandeau du village (jalon
/// N) : sa destination, le temps avant le départ, ses inscrits (pseudo, prêt).
pub fn resume(now: f64) -> Option<common::icebreaker_panneau::Expedition> {
    expedition()
        .as_ref()
        .map(|exp| common::icebreaker_panneau::Expedition {
            destination: exp.destination_name(),
            depart_dans: exp.departure.map(|departure| (departure - now).max(0.0) as f32),
            membres: exp.members.iter().map(|m| (m.alias.clone(), m.ready)).collect(),
        })
}

/// `/expedition <choix>` (the « Départ » panel, V.11): a destination number,
/// `rejoindre`, `pret`, `attendre` or `quitter` — as the answers to the
/// expedition leader.
pub(crate) fn handle_expedition(
    server: &mut crate::Server,
    client: specs::Entity,
    _target: specs::Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), Content> {
    use specs::WorldExt;
    let ecs = server.state.ecs();
    if ecs.read_resource::<IcebreakerExpedition>().leader_pos.is_none() {
        return Err(Content::Plain("Les expéditions partent du village.".to_string()));
    }
    if crate::icebreaker_registre::is_ghost(ecs, client) {
        return Err(Content::Plain(crate::icebreaker_registre::GHOST_REFUSAL.to_string()));
    }
    let Some(id) = args.first().and_then(|word| answer_of_word(word)) else {
        return Err(Content::Plain(
            "/expedition <numéro de destination>, rejoindre, pret, attendre ou quitter".to_string(),
        ));
    };
    let (Some(uid), alias) = (
        ecs.read_storage::<Uid>().get(client).copied(),
        ecs.read_storage::<comp::Player>()
            .get(client)
            .map_or_else(|| "Quelqu'un".to_string(), |player| player.alias.clone()),
    ) else {
        return Err(Content::Plain("Il faut être en jeu.".to_string()));
    };
    let now = ecs.read_resource::<common::resources::Time>().0;
    let outcome = answer(uid, &alias, id, now);
    if let Some(reply) = outcome.reply {
        server.notify_client(client, ServerGeneral::server_msg(ChatType::Meta, Content::Plain(reply)));
    }
    for line in outcome.announce {
        server.notify_players(ServerGeneral::server_msg(ChatType::Meta, Content::Plain(line)));
    }
    Ok(())
}

/// Applies the answer `response_id` of `player` (`alias`) at time `now`.
pub fn answer(player: Uid, alias: &str, response_id: u16, now: f64) -> Outcome {
    answer_to(&mut expedition(), player, alias, response_id, now)
}

fn answer_to(
    expedition: &mut Option<Expedition>,
    player: Uid,
    alias: &str,
    response_id: u16,
    now: f64,
) -> Outcome {
    let mut outcome = Outcome::default();
    match (expedition.as_mut(), response_id) {
        (_, NOT_NOW) => {},
        (None, index) => match destinations()
            .into_iter()
            .find(|(i, _)| *i == usize::from(index))
            .map(|(_, lieu)| lieu)
            .as_ref()
        {
            Some(lieu) if !lieu.actif => {
                outcome.reply = Some(format!("{} n'est pas encore ouvert.", lieu.nom));
            },
            Some(lieu) => {
                outcome.reply = Some(format!(
                    "Tu es inscrit pour {}. Reviens me dire quand tu es prêt.",
                    lieu.nom
                ));
                outcome.announce.push(format!(
                    "{alias} prépare une expédition vers {} : parlez au chef d'expédition \
                     (maison « Départ ») pour la rejoindre.",
                    lieu.nom
                ));
                *expedition = Some(Expedition {
                    destination: usize::from(index),
                    members: vec![Member {
                        uid: player,
                        alias: alias.to_string(),
                        ready: false,
                    }],
                    departure: None,
                });
            },
            None => outcome.reply = Some("Ce n'est pas une destination d'expédition.".to_string()),
        },
        (Some(exp), JOIN) if exp.member(player).is_none() => {
            exp.members.push(Member {
                uid: player,
                alias: alias.to_string(),
                ready: false,
            });
            outcome.announce.push(format!(
                "{alias} rejoint l'expédition vers {}.",
                exp.destination_name()
            ));
            exp.update_departure(now, &mut outcome.announce);
        },
        (Some(exp), READY | WAIT) => {
            if let Some(member) = exp.members.iter_mut().find(|m| m.uid == player) {
                member.ready = response_id == READY;
                outcome.announce.push(if member.ready {
                    format!("{alias} est prêt.")
                } else {
                    format!("{alias} attend encore.")
                });
                exp.update_departure(now, &mut outcome.announce);
            }
        },
        (Some(exp), LEAVE) => {
            exp.members.retain(|m| m.uid != player);
            outcome.announce.push(format!("{alias} quitte l'expédition."));
            if exp.members.is_empty() {
                outcome.announce.push("Expédition annulée.".to_string());
                *expedition = None;
            } else {
                exp.update_departure(now, &mut outcome.announce);
            }
        },
        // A destination while an expedition is being prepared (the question
        // was asked before), or joining twice.
        (Some(exp), _) => {
            outcome.reply = Some(format!(
                "Une expédition vers {} se prépare déjà : parle-moi de nouveau.",
                exp.destination_name()
            ));
        },
    }
    outcome
}

/// Each tick: members gone (`present` false: disconnected, or too far from
/// the leader) leave the expedition; once its time has come, it leaves if its
/// destination `has_room` (a free dungeon slot, in the campaign): returns
/// where to, and who. Also what to tell everyone.
pub fn tick(
    now: f64,
    present: impl Fn(Uid) -> bool,
    has_room: impl Fn(&Lieu) -> bool,
) -> (Option<(Lieu, Vec<Uid>)>, Vec<String>) {
    tick_on(&mut expedition(), now, present, has_room)
}

fn tick_on(
    expedition: &mut Option<Expedition>,
    now: f64,
    present: impl Fn(Uid) -> bool,
    has_room: impl Fn(&Lieu) -> bool,
) -> (Option<(Lieu, Vec<Uid>)>, Vec<String>) {
    let mut announce = Vec::new();
    let Some(exp) = expedition.as_mut() else {
        return (None, announce);
    };
    let before = exp.members.len();
    exp.members.retain(|m| {
        let stays = present(m.uid);
        if !stays {
            announce.push(format!("{} quitte l'expédition (parti ou trop loin).", m.alias));
        }
        stays
    });
    if exp.members.is_empty() {
        announce.push("Expédition annulée.".to_string());
        *expedition = None;
        return (None, announce);
    }
    if exp.members.len() != before {
        exp.update_departure(now, &mut announce);
    }
    if exp.departure.is_some_and(|departure| now >= departure) {
        // No room there: everyone waits and says again when ready.
        if let Some(lieu) = lieux().get(exp.destination)
            && !has_room(lieu)
        {
            announce.push(format!(
                "Pas de place libre à {} : les autres expéditions y sont encore. Redites « Je suis \
                 prêt » plus tard.",
                lieu.nom
            ));
            exp.departure = None;
            exp.members.iter_mut().for_each(|m| m.ready = false);
            return (None, announce);
        }
        let exp = expedition.take().expect("checked above");
        let lieu = lieux().get(exp.destination).cloned();
        if let Some(lieu) = lieu {
            announce.push(format!("L'expédition part vers {} !", lieu.nom));
            return (
                Some((lieu, exp.members.into_iter().map(|m| m.uid).collect())),
                announce,
            );
        }
    }
    (None, announce)
}

/// The village (or the campaign), where `/rentrer` leads from a place run
/// alone.
pub fn village() -> Option<Lieu> {
    lieux()
        .into_iter()
        .find(Lieu::is_home)
}

/// Starts the server of `lieu` if nothing listens on its port yet, from the
/// root of the repository (where this server runs): `icebreaker.ps1` in a
/// console of its own on Windows, `icebreaker.sh` in the background elsewhere.
pub fn ensure_running(lieu: &Lieu) -> Result<(), String> {
    let local = SocketAddr::from((Ipv4Addr::LOCALHOST, lieu.port));
    if TcpStream::connect_timeout(&local, Duration::from_millis(300)).is_ok() {
        return Ok(());
    }
    #[cfg(windows)]
    let mut command = {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        let mut command = Command::new("powershell");
        command
            .args(["-NoExit", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg("icebreaker.ps1")
            .args(&lieu.arguments)
            .args(["-Port", &lieu.port.to_string()])
            .creation_flags(CREATE_NEW_CONSOLE);
        command
    };
    #[cfg(not(windows))]
    let mut command = {
        // `icebreaker.sh <lieu>`, the site and port by environment variables.
        let mut command = Command::new("sh");
        command
            .arg("icebreaker.sh")
            .args(lieu.arguments.first())
            .env("ICEBREAKER_PORT", lieu.port.to_string());
        if let Some(site) = lieu.site() {
            command.env("ICEBREAKER_SITE", site);
        }
        command
    };
    command
        .spawn()
        .map(|_| info!(lieu = %lieu.nom, port = lieu.port, "Icebreaker: place server started"))
        .map_err(|err| format!("Icebreaker : impossible de lancer {} : {err}", lieu.nom))
}

/// Sends the player of `client` to `lieu`, starting its server if needed.
pub fn travel(client: &Client, lieu: &Lieu) {
    match ensure_running(lieu) {
        Ok(()) => client.send_fallible(ServerGeneral::IcebreakerTravel(lieu.port)),
        Err(err) => {
            warn!(%err, "Icebreaker: no travel");
            client.send_fallible(ServerGeneral::server_msg(ChatType::Meta, Content::Plain(err)));
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_destinations_are_the_places_but_the_village() {
        let destinations = destinations();
        assert!(!destinations.is_empty());
        assert!(
            destinations
                .iter()
                .all(|(index, place)| place.nom == lieux()[*index].nom
                    && !place.is_home())
        );
        assert!(village().is_some_and(|village| village.actif));
    }

    #[test]
    fn le_bilan_nomme_le_lieu_comme_le_depart() {
        assert_eq!(nom_du_site("Genere"), "Donjon généré");
        assert_eq!(nom_du_site("Grotte"), "Grotte");
        // Un site qui n'est pas dans `lieux.ron` garde son nom.
        assert_eq!(nom_du_site("Ailleurs"), "Ailleurs");
    }

    #[test]
    fn a_place_already_listening_is_not_started_again() {
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let lieu = Lieu {
            nom: "Essai".to_string(),
            // Would fail to start: nothing is started.
            arguments: vec!["nulle-part".to_string()],
            port: listener.local_addr().unwrap().port(),
            actif: true,
        };
        assert_eq!(ensure_running(&lieu), Ok(()));
    }

    #[test]
    fn two_players_leave_together_once_both_are_ready() {
        let uid = |n| Uid(std::num::NonZeroU64::new(n).unwrap());
        let (flo, sayanel) = (uid(1), uid(2));
        let (donjon, _) = destinations()
            .into_iter()
            .find(|(_, lieu)| lieu.actif)
            .unwrap();
        let donjon = donjon as u16;
        let mut exp = None;

        // Flo chooses the destination, Sayanel joins.
        let outcome = answer_to(&mut exp, flo, "Flo", donjon, 0.0);
        assert!(outcome.reply.is_some() && outcome.announce.len() == 1);
        assert_eq!(question(exp.as_ref(), sayanel).1[0].0, JOIN);
        answer_to(&mut exp, sayanel, "Sayanel", JOIN, 1.0);
        assert_eq!(exp.as_ref().unwrap().members.len(), 2);

        // Flo is ready: no departure yet.
        answer_to(&mut exp, flo, "Flo", READY, 2.0);
        assert_eq!(exp.as_ref().unwrap().departure, None);
        assert_eq!(question(exp.as_ref(), flo).1[0].0, WAIT);

        // Sayanel is ready: departure in 5 s; Sayanel waits: suspended; ready again.
        answer_to(&mut exp, sayanel, "Sayanel", READY, 3.0);
        assert_eq!(exp.as_ref().unwrap().departure, Some(3.0 + DEPARTURE_DELAY));
        answer_to(&mut exp, sayanel, "Sayanel", WAIT, 4.0);
        assert_eq!(exp.as_ref().unwrap().departure, None);
        answer_to(&mut exp, sayanel, "Sayanel", READY, 5.0);

        // Not yet time; then both leave.
        assert!(tick_on(&mut exp, 6.0, |_| true, |_| true).0.is_none());
        // No room there: nobody leaves, everyone is to be ready again.
        let (departure, _) = tick_on(&mut exp, 5.0 + DEPARTURE_DELAY, |_| true, |_| false);
        assert!(departure.is_none());
        assert!(exp.as_ref().unwrap().members.iter().all(|m| !m.ready));
        answer_to(&mut exp, flo, "Flo", READY, 20.0);
        answer_to(&mut exp, sayanel, "Sayanel", READY, 20.0);
        let (departure, _) = tick_on(&mut exp, 20.0 + DEPARTURE_DELAY, |_| true, |_| true);
        let (lieu, members) = departure.unwrap();
        assert_eq!(lieu.nom, lieux()[usize::from(donjon)].nom);
        assert_eq!(members, vec![flo, sayanel]);
        assert_eq!(exp, None);
    }

    #[test]
    fn the_departure_panel_offers_what_the_player_may_do() {
        use common::icebreaker_panneau::{Sorte, Ton, texte};
        let uid = |n| Uid(std::num::NonZeroU64::new(n).unwrap());
        let (flo, sayanel) = (uid(1), uid(2));
        let (donjon, _) = destinations()
            .into_iter()
            .find(|(_, lieu)| lieu.actif)
            .expect("an open destination");
        let commandes = |panneau: &common::icebreaker_panneau::Panneau| {
            panneau
                .actions
                .iter()
                .map(|action| (action.commande.join(" "), action.principale, action.permise))
                .collect::<Vec<_>>()
        };

        // No expedition: the destinations, one line each; a ghost may not choose.
        let libre = panneau_de(None, flo, false, 0.0);
        assert_eq!(libre.lignes.len(), destinations().len());
        assert!(libre.lignes.iter().any(|l| l.action.as_ref().is_some_and(|a| a.permise)));
        assert!(
            libre
                .lignes
                .iter()
                .filter(|l| l.estompee)
                .all(|l| l.action.as_ref().is_some_and(|a| !a.permise)),
            "a closed destination is greyed"
        );
        let fantome = panneau_de(None, flo, true, 0.0);
        assert!(fantome.lignes.iter().all(|l| l.action.as_ref().is_some_and(|a| !a.permise)));
        assert_eq!(fantome.message.map(|(_, sorte)| sorte), Some(Sorte::Refus));

        // Sayanel is in and ready, Flo is not in yet: she may join.
        let mut exp = Expedition {
            destination: donjon,
            members: vec![Member {
                uid: sayanel,
                alias: "Sayanel".into(),
                ready: true,
            }],
            departure: None,
        };
        let dehors = panneau_de(Some(&exp), flo, false, 0.0);
        assert_eq!(commandes(&dehors), vec![("expedition rejoindre".to_string(), true, true)]);

        // Flo in, not ready: « Je suis prêt » is the main action, nobody leaves yet.
        exp.members.push(Member {
            uid: flo,
            alias: "Flo".into(),
            ready: false,
        });
        let attente = panneau_de(Some(&exp), flo, false, 0.0);
        assert_eq!(commandes(&attente), vec![
            ("expedition pret".to_string(), true, true),
            ("expedition quitter".to_string(), false, true)
        ]);
        assert_eq!(attente.chiffres[0].valeur, texte("1 / 2"));
        assert_eq!(attente.chiffres[1].valeur, texte("en attente"));
        assert!(attente.lignes[1].choisie && !attente.lignes[0].choisie, "her own line");
        assert_eq!(attente.lignes[0].pastille, Some((texte("prêt"), Ton::Bon)));

        // Everyone ready, leaving at 5 s: the countdown, and waiting is offered.
        exp.members[1].ready = true;
        exp.departure = Some(5.0);
        let depart = panneau_de(Some(&exp), flo, false, 1.2);
        assert_eq!(commandes(&depart)[0], ("expedition attendre".to_string(), false, true));
        assert_eq!(depart.chiffres[1].valeur, texte("4 s"));
        assert_eq!(depart.message, Some((texte("Tous prêts : départ dans 4 s."), Sorte::Reussite)));
    }

    #[test]
    fn the_panel_buttons_send_the_answers() {
        for id in [JOIN, READY, WAIT, LEAVE, 2] {
            assert_eq!(answer_of_word(&answer_word(id)), Some(id));
        }
        assert_eq!(answer_of_word("n'importe"), None);
        assert_eq!(answer_of_word("1500"), None);
        // The village is no destination.
        let mut exp = None;
        let home = lieux().iter().position(Lieu::is_home).unwrap() as u16;
        let uid = Uid(std::num::NonZeroU64::new(1).unwrap());
        answer_to(&mut exp, uid, "Flo", home, 0.0);
        assert_eq!(exp, None);
    }

    #[test]
    fn a_member_gone_leaves_and_the_last_one_cancels() {
        let uid = |n| Uid(std::num::NonZeroU64::new(n).unwrap());
        let (flo, sayanel) = (uid(1), uid(2));
        let (donjon, _) = destinations()
            .into_iter()
            .find(|(_, lieu)| lieu.actif)
            .unwrap();
        let mut exp = None;
        answer_to(&mut exp, flo, "Flo", donjon as u16, 0.0);
        answer_to(&mut exp, sayanel, "Sayanel", JOIN, 0.0);
        answer_to(&mut exp, flo, "Flo", READY, 0.0);

        // Sayanel, not ready, goes away: Flo, ready, is left alone and leaves.
        let (_, announce) = tick_on(&mut exp, 1.0, |uid| uid == flo, |_| true);
        assert!(announce.iter().any(|line| line.contains("Sayanel")));
        assert_eq!(exp.as_ref().unwrap().departure, Some(1.0 + DEPARTURE_DELAY));

        // Flo leaves the expedition: cancelled.
        answer_to(&mut exp, flo, "Flo", LEAVE, 2.0);
        assert_eq!(exp, None);
    }
}
