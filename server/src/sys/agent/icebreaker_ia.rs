//! Icebreaker IA : le branchement du cerveau de nos ennemis (crate `icebreaker-ia`, décision 0024) dans
//! l'arbre de comportement de Veloren. Pour un ennemi humanoïde, [`noeud`] mesure ce qu'il perçoit,
//! laisse le cerveau décider, puis exécute son intention avec les moyens de Veloren (`idle`, `attack`,
//! `flee`, `path_toward_target`). La décision elle-même se teste sans le jeu :
//! `cargo test -p icebreaker-ia`.

use super::{
    behavior_tree::BehaviorData,
    consts::DAMAGE_MEMORY_DURATION,
    data::{AgentData, Path, ReadData, TargetData},
    util::{is_dead, is_invulnerable},
};
use common::{
    combat::perception_dist_multiplier_from_stealth,
    comp::{
        Alignment, Body, Content, ControlAction, InputKind, UtteranceKind, ability::AbilityInput, agent::Target,
        is_downed,
    },
    icebreaker_ia::{self, Allie, Apercu, Cerveau, Etat, Geste, Id, Intention, Niveau, Observation, Percu, Profil},
    states::utils::StageSection,
    terrain::Block,
    uid::Uid,
    util::Dir,
    vol::ReadVol,
};
use rand::RngExt;
use specs::Entity as EcsEntity;
use std::num::NonZeroU64;
use vek::Vec3;

fn entite(id: Id, read_data: &ReadData) -> Option<EcsEntity> {
    read_data.id_maps.uid_entity(Uid(NonZeroU64::new(id)?))
}

/// Nœud de `BehaviorTree::root()` : prend la main pour nos ennemis humanoïdes, laisse passer les
/// autres agents.
pub fn noeud(bdata: &mut BehaviorData) -> bool {
    let BehaviorData {
        agent,
        agent_data,
        read_data,
        emitters,
        controller,
        rng,
    } = bdata;
    if !matches!(agent_data.alignment, Some(Alignment::Enemy)) {
        return false;
    }
    let mut cerveau = agent.icebreaker_ia.take().unwrap_or_default();
    let Some(profil) = icebreaker_ia::profil(agent_data.body, cerveau.fiche()) else {
        agent.icebreaker_ia = Some(cerveau);
        return false;
    };
    let avant = cerveau.etat();
    let sort_avant = cerveau.sort().map(str::to_string);
    let geste_avant = cerveau.geste();
    // Icebreaker IA.6 : celui qui a une table de sorts choisit lui-même ; il lui faut ceux qu'il peut lancer.
    let prets = if profil.sorts.is_some() { sorts_prets(agent_data) } else { Vec::new() };
    let mut obs = observation(&cerveau, &profil, agent_data, read_data, rng.random());
    obs.sorts = prets.iter().map(|(_, nom)| nom.clone()).collect();
    let intention = cerveau.tick(&obs, &profil);
    let etat = cerveau.etat();
    let retenu = cerveau.retenu();
    let geste = cerveau.geste();
    // `/entrainement ia` : il dit sa défense.
    if icebreaker_ia::afficher()
        && geste != geste_avant
        && let Some(geste) = geste
    {
        let dit = match geste {
            Geste::Esquiver => "esquive !",
            Geste::Parer => "pare !",
        };
        agent_data.chat_npc(Content::Plain(dit.to_string()), emitters);
    }
    let sort = cerveau.sort().and_then(|voulu| prets.iter().find(|(_, nom)| nom == voulu)).map(|(touche, _)| *touche);
    // `/entrainement ia` : le mage dit le sort qu'il choisit.
    if icebreaker_ia::afficher()
        && let Some(voulu) = cerveau.sort().filter(|voulu| sort_avant.as_deref() != Some(*voulu))
    {
        agent_data.chat_npc(Content::Plain(format!("→ {voulu}")), emitters);
    }
    agent.icebreaker_ia = Some(cerveau);

    let yeux = agent_data.pos.0 + Vec3::unit_z() * agent_data.body.map_or(0.0, |b| b.eye_height(agent_data.scale));
    // `agent.target` ne sert plus qu'au compte de qui vise qui (icebreaker_focus) : seulement en combat.
    let cible = match intention {
        Intention::Combattre(id) => entite(id, read_data).and_then(|e| Some((e, read_data.positions.get(e)?))),
        _ => None,
    };
    if cible.is_none() {
        agent.target = None;
    }
    match intention {
        Intention::Combattre(_) if let Some((cible, pos)) = cible => {
            let vers_cible = (pos.0 - agent_data.pos.0).xy();
            let choisie_a = agent
                .target
                .filter(|t| t.target == cible)
                .map_or(read_data.time.0, |t| t.selected_at);
            agent.target = Some(Target {
                target: cible,
                hostile: true,
                selected_at: choisie_a,
                aggro_on: true,
                last_known_pos: Some(pos.0),
            });
            let cible = TargetData::new(pos, cible, read_data);
            agent_data.attack(agent, controller, &cible, read_data, rng);
            // Icebreaker IA.5 : son habileté filtre ce que la façon de se battre de Veloren propose.
            let (roule, pare) = (profil.roule.unwrap_or(true), profil.pare.unwrap_or(true));
            controller.actions.retain(|action| match action {
                ControlAction::StartInput { input, .. } => match input {
                    InputKind::Roll => roule,
                    InputKind::Block => pare,
                    // Le mage à table de sorts (IA.6) choisit lui-même : le choix de Veloren est retiré.
                    InputKind::Primary | InputKind::Secondary | InputKind::Ability(_) => {
                        !retenu && geste.is_none() && profil.sorts.is_none()
                    },
                    _ => true,
                },
                _ => true,
            });
            match geste {
                // Icebreaker IA.7 : une roulade de côté, hors du coup qui arrive.
                Some(Geste::Esquiver) => {
                    let cote = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
                    controller.inputs.move_dir =
                        vers_cible.rotated_z(cote * std::f32::consts::FRAC_PI_2).try_normalized().unwrap_or_default();
                    controller.push_basic_input(InputKind::Roll);
                },
                Some(Geste::Parer) => controller.push_basic_input(InputKind::Block),
                None => {
                    if let Some(touche) = sort {
                        controller.push_basic_input(touche);
                    }
                },
            }
        },
        Intention::Fuir(de) => {
            if etat == Etat::Fuite && avant != Etat::Fuite {
                agent_data.chat_npc_if_allowed_to_speak(
                    Content::localized("npc-speech-cultist_low_health_fleeing"),
                    agent,
                    emitters,
                );
            }
            agent_data.flee(agent, controller, read_data, &common::comp::Pos(de));
        },
        Intention::AllerVoir(lieu) => {
            agent_data.path_toward_target(agent, controller, lieu, read_data, Path::AtTarget, None);
        },
        // Icebreaker IA.8 : coincé, il fonce à travers ce qui le menace, ou se tapit face à lui.
        Intention::Forcer(lieu) => {
            agent_data.path_toward_target(agent, controller, lieu, read_data, Path::AtTarget, None);
        },
        Intention::SeTapir(vers) => {
            if let Some(dir) = Dir::from_unnormalized(vers - yeux) {
                controller.inputs.look_dir = dir;
            }
            controller.push_action(ControlAction::Unwield);
            controller.push_action(ControlAction::Sneak);
        },
        Intention::Prevenir(allie) => {
            agent_data.path_toward_target(agent, controller, allie, read_data, Path::AtTarget, None);
        },
        // Icebreaker IA.10 : un ordre (sa ronde), à l'allure demandée.
        Intention::Obeir(lieu, allure) => {
            agent_data.path_toward_target(agent, controller, lieu, read_data, Path::AtTarget, Some(allure));
        },
        Intention::Rentrer(poste) => {
            agent_data.path_toward_target(agent, controller, poste, read_data, Path::AtTarget, Some(ALLURE_RETOUR));
        },
        Intention::Regarder(vers) => {
            if let Some(dir) = Dir::from_unnormalized(vers - yeux) {
                controller.inputs.look_dir = dir;
            }
        },
        // Vaquer : l'errance de Veloren, ses ordres (rondes) et sa façon d'aller voir un bruit fort.
        Intention::Repos | Intention::Combattre(_) => {
            agent_data.handle_sounds_heard(agent, controller, read_data, emitters, rng);
        },
    }

    // Les signes au-dessus de lui (`creatures.md` : « ! » quand il passe en combat, « ? » quand il
    // cherche) ; le repos ne se dit qu'au lieu d'entraînement (`/entrainement ia`).
    if etat != avant {
        let signe = match etat {
            Etat::Combat => {
                controller.push_utterance(UtteranceKind::Angry);
                Some("!")
            },
            Etat::Enquete => Some("?"),
            // « Chaque décision tirée s'annonce. »
            Etat::Prevenir => Some("Je vais chercher les autres !"),
            Etat::Tapi => Some("Pitié !"),
            Etat::Force => Some("Poussez-vous !"),
            Etat::Deroute => Some("Fuite !"),
            Etat::Fuite => icebreaker_ia::afficher().then_some("fuite !"),
            Etat::Repos => icebreaker_ia::afficher().then_some("… repos"),
        };
        if let Some(signe) = signe {
            agent_data.chat_npc(Content::Plain(signe.to_string()), emitters);
        }
    }
    true
}

/// Cet adversaire arme-t-il un coup de mêlée contre l'ennemi (à `vers_lui` de lui) ? Dans combien de
/// secondes il tombe (0 : il est en train de tomber). Il faut qu'il soit à moins de `garde` blocs et
/// tourné vers l'ennemi.
fn coup_arme(adversaire: EcsEntity, vers_lui: Vec3<f32>, profil: &Profil, read_data: &ReadData) -> Option<f32> {
    let etat = read_data.char_states.get(adversaire)?;
    if etat.is_attack() {
        tracing::trace!(
            melee = etat.is_melee_attack(),
            distance = vers_lui.magnitude(),
            angle = read_data.orientations.get(adversaire).map(|o| o.look_vec().xy().angle_between(vers_lui.xy()).to_degrees()),
            etape = ?etat.stage_section(),
            "Icebreaker IA : un adversaire attaque"
        );
    }
    if !etat.is_melee_attack() || vers_lui.magnitude_squared() > profil.garde.unwrap_or(0.0).powi(2) {
        return None;
    }
    let regard = read_data.orientations.get(adversaire)?.look_vec().xy();
    if regard.angle_between(vers_lui.xy()).to_degrees() > ANGLE_DE_MENACE {
        return None;
    }
    match etat.stage_section()? {
        StageSection::Buildup => {
            let (arme, ecoule) = (etat.durations()?.buildup?, etat.timer()?);
            Some(arme.saturating_sub(ecoule).as_secs_f32())
        },
        StageSection::Action => Some(0.0),
        _ => None,
    }
}

/// Un coup menace celui qui est à moins de tant de degrés du regard de l'attaquant.
const ANGLE_DE_MENACE: f32 = 60.0;

/// Les sorts qu'il a et peut payer à ce tick : (la touche de Veloren, le nom de `combat::Sort`, c'est-à-dire
/// la fin de l'identifiant de la capacité).
fn sorts_prets(agent_data: &AgentData) -> Vec<(InputKind, String)> {
    let touches = [(AbilityInput::Primary, InputKind::Primary), (AbilityInput::Secondary, InputKind::Secondary)]
        .into_iter()
        .chain((0..TOUCHES_DE_SORTS).map(|i| (AbilityInput::Auxiliary(i), InputKind::Ability(i))));
    touches
        .filter_map(|(entree, touche)| {
            let (capacite, _, precise) = agent_data.active_abilities.activate_ability(
                entree,
                Some(agent_data.inventory),
                agent_data.skill_set,
                agent_data.body,
                Some(agent_data.char_state),
                agent_data.stance,
                agent_data.combo,
                agent_data.stats,
                agent_data.buffs,
            )?;
            let nom = precise.ability_id(Some(agent_data.char_state), Some(agent_data.inventory))?.rsplit('.').next()?;
            (capacite.energy_cost() <= agent_data.energy.current()).then(|| (touche, nom.to_string()))
        })
        .collect()
}

/// Combien de touches de compétence regarder, après l'attaque de base et la secondaire.
const TOUCHES_DE_SORTS: usize = 6;

/// Part de sa vitesse à laquelle il rentre à son poste.
const ALLURE_RETOUR: f32 = 0.6;

/// Ce que l'ennemi perçoit à ce tick : la règle est dans `icebreaker_ia::perception`, les mesures
/// (distance, angle, ligne de vue, discrétion) viennent d'ici.
fn observation(
    cerveau: &Cerveau,
    profil: &Profil,
    agent_data: &AgentData,
    read_data: &ReadData,
    hasard: f32,
) -> Observation {
    let temps = read_data.time.0;
    let moi: Vec3<f32> = agent_data.pos.0;
    let hauteur_yeux = |body: Option<&Body>, echelle: f32| body.map_or(0.0, |b| b.eye_height(echelle));
    let yeux = moi + Vec3::unit_z() * hauteur_yeux(agent_data.body, agent_data.scale);
    let regard = agent_data.ori.look_vec().xy();
    let rayon = profil.vue.unwrap_or(0.0).max(profil.dos.unwrap_or(0.0)).max(profil.meute_rayon.unwrap_or(0.0));
    let common::CachedSpatialGrid(grille) = agent_data.cached_spatial_grid;
    let proches: Vec<EcsEntity> = grille
        .in_circle_aabr(moi.xy(), rayon)
        .filter(|e| *e != *agent_data.entity)
        .collect();
    let en_vie = |e: EcsEntity| !is_dead(e, read_data) && !is_invulnerable(e, read_data);

    // Qui vient de le frapper, et quand.
    let frappe_par = agent_data.health.and_then(|h| {
        let par = h.last_change.damage_by()?;
        (h.last_change.amount < 0.0).then(|| (par.uid(), temps - h.last_change.time.0))
    });

    let percus = proches
        .iter()
        .copied()
        .filter(|e| read_data.healths.contains(*e) && en_vie(*e) && agent_data.is_enemy(*e, read_data))
        .filter_map(|e| {
            let pos = read_data.positions.get(e)?.0;
            let uid = *read_data.uids.get(e)?;
            let l_a_frappe_depuis = frappe_par.filter(|(par, _)| *par == uid).map(|(_, depuis)| depuis);
            let vers = pos - moi;
            let ses_yeux = pos
                + Vec3::unit_z()
                    * hauteur_yeux(read_data.bodies.get(e), read_data.scales.get(e).map_or(1.0, |s| s.0));
            let apercu = Apercu {
                distance: vers.magnitude(),
                angle: if vers.xy().magnitude_squared() < 0.01 {
                    0.0
                } else {
                    regard.angle_between(vers.xy()).to_degrees()
                },
                // Comme Veloren : d'yeux à yeux, arrêté par un bloc opaque.
                ligne_de_vue: read_data.terrain.ray(yeux, ses_yeux).until(Block::is_opaque).cast().0.powi(2)
                    >= yeux.distance_squared(ses_yeux) - 0.01,
                discretion: perception_dist_multiplier_from_stealth(
                    read_data.inventories.get(e),
                    read_data.char_states.get(e),
                    agent_data.msm,
                ),
            };
            // « Une créature touchée sait d'où vient le coup, même de dos. »
            let net = match icebreaker_ia::niveau(profil, &apercu) {
                _ if l_a_frappe_depuis.is_some_and(|d| d < DAMAGE_MEMORY_DURATION) => true,
                Niveau::Net => true,
                Niveau::Soupcon => false,
                Niveau::Rien => return None,
            };
            Some(Percu {
                id: uid.0.get(),
                pos,
                net,
                a_terre: is_downed(read_data.healths.get(e), read_data.char_states.get(e)),
                l_a_frappe_depuis,
                dessus: read_data.icebreaker_focus.0.get(&uid).copied().unwrap_or(0),
                approche: vers
                    .try_normalized()
                    .map_or(0.0, |direction| -read_data.velocities.get(e).map_or(0.0, |v| v.0.dot(direction))),
                frappe_dans: coup_arme(e, -vers, profil, read_data),
            })
        })
        .collect::<Vec<Percu>>();

    // Les siens dont il connaît le poste : seulement quand il a une décision à prendre.
    let voisinage = profil.voisinage.unwrap_or(0.0);
    let allies = if percus.is_empty() && !cerveau.alerte() {
        Vec::new()
    } else {
        read_data
            .icebreaker_allies
            .0
            .iter()
            .filter(|a| a.id != agent_data.uid.0.get() && a.pos.distance_squared(moi) < voisinage.powi(2))
            .cloned()
            .collect::<Vec<Allie>>()
    };

    // Les siens autour de lui, lui compris (pour ceux qui n'achèvent qu'à plusieurs).
    let espece = |b: Option<&Body>| match b {
        Some(Body::Humanoid(h)) => Some(h.species),
        _ => None,
    };
    let meute = 1 + profil.meute_rayon.filter(|_| profil.meute.is_some()).map_or(0, |rayon| {
        proches
            .iter()
            .filter(|e| {
                espece(read_data.bodies.get(**e)) == espece(agent_data.body)
                    && matches!(read_data.alignments.get(**e), Some(Alignment::Enemy))
                    && en_vie(**e)
                    && read_data.positions.get(**e).is_some_and(|p| p.0.distance_squared(moi) < rayon.powi(2))
            })
            .count()
    });

    Observation {
        temps,
        moi: agent_data.uid.0.get(),
        pos: moi,
        pv: agent_data.damage.min(1.0),
        meute,
        percus,
        allies,
        hasard,
        attaque: agent_data.char_state.is_attack(),
        sorts: Vec::new(),
        peut_parer: agent_data
            .active_abilities
            .activate_ability(
                AbilityInput::Guard,
                Some(agent_data.inventory),
                agent_data.skill_set,
                agent_data.body,
                Some(agent_data.char_state),
                agent_data.stance,
                agent_data.combo,
                agent_data.stats,
                agent_data.buffs,
            )
            .is_some(),
        cible_en_vie: cerveau
            .cible()
            .and_then(|id| entite(id, read_data))
            .is_some_and(|e| read_data.positions.contains(e) && en_vie(e)),
    }
}
