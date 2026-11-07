//! Icebreaker I.11: a plot whose layout can be replaced while the world runs
//! (a new dungeon or cave for each expedition, decision 0016, option 2). The
//! site and its tiles stay; chunks generated after the change draw the new
//! layout.

use super::*;
use std::sync::{Arc, RwLock};

pub struct Remplacable<T>(RwLock<Arc<T>>);

impl<T> Remplacable<T> {
    pub fn new(layout: T) -> Self { Self(RwLock::new(Arc::new(layout))) }

    /// The layout drawn now.
    pub fn get(&self) -> Arc<T> { self.0.read().unwrap_or_else(|e| e.into_inner()).clone() }

    /// Draws `layout` from now on.
    pub fn replace(&self, layout: T) { *self.0.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(layout); }
}

impl<T: Structure> Structure for Remplacable<T> {
    fn render_inner(&self, site: &Site, land: &Land, painter: &Painter) {
        self.get().render_inner(site, land, painter)
    }

    fn spawn_rules_inner(&self, spawn_rules: &mut SpawnRules, land: &Land, wpos: Vec2<i32>, weight: f32) {
        self.get().spawn_rules_inner(spawn_rules, land, wpos, weight)
    }

    fn rel_terrain_offset(&self, col: &ColumnSample) -> i32 { self.get().rel_terrain_offset(col) }

    fn terrain_surface_at_inner(
        &self,
        wpos: Vec2<i32>,
        old: Block,
        rng: &mut ChaCha8Rng,
        col: &ColumnSample,
        z_off: i32,
        site: &Site,
    ) -> Option<Block> {
        self.get().terrain_surface_at_inner(wpos, old, rng, col, z_off, site)
    }
}
