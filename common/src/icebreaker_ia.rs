//! Icebreaker : la décision de nos ennemis humanoïdes (jalon IA, `docs/icebreaker/PLAN-IA.md`).
//! La logique est dans le crate `icebreaker-ia` ; ici, le chargement de ses réglages
//! (`assets/icebreaker/ia/profils.ron`) et l'interrupteur d'affichage du lieu d'entraînement.

use crate::{
    assets::{AssetExt, Ron},
    comp::Body,
    icebreaker_races::nom,
};
pub use icebreaker_ia::{
    Allie, Apercu, Bruits, Cerveau, Effet, Etat, Evenement, Geste, Id, Intention, Message, Niveau, Observation, Ordre,
    Percu,
    Profil,
    Profils, Reaction, Situation, Sort, niveau,
};
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};

/// Le profil d'un ennemi : la race de son corps, son rôle et son cran d'habileté (`Cerveau::fiche`)
/// (rien hors humanoïde, ou si le fichier manque).
pub fn profil(body: Option<&Body>, (role, cran): (&str, &str)) -> Option<Profil> {
    let Some(Body::Humanoid(humanoide)) = body else {
        return None;
    };
    let profils = Ron::<Profils>::load("icebreaker.ia.profils").ok()?;
    Some(profils.read().0.profil(nom(humanoide.species), role, cran))
}

/// Tous les profils (`assets/icebreaker/ia/profils.ron` ; sans fichier, aucun réglage).
pub fn profils() -> Profils {
    Ron::<Profils>::load("icebreaker.ia.profils").map_or_else(|_| Profils::default(), |r| r.read().0.clone())
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
    /// La part de leurs PV qu'ils ont en arrivant (rien : tous).
    #[serde(default)]
    pub pv: Option<f32>,
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

    /// IA.5, `design/difficulte.md` : chaque cran d'habileté des difficultés a son profil ; plus la
    /// difficulté monte, plus l'ennemi réagit vite et moins il laisse de temps morts ; seul le haut
    /// du tableau roule ; les sens s'aiguisent un peu et restent bornés.
    #[test]
    fn les_crans_d_habilete_suivent_la_difficulte() {
        let profils = Ron::<Profils>::load_expect("icebreaker.ia.profils").read().0.clone();
        let difficultes = crate::icebreaker_difficulte::IcebreakerDifficulte::actuel();
        let crans: Vec<Profil> = (1..=difficultes.max())
            .map(|n| {
                let cran = difficultes.niveau(Some(n)).ia;
                assert!(profils.crans.contains_key(&cran), "le cran {cran} manque dans profils.ron");
                profils.profil("orc", "guerrier", &cran)
            })
            .collect();
        for paire in crans.windows(2) {
            assert!(paire[0].reaction.unwrap() >= paire[1].reaction.unwrap());
            assert!(paire[0].pause.unwrap() >= paire[1].pause.unwrap());
        }
        let (facile, dur) = (crans.first().unwrap(), crans.last().unwrap());
        assert!(facile.reaction.unwrap() > dur.reaction.unwrap() && facile.pause.unwrap() > dur.pause.unwrap());
        // Le bas du tableau frappe simple (enchaînement borné), le haut enchaîne sans limite.
        assert!(facile.enchainement.is_some() && dur.enchainement.is_none());
        assert_eq!((facile.roule, dur.roule), (Some(false), Some(true)));
        assert_eq!((facile.pare, dur.pare), (Some(false), Some(true)));
        for (a, b) in [(facile.vue, dur.vue), (facile.vue_nette, dur.vue_nette), (facile.ouie, dur.ouie)] {
            let (a, b) = (a.unwrap(), b.unwrap());
            assert!(b >= a && b <= a * 1.3, "sens de {a} à {b}");
        }
        assert!(dur.vue_nette.unwrap() < dur.vue.unwrap());
        // IA.7 : le cran bas encaisse un coup armé ; plus le cran monte, plus il s'en défend.
        let defend = |p: &Profil| {
            let poids = p.reactions.get(&Evenement::CoupArme).cloned().unwrap_or_default();
            let total: f32 = poids.iter().map(|(_, p)| *p).sum();
            if total > 0.0 { poids.iter().filter(|(r, _)| *r != Reaction::Rien).map(|(_, p)| *p).sum::<f32>() / total } else { 0.0 }
        };
        assert_eq!(defend(facile), 0.0);
        for paire in crans.windows(2) {
            assert!(defend(&paire[0]) <= defend(&paire[1]));
        }
        assert!(defend(dur) > 0.0 && defend(dur) < 1.0, "un coup doit pouvoir passer");
        assert!(dur.garde.is_some_and(|g| g > 0.0));
    }

    /// IA.6 : le mage a une table de sorts faite de vrais sorts du bâton, qui couvre toutes les distances
    /// où il se bat (il a toujours un sort gratuit à lancer) ; les autres rôles n'en ont pas.
    #[test]
    fn la_table_de_sorts_du_mage() {
        let mage = profil(Some(&corps(humanoid::Species::Elf)), ("mage", "base")).unwrap();
        let table = mage.sorts.expect("le mage n'a pas de table de sorts");
        for sort in &table {
            let _ = Ron::<crate::comp::CharacterAbility>::load_expect(&format!("common.abilities.staff.{}", sort.sort));
            assert!(sort.min < sort.max && sort.poids > 0.0 && sort.recharge >= 0.0, "{}", sort.sort);
        }
        // L'attaque de base et la secondaire ne coûtent pas d'énergie à lancer et ne rechargent pas : de 3 à 40
        // blocs, l'une des deux porte.
        let gratuits = ["firebomb", "flamethrower"];
        for distance in 3..=40 {
            assert!(
                table.iter().any(|s| gratuits.contains(&s.sort.as_str()) && s.recharge == 0.0 && s.porte(distance as f32)),
                "rien à lancer à {distance} blocs"
            );
        }
        for role in ["guerrier", "archer", "chef"] {
            assert!(profil(Some(&corps(humanoid::Species::Elf)), (role, "base")).unwrap().sorts.is_none(), "{role}");
        }
    }

    /// IA.8, `creatures.md` « Le fuyard n'oublie pas », « Coincé » : celui qui fuit fuit plus longtemps qu'un
    /// combattant ne poursuit ; coincé, il peut se tapir ou forcer le passage ; le chef ne fuit jamais (Q70).
    #[test]
    fn la_fuite_et_le_coince() {
        let danari = profil(Some(&corps(humanoid::Species::Danari)), ("guerrier", "base")).unwrap();
        assert!(danari.fuite.is_some_and(|seuil| seuil > 0.0));
        assert!(danari.fuite_memoire.unwrap() > danari.poursuite.unwrap());
        assert!(danari.coince_temps.is_some() && danari.coince_portee.is_some() && danari.forcer_duree.is_some());
        let coince = &danari.reactions[&Evenement::Coince];
        for reaction in [Reaction::SeTapir, Reaction::Forcer] {
            assert!(coince.iter().any(|(r, poids)| *r == reaction && *poids > 0.0), "{reaction:?}");
        }
        let chef = profil(Some(&corps(humanoid::Species::Danari)), ("chef", "base")).unwrap();
        assert!(!chef.fuite.is_some_and(|seuil| seuil > 0.0), "le chef danari fuit");
    }

    /// IA.9, `creatures.md` « Appel à l'aide », « Moral : la mort du chef » : les fragiles (archer, mage) appellent
    /// à l'aide, les guerriers et le chef défendent ; l'appel s'entend de plus loin qu'un coup ; à la mort du chef,
    /// les fragiles fuient plus longtemps que les guerriers, et le chef lui-même n'est pas concerné.
    #[test]
    fn appel_a_l_aide_et_deroute() {
        let de = |role: &str| profil(Some(&corps(humanoid::Species::Orc)), (role, "base")).unwrap();
        let appelle = |p: &Profil| {
            p.appel_pv.is_some_and(|seuil| seuil > 0.0)
                && p.reactions.get(&Evenement::Touche).is_some_and(|r| r.iter().any(|(r, poids)| *r == Reaction::AppelerALAide && *poids > 0.0))
        };
        assert!(appelle(&de("archer")) && appelle(&de("mage")) && !appelle(&de("guerrier")) && !appelle(&de("chef")));
        assert!(de("guerrier").defend == Some(true) && de("chef").defend == Some(true) && de("archer").defend != Some(true));
        assert!(de("guerrier").defend_duree.is_some_and(|d| d > 0.0) && de("archer").appel_delai.is_some_and(|d| d > 0.0));
        let bruits = bruits();
        assert!(bruits.portee("appel") > bruits.portee("combat") && bruits.portee("chef") > 0.0);
        let deroute = |role: &str| de(role).deroute.unwrap_or(0.0);
        assert!(deroute("guerrier") > 0.0 && deroute("archer") > deroute("guerrier") && deroute("chef") == 0.0);
        assert!(de("guerrier").reactions[&Evenement::ChefTombe].iter().any(|(r, _)| *r == Reaction::Deroute));
    }

    /// Y.4, IA.11 : les orcs s'enragent sous une part de leurs PV ; les autres races non.
    #[test]
    fn seuls_les_orcs_s_enragent() {
        let de = |espece| profil(Some(&corps(espece)), ("guerrier", "base")).unwrap();
        let s_enrage = |p: &Profil| {
            p.pv_bas.is_some_and(|seuil| seuil > 0.0 && seuil < 1.0)
                && p.rage.is_some_and(|r| r.force > 0.0 && r.duree > 0.0)
                && p.reactions.get(&Evenement::PvBas).is_some_and(|r| r.iter().any(|(r, poids)| *r == Reaction::Rage && *poids > 0.0))
        };
        assert!(s_enrage(&de(humanoid::Species::Orc)));
        for espece in [humanoid::Species::Human, humanoid::Species::Dwarf, humanoid::Species::Elf, humanoid::Species::Danari] {
            assert!(!s_enrage(&de(espece)), "{espece:?}");
        }
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
                assert!(g.pv.is_none_or(|pv| pv > 0.0 && pv <= 1.0), "{nom}");
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
