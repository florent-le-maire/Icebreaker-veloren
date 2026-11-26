//! Icebreaker IA : le branchement du cerveau de nos ennemis (crate `icebreaker-ia`, décision 0024) dans
//! l'arbre de comportement de Veloren. Pour un ennemi humanoïde, [`noeud`] construit ce qu'il perçoit,
//! laisse le cerveau décider, puis exécute son intention avec les moyens de Veloren (`idle`, `attack`,
//! `flee`). La décision elle-même se teste sans le jeu : `cargo test -p icebreaker-ia`.
//!
//! IA.1 : la perception est encore celle de Veloren (`detects_other`) ; IA.2 la remplace.

use super::{
    behavior_tree::BehaviorData,
    consts::DAMAGE_MEMORY_DURATION,
    data::{AgentData, ReadData, TargetData},
    util::{is_dead, is_invulnerable},
};
use common::{
    comp::{Agent, Alignment, Body, Content, Controller, UtteranceKind, agent::Target, is_downed},
    icebreaker_ia::{self, Cerveau, Etat, Id, Intention, Observation, Percu, Profil},
    uid::Uid,
};
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
    let Some(profil) = icebreaker_ia::profil(agent_data.body) else {
        return false;
    };

    let mut cerveau = agent.icebreaker_ia.take().unwrap_or_default();
    let avant = cerveau.etat();
    let obs = observation(&cerveau, &profil, agent, agent_data, controller, read_data);
    let intention = cerveau.tick(&obs, &profil);
    let etat = cerveau.etat();
    agent.icebreaker_ia = Some(cerveau);

    // L'intention vise quelqu'un qui n'est plus là : on vaque ce tick-ci.
    let vise = |id: Id| entite(id, read_data).and_then(|e| Some((e, read_data.positions.get(e)?)));
    match intention {
        Intention::Combattre(id) if let Some((cible, pos)) = vise(id) => {
            if agent.target.is_none() {
                controller.push_utterance(UtteranceKind::Angry);
            }
            // `agent.target` reste à jour : le compte de qui vise qui (icebreaker_focus) le lit.
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
        },
        Intention::Fuir(id) if let Some((_, pos)) = vise(id) => {
            agent.target = None;
            if avant != Etat::Fuite {
                agent_data.chat_npc_if_allowed_to_speak(
                    Content::localized("npc-speech-cultist_low_health_fleeing"),
                    agent,
                    emitters,
                );
            }
            agent_data.flee(agent, controller, read_data, pos);
        },
        _ => {
            agent.target = None;
            // L'errance de Veloren, ses ordres (rondes) et sa façon d'aller voir un bruit fort.
            agent_data.handle_sounds_heard(agent, controller, read_data, emitters, rng);
        },
    }

    if etat != avant && icebreaker_ia::afficher() {
        let texte = match etat {
            Etat::Repos => "… repos",
            Etat::Combat => "! combat",
            Etat::Fuite => "fuite !",
        };
        agent_data.chat_npc(Content::Plain(texte.to_string()), emitters);
    }
    true
}

/// Ce que l'ennemi perçoit à ce tick.
fn observation(
    cerveau: &Cerveau,
    profil: &Profil,
    agent: &Agent,
    agent_data: &AgentData,
    controller: &Controller,
    read_data: &ReadData,
) -> Observation {
    let temps = read_data.time.0;
    let moi: Vec3<f32> = agent_data.pos.0;
    let common::CachedSpatialGrid(grille) = agent_data.cached_spatial_grid;
    let proches: Vec<EcsEntity> = grille
        .in_circle_aabr(moi.xy(), agent.psyche.search_dist())
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
            let pos = read_data.positions.get(e)?;
            let uid = *read_data.uids.get(e)?;
            let l_a_frappe_depuis = frappe_par.filter(|(par, _)| *par == uid).map(|(_, depuis)| depuis);
            // Il le voit ou le sent tout près (Veloren), ou il vient d'en recevoir un coup : « une
            // créature touchée sait d'où vient le coup, même de dos ».
            let percu = l_a_frappe_depuis.is_some_and(|d| d < DAMAGE_MEMORY_DURATION)
                || agent_data.detects_other(agent, controller, &e, pos, read_data.scales.get(e), read_data);
            percu.then(|| Percu {
                id: uid.0.get(),
                distance: pos.0.distance(moi),
                a_terre: is_downed(read_data.healths.get(e), read_data.char_states.get(e)),
                l_a_frappe_depuis,
                dessus: read_data.icebreaker_focus.0.get(&uid).copied().unwrap_or(0),
            })
        })
        .collect();

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
        pv: agent_data.damage.min(1.0),
        meute,
        percus,
        cible_en_vie: cerveau
            .cible()
            .and_then(|id| entite(id, read_data))
            .is_some_and(|e| read_data.positions.contains(e) && en_vie(e)),
    }
}
