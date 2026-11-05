//! Icebreaker: the cart and the common registry of the campaign (jalon R,
//! decision 0017). With the registry open (`ICEBREAKER_REGISTRE=1`, set by
//! `icebreaker.ps1` for the campaign and the village), everyone arrives at the
//! village as a ghost (Veloren's spectator) and takes a recruit at the cart.
//!
//! A recruit taken joins the registry: a Veloren character saved under a
//! fixed account (`REGISTRE_UUID`), which any player may play, one at a time,
//! unless someone reserved it. Who plays whom is not saved: it is read from the
//! players in game. The cart, the registry and the reservations are saved in
//! the campaign directory; the settings are in `assets/icebreaker/registre.ron`.

use crate::{icebreaker_classes, icebreaker_warehouse::write_campaign_file};
use common::{
    assets::{AssetExt, Ron},
    character::CharacterId,
    comp::{ItemKey, Player, Presence, PresenceKind},
};
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};
use specs::{Entity, Join, ReadStorage, World, WorldExt};
use std::{collections::HashMap, path::PathBuf};
use tracing::{info, warn};

/// The account the characters of the registry are saved under.
pub const REGISTRE_UUID: &str = "icebreaker-registre";

/// File of the campaign directory holding the cart and the registry.
const REGISTRE_FILE: &str = "registre.ron";

/// What a ghost is told when it tries what needs a body.
pub const GHOST_REFUSAL: &str =
    "Un fantôme ne peut pas faire ça : engagez une recrue à la charrette.";

/// Whether the registry of the campaign is open.
pub fn active() -> bool { std::env::var_os("ICEBREAKER_REGISTRE").is_some_and(|v| v == "1") }

/// Whether `entity` is a ghost: in game without a character.
pub fn is_ghost(ecs: &World, entity: Entity) -> bool {
    ecs.read_storage::<Presence>()
        .get(entity)
        .is_some_and(|presence| matches!(presence.kind, PresenceKind::Spectator))
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegistreConfig {
    /// Recruits offered each new day of map time.
    pub recrues_par_jour: usize,
    /// The cart is filled up to this on each return from a dungeon.
    pub minimum_charrette: usize,
    /// Characters in the registry, at most.
    pub places_registre: usize,
    /// Characters an account may reserve.
    pub reservations_par_compte: usize,
    /// Names of the recruits.
    pub noms: Vec<String>,
    /// Where the recruits of the day stand, in front of the cart (X.8): offsets
    /// from the cart post, in blocks.
    #[serde(default)]
    pub postes_recrues: Vec<(f32, f32)>,
}

pub fn config() -> RegistreConfig {
    Ron::<RegistreConfig>::load_expect("icebreaker.registre")
        .read()
        .0
        .clone()
}

/// A recruit waiting in the cart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recrue {
    pub nom: String,
    /// Id of its class (`classes.ron`).
    pub classe: String,
}

/// A character of the registry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fiche {
    /// Its Veloren character id.
    pub id: i64,
    pub nom: String,
    pub classe: String,
    /// The account (uuid) that reserved it, if any.
    #[serde(default)]
    pub reserve_par: Option<String>,
}

/// What is saved.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Etat {
    charrette: Vec<Recrue>,
    personnages: Vec<Fiche>,
    /// The day of map time the cart was last renewed.
    jour: Option<i64>,
}

#[derive(Default)]
pub struct IcebreakerRegistre {
    path: Option<PathBuf>,
    etat: Etat,
    /// Recruits being saved as characters (the database answers later), per
    /// player who took them.
    en_creation: HashMap<Entity, Recrue>,
}

/// The day of map time at map time `time_of_day` (seconds).
pub fn jour_de(time_of_day: f64) -> i64 { (time_of_day / 86400.0).floor() as i64 }

/// The class name of `id`, for the players.
fn nom_de_classe(id: &str) -> String {
    icebreaker_classes::class(id).map_or_else(|| id.to_string(), |class| class.nom)
}

impl IcebreakerRegistre {
    /// The registry kept in `campaign_dir`.
    pub fn new(campaign_dir: Option<PathBuf>) -> Self {
        let path = campaign_dir.map(|dir| dir.join(REGISTRE_FILE));
        let etat = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|text| {
                ron::from_str::<Etat>(&text).unwrap_or_else(|err| {
                    warn!(?err, "Icebreaker: unreadable registry, starting empty");
                    Etat::default()
                })
            })
            .unwrap_or_default();
        if path.is_some() && active() {
            info!(
                charrette = etat.charrette.len(),
                personnages = etat.personnages.len(),
                "Icebreaker: registry"
            );
        }
        Self {
            path,
            etat,
            en_creation: HashMap::new(),
        }
    }

    fn save(&self) {
        if let Some(path) = &self.path
            && let Err(err) = write_campaign_file(path, &self.etat)
        {
            warn!(?err, "Icebreaker: could not save the registry");
        }
    }

    pub fn charrette(&self) -> &[Recrue] { &self.etat.charrette }

    pub fn personnages(&self) -> &[Fiche] { &self.etat.personnages }

    pub fn fiche(&self, id: CharacterId) -> Option<&Fiche> {
        self.etat.personnages.iter().find(|fiche| fiche.id == id.0)
    }

    /// A new recruit: a class and a name not taken yet.
    fn tirer(&self, config: &RegistreConfig, rng: &mut impl rand::Rng) -> Recrue {
        let classes = icebreaker_classes::classes();
        let classe = classes
            .choose(rng)
            .map_or_else(|| "guerrier".to_string(), |class| class.id.clone());
        let pris = |nom: &str| {
            self.etat.charrette.iter().any(|r| r.nom == nom)
                || self.etat.personnages.iter().any(|f| f.nom == nom)
                || self.en_creation.values().any(|r| r.nom == nom)
        };
        let libres: Vec<&String> = config.noms.iter().filter(|nom| !pris(nom)).collect();
        let nom = libres.choose(rng).map(|nom| nom.to_string()).unwrap_or_else(|| {
            let base = config.noms.choose(rng).map_or("Recrue", String::as_str);
            (2..)
                .map(|n| format!("{base} {n}"))
                .find(|nom| !pris(nom))
                .unwrap_or_default()
        });
        Recrue { nom, classe }
    }

    /// A new cart on each new day of map time; whether it was renewed.
    pub fn renouveler(&mut self, jour: i64, config: &RegistreConfig, rng: &mut impl rand::Rng) -> bool {
        if self.etat.jour == Some(jour) {
            return false;
        }
        self.etat.jour = Some(jour);
        self.etat.charrette.clear();
        for _ in 0..config.recrues_par_jour {
            let recrue = self.tirer(config, rng);
            self.etat.charrette.push(recrue);
        }
        self.save();
        true
    }

    /// The cart filled up to its minimum (on a return from a dungeon); whether
    /// recruits came.
    pub fn completer(&mut self, config: &RegistreConfig, rng: &mut impl rand::Rng) -> bool {
        let before = self.etat.charrette.len();
        while self.etat.charrette.len() < config.minimum_charrette {
            let recrue = self.tirer(config, rng);
            self.etat.charrette.push(recrue);
        }
        let came = self.etat.charrette.len() > before;
        if came {
            self.save();
        }
        came
    }

    /// Whether the account `uuid` has someone to take: a recruit, or a
    /// character of the registry free for it (`joueurs`: who plays whom).
    pub fn de_quoi_jouer(&self, uuid: &str, joueurs: &Joueurs) -> bool {
        !self.etat.charrette.is_empty()
            || self
                .etat
                .personnages
                .iter()
                .any(|fiche| self.peut_jouer_fiche(fiche, uuid, joueurs).is_ok())
    }

    /// Takes the recruit at `index` of the cart for the player `entity`, until
    /// it is saved as a character (`cree`).
    pub fn engager(
        &mut self,
        index: usize,
        entity: Entity,
        config: &RegistreConfig,
    ) -> Result<Recrue, String> {
        if self.en_creation.contains_key(&entity) {
            return Err("Votre recrue arrive.".to_string());
        }
        if self.etat.personnages.len() + self.en_creation.len() >= config.places_registre {
            return Err(format!(
                "Le registre est plein ({} personnages).",
                config.places_registre
            ));
        }
        if index >= self.etat.charrette.len() {
            return Err("Cette recrue n'est plus à la charrette.".to_string());
        }
        let recrue = self.etat.charrette.remove(index);
        self.en_creation.insert(entity, recrue.clone());
        self.save();
        Ok(recrue)
    }

    /// The recruit taken by `entity` is saved as the character `id`: it joins
    /// the registry.
    pub fn cree(&mut self, entity: Entity, id: CharacterId) -> Option<Fiche> {
        let recrue = self.en_creation.remove(&entity)?;
        let fiche = Fiche {
            id: id.0,
            nom: recrue.nom,
            classe: recrue.classe,
            reserve_par: None,
        };
        self.etat.personnages.push(fiche.clone());
        self.save();
        Some(fiche)
    }

    /// The recruit taken by `entity` could not be saved: back to the cart.
    pub fn echec(&mut self, entity: Entity) -> bool {
        let Some(recrue) = self.en_creation.remove(&entity) else {
            return false;
        };
        self.etat.charrette.push(recrue);
        self.save();
        true
    }

    fn peut_jouer_fiche(&self, fiche: &Fiche, uuid: &str, joueurs: &Joueurs) -> Result<(), String> {
        if let Some(owner) = &fiche.reserve_par
            && owner != uuid
        {
            return Err(format!("{} est réservé par un autre joueur.", fiche.nom));
        }
        if let Some(joueur) = joueurs.get(&fiche.id)
            && joueur.uuid != uuid
        {
            return Err(format!("{} est déjà joué par {}.", fiche.nom, joueur.alias));
        }
        Ok(())
    }

    /// Whether the account `uuid` may play the character `id` of the registry.
    pub fn peut_jouer(&self, id: CharacterId, uuid: &str, joueurs: &Joueurs) -> Result<(), String> {
        let fiche = self
            .fiche(id)
            .ok_or_else(|| "Ce personnage n'est pas au registre.".to_string())?;
        self.peut_jouer_fiche(fiche, uuid, joueurs)
    }

    /// Reserves the character `id` for the account `uuid`.
    pub fn reserver(&mut self, id: CharacterId, uuid: &str, config: &RegistreConfig) -> Result<String, String> {
        let deja = self
            .etat
            .personnages
            .iter()
            .filter(|fiche| fiche.reserve_par.as_deref() == Some(uuid))
            .count();
        let fiche = self
            .etat
            .personnages
            .iter_mut()
            .find(|fiche| fiche.id == id.0)
            .ok_or_else(|| "Ce personnage n'est pas au registre.".to_string())?;
        match fiche.reserve_par.as_deref() {
            Some(owner) if owner == uuid => return Ok(format!("{} est déjà à vous.", fiche.nom)),
            Some(_) => return Err(format!("{} est réservé par un autre joueur.", fiche.nom)),
            None => {},
        }
        if deja >= config.reservations_par_compte {
            return Err(format!(
                "Vous avez déjà {} personnages réservés : libérez-en un.",
                config.reservations_par_compte
            ));
        }
        fiche.reserve_par = Some(uuid.to_string());
        let nom = fiche.nom.clone();
        self.save();
        Ok(format!("{nom} est réservé pour vous."))
    }

    /// Gives back to everyone the character `id` reserved by `uuid`.
    pub fn liberer(&mut self, id: CharacterId, uuid: &str) -> Result<String, String> {
        let fiche = self
            .etat
            .personnages
            .iter_mut()
            .find(|fiche| fiche.id == id.0 && fiche.reserve_par.as_deref() == Some(uuid))
            .ok_or_else(|| "Vous n'avez pas réservé ce personnage.".to_string())?;
        fiche.reserve_par = None;
        let nom = fiche.nom.clone();
        self.save();
        Ok(format!("{nom} est de nouveau à tout le monde."))
    }

    /// Icebreaker : le personnage `id`, mort en expédition, quitte le registre pour de bon
    /// (T.6) ; sa fiche, s'il en avait une.
    pub fn retirer(&mut self, id: CharacterId) -> Option<Fiche> {
        let index = self.etat.personnages.iter().position(|fiche| fiche.id == id.0)?;
        let fiche = self.etat.personnages.remove(index);
        self.save();
        Some(fiche)
    }
}

/// Who plays a character: account and name.
#[derive(Clone, Debug, PartialEq)]
pub struct Joueur {
    pub uuid: String,
    pub alias: String,
}

/// Who plays which character (by id).
pub type Joueurs = HashMap<i64, Joueur>;

/// Who plays which character, from the players in game.
pub fn joueurs(presences: &ReadStorage<Presence>, players: &ReadStorage<Player>) -> Joueurs {
    (presences, players)
        .join()
        .filter_map(|(presence, player)| match presence.kind {
            PresenceKind::Character(id) | PresenceKind::LoadingCharacter(id) => Some((id.0, Joueur {
                uuid: player.uuid().to_string(),
                alias: player.alias.clone(),
            })),
            _ => None,
        })
        .collect()
}

/// The cart's panel for the account `uuid` playing `current` (none: a ghost):
/// lines, and buttons (label, chat command).
pub fn panel_for(
    registre: &IcebreakerRegistre,
    uuid: &str,
    current: Option<i64>,
    joueurs: &Joueurs,
) -> (Vec<(String, Vec<ItemKey>)>, Vec<(String, Vec<String>)>) {
    let config = config();
    let command = |words: &[&str]| words.iter().map(|w| w.to_string()).collect::<Vec<_>>();
    // The icons of a class (X.8): its weapon and its chest armour.
    let icones = |classe: &str| icebreaker_classes::class(classe).map(|c| icebreaker_classes::icones(&c)).unwrap_or_default();
    let mut lines: Vec<(String, Vec<ItemKey>)> = Vec::new();
    let mut buttons = Vec::new();
    if registre.charrette().is_empty() {
        lines.push(("Personne à la charrette aujourd'hui.".to_string(), Vec::new()));
    } else {
        lines.push(("À la charrette :".to_string(), Vec::new()));
    }
    for (i, recrue) in registre.charrette().iter().enumerate() {
        let classe = nom_de_classe(&recrue.classe);
        lines.push((format!("  {}, {classe}", recrue.nom), icones(&recrue.classe)));
        if current.is_none() {
            buttons.push((
                format!("Engager {} ({classe})", recrue.nom),
                command(&["registre", "engager", &i.to_string()]),
            ));
        }
    }
    lines.push((
        format!(
            "Registre ({}/{}) :",
            registre.personnages().len(),
            config.places_registre
        ),
        Vec::new(),
    ));
    // Those reserved by the player first.
    let mut fiches: Vec<&Fiche> = registre.personnages().iter().collect();
    fiches.sort_by_key(|fiche| fiche.reserve_par.as_deref() != Some(uuid));
    let mine = fiches
        .iter()
        .filter(|fiche| fiche.reserve_par.as_deref() == Some(uuid))
        .count();
    for fiche in fiches {
        let id = fiche.id.to_string();
        let etat = match (joueurs.get(&fiche.id), fiche.reserve_par.as_deref()) {
            (Some(joueur), _) if joueur.uuid == uuid => "vous".to_string(),
            (Some(joueur), _) => format!("joué par {}", joueur.alias),
            (None, Some(owner)) if owner == uuid => "réservé pour vous".to_string(),
            (None, Some(_)) => "réservé".to_string(),
            (None, None) => "libre".to_string(),
        };
        lines.push((
            format!("  {}, {} : {etat}", fiche.nom, nom_de_classe(&fiche.classe)),
            icones(&fiche.classe),
        ));
        let mien = fiche.reserve_par.as_deref() == Some(uuid);
        if current != Some(fiche.id) && registre.peut_jouer_fiche(fiche, uuid, joueurs).is_ok() {
            buttons.push((format!("Jouer {}", fiche.nom), command(&["registre", "jouer", &id])));
        }
        if mien {
            buttons.push((format!("Libérer {}", fiche.nom), command(&["registre", "liberer", &id])));
        } else if fiche.reserve_par.is_none() && mine < config.reservations_par_compte {
            buttons.push((format!("Réserver {}", fiche.nom), command(&["registre", "reserver", &id])));
        }
    }
    if current.is_some() {
        buttons.push((
            "Rendre au registre (redevenir fantôme)".to_string(),
            command(&["registre", "rendre"]),
        ));
    }
    (lines, buttons)
}

/// `/registre engager <n> | jouer <id> | rendre | reserver <id> | liberer
/// <id>` (the buttons of the cart's panel).
pub(crate) fn handle_registre(
    server: &mut crate::Server,
    client: Entity,
    _target: Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), common::comp::Content> {
    use common::comp::{ChatType, Content};
    use common_net::msg::ServerGeneral;
    let plain = |text: String| Content::Plain(text);
    if !active() {
        return Err(plain("Pas de registre ici (au village seulement).".to_string()));
    }
    let ecs = server.state.ecs();
    let Some(uuid) = ecs
        .read_storage::<Player>()
        .get(client)
        .map(|player| player.uuid().to_string())
    else {
        return Err(plain("Il faut être en jeu.".to_string()));
    };
    let current = match ecs.read_storage::<Presence>().get(client).map(|p| p.kind) {
        Some(PresenceKind::Character(id)) => Some(id),
        _ => None,
    };
    let ghost = is_ghost(ecs, client);
    let config = config();
    let id_arg = || {
        args.get(1)
            .and_then(|word| word.parse::<i64>().ok())
            .map(CharacterId)
            .ok_or_else(|| plain("Quel personnage ? (son numéro)".to_string()))
    };
    let reply = match args.first().map(String::as_str) {
        Some("engager") => {
            if !ghost {
                return Err(plain(
                    "Rendez d'abord votre personnage au registre pour en engager un autre."
                        .to_string(),
                ));
            }
            let index = args
                .get(1)
                .and_then(|word| word.parse::<usize>().ok())
                .ok_or_else(|| plain("Quelle recrue ? (son rang à la charrette)".to_string()))?;
            let recrue = ecs
                .write_resource::<IcebreakerRegistre>()
                .engager(index, client, &config)
                .map_err(plain)?;
            let class = icebreaker_classes::class(&recrue.classe)
                .ok_or_else(|| plain(format!("Classe inconnue : {}", recrue.classe)))?;
            let body = common::comp::Body::Humanoid(common::comp::humanoid::Body::random());
            let created = {
                let mut updater = ecs.system_data::<specs::WriteExpect<
                    crate::persistence::character_updater::CharacterUpdater,
                >>();
                crate::character_creator::create_recruit(
                    client,
                    REGISTRE_UUID.to_string(),
                    recrue.nom.clone(),
                    &class,
                    body,
                    &mut updater,
                    None,
                )
            };
            if let Err(err) = created {
                ecs.write_resource::<IcebreakerRegistre>().echec(client);
                return Err(plain(format!("La recrue n'a pas pu être engagée : {err}")));
            }
            info!(nom = recrue.nom, classe = recrue.classe, "Icebreaker: recruit taken");
            format!("{} ({}) arrive…", recrue.nom, class.nom)
        },
        Some("jouer") => {
            let id = id_arg()?;
            if current == Some(id) {
                return Err(plain("Vous le jouez déjà.".to_string()));
            }
            let joueurs = joueurs(&ecs.read_storage(), &ecs.read_storage());
            ecs.read_resource::<IcebreakerRegistre>()
                .peut_jouer(id, &uuid, &joueurs)
                .map_err(plain)?;
            server.notify_client(client, ServerGeneral::IcebreakerJouer(Some(id)));
            return Ok(());
        },
        Some("rendre") => {
            if current.is_none() {
                return Err(plain("Vous êtes déjà un fantôme.".to_string()));
            }
            server.notify_client(client, ServerGeneral::IcebreakerJouer(None));
            return Ok(());
        },
        Some("reserver") => {
            let id = id_arg()?;
            ecs.write_resource::<IcebreakerRegistre>()
                .reserver(id, &uuid, &config)
                .map_err(plain)?
        },
        Some("liberer") => {
            let id = id_arg()?;
            ecs.write_resource::<IcebreakerRegistre>()
                .liberer(id, &uuid)
                .map_err(plain)?
        },
        _ => {
            return Err(plain(
                "/registre engager <rang>, jouer <numéro>, rendre, reserver <numéro> ou liberer \
                 <numéro>"
                    .to_string(),
            ));
        },
    };
    server.notify_client(client, ServerGeneral::server_msg(ChatType::Meta, plain(reply)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> RegistreConfig {
        RegistreConfig {
            recrues_par_jour: 3,
            minimum_charrette: 2,
            places_registre: 4,
            reservations_par_compte: 2,
            noms: vec!["Anna".into(), "Bob".into(), "Cleo".into()],
            postes_recrues: Vec::new(),
        }
    }

    fn entity(world: &mut World) -> Entity {
        use specs::Builder;
        world.create_entity().build()
    }

    #[test]
    fn icebreaker_registre_the_cart_is_renewed_each_day_and_filled_on_return() {
        let (config, mut rng) = (config(), rand::rng());
        let mut registre = IcebreakerRegistre::default();
        assert!(registre.renouveler(0, &config, &mut rng));
        assert_eq!(registre.charrette().len(), 3);
        assert!(!registre.renouveler(0, &config, &mut rng), "same day");
        let classes: Vec<String> = icebreaker_classes::classes().into_iter().map(|c| c.id).collect();
        let mut noms: Vec<&str> = registre.charrette().iter().map(|r| r.nom.as_str()).collect();
        assert!(registre.charrette().iter().all(|r| classes.contains(&r.classe)));
        noms.sort();
        noms.dedup();
        assert_eq!(noms.len(), 3, "names are not taken twice");

        let mut world = World::new();
        let (a, b) = (entity(&mut world), entity(&mut world));
        registre.engager(0, a, &config).unwrap();
        registre.engager(0, b, &config).unwrap();
        registre.cree(a, CharacterId(1));
        assert_eq!(registre.charrette().len(), 1);
        assert!(registre.completer(&config, &mut rng), "back from a dungeon");
        assert_eq!(registre.charrette().len(), 2);
        assert!(!registre.completer(&config, &mut rng), "at the minimum");
        // Every name is taken: the new one gets a number.
        let nouveau = &registre.charrette()[1].nom;
        assert!(nouveau.contains(' '), "{nouveau}");
        assert!(registre.renouveler(1, &config, &mut rng));
        assert_eq!(registre.charrette().len(), 3);
    }

    #[test]
    fn icebreaker_registre_a_recruit_taken_joins_the_registry() {
        let (config, mut rng) = (config(), rand::rng());
        let mut registre = IcebreakerRegistre::default();
        registre.renouveler(0, &config, &mut rng);
        let mut world = World::new();
        let a = entity(&mut world);
        let recrue = registre.engager(1, a, &config).unwrap();
        assert!(registre.engager(0, a, &config).is_err(), "one at a time");
        assert!(registre.engager(9, entity(&mut world), &config).is_err());
        let fiche = registre.cree(a, CharacterId(42)).unwrap();
        assert_eq!((fiche.nom, fiche.classe), (recrue.nom, recrue.classe));
        assert!(registre.fiche(CharacterId(42)).is_some());

        // A failed save gives the recruit back to the cart.
        let b = entity(&mut world);
        let before = registre.charrette().len();
        registre.engager(0, b, &config).unwrap();
        assert!(registre.echec(b));
        assert_eq!(registre.charrette().len(), before);
    }

    #[test]
    fn icebreaker_registre_the_registry_has_a_limit() {
        let (config, mut rng) = (config(), rand::rng());
        let mut registre = IcebreakerRegistre::default();
        let mut world = World::new();
        for id in 0..4 {
            registre.completer(&config, &mut rng);
            let e = entity(&mut world);
            registre.engager(0, e, &config).unwrap();
            registre.cree(e, CharacterId(id));
        }
        registre.completer(&config, &mut rng);
        assert!(registre.engager(0, entity(&mut world), &config).is_err());
    }

    fn registre_de(ids: &[i64]) -> IcebreakerRegistre {
        let mut registre = IcebreakerRegistre::default();
        for id in ids {
            registre.etat.personnages.push(Fiche {
                id: *id,
                nom: format!("P{id}"),
                classe: "mage".into(),
                reserve_par: None,
            });
        }
        registre
    }

    #[test]
    fn icebreaker_registre_one_player_per_character() {
        let registre = registre_de(&[1, 2]);
        let mut joueurs = Joueurs::new();
        joueurs.insert(1, Joueur {
            uuid: "a".into(),
            alias: "Alice".into(),
        });
        assert!(registre.peut_jouer(CharacterId(1), "b", &joueurs).is_err());
        assert!(registre.peut_jouer(CharacterId(1), "a", &joueurs).is_ok());
        assert!(registre.peut_jouer(CharacterId(2), "b", &joueurs).is_ok());
        assert!(registre.peut_jouer(CharacterId(3), "b", &joueurs).is_err(), "not in the registry");
    }

    #[test]
    fn icebreaker_registre_reservations() {
        let config = config();
        let mut registre = registre_de(&[1, 2, 3]);
        let none = Joueurs::new();
        registre.reserver(CharacterId(1), "a", &config).unwrap();
        assert!(registre.peut_jouer(CharacterId(1), "b", &none).is_err(), "reserved by a");
        assert!(registre.peut_jouer(CharacterId(1), "a", &none).is_ok());
        assert!(registre.reserver(CharacterId(1), "b", &config).is_err());
        registre.reserver(CharacterId(2), "a", &config).unwrap();
        assert!(registre.reserver(CharacterId(3), "a", &config).is_err(), "2 at most here");
        assert!(registre.liberer(CharacterId(1), "b").is_err());
        registre.liberer(CharacterId(1), "a").unwrap();
        registre.reserver(CharacterId(3), "a", &config).unwrap();
        assert!(registre.peut_jouer(CharacterId(1), "b", &none).is_ok());
        assert!(registre.de_quoi_jouer("b", &none));
        assert!(!registre_de(&[]).de_quoi_jouer("b", &none));
    }

    #[test]
    fn icebreaker_registre_is_saved_and_read_back() {
        let dir = std::env::temp_dir().join(format!("icebreaker-registre-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (config, mut rng) = (config(), rand::rng());
        {
            let mut registre = IcebreakerRegistre::new(Some(dir.clone()));
            registre.renouveler(5, &config, &mut rng);
            let mut world = World::new();
            let a = entity(&mut world);
            registre.engager(0, a, &config).unwrap();
            registre.cree(a, CharacterId(7));
            registre.reserver(CharacterId(7), "a", &config).unwrap();
        }
        let registre = IcebreakerRegistre::new(Some(dir.clone()));
        assert_eq!(registre.charrette().len(), 2);
        assert_eq!(registre.fiche(CharacterId(7)).and_then(|f| f.reserve_par.clone()), Some("a".into()));
        assert_eq!(registre.etat.jour, Some(5));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn icebreaker_registre_panel_offers_what_the_player_may_do() {
        let mut registre = registre_de(&[1, 2]);
        registre.etat.charrette.push(Recrue {
            nom: "Anna".into(),
            classe: "archer".into(),
        });
        registre.etat.personnages[1].reserve_par = Some("b".into());
        let (lines, buttons) = panel_for(&registre, "a", None, &Joueurs::new());
        let labels: Vec<&str> = buttons.iter().map(|(label, _)| label.as_str()).collect();
        assert!(
            lines.iter().any(|(l, icones)| l.contains("Anna, archer") && icones.len() == 2),
            "{lines:?}"
        );
        assert!(labels.contains(&"Engager Anna (archer)"));
        assert!(labels.contains(&"Jouer P1"));
        assert!(!labels.contains(&"Jouer P2"), "reserved by b");
        assert!(!labels.iter().any(|l| l.starts_with("Rendre")));
        let (_, buttons) = panel_for(&registre, "a", Some(1), &Joueurs::new());
        let labels: Vec<&str> = buttons.iter().map(|(label, _)| label.as_str()).collect();
        assert!(!labels.iter().any(|l| l.starts_with("Engager")), "a character does not hire");
        assert!(labels.iter().any(|l| l.starts_with("Rendre")));
    }

    #[test]
    fn icebreaker_registre_a_dead_character_leaves_the_registry() {
        let (config, mut rng) = (config(), rand::rng());
        let mut registre = IcebreakerRegistre::default();
        registre.renouveler(0, &config, &mut rng);
        let mut world = World::new();
        let a = entity(&mut world);
        registre.engager(0, a, &config).unwrap();
        registre.cree(a, CharacterId(7));
        assert!(registre.retirer(CharacterId(7)).is_some());
        assert!(registre.fiche(CharacterId(7)).is_none());
        assert!(registre.retirer(CharacterId(7)).is_none(), "already gone");
    }
}
