//! Icebreaker A.11 et A.12 : la salle d'armes du village (jalon A, `docs/icebreaker/PLAN-CLASSES.md`).
//!
//! On y dépense les points d'arme d'un jeu d'armes (débloquer une compétence, monter son niveau),
//! on y place ses deux compétences d'arme, et avec de l'or on y apprend un jeu d'armes, un ultime ou
//! un passif. Le panneau (`panneau`) est une fonction pure de ce que le personnage a acquis, de ce
//! qu'il tient et de sa bourse ; les règles (`debloquer`, `monter`, `placer`, `apprendre_*`,
//! `activer_*`) sont des fonctions pures sur [`Acquis`] qui rendent le message de réussite ou le
//! refus. Les prix et les coûts viennent de `assets/icebreaker/salle_armes.ron`.

use common::{
    comp::{
        Inventory,
        inventory::{
            item::{ItemKind, tool::ToolKind},
            slot::EquipSlot,
        },
    },
    icebreaker_competences::{
        Acquis, AcquisJeu, IcebreakerCompetences, JeuArmes, Registre, SalleArmes, jeu_en_main,
        registre,
    },
    icebreaker_panneau::{Chiffre, Jauge, Ligne, Onglet, Panneau, Sorte, Ton, texte},
};

/// L'icône du HUD du panneau (celles de `vue_village.ron`).
const ICONE: &str = "epees";

/// « 1 point », « 2 points ».
fn points(n: u32) -> String { format!("{n} point{}", if n > 1 { "s" } else { "" }) }

/// Des secondes de recharge, sans décimale inutile : « 10 s », « 7,5 s ».
fn secondes(s: f32) -> String {
    if (s - s.round()).abs() < 0.05 {
        format!("{:.0} s", s)
    } else {
        format!("{:.1} s", s).replace('.', ",")
    }
}

/// Ce qu'on tient en main droite, pour le dire à un joueur dont l'arme n'est pas un jeu d'armes.
fn arme_tenue(inventaire: Option<&Inventory>) -> Option<&'static str> {
    let objet = inventaire?.equipped(EquipSlot::ActiveMainhand)?;
    let ItemKind::Tool(outil) = &*objet.kind() else {
        return None;
    };
    Some(match outil.kind {
        ToolKind::Sword => "une épée",
        ToolKind::Axe => "une hache",
        ToolKind::Hammer => "un marteau",
        ToolKind::Bow => "un arc",
        ToolKind::Staff => "un bâton",
        ToolKind::Sceptre => "un sceptre",
        ToolKind::Dagger => "une dague",
        ToolKind::Shield => "un bouclier",
        ToolKind::Spear => "une lance",
        _ => "cet objet",
    })
}

/// Un panneau qui n'a rien à montrer sinon pourquoi.
fn refus(message: &str) -> Panneau {
    Panneau::nouveau(ICONE, "Salle d'armes", "").message(message, Sorte::Refus)
}

/// Le jeu d'armes en mains, s'il est appris : (id, définition, ce qui est acquis).
type EnMains<'a> = (&'a str, &'a JeuArmes, &'a AcquisJeu);

/// Les compétences du jeu d'armes en mains : une ligne par compétence du jeu.
fn onglet_competences(en_mains: Option<EnMains<'_>>) -> Vec<Ligne> {
    let Some((_, def, acquis)) = en_mains else {
        return vec![Ligne::nouvelle("Prenez en mains une arme que vous savez manier.").estompee()];
    };
    def.competences
        .iter()
        .map(|(id, competence)| {
            let ligne = Ligne::nouvelle(competence.nom.clone()).icone(def.icone.clone());
            let Some(niveau) = acquis.competences.get(id).copied() else {
                return ligne.sous_titre("à débloquer").estompee();
            };
            let touche = acquis
                .placees
                .iter()
                .position(|placee| placee.as_deref() == Some(id.as_str()));
            let ligne = ligne.sous_titre(format!(
                "niveau {niveau} · recharge {}",
                secondes(competence.effectif(niveau).recharge)
            ));
            match touche {
                Some(touche) => ligne.pastille(format!("touche {}", touche + 1), Ton::Info),
                None if niveau >= competence.niveau_max() => ligne.pastille("niveau max", Ton::Bon),
                None => ligne,
            }
        })
        .collect()
}

/// Les jeux d'armes de la classe.
fn onglet_armes(
    registre: &Registre,
    classe: &str,
    acquis: &Acquis,
    en_mains: Option<&str>,
    salle: &SalleArmes,
) -> Vec<Ligne> {
    registre
        .jeux_de(classe)
        .map(|(id, jeu)| {
            let ligne = Ligne::nouvelle(jeu.nom.clone()).icone(jeu.icone.clone());
            let ligne = match acquis.jeux.get(id) {
                Some(appris) => ligne.sous_titre(format!(
                    "maîtrise {} · {}",
                    appris.maitrise,
                    points(appris.points_disponibles(salle))
                )),
                None => ligne.sous_titre("à apprendre"),
            };
            if en_mains == Some(id) {
                ligne.pastille("en mains", Ton::Bon)
            } else {
                ligne
            }
        })
        .collect()
}

/// Les ultimes de la classe.
fn onglet_ultimes(registre: &Registre, classe: &str, acquis: &Acquis) -> Vec<Ligne> {
    registre
        .ultimes_de(classe)
        .iter()
        .map(|(id, ultime)| {
            let ligne = Ligne::nouvelle(ultime.nom.clone())
                .sous_titre(format!(
                    "{} · recharge {}",
                    ultime.description,
                    secondes(ultime.recharge)
                ));
            if acquis.ultime.as_deref() == Some(id.as_str()) {
                ligne.pastille("actif", Ton::Bon)
            } else {
                ligne
            }
        })
        .collect()
}

/// Les passifs de la classe.
fn onglet_passifs(registre: &Registre, classe: &str, acquis: &Acquis) -> Vec<Ligne> {
    registre
        .passifs_de(classe)
        .map(|passifs| passifs.passifs.as_slice())
        .unwrap_or_default()
        .iter()
        .map(|(id, passif)| {
            let ligne = Ligne::nouvelle(passif.nom.clone()).sous_titre(passif.description.clone());
            if acquis.passif.as_deref() == Some(id.as_str()) {
                ligne.pastille("actif", Ton::Bon)
            } else {
                ligne
            }
        })
        .collect()
}

/// Le panneau de la salle d'armes pour un personnage : `competences` (aucune : un fantôme, ou une
/// classe que nos données ne décrivent pas, dit par `fantome`), ce qu'il tient (`inventaire`) et
/// l'or de sa bourse.
pub fn panneau(
    competences: Option<&IcebreakerCompetences>,
    inventaire: Option<&Inventory>,
    bourse: u64,
    fantome: bool,
    salle: &SalleArmes,
    message: Option<(String, Sorte)>,
) -> Panneau {
    let Some(competences) = competences else {
        return refus(if fantome {
            crate::icebreaker_registre::GHOST_REFUSAL
        } else {
            "Cette classe n'a pas encore de compétences à régler ici."
        });
    };
    let registre = registre();
    let (classe, acquis) = (competences.classe.as_str(), &competences.acquis);
    let tenu = jeu_en_main(classe, inventaire);
    let en_mains: Option<EnMains<'_>> =
        tenu.and_then(|(id, def)| acquis.jeux.get(id).map(|appris| (id, def, appris)));

    let description = match (en_mains, tenu) {
        (Some((_, def, appris)), _) => format!(
            "{} : {} à dépenser.",
            def.nom,
            points(appris.points_disponibles(salle))
        ),
        (None, Some((_, def))) => format!("Vous tenez « {} » : arme non apprise.", def.nom),
        (None, None) => match arme_tenue(inventaire) {
            Some(arme) => format!("Vous tenez {arme} : arme non apprise."),
            None => "Prenez une arme en mains pour voir ses compétences.".to_string(),
        },
    };
    let mut panneau = Panneau::nouveau(ICONE, "Salle d'armes", description);
    if let Some((_, _, appris)) = en_mains {
        let disponibles = appris.points_disponibles(salle);
        panneau.chiffres.push(Chiffre::nouveau(
            disponibles.to_string(),
            "points d'arme",
            if disponibles > 0 { Ton::Bon } else { Ton::Normal },
        ));
        let (_, reste, cout) = salle.progression(appris.maitrise);
        panneau.jauge = Some(Jauge {
            legende: texte("Maîtrise"),
            texte: texte(format!("{reste} / {cout} vers le prochain point")),
            part: (reste as f32 / cout.max(1) as f32).min(1.0),
            ton: Ton::Info,
        });
    }
    panneau
        .chiffres
        .push(Chiffre::nouveau(format!("{bourse} or"), "sur vous", Ton::Or));

    panneau.onglets = vec![
        Onglet {
            libelle: texte("Compétences"),
            lignes: onglet_competences(en_mains),
        },
        Onglet {
            libelle: texte("Armes"),
            lignes: onglet_armes(registre, classe, acquis, tenu.map(|(id, _)| id), salle),
        },
        Onglet {
            libelle: texte("Ultimes"),
            lignes: onglet_ultimes(registre, classe, acquis),
        },
        Onglet {
            libelle: texte("Passifs"),
            lignes: onglet_passifs(registre, classe, acquis),
        },
    ];
    if let Some((texte, sorte)) = message {
        panneau = panneau.message(texte, sorte);
    }
    panneau
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{
        comp::{Item, inventory::loadout_builder::LoadoutBuilder},
        icebreaker_competences::registre,
    };

    /// Un inventaire qui tient les armes du jeu d'armes `jeu` (aucun : mains vides).
    fn inventaire(jeu: Option<&str>) -> Inventory {
        let armes = jeu.and_then(|jeu| registre().jeu(jeu)).map(|jeu| jeu.armes.clone());
        let loadout = LoadoutBuilder::empty()
            .active_mainhand(armes.as_ref().map(|a| Item::new_from_asset_expect(&a.0)))
            .active_offhand(
                armes
                    .as_ref()
                    .and_then(|a| a.1.as_deref())
                    .map(Item::new_from_asset_expect),
            )
            .build();
        Inventory::with_loadout_humanoid(loadout)
    }

    /// Un guerrier qui connaît la grande épée (balayage lourd et tourbillon, placés) avec `maitrise`.
    fn guerrier(maitrise: u32) -> IcebreakerCompetences {
        let mut acquis = Acquis::lot(
            "grande_epee",
            &["balayage_lourd", "tourbillon"],
            Some("fureur"),
            Some("robuste"),
        );
        acquis.jeux.get_mut("grande_epee").unwrap().maitrise = maitrise;
        IcebreakerCompetences::nouveau("guerrier", acquis).expect("guerrier décrit")
    }

    fn lignes(panneau: &Panneau, onglet: usize) -> &[Ligne] { &panneau.onglets[onglet].lignes }

    fn ligne<'a>(lignes: &'a [Ligne], titre: &str) -> &'a Ligne {
        lignes
            .iter()
            .find(|l| l.titre == texte(titre))
            .unwrap_or_else(|| panic!("pas de ligne {titre} dans {lignes:?}"))
    }

    fn message(panneau: &Panneau) -> (String, Sorte) {
        match &panneau.message {
            Some((common::comp::Content::Plain(m), sorte)) => (m.clone(), *sorte),
            autre => panic!("pas de message : {autre:?}"),
        }
    }

    #[test]
    fn le_panneau_montre_les_points_la_bourse_et_la_maitrise() {
        let salle = SalleArmes::actuel();
        // 100 de maîtrise : le premier point ; 50 de plus sur le suivant (150).
        let competences = guerrier(150);
        let inventaire = inventaire(Some("grande_epee"));
        let p = panneau(Some(&competences), Some(&inventaire), 120, false, &salle, None);
        assert_eq!(p.description, texte("Grande épée : 1 point à dépenser."));
        assert_eq!(p.chiffres[0].valeur, texte("1"));
        assert_eq!(p.chiffres[1].valeur, texte("120 or"));
        assert_eq!(p.chiffres[1].ton, Ton::Or);
        let jauge = p.jauge.expect("jauge");
        assert_eq!(jauge.texte, texte("50 / 150 vers le prochain point"));
        assert!((jauge.part - 50.0 / 150.0).abs() < 1e-6);
        assert_eq!(p.onglets.len(), 4);
        assert!(p.message.is_none());
        assert!(p.actions.is_empty(), "A.11 : rien à faire encore");
    }

    #[test]
    fn l_onglet_des_competences_dit_le_niveau_la_touche_et_ce_qui_reste_a_debloquer() {
        let salle = SalleArmes::actuel();
        let mut competences = guerrier(0);
        {
            let jeu = competences.acquis.jeux.get_mut("grande_epee").unwrap();
            let max = registre().competence("balayage_lourd").unwrap().competence.niveau_max();
            jeu.competences.insert("balayage_lourd".into(), max);
            jeu.competences.insert("tourbillon".into(), 2);
            jeu.placees = [None, Some("tourbillon".into())];
        }
        let inventaire = inventaire(Some("grande_epee"));
        let p = panneau(Some(&competences), Some(&inventaire), 0, false, &salle, None);
        let l = lignes(&p, 0);
        let def = registre().jeu("grande_epee").unwrap();
        assert_eq!(l.len(), def.competences.len(), "une ligne par compétence du jeu");

        let tourbillon = ligne(l, "Tourbillon");
        assert_eq!(tourbillon.sous_titre, texte("niveau 2 · recharge 10 s"));
        assert_eq!(tourbillon.pastille, Some((texte("touche 2"), Ton::Info)));
        assert!(!tourbillon.estompee);

        let balayage = ligne(l, "Balayage lourd");
        assert_eq!(balayage.pastille, Some((texte("niveau max"), Ton::Bon)));

        let inconnue = ligne(l, "Fendeur de terre");
        assert_eq!(inconnue.sous_titre, texte("à débloquer"));
        assert!(inconnue.estompee && inconnue.pastille.is_none());
    }

    #[test]
    fn l_onglet_des_armes_dit_la_maitrise_les_points_et_ce_qu_on_tient() {
        let salle = SalleArmes::actuel();
        let competences = guerrier(320);
        let inventaire = inventaire(Some("grande_epee"));
        let p = panneau(Some(&competences), Some(&inventaire), 0, false, &salle, None);
        let l = lignes(&p, 1);
        assert_eq!(l.len(), registre().jeux_de("guerrier").count());
        let epee = ligne(l, "Grande épée");
        // 320 : 100 + 150 = 250 → 2 points gagnés.
        assert_eq!(epee.sous_titre, texte("maîtrise 320 · 2 points"));
        assert_eq!(epee.pastille, Some((texte("en mains"), Ton::Bon)));
        let hache = ligne(l, "Hache à deux mains");
        assert_eq!(hache.sous_titre, texte("à apprendre"));
        assert!(hache.pastille.is_none());
    }

    #[test]
    fn les_ultimes_et_les_passifs_montrent_celui_qui_est_actif() {
        let salle = SalleArmes::actuel();
        let competences = guerrier(0);
        let inventaire = inventaire(Some("grande_epee"));
        let p = panneau(Some(&competences), Some(&inventaire), 0, false, &salle, None);
        let ultimes = lignes(&p, 2);
        assert_eq!(ultimes.len(), registre().ultimes_de("guerrier").len());
        let fureur = ligne(ultimes, "Fureur");
        assert_eq!(fureur.pastille, Some((texte("actif"), Ton::Bon)));
        let (_, ultime) = &registre().ultimes_de("guerrier")[0];
        assert_eq!(
            fureur.sous_titre,
            texte(format!("{} · recharge {}", ultime.description, secondes(ultime.recharge)))
        );
        assert_eq!(ultimes.iter().filter(|l| l.pastille.is_some()).count(), 1);

        let passifs = lignes(&p, 3);
        let robuste = ligne(passifs, "Robuste");
        assert_eq!(robuste.pastille, Some((texte("actif"), Ton::Bon)));
        assert_eq!(passifs.iter().filter(|l| l.pastille.is_some()).count(), 1);
        assert!(!robuste.sous_titre.eq(&texte("")), "sa description");
    }

    #[test]
    fn une_arme_non_apprise_ou_des_mains_vides_se_disent() {
        let salle = SalleArmes::actuel();
        let competences = guerrier(0);

        // Une hache : un jeu d'armes du guerrier, mais pas appris.
        let hache = inventaire(Some("hache"));
        let p = panneau(Some(&competences), Some(&hache), 0, false, &salle, None);
        assert_eq!(p.description, texte("Vous tenez « Hache à deux mains » : arme non apprise."));
        assert_eq!(p.chiffres.len(), 1, "pas de points à montrer");
        assert!(p.jauge.is_none());
        assert_eq!(lignes(&p, 0).len(), 1);
        assert!(lignes(&p, 0)[0].estompee);
        assert_eq!(p.onglets.len(), 4, "on peut quand même apprendre ce jeu");

        let vide = inventaire(None);
        let p = panneau(Some(&competences), Some(&vide), 0, false, &salle, None);
        assert_eq!(p.description, texte("Prenez une arme en mains pour voir ses compétences."));
    }

    #[test]
    fn un_fantome_ou_une_classe_sans_competences_est_refuse() {
        let salle = SalleArmes::actuel();
        let p = panneau(None, None, 0, true, &salle, None);
        assert_eq!(message(&p), (crate::icebreaker_registre::GHOST_REFUSAL.to_string(), Sorte::Refus));
        assert!(p.onglets.is_empty());
        let p = panneau(None, None, 0, false, &salle, None);
        assert_eq!(message(&p).1, Sorte::Refus);
        assert_ne!(message(&p).0, crate::icebreaker_registre::GHOST_REFUSAL);
    }
}
