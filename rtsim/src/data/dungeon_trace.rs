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

    /// Collected sprites whose position satisfies `filter` (typically: inside a
    /// given chunk).
    pub fn collected_where(&self, filter: impl Fn(Vec3<i32>) -> bool) -> Vec<Vec3<i32>> {
        self.collected.iter().copied().filter(|p| filter(*p)).collect()
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
}
