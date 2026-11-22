//! Icebreaker VF : le village fixe (décision 0022), construit d'après
//! `assets/icebreaker/village_plan.ron` au lieu de la ville tirée au hasard de
//! Veloren. Tout le village est à une seule altitude ; autour, une bordure de
//! raccord passe en pente douce à la hauteur du pays.

use super::*;
use crate::{
    Land,
    icebreaker::village_plan::{Cote, Decor, Forme, VillagePlan},
    site::namegen::NameGen,
};
use crate::ColumnSample;
use common::{
    calendar::Calendar,
    terrain::{Block, BlockKind, SpriteKind, sprite::RelativeNeighborPosition},
};
use rand_chacha::ChaCha8Rng;
use rand::prelude::*;
use tracing::{error, info};
use vek::*;

/// Le terrain du village : plat dans son emprise, en pente douce dans la
/// bordure. Il règle la hauteur du sol et dessine les cimetières et enclos
/// (VF.4).
pub struct IcebreakerTerrain {
    /// L'emprise du village (blocs, `max` exclu), à l'altitude `alt`.
    emprise: Aabr<i32>,
    alt: i32,
    /// La largeur de la bordure (blocs).
    marge: i32,
    /// VF.4 : les cimetières et enclos (blocs, `max` exclu), avec le côté de
    /// leur entrée.
    decors: Vec<(Aabr<i32>, Decor, Cote)>,
}

/// Icebreaker VF.4 : un nombre tiré d'une position (le même à chaque fois).
fn hache(p: Vec2<i32>) -> u32 {
    let mut h = (p.x as u32).wrapping_mul(0x9E37_79B1) ^ (p.y as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

/// Icebreaker VF.4 : la clôture au bord de `aabr` (blocs, `max` exclu) en
/// `wpos` : (orientation, coin ?) du piquet de Veloren ; rien hors du bord, ni
/// dans l'entrée (4 blocs au milieu du côté `entree`).
pub(crate) fn cloture(aabr: Aabr<i32>, entree: Cote, wpos: Vec2<i32>) -> Option<(u8, bool)> {
    let (min, max) = (aabr.min, aabr.max - 1);
    if wpos.x < min.x || wpos.y < min.y || wpos.x > max.x || wpos.y > max.y {
        return None;
    }
    let (ouest, est, sud, nord) = (wpos.x == min.x, wpos.x == max.x, wpos.y == min.y, wpos.y == max.y);
    if !(ouest || est || sud || nord) {
        return None;
    }
    let centre = aabr.center();
    let dans_entree = |c: i32, milieu: i32| (-2..2).contains(&(c - milieu));
    let entre = match entree {
        Cote::Nord => nord && dans_entree(wpos.x, centre.x),
        Cote::Sud => sud && dans_entree(wpos.x, centre.x),
        Cote::Est => est && dans_entree(wpos.y, centre.y),
        Cote::Ouest => ouest && dans_entree(wpos.y, centre.y),
    };
    if entre {
        return None;
    }
    // Comme les champs de Veloren (`farm_field.rs`).
    let coin = (ouest || est) && (sud || nord);
    let ori = if coin {
        match (ouest, sud) {
            (true, true) => 4,
            (false, true) => 6,
            (true, false) => 2,
            (false, false) => 0,
        }
    } else if ouest || est {
        2
    } else {
        0
    };
    Some((ori, coin))
}

impl IcebreakerTerrain {
    /// VF.4 : le bloc du décor en `wpos`, `z_off` au-dessus du sol (0 : le sol).
    fn decor_at(&self, wpos: Vec2<i32>, old: Block, z_off: i32) -> Option<Block> {
        let (aabr, decor, entree) = self.decors.iter().find(|(aabr, ..)| aabr.contains_point(wpos))?;
        let h = hache(wpos);
        if z_off == 1
            && let Some((ori, coin)) = cloture(*aabr, *entree, wpos)
        {
            let voisins = if coin { RelativeNeighborPosition::L } else { RelativeNeighborPosition::I };
            let block = old.into_vacant().with_sprite(SpriteKind::FenceWoodWoodland);
            let block = block.with_ori(ori).unwrap_or(block);
            return Some(block.with_adjacent_type(voisins).unwrap_or(block));
        }
        let rel = wpos - aabr.min;
        let taille = aabr.size();
        match decor {
            Decor::Cimetiere => {
                // Une allée de terre depuis l'entrée, les tombes en rangs de part et d'autre.
                let allee = match entree {
                    Cote::Nord | Cote::Sud => (rel.x - taille.w / 2).abs() <= 1,
                    Cote::Est | Cote::Ouest => (rel.y - taille.h / 2).abs() <= 1,
                };
                if z_off == 0 && allee {
                    return Some(Block::new(BlockKind::Earth, Rgb::new(110, 85, 60)));
                }
                let dedans = rel.x >= 2 && rel.y >= 2 && rel.x < taille.w - 2 && rel.y < taille.h - 2;
                if z_off != 1 || allee || !dedans {
                    return None;
                }
                let tombe = rel.x % 4 == 2 && rel.y % 5 == 2;
                if tombe && h % 7 != 0 {
                    let sprite = if h % 3 == 0 { SpriteKind::Grave } else { SpriteKind::Gravestone };
                    // Face à l'allée.
                    let ori = match entree {
                        Cote::Nord | Cote::Sud => if rel.x < taille.w / 2 { 2 } else { 6 },
                        Cote::Est | Cote::Ouest => if rel.y < taille.h / 2 { 0 } else { 4 },
                    };
                    let block = old.into_vacant().with_sprite(sprite);
                    Some(block.with_ori(ori).unwrap_or(block))
                } else if h % 61 == 0 {
                    Some(old.into_vacant().with_sprite(SpriteKind::WhiteFlower))
                } else {
                    None
                }
            },
            Decor::Enclos => {
                if z_off == 0 {
                    // Terre battue, un peu inégale.
                    let teinte = (h % 16) as u8;
                    return Some(Block::new(BlockKind::Earth, Rgb::new(105 + teinte, 80 + teinte, 55)));
                }
                // Du foin en tas, plutôt au fond (loin de l'entrée).
                let fond = match entree {
                    Cote::Nord => rel.y < taille.h / 3,
                    Cote::Sud => rel.y >= taille.h * 2 / 3,
                    Cote::Est => rel.x < taille.w / 3,
                    Cote::Ouest => rel.x >= taille.w * 2 / 3,
                };
                let dedans = rel.x >= 1 && rel.y >= 1 && rel.x < taille.w - 1 && rel.y < taille.h - 1;
                (z_off == 1 && dedans && fond && h % 9 == 0)
                    .then(|| old.into_vacant().with_sprite(SpriteKind::Hay))
            },
            // Les champs sont des champs de Veloren (`FarmField`).
            Decor::Champ => None,
        }
    }

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

    // Icebreaker VF.4 : cimetière et enclos.
    fn terrain_surface_at_inner(
        &self,
        wpos: Vec2<i32>,
        old: Block,
        _rng: &mut ChaCha8Rng,
        _col: &ColumnSample,
        z_off: i32,
        _site: &Site,
    ) -> Option<Block> {
        self.decor_at(wpos, old, z_off)
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
                // VF.4 : un champ est un champ de Veloren, à la hauteur du village.
                Forme::Terrain if lieu.decor == Some(Decor::Champ) => {
                    let field = plot::FarmField::icebreaker(&mut reseed(&mut rng), &site, aabr, alt);
                    let id = site.create_plot(Plot {
                        kind: PlotKind::FarmField(field),
                        root_tile: aabr.center(),
                        tiles: aabr_tiles(aabr).collect(),
                    });
                    site.blit_aabr(aabr, Tile {
                        kind: TileKind::Field,
                        plot: Some(id),
                        hard_alt: Some(alt),
                    });
                    continue;
                },
                // Le terrain commun s'en charge (et du cimetière, de l'enclos).
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
            decors: plan
                .lieux
                .iter()
                .filter(|lieu| lieu.forme == Forme::Terrain)
                .filter_map(|lieu| {
                    let decor = lieu.decor.filter(|d| *d != Decor::Champ)?;
                    let tuiles = lieu.rect().aabr();
                    let blocs = Aabr {
                        min: site.tile_wpos(tuiles.min),
                        max: site.tile_wpos(tuiles.max),
                    };
                    Some((blocs, decor, lieu.porte))
                })
                .collect(),
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
