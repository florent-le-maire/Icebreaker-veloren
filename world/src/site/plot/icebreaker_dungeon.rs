//! Icebreaker: the dungeon of our generator (`crate::icebreaker::dungeon`),
//! drawn under the terrain. A tunnel climbs from the entrance room up to the
//! surface, at the site's origin.

use super::*;
use crate::{
    Land,
    icebreaker::dungeon::{self, Dungeon, RenderSettings, plan},
    site::namegen::NameGen,
    util::{RandomField, Sampler},
};
use common::terrain::{Block, BlockKind, SpriteKind};
use rand::prelude::*;
use tracing::{error, info};
use vek::*;

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
                // In the middle of the steps: step c / 2 is c / 2 + 1 above the floor.
                s if s == 'H' || plan::stairs_direction(s).is_some() => {
                    (min.xy() + c / 2, min.z + 1 + c / 2)
                },
                _ => (min.xy(), min.z),
            };
            let solid = |b: Option<Block>| b.is_some_and(|b| b.is_solid());
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
                '_' => painter.aabb(self.cell_aabb(cell, 0, h, 0)).clear(),
                'D' => {
                    // A passage through the wall, along the floors on both sides.
                    let along_x = (-1..=1).step_by(2).any(|d| {
                        plan::is_walkable(plan.get(cell.offset(d, 0, 0)))
                    });
                    let dir = if along_x { (1, 0) } else { (0, 1) };
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(floor.clone());
                    for u in 0..c {
                        let slice = self.slice(cell, dir, u, 1);
                        let z = self.cell_min(cell).z;
                        painter
                            .aabb(Aabb {
                                min: slice.min.with_z(z + 1),
                                max: slice.max.with_z(z + h.min(4)),
                            })
                            .clear();
                    }
                },
                symbol if symbol == 'H' || plan::stairs_direction(symbol).is_some() => {
                    painter.aabb(self.cell_aabb(cell, 1, h, 0)).clear();
                    painter.aabb(self.cell_aabb(cell, 0, 1, 0)).fill(floor.clone());
                    let dir = plan::stairs_direction(symbol).unwrap_or_else(|| {
                        // A ladder becomes steps toward the floor around the hole above.
                        plan::DIRECTIONS
                            .into_iter()
                            .find(|(dx, dy)| plan::is_walkable(plan.get(cell.offset(*dx, *dy, 1))))
                            .unwrap_or((1, 0))
                    });
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
                        painter.aabb(self.cell_aabb(cell, 1, h, 1)).fill(rock.clone());
                    }
                },
            }
        }
        self.render_tunnel(land, painter, &rock);

        // Some light here and there on the floors.
        for (cell, symbol) in &plan.cells {
            if *symbol == '.' && RandomField::new(7).get(Vec3::new(cell.x, cell.y, cell.level)) % 6 == 0
            {
                let min = self.cell_min(*cell);
                painter.sprite(
                    (min.xy() + c / 2).with_z(min.z + 1),
                    SpriteKind::FireBowlGround,
                );
            }
        }
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
            let mission = dungeon::Mission {
                seed: rng.random(),
                ..dungeon::Mission::default()
            };
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
