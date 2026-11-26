//! Icebreaker : la décision de nos ennemis humanoïdes (jalon IA, `docs/icebreaker/PLAN-IA.md`).
//! La logique est dans le crate `icebreaker-ia` ; ici, le chargement de ses réglages
//! (`assets/icebreaker/ia/profils.ron`) et l'interrupteur d'affichage du lieu d'entraînement.

use crate::{
    assets::{AssetExt, Ron},
    comp::Body,
    icebreaker_races::nom,
};
pub use icebreaker_ia::{
    Allie, Apercu, Bruits, Cerveau, Etat, Evenement, Id, Intention, Message, Niveau, Observation, Percu, Profil,
    Profils, Reaction, niveau,
};
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};

/// Le profil d'un ennemi : la race de son corps, son rôle et son cran d'habileté (`Cerveau::fiche`)
/// (rien hors humanoïde, ou si le fichier manque). Relu à chaque appel : un réglage changé dans le
/// fichier vaut tout de suite.
pub fn profil(body: Option<&Body>, (role, cran): (&str, &str)) -> Option<Profil> {
    let Some(Body::Humanoid(humanoide)) = body else {
        return None;
    };
    let profils = Ron::<Profils>::load("icebreaker.ia.profils").ok()?;
    Some(profils.read().0.profil(nom(humanoide.species), role, cran))
}

/// Nos ennemis, tels que les leurs les connaissent (poste, alerté ou non, parti prévenir) : refait à
/// chaque tick par le serveur (`server/src/sys/icebreaker_ia_monde.rs`), car les agents tournent en
/// parallèle et ne se voient pas.
#[derive(Clone, Debug, Default)]
pub struct IcebreakerIaAllies(pub Vec<Allie>);

/// Les bruits et leur portée (`assets/icebreaker/ia/bruits.ron` ; sans fichier, personne n'entend rien).
pub fn bruits() -> Bruits {
    Ron::<Bruits>::load("icebreaker.ia.bruits").map_or_else(|_| Bruits::default(), |r| r.read().0.clone())
}

/// Un groupe d'ennemis d'une scène d'essai.
#[derive(Clone, Debug, Deserialize)]
pub struct Groupe {
    /// "orc", "danari"… (`arene.ron` → `races`).
    pub race: String,
    /// "guerrier", "archer", "mage", "chef".
    pub role: String,
    pub nombre: u32,
    /// Où, depuis le centre de l'arène (x vers l'est, y vers le nord).
    pub en: (f32, f32),
    /// Vers où ils sont tournés en arrivant (rien : comme Veloren les pose).
    #[serde(default)]
    pub regard: Option<(f32, f32)>,
    /// Ils ne flânent pas au repos.
    #[serde(default)]
    pub immobile: bool,
}

/// Une scène d'essai du lieu d'entraînement (`/entrainement scene <nom>`).
#[derive(Clone, Debug, Deserialize)]
pub struct Scene {
    /// Ce qu'on y essaie, dit au joueur.
    pub texte: String,
    /// Où le joueur est posé, depuis le centre de l'arène (rien : il reste où il est).
    #[serde(default)]
    pub joueur: Option<(f32, f32)>,
    pub groupes: Vec<Groupe>,
}

/// Le fichier `scenes.ron`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Scenes {
    pub scenes: std::collections::HashMap<String, Scene>,
}

/// Les scènes d'essai (`assets/icebreaker/ia/scenes.ron` ; sans fichier, aucune).
pub fn scenes() -> Scenes {
    Ron::<Scenes>::load("icebreaker.ia.scenes").map_or_else(|_| Scenes::default(), |r| r.read().0.clone())
}

/// `/entrainement ia` : chaque ennemi dit son état au-dessus de lui quand il en change.
static AFFICHER: AtomicBool = AtomicBool::new(false);

pub fn afficher() -> bool { AFFICHER.load(Ordering::Relaxed) }

pub fn regler_afficher(oui: bool) { AFFICHER.store(oui, Ordering::Relaxed) }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::humanoid;

    fn corps(species: humanoid::Species) -> Body {
        Body::Humanoid(humanoid::Body { species, ..humanoid::Body::random() })
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let profils = Ron::<Profils>::load_expect("icebreaker.ia.profils").read().0.clone();
        for race in ["human", "orc", "dwarf", "elf", "draugr", "danari"] {
            assert!(profils.races.contains_key(race), "{race} manque dans profils.ron");
        }
        // Tous se répartissent et rechoisissent leur cible.
        let orc = profil(Some(&corps(humanoid::Species::Orc)), ("guerrier", "")).unwrap();
        assert!(orc.repartition.is_some_and(|r| r > 0.0) && orc.reciblage.is_some());
        // Tous voient : net de moins loin que soupçonné, dans un cône, et un peu dans leur dos.
        let (vue, nette) = (orc.vue.unwrap(), orc.vue_nette.unwrap());
        assert!(nette < vue && orc.dos.unwrap() < nette && orc.cone.is_some_and(|c| c > 0.0 && c < 360.0));
        // Tous cherchent une cible perdue : poursuite, trajet, fouille, puis retour.
        assert!(orc.poursuite.is_some() && orc.fouille.is_some() && orc.soupcon.is_some());
        assert!(orc.trajet.unwrap() > orc.poursuite.unwrap() && orc.arrivee.is_some_and(|a| a > 0.0));
        // Y.1, Y.2 : les danari achèvent à plusieurs et fuient blessés ; pas les orcs.
        let danari = profil(Some(&corps(humanoid::Species::Danari)), ("guerrier", "")).unwrap();
        assert!(danari.acheve_a(2) && !danari.acheve_a(1));
        assert!(danari.fuite.is_some() && orc.fuite.is_none());
    }

    /// Les scènes d'essai ne nomment que des races et des rôles de l'arène, et tiennent sur le plateau.
    #[test]
    fn scenes_lisibles() {
        let scenes = Ron::<Scenes>::load_expect("icebreaker.ia.scenes").read().0.clone();
        let arene = crate::icebreaker_arene::config();
        assert!(scenes.scenes.contains_key("ecoute") && scenes.scenes.contains_key("massacre"));
        for (nom, scene) in &scenes.scenes {
            assert!(!scene.groupes.is_empty() && !scene.texte.is_empty(), "{nom}");
            let sur_le_plateau = |(x, y): (f32, f32)| x.hypot(y) < arene.rayon_terrain - 2.0;
            assert!(scene.joueur.is_none_or(sur_le_plateau), "{nom} : joueur hors du plateau");
            for g in &scene.groupes {
                assert!(arene.races.iter().any(|(id, _)| *id == g.race), "{nom} : race {}", g.race);
                assert!(arene.roles.iter().any(|(id, _)| *id == g.role), "{nom} : rôle {}", g.role);
                assert!(g.nombre >= 1 && sur_le_plateau(g.en), "{nom}");
            }
        }
    }

    /// IA.3 : tous entendent ; une mort s'entend de plus loin qu'un coup ; un dormeur entend moins bien.
    #[test]
    fn bruits_lisibles() {
        let bruits = Ron::<Bruits>::load_expect("icebreaker.ia.bruits").read().0.clone();
        assert!(bruits.portee("combat") > 0.0 && bruits.portee("mort") > bruits.portee("combat"));
        assert!(bruits.hauteur >= 1.0 && bruits.endormi > 0.0 && bruits.endormi < 1.0);
        let orc = profil(Some(&corps(humanoid::Species::Orc)), ("guerrier", "")).unwrap();
        assert!(orc.ouie.is_some_and(|o| o > 0.0));
        // IA.4 : un cri s'entend.
        assert!(bruits.portee("cri") > 0.0);
    }

    /// IA.4 : à la vue d'un intrus, un guerrier peut partir prévenir, mais se bat le plus souvent ; le
    /// chef ne quitte pas les siens (Q70).
    #[test]
    fn qui_part_prevenir() {
        let poids = |role: &str, evenement, reaction| {
            let profil = profil(Some(&corps(humanoid::Species::Orc)), (role, "")).unwrap();
            profil.reactions.get(&evenement).into_iter().flatten().filter(|(r, _)| *r == reaction).map(|(_, p)| *p).sum::<f32>()
        };
        for evenement in [Evenement::VoitIntrus, Evenement::AllieTombe] {
            assert!(poids("guerrier", evenement, Reaction::Prevenir) > 0.0);
            assert_eq!(poids("chef", evenement, Reaction::Prevenir), 0.0);
        }
        assert!(poids("guerrier", Evenement::VoitIntrus, Reaction::Combattre) > poids("guerrier", Evenement::VoitIntrus, Reaction::Prevenir));
        let orc = profil(Some(&corps(humanoid::Species::Orc)), ("guerrier", "")).unwrap();
        assert!(orc.voisinage.is_some_and(|v| v > 0.0) && orc.prevenir_a.is_some() && orc.tirage_delai.is_some());
    }
}
