//! Icebreaker : la décision de nos ennemis humanoïdes (jalon IA, `docs/icebreaker/PLAN-IA.md`).
//! La logique est dans le crate `icebreaker-ia` ; ici, le chargement de ses réglages
//! (`assets/icebreaker/ia/profils.ron`) et l'interrupteur d'affichage du lieu d'entraînement.

use crate::{
    assets::{AssetExt, Ron},
    comp::Body,
    icebreaker_races::nom,
};
pub use icebreaker_ia::{
    Apercu, Cerveau, Etat, Id, Intention, Niveau, Observation, Percu, Profil, Profils, niveau,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Le profil de la race d'un corps (rien hors humanoïde, ou si le fichier manque). Relu à chaque
/// appel : un réglage changé dans le fichier vaut tout de suite.
pub fn profil(body: Option<&Body>) -> Option<Profil> {
    let Some(Body::Humanoid(humanoide)) = body else {
        return None;
    };
    let profils = Ron::<Profils>::load("icebreaker.ia.profils").ok()?;
    Some(profils.read().0.profil(nom(humanoide.species)))
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
        let orc = profil(Some(&corps(humanoid::Species::Orc))).unwrap();
        assert!(orc.repartition.is_some_and(|r| r > 0.0) && orc.reciblage.is_some());
        // Tous voient : net de moins loin que soupçonné, dans un cône, et un peu dans leur dos.
        let (vue, nette) = (orc.vue.unwrap(), orc.vue_nette.unwrap());
        assert!(nette < vue && orc.dos.unwrap() < nette && orc.cone.is_some_and(|c| c > 0.0 && c < 360.0));
        // Tous cherchent une cible perdue : poursuite, trajet, fouille, puis retour.
        assert!(orc.poursuite.is_some() && orc.fouille.is_some() && orc.soupcon.is_some());
        assert!(orc.trajet.unwrap() > orc.poursuite.unwrap() && orc.arrivee.is_some_and(|a| a > 0.0));
        // Y.1, Y.2 : les danari achèvent à plusieurs et fuient blessés ; pas les orcs.
        let danari = profil(Some(&corps(humanoid::Species::Danari))).unwrap();
        assert!(danari.acheve_a(2) && !danari.acheve_a(1));
        assert!(danari.fuite.is_some() && orc.fuite.is_none());
    }
}
