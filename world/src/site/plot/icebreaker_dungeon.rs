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
    generation::EntityInfo,
    terrain::{Block, BlockKind, SpriteCfg, SpriteKind, UnlockKind},
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use tracing::{error, info};
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
        let (r, g, b) = self.render.wall_color;
        let rock = Fill::Brick(BlockKind::Rock, Rgb::new(r, g, b), 12);
        let (r, g, b) = self.render.floor_color;
        let floor = Fill::Brick(BlockKind::Rock, Rgb::new(r, g, b), 8);
        let plan = &self.dungeon.plan;

        // Solid first: every cell of the plan is rock, floors included.
        for (cell, symbol) in &plan.cells {
            if !matches!(symbol, ' ' | '%' | '~') {
                painter.aabb(self.cell_aabb(*cell, 0, h, 0)).fill(rock.clone());
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
                painter.aabb(self.cell_aabb(cell, h, h + 1, 0)).fill(rock.clone());
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
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(floor.clone());
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
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(floor.clone());
                    if let Some(wall) = self.ladder_wall(cell) {
                        self.ladder(painter, cell, wall, 1, h);
                    }
                },
                symbol if plan::stairs_direction(symbol).is_some() => {
                    painter.aabb(self.cell_aabb(cell, 1, h, 0)).clear();
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(floor.clone());
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
                            .fill(floor.clone());
                    }
                },
                symbol => {
                    painter.aabb(self.cell_aabb(cell, 1, h, 0)).clear();
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(floor.clone());
                    if symbol == 'O' {
                        // A 3 × 3 column whatever the cell size (Icebreaker F.1).
                        painter.aabb(self.cell_aabb(cell, 1, h, (c / 2 - 1).max(1))).fill(rock.clone());
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
                    painter.rotated_sprite(pos, kind, common::util::Dir2::from_vec2(-wall).sprite_ori());
                },
                dungeon::DecorPlacement::Center { around } => {
                    let spot = self.spot(decor.cell);
                    painter.sprite(spot, kind);
                    if let Some(chair) = around.as_deref().and_then(dungeon::settings::sprite_kind) {
                        for (dx, dy) in plan::DIRECTIONS {
                            let facing = common::util::Dir2::from_vec2(Vec2::new(-dx, -dy)).sprite_ori();
                            painter.rotated_sprite(spot + Vec3::new(dx, dy, 0), chair, facing);
                        }
                    }
                },
                dungeon::DecorPlacement::Scatter { chance } => {
                    let noise = RandomField::new(13);
                    for x in 1..c - 1 {
                        for y in 1..c - 1 {
                            let pos = (min.xy() + Vec2::new(x, y)).with_z(min.z + 1);
                            if (x != c / 2 || y != c / 2) && noise.chance(pos, *chance) {
                                painter.sprite(pos, kind);
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
                painter.rotated_sprite(center, chest, (noise % 4) as u8 * 2);
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
                painter.spawn(
                    EntityInfo::at(self.spawn_pos(*cell))
                        .with_asset_expect(asset, &mut rng, None),
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
    }
}

/// The chest sprite named in the data (its loot comes from Veloren).
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
        let generated = dungeon::Settings::load().and_then(|settings| {
            let mission = dungeon::Mission::from_env(rng.random())?;
            let dungeon = dungeon::generate(&settings, &mission)?;
            Ok((dungeon, settings))
        });
        let (dungeon, settings) = match generated {
            Ok(generated) => generated,
            Err(e) => {
                error!(?e, "Icebreaker dungeon not generated");
                return site;
            },
        };
        info!("Icebreaker dungeon: {}\n{}", dungeon.summary, dungeon.plan.to_text());
        let plot = IcebreakerDungeon::new(dungeon, settings.render, land, origin);
        for (cell, key) in &plot.dungeon.locks {
            let (keyhole, dir) = (plot.keyhole_pos(*cell), plot.door_dir(*cell));
            info!(?keyhole, ?dir, key = key.item, "Icebreaker dungeon locked door");
        }
        for (cell, key) in &plot.dungeon.keys {
            info!(chest = ?plot.spot(*cell), key = key.item, "Icebreaker dungeon key");
        }
        let lights = &plot.dungeon.lights;
        let torches = lights.iter().filter(|(_, t)| t.is_some()).count();
        info!(torches, braziers = lights.len() - torches, "Icebreaker dungeon lights");
        let bounds = plot.bounds();
        let tile_size = TILE_SIZE as i32;
        let aabr = Aabr {
            min: (bounds.min - origin).map(|e| e.div_euclid(tile_size)) - 1,
            max: (bounds.max - origin).map(|e| e.div_euclid(tile_size)) + 2,
        };
        let plot = site.create_plot(Plot {
            kind: PlotKind::IcebreakerDungeon(plot),
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
