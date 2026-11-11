//! Icebreaker : les panneaux typés du plan du village (jalon N, décision 0019).
//!
//! Le client dit ce qu'il a d'ouvert (`Demande`) ; le serveur s'en souvient
//! (`IcebreakerPanneaux`) et envoie les bâtiments et le panneau quand ils
//! changent (`sys::icebreaker_panneaux`), au lieu d'attendre qu'on les lui
//! redemande chaque seconde. Les panneaux des bâtiments simples sont construits
//! ici, à partir de chiffres seulement, pour être testés ; ceux de la charrette
//! et du départ le sont dans `icebreaker_registre` et `icebreaker_expedition`.

use crate::icebreaker_buildings::Place;
use common::{
    comp::ItemKey,
    icebreaker_panneau::{Action, Batiment, Chiffre, Jauge, Panneau, Sorte, Ton, texte},
};
use specs::Entity;
use std::collections::HashMap;

/// Tous les combien de secondes un panneau ouvert est reconstruit pour voir
/// s'il a changé (ce que font les autres joueurs, le temps qui passe) ; une
/// commande du joueur le fait reconstruire tout de suite.
pub const PERIODE: f64 = 0.5;

/// Ce qu'un joueur a d'ouvert, et ce qui lui a été envoyé en dernier.
#[derive(Default)]
pub struct Ouvert {
    /// Le bâtiment dont le panneau est ouvert, s'il y en a un.
    pub place: Option<Place>,
    pub batiments: Option<Vec<Batiment>>,
    pub panneau: Option<Panneau>,
    /// À reconstruire au prochain tic, sans attendre `PERIODE`.
    pub a_revoir: bool,
    /// Quand il a été reconstruit en dernier (`Time`).
    pub vu: f64,
    /// Icebreaker A.12 : ce que la dernière commande `/salle` a fait, montré dans le panneau de la
    /// salle d'armes.
    pub salle_message: Option<(String, Sorte)>,
}

/// Qui a le plan du village ouvert.
#[derive(Default)]
pub struct IcebreakerPanneaux {
    pub ouverts: HashMap<Entity, Ouvert>,
}

impl IcebreakerPanneaux {
    /// Le plan s'ouvre : tout est à envoyer.
    pub fn ouvrir(&mut self, entity: Entity) {
        self.ouverts.insert(entity, Ouvert {
            a_revoir: true,
            ..Default::default()
        });
    }

    /// Un clic sur `place` : son panneau est à envoyer.
    pub fn choisir(&mut self, entity: Entity, place: Place) {
        let ouvert = self.ouverts.entry(entity).or_default();
        ouvert.place = Some(place);
        ouvert.panneau = None;
        ouvert.salle_message = None;
        ouvert.a_revoir = true;
    }

    /// Le panneau se ferme, le plan reste.
    pub fn fermer_panneau(&mut self, entity: Entity) {
        if let Some(ouvert) = self.ouverts.get_mut(&entity) {
            ouvert.place = None;
            ouvert.panneau = None;
        }
    }

    /// Le plan se ferme : plus rien n'est envoyé.
    pub fn fermer(&mut self, entity: Entity) { self.ouverts.remove(&entity); }

    /// Le joueur vient d'agir (une commande) : son panneau est reconstruit au
    /// prochain tic.
    pub fn marquer(&mut self, entity: Entity) {
        if let Some(ouvert) = self.ouverts.get_mut(&entity) {
            ouvert.a_revoir = true;
        }
    }
}

/// Des heures de carte, sans décimale inutile : « 6 h », « 2,5 h ».
pub fn heures(h: f64) -> String {
    if (h - h.round()).abs() < 0.05 {
        format!("{:.0} h", h)
    } else {
        format!("{:.1} h", h).replace('.', ",")
    }
}

/// La caisse du village : ce qu'elle a, ce que le joueur a sur lui, et les
/// versements, grisés quand il n'a pas assez d'or.
pub fn caisse(caisse: u64, bourse: u64, prix_maison: u64) -> Panneau {
    let mut panneau = Panneau::nouveau("lingot", "Caisse", "Elle paie les chantiers du village.");
    panneau.chiffres = vec![
        Chiffre::nouveau(format!("{caisse} or"), "dans la caisse", Ton::Or),
        Chiffre::nouveau(format!("{bourse} or"), "sur vous", Ton::Normal),
    ];
    let assez = caisse >= prix_maison;
    panneau.jauge = Some(Jauge {
        legende: texte(format!("Prochaine maison : {prix_maison} or")),
        texte: texte(if assez {
            "assez".to_string()
        } else {
            format!("manque {} or", prix_maison - caisse)
        }),
        part: (caisse as f32 / prix_maison.max(1) as f32).min(1.0),
        ton: if assez { Ton::Bon } else { Ton::Info },
    });
    panneau.colonnes = 2;
    panneau.actions = [10u64, 50, 100]
        .into_iter()
        .map(|n| {
            Action::nouvelle(format!("Verser {n} or"), &["caisse", &n.to_string()])
                .permise(bourse >= n)
        })
        .chain([Action::nouvelle("Tout verser", &["caisse", "tout"])
            .principale()
            .permise(bourse > 0)])
        .collect();
    match bourse {
        0 => panneau.message("Vous n'avez pas d'or sur vous.", Sorte::Info),
        1..100 => panneau.message(format!("Vous n'avez que {bourse} or sur vous."), Sorte::Info),
        _ => panneau,
    }
}

/// L'infirmerie : la vie du joueur (`vie` : actuelle et maximale ; aucune pour
/// un fantôme), le prix, et le soin, grisé avec sa raison s'il est impossible.
pub fn infirmerie(prix: u64, bourse: u64, vie: Option<(f32, f32)>) -> Panneau {
    let mut panneau = Panneau::nouveau("potion", "Infirmerie", "Un soin complet contre de l'or.");
    let blesse = vie.is_some_and(|(actuelle, maximale)| actuelle < maximale);
    if let Some((actuelle, maximale)) = vie {
        panneau.chiffres.push(Chiffre::nouveau(
            format!("{:.0} / {:.0}", actuelle.ceil(), maximale),
            "vos points de vie",
            if blesse { Ton::Mauvais } else { Ton::Bon },
        ));
    }
    panneau
        .chiffres
        .push(Chiffre::nouveau(format!("{prix} or"), "prix du soin", Ton::Or));
    panneau.actions = vec![
        Action::nouvelle(format!("Me soigner ({prix} or)"), &["soigner"])
            .principale()
            .permise(blesse && bourse >= prix),
    ];
    match vie {
        None => panneau.message(crate::icebreaker_registre::GHOST_REFUSAL, Sorte::Refus),
        Some(_) if !blesse => panneau.message("Vous n'êtes pas blessé.", Sorte::Info),
        Some(_) if bourse < prix => {
            panneau.message(format!("Il vous manque {} or.", prix - bourse), Sorte::Refus)
        },
        Some(_) => panneau.message(format!("Vous avez {bourse} or sur vous."), Sorte::Info),
    }
}

/// Un terrain à bâtir : le prix face à la caisse, la durée du chantier.
pub fn terrain(rang: usize, prix: u64, heures_chantier: f64, caisse: u64) -> Panneau {
    let mut panneau = Panneau::nouveau(
        "buche",
        "Terrain à bâtir",
        "Une maison de plus, puis de nouveaux habitants.",
    );
    panneau.chiffres = vec![
        Chiffre::nouveau(format!("{prix} or"), "pris dans la caisse", Ton::Or),
        Chiffre::nouveau(heures(heures_chantier), "de chantier", Ton::Normal),
    ];
    let assez = caisse >= prix;
    panneau.jauge = Some(Jauge {
        legende: texte("Caisse du village"),
        texte: texte(format!("{caisse} / {prix} or")),
        part: (caisse as f32 / prix.max(1) as f32).min(1.0),
        ton: if assez { Ton::Bon } else { Ton::Info },
    });
    panneau.actions = vec![
        Action::nouvelle(format!("Bâtir ({prix} or)"), &["batir", &rang.to_string()])
            .principale()
            .permise(assez),
    ];
    if assez {
        panneau
    } else {
        panneau.message(
            format!("Il manque {} or dans la caisse.", prix - caisse),
            Sorte::Refus,
        )
    }
}

/// Un chantier : ce qu'il reste, et l'avancement.
pub fn chantier(reste: f64, heures_chantier: f64) -> Panneau {
    let mut panneau = Panneau::nouveau("outils", "Chantier", "Une maison est en construction.");
    panneau.chiffres = vec![Chiffre::nouveau(
        heures(reste.ceil()),
        "avant la fin, en heures de carte",
        Ton::Normal,
    )];
    let fait = (heures_chantier - reste).max(0.0);
    panneau.jauge = Some(Jauge {
        legende: texte("Avancement"),
        texte: texte(format!("{} sur {}", heures(fait), heures(heures_chantier))),
        part: (fait / heures_chantier.max(0.01)) as f32,
        ton: Ton::Info,
    });
    panneau.message("De nouveaux habitants arriveront à la fin.", Sorte::Info)
}

/// L'entrepôt : ce qu'il garde, en cases. `fantome` : sans personnage, rien à
/// déposer.
pub fn entrepot(objets: Vec<(ItemKey, u32)>, fantome: bool) -> Panneau {
    let mut panneau =
        Panneau::nouveau("utilitaire", "Entrepôt", "Ce que le village garde en commun.");
    let (sortes, total) = (objets.len(), objets.iter().map(|(_, n)| u64::from(*n)).sum::<u64>());
    panneau.objets = objets;
    panneau.actions = vec![
        Action::nouvelle("Déposer ou reprendre", &["echanger", "entrepot"])
            .principale()
            .permise(!fantome),
    ];
    if fantome {
        panneau.message(crate::icebreaker_registre::GHOST_REFUSAL, Sorte::Refus)
    } else if sortes == 0 {
        panneau.message("L'entrepôt est vide.", Sorte::Info)
    } else {
        panneau.message(
            format!("{sortes} sortes d'objets, {total} en tout."),
            Sorte::Info,
        )
    }
}

/// L'intendance : acheter et vendre.
pub fn intendance(bourse: u64, fantome: bool) -> Panneau {
    let mut panneau = Panneau::nouveau(
        "vivres",
        "Intendance",
        "Potions et vivres, aux prix de la ville.",
    );
    panneau.chiffres = vec![Chiffre::nouveau(format!("{bourse} or"), "sur vous", Ton::Normal)];
    panneau.actions = vec![
        Action::nouvelle("Acheter ou vendre", &["echanger", "intendance"])
            .principale()
            .permise(!fantome),
    ];
    if fantome {
        panneau.message(crate::icebreaker_registre::GHOST_REFUSAL, Sorte::Refus)
    } else {
        panneau.message("L'intendant rachète ce que vous rapportez.", Sorte::Info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icebreaker_buildings::Building;
    use specs::{Builder, World, WorldExt};

    fn permises(panneau: &Panneau) -> Vec<bool> {
        panneau.actions.iter().map(|action| action.permise).collect()
    }

    fn message(panneau: &Panneau) -> (String, Sorte) {
        match &panneau.message {
            Some((common::comp::Content::Plain(message), sorte)) => (message.clone(), *sorte),
            other => panic!("no plain message: {other:?}"),
        }
    }

    #[test]
    fn la_caisse_grise_les_versements_plus_gros_que_la_bourse() {
        let panneau = caisse(420, 60, 300);
        assert_eq!(permises(&panneau), vec![true, true, false, true]);
        assert!(panneau.actions[3].principale, "tout verser");
        assert_eq!(panneau.colonnes, 2);
        assert_eq!(message(&panneau), ("Vous n'avez que 60 or sur vous.".into(), Sorte::Info));
        let jauge = panneau.jauge.unwrap();
        assert_eq!(jauge.part, 1.0);
        assert_eq!(jauge.ton, Ton::Bon);

        let vide = caisse(120, 0, 300);
        assert_eq!(permises(&vide), vec![false; 4]);
        assert_eq!(message(&vide).0, "Vous n'avez pas d'or sur vous.");
        assert!((vide.jauge.unwrap().part - 0.4).abs() < 1e-6);

        assert!(caisse(0, 500, 300).message.is_none(), "rien à dire quand tout est permis");
    }

    #[test]
    fn l_infirmerie_dit_pourquoi_le_soin_est_grise() {
        let pas_blesse = infirmerie(60, 500, Some((100.0, 100.0)));
        assert_eq!(permises(&pas_blesse), vec![false]);
        assert_eq!(message(&pas_blesse).0, "Vous n'êtes pas blessé.");

        let pauvre = infirmerie(60, 40, Some((62.0, 100.0)));
        assert_eq!(permises(&pauvre), vec![false]);
        assert_eq!(message(&pauvre), ("Il vous manque 20 or.".into(), Sorte::Refus));
        assert_eq!(pauvre.chiffres[0].valeur, texte("62 / 100"));
        assert_eq!(pauvre.chiffres[0].ton, Ton::Mauvais);

        let soignable = infirmerie(60, 60, Some((62.0, 100.0)));
        assert_eq!(permises(&soignable), vec![true]);

        let fantome = infirmerie(60, 0, None);
        assert_eq!(permises(&fantome), vec![false]);
        assert_eq!(fantome.chiffres.len(), 1, "pas de vie à montrer");
        assert_eq!(message(&fantome).1, Sorte::Refus);
    }

    #[test]
    fn un_terrain_ne_se_batit_que_si_la_caisse_suffit() {
        let trop_cher = terrain(7, 300, 6.0, 120);
        assert_eq!(permises(&trop_cher), vec![false]);
        assert_eq!(message(&trop_cher), ("Il manque 180 or dans la caisse.".into(), Sorte::Refus));
        assert_eq!(trop_cher.actions[0].commande, vec!["batir".to_string(), "7".to_string()]);
        assert_eq!(trop_cher.chiffres[1].valeur, texte("6 h"));

        let possible = terrain(7, 300, 6.0, 300);
        assert_eq!(permises(&possible), vec![true]);
        assert!(possible.message.is_none());
    }

    #[test]
    fn un_chantier_montre_ce_qui_reste_et_son_avancement() {
        let panneau = chantier(3.6, 6.0);
        assert_eq!(panneau.chiffres[0].valeur, texte("4 h"));
        let jauge = panneau.jauge.unwrap();
        assert!((jauge.part - 0.4).abs() < 1e-6, "{jauge:?}");
        assert_eq!(jauge.texte, texte("2,4 h sur 6 h"));
        assert!(panneau.actions.is_empty());
    }

    #[test]
    fn un_fantome_ne_depose_rien_a_l_entrepot_ni_a_l_intendance() {
        assert_eq!(permises(&entrepot(Vec::new(), false)), vec![true]);
        assert_eq!(message(&entrepot(Vec::new(), false)).0, "L'entrepôt est vide.");
        assert_eq!(permises(&entrepot(Vec::new(), true)), vec![false]);
        assert_eq!(permises(&intendance(60, false)), vec![true]);
        assert_eq!(permises(&intendance(0, true)), vec![false]);
    }

    #[test]
    fn le_serveur_se_souvient_de_ce_que_chacun_a_d_ouvert() {
        let mut world = World::new();
        let joueur = world.create_entity().build();
        let caisse = Place::Building(Building::Treasury);
        let mut panneaux = IcebreakerPanneaux::default();

        // Rien d'ouvert : une commande ne marque rien.
        panneaux.marquer(joueur);
        assert!(panneaux.ouverts.is_empty());

        panneaux.ouvrir(joueur);
        assert!(panneaux.ouverts[&joueur].a_revoir);
        assert_eq!(panneaux.ouverts[&joueur].place, None);

        panneaux.ouverts.get_mut(&joueur).unwrap().a_revoir = false;
        panneaux.choisir(joueur, caisse);
        assert_eq!(panneaux.ouverts[&joueur].place, Some(caisse));
        assert!(panneaux.ouverts[&joueur].a_revoir);

        panneaux.ouverts.get_mut(&joueur).unwrap().a_revoir = false;
        panneaux.marquer(joueur);
        assert!(panneaux.ouverts[&joueur].a_revoir, "une commande fait reconstruire");

        panneaux.fermer_panneau(joueur);
        assert_eq!(panneaux.ouverts[&joueur].place, None);
        panneaux.fermer(joueur);
        assert!(panneaux.ouverts.is_empty());
    }

    #[test]
    fn les_heures_s_ecrivent_sans_decimale_inutile() {
        assert_eq!(heures(6.0), "6 h");
        assert_eq!(heures(2.5), "2,5 h");
    }
}
