//! Icebreaker: the recruits of the day stand in front of the cart (X.8), one
//! NPC each, with the equipment of their class and their name above them. They
//! follow the cart: hired or gone with a new day, their NPC goes; back with
//! their chunk, like the other village NPCs. `sys::icebreaker_charrette` keeps
//! them in step every second.

use crate::{icebreaker_classes, icebreaker_registre::Recrue};
use common::{
    comp::{self, Alignment, Body, Content, agent::TradingBehavior},
    event::NpcBuilder,
};
use specs::{Component, DenseVecStorage};
use std::collections::HashMap;
use vek::*;

/// Marks the NPC standing for a recruit of the cart, by the recruit's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecrueCharrette(pub String);

impl Component for RecrueCharrette {
    type Storage = DenseVecStorage<Self>;
}

/// Where the recruits stand, and which ones are being created.
#[derive(Default)]
pub struct IcebreakerCharrette {
    /// One post per recruit, in front of the cart (none: no recruit shown).
    posts: Vec<Vec3<f32>>,
    /// Recruits whose NPC was asked for, and when (`Time`): the NPC takes a
    /// tick to exist, and is then marked by its name.
    pending: HashMap<String, f64>,
    /// When the recruits were last put in step.
    pub last_run: f64,
}

/// How long a creation is waited for before asking again.
const PENDING_TIMEOUT: f64 = 5.0;

impl IcebreakerCharrette {
    pub fn new(posts: Vec<Vec3<f32>>) -> Self {
        Self {
            posts,
            pending: HashMap::new(),
            last_run: 0.0,
        }
    }

    /// Where the `index`th recruit stands, if recruits are shown at all (past
    /// the posts, they share the last one).
    pub fn post(&self, index: usize) -> Option<Vec3<f32>> {
        self.posts
            .get(index.min(self.posts.len().saturating_sub(1)))
            .copied()
    }

    /// Whether `nom` was asked for less than `PENDING_TIMEOUT` ago.
    pub fn pending(&self, nom: &str, now: f64) -> bool {
        self.pending.get(nom).is_some_and(|t| now - t < PENDING_TIMEOUT)
    }

    /// Whether `nom` was asked for at all and not seen yet.
    pub fn asked_for(&self, nom: &str) -> bool { self.pending.contains_key(nom) }

    pub fn asked(&mut self, nom: &str, now: f64) { self.pending.insert(nom.to_string(), now); }

    pub fn done(&mut self, nom: &str) { self.pending.remove(nom); }
}

/// The recruit `recrue` as an NPC standing at `pos`: its class's equipment,
/// its name, no trade, a word when talked to.
pub fn recruit_npc(recrue: &Recrue, pos: Vec3<f32>) -> Option<NpcBuilder> {
    let class = icebreaker_classes::class(&recrue.classe)?;
    let body = Body::Humanoid(comp::humanoid::Body::random());
    let inventory = icebreaker_classes::starting_inventory(&class);
    let mut agent = comp::Agent::from_body(&body)
        .with_patrol_origin(pos)
        .with_idle_wander_factor(0.0);
    agent.behavior.allow(comp::agent::BehaviorCapability::SPEAK);
    agent.behavior.trading_behavior = TradingBehavior::Recrue;
    Some(
        NpcBuilder::new(
            comp::Stats::new(Content::Plain(recrue.nom.clone()), body),
            body,
            Alignment::Npc,
        )
        .with_health(comp::Health::new(body))
        .with_inventory(inventory)
        .with_agent(agent),
    )
}

/// What to do so that the NPCs (`present`, by name) match the cart
/// (`wanted`): the names to create, and the names to remove.
pub fn synchroniser(wanted: &[String], present: &[String]) -> (Vec<String>, Vec<String>) {
    let a_creer = wanted
        .iter()
        .filter(|nom| !present.contains(nom))
        .cloned()
        .collect();
    let a_retirer = present
        .iter()
        .filter(|nom| !wanted.contains(nom))
        .cloned()
        .collect();
    (a_creer, a_retirer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noms(list: &[&str]) -> Vec<String> { list.iter().map(|s| s.to_string()).collect() }

    #[test]
    fn the_npcs_follow_the_cart() {
        let (a_creer, a_retirer) =
            synchroniser(&noms(&["Leif", "Ragna"]), &noms(&["Ragna", "Clovis"]));
        assert_eq!(a_creer, noms(&["Leif"]));
        assert_eq!(a_retirer, noms(&["Clovis"]));
        assert_eq!(synchroniser(&[], &[]), (Vec::new(), Vec::new()));
    }

    #[test]
    fn the_posts_are_shared_past_the_last_and_absent_without_any() {
        let posts = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)];
        let charrette = IcebreakerCharrette::new(posts);
        assert_eq!(charrette.post(1), Some(Vec3::new(2.0, 0.0, 0.0)));
        assert_eq!(charrette.post(7), Some(Vec3::new(2.0, 0.0, 0.0)));
        assert_eq!(IcebreakerCharrette::default().post(0), None);
    }

    #[test]
    fn a_recruit_npc_wears_its_class_and_its_name() {
        let recrue = Recrue {
            nom: "Leif".into(),
            classe: "guerrier".into(),
        };
        let npc = recruit_npc(&recrue, Vec3::zero()).expect("guerrier exists");
        assert_eq!(npc.stats.name, Content::Plain("Leif".into()));
        assert!(
            npc.inventory
                .equipped(common::comp::inventory::slot::EquipSlot::ActiveMainhand)
                .is_some()
        );
        let unknown = Recrue {
            nom: "X".into(),
            classe: "inconnue".into(),
        };
        assert!(recruit_npc(&unknown, Vec3::zero()).is_none());
    }
}
