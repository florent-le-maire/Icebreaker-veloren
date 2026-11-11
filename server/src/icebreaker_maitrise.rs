//! Icebreaker : la maîtrise d'arme (jalon A, A.9). En expédition, l'expérience des ennemis tués fait
//! monter la maîtrise du jeu d'armes que le personnage tient (`events/entity_manipulation.rs`) ;
//! la maîtrise donne des points d'arme, dépensés à la salle d'armes. Ici : ce que le serveur en
//! dit au joueur. Règles et réglages : `common::icebreaker_competences`,
//! `assets/icebreaker/salle_armes.ron`.

use crate::{icebreaker_bilan::Maitrise, icebreaker_registre::IcebreakerRegistre};
use common::{
    comp::{Presence, PresenceKind},
    icebreaker_competences::{IcebreakerCompetences, SalleArmes, registre},
};
use specs::{Entity, World, WorldExt};

/// Le nom d'un jeu d'armes pour les joueurs (son id s'il n'est plus dans les données).
fn nom_du_jeu(jeu: &str) -> String {
    registre()
        .jeu(jeu)
        .map_or_else(|| jeu.to_string(), |def| def.nom.clone())
}

/// Ce qu'on dit au joueur dont le jeu d'armes `jeu` vient de gagner `points` points d'arme.
pub fn annonce_point(jeu: &str, points: u32) -> String {
    let gagne = if points > 1 {
        format!("{points} points d'arme gagnés")
    } else {
        "un point d'arme gagné".to_string()
    };
    format!("{} : {gagne}. À dépenser à la salle d'armes.", nom_du_jeu(jeu))
}

/// Pour le bilan de retour : la maîtrise que le personnage joué par `entity` a gagnée depuis que sa
/// fiche a été écrite (son départ en expédition), par jeu d'armes.
pub fn progres_de(ecs: &World, entity: Entity) -> Vec<Maitrise> {
    let Some(PresenceKind::Character(id)) = ecs
        .read_storage::<Presence>()
        .get(entity)
        .map(|presence| presence.kind)
    else {
        return Vec::new();
    };
    let Some(avant) = ecs
        .read_resource::<IcebreakerRegistre>()
        .fiche(id)
        .map(|fiche| fiche.acquis_ou_defaut())
    else {
        return Vec::new();
    };
    let competences = ecs.read_storage::<IcebreakerCompetences>();
    let Some(competences) = competences.get(entity) else {
        return Vec::new();
    };
    competences
        .acquis
        .progres_depuis(&avant, &SalleArmes::actuel())
        .into_iter()
        .map(|progres| Maitrise {
            jeu: nom_du_jeu(&progres.jeu),
            gain: progres.gain,
            points: progres.points,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icebreaker_maitrise_un_point_gagne_s_annonce() {
        assert_eq!(
            annonce_point("grande_epee", 1),
            "Grande épée : un point d'arme gagné. À dépenser à la salle d'armes."
        );
        assert_eq!(
            annonce_point("marteau", 2),
            "Marteau à deux mains : 2 points d'arme gagnés. À dépenser à la salle d'armes."
        );
        // Un jeu d'armes disparu des données : son id, sans planter.
        assert!(annonce_point("inconnu", 1).starts_with("inconnu : "));
    }
}
