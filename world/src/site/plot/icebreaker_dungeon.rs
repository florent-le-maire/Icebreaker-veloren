//! Icebreaker: the dungeon of our generator (`crate::icebreaker::dungeon`),
//! drawn under the terrain. A tunnel climbs from the entrance room up to the
//! surface, at the site's origin.

use super::*;
use crate::{
    Land,
    icebreaker::dungeon::{self, Dungeon, RenderSettings, plan, populate},
    site::namegen::NameGen,
    util::{RandomField, Sampler},
};
use common::{
    comp::item::ItemDefinitionIdOwned,
    lottery::LootSpec,
    terrain::{Block, SpriteCfg, SpriteKind, TerrainChunkSize, UnlockKind},
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use tracing::{debug, error, info};
use vek::*;

/// See [`IcebreakerDungeon::mission_targets`].
#[derive(Clone, Debug)]
pub struct MissionTargets {
    /// Where the leader appears, and its kind (`chief` or `boss`).
    pub leader: Option<(Vec3<i32>, String)>,
    pub creatures: Vec<Vec3<i32>>,
    pub chests: Vec<Vec3<i32>>,
    /// Where the asleep creatures appear (Icebreaker G.9), and how they wake.
    pub sleepers: Vec<Vec3<i32>>,
    pub sleep: dungeon::settings::SleepSettings,
    /// Icebreaker Y.3: the rounds (stops, then where their members appear),
    /// and how they go.
    pub rondes: Vec<(Vec<Vec3<i32>>, Vec<Vec3<i32>>)>,
    pub ronde: Option<common::icebreaker_races::Ronde>,
}

pub struct IcebreakerDungeon {
    dungeon: Dungeon,
    render: RenderSettings,
    /// World position of the corner of cell (0, 0).
    base: Vec2<i32>,
    /// Altitude of the floor of level 0.
    floor0: i32,
    /// Where the tunnel comes out, on the surface.
    entrance: Vec2<i32>,
}

impl IcebreakerDungeon {
    fn new(dungeon: Dungeon, render: RenderSettings, land: &Land, origin: Vec2<i32>) -> Self {
        let c = render.cell_size;
        let tunnel = &dungeon.tunnel;
        let (dx, dy) = tunnel.dir;
        // The tunnel ends at the site's origin.
        let last = *tunnel.cells.last().expect("a tunnel has cells");
        let mouth_rel = Vec2::new(last.x * c + c / 2, last.y * c + c / 2)
            + Vec2::new(dx, dy) * (c / 2);
        let base = origin - mouth_rel;
        let rise = tunnel.cells.len() as i32 * c - 1;
        let floor0 = land.get_alt_approx(origin) as i32 - rise - last.level * render.level_height;
        Self {
            dungeon,
            render,
            base,
            floor0,
            entrance: origin + Vec2::new(dx, dy) * 3,
        }
    }

    /// Icebreaker F.8: the dungeon of the mission, its tunnel coming out at
    /// `origin`, drawn again with other seeds (`placement` of the settings)
    /// while it comes too close to an edge of the world: the chunks past the
    /// edge are never generated, and such a dungeon could not be finished.
    /// `world`: the size of the world, in chunks; `mission`: the mission of a
    /// seed. Also gives whether the dungeon fits.
    fn generate_inside(
        land: &Land,
        world: Vec2<i32>,
        rng: &mut impl Rng,
        origin: Vec2<i32>,
        mission: impl Fn(u64) -> Result<dungeon::Mission, String>,
    ) -> Result<(Self, bool), String> {
        let settings = dungeon::Settings::load()?;
        let placement = &settings.placement;
        let chunk = TerrainChunkSize::RECT_SIZE.map(|e| e as i32);
        let inside = Aabr {
            min: chunk * placement.edge_margin,
            max: (world - placement.edge_margin) * chunk,
        };
        // Icebreaker I.11: and within the area kept for it.
        let kept = Self::emprise(&settings, origin);
        let fits = |b: Aabr<i32>| {
            b.min.x >= inside.min.x.max(kept.min.x)
                && b.min.y >= inside.min.y.max(kept.min.y)
                && b.max.x <= inside.max.x.min(kept.max.x)
                && b.max.y <= inside.max.y.min(kept.max.y)
        };
        let mut plot = None;
        for attempt in 1..=placement.fit_tries {
            let dungeon = dungeon::generate(&settings, &mission(rng.random())?)?;
            let tried = Self::new(dungeon, settings.render.clone(), land, origin);
            let bounds = tried.bounds();
            if fits(bounds) {
                if attempt > 1 {
                    info!(attempt, "Icebreaker dungeon redrawn to fit inside the world and its kept area");
                }
                return Ok((tried, true));
            }
            debug!(attempt, ?bounds, ?inside, "Icebreaker dungeon past the edge of the world or of its kept area");
            plot = Some(tried);
        }
        error!(
            ?origin,
            tries = placement.fit_tries,
            "Icebreaker dungeon goes past the edge of the world or of its kept area"
        );
        Ok((plot.expect("fit_tries is at least 1"), false))
    }

    /// Icebreaker I.11: the area kept for the dungeons whose tunnel comes out
    /// at `origin`, the first one and those drawn again in its place
    /// (`placement.emprise` chunks on each side).
    fn emprise(settings: &dungeon::Settings, origin: Vec2<i32>) -> Aabr<i32> {
        let half = TerrainChunkSize::RECT_SIZE.map(|e| e as i32) * settings.placement.emprise;
        Aabr {
            min: origin - half,
            max: origin + half,
        }
    }

    /// Where players arrive, on the surface just outside the tunnel.
    pub fn entrance(&self) -> Vec2<i32> { self.entrance }

    pub fn dungeon(&self) -> &Dungeon { &self.dungeon }

    /// What the mission counts (Icebreaker G.7): where each creature appears,
    /// the leader of the dungeon apart (its boss, or else its chief), and where
    /// the chests are, as positions of rtsim's dungeon slots and collected
    /// sprites.
    pub fn mission_targets(&self) -> MissionTargets {
        let faction = &self.dungeon.faction;
        let rank = |kind: &str| ["chief", "boss"].iter().position(|k| *k == kind);
        let mut targets = MissionTargets {
            leader: None,
            creatures: Vec::new(),
            chests: Vec::new(),
            sleepers: Vec::new(),
            sleep: self.dungeon.sleep.clone(),
            rondes: self
                .dungeon
                .rondes
                .iter()
                .map(|r| {
                    let spots = |cells: &[plan::Cell]| cells.iter().map(|c| self.spot(*c)).collect();
                    (spots(&r.route), spots(&r.members))
                })
                .collect(),
            ronde: self
                .dungeon
                .species
                .as_deref()
                .and_then(|s| common::icebreaker_races::IcebreakerRaces::actuel().races.get(s).cloned())
                .and_then(|c| c.ronde),
        };
        for (cell, symbol) in &self.dungeon.plan.cells {
            let spot = self.spot(*cell);
            if *symbol == '$' {
                targets.chests.push(spot);
            } else if let Some(kind) = populate::kind_of(*symbol)
                && faction.creatures.get(kind).is_some_and(|a| !a.is_empty())
            {
                targets.creatures.push(spot);
                if symbol.is_ascii_uppercase() {
                    targets.sleepers.push(spot);
                }
                if rank(kind) > targets.leader.as_ref().and_then(|(_, k)| rank(k)) {
                    targets.leader = Some((spot, kind.to_string()));
                }
            }
        }
        targets
    }

    /// The block in the middle of a cell, just above its floor (where its
    /// creature or chest stands).
    fn spot(&self, cell: plan::Cell) -> Vec3<i32> { self.spawn_pos(cell).map(|e| e.floor() as i32) }

    /// Icebreaker B.15: the loot table of a decor sprite, ours (`butin.ron`,
    /// caisse) instead of Veloren's when it has one (a crate), none otherwise.
    fn decor_loot(&self, kind: SpriteKind) -> Option<String> {
        matches!(kind.default_loot_spec(), Some(Some(LootSpec::LootTable(_))))
            .then(|| self.dungeon.butin.caisse.clone())
    }

    /// A decor sprite (F.6), with our loot table if it holds one (B.15).
    fn decor_sprite(&self, painter: &Painter, pos: Vec3<i32>, kind: SpriteKind, ori: u8) {
        match self.decor_loot(kind) {
            Some(loot_table) => painter.rotated_sprite_with_cfg(pos, kind, ori, SpriteCfg {
                loot_table: Some(loot_table),
                ..SpriteCfg::default()
            }),
            None => painter.rotated_sprite(pos, kind, ori),
        }
    }

    /// The direction of the passage through a door cell.
    fn door_dir(&self, cell: plan::Cell) -> (i32, i32) {
        let plan = &self.dungeon.plan;
        let along_x = (-1..=1).step_by(2).any(|d| plan::is_walkable(plan.get(cell.offset(d, 0, 0))));
        if along_x { (1, 0) } else { (0, 1) }
    }

    /// A light of a cell (G.10): a torch on the wall toward `torch`, near the
    /// ceiling, facing the room; otherwise a brazier in the middle.
    fn light(&self, cell: plan::Cell, torch: Option<(i32, i32)>) -> (Vec3<i32>, SpriteKind, u8) {
        match torch {
            Some((dx, dy)) => {
                let wall = Vec2::new(dx, dy);
                let c = self.render.cell_size;
                let min = self.cell_min(cell);
                // The last block before the wall, two below the ceiling (Icebreaker F.1: any cell size).
                let along = |w: i32| if w > 0 { c - 1 } else if w < 0 { 0 } else { c / 2 };
                let pos = (min.xy() + wall.map(along)).with_z(min.z + self.render.level_height - 2);
                let facing = common::util::Dir2::from_vec2(-wall).sprite_ori_legacy();
                (pos, SpriteKind::WallSconce, facing)
            },
            None => (self.spot(cell), SpriteKind::FireBowlGround, 0),
        }
    }

    /// The wall a ladder cell (`H`) leans on: its first wall neighbour.
    fn ladder_wall(&self, cell: plan::Cell) -> Option<(i32, i32)> {
        let plan = &self.dungeon.plan;
        plan::DIRECTIONS.into_iter().find(|(dx, dy)| plan.get(cell.offset(*dx, *dy, 0)) == '#')
    }

    /// Icebreaker F.2: the ladder of a shaft, `Ladder` sprites against the wall
    /// toward `wall`, from height `from` to `to` (excluded) of `cell`. Veloren
    /// climbs any wall; the sprites show where.
    fn ladder(&self, painter: &Painter, cell: plan::Cell, (dx, dy): (i32, i32), from: i32, to: i32) {
        let c = self.render.cell_size;
        let min = self.cell_min(cell);
        let along = |w: i32| if w > 0 { c - 1 } else if w < 0 { 0 } else { c / 2 };
        let xy = min.xy() + Vec2::new(dx, dy).map(along);
        let facing = common::util::Dir2::from_vec2(Vec2::new(-dx, -dy)).sprite_ori();
        for z in from..to {
            painter.rotated_sprite(xy.with_z(min.z + z), SpriteKind::Ladder, facing);
        }
    }

    /// The keyhole of a locked door, in the middle of its passage.
    fn keyhole_pos(&self, cell: plan::Cell) -> Vec3<i32> {
        self.spot(cell) + Vec3::unit_z() * ((self.render.level_height - 2) / 2)
    }

    /// Icebreaker F.22 : where the object of a special room stands: in the middle of the place its room gives it,
    /// on its floor.
    fn special_center(&self, special: &dungeon::Special) -> Vec3<f32> {
        let (pos, size) = (special.pos, special.size);
        self.cell_min(special.corner).as_::<f32>()
            + Vec3::new(pos.0 as f32 + size.0 as f32 / 2.0, pos.1 as f32 + size.1 as f32 / 2.0, pos.2 as f32 + 1.0)
    }

    /// Icebreaker : the objects of the special rooms (`/aller`): the role of the room, where the object stands,
    /// and a place three blocks in front of it.
    pub fn special_places(&self) -> Vec<(dungeon::rooms::RoomRole, Vec3<f32>, Vec3<f32>)> {
        self.dungeon
            .speciaux
            .iter()
            .map(|special| {
                let center = self.special_center(special);
                let angle = f32::from(special.ori) * std::f32::consts::FRAC_PI_4;
                (special.role, center, center + Vec3::new(angle.cos(), angle.sin(), 0.0) * 3.0)
            })
            .collect()
    }

    /// Where the creature of a marker cell appears.
    pub fn spawn_pos(&self, cell: plan::Cell) -> Vec3<f32> {
        let min = self.cell_min(cell);
        (min.xy() + self.render.cell_size / 2).with_z(min.z + 1).as_::<f32>() + Vec3::new(0.5, 0.5, 0.0)
    }

    /// Checks the drawn blocks (`block_at`) against the plan: every cell one
    /// can reach from the tunnel has a solid floor and room above it. Returns
    /// the problems found.
    pub fn check_blocks(&self, mut block_at: impl FnMut(Vec3<i32>) -> Option<Block>) -> Vec<String> {
        let plan = &self.dungeon.plan;
        let c = self.render.cell_size;
        let top = *self.dungeon.tunnel.cells.last().expect("a tunnel has cells");
        let mut problems = Vec::new();
        for cell in plan.reachable(top) {
            let symbol = plan.get(cell);
            let min = self.cell_min(cell);
            // Where to look in the cell, and the height of what one stands on.
            let (spot, ground) = match symbol {
                '~' | '_' => continue,
                'D' => (min.xy() + c / 2, min.z),
                // In the middle of the steps: the step under the centre block (Icebreaker F.1: even sizes too).
                s if plan::stairs_direction(s).is_some() => {
                    let (dx, dy) = plan::stairs_direction(s).expect("stairs");
                    let u = if dx + dy > 0 { c / 2 } else { c - 1 - c / 2 };
                    (min.xy() + c / 2, min.z + 1 + u)
                },
                _ => (min.xy(), min.z),
            };
            // A locked door opens with its key: it doesn't block the way.
            let solid = |b: Option<Block>| {
                b.is_some_and(|b| {
                    b.is_solid() && !matches!(b.get_sprite(), Some(SpriteKind::KeyDoor | SpriteKind::Keyhole))
                })
            };
            if !solid(block_at(spot.with_z(ground))) {
                problems.push(format!("{cell:?} '{symbol}' : pas de sol en {:?}", spot.with_z(ground)));
            }
            if (1..=2).any(|dz| solid(block_at(spot.with_z(ground + dz)))) {
                problems.push(format!(
                    "{cell:?} '{symbol}' : bouché au-dessus de {:?}",
                    spot.with_z(ground)
                ));
            }
        }
        // The steps of the tunnel, up to the surface.
        let tunnel = &self.dungeon.tunnel;
        let start = self.cell_min(tunnel.cells[0]).z;
        for (k, cell) in tunnel.cells.iter().enumerate() {
            for u in 0..c {
                let step = self.slice(*cell, tunnel.dir, u, 1).center().with_z(start + k as i32 * c + u);
                let solid = |b: Option<Block>| b.is_some_and(|b| b.is_solid());
                if !solid(block_at(step)) || (1..=2).any(|dz| solid(block_at(step + Vec3::unit_z() * dz))) {
                    problems.push(format!("tunnel : marche ratée en {step:?}"));
                }
            }
        }
        // The locked doors and the chests of their keys.
        for (cell, _) in &self.dungeon.locks {
            let pos = self.keyhole_pos(*cell);
            if block_at(pos).and_then(|b| b.get_sprite()) != Some(SpriteKind::Keyhole) {
                problems.push(format!("{cell:?} : pas de serrure en {pos:?}"));
            }
        }
        for (cell, _) in &self.dungeon.keys {
            let pos = self.spot(*cell);
            if block_at(pos).and_then(|b| b.get_sprite()).is_none_or(|s| !s.is_defined_as_container()) {
                problems.push(format!("{cell:?} : pas de coffre à clé en {pos:?}"));
            }
        }
        // The lights (G.10): a torch hangs on a solid wall.
        for (cell, torch) in &self.dungeon.lights {
            let (pos, sprite, _) = self.light(*cell, *torch);
            if let Some((dx, dy)) = torch
                && !block_at(pos + Vec2::new(*dx, *dy).with_z(0)).is_some_and(|b| b.is_solid())
            {
                problems.push(format!("{cell:?} : torche sans mur en {pos:?}"));
            }
            if block_at(pos).and_then(|b| b.get_sprite()) != Some(sprite) {
                problems.push(format!("{cell:?} : pas de lumière en {pos:?}"));
            }
        }
        problems
    }

    /// The blocks covered by the plan (including the tunnel).
    pub fn bounds(&self) -> Aabr<i32> {
        let c = self.render.cell_size;
        let (min_x, max_x, min_y, max_y) = self.dungeon.plan.bounds().unwrap_or_default();
        Aabr {
            min: self.base + Vec2::new(min_x, min_y) * c,
            max: self.base + Vec2::new(max_x + 1, max_y + 1) * c,
        }
    }

    fn cell_min(&self, cell: plan::Cell) -> Vec3<i32> {
        let c = self.render.cell_size;
        (self.base + Vec2::new(cell.x, cell.y) * c)
            .with_z(self.floor0 + cell.level * self.render.level_height)
    }

    /// The blocks of `cell` between heights `from` and `to` (from its floor,
    /// excluded end), shrunk by `inset` on every side.
    fn cell_aabb(&self, cell: plan::Cell, from: i32, to: i32, inset: i32) -> Aabb<i32> {
        let min = self.cell_min(cell);
        let c = self.render.cell_size;
        Aabb {
            min: (min.xy() + inset).with_z(min.z + from),
            max: (min.xy() + c - inset).with_z(min.z + to),
        }
    }

    /// The blocks of `cell` in the slice `u` (0 to cell size - 1) along
    /// direction (dx, dy), `inset` blocks narrower on both sides.
    fn slice(&self, cell: plan::Cell, (dx, dy): (i32, i32), u: i32, inset: i32) -> Aabr<i32> {
        let min = self.cell_min(cell).xy();
        let c = self.render.cell_size;
        let along = if dx + dy > 0 { u } else { c - 1 - u };
        if dx != 0 {
            Aabr {
                min: Vec2::new(min.x + along, min.y + inset),
                max: Vec2::new(min.x + along + 1, min.y + c - inset),
            }
        } else {
            Aabr {
                min: Vec2::new(min.x + inset, min.y + along),
                max: Vec2::new(min.x + c - inset, min.y + along + 1),
            }
        }
    }

    /// Icebreaker F.27 : a surface of `cell` drawn in a material, block by block (its height counted from the
    /// floor of the cell's level).
    fn materiau_fill(&self, cell: plan::Cell, m: &dungeon::materiaux::Materiau) -> Fill {
        let (m, z0, etage) = (m.clone(), self.cell_min(cell).z, self.render.level_height);
        Fill::Sampling(std::sync::Arc::new(move |p| Some(m.bloc(p, p.z - z0, etage))))
    }

    /// The habillage of the room `cell` belongs to, or that of the dungeon.
    fn habillage(&self, cell: plan::Cell) -> &dungeon::materiaux::Habillage {
        let h = &self.dungeon.habillage;
        h.de(cell).unwrap_or(&h.defaut)
    }

    fn sol_fill(&self, cell: plan::Cell) -> Fill {
        let h = &self.dungeon.habillage;
        self.materiau_fill(cell, h.materiaux.sol(self.habillage(cell)))
    }

    fn plafond_fill(&self, cell: plan::Cell) -> Fill {
        let h = &self.dungeon.habillage;
        self.materiau_fill(cell, h.materiaux.plafond(self.habillage(cell)))
    }

    /// Icebreaker F.28 : a wall cell, each block in the walls of the room it is nearest to (a wall between two
    /// rooms shows each room its own side) ; the walls of the dungeon where no room is next to it.
    fn mur_fill(&self, cell: plan::Cell) -> Fill {
        let h = &self.dungeon.habillage;
        if h.de(cell).is_some() {
            return self.materiau_fill(cell, h.materiaux.mur(self.habillage(cell)));
        }
        let cotes: Vec<((i32, i32), dungeon::materiaux::Materiau)> = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .filter_map(|(dx, dy)| h.de(cell.offset(dx, dy, 0)).map(|hab| ((dx, dy), h.materiaux.mur(hab).clone())))
            .collect();
        let defaut = h.materiaux.mur(&h.defaut).clone();
        let (min, c, etage) = (self.cell_min(cell), self.render.cell_size, self.render.level_height);
        Fill::Sampling(std::sync::Arc::new(move |p| {
            let (lx, ly) = (p.x - min.x, p.y - min.y);
            let distance = |d: (i32, i32)| match d {
                (-1, 0) => lx,
                (1, 0) => c - 1 - lx,
                (0, -1) => ly,
                _ => c - 1 - ly,
            };
            let m = cotes.iter().min_by_key(|(d, _)| distance(*d)).map_or(&defaut, |(_, m)| m);
            Some(m.bloc(p, p.z - min.z, etage))
        }))
    }

    fn render_tunnel(&self, land: &Land, painter: &Painter, rock: &Fill) {
        let c = self.render.cell_size;
        let tunnel = &self.dungeon.tunnel;
        let start = self.cell_min(tunnel.cells[0]).z;
        for (k, cell) in tunnel.cells.iter().enumerate() {
            for u in 0..c {
                let t = k as i32 * c + u;
                let floor = start + t;
                let slice = self.slice(*cell, tunnel.dir, u, 1);
                let surface = land.get_alt_approx(slice.center()) as i32;
                // A trench open to the sky where the rock above gets thin.
                let top = if floor + self.render.level_height >= surface {
                    surface.max(floor + self.render.level_height) + 6
                } else {
                    floor + self.render.level_height
                };
                painter
                    .aabb(Aabb {
                        min: slice.min.with_z(start),
                        max: slice.max.with_z(floor + 1),
                    })
                    .fill(rock.clone());
                painter
                    .aabb(Aabb {
                        min: slice.min.with_z(floor + 1),
                        max: slice.max.with_z(top),
                    })
                    .clear();
            }
        }
    }
}

impl Structure for IcebreakerDungeon {
    #[cfg(feature = "dyn-lib")]
    #[unsafe(export_name = "as_dyn_structure_icebreaker_dungeon")]
    fn as_dyn_outer(&self) -> Option<(&dyn Structure, &'static str)> {
        Some((Self::as_dyn_impl(self), "as_dyn_structure_icebreaker_dungeon"))
    }

    fn spawn_rules_inner(
        &self,
        spawn_rules: &mut SpawnRules,
        _land: &Land,
        wpos: Vec2<i32>,
        _weight: f32,
    ) {
        spawn_rules.trees &= wpos.distance_squared(self.entrance) > 24 * 24;
        spawn_rules.waypoints = false;
    }

    fn render_inner(&self, _site: &Site, land: &Land, painter: &Painter) {
        let h = self.render.level_height;
        let c = self.render.cell_size;
        // Icebreaker F.27 : the rock outside the rooms (the tunnel) in the walls of the dungeon.
        let habillage = &self.dungeon.habillage;
        let roche = habillage.materiaux.mur(&habillage.defaut).clone();
        let rock = Fill::Sampling(std::sync::Arc::new(move |p| Some(roche.bloc(p, 3, 7))));
        let plan = &self.dungeon.plan;

        // Solid first: every cell of the plan is rock, floors included.
        for (cell, symbol) in &plan.cells {
            if !matches!(symbol, ' ' | '%' | '~') {
                painter.aabb(self.cell_aabb(*cell, 0, h, 0)).fill(self.mur_fill(*cell));
            }
        }
        // Then the open spaces.
        for (cell, symbol) in &plan.cells {
            let cell = *cell;
            let open = plan::is_walkable(*symbol)
                || *symbol == '_'
                || plan::stairs_direction(*symbol).is_some();
            if !open || *symbol == '~' {
                continue;
            }
            if plan.is_free(cell.offset(0, 0, 1)) {
                // Ceiling where nothing is above.
                painter.aabb(self.cell_aabb(cell, h, h + 1, 0)).fill(self.plafond_fill(cell));
            }
            match *symbol {
                '_' => {
                    painter.aabb(self.cell_aabb(cell, 0, h, 0)).clear();
                    // The ladder of the shaft below goes on up to this floor.
                    let below = cell.offset(0, 0, -1);
                    if plan.get(below) == 'H'
                        && let Some(wall) = self.ladder_wall(below).filter(|(dx, dy)| plan.get(cell.offset(*dx, *dy, 0)) == '#')
                    {
                        self.ladder(painter, cell, wall, 0, h);
                    }
                },
                'D' => {
                    // A passage through the wall, along the floors on both sides.
                    let dir = self.door_dir(cell);
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(self.sol_fill(cell));
                    for u in 0..c {
                        let slice = self.slice(cell, dir, u, 1);
                        let z = self.cell_min(cell).z;
                        painter
                            .aabb(Aabb {
                                min: slice.min.with_z(z + 1),
                                max: slice.max.with_z(z + h - 1),
                            })
                            .clear();
                    }
                },
                'H' => {
                    // Icebreaker F.2: a shaft with a ladder on its wall instead of steps.
                    painter.aabb(self.cell_aabb(cell, 1, h, 0)).clear();
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(self.sol_fill(cell));
                    if let Some(wall) = self.ladder_wall(cell) {
                        self.ladder(painter, cell, wall, 1, h);
                    }
                },
                symbol if plan::stairs_direction(symbol).is_some() => {
                    painter.aabb(self.cell_aabb(cell, 1, h, 0)).clear();
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(self.sol_fill(cell));
                    let dir = plan::stairs_direction(symbol).expect("stairs");
                    // One step per block: the last one is level with the floor above.
                    for u in 0..c {
                        let slice = self.slice(cell, dir, u, 0);
                        let z = self.cell_min(cell).z;
                        painter
                            .aabb(Aabb {
                                min: slice.min.with_z(z + 1),
                                max: slice.max.with_z(z + 2 + u),
                            })
                            .fill(self.sol_fill(cell));
                    }
                },
                symbol => {
                    painter.aabb(self.cell_aabb(cell, 1, h, 0)).clear();
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(self.sol_fill(cell));
                    if symbol == 'O' {
                        // A 3 × 3 column whatever the cell size (Icebreaker F.1).
                        painter.aabb(self.cell_aabb(cell, 1, h, (c / 2 - 1).max(1))).fill(self.mur_fill(cell));
                    }
                },
            }
        }
        self.render_tunnel(land, painter, &rock);

        // Lights by the inhabitants (G.10): torches on the walls, braziers on the floors.
        for (cell, torch) in &self.dungeon.lights {
            let (pos, sprite, ori) = self.light(*cell, *torch);
            painter.rotated_sprite(pos, sprite, ori);
        }

        // The decor by the role of the rooms (Icebreaker F.6).
        for decor in &self.dungeon.decor {
            let Some(kind) = dungeon::settings::sprite_kind(&decor.sprite) else {
                continue;
            };
            let min = self.cell_min(decor.cell);
            let along = |w: i32| if w > 0 { c - 1 } else if w < 0 { 0 } else { c / 2 };
            match &decor.place {
                dungeon::DecorPlacement::Wall((dx, dy)) => {
                    let wall = Vec2::new(*dx, *dy);
                    let pos = (min.xy() + wall.map(along)).with_z(min.z + 1);
                    let ori = common::util::Dir2::from_vec2(-wall).sprite_ori();
                    self.decor_sprite(painter, pos, kind, ori);
                },
                dungeon::DecorPlacement::Center { around } => {
                    let spot = self.spot(decor.cell);
                    self.decor_sprite(painter, spot, kind, 0);
                    if let Some(chair) = around.as_deref().and_then(dungeon::settings::sprite_kind) {
                        for (dx, dy) in plan::DIRECTIONS {
                            let facing = common::util::Dir2::from_vec2(Vec2::new(-dx, -dy)).sprite_ori();
                            painter.rotated_sprite(spot + Vec3::new(dx, dy, 0), chair, facing);
                        }
                    }
                },
                // Icebreaker F.14 : placed by hand in the rooms workshop, block by block.
                dungeon::DecorPlacement::Block { pos, ori, mirror } => {
                    let pos = min + Vec3::new(pos.0, pos.1, pos.2 + 1);
                    let block = Block::empty().with_sprite(kind);
                    let block = block.with_ori(*ori % 8).unwrap_or(block);
                    let block = if *mirror {
                        block.with_attr(common::terrain::sprite::MirrorX(true)).unwrap_or(block)
                    } else {
                        block
                    };
                    let fill = match self.decor_loot(kind) {
                        Some(loot_table) => Fill::CfgSprite(block, SpriteCfg {
                            loot_table: Some(loot_table),
                            ..SpriteCfg::default()
                        }),
                        None => Fill::Sprite(block),
                    };
                    painter.aabb(Aabb { min: pos, max: pos + 1 }).fill(fill);
                },
                dungeon::DecorPlacement::Scatter { chance } => {
                    let noise = RandomField::new(13);
                    for x in 1..c - 1 {
                        for y in 1..c - 1 {
                            let pos = (min.xy() + Vec2::new(x, y)).with_z(min.z + 1);
                            if (x != c / 2 || y != c / 2) && noise.chance(pos, *chance) {
                                self.decor_sprite(painter, pos, kind, 0);
                            }
                        }
                    }
                },
            }
        }

        // The people and the chests of the plan.
        let faction = &self.dungeon.faction;
        let chest = chest_sprite(&faction.chest);
        for (cell, symbol) in &plan.cells {
            let min = self.cell_min(*cell);
            let center = (min.xy() + c / 2).with_z(min.z + 1);
            let noise = RandomField::new(11).get(Vec3::new(cell.x, cell.y, cell.level));
            if *symbol == '$' {
                // Icebreaker B.5: the loot of its tier (butin.ron), not the sprite's.
                let palier = self.dungeon.paliers.get(cell).copied().unwrap_or(1);
                painter.rotated_sprite_with_cfg(center, chest, (noise % 4) as u8 * 2, SpriteCfg {
                    loot_table: Some(self.dungeon.butin.coffre(palier).to_string()),
                    ..SpriteCfg::default()
                });
            } else if let Some(kind) = populate::kind_of(*symbol)
                && let Some(assets) = faction.creatures.get(kind).filter(|a| !a.is_empty())
            {
                // The painter's area includes its far edge, shared with the next
                // chunk: spawn there only once.
                let area = painter.render_aabr();
                if center.x == area.max.x || center.y == area.max.y {
                    continue;
                }
                let asset = &assets[noise as usize % assets.len()];
                let mut rng = ChaCha8Rng::seed_from_u64(noise as u64);
                // Icebreaker B.14: it drops the loot of its tier (butin.ron).
                let palier = self.dungeon.paliers.get(cell).copied().unwrap_or(1);
                let loot = self.dungeon.butin.creature(kind, palier).to_string();
                painter.spawn(
                    // Icebreaker H.2: a humanoid of the race of the dungeon.
                    crate::icebreaker::races::entity(asset, self.dungeon.species.as_deref(), self.spawn_pos(*cell), &mut rng)
                        .with_loot_drop(common::lottery::LootSpec::LootTable(loot)),
                );
            }
        }

        // Icebreaker G.8: a locked door is a wall of key door blocks across its
        // passage, its keyhole in the middle, which consumes the key of this
        // door; the key lies in a chest where the generator put it.
        for (cell, key) in &self.dungeon.locks {
            let dir = self.door_dir(*cell);
            let slice = self.slice(*cell, dir, c / 2, 1);
            let z = self.cell_min(*cell).z;
            painter
                .aabb(Aabb {
                    min: slice.min.with_z(z + 1),
                    max: slice.max.with_z(z + h - 1),
                })
                .fill(Fill::Block(Block::air(SpriteKind::KeyDoor)));
            let unlock = UnlockKind::Consumes(ItemDefinitionIdOwned::Simple(key.item.clone()));
            painter.rotated_sprite_with_cfg(
                self.keyhole_pos(*cell),
                SpriteKind::Keyhole,
                if dir.0 != 0 { 2 } else { 0 },
                SpriteCfg {
                    unlock: Some(unlock),
                    ..SpriteCfg::default()
                },
            );
        }
        for (cell, key) in &self.dungeon.keys {
            painter.rotated_sprite_with_cfg(self.spot(*cell), chest, 0, SpriteCfg {
                loot_table: Some(key.loot.clone()),
                ..SpriteCfg::default()
            });
        }

        // Icebreaker F.22 : the objects of the special rooms (the portal, the altar), in the middle of the place
        // their room gives them, standing on its floor.
        for special in &self.dungeon.speciaux {
            let center = self.special_center(special);
            // The painter's area includes its far edge, shared with the next chunk: spawn there only once.
            let (area, block) = (painter.render_aabr(), center.xy().map(|e| e.floor() as i32));
            if block.x == area.max.x || block.y == area.max.y {
                continue;
            }
            painter.spawn(EntityInfo::at(center).into_special(common::generation::SpecialEntity::IcebreakerObjet {
                body: special_body(special.role),
                ori: special.ori,
            }));
        }
    }
}

/// Icebreaker F.22 : the object a special room holds (the portal starts unlit).
fn special_body(role: dungeon::rooms::RoomRole) -> common::comp::object::Body {
    match role {
        dungeon::rooms::RoomRole::Altar => common::comp::object::Body::IcebreakerAutel,
        _ => common::comp::object::Body::IcebreakerPortail,
    }
}

/// The chest sprite named in the data (its loot comes from `butin.ron`, B.5).
fn chest_sprite(name: &str) -> SpriteKind {
    match name {
        "DungeonChest1" => SpriteKind::DungeonChest1,
        "DungeonChest2" => SpriteKind::DungeonChest2,
        "DungeonChest3" => SpriteKind::DungeonChest3,
        "DungeonChest4" => SpriteKind::DungeonChest4,
        "DungeonChest5" => SpriteKind::DungeonChest5,
        _ => SpriteKind::DungeonChest0,
    }
}

impl Site {
    /// Icebreaker: the site of a dungeon from our generator, its tunnel
    /// coming out at `origin`.
    pub fn generate_icebreaker_dungeon(land: &Land, rng: &mut impl Rng, origin: Vec2<i32>) -> Self {
        let mut rng = reseed(rng);
        let mut site = Site {
            origin,
            name: Some(format!("{} (Icebreaker)", NameGen::location(&mut rng).generate())),
            kind: Some(SiteKind::IcebreakerDungeon),
            ..Site::default()
        };
        let world = land.size().map(|e| e as i32);
        let plot = match IcebreakerDungeon::generate_inside(land, world, &mut rng, origin, dungeon::Mission::from_env) {
            Ok((plot, _)) => plot,
            Err(e) => {
                error!(?e, "Icebreaker dungeon not generated");
                return site;
            },
        };
        plot.log();
        // Icebreaker I.11: the site covers the area kept for the dungeon, so
        // that one drawn again in its place (I.12) is drawn whole.
        let bounds = match dungeon::Settings::load() {
            Ok(settings) => IcebreakerDungeon::emprise(&settings, origin),
            Err(_) => plot.bounds(),
        };
        let tile_size = TILE_SIZE as i32;
        let aabr = Aabr {
            min: (bounds.min - origin).map(|e| e.div_euclid(tile_size)) - 1,
            max: (bounds.max - origin).map(|e| e.div_euclid(tile_size)) + 2,
        };
        let plot = site.create_plot(Plot {
            kind: PlotKind::IcebreakerDungeon(Remplacable::new(plot)),
            root_tile: aabr.center(),
            tiles: aabr_tiles(aabr).collect(),
        });
        site.blit_aabr(aabr, Tile {
            kind: TileKind::Building,
            plot: Some(plot),
            hard_alt: None,
        });
        site
    }
}

impl IcebreakerDungeon {
    /// Tells what was generated.
    fn log(&self) {
        info!("Icebreaker dungeon: {}\n{}", self.dungeon.summary, self.dungeon.plan.to_text());
        // Icebreaker F.10 : where to stand to look at a room (`/goto`) : the free floor of the entrance level
        // nearest to the middle of the plan, and the corner of the plan (a cell is `cell` blocks wide).
        let floor: Vec<plan::Cell> =
            self.dungeon.plan.cells.iter().filter(|(c, s)| c.level == 0 && **s == '.').map(|(c, _)| *c).collect();
        let n = floor.len().max(1) as i32;
        let (mx, my) = (floor.iter().map(|c| c.x).sum::<i32>() / n, floor.iter().map(|c| c.y).sum::<i32>() / n);
        if let Some(middle) = floor.iter().min_by_key(|c| (c.x - mx).pow(2) + (c.y - my).pow(2)) {
            let corner = self.spawn_pos(plan::Cell::new(0, 0, 0));
            info!(middle = ?self.spawn_pos(*middle), ?corner, cell = self.render.cell_size, "Icebreaker dungeon viewpoint");
        }
        for (cell, key) in &self.dungeon.locks {
            let (keyhole, dir) = (self.keyhole_pos(*cell), self.door_dir(*cell));
            info!(?keyhole, ?dir, key = key.item, "Icebreaker dungeon locked door");
        }
        for (cell, key) in &self.dungeon.keys {
            info!(chest = ?self.spot(*cell), key = key.item, "Icebreaker dungeon key");
        }
        for special in &self.dungeon.speciaux {
            info!(role = special.role.name(), at = ?self.spawn_pos(special.cell()), "Icebreaker dungeon special room");
        }
        let lights = &self.dungeon.lights;
        let torches = lights.iter().filter(|(_, t)| t.is_some()).count();
        info!(torches, braziers = lights.len() - torches, "Icebreaker dungeon lights");
    }
}

impl Remplacable<IcebreakerDungeon> {
    /// Icebreaker I.11: draws in its place a new dungeon of `seed`, its
    /// tunnel coming out at the same `origin`, within the area kept for it.
    /// The current one stays if none fits; says whether it was replaced.
    pub fn regenerate(&self, land: &Land, origin: Vec2<i32>, seed: u64) -> Result<bool, String> {
        let world = land.size().map(|e| e as i32);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (plot, fits) =
            IcebreakerDungeon::generate_inside(land, world, &mut rng, origin, dungeon::Mission::from_env)?;
        if fits {
            plot.log();
            self.replace(plot);
        }
        Ok(fits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icebreaker::site_area;

    /// Icebreaker F.8: an entrance placed in a corner of the area that
    /// `site_area` allows still gets a dungeon that stays inside the world, for
    /// every size, in the world of the campaign (2^7 chunks).
    #[test]
    fn dungeon_in_a_corner_stays_inside_the_world() {
        let world = Vec2::broadcast(128);
        let area = site_area(SiteKind::IcebreakerDungeon, Aabr { min: Vec2::zero(), max: world });
        let chunk = TerrainChunkSize::RECT_SIZE.map(|e| e as i32);
        for corner in [area.min, area.max, Vec2::new(area.min.x, area.max.y)] {
            let origin = corner * chunk + chunk / 2;
            for size in ["small", "medium", "large"] {
                for seed in 0..10 {
                    let mut rng = ChaCha8Rng::seed_from_u64(seed);
                    let mission = |seed| {
                        Ok(dungeon::Mission {
                            seed,
                            size: size.into(),
                            ..dungeon::Mission::default()
                        })
                    };
                    let (plot, fits) =
                        IcebreakerDungeon::generate_inside(&Land::empty(), world, &mut rng, origin, mission)
                            .unwrap();
                    assert!(fits, "{size} {seed} {origin} : donjon hors du monde {:?}", plot.bounds());
                }
            }
        }
    }

    /// Too close to the edge, no dungeon fits: it is said.
    #[test]
    fn dungeon_at_the_edge_does_not_fit() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let mission = |seed| Ok(dungeon::Mission { seed, size: "large".into(), ..dungeon::Mission::default() });
        let (_, fits) =
            IcebreakerDungeon::generate_inside(&Land::empty(), Vec2::broadcast(8), &mut rng, Vec2::new(40, 40), mission)
                .unwrap();
        assert!(!fits);
    }

    /// Icebreaker B.15: a crate of the decor holds our table, not Veloren's;
    /// the decor without loot stays without.
    #[test]
    fn decor_crates_hold_our_loot() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let mission = |seed| Ok(dungeon::Mission { seed, ..dungeon::Mission::default() });
        let (plot, _) =
            IcebreakerDungeon::generate_inside(&Land::empty(), Vec2::broadcast(8), &mut rng, Vec2::new(40, 40), mission)
                .unwrap();
        let caisse = Some("common.loot_tables.icebreaker.butin.caisse");
        assert_eq!(plot.decor_loot(SpriteKind::Crate).as_deref(), caisse);
        for kind in [SpriteKind::CrateBlock, SpriteKind::Barrel, SpriteKind::SackLeatherM] {
            assert_eq!(plot.decor_loot(kind), None, "{kind:?}");
        }
    }

    #[test]
    fn only_our_dungeon_keeps_away_from_the_edges() {
        let world = Aabr { min: Vec2::zero(), max: Vec2::broadcast(128) };
        let area = site_area(SiteKind::IcebreakerDungeon, world);
        assert!(area.min.x > 0 && area.max.y < 128);
        assert_eq!(site_area(SiteKind::Gnarling, world), world);
        // A world too small for the margin is kept whole.
        let tiny = Aabr { min: Vec2::zero(), max: Vec2::broadcast(4) };
        assert_eq!(site_area(SiteKind::IcebreakerDungeon, tiny), tiny);
    }

    /// Icebreaker I.11: a dungeon drawn again in its place is another one,
    /// its tunnel at the same entrance, within the area kept for it.
    #[test]
    fn a_dungeon_drawn_again_is_new_and_stays_in_its_area() {
        let settings = dungeon::Settings::load().unwrap();
        let land = Land::empty();
        let origin = Vec2::broadcast(64 * 32);
        let mission = |seed| Ok(dungeon::Mission { seed, ..dungeon::Mission::default() });
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let (first, fits) =
            IcebreakerDungeon::generate_inside(&land, Vec2::broadcast(128), &mut rng, origin, mission).unwrap();
        assert!(fits);
        let kept = IcebreakerDungeon::emprise(&settings, origin);
        let slot = Remplacable::new(first);
        let mut plans = vec![slot.get().dungeon.plan.to_text()];
        for seed in 1..6 {
            let (plot, fits) = IcebreakerDungeon::generate_inside(
                &land,
                Vec2::broadcast(128),
                &mut ChaCha8Rng::seed_from_u64(seed),
                origin,
                mission,
            )
            .unwrap();
            assert!(fits, "graine {seed}");
            slot.replace(plot);
            let now = slot.get();
            let b = now.bounds();
            assert!(kept.contains_point(b.min) && kept.contains_point(b.max), "graine {seed} : {b:?} hors de {kept:?}");
            assert!(now.entrance().distance_squared(origin) < 32 * 32, "graine {seed} : entrée loin du site");
            plans.push(now.dungeon.plan.to_text());
        }
        plans.sort();
        plans.dedup();
        assert_eq!(plans.len(), 6, "deux plans identiques");
    }
}
