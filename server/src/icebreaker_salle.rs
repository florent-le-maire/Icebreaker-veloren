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
        Acquis, AcquisJeu, Competence, IcebreakerCompetences, IcebreakerPassif, JeuArmes,
        NIVEAU_DEPART, PLACES_ARME, Registre, SalleArmes, Sorte as SorteCompetence, jeu_en_main,
        registre,
    },
    icebreaker_panneau::{Action, Chiffre, Jauge, Ligne, Onglet, Panneau, Sorte, Ton, texte},
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

// ---------------------------------------------------------------------------------------------
// A.12 : les règles, pures, sur `Acquis`. Chacune rend le message de réussite ou le refus.
// ---------------------------------------------------------------------------------------------

/// Ce qu'un achat coûte en or et remet en mains : à prendre dans la bourse et le sac par l'appelant,
/// une fois la règle appliquée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Achat {
    pub message: String,
    pub prix: u32,
    /// Ids d'objets à remettre dans le sac (les armes d'un jeu d'armes appris).
    pub armes: Vec<String>,
}

fn jeu_appris<'a>(acquis: &'a mut Acquis, jeu: &str) -> Result<&'a mut AcquisJeu, String> {
    acquis
        .jeux
        .get_mut(jeu)
        .ok_or_else(|| "Vous ne connaissez pas ce jeu d'armes.".to_string())
}

/// La compétence `id`, qui doit être une compétence d'arme du jeu d'armes `jeu`.
fn competence_du_jeu(jeu: &str, id: &str) -> Result<&'static Competence, String> {
    let entree = registre()
        .competence(id)
        .ok_or_else(|| format!("Compétence inconnue : {id}."))?;
    match &entree.sorte {
        SorteCompetence::Arme { jeu: du_jeu } if du_jeu == jeu => Ok(&entree.competence),
        SorteCompetence::Arme { .. } => Err(format!(
            "{} est une compétence d'un autre jeu d'armes.",
            entree.competence.nom
        )),
        SorteCompetence::Ultime { .. } => Err(format!(
            "{} est un ultime, pas une compétence d'arme.",
            entree.competence.nom
        )),
    }
}

/// Il manque des points d'arme ?
fn assez_de_points(appris: &AcquisJeu, cout: u32, salle: &SalleArmes, but: &str) -> Result<(), String> {
    let disponibles = appris.points_disponibles(salle);
    if disponibles < cout {
        Err(format!(
            "Il faut {} d'arme pour {but} (vous en avez {disponibles}).",
            points(cout)
        ))
    } else {
        Ok(())
    }
}

/// Débloque la compétence `id` du jeu d'armes en mains `jeu` : niveau 1, placée sur une touche
/// libre s'il y en a une.
pub fn debloquer(acquis: &mut Acquis, jeu: &str, id: &str, salle: &SalleArmes) -> Result<String, String> {
    let competence = competence_du_jeu(jeu, id)?;
    let appris = jeu_appris(acquis, jeu)?;
    if appris.competences.contains_key(id) {
        return Err(format!("{} est déjà débloquée.", competence.nom));
    }
    assez_de_points(appris, salle.cout_debloquer, salle, &format!("débloquer {}", competence.nom))?;
    appris.points_depenses += salle.cout_debloquer;
    appris.competences.insert(id.to_string(), NIVEAU_DEPART);
    Ok(match appris.placees.iter().position(Option::is_none) {
        Some(touche) => {
            appris.placees[touche] = Some(id.to_string());
            format!("{} débloquée, sur la touche {}.", competence.nom, touche + 1)
        },
        None => format!("{} débloquée.", competence.nom),
    })
}

/// Les points d'arme que coûte le prochain niveau de `competence` pour `appris` ; `None` : au
/// niveau maximum, inconnue, ou les réglages ne prévoient pas ce niveau.
pub fn cout_monter(appris: &AcquisJeu, id: &str, competence: &Competence, salle: &SalleArmes) -> Option<u32> {
    let niveau = *appris.competences.get(id)?;
    (niveau < competence.niveau_max())
        .then(|| salle.cout_niveau(niveau + 1))
        .flatten()
}

/// Monte la compétence connue `id` d'un niveau ; le message cite ce que le niveau apporte.
pub fn monter(acquis: &mut Acquis, jeu: &str, id: &str, salle: &SalleArmes) -> Result<String, String> {
    let competence = competence_du_jeu(jeu, id)?;
    let appris = jeu_appris(acquis, jeu)?;
    let Some(niveau) = appris.competences.get(id).copied() else {
        return Err(format!("{} n'est pas encore débloquée.", competence.nom));
    };
    if niveau >= competence.niveau_max() {
        return Err(format!("{} est déjà au niveau maximum.", competence.nom));
    }
    let vise = niveau + 1;
    let cout = salle
        .cout_niveau(vise)
        .ok_or_else(|| format!("Les réglages ne prévoient pas de niveau {vise}."))?;
    assez_de_points(appris, cout, salle, &format!("monter {} au niveau {vise}", competence.nom))?;
    appris.points_depenses += cout;
    appris.competences.insert(id.to_string(), vise);
    let texte = &competence.niveaux[usize::from(vise - NIVEAU_DEPART) - 1].texte;
    Ok(if texte.is_empty() {
        format!("{} niveau {vise}.", competence.nom)
    } else {
        format!("{} niveau {vise} : {texte}", competence.nom)
    })
}

/// La compétence connue que la touche `touche` (0 ou 1) recevrait au prochain appui : la suivante
/// du jeu d'armes, dans son ordre, en sautant celle de l'autre touche et en revenant à la première
/// après la dernière. `None` : il n'y en a pas d'autre.
pub fn suivante<'a>(def: &'a JeuArmes, appris: &AcquisJeu, touche: usize) -> Option<&'a str> {
    let actuelle = appris.placees.get(touche)?.as_deref();
    let connues: Vec<&str> = def
        .competences
        .iter()
        .map(|(id, _)| id.as_str())
        .filter(|id| appris.competences.contains_key(*id))
        .filter(|id| {
            !appris
                .placees
                .iter()
                .enumerate()
                .any(|(autre, placee)| autre != touche && placee.as_deref() == Some(*id))
        })
        .collect();
    let debut = actuelle
        .and_then(|actuelle| connues.iter().position(|id| *id == actuelle))
        .map_or(0, |position| position + 1);
    let prochaine = *connues.get(debut % connues.len().max(1))?;
    (Some(prochaine) != actuelle).then_some(prochaine)
}

/// Met sur la touche `touche` (1 ou 2) la compétence connue suivante du jeu d'armes en mains.
pub fn placer(acquis: &mut Acquis, jeu: &str, touche: usize) -> Result<String, String> {
    let def = registre()
        .jeu(jeu)
        .ok_or_else(|| "Jeu d'armes inconnu.".to_string())?;
    if touche == 0 || touche > PLACES_ARME {
        return Err(format!("Touche 1 ou 2, pas {touche}."));
    }
    let appris = jeu_appris(acquis, jeu)?;
    let id = suivante(def, appris, touche - 1)
        .ok_or_else(|| format!("Pas d'autre compétence connue à mettre sur la touche {touche}."))?
        .to_string();
    let nom = registre().competence(&id).map_or(id.clone(), |e| e.competence.nom.clone());
    appris.placees[touche - 1] = Some(id);
    Ok(format!("{nom} est sur la touche {touche}."))
}

/// L'or qui manque, ou rien.
fn assez_d_or(bourse: u64, prix: u32, but: &str) -> Result<(), String> {
    if bourse < u64::from(prix) {
        Err(format!(
            "Il vous manque {} or pour {but} ({prix} or).",
            u64::from(prix) - bourse
        ))
    } else {
        Ok(())
    }
}

/// Apprend le jeu d'armes `id` de `classe` : ses deux premières compétences au niveau 1, placées ;
/// ses armes sont à remettre.
pub fn apprendre_jeu(
    acquis: &mut Acquis,
    classe: &str,
    id: &str,
    salle: &SalleArmes,
    bourse: u64,
) -> Result<Achat, String> {
    let def = registre()
        .jeu(id)
        .filter(|def| def.classe == classe)
        .ok_or_else(|| format!("Jeu d'armes inconnu : {id}."))?;
    if acquis.jeux.contains_key(id) {
        return Err(format!("Vous connaissez déjà {}.", def.nom));
    }
    assez_d_or(bourse, salle.prix_jeu_armes, &format!("apprendre {}", def.nom))?;
    let mut appris = AcquisJeu::default();
    for (place, (competence, _)) in appris.placees.iter_mut().zip(&def.competences) {
        *place = Some(competence.clone());
        appris.competences.insert(competence.clone(), NIVEAU_DEPART);
    }
    acquis.jeux.insert(id.to_string(), appris);
    let mut armes = vec![def.armes.0.clone()];
    armes.extend(def.armes.1.clone());
    Ok(Achat {
        message: format!("{} appris : vos armes sont dans le sac.", def.nom),
        prix: salle.prix_jeu_armes,
        armes,
    })
}

/// Apprend un choix (ultime ou passif) : il devient le choix actif.
fn apprendre_choix(
    connus: &mut Vec<String>,
    actif: &mut Option<String>,
    existe: Option<&str>,
    id: &str,
    (prix, bourse): (u32, u64),
    genre: &str,
) -> Result<Achat, String> {
    let nom = existe.ok_or_else(|| format!("{genre} inconnu : {id}."))?;
    if connus.iter().any(|connu| connu == id) {
        return Err(format!("Vous connaissez déjà {nom}."));
    }
    assez_d_or(bourse, prix, &format!("apprendre {nom}"))?;
    connus.push(id.to_string());
    *actif = Some(id.to_string());
    Ok(Achat {
        message: format!("{nom} appris : c'est votre {} actif.", genre.to_lowercase()),
        prix,
        armes: Vec::new(),
    })
}

/// Active un choix connu (ultime ou passif).
fn activer_choix(
    connus: &[String],
    actif: &mut Option<String>,
    existe: Option<&str>,
    id: &str,
    genre: &str,
) -> Result<String, String> {
    let nom = existe.ok_or_else(|| format!("{genre} inconnu : {id}."))?;
    if !connus.iter().any(|connu| connu == id) {
        return Err(format!("Vous ne connaissez pas encore {nom}."));
    }
    if actif.as_deref() == Some(id) {
        return Err(format!("{nom} est déjà actif."));
    }
    *actif = Some(id.to_string());
    Ok(format!("{nom} est votre {} actif.", genre.to_lowercase()))
}

fn nom_ultime(classe: &str, id: &str) -> Option<&'static str> {
    registre()
        .ultimes_de(classe)
        .iter()
        .find(|(ultime, _)| ultime == id)
        .map(|(_, c)| c.nom.as_str())
}

fn nom_passif(classe: &str, id: &str) -> Option<&'static str> {
    registre()
        .passifs_de(classe)?
        .passifs
        .iter()
        .find(|(passif, _)| passif == id)
        .map(|(_, p)| p.nom.as_str())
}

pub fn apprendre_ultime(
    acquis: &mut Acquis,
    classe: &str,
    id: &str,
    salle: &SalleArmes,
    bourse: u64,
) -> Result<Achat, String> {
    apprendre_choix(
        &mut acquis.ultimes,
        &mut acquis.ultime,
        nom_ultime(classe, id),
        id,
        (salle.prix_ultime, bourse),
        "Ultime",
    )
}

pub fn apprendre_passif(
    acquis: &mut Acquis,
    classe: &str,
    id: &str,
    salle: &SalleArmes,
    bourse: u64,
) -> Result<Achat, String> {
    apprendre_choix(
        &mut acquis.passifs,
        &mut acquis.passif,
        nom_passif(classe, id),
        id,
        (salle.prix_passif, bourse),
        "Passif",
    )
}

pub fn activer_ultime(acquis: &mut Acquis, classe: &str, id: &str) -> Result<String, String> {
    activer_choix(&acquis.ultimes, &mut acquis.ultime, nom_ultime(classe, id), id, "Ultime")
}

pub fn activer_passif(acquis: &mut Acquis, classe: &str, id: &str) -> Result<String, String> {
    activer_choix(&acquis.passifs, &mut acquis.passif, nom_passif(classe, id), id, "Passif")
}

/// Le jeu d'armes en mains, s'il est appris : (id, définition, ce qui est acquis).
type EnMains<'a> = (&'a str, &'a JeuArmes, &'a AcquisJeu);

/// Les compétences du jeu d'armes en mains : une ligne par compétence du jeu.
fn onglet_competences(en_mains: Option<EnMains<'_>>, salle: &SalleArmes) -> Vec<Ligne> {
    let Some((_, def, acquis)) = en_mains else {
        return vec![Ligne::nouvelle("Prenez en mains une arme que vous savez manier.").estompee()];
    };
    let disponibles = acquis.points_disponibles(salle);
    def.competences
        .iter()
        .map(|(id, competence)| {
            let ligne = Ligne::nouvelle(competence.nom.clone()).icone(def.icone.clone());
            let Some(niveau) = acquis.competences.get(id).copied() else {
                return ligne.sous_titre("à débloquer").estompee().action(
                    Action::nouvelle(
                        format!("Débloquer ({} pt)", salle.cout_debloquer),
                        &["salle", "debloquer", id],
                    )
                    .principale()
                    .permise(disponibles >= salle.cout_debloquer),
                );
            };
            let monter = cout_monter(acquis, id, competence, salle).map(|cout| {
                Action::nouvelle(format!("Monter ({cout} pt)"), &["salle", "monter", id])
                    .principale()
                    .permise(disponibles >= cout)
            });
            let touche = acquis
                .placees
                .iter()
                .position(|placee| placee.as_deref() == Some(id.as_str()));
            let ligne = ligne.sous_titre(format!(
                "niveau {niveau} · recharge {}",
                secondes(competence.effectif(niveau).recharge)
            ));
            let ligne = match touche {
                Some(touche) => ligne.pastille(format!("touche {}", touche + 1), Ton::Info),
                None if niveau >= competence.niveau_max() => ligne.pastille("niveau max", Ton::Bon),
                None => ligne,
            };
            match monter {
                Some(action) => ligne.action(action),
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
    bourse: u64,
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
                None => ligne.sous_titre("à apprendre").action(
                    Action::nouvelle(
                        format!("Apprendre ({} or)", salle.prix_jeu_armes),
                        &["salle", "apprendre", "jeu", id],
                    )
                    .principale()
                    .permise(bourse >= u64::from(salle.prix_jeu_armes)),
                ),
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
fn onglet_ultimes(registre: &Registre, classe: &str, acquis: &Acquis, salle: &SalleArmes, bourse: u64) -> Vec<Ligne> {
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
            } else if acquis.ultimes.contains(id) {
                ligne.action(Action::nouvelle("Activer", &["salle", "activer", "ultime", id]))
            } else {
                ligne.action(
                    Action::nouvelle(
                        format!("Apprendre ({} or)", salle.prix_ultime),
                        &["salle", "apprendre", "ultime", id],
                    )
                    .principale()
                    .permise(bourse >= u64::from(salle.prix_ultime)),
                )
            }
        })
        .collect()
}

/// Les passifs de la classe.
fn onglet_passifs(registre: &Registre, classe: &str, acquis: &Acquis, salle: &SalleArmes, bourse: u64) -> Vec<Ligne> {
    registre
        .passifs_de(classe)
        .map(|passifs| passifs.passifs.as_slice())
        .unwrap_or_default()
        .iter()
        .map(|(id, passif)| {
            let ligne = Ligne::nouvelle(passif.nom.clone()).sous_titre(passif.description.clone());
            if acquis.passif.as_deref() == Some(id.as_str()) {
                ligne.pastille("actif", Ton::Bon)
            } else if acquis.passifs.contains(id) {
                ligne.action(Action::nouvelle("Activer", &["salle", "activer", "passif", id]))
            } else {
                ligne.action(
                    Action::nouvelle(
                        format!("Apprendre ({} or)", salle.prix_passif),
                        &["salle", "apprendre", "passif", id],
                    )
                    .principale()
                    .permise(bourse >= u64::from(salle.prix_passif)),
                )
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
            lignes: onglet_competences(en_mains, salle),
        },
        Onglet {
            libelle: texte("Armes"),
            lignes: onglet_armes(registre, classe, acquis, tenu.map(|(id, _)| id), salle, bourse),
        },
        Onglet {
            libelle: texte("Ultimes"),
            lignes: onglet_ultimes(registre, classe, acquis, salle, bourse),
        },
        Onglet {
            libelle: texte("Passifs"),
            lignes: onglet_passifs(registre, classe, acquis, salle, bourse),
        },
    ];
    // Les deux touches d'arme : chaque appui y met la compétence connue suivante.
    if let Some((_, def, appris)) = en_mains {
        panneau.colonnes = 2;
        panneau.actions = (0..PLACES_ARME)
            .map(|touche| {
                let nom = appris.placees[touche]
                    .as_deref()
                    .and_then(|id| registre.competence(id))
                    .map_or("vide", |entree| entree.competence.nom.as_str());
                Action::nouvelle(
                    format!("Touche {} : {nom}", touche + 1),
                    &["salle", "placer", &(touche + 1).to_string()],
                )
                .permise(suivante(def, appris, touche).is_some())
            })
            .collect();
    }
    let prix_mini = [salle.prix_jeu_armes, salle.prix_ultime, salle.prix_passif]
        .into_iter()
        .min()
        .unwrap_or(0);
    let manque = match en_mains {
        Some((_, _, appris)) if appris.points_disponibles(salle) == 0 => Some(
            "Aucun point d'arme à dépenser avec ce jeu : la maîtrise se gagne en expédition."
                .to_string(),
        ),
        _ if bourse < u64::from(prix_mini) => Some(format!(
            "Il vous manque de l'or : un jeu d'armes coûte {} or, un ultime {}, un passif {}.",
            salle.prix_jeu_armes, salle.prix_ultime, salle.prix_passif
        )),
        _ => None,
    };
    match (message, manque) {
        (Some((texte, sorte)), _) => panneau.message(texte, sorte),
        (None, Some(texte)) => panneau.message(texte, Sorte::Info),
        (None, None) => panneau,
    }
}

/// `/salle debloquer <id> | monter <id> | placer <1|2> | apprendre <jeu|ultime|passif> <id> |
/// activer <ultime|passif> <id>` (les boutons du panneau de la salle d'armes). Au village seulement,
/// pour un personnage qui a nos compétences.
pub(crate) fn handle_salle(
    server: &mut crate::Server,
    client: specs::Entity,
    _target: specs::Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), common::comp::Content> {
    use common::comp::{ChatType, Content, Item, Pos};
    use common_net::msg::ServerGeneral;
    use specs::WorldExt;
    let plain = |texte: &str| Content::Plain(texte.to_string());
    let ecs = server.state.ecs();
    let au_village = world::icebreaker::is_village()
        && ecs
            .read_storage::<Pos>()
            .get(client)
            .is_some_and(|pos| {
                ecs.read_resource::<crate::icebreaker_buildings::IcebreakerBuildings>()
                    .in_village(pos.0.xy())
            });
    if !au_village {
        return Err(plain("La salle d'armes est au village."));
    }
    let Some(avant) = ecs.read_storage::<IcebreakerCompetences>().get(client).cloned() else {
        return Err(plain(
            "Il faut jouer un personnage qui a des compétences à régler (la charrette).",
        ));
    };
    let salle = SalleArmes::actuel();
    let classe = avant.classe.clone();
    let (jeu, bourse, libres) = {
        let inventaires = ecs.read_storage::<common::comp::Inventory>();
        let inventaire = inventaires.get(client);
        (
            jeu_en_main(&classe, inventaire).map(|(id, _)| id),
            inventaire.map_or(0, |i| i.item_count(&crate::icebreaker_services::coins())),
            inventaire.map_or(0, |i| i.free_slots()),
        )
    };
    let en_mains = || jeu.ok_or_else(|| plain("Prenez en mains une arme que vous savez manier."));
    let mut acquis = avant.acquis.clone();
    let mots: Vec<&str> = args.iter().map(String::as_str).collect();
    let (message, achat) = match mots.as_slice() {
        ["debloquer", id] => (debloquer(&mut acquis, en_mains()?, id, &salle), None),
        ["monter", id] => (monter(&mut acquis, en_mains()?, id, &salle), None),
        ["placer", touche] => {
            let touche = touche.parse::<usize>().map_err(|_| plain("Touche 1 ou 2."))?;
            (placer(&mut acquis, en_mains()?, touche), None)
        },
        ["apprendre", sorte, id] => {
            let achat = match *sorte {
                "jeu" => apprendre_jeu(&mut acquis, &classe, id, &salle, bourse),
                "ultime" => apprendre_ultime(&mut acquis, &classe, id, &salle, bourse),
                "passif" => apprendre_passif(&mut acquis, &classe, id, &salle, bourse),
                _ => Err("Apprendre un jeu, un ultime ou un passif.".to_string()),
            };
            match achat {
                Ok(achat) => (Ok(achat.message.clone()), Some(achat)),
                Err(refus) => (Err(refus), None),
            }
        },
        ["activer", "ultime", id] => (activer_ultime(&mut acquis, &classe, id), None),
        ["activer", "passif", id] => (activer_passif(&mut acquis, &classe, id), None),
        _ => {
            return Err(plain(
                "/salle debloquer <id>, monter <id>, placer <1|2>, apprendre <jeu|ultime|passif> \
                 <id> ou activer <ultime|passif> <id>",
            ));
        },
    };
    let message = message.map_err(Content::Plain)?;
    if let Some(achat) = achat {
        // Les armes d'abord, pour ne rien prendre si le sac est plein.
        let objets = achat
            .armes
            .iter()
            .map(|id| Item::new_from_asset(id).map_err(|_| plain("Arme introuvable dans les données.")))
            .collect::<Result<Vec<_>, _>>()?;
        if libres < objets.len() {
            return Err(Content::Plain(format!(
                "Votre sac est plein : libérez {} place{} pour les armes.",
                objets.len(),
                if objets.len() > 1 { "s" } else { "" }
            )));
        }
        if !objets.is_empty() {
            let mut inventaires = ecs.write_storage::<common::comp::Inventory>();
            let Some(mut inventaire) = inventaires.get_mut(client) else {
                return Err(plain("Pas de sac."));
            };
            if inventaire.push_all(objets.into_iter()).is_err() {
                return Err(plain("Votre sac est plein."));
            }
        }
        let pris = crate::icebreaker_treasury::take_coins(ecs, client, Some(u64::from(achat.prix)));
        if pris < u64::from(achat.prix) {
            return Err(plain("L'or n'a pas pu être pris."));
        }
    }
    // Ce que le personnage a acquis change, puis sa fiche, puis ses effets permanents.
    let nouveau = {
        let mut stockage = ecs.write_storage::<IcebreakerCompetences>();
        let Some(mut competences) = stockage.get_mut(client) else {
            return Err(plain("Il faut jouer un personnage."));
        };
        competences.acquis = acquis;
        competences.clone()
    };
    if nouveau.acquis.passif != avant.acquis.passif || nouveau.acquis.passifs != avant.acquis.passifs {
        let passif = IcebreakerPassif::de(&nouveau);
        let _ = ecs.write_storage::<IcebreakerPassif>().insert(client, passif);
    }
    crate::icebreaker_registre::sauver_acquis(ecs, client);
    if let Some(ouvert) = ecs
        .write_resource::<crate::icebreaker_panneaux::IcebreakerPanneaux>()
        .ouverts
        .get_mut(&client)
    {
        ouvert.salle_message = Some((message.clone(), Sorte::Reussite));
    }
    server.notify_client(
        client,
        ServerGeneral::server_msg(ChatType::Meta, Content::Plain(message)),
    );
    Ok(())
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
        // Les deux touches d'arme, en bas, communes aux onglets.
        assert_eq!(p.actions.len(), 2);
        assert_eq!(p.actions[0].libelle, texte("Touche 1 : Balayage lourd"));
        assert_eq!(p.actions[1].libelle, texte("Touche 2 : Tourbillon"));
        assert_eq!(p.actions[1].commande, vec!["salle".to_string(), "placer".into(), "2".into()]);
        assert!(
            p.actions.iter().all(|a| !a.permise),
            "deux compétences connues, deux touches : rien d'autre à placer"
        );
    }

    fn commande(l: &Ligne) -> (String, bool) {
        let a = l.action.as_ref().unwrap_or_else(|| panic!("pas d'action : {l:?}"));
        (a.commande.join(" "), a.permise)
    }

    #[test]
    fn les_lignes_portent_leur_bouton_grise_quand_c_est_impossible() {
        let salle = SalleArmes::actuel();
        let inventaire = inventaire(Some("grande_epee"));
        // 1 point, 125 or : on peut débloquer et monter, mais pas apprendre un jeu ni un ultime.
        let competences = guerrier(100);
        let p = panneau(Some(&competences), Some(&inventaire), 125, false, &salle, None);
        let c = lignes(&p, 0);
        assert_eq!(ligne(c, "Fendeur de terre").action.as_ref().unwrap().libelle, texte("Débloquer (1 pt)"));
        assert_eq!(commande(ligne(c, "Fendeur de terre")), ("salle debloquer fendeur_de_terre".into(), true));
        let tourbillon = ligne(c, "Tourbillon");
        assert_eq!(tourbillon.action.as_ref().unwrap().libelle, texte("Monter (1 pt)"));
        assert_eq!(commande(tourbillon), ("salle monter tourbillon".into(), true));
        let armes = lignes(&p, 1);
        let hache = ligne(armes, "Hache à deux mains");
        assert_eq!(hache.action.as_ref().unwrap().libelle, texte("Apprendre (150 or)"));
        assert_eq!(commande(hache), ("salle apprendre jeu hache".into(), false));
        assert!(ligne(armes, "Grande épée").action.is_none(), "déjà appris");
        let ultimes = lignes(&p, 2);
        assert_eq!(commande(ligne(ultimes, "Séisme")), ("salle apprendre ultime seisme".into(), false));
        assert!(ligne(ultimes, "Fureur").action.is_none(), "actif");
        let passifs = lignes(&p, 3);
        assert_eq!(commande(ligne(passifs, "Peau dure")), ("salle apprendre passif peau_dure".into(), true));

        // Plus de points : les boutons de points sont grisés, et le panneau le dit.
        let sans = guerrier(0);
        let p = panneau(Some(&sans), Some(&inventaire), 500, false, &salle, None);
        assert!(!commande(ligne(lignes(&p, 0), "Fendeur de terre")).1);
        assert!(!commande(ligne(lignes(&p, 0), "Tourbillon")).1);
        assert_eq!(message(&p).1, Sorte::Info);

        // Un ultime connu et inactif : « Activer », jamais grisé.
        let mut deux = guerrier(0);
        deux.acquis.ultimes.push("seisme".into());
        let p = panneau(Some(&deux), Some(&inventaire), 0, false, &salle, None);
        let seisme = ligne(lignes(&p, 2), "Séisme");
        assert_eq!(seisme.action.as_ref().unwrap().libelle, texte("Activer"));
        assert_eq!(commande(seisme), ("salle activer ultime seisme".into(), true));

        // Le message de la dernière commande passe devant.
        let p = panneau(Some(&sans), Some(&inventaire), 500, false, &salle, Some(("Fait.".into(), Sorte::Reussite)));
        assert_eq!(message(&p), ("Fait.".into(), Sorte::Reussite));
    }

    fn acquis_grande_epee(maitrise: u32) -> Acquis {
        guerrier(maitrise).acquis
    }

    #[test]
    fn debloquer_coute_des_points_et_prend_une_touche_libre() {
        let salle = SalleArmes::actuel();
        let mut acquis = acquis_grande_epee(100);
        // Un point : fendeur de terre, mais les deux touches sont prises.
        let message = debloquer(&mut acquis, "grande_epee", "fendeur_de_terre", &salle).unwrap();
        assert_eq!(message, "Fendeur de terre débloquée.");
        let jeu = &acquis.jeux["grande_epee"];
        assert_eq!(jeu.competences["fendeur_de_terre"], 1);
        assert_eq!(jeu.points_depenses, salle.cout_debloquer);
        assert_eq!(jeu.points_disponibles(&salle), 0);
        assert_eq!(jeu.placees, [Some("balayage_lourd".into()), Some("tourbillon".into())]);

        // Refus : plus de points, déjà connue, id inconnu, ultime, autre jeu, jeu non appris.
        let (mut autre, def) = (acquis.clone(), registre().jeu("grande_epee").unwrap());
        let inconnue = def.competences.iter().map(|(id, _)| id).find(|id| !autre.jeux["grande_epee"].competences.contains_key(*id)).unwrap().clone();
        assert!(debloquer(&mut autre, "grande_epee", &inconnue, &salle).unwrap_err().contains("Il faut 1 point"));
        assert!(debloquer(&mut autre, "grande_epee", "tourbillon", &salle).unwrap_err().contains("déjà débloquée"));
        assert!(debloquer(&mut autre, "grande_epee", "nimporte", &salle).unwrap_err().contains("inconnue"));
        assert!(debloquer(&mut autre, "grande_epee", "fureur", &salle).unwrap_err().contains("ultime"));
        let d_ailleurs = registre().jeu("hache").unwrap().competences[0].0.clone();
        assert!(debloquer(&mut autre, "grande_epee", &d_ailleurs, &salle).unwrap_err().contains("autre jeu"));
        assert!(debloquer(&mut autre, "hache", &d_ailleurs, &salle).unwrap_err().contains("ne connaissez pas"));
        assert_eq!(autre, acquis, "un refus ne change rien");

        // Une touche libre reçoit la compétence.
        let mut libre = acquis_grande_epee(450);
        libre.jeux.get_mut("grande_epee").unwrap().placees = [Some("tourbillon".into()), None];
        let message = debloquer(&mut libre, "grande_epee", "fendeur_de_terre", &salle).unwrap();
        assert_eq!(message, "Fendeur de terre débloquée, sur la touche 2.");
        assert_eq!(libre.jeux["grande_epee"].placees[1].as_deref(), Some("fendeur_de_terre"));
    }

    #[test]
    fn monter_coute_le_cout_du_niveau_vise_et_cite_son_texte() {
        let salle = SalleArmes::actuel();
        // 450 : 100 + 150 + 200 = 3 points.
        let mut acquis = acquis_grande_epee(450);
        let message = monter(&mut acquis, "grande_epee", "tourbillon", &salle).unwrap();
        assert_eq!(message, "Tourbillon niveau 2 : 10 % de dégâts en plus.");
        let message = monter(&mut acquis, "grande_epee", "tourbillon", &salle).unwrap();
        assert_eq!(message, "Tourbillon niveau 3 : Un troisième tour d'épée, et la zone s'élargit.");
        let jeu = &acquis.jeux["grande_epee"];
        assert_eq!(jeu.competences["tourbillon"], 3);
        assert_eq!(
            jeu.points_depenses,
            salle.cout_niveau(2).unwrap() + salle.cout_niveau(3).unwrap()
        );

        // Plus assez de points pour le niveau 4.
        let avant = acquis.clone();
        let refus = monter(&mut acquis, "grande_epee", "tourbillon", &salle).unwrap_err();
        assert!(refus.contains("niveau 4") && refus.contains("Il faut"), "{refus}");
        assert_eq!(acquis, avant);

        // Au niveau maximum, inconnue, d'un autre jeu.
        let max = registre().competence("tourbillon").unwrap().competence.niveau_max();
        acquis.jeux.get_mut("grande_epee").unwrap().competences.insert("tourbillon".into(), max);
        assert!(monter(&mut acquis, "grande_epee", "tourbillon", &salle).unwrap_err().contains("maximum"));
        assert!(monter(&mut acquis, "grande_epee", "fendeur_de_terre", &salle).unwrap_err().contains("pas encore"));
        assert!(monter(&mut acquis, "grande_epee", "zzz", &salle).unwrap_err().contains("inconnue"));
        let d_ailleurs = registre().jeu("hache").unwrap().competences[0].0.clone();
        assert!(monter(&mut acquis, "grande_epee", &d_ailleurs, &salle).unwrap_err().contains("autre jeu"));
    }

    #[test]
    fn placer_met_la_competence_connue_suivante_en_sautant_l_autre_touche() {
        let salle = SalleArmes::actuel();
        let mut acquis = acquis_grande_epee(450);
        // Seulement deux compétences connues, sur les deux touches : rien à placer.
        assert!(placer(&mut acquis, "grande_epee", 1).unwrap_err().contains("Pas d'autre"));
        assert!(placer(&mut acquis, "grande_epee", 3).unwrap_err().contains("Touche 1 ou 2"));

        debloquer(&mut acquis, "grande_epee", "fendeur_de_terre", &salle).unwrap();
        // Ordre du jeu : balayage_lourd, tourbillon, fendeur_de_terre, … Touche 1 (balayage, l'autre
        // touche a le tourbillon) : fendeur, puis on revient au balayage.
        assert_eq!(placer(&mut acquis, "grande_epee", 1).unwrap(), "Fendeur de terre est sur la touche 1.");
        assert_eq!(acquis.jeux["grande_epee"].placees[0].as_deref(), Some("fendeur_de_terre"));
        assert_eq!(placer(&mut acquis, "grande_epee", 1).unwrap(), "Balayage lourd est sur la touche 1.");
        // Touche 2 (tourbillon ; l'autre a le balayage) : fendeur, puis tourbillon.
        placer(&mut acquis, "grande_epee", 2).unwrap();
        assert_eq!(acquis.jeux["grande_epee"].placees[1].as_deref(), Some("fendeur_de_terre"));
        placer(&mut acquis, "grande_epee", 2).unwrap();
        assert_eq!(acquis.jeux["grande_epee"].placees[1].as_deref(), Some("tourbillon"));
        assert!(placer(&mut acquis, "hache", 1).unwrap_err().contains("ne connaissez pas"));
    }

    #[test]
    fn apprendre_un_jeu_d_armes_coute_de_l_or_et_remet_ses_armes() {
        let salle = SalleArmes::actuel();
        let mut acquis = acquis_grande_epee(0);
        let avant = acquis.clone();
        let refus = apprendre_jeu(&mut acquis, "guerrier", "hache", &salle, 149).unwrap_err();
        assert!(refus.contains("Il vous manque 1 or"), "{refus}");
        assert!(apprendre_jeu(&mut acquis, "guerrier", "grande_epee", &salle, 999).unwrap_err().contains("déjà"));
        assert!(apprendre_jeu(&mut acquis, "guerrier", "inconnu", &salle, 999).is_err());
        assert!(apprendre_jeu(&mut acquis, "mage", "hache", &salle, 999).is_err(), "pas de sa classe");
        assert_eq!(acquis, avant);

        let achat = apprendre_jeu(&mut acquis, "guerrier", "hache", &salle, 150).unwrap();
        assert_eq!(achat.prix, salle.prix_jeu_armes);
        assert_eq!(achat.armes, vec![registre().jeu("hache").unwrap().armes.0.clone()]);
        assert_eq!(achat.message, "Hache à deux mains appris : vos armes sont dans le sac.");
        let jeu = &acquis.jeux["hache"];
        let def = registre().jeu("hache").unwrap();
        let (a, b) = (def.competences[0].0.clone(), def.competences[1].0.clone());
        assert_eq!(jeu.competences.len(), 2);
        assert!(jeu.competences.values().all(|n| *n == NIVEAU_DEPART));
        assert_eq!(jeu.placees, [Some(a), Some(b)]);
        assert_eq!((jeu.maitrise, jeu.points_depenses), (0, 0));

        // Deux armes : épée et bouclier.
        let achat = apprendre_jeu(&mut acquis, "guerrier", "epee_bouclier", &salle, 150).unwrap();
        assert_eq!(achat.armes.len(), 2);
    }

    #[test]
    fn apprendre_et_activer_un_ultime_ou_un_passif() {
        let salle = SalleArmes::actuel();
        let mut acquis = acquis_grande_epee(0);
        let avant = acquis.clone();
        assert!(apprendre_ultime(&mut acquis, "guerrier", "seisme", &salle, 199).unwrap_err().contains("Il vous manque 1 or"));
        assert!(apprendre_ultime(&mut acquis, "guerrier", "fureur", &salle, 999).unwrap_err().contains("déjà"));
        assert!(apprendre_ultime(&mut acquis, "guerrier", "balayage_lourd", &salle, 999).unwrap_err().contains("inconnu"));
        assert!(apprendre_passif(&mut acquis, "guerrier", "robuste", &salle, 999).unwrap_err().contains("déjà"));
        assert!(apprendre_passif(&mut acquis, "guerrier", "zzz", &salle, 999).is_err());
        assert!(activer_ultime(&mut acquis, "guerrier", "seisme").unwrap_err().contains("pas encore"));
        assert!(activer_ultime(&mut acquis, "guerrier", "fureur").unwrap_err().contains("déjà actif"));
        assert_eq!(acquis, avant);

        let achat = apprendre_ultime(&mut acquis, "guerrier", "seisme", &salle, 200).unwrap();
        assert_eq!((achat.prix, achat.armes.len()), (salle.prix_ultime, 0));
        assert_eq!(acquis.ultime.as_deref(), Some("seisme"), "il devient l'ultime actif");
        assert_eq!(acquis.ultimes, vec!["fureur".to_string(), "seisme".into()]);
        activer_ultime(&mut acquis, "guerrier", "fureur").unwrap();
        assert_eq!(acquis.ultime.as_deref(), Some("fureur"));

        let achat = apprendre_passif(&mut acquis, "guerrier", "peau_dure", &salle, 120).unwrap();
        assert_eq!(achat.prix, salle.prix_passif);
        assert_eq!(acquis.passif.as_deref(), Some("peau_dure"));
        assert_eq!(activer_passif(&mut acquis, "guerrier", "robuste").unwrap(), "Robuste est votre passif actif.");
        assert_eq!(acquis.passif.as_deref(), Some("robuste"));
        assert!(apprendre_passif(&mut acquis, "mage", "robuste", &salle, 999).is_err());
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
