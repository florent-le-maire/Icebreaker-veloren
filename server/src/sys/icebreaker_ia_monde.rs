//! Icebreaker IA : ce qui se passe entre nos ennemis (décision 0024). Les agents tournent en
//! parallèle et ne se voient pas décider ; ce système, lui, passe après eux, un par un :
//! - il produit les bruits (un coup qui touche, une mort) et les fait entendre aux cerveaux à portée
//!   (`icebreaker_ia::Bruits`, `assets/icebreaker/ia/bruits.ron`) ; un dormeur qui entend se réveille
//!   (`icebreaker_sleep`) ;
//! - il porte les cris d'un cerveau (`Cerveau::cris`) à ceux qui les entendent (`Cerveau::recevoir`) ;
//! - il tient la liste des nôtres (`IcebreakerIaAllies`) que les agents lisent au tick suivant.

use common::{
    comp::{
        Agent, Alignment, Body, Content, Health, Pos, UnresolvedChatMsg,
        agent::{AgentEvent, Sound, SoundKind},
    },
    event::{ChatEvent, EventBus},
    icebreaker_ia::{self, Allie, IcebreakerIaAllies, Message},
    resources::{EntitiesDiedLastTick, Time},
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use hashbrown::HashSet;
use specs::{Entities, Entity, Join, Read, ReadStorage, Write, WriteStorage};
use vek::Vec3;

/// Ce que le système retient d'un tick à l'autre.
#[derive(Default)]
pub struct IcebreakerIaMonde {
    /// Les coups reçus jusqu'à cette heure ont déjà fait leur bruit.
    vu_jusqu_a: f64,
    /// Les bruits demandés par ailleurs (lieu d'entraînement), à faire entendre au prochain tick.
    demandes: Vec<(&'static str, Vec3<f32>)>,
    /// Nos ennemis en vie au tick précédent : un mort qui en était est un allié tombé.
    connus: HashSet<Entity>,
}

impl IcebreakerIaMonde {
    /// Un bruit de cette sorte (`bruits.ron`) à cet endroit.
    pub fn bruit(&mut self, sorte: &'static str, lieu: Vec3<f32>) { self.demandes.push((sorte, lieu)); }
}

/// Ce qui s'entend à ce tick.
enum Son {
    /// Un bruit sans auteur connu : celui qui l'entend va voir.
    Bruit(&'static str, Vec3<f32>),
    /// Un des nôtres est mort là.
    Mort(Vec3<f32>),
    /// Le cri d'un des nôtres, fait là.
    Cri(Entity, Vec3<f32>, Message),
}

/// Ce qu'un ennemi dit quand il crie ce message.
fn paroles(message: &Message) -> &'static str {
    match message {
        Message::Signalement { .. } => "Des intrus !",
        Message::AllieTombe { .. } => "Il est tombé !",
    }
}

#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Read<'a, EntitiesDiedLastTick>,
        Read<'a, EventBus<ChatEvent>>,
        Write<'a, IcebreakerIaMonde>,
        Write<'a, IcebreakerIaAllies>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Alignment>,
        ReadStorage<'a, Uid>,
        WriteStorage<'a, Agent>,
    );

    const NAME: &'static str = "icebreaker_ia_monde";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (entities, time, morts, chat_bus, mut monde, mut allies, positions, healths, bodies, alignments, uids, mut agents): Self::SystemData,
    ) {
        // Les sons de ce tick : les coups, les morts, les bruits demandés, les cris.
        let depuis = monde.vu_jusqu_a;
        monde.vu_jusqu_a = time.0;
        let mut sons: Vec<Son> = (&positions, &healths)
            .join()
            .filter(|(_, h)| h.last_change.amount < 0.0 && h.last_change.time.0 > depuis)
            .map(|(pos, _)| Son::Bruit("combat", pos.0))
            .collect();
        sons.extend(morts.0.iter().map(|(entite, pos)| {
            if monde.connus.contains(entite) { Son::Mort(pos.0) } else { Son::Bruit("mort", pos.0) }
        }));
        let demandes = std::mem::take(&mut monde.demandes);
        sons.extend(demandes.into_iter().map(|(sorte, lieu)| Son::Bruit(sorte, lieu)));

        // Les nôtres : leur liste pour le tick suivant, et ce qu'ils viennent de crier.
        allies.0.clear();
        monde.connus.clear();
        let mut chat = chat_bus.emitter();
        for (entite, agent, pos, uid, _, _) in (&entities, &mut agents, &positions, &uids, &bodies, &alignments)
            .join()
            .filter(|(.., body, alignment)| matches!(alignment, Alignment::Enemy) && body.is_humanoid())
        {
            monde.connus.insert(entite);
            let Some(cerveau) = agent.icebreaker_ia.as_mut() else { continue };
            allies.0.push(Allie {
                id: uid.0.get(),
                pos: pos.0,
                alerte: cerveau.alerte(),
                messager: cerveau.messager(),
            });
            for message in cerveau.cris() {
                chat.emit(ChatEvent {
                    msg: UnresolvedChatMsg::npc(*uid, Content::Plain(paroles(&message).to_string())),
                    from_client: false,
                });
                sons.push(Son::Cri(entite, pos.0, message));
            }
        }
        if sons.is_empty() {
            return;
        }

        let portees = icebreaker_ia::bruits();
        for (entite, agent, pos, body, _) in (&entities, &mut agents, &positions, &bodies, &alignments)
            .join()
            .filter(|(.., a)| matches!(a, Alignment::Enemy))
        {
            let fiche = agent.icebreaker_ia.as_ref().map_or(("", ""), |c| c.fiche());
            let Some(profil) = icebreaker_ia::profil(Some(body), fiche) else { continue };
            let finesse = profil.ouie.unwrap_or(0.0) * if agent.icebreaker_asleep { portees.endormi } else { 1.0 };
            for son in &sons {
                let (sorte, lieu) = match son {
                    Son::Bruit(sorte, lieu) => (*sorte, *lieu),
                    Son::Mort(lieu) => ("mort", *lieu),
                    Son::Cri(auteur, ..) if *auteur == entite => continue,
                    Son::Cri(_, lieu, _) => ("cri", *lieu),
                };
                if !portees.entend(sorte, finesse, lieu - pos.0) {
                    continue;
                }
                if agent.icebreaker_asleep {
                    // Le réveil est l'affaire de `icebreaker_sleep` : il guette un son dans la boîte.
                    agent.inbox.push_back(AgentEvent::ServerSound(Sound::new(SoundKind::Unknown, lieu, 1.0, time.0)));
                    break;
                }
                let cerveau = agent.icebreaker_ia.get_or_insert_default();
                match son {
                    Son::Bruit(..) => cerveau.entendre(lieu, time.0),
                    Son::Mort(lieu) => cerveau.recevoir(Message::AllieTombe { lieu: *lieu }, time.0),
                    Son::Cri(_, _, message) => cerveau.recevoir(*message, time.0),
                }
                tracing::debug!(sorte, ?lieu, "Icebreaker IA : un ennemi entend");
            }
        }
    }
}
