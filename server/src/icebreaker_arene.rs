//! Icebreaker : le lieu d'entraînement (jalon O, `docs/icebreaker/PLAN-ENTRAINEMENT.md`,
//! `assets/icebreaker/arene.ron`). Lancé seul (`icebreaker.ps1 entrainement`), sans registre ni
//! charrette : on y entre avec tout ce que sa classe peut apprendre, et `/entrainement` change
//! classe, armes, compétences et niveaux, fait venir ennemis et alliés.

use crate::{
    icebreaker_classes::{self, Class},
    icebreaker_salle::{activer_passif, activer_ultime, competence_du_jeu},
};
use common::{
    comp::Inventory,
    icebreaker_arene::AreneConfig,
    icebreaker_competences::{
        Acquis, IcebreakerCompetences, IcebreakerPassif, JeuArmes, NIVEAU_DEPART, PLACES_ARME,
        jeu_en_main, registre,
    },
};
use specs::{Entity, World, WorldExt};

/// La classe et le jeu d'armes d'un personnage qui arrive avec `inventaire` : la classe dont il
/// tient un jeu d'armes, sinon la classe par défaut et son jeu par défaut. `None` si nos données
/// ne décrivent pas la classe par défaut.
fn classe_tenue(
    inventaire: &Inventory,
    classes: &[Class],
    config: &AreneConfig,
) -> Option<(Class, &'static JeuArmes)> {
    classes
        .iter()
        .find_map(|class| {
            let (_, jeu) = jeu_en_main(&class.id, Some(inventaire))?;
            Some((class.clone(), jeu))
        })
        .or_else(|| {
            let class = classes.iter().find(|class| class.id == config.classe_par_defaut)?;
            let (_, jeu) = registre().jeu_par_defaut(&class.id)?;
            Some((class.clone(), jeu))
        })
}

/// Ce avec quoi un personnage de `class` joue au lieu d'entraînement, `jeu` en mains : tout ce que
/// sa classe peut apprendre, au niveau `niveau`, et l'équipement de la classe.
pub fn fiche(class: &Class, jeu: &JeuArmes, niveau: u8) -> Option<(IcebreakerCompetences, Inventory)> {
    let competences = IcebreakerCompetences::nouveau(&class.id, Acquis::complet(&class.id, niveau))?;
    Some((competences, icebreaker_classes::inventaire_avec_jeu(class, jeu)))
}

/// Au lieu d'entraînement, ce avec quoi un personnage entre en jeu : `inventaire` devient
/// l'équipement de sa classe (celle de l'arme qu'il tenait, sinon la classe par défaut). `None`
/// ailleurs : rien ne change.
pub fn arrivee(inventaire: &mut Inventory) -> Option<IcebreakerCompetences> {
    if !world::icebreaker::is_entrainement() {
        return None;
    }
    let config = common::icebreaker_arene::config();
    let (class, jeu) = classe_tenue(inventaire, &icebreaker_classes::classes(), &config)?;
    let (competences, equipement) = fiche(&class, jeu, config.niveau)?;
    *inventaire = equipement;
    Some(competences)
}

// ---------------------------------------------------------------------------------------------
// O.3 : régler le personnage. Les règles sont celles de la salle d'armes, sans les prix.
// ---------------------------------------------------------------------------------------------

/// Met au niveau `niveau` (ramené entre 1 et le plus haut niveau de chacune) la compétence `cible`
/// du jeu d'armes `jeu`, ou toutes ses compétences si `cible` vaut « tout ».
pub fn regler_niveau(acquis: &mut Acquis, jeu: &str, cible: &str, niveau: u8) -> Result<String, String> {
    let def = registre().jeu(jeu).ok_or_else(|| "Jeu d'armes inconnu.".to_string())?;
    let appris = acquis
        .jeux
        .get_mut(jeu)
        .ok_or_else(|| "Vous ne connaissez pas ce jeu d'armes.".to_string())?;
    if cible == "tout" {
        for (id, competence) in &def.competences {
            let niveau = niveau.clamp(NIVEAU_DEPART, competence.niveau_max());
            appris.competences.insert(id.clone(), niveau);
        }
        Ok(format!("{} : toutes les compétences au niveau {}.", def.nom, niveau.max(NIVEAU_DEPART)))
    } else {
        let competence = competence_du_jeu(jeu, cible)?;
        let niveau = niveau.clamp(NIVEAU_DEPART, competence.niveau_max());
        appris.competences.insert(cible.to_string(), niveau);
        Ok(format!("{} est au niveau {niveau}.", competence.nom))
    }
}

/// Met la compétence `id` du jeu d'armes `jeu` sur la touche `touche` (1 ou 2) ; si elle était
/// sur l'autre touche, les deux s'échangent.
pub fn placer_sur(acquis: &mut Acquis, jeu: &str, touche: usize, id: &str) -> Result<String, String> {
    if touche == 0 || touche > PLACES_ARME {
        return Err(format!("Touche 1 ou 2, pas {touche}."));
    }
    let competence = competence_du_jeu(jeu, id)?;
    let appris = acquis
        .jeux
        .get_mut(jeu)
        .ok_or_else(|| "Vous ne connaissez pas ce jeu d'armes.".to_string())?;
    appris.competences.entry(id.to_string()).or_insert(NIVEAU_DEPART);
    match appris.placees.iter().position(|placee| placee.as_deref() == Some(id)) {
        Some(autre) => appris.placees.swap(autre, touche - 1),
        None => appris.placees[touche - 1] = Some(id.to_string()),
    }
    Ok(format!("{} est sur la touche {touche}.", competence.nom))
}

/// Les ids d'une liste, pour dire ce qui est possible : « a, b, c ».
fn liste<'a>(ids: impl IntoIterator<Item = &'a str>) -> String {
    ids.into_iter().collect::<Vec<_>>().join(", ")
}

/// Pose `competences` sur le personnage : ses effets permanents et ses PV max suivent ; avec
/// `pv_pleins`, il retrouve tous ses PV.
fn ecrire(ecs: &World, client: Entity, competences: IcebreakerCompetences, pv_pleins: bool) {
    let passif = IcebreakerPassif::de(&competences);
    if let Some(mut pv) = ecs.write_storage::<common::comp::Health>().get_mut(client) {
        if let Some(maximum) = pv.needs_maximum_update(passif.pv_max()) {
            pv.update_internal_integer_maximum(maximum);
        }
        if pv_pleins && !pv.is_dead {
            pv.set_fraction(1.0);
        }
    }
    let _ = ecs.write_storage::<IcebreakerPassif>().insert(client, passif);
    let _ = ecs.write_storage::<IcebreakerCompetences>().insert(client, competences);
}

/// Change ce que le personnage a acquis, avec `regle(acquis, classe, jeu en mains)`.
fn modifier(
    ecs: &World,
    client: Entity,
    regle: impl FnOnce(&mut Acquis, &str, Option<&str>) -> Result<String, String>,
) -> Result<String, String> {
    let mut competences = ecs
        .read_storage::<IcebreakerCompetences>()
        .get(client)
        .cloned()
        .ok_or_else(|| "Il faut jouer un personnage.".to_string())?;
    let jeu = jeu_en_main(&competences.classe, ecs.read_storage::<Inventory>().get(client))
        .map(|(id, _)| id);
    let classe = competences.classe.clone();
    let message = regle(&mut competences.acquis, &classe, jeu)?;
    ecrire(ecs, client, competences, false);
    Ok(message)
}

const AIDE: &str = "/entrainement classe <id>, arme <jeu>, placer <1|2> [id], niveau <id|tout> \
                    <1-5>, ultime <id>, passif <id>, soin, entree";

/// `/entrainement …` : tout régler au lieu d'entraînement (les boutons de ses panneaux). Sans rôle
/// requis, refusée ailleurs.
pub(crate) fn handle_entrainement(
    server: &mut crate::Server,
    client: Entity,
    _target: Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), common::comp::Content> {
    use common::comp::{CharacterState, ChatType, Content, Energy, ForceUpdate, Health, Pos};
    use common_net::msg::ServerGeneral;
    if !world::icebreaker::is_entrainement() {
        return Err(Content::Plain(
            "/entrainement ne marche qu'au lieu d'entraînement (icebreaker.ps1 entrainement)."
                .to_string(),
        ));
    }
    let config = common::icebreaker_arene::config();
    let ecs = server.state.ecs();
    let en_mains = |jeu: Option<&str>| {
        jeu.map(str::to_string)
            .ok_or_else(|| "Prenez en mains une arme de votre classe (/entrainement arme <jeu>).".to_string())
    };
    let mots: Vec<&str> = args.iter().map(String::as_str).collect();
    let message: Result<String, String> = match mots.as_slice() {
        ["classe", id] => (|| {
            let classes = icebreaker_classes::classes();
            let class = classes.iter().find(|class| class.id == *id).ok_or_else(|| {
                format!("Classe inconnue : {id} ({}).", liste(classes.iter().map(|c| c.id.as_str())))
            })?;
            let (_, jeu) = registre()
                .jeu_par_defaut(&class.id)
                .ok_or_else(|| format!("La classe {id} n'a pas de jeu d'armes."))?;
            let (competences, equipement) = fiche(class, jeu, config.niveau)
                .ok_or_else(|| format!("La classe {id} n'a pas de compétences."))?;
            let _ = ecs.write_storage::<Inventory>().insert(client, equipement);
            ecrire(ecs, client, competences, true);
            Ok(format!(
                "Vous êtes {} : {} en mains, compétences au niveau {}.",
                class.nom, jeu.nom, config.niveau
            ))
        })(),
        ["arme", id] => (|| {
            let classe = ecs
                .read_storage::<IcebreakerCompetences>()
                .get(client)
                .map(|competences| competences.classe.clone())
                .ok_or_else(|| "Il faut jouer un personnage.".to_string())?;
            let jeu = registre().jeu(id).filter(|jeu| jeu.classe == classe).ok_or_else(|| {
                format!(
                    "Pas un jeu d'armes de votre classe : {id} ({}).",
                    liste(registre().jeux_de(&classe).map(|(id, _)| id))
                )
            })?;
            let mut inventaires = ecs.write_storage::<Inventory>();
            let mut inventaire =
                inventaires.get_mut(client).ok_or_else(|| "Pas de sac.".to_string())?;
            icebreaker_classes::prendre_jeu(&mut inventaire, jeu);
            Ok(format!("{} en mains.", jeu.nom))
        })(),
        ["placer", touche] => modifier(ecs, client, |acquis, _, jeu| {
            let touche = touche.parse::<usize>().map_err(|_| "Touche 1 ou 2.".to_string())?;
            crate::icebreaker_salle::placer(acquis, &en_mains(jeu)?, touche)
        }),
        ["placer", touche, id] => modifier(ecs, client, |acquis, _, jeu| {
            let touche = touche.parse::<usize>().map_err(|_| "Touche 1 ou 2.".to_string())?;
            placer_sur(acquis, &en_mains(jeu)?, touche, id)
        }),
        ["niveau", cible, niveau] => modifier(ecs, client, |acquis, _, jeu| {
            let niveau = niveau.parse::<u8>().map_err(|_| "Niveau de 1 à 5.".to_string())?;
            regler_niveau(acquis, &en_mains(jeu)?, cible, niveau)
        }),
        ["ultime", id] => modifier(ecs, client, |acquis, classe, _| activer_ultime(acquis, classe, id)),
        ["passif", id] => modifier(ecs, client, |acquis, classe, _| activer_passif(acquis, classe, id)),
        ["soin"] => (|| {
            let mut pvs = ecs.write_storage::<Health>();
            let mut pv = pvs.get_mut(client).ok_or_else(|| "Il faut jouer un personnage.".to_string())?;
            if pv.is_dead {
                return Err("Vous êtes mort : vous réapparaissez à l'entrée.".to_string());
            }
            pv.refresh_death_protection();
            pv.set_fraction(1.0);
            if let Some(mut etat) = ecs.write_storage::<CharacterState>().get_mut(client)
                && matches!(*etat, CharacterState::Crawl)
            {
                *etat = CharacterState::Idle(Default::default());
            }
            if let Some(mut energie) = ecs.write_storage::<Energy>().get_mut(client) {
                energie.refresh();
            }
            if let Some(mut competences) = ecs.write_storage::<IcebreakerCompetences>().get_mut(client) {
                competences.recharges.clear();
            }
            Ok("PV et énergie pleins, compétences prêtes.".to_string())
        })(),
        ["entree"] => (|| {
            let entree = ecs.read_resource::<crate::SpawnPoint>().0;
            let mut positions = ecs.write_storage::<Pos>();
            let pos = positions.get_mut(client).ok_or_else(|| "Il faut jouer un personnage.".to_string())?;
            pos.0 = entree;
            if let Some(force) = ecs.write_storage::<ForceUpdate>().get_mut(client) {
                force.update();
            }
            Ok("Retour à l'entrée.".to_string())
        })(),
        _ => Err(AIDE.to_string()),
    };
    let message = message.map_err(Content::Plain)?;
    server.notify_client(
        client,
        ServerGeneral::server_msg(ChatType::Meta, Content::Plain(message)),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AreneConfig { common::icebreaker_arene::config() }

    #[test]
    fn arrivee_sans_arme_classe_par_defaut() {
        let config = config();
        let classes = icebreaker_classes::classes();
        let (class, jeu) = classe_tenue(&Inventory::with_empty(), &classes, &config).unwrap();
        assert_eq!(class.id, config.classe_par_defaut);
        let (competences, inventaire) = fiche(&class, jeu, config.niveau).unwrap();
        // Il tient le jeu d'armes par défaut de sa classe, et ses trois touches sont garnies.
        let (id, _) = jeu_en_main(&class.id, Some(&inventaire)).unwrap();
        assert_eq!(Some(id), registre().jeu_par_defaut(&class.id).map(|(id, _)| id));
        for touche in 0..3 {
            assert!(competences.sur_touche(touche, Some(&inventaire)).is_some(), "{touche}");
        }
    }

    #[test]
    fn arrivee_garde_la_classe_de_l_arme_tenue() {
        let config = config();
        let classes = icebreaker_classes::classes();
        for class in &classes {
            for (id, jeu) in registre().jeux_de(&class.id) {
                let tenu = icebreaker_classes::inventaire_avec_jeu(class, jeu);
                let (trouvee, jeu) = classe_tenue(&tenu, &classes, &config).unwrap();
                assert_eq!(trouvee.id, class.id, "{id}");
                let (competences, inventaire) = fiche(&trouvee, jeu, NIVEAU_DEPART).unwrap();
                assert_eq!(jeu_en_main(&class.id, Some(&inventaire)).map(|(id, _)| id), Some(id));
                assert!(competences.sur_touche(0, Some(&inventaire)).is_some(), "{id}");
            }
        }
    }

    #[test]
    fn chaque_classe_a_sa_fiche() {
        // Toutes les classes de `classes.ron` sont décrites par nos données.
        for class in icebreaker_classes::classes() {
            let (_, jeu) = registre().jeu_par_defaut(&class.id).unwrap();
            assert!(fiche(&class, jeu, 5).is_some(), "{}", class.id);
        }
    }

    /// Un jeu d'armes du guerrier et ses compétences, pour les règles.
    fn jeu_d_essai() -> (&'static str, Vec<&'static str>) {
        let (id, jeu) = registre().jeu_par_defaut("guerrier").unwrap();
        (id, jeu.competences.iter().map(|(id, _)| id.as_str()).collect())
    }

    #[test]
    fn niveau_regle_et_borne() {
        let (jeu, ids) = jeu_d_essai();
        let mut acquis = Acquis::complet("guerrier", 1);
        regler_niveau(&mut acquis, jeu, ids[0], 3).unwrap();
        assert_eq!(acquis.jeux[jeu].competences[ids[0]], 3);
        assert_eq!(acquis.jeux[jeu].competences[ids[1]], 1);
        // Au-delà du plus haut niveau, ou sous le premier : borné.
        let max = registre().competence(ids[0]).unwrap().competence.niveau_max();
        regler_niveau(&mut acquis, jeu, ids[0], 99).unwrap();
        assert_eq!(acquis.jeux[jeu].competences[ids[0]], max);
        regler_niveau(&mut acquis, jeu, "tout", 0).unwrap();
        assert!(acquis.jeux[jeu].competences.values().all(|n| *n == NIVEAU_DEPART));
        regler_niveau(&mut acquis, jeu, "tout", 5).unwrap();
        assert!(acquis.jeux[jeu].competences.values().all(|n| *n > NIVEAU_DEPART));
    }

    #[test]
    fn niveau_refuse_la_competence_d_un_autre_jeu() {
        let (jeu, _) = jeu_d_essai();
        let mut acquis = Acquis::complet("guerrier", 1);
        // Une compétence du mage, un ultime, un id inconnu.
        let (_, baton) = registre().jeu_par_defaut("mage").unwrap();
        assert!(regler_niveau(&mut acquis, jeu, &baton.competences[0].0, 2).is_err());
        let ultime = &registre().ultimes_de("guerrier")[0].0;
        assert!(regler_niveau(&mut acquis, jeu, ultime, 2).is_err());
        assert!(regler_niveau(&mut acquis, jeu, "inconnue", 2).is_err());
        assert_eq!(acquis, Acquis::complet("guerrier", 1));
    }

    #[test]
    fn placer_directement_et_echanger() {
        let (jeu, ids) = jeu_d_essai();
        let mut acquis = Acquis::complet("guerrier", 1);
        assert_eq!(acquis.jeux[jeu].placees, [Some(ids[0].to_string()), Some(ids[1].to_string())]);
        // Une autre compétence sur la touche 2.
        placer_sur(&mut acquis, jeu, 2, ids[3]).unwrap();
        assert_eq!(acquis.jeux[jeu].placees, [Some(ids[0].to_string()), Some(ids[3].to_string())]);
        // Celle de la touche 1 sur la touche 2 : elles s'échangent.
        placer_sur(&mut acquis, jeu, 2, ids[0]).unwrap();
        assert_eq!(acquis.jeux[jeu].placees, [Some(ids[3].to_string()), Some(ids[0].to_string())]);
        assert!(placer_sur(&mut acquis, jeu, 3, ids[1]).is_err());
        assert!(placer_sur(&mut acquis, jeu, 0, ids[1]).is_err());
        assert!(placer_sur(&mut acquis, jeu, 1, "inconnue").is_err());
    }
}
