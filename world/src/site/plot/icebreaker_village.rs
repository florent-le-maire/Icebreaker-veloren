//! Icebreaker VF : le village fixe (décision 0022), construit d'après
//! `assets/icebreaker/village_plan.ron` au lieu de la ville tirée au hasard de
//! Veloren. Tout le village est à une seule altitude ; autour, une bordure de
//! raccord passe en pente douce à la hauteur du pays.

use super::*;
use crate::{
    Land,
    icebreaker::village_plan::{Forme, VillagePlan},
    site::namegen::NameGen,
};
use common::calendar::Calendar;
use rand::prelude::*;
use tracing::{error, info};
use vek::*;

/// Le terrain du village : plat dans son emprise, en pente douce dans la
/// bordure. Rien n'y est dessiné, il ne fait que régler la hauteur du sol.
pub struct IcebreakerTerrain {
    /// L'emprise du village (blocs, `max` exclu), à l'altitude `alt`.
    emprise: Aabr<i32>,
    alt: i32,
    /// La largeur de la bordure (blocs).
    marge: i32,
}

impl IcebreakerTerrain {
    /// Combien le sol de `wpos` suit l'altitude du village : 1 dans
    /// l'emprise, puis jusqu'à 0 au bout de la bordure.
    fn poids(&self, wpos: Vec2<i32>) -> f32 {
        let dist = self.emprise.as_::<f32>().distance_to_point(wpos.as_());
        if self.marge <= 0 {
            return if dist <= 0.0 { 1.0 } else { 0.0 };
        }
        let t = (dist / self.marge as f32).clamp(0.0, 1.0);
        1.0 - t * t * (3.0 - 2.0 * t)
    }
}

impl Structure for IcebreakerTerrain {
    fn spawn_rules_inner(&self, spawn_rules: &mut SpawnRules, _land: &Land, wpos: Vec2<i32>, weight: f32) {
        let poids = self.poids(wpos);
        if poids >= 1.0 {
            spawn_rules.prefer_alt(self.alt as f32, weight + weight.powi(8) * 25.0);
        } else {
            spawn_rules.prefer_alt(self.alt as f32, weight * poids);
        }
    }
}

impl Site {
    pub fn generate_icebreaker_village(
        land: &Land,
        index: IndexRef,
        rng: &mut impl Rng,
        origin: Vec2<i32>,
        calendar: Option<&Calendar>,
    ) -> Self {
        let mut rng = reseed(rng);
        // Sans sorte pendant qu'on pose la place et les chemins : la place de
        // Veloren n'y met alors pas de décor (posé à la hauteur d'origine du
        // terrain, il flotterait ou s'enterrerait sur le sol aplani).
        let mut site = Site {
            origin,
            name: Some(NameGen::location(&mut rng).generate_town()),
            kind: None,
            ..Site::default()
        };
        let plan = match VillagePlan::load() {
            Ok(plan) => plan,
            Err(e) => {
                error!(?e, "Icebreaker : plan du village illisible, village vide");
                site.kind = Some(SiteKind::Refactor);
                return site;
            },
        };
        let emprise = plan.emprise();
        let alt = village_alt(land, &site, emprise);

        // La place (la première : on y apparaît), puis les chemins.
        let road_kind = plot::RoadKind {
            lights: plot::RoadLights::Default,
            material: plot::RoadMaterial::Cobblestone,
        };
        for rect in std::iter::once(plan.place).chain(plan.chemins.iter().copied()) {
            let aabr = rect.aabr();
            let mut plaza = plot::Plaza::generate(aabr, road_kind, &site, land, index, &mut rng);
            plaza.alt = alt;
            plaza.hard_alt = None;
            let id = site.create_plot(Plot {
                kind: PlotKind::Plaza(plaza),
                root_tile: aabr.center(),
                tiles: aabr_tiles(aabr).collect(),
            });
            site.plazas.push(id);
            site.blit_aabr(aabr, Tile {
                kind: TileKind::Road {
                    a: 0,
                    b: 0,
                    w: 0,
                    alt: alt as f32,
                },
                plot: Some(id),
                hard_alt: Some(alt),
            });
        }
        site.kind = Some(SiteKind::Refactor);

        // Les bâtiments, à leur place.
        for lieu in &plan.lieux {
            let aabr = lieu.bati_aabr();
            let door_tile = lieu.porte_tuile();
            let door_dir = lieu.porte.dir();
            let kind = match lieu.forme {
                Forme::Maison => PlotKind::House(plot::House::generate(
                    land,
                    &mut reseed(&mut rng),
                    &site,
                    door_tile,
                    door_dir,
                    aabr,
                    calendar,
                    Some(alt),
                )),
                Forme::Atelier => PlotKind::Workshop(plot::Workshop::generate(
                    land,
                    &mut reseed(&mut rng),
                    &site,
                    door_tile,
                    door_dir,
                    aabr,
                    Some(alt),
                )),
                Forme::Taverne => PlotKind::Tavern(plot::Tavern::generate(
                    land,
                    index,
                    &mut reseed(&mut rng),
                    &site,
                    door_tile,
                    Dir2::from_vec2(door_dir),
                    aabr,
                    Some(alt),
                )),
                // Le terrain commun s'en charge.
                Forme::Terrain => continue,
            };
            let id = site.create_plot(Plot {
                kind,
                root_tile: aabr.center(),
                tiles: aabr_tiles(aabr).collect(),
            });
            site.blit_aabr(aabr, Tile {
                kind: TileKind::Building,
                plot: Some(id),
                hard_alt: Some(alt),
            });
        }

        // Le reste de l'emprise et la bordure : le terrain commun.
        let tile_size = TILE_SIZE as i32;
        let marge_tuiles = plan.marge.max(0);
        let terrain = IcebreakerTerrain {
            emprise: Aabr {
                min: site.tile_wpos(emprise.min),
                max: site.tile_wpos(emprise.max),
            },
            alt,
            marge: marge_tuiles * tile_size,
        };
        let autour = Aabr {
            min: emprise.min - marge_tuiles,
            max: emprise.max + marge_tuiles,
        };
        let libres = aabr_tiles(autour)
            .filter(|t| site.tiles.get(*t).plot.is_none())
            .collect::<Vec<_>>();
        let id = site.create_plot(Plot {
            kind: PlotKind::IcebreakerTerrain(terrain),
            root_tile: emprise.center(),
            tiles: libres.iter().copied().collect(),
        });
        for t in libres {
            let dedans = emprise.min.x <= t.x && t.x < emprise.max.x && emprise.min.y <= t.y && t.y < emprise.max.y;
            site.tiles.set(t, Tile {
                kind: TileKind::Field,
                plot: Some(id),
                hard_alt: dedans.then_some(alt),
            });
        }
        info!(?origin, alt, plots = site.plots.values().len(), "Icebreaker : village fixe construit");
        site
    }
}

/// L'altitude du village : la moyenne du terrain d'origine sur son emprise
/// (tuiles).
fn village_alt(land: &Land, site: &Site, emprise: Aabr<i32>) -> i32 {
    let (somme, n) = aabr_tiles(emprise).fold((0.0, 0.0), |(s, n), t| {
        (s + land.get_alt_approx(site.tile_center_wpos(t)), n + 1.0)
    });
    let moyenne = if n > 0.0 { somme / n } else { land.get_alt_approx(site.origin) };
    moyenne.round() as i32
}
