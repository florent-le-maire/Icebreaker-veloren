//! Icebreaker: the mission of a dungeon of our generator. It is won by killing
//! the leader of the dungeon (its chief, or its boss), who always guards the
//! goal room; everyone is then told, with what was killed and looted. Each
//! dungeon has its own: the one of a lone dungeon, or one per dungeon slot of
//! the campaign (I.10).

use common::{comp::Content, rtsim::DungeonSlot, store::Id};
use vek::*;
use crate::sys::icebreaker_sleep::{IcebreakerSleep, SleepRules};
use tracing::info;
use world::site::Site;

/// The missions of the dungeons of this server, by site.
#[derive(Default)]
pub struct IcebreakerMission(Vec<(Id<Site>, Mission)>);

impl IcebreakerMission {
    /// Sets the mission of the dungeon of `site` (`None`: it has none).
    pub fn set(&mut self, site: Id<Site>, mission: Option<Mission>) {
        self.0.retain(|(s, _)| *s != site);
        self.0.extend(mission.map(|m| (site, m)));
    }

    /// The mission whose leader is the NPC of this slot.
    pub fn led_by(&self, slot: &DungeonSlot) -> Option<&Mission> {
        self.0.iter().map(|(_, m)| m).find(|m| m.is_leader(slot))
    }

    pub fn len(&self) -> usize { self.0.len() }

    /// Icebreaker : la mission du donjon de `site`, s'il en a une.
    pub fn of(&self, site: Id<Site>) -> Option<&Mission> {
        self.0.iter().find(|(s, _)| *s == site).map(|(_, m)| m)
    }
}

/// Sets the mission and the sleepers of the dungeon of `site` from its plan
/// (none if it is not one of our generator's dungeons or caves).
pub fn set_dungeon(missions: &mut IcebreakerMission, sleep: &mut IcebreakerSleep, id: Id<Site>, site: &Site) {
    let targets = world::icebreaker::dungeon_mission(site);
    if let Some(t) = &targets {
        let (creatures, chests, asleep) = (t.creatures.len(), t.chests.len(), t.sleepers.len());
        info!(?id, leader = ?t.leader, creatures, chests, asleep, "Icebreaker dungeon mission");
        sleep.rules = SleepRules {
            wake_radius: t.sleep.wake_radius,
            wake_radius_sneaking: t.sleep.wake_radius_sneaking,
            alert_radius: t.sleep.alert_radius,
            zzz_interval: t.sleep.zzz_interval,
        };
    }
    sleep.set(id, targets.iter().flat_map(|t| t.sleepers.iter().copied()).collect());
    missions.set(id, targets.map(|t| Mission::new(t.leader, t.creatures, t.chests)));
}

/// Icebreaker : ce qui a été fait dans un donjon (le bilan de retour).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decompte {
    /// Le chef est tombé.
    pub accomplie: bool,
    pub tuees: usize,
    pub creatures: usize,
    pub ouverts: usize,
    pub coffres: usize,
}

/// What the mission of one dungeon counts, as spawn positions of dungeon slots
/// (creatures) and positions of sprites (chests).
#[derive(Default)]
pub struct Mission {
    /// Where the leader appears, and its kind (`chief` or `boss`).
    leader: Option<(Vec3<i32>, String)>,
    creatures: Vec<Vec3<i32>>,
    chests: Vec<Vec3<i32>>,
}

impl Mission {
    pub fn new(leader: Option<(Vec3<i32>, String)>, creatures: Vec<Vec3<i32>>, chests: Vec<Vec3<i32>>) -> Self {
        Self {
            leader,
            creatures,
            chests,
        }
    }

    /// Whether the NPC of this slot is the leader of the dungeon.
    pub fn is_leader(&self, slot: &DungeonSlot) -> bool {
        slot.nth == 0 && self.leader.as_ref().is_some_and(|(wpos, _)| *wpos == slot.wpos)
    }

    /// Icebreaker : ce que la mission compte, au moment où on la regarde.
    pub fn decompte(
        &self,
        is_killed: impl Fn(&DungeonSlot) -> bool,
        is_collected: impl Fn(Vec3<i32>) -> bool,
    ) -> Decompte {
        Decompte {
            accomplie: self
                .leader
                .as_ref()
                .is_some_and(|(wpos, _)| is_killed(&DungeonSlot { wpos: *wpos, nth: 0 })),
            tuees: self
                .creatures
                .iter()
                .filter(|wpos| is_killed(&DungeonSlot { wpos: **wpos, nth: 0 }))
                .count(),
            creatures: self.creatures.len(),
            ouverts: self.chests.iter().filter(|wpos| is_collected(**wpos)).count(),
            coffres: self.chests.len(),
        }
    }

    /// The message telling everyone the mission is accomplished, with the
    /// creatures killed and the chests opened so far.
    pub fn accomplished_message(
        &self,
        is_killed: impl Fn(&DungeonSlot) -> bool,
        is_collected: impl Fn(Vec3<i32>) -> bool,
    ) -> Content {
        let leader = match self.leader.as_ref().map(|(_, kind)| kind.as_str()) {
            Some("boss") => "le boss",
            _ => "le chef",
        };
        let Decompte {
            tuees: killed,
            creatures,
            ouverts: opened,
            coffres,
            ..
        } = self.decompte(is_killed, is_collected);
        Content::Plain(format!(
            "Mission accomplie : {leader} du donjon est tombé ! Bilan : {killed} créature(s) tuée(s) \
             sur {creatures}, {opened} coffre(s) ouvert(s) sur {coffres}.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_leader_ends_the_mission_with_a_tally() {
        let spot = |x| Vec3::new(x, 0, 10);
        let mission = Mission::new(
            Some((spot(1), "chief".to_string())),
            vec![spot(1), spot(2), spot(3)],
            vec![spot(4), spot(5)],
        );
        assert!(mission.is_leader(&DungeonSlot { wpos: spot(1), nth: 0 }));
        assert!(!mission.is_leader(&DungeonSlot { wpos: spot(2), nth: 0 }));
        assert!(!mission.is_leader(&DungeonSlot { wpos: spot(1), nth: 1 }));
        assert!(!Mission::default().is_leader(&DungeonSlot { wpos: spot(1), nth: 0 }));

        let decompte = mission.decompte(|slot| slot.wpos.x <= 2, |wpos| wpos.x == 5);
        assert_eq!(decompte, Decompte {
            accomplie: true,
            tuees: 2,
            creatures: 3,
            ouverts: 1,
            coffres: 2,
        });
        // Le chef est vivant : la mission n'est pas accomplie.
        assert!(!mission.decompte(|slot| slot.wpos.x == 2, |_| false).accomplie);
        assert!(!Mission::default().decompte(|_| true, |_| true).accomplie);

        let message = mission.accomplished_message(|slot| slot.wpos.x <= 2, |wpos| wpos.x == 5);
        assert_eq!(
            message,
            Content::Plain(
                "Mission accomplie : le chef du donjon est tombé ! Bilan : 2 créature(s) tuée(s) sur 3, 1 \
                 coffre(s) ouvert(s) sur 2."
                    .to_string()
            )
        );
    }
}

#[cfg(test)]
mod per_dungeon {
    use super::*;

    #[test]
    fn each_dungeon_slot_has_its_own_mission() {
        let at = |x| DungeonSlot { wpos: Vec3::new(x, 0, 10), nth: 0 };
        let mission = |x| Some(Mission::new(Some((Vec3::new(x, 0, 10), "chief".into())), vec![], vec![]));
        let mut missions = IcebreakerMission::default();
        missions.set(Id::new(0), mission(1));
        missions.set(Id::new(1), mission(1000));
        assert!(missions.led_by(&at(1)).is_some());
        assert!(missions.led_by(&at(1000)).is_some());
        // A new plan for slot 1: its old leader leads nothing any more.
        missions.set(Id::new(1), mission(2000));
        assert!(missions.led_by(&at(1000)).is_none());
        assert!(missions.led_by(&at(2000)).is_some());
        missions.set(Id::new(0), None);
        assert!(missions.led_by(&at(1)).is_none());
        assert_eq!(missions.len(), 1);
    }
}
