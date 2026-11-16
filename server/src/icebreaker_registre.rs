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
    comp::{Player, Presence, PresenceKind},
    icebreaker_competences::{Acquis, IcebreakerCompetences},
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

/// Whether `entity` is a ghost: in game without a character (Veloren's spectator, or the ghost
/// with a body of the village, FT.4).
pub fn is_ghost(ecs: &World, entity: Entity) -> bool {
    ecs.read_storage::<Presence>()
        .get(entity)
        .is_some_and(|presence| {
            matches!(presence.kind, PresenceKind::Spectator | PresenceKind::IcebreakerFantome)
        })
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
    /// Icebreaker A.8 : compétences d'arme tirées avec chaque recrue d'une classe décrite.
    #[serde(default = "competences_tirees_par_defaut")]
    pub competences_tirees: usize,
    /// Names of the recruits.
    pub noms: Vec<String>,
    /// Where the recruits of the day stand, in front of the cart (X.8): offsets
    /// from the cart post, in blocks.
    #[serde(default)]
    pub postes_recrues: Vec<(f32, f32)>,
}

fn competences_tirees_par_defaut() -> usize { 2 }

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
    /// Icebreaker : le lot tiré avec la recrue (jalon A) ; vide : le lot par défaut de sa classe.
    #[serde(default)]
    pub acquis: Acquis,
}

impl Recrue {
    /// Icebreaker A.8 : le lot de la recrue ; pour une recrue d'avant le jalon A (lot vide), le lot par
    /// défaut de sa classe.
    pub fn lot(&self) -> Acquis {
        if self.acquis.est_vide() {
            Acquis::par_defaut(&self.classe)
        } else {
            self.acquis.clone()
        }
    }
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
    /// Icebreaker : ce que le personnage a acquis (jalon A) : jeux d'armes, compétences et
    /// niveaux, ultimes, passifs. Vide pour une fiche d'avant le jalon : voir `acquis_ou_defaut`.
    #[serde(default)]
    pub acquis: Acquis,
}

impl Fiche {
    /// Icebreaker : ce que le personnage a acquis ; pour une fiche d'avant le jalon A, le lot par
    /// défaut de sa classe.
    pub fn acquis_ou_defaut(&self) -> Acquis {
        if self.acquis.est_vide() {
            Acquis::par_defaut(&self.classe)
        } else {
            self.acquis.clone()
        }
    }
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
        // Icebreaker A.8 : le lot tiré avec la recrue (vide si sa classe n'est pas décrite).
        let acquis = Acquis::tirer(&classe, config.competences_tirees, rng);
        Recrue { nom, classe, acquis }
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
            acquis: recrue.acquis,
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

    /// Icebreaker : ce que le personnage `id` a acquis en jeu repart dans sa fiche (jalon A) ; vrai
    /// si la fiche a changé.
    pub fn ecrire_acquis(&mut self, id: CharacterId, acquis: &Acquis) -> bool {
        let Some(fiche) = self.etat.personnages.iter_mut().find(|fiche| fiche.id == id.0) else {
            return false;
        };
        if fiche.acquis == *acquis {
            return false;
        }
        fiche.acquis = acquis.clone();
        self.save();
        true
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

/// Icebreaker : le personnage que joue `entity`, s'il en joue un.
fn personnage_de(ecs: &World, entity: Entity) -> Option<CharacterId> {
    match ecs.read_storage::<Presence>().get(entity)?.kind {
        PresenceKind::Character(id) => Some(id),
        _ => None,
    }
}

/// Icebreaker : les compétences avec lesquelles le personnage du registre que joue `entity` entre
/// en jeu (jalon A) : ce que dit sa fiche. `None` hors registre, ou si nos données ne décrivent pas
/// sa classe (il garde alors les capacités de Veloren).
pub fn competences_de(ecs: &World, entity: Entity) -> Option<IcebreakerCompetences> {
    if !active() {
        return None;
    }
    let id = personnage_de(ecs, entity)?;
    let registre = ecs.read_resource::<IcebreakerRegistre>();
    let fiche = registre.fiche(id)?;
    IcebreakerCompetences::nouveau(&fiche.classe, fiche.acquis_ou_defaut())
}

/// Icebreaker : ce que le personnage joué par `entity` a acquis repart dans sa fiche (jalon A) :
/// au retour d'expédition et quand le personnage est sauvé.
pub fn sauver_acquis(ecs: &World, entity: Entity) {
    let Some(id) = personnage_de(ecs, entity) else {
        return;
    };
    let acquis = ecs
        .read_storage::<IcebreakerCompetences>()
        .get(entity)
        .map(|competences| competences.acquis.clone());
    if let Some(acquis) = acquis {
        ecs.write_resource::<IcebreakerRegistre>()
            .ecrire_acquis(id, &acquis);
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

/// Icebreaker A.8 : « guerrier · grande épée » : le nom de la classe, puis le ou les jeux d'armes de
/// `acquis` en minuscules. Les classes sans données n'ont que leur nom.
fn sous_titre_classe(classe: &str, acquis: &Acquis) -> String {
    let jeux: Vec<String> = acquis
        .jeux
        .keys()
        .filter_map(|id| common::icebreaker_competences::registre().jeu(id))
        .map(|jeu| jeu.nom.to_lowercase())
        .collect();
    if jeux.is_empty() {
        nom_de_classe(classe)
    } else {
        format!("{} · {}", nom_de_classe(classe), jeux.join(", "))
    }
}

/// Icebreaker A.8 : la seconde ligne d'une recrue d'une classe décrite : ses compétences d'arme, puis
/// son ultime et son passif. `None` pour une classe sans données.
fn ligne_du_lot(recrue: &Recrue) -> Option<common::icebreaker_panneau::Ligne> {
    let registre = common::icebreaker_competences::registre();
    let lot = recrue.lot();
    let (_, jeu) = lot.jeux.iter().next()?;
    let nom_competence = |id: &String| registre.competence(id).map(|e| e.competence.nom.clone());
    let competences: Vec<String> = jeu.placees.iter().flatten().filter_map(nom_competence).collect();
    let mut suite = Vec::new();
    if let Some(nom) = lot.ultime.as_ref().and_then(nom_competence) {
        suite.push(format!("ultime {nom}"));
    }
    if let Some(passif) = &lot.passif
        && let Some((_, p)) = registre
            .passifs_de(&recrue.classe)
            .and_then(|ps| ps.passifs.iter().find(|(id, _)| id == passif))
    {
        suite.push(format!("passif {}", p.nom));
    }
    Some(
        common::icebreaker_panneau::Ligne::nouvelle(competences.join(", "))
            .sous_titre(suite.join(" · "))
            .estompee(),
    )
}

/// Icebreaker : le panneau typé de la charrette (jalon N) pour le compte `uuid`
/// qui joue `current` (aucun : un fantôme). Les recrues du jour et le registre
/// sont deux onglets ; chaque personnage est une ligne avec l'icône de sa classe,
/// son état, le cadenas (réserver) et ce qu'on peut en faire.
pub fn panneau_pour(
    registre: &IcebreakerRegistre,
    uuid: &str,
    current: Option<i64>,
    joueurs: &Joueurs,
) -> common::icebreaker_panneau::Panneau {
    use common::icebreaker_panneau::{Action, Ligne, Onglet, Panneau, Sorte, Ton, texte};
    let config = config();
    let icone = |classe: &str| {
        icebreaker_classes::class(classe)
            .map(|class| class.icone)
            .unwrap_or_default()
    };
    let recrues = if registre.charrette().is_empty() {
        vec![Ligne::nouvelle("Personne à la charrette aujourd'hui.").estompee()]
    } else {
        registre
            .charrette()
            .iter()
            .enumerate()
            .flat_map(|(i, recrue)| {
                // La commande garde le rang de la recrue, pas celui de la ligne (A.8).
                let ligne = Ligne::nouvelle(recrue.nom.clone())
                    .icone(icone(&recrue.classe))
                    .sous_titre(sous_titre_classe(&recrue.classe, &recrue.lot()))
                    .action(
                        Action::nouvelle("Engager", &["registre", "engager", &i.to_string()])
                            .principale()
                            .permise(current.is_none()),
                    );
                std::iter::once(ligne).chain(ligne_du_lot(recrue))
            })
            .collect()
    };
    // Those reserved by the player first.
    let mut fiches: Vec<&Fiche> = registre.personnages().iter().collect();
    fiches.sort_by_key(|fiche| fiche.reserve_par.as_deref() != Some(uuid));
    let mine = fiches
        .iter()
        .filter(|fiche| fiche.reserve_par.as_deref() == Some(uuid))
        .count();
    let lignes = fiches
        .iter()
        .map(|fiche| {
            let id = fiche.id.to_string();
            let classe = sous_titre_classe(&fiche.classe, &fiche.acquis_ou_defaut());
            let ligne = Ligne::nouvelle(fiche.nom.clone()).icone(icone(&fiche.classe));
            let jouer = Action::nouvelle("Jouer", &["registre", "jouer", &id]).permise(
                current.is_none() && registre.peut_jouer_fiche(fiche, uuid, joueurs).is_ok(),
            );
            if current == Some(fiche.id) {
                return ligne.sous_titre(classe).pastille("vous", Ton::Or).choisie();
            }
            match (joueurs.get(&fiche.id), fiche.reserve_par.as_deref()) {
                (Some(joueur), _) if joueur.uuid == uuid => {
                    ligne.sous_titre(classe).pastille("vous", Ton::Or).choisie()
                },
                (Some(joueur), _) => ligne
                    .sous_titre(format!("{classe}, joué par {}", joueur.alias))
                    .pastille("en jeu", Ton::Discret)
                    .estompee(),
                (None, Some(owner)) if owner == uuid => ligne
                    .sous_titre(classe)
                    .pastille("à vous", Ton::Info)
                    .cadenas(true, &["registre", "liberer", &id])
                    .action(jouer.principale()),
                (None, Some(_)) => ligne
                    .sous_titre(classe)
                    .pastille("réservé", Ton::Discret)
                    .estompee(),
                (None, None) => {
                    let ligne = ligne.sous_titre(classe).action(jouer);
                    if mine < config.reservations_par_compte {
                        ligne.cadenas(false, &["registre", "reserver", &id])
                    } else {
                        ligne
                    }
                },
            }
        })
        .collect();
    let joue = current
        .and_then(|id| registre.personnages().iter().find(|fiche| fiche.id == id))
        .map(|fiche| fiche.nom.clone());
    let mut panneau = Panneau::nouveau("armure", "Charrette", match (&joue, current) {
        (Some(nom), _) => format!("Vous jouez {nom}."),
        (None, Some(_)) => "Vous jouez un personnage.".to_string(),
        (None, None) => "Fantôme : choisissez un personnage.".to_string(),
    });
    panneau.onglets = vec![
        Onglet {
            libelle: texte(format!("Recrues ({})", registre.charrette().len())),
            lignes: recrues,
        },
        Onglet {
            libelle: texte(format!(
                "Registre ({}/{})",
                registre.personnages().len(),
                config.places_registre
            )),
            lignes,
        },
    ];
    panneau.onglet = u8::from(current.is_some() || mine > 0);
    if current.is_some() {
        panneau.actions.push(Action::nouvelle(
            match &joue {
                Some(nom) => format!("Rendre {nom} au registre"),
                None => "Rendre au registre".to_string(),
            },
            &["registre", "rendre"],
        ));
        panneau.message("Rendez votre personnage pour en changer.", Sorte::Info)
    } else {
        panneau.message(
            format!(
                "Le cadenas réserve un personnage ({} au plus).",
                config.reservations_par_compte
            ),
            Sorte::Info,
        )
    }
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
                    &recrue.acquis,
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
            competences_tirees: 2,
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
                acquis: Acquis::default(),
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
        use common::icebreaker_panneau::{Ligne, Ton, texte};
        let mut registre = registre_de(&[1, 2]);
        registre.etat.charrette.push(Recrue {
            nom: "Anna".into(),
            classe: "archer".into(),
            acquis: Acquis::default(),
        });
        registre.etat.personnages[1].reserve_par = Some("b".into());
        let ligne = |lignes: &[Ligne], nom: &str| {
            lignes
                .iter()
                .find(|l| l.titre == texte(nom))
                .cloned()
                .unwrap_or_else(|| panic!("no line {nom} in {lignes:?}"))
        };
        let permise = |l: &Ligne| l.action.as_ref().is_some_and(|action| action.permise);

        // A ghost: the recruits first, hiring and playing allowed.
        let panneau = panneau_pour(&registre, "a", None, &Joueurs::new());
        assert_eq!(panneau.onglet, 0);
        let anna = ligne(&panneau.onglets[0].lignes, "Anna");
        assert_eq!(anna.sous_titre, texte("archer · arc"));
        assert_eq!(anna.icone, "arc", "the icon of its class");
        assert!(permise(&anna));
        let fiches = &panneau.onglets[1].lignes;
        let p1 = ligne(fiches, "P1");
        assert!(permise(&p1));
        assert_eq!(p1.cadenas.as_ref().map(|(ferme, _)| *ferme), Some(false));
        let p2 = ligne(fiches, "P2");
        assert!(p2.action.is_none() && p2.estompee, "reserved by b");
        assert_eq!(p2.pastille, Some((texte("réservé"), Ton::Discret)));
        assert!(panneau.actions.is_empty(), "nothing to give back");

        // Playing P1: the registry first, hiring greyed, giving back offered.
        let panneau = panneau_pour(&registre, "a", Some(1), &Joueurs::new());
        assert_eq!(panneau.onglet, 1);
        assert!(
            !permise(&ligne(&panneau.onglets[0].lignes, "Anna")),
            "a character does not hire"
        );
        let p1 = ligne(&panneau.onglets[1].lignes, "P1");
        assert!(p1.choisie && p1.action.is_none());
        assert_eq!(p1.pastille, Some((texte("vous"), Ton::Or)));
        assert_eq!(panneau.actions.len(), 1);
        assert_eq!(panneau.actions[0].commande, vec![
            "registre".to_string(),
            "rendre".to_string()
        ]);
    }

    // Jalon A.8 : le lot tiré, montré sur deux lignes, avec les armes qui vont avec.
    #[test]
    fn icebreaker_registre_a_drawn_recruit_shows_its_set_on_two_lines() {
        use common::icebreaker_panneau::texte;
        use common::icebreaker_competences::{jeu_en_main, registre as donnees};
        let mut registre = IcebreakerRegistre::default();
        let mut rng = rand::rng();
        // Un guerrier au lot connu (marteau), puis un archer, puis un autre guerrier.
        registre.etat.charrette = vec![
            Recrue {
                nom: "Leif".into(),
                classe: "guerrier".into(),
                acquis: Acquis::lot("marteau", &["moulinet"], Some("envol"), Some("robuste")),
            },
            Recrue { nom: "Anna".into(), classe: "archer".into(), acquis: Acquis::default() },
            Recrue {
                nom: "Bob".into(),
                classe: "guerrier".into(),
                acquis: Acquis::tirer("guerrier", 2, &mut rng),
            },
        ];
        let panneau = panneau_pour(&registre, "a", None, &Joueurs::new());
        let lignes = &panneau.onglets[0].lignes;
        // Deux lignes par recrue d'une classe décrite (guerrier, archer).
        assert_eq!(lignes.len(), 6, "{lignes:?}");
        assert_eq!(lignes[0].titre, texte("Leif"));
        assert_eq!(lignes[0].sous_titre, texte("guerrier · marteau à deux mains"));
        assert_eq!(lignes[1].sous_titre, texte("ultime Envol · passif Robuste"));
        assert!(lignes[1].estompee && lignes[1].action.is_none());
        assert_eq!(lignes[2].titre, texte("Anna"));
        assert_eq!(lignes[2].sous_titre, texte("archer · arc"));
        assert_eq!(lignes[3].sous_titre, texte("ultime Volée mortelle · passif Tir puissant"));
        assert_eq!(lignes[4].titre, texte("Bob"));
        assert!(lignes[3].estompee && lignes[5].estompee && lignes[5].action.is_none());
        // Les numéros des commandes sont ceux des recrues, pas des lignes.
        let rang = |l: &common::icebreaker_panneau::Ligne| {
            l.action.as_ref().map(|a| a.commande.last().cloned().unwrap())
        };
        assert_eq!(
            [rang(&lignes[0]), rang(&lignes[2]), rang(&lignes[4])],
            [Some("0".to_string()), Some("1".to_string()), Some("2".to_string())]
        );
        // Un personnage d'une classe décrite montre son jeu d'armes dans le registre.
        registre.etat.personnages.push(Fiche {
            id: 3,
            nom: "Mael".into(),
            classe: "guerrier".into(),
            reserve_par: None,
            acquis: Acquis::lot("hache", &[], None, None),
        });
        let panneau = panneau_pour(&registre, "a", None, &Joueurs::new());
        assert_eq!(panneau.onglets[1].lignes[0].sous_titre, texte("guerrier · hache à deux mains"));

        // La recrue engagée a en mains les armes de son jeu ; le PNJ aussi.
        let class = icebreaker_classes::class("guerrier").unwrap();
        for _ in 0..20 {
            let lot = Acquis::tirer("guerrier", 2, &mut rng);
            let (jeu, _) = lot.jeux.iter().next().unwrap();
            let inventaire = icebreaker_classes::inventaire_du_lot(&class, &lot);
            assert_eq!(jeu_en_main("guerrier", Some(&inventaire)).map(|(id, _)| id), Some(jeu.as_str()));
            assert!(donnees().jeu(jeu).is_some());
        }
        // Une recrue tirée par la charrette porte un lot si sa classe est décrite.
        for _ in 0..20 {
            let recrue = registre.tirer(&config(), &mut rng);
            assert_eq!(recrue.acquis.est_vide(), !donnees().classe_geree(&recrue.classe), "{recrue:?}");
        }
    }

    #[test]
    fn icebreaker_registre_an_old_settings_file_is_still_read() {
        // Sans `competences_tirees`, le réglage par défaut s'applique.
        let ancien = r#"(recrues_par_jour: 3, minimum_charrette: 2, places_registre: 10,
            reservations_par_compte: 5, noms: ["Anna"])"#;
        let config: RegistreConfig = ron::from_str(ancien).expect("an old registre.ron");
        assert_eq!(config.competences_tirees, 2);
        // Le fichier du dépôt se lit.
        assert!(super::config().competences_tirees >= 1);
    }

    #[test]
    fn icebreaker_registre_a_recruit_without_a_set_gets_its_default() {
        // Une recrue d'avant le jalon A (lot vide) montre le jeu par défaut de sa classe.
        let recrue = Recrue { nom: "X".into(), classe: "guerrier".into(), acquis: Acquis::default() };
        assert_eq!(recrue.lot(), Acquis::par_defaut("guerrier"));
        assert!(ligne_du_lot(&recrue).is_some());
        let inconnue = Recrue { classe: "classe inconnue".into(), ..recrue };
        assert!(ligne_du_lot(&inconnue).is_none());
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

    // Jalon A : la fiche porte ce que le personnage a acquis.
    #[test]
    fn icebreaker_registre_a_sheet_keeps_what_the_character_learnt() {
        // Une fiche d'avant le jalon A se relit, et reçoit le lot par défaut de sa classe.
        let ancien = r#"(charrette: [(nom: "Anna", classe: "guerrier")],
            personnages: [(id: 7, nom: "Leif", classe: "guerrier", reserve_par: None)], jour: Some(2))"#;
        let etat: Etat = ron::from_str(ancien).expect("an old registry file");
        assert!(etat.charrette[0].acquis.est_vide());
        let fiche = &etat.personnages[0];
        assert!(fiche.acquis.est_vide());
        let lot = fiche.acquis_ou_defaut();
        assert_eq!(lot, Acquis::par_defaut("guerrier"));
        assert!(lot.jeux.contains_key("grande_epee"), "{lot:?}");
        // Une classe que nos données ne décrivent pas n'a rien, et pas de composant.
        assert!(IcebreakerCompetences::nouveau("classe inconnue", Acquis::default()).is_none());
        assert!(IcebreakerCompetences::nouveau("guerrier", lot.clone()).is_some());

        // Ce qui est acquis en jeu repart dans la fiche, et se relit du fichier.
        let dir = std::env::temp_dir().join(format!("icebreaker-acquis-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut appris = lot.clone();
        appris.jeux.get_mut("grande_epee").unwrap().maitrise = 120;
        {
            let mut registre = IcebreakerRegistre::new(Some(dir.clone()));
            registre.etat.personnages.push(etat.personnages[0].clone());
            assert!(!registre.ecrire_acquis(CharacterId(8), &appris), "not in the registry");
            assert!(registre.ecrire_acquis(CharacterId(7), &appris));
            assert!(!registre.ecrire_acquis(CharacterId(7), &appris), "nothing new");
        }
        let registre = IcebreakerRegistre::new(Some(dir.clone()));
        assert_eq!(registre.fiche(CharacterId(7)).map(|f| f.acquis_ou_defaut()), Some(appris));
        let _ = std::fs::remove_dir_all(dir);
    }
}
