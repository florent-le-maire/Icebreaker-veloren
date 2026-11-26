//! Icebreaker IA : ce qui se passe entre nos ennemis (décision 0024). Les agents tournent en
//! parallèle et ne se voient pas décider ; ce système, lui, passe après eux, un par un : il produit
//! les bruits (un coup qui touche, une mort) et les fait entendre aux cerveaux à portée
//! (`icebreaker_ia::Bruits`, `assets/icebreaker/ia/bruits.ron`). Un dormeur qui entend se réveille
//! (`icebreaker_sleep`).

use common::{
    comp::{
        Agent, Alignment, Body, Health, Pos,
        agent::{AgentEvent, Sound, SoundKind},
    },
    icebreaker_ia,
    resources::{EntitiesDiedLastTick, Time},
};
use common_ecs::{Job, Origin, Phase, System};
use specs::{Join, Read, ReadStorage, Write, WriteStorage};
use vek::Vec3;

/// Ce que le système retient d'un tick à l'autre.
#[derive(Default)]
pub struct IcebreakerIaMonde {
    /// Les coups reçus jusqu'à cette heure ont déjà fait leur bruit.
    vu_jusqu_a: f64,
    /// Les bruits demandés par ailleurs (cris, lieu d'entraînement), à faire entendre au prochain tick.
    demandes: Vec<(&'static str, Vec3<f32>)>,
}

impl IcebreakerIaMonde {
    /// Un bruit de cette sorte (`bruits.ron`) à cet endroit.
    pub fn bruit(&mut self, sorte: &'static str, lieu: Vec3<f32>) { self.demandes.push((sorte, lieu)); }
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Read<'a, Time>,
        Read<'a, EntitiesDiedLastTick>,
        Write<'a, IcebreakerIaMonde>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Alignment>,
        WriteStorage<'a, Agent>,
    );

    const NAME: &'static str = "icebreaker_ia_monde";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (time, morts, mut monde, positions, healths, bodies, alignments, mut agents): Self::SystemData,
    ) {
        // Les bruits de ce tick : (sorte, endroit).
        let depuis = monde.vu_jusqu_a;
        monde.vu_jusqu_a = time.0;
        let mut bruits: Vec<(&str, Vec3<f32>)> = (&positions, &healths)
            .join()
            .filter(|(_, h)| h.last_change.amount < 0.0 && h.last_change.time.0 > depuis)
            .map(|(pos, _)| ("combat", pos.0))
            .collect();
        bruits.extend(morts.0.iter().map(|(_, pos)| ("mort", pos.0)));
        bruits.append(&mut monde.demandes);
        if bruits.is_empty() {
            return;
        }

        let portees = icebreaker_ia::bruits();
        for (agent, pos, body, _) in
            (&mut agents, &positions, &bodies, &alignments).join().filter(|(.., a)| matches!(a, Alignment::Enemy))
        {
            let Some(profil) = icebreaker_ia::profil(Some(body)) else { continue };
            let finesse = profil.ouie.unwrap_or(0.0) * if agent.icebreaker_asleep { portees.endormi } else { 1.0 };
            let Some((_, lieu)) =
                bruits.iter().rev().find(|(sorte, lieu)| portees.entend(sorte, finesse, *lieu - pos.0))
            else {
                continue;
            };
            if agent.icebreaker_asleep {
                // Le réveil est l'affaire de `icebreaker_sleep` : il guette un son dans la boîte.
                agent.inbox.push_back(AgentEvent::ServerSound(Sound::new(SoundKind::Unknown, *lieu, 1.0, time.0)));
            } else {
                agent.icebreaker_ia.get_or_insert_default().entendre(*lieu, time.0);
                tracing::debug!(?lieu, "Icebreaker IA : un ennemi entend un bruit");
            }
        }
    }
}
