use common::rtsim::DungeonSlot;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use vek::*;

/// What players have done inside dungeons, so that dungeons keep their traces
/// across chunk reloads and server restarts: killed NPCs stay dead and
/// collected sprites (chests, loot) stay collected.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DungeonTraces {
    killed: HashSet<DungeonSlot>,
    collected: HashSet<Vec3<i32>>,
}

impl DungeonTraces {
    pub fn record_kill(&mut self, slot: DungeonSlot) { self.killed.insert(slot); }

    pub fn is_killed(&self, slot: &DungeonSlot) -> bool { self.killed.contains(slot) }

    pub fn record_collected(&mut self, wpos: Vec3<i32>) { self.collected.insert(wpos); }

    pub fn is_collected(&self, wpos: Vec3<i32>) -> bool { self.collected.contains(&wpos) }

    /// Collected sprites whose position satisfies `filter` (typically: inside a
    /// given chunk).
    pub fn collected_where(&self, filter: impl Fn(Vec3<i32>) -> bool) -> Vec<Vec3<i32>> {
        self.collected.iter().copied().filter(|p| filter(*p)).collect()
    }

    /// Forgets what was done where `area` holds (a dungeon made new again for
    /// another expedition, Icebreaker decision 0016).
    /// Returns how many kills and collected sprites were forgotten.
    pub fn forget_where(&mut self, area: impl Fn(Vec3<i32>) -> bool) -> (usize, usize) {
        let before = (self.killed.len(), self.collected.len());
        self.killed.retain(|slot| !area(slot.wpos));
        self.collected.retain(|wpos| !area(*wpos));
        (before.0 - self.killed.len(), before.1 - self.collected.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traces_survive_a_save_round_trip() {
        let mut traces = DungeonTraces::default();
        let slot = DungeonSlot {
            wpos: Vec3::new(10, -4, 120),
            nth: 1,
        };
        traces.record_kill(slot);
        traces.record_collected(Vec3::new(12, -4, 120));

        let bytes = rmp_serde::encode::to_vec_named(&traces).unwrap();
        let loaded: DungeonTraces = rmp_serde::decode::from_slice(&bytes).unwrap();

        assert!(loaded.is_killed(&slot));
        assert!(!loaded.is_killed(&DungeonSlot { nth: 0, ..slot }));
        assert_eq!(loaded.collected_where(|p| p.x >= 0), vec![Vec3::new(
            12, -4, 120
        )]);
        assert!(loaded.collected_where(|p| p.x < 0).is_empty());
    }

    #[test]
    fn a_dungeon_made_new_forgets_its_traces_only() {
        let mut traces = DungeonTraces::default();
        let here = DungeonSlot {
            wpos: Vec3::new(10, 0, 100),
            nth: 0,
        };
        let there = DungeonSlot {
            wpos: Vec3::new(5000, 0, 100),
            nth: 0,
        };
        traces.record_kill(here);
        traces.record_kill(there);
        traces.record_collected(Vec3::new(12, 0, 100));
        traces.record_collected(Vec3::new(5002, 0, 100));

        assert_eq!(traces.forget_where(|p| p.x < 1000), (1, 1));
        assert!(!traces.is_killed(&here));
        assert!(traces.is_killed(&there));
        assert!(!traces.is_collected(Vec3::new(12, 0, 100)));
        assert!(traces.is_collected(Vec3::new(5002, 0, 100)));
    }
}
