//! Icebreaker: the cave (« Grotte », jalon K), dug under the terrain by
//! `crate::icebreaker::grotte`. Its mouth is at the site's origin.

use super::*;
use crate::{
    Land,
    icebreaker::grotte::{self, Grotte},
    site::namegen::NameGen,
};
use rand::prelude::*;
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
        spawn_rules.trees &= wpos.distance_squared(self.grotte.mouth.xy()) > 16 * 16;
        spawn_rules.waypoints = false;
    }

    fn render_inner(&self, _site: &Site, _land: &Land, painter: &Painter) {
        // The pockets, then the bores: air in the rock.
        for pocket in &self.grotte.pockets {
            painter.ellipsoid(pocket.aabb()).clear();
        }
        for bore in self.grotte.bores.iter().chain(&self.grotte.dead_ends) {
            painter.line(bore.a, bore.b, bore.radius).clear();
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
        let grotte = grotte::generate(&settings, rng.random(), mouth);
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
