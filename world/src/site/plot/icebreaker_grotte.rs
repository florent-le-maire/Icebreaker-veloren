//! Icebreaker: the cave (« Grotte », jalon K), dug under the terrain by
//! `crate::icebreaker::grotte`. Its mouth is at the site's origin.

use super::*;
use crate::{
    Land,
    icebreaker::{
        dungeon::settings::{SleepSettings, sprite_kind},
        grotte::{self, Grotte, SpeleothemKind, bump},
    },
    site::namegen::NameGen,
    util::{FastNoise, RandomField, Sampler},
};
use common::{
    generation::EntityInfo,
    terrain::{Block, BlockKind, SpriteCfg},
};
use rand_chacha::ChaCha8Rng;
use rand::prelude::*;
use std::sync::Arc;
use tracing::{error, info};
use vek::*;

pub struct IcebreakerGrotte {
    grotte: Grotte,
    /// Where players arrive, on the surface in front of the mouth.
    entrance: Vec2<i32>,
}

impl IcebreakerGrotte {
    fn new(grotte: Grotte) -> Self {
        let entrance = grotte.mouth.xy() - grotte.dir * 4;
        Self { grotte, entrance }
    }

    pub fn entrance(&self) -> Vec2<i32> { self.entrance }

    pub fn grotte(&self) -> &Grotte { &self.grotte }

    /// The blocks of the cave, on the map.
    pub fn bounds(&self) -> Aabr<i32> {
        let bounds = self.grotte.bounds();
        Aabr {
            min: bounds.min.xy() - 4,
            max: bounds.max.xy() + 4,
        }
    }

    /// What the mission counts (Icebreaker K.5): every inhabitant, the chief
    /// as leader, the chests. No one sleeps in the cave.
    pub fn mission_targets(&self) -> MissionTargets {
        let grotte = &self.grotte;
        MissionTargets {
            leader: grotte.inhabitants.iter().find(|i| i.chief).map(|i| (i.pos, "chief".to_string())),
            creatures: grotte.inhabitants.iter().map(|i| i.pos).collect(),
            chests: grotte.chests.clone(),
            sleepers: Vec::new(),
            sleep: SleepSettings {
                wake_radius: 0.0,
                wake_radius_sneaking: 0.0,
                alert_radius: 0.0,
                zzz_interval: 1.0e9,
            },
        }
    }
}

/// Sprites scattered with a chance per block, each with its own draw.
fn scatter(sprites: &[(String, f32)], salt: u32) -> Fill {
    let sprites: Vec<(SpriteKind, f32)> = sprites
        .iter()
        .filter_map(|(name, chance)| Some((sprite_kind(name)?, *chance)))
        .collect();
    Fill::Sampling(Arc::new(move |pos| {
        sprites
            .iter()
            .enumerate()
            .find(|(i, (_, chance))| RandomField::new(salt + *i as u32).chance(pos, *chance))
            .map(|(_, (sprite, _))| Block::air(*sprite))
    }))
}

impl Structure for IcebreakerGrotte {
    #[cfg(feature = "dyn-lib")]
    #[unsafe(export_name = "as_dyn_structure_icebreaker_grotte")]
    fn as_dyn_outer(&self) -> Option<(&dyn Structure, &'static str)> {
        Some((Self::as_dyn_impl(self), "as_dyn_structure_icebreaker_grotte"))
    }

    fn spawn_rules_inner(
        &self,
        spawn_rules: &mut SpawnRules,
        _land: &Land,
        wpos: Vec2<i32>,
        _weight: f32,
    ) {
        // No tree over the cave: the entrance may run in the open for a while.
        spawn_rules.trees &= !self.bounds().contains_point(wpos);
        spawn_rules.waypoints = false;
    }

    fn render_inner(&self, _site: &Site, _land: &Land, painter: &Painter) {
        let grotte = &self.grotte;
        let look = &grotte.look;
        let (bosses, rugosite) = (grotte.relief.bosses, grotte.relief.rugosite);
        let (r, g, b) = look.roche;
        let rock = Block::new(BlockKind::Rock, Rgb::new(r, g, b));
        let (r, g, b) = look.lueur;
        let glow = Block::new(BlockKind::GlowingRock, Rgb::new(r, g, b));
        let glow_chance = look.lueur_chance;

        // The pockets (Icebreaker K.3): an ellipsoid whose walls wander with
        // the noise, a rubble floor with bumps and glowing rock, sprites on the
        // floor and under the ceiling.
        for pocket in &grotte.pockets {
            let (center, radius) = (pocket.center, pocket.radius);
            let margin = (radius.x.max(radius.y) as f32 * rugosite).ceil() as i32 + 1;
            // The noise is in -1..1: the walls wander by `rugosite` of the radius.
            let inside = move |pos: Vec3<i32>| {
                let d = (pos - center).as_::<f32>() / radius.as_::<f32>();
                let noise = FastNoise::new(913).get(pos.as_::<f64>() / 5.0);
                d.magnitude() < 1.0 + rugosite * noise
            };
            let frame = Aabb {
                min: center - radius - margin,
                max: center + radius + margin,
            };
            // The dome of the pocket, and a flat floor under it, 0.92 of its
            // radius: the bores arrive on it (at 0.86), whatever the noise does.
            let dome = painter.ellipsoid(frame).sample(inside);
            let wide = (radius.xy().as_::<f32>() * 0.92).map(|e| e.round() as i32);
            let floor_disc = painter
                .cylinder(Aabb {
                    min: (center.xy() - wide).with_z(center.z - radius.z),
                    max: (center.xy() + wide).with_z(center.z + 1),
                });
            let cavity = dome.union(floor_disc);
            cavity.clear();
            let floor_z = move |xy: Vec2<i32>| center.z - radius.z + bump(xy, bosses);
            cavity
                .sample(move |pos| pos.z <= floor_z(pos.xy()))
                .fill(Fill::Sampling(Arc::new(move |pos| {
                    Some(if RandomField::new(917).chance(pos, glow_chance) { glow } else { rock })
                })));
            cavity
                .sample(move |pos| pos.z == floor_z(pos.xy()) + 1)
                .fill(scatter(&look.sol, 920));
            // The ores (Icebreaker K.4): at the foot of the walls, by depth.
            let (veins, level, bord) = (grotte.filons.clone(), pocket.level, grotte.filons.bord);
            let rim = move |xy: Vec2<i32>| ((xy - center.xy()).as_::<f32>() / radius.xy().as_::<f32>()).magnitude_squared();
            cavity
                .sample(move |pos| pos.z == floor_z(pos.xy()) + 1 && rim(pos.xy()) >= bord)
                .fill(Fill::Sampling(Arc::new(move |pos| veins.ore(level, pos).map(Block::air))));
            cavity
                .sample(move |pos| pos.z > center.z && !inside(pos + Vec3::unit_z()))
                .fill(scatter(&look.plafond, 940));
        }
        // The concretions: tapered spikes of rock, or of glowing rock (before
        // the bores, which cut through them).
        for s in &grotte.speleothems {
            let (tip, fill) = match s.kind {
                SpeleothemKind::Stalagmite => (s.base + Vec3::unit_z() * s.height, Fill::Block(rock)),
                SpeleothemKind::Stalactite => (s.base - Vec3::unit_z() * s.height, Fill::Block(rock)),
                SpeleothemKind::Crystal => (s.base + Vec3::unit_z() * s.height, Fill::Block(glow)),
            };
            painter.line_two_radius(s.base, tip, 2.0, 0.5).fill(fill);
        }
        // The bores: a tube, its walls and ceiling roughened by the noise; its
        // floor follows the axis one block at a time (one walks it back up),
        // except inside the pockets, where their floor rules.
        let pockets: Vec<Aabb<i32>> = grotte
            .pockets
            .iter()
            .map(|p| Aabb {
                min: p.aabb().min - 1,
                max: p.aabb().max + 1,
            })
            .collect();
        let pockets = Arc::new(pockets);
        for bore in grotte.bores.iter().chain(&grotte.dead_ends) {
            painter.line(bore.a, bore.b, bore.radius).clear();
            let (a, ab, r) = (bore.a.as_::<f32>(), (bore.b - bore.a).as_::<f32>(), bore.radius);
            let axis_z = move |pos: Vec3<i32>| {
                let t = ((pos.as_::<f32>() - a).dot(ab) / ab.magnitude_squared().max(1.0)).clamp(0.0, 1.0);
                a.z + t * ab.z
            };
            painter
                .line(bore.a, bore.b, bore.radius + 1.5)
                .sample(move |pos| {
                    pos.z as f32 >= axis_z(pos) && FastNoise::new(919).get(pos.as_::<f64>() / 4.0) > 0.6
                })
                .clear();
            let pockets = Arc::clone(&pockets);
            painter
                .line(bore.a, bore.b, bore.radius)
                .sample(move |pos| {
                    pos.z <= axis_z(pos).round() as i32 - r as i32 && !pockets.iter().any(|p| p.contains_point(pos))
                })
                .fill(Fill::Block(rock));
        }
        // The inhabitants and the chests (Icebreaker K.5). The painter's area
        // includes its far edge, shared with the next chunk: spawn there only
        // once.
        let area = painter.render_aabr();
        for (i, inhabitant) in grotte.inhabitants.iter().enumerate() {
            let pos = inhabitant.pos;
            if pos.x == area.max.x || pos.y == area.max.y {
                continue;
            }
            let mut rng = ChaCha8Rng::seed_from_u64(i as u64);
            painter.spawn(
                EntityInfo::at(pos.as_::<f32>() + Vec3::new(0.5, 0.5, 0.0)).with_asset_expect(&inhabitant.asset, &mut rng, None),
            );
        }
        if let Some(chest) = sprite_kind(&grotte.chest) {
            for (i, pos) in grotte.chests.iter().enumerate() {
                // Icebreaker B.5: the loot of its tier (butin.ron), not the sprite's.
                let loot_table = grotte
                    .butin
                    .as_ref()
                    .map(|b| b.coffre(grotte.chest_tiers.get(i).copied().unwrap_or(1)).to_string());
                painter.rotated_sprite_with_cfg(*pos, chest, (i % 4) as u8 * 2, SpriteCfg {
                    loot_table,
                    ..SpriteCfg::default()
                });
            }
        }
    }
}

impl Site {
    /// Icebreaker: the site of a cave, its mouth at `origin`.
    pub fn generate_icebreaker_grotte(land: &Land, rng: &mut impl Rng, origin: Vec2<i32>) -> Self {
        let mut rng = reseed(rng);
        let mut site = Site {
            origin,
            name: Some(format!("{} (grotte Icebreaker)", NameGen::location(&mut rng).generate())),
            kind: Some(SiteKind::IcebreakerGrotte),
            ..Site::default()
        };
        let settings = match grotte::Settings::load() {
            Ok(settings) => settings,
            Err(e) => {
                error!(?e, "Icebreaker grotte not generated");
                return site;
            },
        };
        let mouth = origin.with_z(land.get_alt_approx(origin) as i32);
        // The mood asked for (`-Ambiance` of the launcher), or one drawn by the seed.
        let ambiance = std::env::var("ICEBREAKER_GROTTE_AMBIANCE").ok().filter(|a| !a.is_empty());
        let grotte = grotte::generate(&settings, rng.random(), mouth, ambiance.as_deref(), &|xy| land.get_alt_approx(xy) as i32);
        info!("Icebreaker grotte: {}", grotte.summary);
        let plot = IcebreakerGrotte::new(grotte);
        let bounds = plot.bounds();
        let tile_size = TILE_SIZE as i32;
        let aabr = Aabr {
            min: (bounds.min - origin).map(|e| e.div_euclid(tile_size)) - 1,
            max: (bounds.max - origin).map(|e| e.div_euclid(tile_size)) + 2,
        };
        let plot = site.create_plot(Plot {
            kind: PlotKind::IcebreakerGrotte(plot),
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
