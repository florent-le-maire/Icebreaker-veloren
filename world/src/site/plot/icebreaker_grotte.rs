//! Icebreaker: the cave (« Grotte », jalon K), dug under the terrain by
//! `crate::icebreaker::grotte`. Its mouth is at the site's origin.

use super::*;
use crate::{
    Land,
    icebreaker::{
        dungeon::settings::{SleepSettings, sprite_kind},
        grotte::{self, Grotte},
    },
    site::namegen::NameGen,
    util::RandomField,
};
use common::terrain::{Block, BlockKind, SpriteCfg};
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
        let entrance = grotte.arrival.xy();
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
            // Icebreaker Y.3: no rounds in the cave (no rooms).
            rondes: Vec::new(),
            ronde: None,
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
        _wpos: Vec2<i32>,
        _weight: f32,
    ) {
        spawn_rules.waypoints = false;
    }

    fn render_inner(&self, _site: &Site, _land: &Land, painter: &Painter) {
        let grotte = Arc::new(self.grotte.clone());
        let look = &grotte.look;
        let (r, g, b) = look.roche;
        let rock = Block::new(BlockKind::Rock, Rgb::new(r, g, b));
        let (r, g, b) = look.lueur;
        let glow = Block::new(BlockKind::GlowingRock, Rgb::new(r, g, b));
        let glow_chance = look.lueur_chance;
        let up = Vec3::unit_z();

        // Icebreaker U.4: the shape is a grid of blocks (`icebreaker-caverne`):
        // its air is cleared, and the rock that touches it takes the colours
        // of the mood, with glowing rock here and there. Above the mouth the
        // ground stays as it is: the entrance opens in it.
        let frame = painter.aabb(grotte.bounds());
        let cave = Arc::clone(&grotte);
        let air = move |pos: Vec3<i32>| cave.air(pos);
        frame.sample(air.clone()).clear();
        let a = air.clone();
        frame
            .sample(move |pos| {
                !a(pos)
                    && (a(pos + up)
                        || a(pos - up)
                        || a(pos + Vec3::unit_x())
                        || a(pos - Vec3::unit_x())
                        || a(pos + Vec3::unit_y())
                        || a(pos - Vec3::unit_y()))
            })
            .fill(Fill::Sampling(Arc::new(move |pos| {
                Some(if RandomField::new(917).chance(pos, glow_chance) { glow } else { rock })
            })));
        // Sprites on the floors and under the ceilings.
        let a = air.clone();
        frame.sample(move |pos| a(pos) && !a(pos - up)).fill(scatter(&look.sol, 920));
        let a = air.clone();
        frame.sample(move |pos| a(pos) && !a(pos + up)).fill(scatter(&look.plafond, 940));
        // The ores (Icebreaker K.4): on the floor at the foot of a wall, off
        // the trail, by the level of the zone.
        let (a, cave) = (air.clone(), Arc::clone(&grotte));
        let ores = Arc::clone(&grotte);
        frame
            .sample(move |pos| {
                let p = pos - cave.mouth;
                a(pos)
                    && !a(pos - up)
                    && cave.filons.strikes(pos)
                    && !cave.in_safe_room(pos)
                    && !cave.caverne.grille.sentier(p.x, p.y, p.z - 1)
                    && !(a(pos + Vec3::unit_x()) && a(pos - Vec3::unit_x()) && a(pos + Vec3::unit_y()) && a(pos - Vec3::unit_y()))
            })
            .fill(Fill::Sampling(Arc::new(move |pos| ores.filons.ore(ores.level_at(pos), pos).map(Block::air))));
        // The veins (Icebreaker U.14): one ore, block after block along a wall.
        let (veins, ores) = (Arc::clone(&grotte.vein_blocks), Arc::clone(&grotte.vein_blocks));
        frame
            .sample(move |pos| veins.contains_key(&pos))
            .fill(Fill::Sampling(Arc::new(move |pos| ores.get(&pos).copied().map(Block::air))));
        // The crystals: tapered spikes of glowing rock.
        for (base, height) in &grotte.crystals {
            painter.line_two_radius(*base, *base + up * *height, 2.0, 0.5).fill(Fill::Block(glow));
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
            // Icebreaker H.4: a humanoid of the race of the cave.
            let at = pos.as_::<f32>() + Vec3::new(0.5, 0.5, 0.0);
            let mut info = crate::icebreaker::races::entity(&inhabitant.asset, grotte.species.as_deref(), at, &mut rng);
            // Icebreaker B.14: it drops the loot of its tier (butin.ron).
            if let Some(loot) = &inhabitant.loot {
                info = info.with_loot_drop(common::lottery::LootSpec::LootTable(loot.clone()));
            }
            painter.spawn(info);
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
        let (plot, _) = IcebreakerGrotte::generate_inside(&settings, land, origin, u64::from(rng.random::<u32>()), None);
        // Icebreaker I.11: the site covers the area kept for the cave, so that
        // one drawn again in its place (I.12) is drawn whole.
        let bounds = IcebreakerGrotte::emprise(&settings, origin);
        let tile_size = TILE_SIZE as i32;
        let aabr = Aabr {
            min: (bounds.min - origin).map(|e| e.div_euclid(tile_size)) - 1,
            max: (bounds.max - origin).map(|e| e.div_euclid(tile_size)) + 2,
        };
        let plot = site.create_plot(Plot {
            kind: PlotKind::IcebreakerGrotte(Remplacable::new(plot)),
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

/// Seeds tried in turn for a cave that stays within the area kept for it.
const FIT_TRIES: u64 = 10;

impl IcebreakerGrotte {
    /// Icebreaker I.11: the area kept for the caves whose mouth is at
    /// `origin`, the first one and those drawn again in its place (`emprise`
    /// chunks on each side).
    fn emprise(settings: &grotte::Settings, origin: Vec2<i32>) -> Aabr<i32> {
        let half = TerrainChunkSize::RECT_SIZE.map(|e| e as i32) * settings.emprise;
        Aabr {
            min: origin - half,
            max: origin + half,
        }
    }

    /// The cave of `seed` whose mouth is at `origin`, drawn again with the
    /// next seeds while it goes past the area kept for it; and whether it fits.
    /// Icebreaker U.11: `voulu` is the shape the party chose (a preset of
    /// `formes.ron`), if any.
    fn generate_inside(
        settings: &grotte::Settings,
        land: &Land,
        origin: Vec2<i32>,
        seed: u64,
        voulu: Option<&str>,
    ) -> (Self, bool) {
        let mouth = origin.with_z(land.get_alt_approx(origin) as i32);
        // The mood and the shape asked for (`-Ambiance` and `-Profil` of the launcher), or drawn by the seed.
        let ambiance = std::env::var("ICEBREAKER_GROTTE_AMBIANCE").ok().filter(|a| !a.is_empty());
        let profil = voulu
            .map(str::to_string)
            .or_else(|| std::env::var("ICEBREAKER_GROTTE_PROFIL").ok().filter(|p| !p.is_empty()));
        // Icebreaker H.4: the race asked for (`-Race`), or one drawn by the seed.
        let race = std::env::var("ICEBREAKER_DONJON_RACE").ok().filter(|r| !r.is_empty());
        let kept = Self::emprise(settings, origin);
        let mut plot = None;
        for seed in seed..seed + FIT_TRIES {
            let surface = |xy| land.get_alt_approx(xy) as i32;
            let mut grotte = grotte::generate(settings, seed, mouth, ambiance.as_deref(), profil.as_deref(), &surface);
            if let Some(race) = &race {
                let drawn = format!("race {}", grotte.species.as_deref().unwrap_or("des fichiers"));
                grotte.summary = grotte.summary.replacen(&drawn, &format!("race {race}"), 1);
                grotte.species = Some(race.clone());
            }
            let tried = Self::new(grotte);
            let b = tried.bounds();
            if kept.contains_point(b.min) && kept.contains_point(b.max) {
                info!("Icebreaker grotte: {}", tried.grotte.summary);
                return (tried, true);
            }
            plot = Some(tried);
        }
        error!(?origin, "Icebreaker grotte goes past the area kept for it");
        (plot.expect("FIT_TRIES is at least 1"), false)
    }
}

impl Remplacable<IcebreakerGrotte> {
    /// Icebreaker I.11: draws in its place a new cave of `seed`, its mouth at
    /// the same `origin`, within the area kept for it. The current one stays
    /// if none fits; says whether it was replaced. `profil`: the shape asked for.
    pub fn regenerate(&self, land: &Land, origin: Vec2<i32>, seed: u64, profil: Option<&str>) -> Result<bool, String> {
        let settings = grotte::Settings::load()?;
        let (plot, fits) = IcebreakerGrotte::generate_inside(&settings, land, origin, seed, profil);
        if fits {
            self.replace(plot);
        }
        Ok(fits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Icebreaker I.11: a cave drawn again in its place is another one, its
    /// mouth at the same place, within the area kept for it.
    #[test]
    fn a_cave_drawn_again_is_new_and_stays_in_its_area() {
        let settings = grotte::Settings::load().unwrap();
        let land = Land::empty();
        let origin = Vec2::broadcast(64 * 32);
        let kept = IcebreakerGrotte::emprise(&settings, origin);
        let (first, fits) = IcebreakerGrotte::generate_inside(&settings, &land, origin, 0, None);
        assert!(fits);
        let slot = Remplacable::new(first);
        let mut caves = vec![slot.get().grotte.summary.clone()];
        for seed in [100, 200, 300] {
            assert!(slot.regenerate(&land, origin, seed, None).unwrap(), "graine {seed}");
            let now = slot.get();
            let b = now.bounds();
            assert!(kept.contains_point(b.min) && kept.contains_point(b.max), "graine {seed} : {b:?} hors de {kept:?}");
            assert_eq!(now.grotte.mouth.xy(), origin);
            caves.push(now.grotte.summary.clone());
        }
        caves.sort();
        caves.dedup();
        assert_eq!(caves.len(), 4, "deux grottes identiques");
        // Icebreaker U.11: the shape asked for, and the seed kept to show it.
        for preset in crate::icebreaker::grotte_presets() {
            assert!(slot.regenerate(&land, origin, 400, Some(&preset)).unwrap(), "{preset}");
            let now = slot.get();
            assert_eq!(now.grotte.profil, preset);
            assert!((400..400 + FIT_TRIES).contains(&now.grotte.seed));
            assert!(now.grotte.summary.contains(&format!("graine {}", now.grotte.seed)), "{}", now.grotte.summary);
        }
    }
}
