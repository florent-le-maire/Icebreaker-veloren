//! Icebreaker VF : génère un petit monde avec le village fixe (comme
//! `icebreaker.ps1 village`), en dessine la vue de dessus dans une image
//! (un pixel par bloc, nord en haut) et dit si son sol est plat.
//!
//! `cargo run -p veloren-world --example icebreaker_village -- [graine] [image.png]`
//!
//! Le sol plat se mesure sur la place, les chemins et les terrains nus de
//! l'emprise : la hauteur du premier bloc plein (pas une plante) de chaque
//! colonne, sans compter les bords des tuiles (lampadaires, clôtures, marches) ;
//! réussi si 97 % des colonnes sont à un bloc près de la médiane.

use common::{
    terrain::{Block, SpriteKind, TerrainChunkSize},
    vol::{ReadVol, RectVolSize},
};
use std::collections::HashMap;
use vek::*;
use veloren_world::{
    World,
    icebreaker::village_plan::VillagePlan,
    site::{SiteKind, plot::PlotKind},
    sim::{FileOpts, GenOpts, WorldOpts},
};

fn main() {
    let seed = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1234);
    let sortie = std::env::args().nth(2).unwrap_or_else(|| format!("village-{seed}.png"));
    // SAFETY: set before any other thread is started.
    unsafe { std::env::set_var("ICEBREAKER_INSTANCE_SITE", "Town") };
    let threadpool = rayon::ThreadPoolBuilder::new().build().unwrap();
    let (world, index) = World::generate(
        seed,
        WorldOpts {
            seed_elements: true,
            world_file: FileOpts::Generate(GenOpts {
                x_lg: 6,
                y_lg: 6,
                ..GenOpts::default()
            }),
            calendar: None,
        },
        &threadpool,
        &|_| {},
    );
    let index = index.as_index_ref();
    let Some(site) = world
        .civs()
        .sites()
        .filter_map(|site| site.site_tmp)
        .map(|id| &index.sites[id])
        .find(|site| site.kind == Some(SiteKind::Refactor))
    else {
        eprintln!("pas de village dans ce monde (graine {seed})");
        std::process::exit(2);
    };
    let plan = VillagePlan::load().expect("plan du village");
    let place = site.tile_center_wpos(site.plot(site.plazas().next().expect("place")).root_tile());
    println!(
        "apparition : en partant du bas {:?}, depuis le sol de la place {:?}",
        world.find_accessible_pos(index, place, false),
        world.find_accessible_pos_above(index, place, veloren_world::icebreaker::village_sol(site).unwrap_or(0) - 2)
    );
    let emprise = plan.emprise();
    let bounds = Aabr {
        min: site.tile_wpos(emprise.min - plan.marge),
        max: site.tile_wpos(emprise.max + plan.marge),
    };

    // Les tronçons qui couvrent le village et sa bordure.
    let chunk_size = TerrainChunkSize::RECT_SIZE.map(|e| e as i32);
    let key_of = |wpos: Vec2<i32>| wpos.map2(chunk_size, |e, s| e.div_euclid(s));
    let (low, high) = (key_of(bounds.min), key_of(bounds.max));
    let mut chunks = HashMap::new();
    for x in low.x..=high.x {
        for y in low.y..=high.y {
            let key = Vec2::new(x, y);
            chunks.insert(key, world.generate_chunk(index, key, None, || false, None).ok().map(|(c, _)| c));
        }
    }
    // Le bloc le plus haut de la colonne, et le premier sol plein (hors plantes).
    let colonne = |wpos: Vec2<i32>| -> Option<(i32, Block, i32)> {
        let key = key_of(wpos);
        let chunk = chunks.get(&key)?.as_ref()?;
        let local = wpos - key * chunk_size;
        let mut haut = None;
        for z in (chunk.get_min_z()..chunk.get_max_z()).rev() {
            let block = *chunk.get(local.with_z(z)).ok()?;
            if haut.is_none() && (block.is_filled() || block.kind().is_liquid() || block.get_sprite().is_some_and(|s| s != SpriteKind::Empty)) {
                haut = Some((z, block));
            }
            if block.is_filled() {
                let (hz, hb) = haut.unwrap_or((z, block));
                return Some((hz, hb, z));
            }
        }
        None
    };

    // Le sol plat : place, chemins et terrains de l'emprise, loin des bords de tuile.
    let mut sols = Vec::new();
    let tile = site.tile_wpos(Vec2::one()) - site.tile_wpos(Vec2::zero());
    for ty in emprise.min.y..emprise.max.y {
        for tx in emprise.min.x..emprise.max.x {
            let t = Vec2::new(tx, ty);
            let plat = site.tiles.get(t).plot.is_some_and(|p| {
                matches!(site.plot(p).kind(), PlotKind::Plaza(_) | PlotKind::IcebreakerTerrain(_))
            });
            if !plat {
                continue;
            }
            let coin = site.tile_wpos(t);
            for dy in 1..tile.y - 1 {
                for dx in 1..tile.x - 1 {
                    if let Some((_, _, sol)) = colonne(coin + Vec2::new(dx, dy)) {
                        sols.push(sol);
                    }
                }
            }
        }
    }
    sols.sort_unstable();
    let quantile = |q: f32| sols.get(((sols.len() as f32 - 1.0) * q) as usize).copied().unwrap_or(0);
    // Les toits qui débordent de leur tuile comptent comme un sol plus haut : on dit quelle part
    // des colonnes est à un bloc près de la médiane.
    let mediane = quantile(0.5);
    let a_plat = sols.iter().filter(|z| (**z - mediane).abs() <= 1).count() as f32 / sols.len().max(1) as f32;
    println!(
        "graine {seed} : village à {:?}, sol sur {} colonnes : médiane {mediane}, {:.1} % à un bloc près (de {} à {})",
        site.origin,
        sols.len(),
        a_plat * 100.0,
        quantile(0.0),
        quantile(1.0)
    );

    // L'image : couleur du bloc du dessus, éclairée selon sa hauteur ; nord en haut.
    let size = bounds.size();
    let mut eau = 0;
    let mut image = image::RgbImage::new(size.w as u32, size.h as u32);
    for y in 0..size.h {
        for x in 0..size.w {
            let wpos = bounds.min + Vec2::new(x, y);
            let couleur = colonne(wpos).map_or(Rgb::new(255, 0, 255), |(z, block, _)| {
                let base = if block.kind().is_liquid() {
                    eau += 1;
                    Rgb::new(40, 90, 230)
                } else {
                    block.get_color().unwrap_or(Rgb::new(120, 120, 120))
                };
                let eclairage = (1.0 + (z - mediane) as f32 * 0.03).clamp(0.4, 1.6);
                base.map(|c| (c as f32 * eclairage).clamp(0.0, 255.0) as u8)
            });
            image.put_pixel(x as u32, (size.h - 1 - y) as u32, image::Rgb([couleur.r, couleur.g, couleur.b]));
        }
    }
    image.save(&sortie).expect("image enregistrée");
    println!("colonnes d'eau dans l'image : {eau}");
    println!("image : {sortie} ({}×{} blocs)", size.w, size.h);
    std::process::exit(if a_plat >= 0.97 { 0 } else { 1 });
}
