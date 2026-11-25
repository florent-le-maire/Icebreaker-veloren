//! Icebreaker FO.4: the forest (« Forêt », jalon FO), placed on the ground by
//! `crate::icebreaker::foret`. Its middle is at the site's origin; the ground
//! of the site is levelled, and the blocks are those of the grid of
//! `icebreaker-foret` (what the workshop page shows).

use super::*;
use crate::{
    Land,
    icebreaker::{
        dungeon::settings::{SleepSettings, sprite_kind},
        foret::{self, Lieu},
    },
    site::namegen::NameGen,
};
use common::terrain::{Block, BlockKind, SpriteCfg};
use ::icebreaker_foret::grille;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::sync::Arc;
use tracing::{error, info};
use vek::*;

pub struct IcebreakerForet {
    lieu: Arc<Lieu>,
    couleurs: foret::Couleurs,
    /// The ladders: each block, and the side of its trunk.
    echelles: Vec<(Vec3<i32>, Vec2<i32>)>,
    /// Icebreaker FO.9: the lanterns and the lianas.
    decors: Vec<(Vec3<i32>, u8)>,
}

impl IcebreakerForet {
    fn new(lieu: Lieu, couleurs: foret::Couleurs) -> Self {
        let echelles = lieu.echelles();
        let decors = lieu.decors();
        Self {
            lieu: Arc::new(lieu),
            couleurs,
            echelles,
            decors,
        }
    }

    pub fn lieu(&self) -> &Lieu { &self.lieu }

    /// Where players arrive: the clearing at the edge of the forest.
    pub fn entrance(&self) -> Vec2<i32> { self.lieu.arrival().xy() }

    /// Icebreaker FO.5: what the mission counts, as in the cave: every
    /// inhabitant, the chief as leader, the chests. No one sleeps, no rounds.
    pub fn mission_targets(&self) -> MissionTargets {
        let lieu = &self.lieu;
        MissionTargets {
            leader: lieu.inhabitants.iter().find(|i| i.chief).map(|i| (i.pos, "chief".to_string())),
            creatures: lieu.inhabitants.iter().map(|i| i.pos).collect(),
            chests: lieu.chests.clone(),
            sleepers: Vec::new(),
            sleep: SleepSettings {
                wake_radius: 0.0,
                wake_radius_sneaking: 0.0,
                alert_radius: 0.0,
                zzz_interval: 1.0e9,
            },
            rondes: Vec::new(),
            ronde: None,
        }
    }

    /// The forest of `seed` around `origin`. `voulu`: the shape asked for (a
    /// preset of `formes.ron`), `-Profil` of the launcher otherwise.
    fn generate(
        settings: &foret::Settings,
        land: &Land,
        origin: Vec2<i32>,
        seed: u64,
        voulu: Option<&str>,
    ) -> Result<Self, String> {
        let profil = voulu
            .map(str::to_string)
            .or_else(|| std::env::var("ICEBREAKER_FORET_PROFIL").ok().filter(|p| !p.is_empty()));
        let base = origin.with_z(land.get_alt_approx(origin) as i32);
        let lieu = foret::generate(settings, seed, base, profil.as_deref())?;
        // Where to go and look, for who tests it (`/goto`).
        let pont = lieu.foret.ponts.first().map(|p| {
            lieu.origine + Vec3::new((p.a[0] + p.b[0]) / 2.0, (p.a[1] + p.b[1]) / 2.0, p.a[2].max(p.b[2]) + 1.0).as_::<i32>()
        });
        // In front of the door of a hollow trunk, and in a hut.
        let porte = lieu.foret.arbres.iter().find(|t| t.creux > 0).map(|t| {
            let (a, d) = (t.portes[0].0, (t.rayon + t.racines() + 2) as f32);
            lieu.origine + Vec3::new(t.pied[0] + (d * a.cos()) as i32, t.pied[1] + (d * a.sin()) as i32, t.pied[2] + 1)
        });
        let cabane = lieu.foret.cabanes.first().map(|h| lieu.origine + Vec3::from(h.centre));
        // Icebreaker FO.5: the chief, a shooter up on a platform, a chest.
        let chef = lieu.inhabitants.iter().find(|i| i.chief).map(|i| i.pos);
        let guetteur = lieu.inhabitants.iter().map(|i| i.pos).find(|p| lieu.bloc(*p - Vec3::unit_z()) == grille::PLANCHE);
        let coffre = lieu.chests.first();
        info!(centre = ?lieu.origine, arrivee = ?lieu.arrival(), ?pont, ?porte, ?cabane, ?chef, ?guetteur, ?coffre, "Icebreaker forêt: {}", lieu.summary);
        Ok(Self::new(lieu, settings.couleurs.clone()))
    }
}

impl Structure for IcebreakerForet {
    #[cfg(feature = "dyn-lib")]
    #[unsafe(export_name = "as_dyn_structure_icebreaker_foret")]
    fn as_dyn_outer(&self) -> Option<(&dyn Structure, &'static str)> {
        Some((Self::as_dyn_impl(self), "as_dyn_structure_icebreaker_foret"))
    }

    fn spawn_rules_inner(&self, spawn_rules: &mut SpawnRules, _land: &Land, _wpos: Vec2<i32>, weight: f32) {
        // Our own trees, on a ground levelled at the height of the grid.
        spawn_rules.trees = false;
        spawn_rules.waypoints = false;
        spawn_rules.prefer_alt(self.lieu.origine.z as f32, weight);
    }

    fn render_inner(&self, _site: &Site, _land: &Land, painter: &Painter) {
        let c = &self.couleurs;
        let bloc = |kind, (r, g, b): (u8, u8, u8)| Block::new(kind, Rgb::new(r, g, b));
        let bounds = self.lieu.bounds();
        // Three blocks of ground under the forest: the levelled terrain may be
        // a block off.
        const SOUS_SOL: i32 = 3;
        let frame = painter.aabb(Aabb {
            min: bounds.min - Vec3::unit_z() * SOUS_SOL,
            max: bounds.max,
        });

        // The air of the grid is cleared, over the ground.
        let lieu = Arc::clone(&self.lieu);
        frame
            .sample(move |pos| {
                lieu.sol(pos.xy()).is_some_and(|sol| pos.z >= sol) && matches!(lieu.bloc(pos), grille::AIR | grille::ECHELLE | grille::LANTERNE | grille::LIANE)
            })
            .clear();

        // The ground: grass, the trails and the clearings on top, earth under.
        let lieu = Arc::clone(&self.lieu);
        let terre = bloc(BlockKind::Earth, c.sentier);
        let dessus = [
            bloc(BlockKind::Grass, c.herbe),
            bloc(BlockKind::Earth, c.sentier),
            bloc(BlockKind::Grass, c.clairiere),
            bloc(BlockKind::Earth, c.sente),
        ];
        let sols = Arc::clone(&self.lieu);
        frame
            .sample(move |pos| lieu.sol(pos.xy()).is_some_and(|sol| pos.z < sol && pos.z >= sol - SOUS_SOL))
            .fill(Fill::Sampling(Arc::new(move |pos| {
                let sol = sols.sol(pos.xy())?;
                Some(if pos.z == sol - 1 { dessus[usize::from(sols.nature(pos.xy())).min(3)] } else { terre })
            })));

        // The blocks of the grid.
        let lieu = Arc::clone(&self.lieu);
        let blocs = Arc::clone(&self.lieu);
        let (tronc, racine, feuilles, planche, rambarde, buisson) = (
            bloc(BlockKind::Wood, c.tronc),
            bloc(BlockKind::Wood, c.racine),
            bloc(BlockKind::Leaves, c.feuilles),
            bloc(BlockKind::Wood, c.planche),
            bloc(BlockKind::Wood, c.rambarde),
            bloc(BlockKind::Leaves, c.buisson),
        );
        frame
            .sample(move |pos| !matches!(lieu.bloc(pos), grille::AIR | grille::ECHELLE | grille::LANTERNE | grille::LIANE))
            .fill(Fill::Sampling(Arc::new(move |pos| {
                Some(match blocs.bloc(pos) {
                    grille::BOIS => tronc,
                    grille::RACINE => racine,
                    grille::FEUILLES => feuilles,
                    grille::PLANCHE => planche,
                    grille::RAMBARDE => rambarde,
                    grille::BUISSON => buisson,
                    _ => return None,
                })
            })));

        // The ladders: `Ladder` sprites against the trunk (Veloren climbs any
        // wall; the sprites show where, as in the shafts of the dungeon).
        let area = painter.render_aabr();
        for (pos, vers) in &self.echelles {
            if area.contains_point(pos.xy()) {
                let facing = common::util::Dir2::from_vec2(-*vers).sprite_ori();
                painter.rotated_sprite(*pos, SpriteKind::Ladder, facing);
            }
        }
        // Icebreaker FO.9: the lanterns and the lianas.
        for (pos, bloc) in &self.decors {
            if area.contains_point(pos.xy()) {
                painter.sprite(*pos, if *bloc == grille::LANTERNE { SpriteKind::Lantern } else { SpriteKind::Liana });
            }
        }
        // Icebreaker FO.5: the inhabitants and the chests, as in the cave. The
        // painter's area includes its far edge, shared with the next chunk:
        // spawn there only once.
        let lieu = &self.lieu;
        for (i, inhabitant) in lieu.inhabitants.iter().enumerate() {
            let pos = inhabitant.pos;
            if !area.contains_point(pos.xy()) || pos.x == area.max.x || pos.y == area.max.y {
                continue;
            }
            let mut rng = ChaCha8Rng::seed_from_u64(i as u64);
            let at = pos.as_::<f32>() + Vec3::new(0.5, 0.5, 0.0);
            let mut info = crate::icebreaker::races::entity(&inhabitant.asset, lieu.species.as_deref(), at, &mut rng);
            if let Some(loot) = &inhabitant.loot {
                info = info.with_loot_drop(common::lottery::LootSpec::LootTable(loot.clone()));
            }
            painter.spawn(info);
        }
        if let Some(chest) = sprite_kind(&lieu.chest) {
            for (i, pos) in lieu.chests.iter().enumerate() {
                if !area.contains_point(pos.xy()) {
                    continue;
                }
                let loot_table = lieu.butin.as_ref().map(|b| b.coffre(lieu.chest_tiers.get(i).copied().unwrap_or(1)).to_string());
                painter.rotated_sprite_with_cfg(*pos, chest, (i % 4) as u8 * 2, SpriteCfg {
                    loot_table,
                    ..SpriteCfg::default()
                });
            }
        }
    }
}

impl Site {
    /// Icebreaker FO.4: the site of a forest, its middle at `origin`.
    pub fn generate_icebreaker_foret(land: &Land, rng: &mut impl Rng, origin: Vec2<i32>) -> Self {
        let mut rng = reseed(rng);
        let mut site = Site {
            origin,
            name: Some(format!("{} (forêt Icebreaker)", NameGen::location(&mut rng).generate())),
            kind: Some(SiteKind::IcebreakerForet),
            ..Site::default()
        };
        let plot = foret::Settings::load()
            .and_then(|settings| Ok((IcebreakerForet::generate(&settings, land, origin, u64::from(rng.random::<u32>()), None)?, settings)));
        let (plot, settings) = match plot {
            Ok(plot) => plot,
            Err(e) => {
                error!(?e, "Icebreaker forêt not generated");
                return site;
            },
        };
        // The site covers the area kept for the forest, so that one drawn
        // again in its place is drawn whole.
        let half = TerrainChunkSize::RECT_SIZE.map(|e| e as i32) * settings.emprise;
        let tile_size = TILE_SIZE as i32;
        let aabr = Aabr {
            min: (-half).map(|e| e.div_euclid(tile_size)) - 1,
            max: half.map(|e| e.div_euclid(tile_size)) + 2,
        };
        let plot = site.create_plot(Plot {
            kind: PlotKind::IcebreakerForet(Remplacable::new(plot)),
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

impl Remplacable<IcebreakerForet> {
    /// Draws in its place a new forest of `seed`, around the same `origin`.
    /// `profil`: the shape asked for.
    /// Icebreaker DF: `difficulte`, the one of the expedition (none: as the launcher says, or the default one).
    pub fn regenerate(
        &self,
        land: &Land,
        origin: Vec2<i32>,
        seed: u64,
        profil: Option<&str>,
        difficulte: Option<u8>,
    ) -> Result<bool, String> {
        let mut settings = foret::Settings::load()?;
        if difficulte.is_some() {
            settings.difficulte = common::icebreaker_difficulte::niveau(difficulte);
        }
        self.replace(IcebreakerForet::generate(&settings, land, origin, seed, profil)?);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A forest drawn again in its place is another one, of the shape asked for.
    #[test]
    fn a_forest_drawn_again_is_new() {
        let settings = foret::Settings::load().unwrap();
        let land = Land::empty();
        let origin = Vec2::broadcast(64 * 32);
        let slot = Remplacable::new(IcebreakerForet::generate(&settings, &land, origin, 0, None).unwrap());
        let first = slot.get().lieu().summary.clone();
        for preset in settings.formes.as_ref().unwrap().profils.keys() {
            assert!(slot.regenerate(&land, origin, 400, Some(preset)).unwrap());
            let now = slot.get();
            assert_eq!(&now.lieu().profil, preset);
            assert_eq!(now.lieu().origine.xy(), origin);
            assert_ne!(now.lieu().summary, first);
            // A ladder leans on its trunk.
            for (pos, vers) in &now.echelles {
                assert_eq!(now.lieu().bloc(*pos + vers.with_z(0)), grille::BOIS);
            }
            // Icebreaker FO.5: the mission is the chief of the new forest.
            let targets = now.mission_targets();
            let chef = now.lieu().inhabitants.iter().find(|i| i.chief).unwrap();
            assert_eq!(targets.leader, Some((chef.pos, "chief".to_string())));
            assert_eq!(targets.creatures.len(), now.lieu().inhabitants.len());
            assert_eq!(targets.chests, now.lieu().chests);
        }
    }
}
