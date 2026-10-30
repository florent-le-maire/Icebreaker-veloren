//! Icebreaker: the mission of a dungeon of our generator. It is won by killing
//! the leader of the dungeon (its chief, or its boss), who always guards the
//! goal room; everyone is then told, with what was killed and looted.

use common::{comp::Content, rtsim::DungeonSlot};
use vek::*;

/// What the mission counts, as spawn positions of dungeon slots (creatures)
/// and positions of sprites (chests). Empty outside a generated dungeon.
#[derive(Default)]
pub struct IcebreakerMission {
    /// Where the leader appears, and its kind (`chief` or `boss`).
    leader: Option<(Vec3<i32>, String)>,
    creatures: Vec<Vec3<i32>>,
    chests: Vec<Vec3<i32>>,
}

impl IcebreakerMission {
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
        let killed = self
            .creatures
            .iter()
            .filter(|wpos| is_killed(&DungeonSlot { wpos: **wpos, nth: 0 }))
            .count();
        let opened = self.chests.iter().filter(|wpos| is_collected(**wpos)).count();
        Content::Plain(format!(
            "Mission accomplie : {leader} du donjon est tombé ! Bilan : {killed} créature(s) tuée(s) \
             sur {}, {opened} coffre(s) ouvert(s) sur {}.",
            self.creatures.len(),
            self.chests.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_leader_ends_the_mission_with_a_tally() {
        let spot = |x| Vec3::new(x, 0, 10);
        let mission = IcebreakerMission::new(
            Some((spot(1), "chief".to_string())),
            vec![spot(1), spot(2), spot(3)],
            vec![spot(4), spot(5)],
        );
        assert!(mission.is_leader(&DungeonSlot { wpos: spot(1), nth: 0 }));
        assert!(!mission.is_leader(&DungeonSlot { wpos: spot(2), nth: 0 }));
        assert!(!mission.is_leader(&DungeonSlot { wpos: spot(1), nth: 1 }));
        assert!(!IcebreakerMission::default().is_leader(&DungeonSlot { wpos: spot(1), nth: 0 }));

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
