//! Icebreaker : le bilan de retour d'expédition (jalon N, N.8).
//!
//! Quand un joueur rentre au village, ou que l'expédition a péri, le serveur lui
//! envoie un panneau typé (`ServerGeneral::IcebreakerBilan`) : le client le montre
//! dans une fenêtre centrée et la ferme lui-même. Le chat ne garde qu'une ligne.
//! Le panneau est construit ici, à partir de chiffres seulement, pour être testé.

use crate::{
    icebreaker_mission::{Decompte, IcebreakerMission},
    icebreaker_slots::Slot,
    rtsim::RtSim,
};
use common::{
    comp::ItemKey,
    icebreaker_panneau::{Chiffre, Panneau, Sorte, Ton},
};

/// Ce que le joueur ramène de son expédition.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Retour {
    /// Le nom du lieu de l'expédition (« Donjon »).
    pub lieu: String,
    /// Ce que la mission a compté ; rien si le lieu n'a pas de mission.
    pub decompte: Option<Decompte>,
    /// L'or rapporté au village.
    pub or: u64,
    /// Les objets rapportés, avec leur quantité.
    pub objets: Vec<(ItemKey, u32)>,
    /// Le personnage est mort en expédition : il est perdu, son sac reste sur place.
    pub peri: bool,
}

/// Ce que la mission du lieu a compté, d'après les traces du monde (`None` : le lieu n'a pas de
/// mission).
pub fn decompte_de(missions: &IcebreakerMission, rtsim: &RtSim, slot: &Slot) -> Option<Decompte> {
    missions.of(slot.site).map(|mission| {
        mission.decompte(
            |slot| rtsim.is_dungeon_npc_killed(slot),
            |wpos| rtsim.is_dungeon_sprite_collected(wpos),
        )
    })
}

/// Le panneau du bilan : titre, lieu et mission, chiffres, objets rapportés, et le message.
pub fn panneau(retour: &Retour) -> Panneau {
    let mission = if retour.peri {
        Some("L'expédition a péri.")
    } else {
        retour.decompte.map(|d| {
            if d.accomplie {
                "Mission accomplie : le chef est tombé"
            } else {
                "Mission non accomplie : le chef est encore debout"
            }
        })
    };
    let description = match (retour.lieu.as_str(), mission) {
        ("", Some(mission)) => mission.to_string(),
        (lieu, Some(mission)) => format!("{lieu} · {mission}"),
        (lieu, None) => lieu.to_string(),
    };
    let mut panneau = Panneau::nouveau("", "Retour d'expédition", description);
    if let Some(d) = retour.decompte {
        panneau.chiffres.push(Chiffre::nouveau(
            format!("{} / {}", d.tuees, d.creatures),
            "créatures tuées",
            Ton::Normal,
        ));
        panneau.chiffres.push(Chiffre::nouveau(
            format!("{} / {}", d.ouverts, d.coffres),
            "coffres ouverts",
            Ton::Normal,
        ));
    }
    panneau.chiffres.push(Chiffre::nouveau(
        retour.or.to_string(),
        "or rapportés au village",
        Ton::Or,
    ));
    if retour.peri {
        panneau
            .chiffres
            .push(Chiffre::nouveau("1", "personnage perdu", Ton::Mauvais));
    }
    panneau.objets = retour.objets.clone();
    if retour.peri {
        panneau = panneau.message(
            "Mort en expédition : personnage perdu, sac laissé sur place.",
            Sorte::Refus,
        );
    } else if retour.or == 0 && retour.objets.is_empty() {
        panneau = panneau.message("Les mains vides.", Sorte::Info);
    }
    panneau
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::icebreaker_panneau::texte;

    fn clef(id: &str) -> ItemKey { ItemKey::from(&common::comp::Item::new_from_asset_expect(id)) }

    fn decompte(accomplie: bool) -> Decompte {
        Decompte {
            accomplie,
            tuees: 12,
            creatures: 15,
            ouverts: 3,
            coffres: 4,
        }
    }

    fn chiffres(panneau: &Panneau) -> Vec<(String, String, Ton)> {
        let dire = |c: &common::comp::Content| match c {
            common::comp::Content::Plain(s) => s.clone(),
            autre => format!("{autre:?}"),
        };
        panneau
            .chiffres
            .iter()
            .map(|c| (dire(&c.valeur), dire(&c.legende), c.ton))
            .collect()
    }

    #[test]
    fn un_retour_avec_or_et_objets_montre_le_decompte_et_ce_qui_est_rapporte() {
        let pomme = clef("common.items.food.apple");
        let panneau = panneau(&Retour {
            lieu: "Donjon".to_string(),
            decompte: Some(decompte(true)),
            or: 96,
            objets: vec![(pomme.clone(), 2)],
            peri: false,
        });
        assert_eq!(panneau.titre, texte("Retour d'expédition"));
        assert_eq!(
            panneau.description,
            texte("Donjon · Mission accomplie : le chef est tombé")
        );
        assert_eq!(chiffres(&panneau), vec![
            ("12 / 15".to_string(), "créatures tuées".to_string(), Ton::Normal),
            ("3 / 4".to_string(), "coffres ouverts".to_string(), Ton::Normal),
            ("96".to_string(), "or rapportés au village".to_string(), Ton::Or),
        ]);
        assert_eq!(panneau.objets, vec![(pomme, 2)]);
        assert_eq!(panneau.message, None);
        assert!(panneau.actions.is_empty(), "la fermeture est faite par le client");
    }

    #[test]
    fn une_mission_non_accomplie_le_dit() {
        let panneau = panneau(&Retour {
            lieu: "Donjon".to_string(),
            decompte: Some(decompte(false)),
            or: 5,
            ..Default::default()
        });
        assert_eq!(
            panneau.description,
            texte("Donjon · Mission non accomplie : le chef est encore debout")
        );
    }

    #[test]
    fn les_mains_vides_sont_dites() {
        let panneau = panneau(&Retour {
            lieu: "Donjon".to_string(),
            decompte: Some(decompte(false)),
            ..Default::default()
        });
        assert_eq!(panneau.message, Some((texte("Les mains vides."), Sorte::Info)));
        assert!(panneau.objets.is_empty());
        // De l'or seul, ou un objet seul, ce n'est pas les mains vides.
        let or = super::panneau(&Retour { or: 1, ..Default::default() });
        assert_eq!(or.message, None);
        let objet = super::panneau(&Retour {
            objets: vec![(clef("common.items.food.apple"), 1)],
            ..Default::default()
        });
        assert_eq!(objet.message, None);
    }

    #[test]
    fn une_expedition_perie_perd_le_personnage_et_le_sac() {
        let panneau = panneau(&Retour {
            lieu: "Donjon".to_string(),
            decompte: Some(decompte(true)),
            peri: true,
            ..Default::default()
        });
        assert_eq!(panneau.description, texte("Donjon · L'expédition a péri."));
        assert_eq!(
            chiffres(&panneau).last(),
            Some(&("1".to_string(), "personnage perdu".to_string(), Ton::Mauvais))
        );
        assert_eq!(
            panneau.message,
            Some((
                texte("Mort en expédition : personnage perdu, sac laissé sur place."),
                Sorte::Refus
            ))
        );
    }

    #[test]
    fn sans_decompte_pas_de_tuiles_de_creatures_ni_de_coffres() {
        let panneau = panneau(&Retour {
            lieu: "Donjon".to_string(),
            or: 10,
            ..Default::default()
        });
        assert_eq!(panneau.description, texte("Donjon"));
        assert_eq!(chiffres(&panneau), vec![(
            "10".to_string(),
            "or rapportés au village".to_string(),
            Ton::Or
        )]);
        // Un personnage perdu sans lieu connu : la mission seule.
        let perdu = super::panneau(&Retour { peri: true, ..Default::default() });
        assert_eq!(perdu.description, texte("L'expédition a péri."));
    }
}
