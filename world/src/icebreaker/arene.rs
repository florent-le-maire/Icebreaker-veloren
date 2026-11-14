//! Icebreaker : le plateau du lieu d'entraînement (jalon O). Le monde de ce lieu n'a aucun site :
//! on choisit l'endroit le plus plat près du centre de la carte et on y nivelle un disque
//! (`assets/icebreaker/arene.ron`), au moment où chaque tronçon est généré.

use crate::sim::WorldSim;
use common::{
    icebreaker_arene::AreneConfig,
    terrain::{Block, BlockKind, TerrainChunk, TerrainChunkSize},
    vol::{RectVolSize, WriteVol},
};
use std::sync::OnceLock;
use vek::*;

/// Le plateau : son centre (en blocs) et la hauteur de son sol (où l'on marche).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Arene {
    pub centre: Vec2<i32>,
    pub alt: i32,
}

/// Le plateau de ce monde ; `None` si toute la zone cherchée est sous l'eau. Calculé une fois :
/// un serveur n'a qu'un monde.
pub fn place(sim: &WorldSim) -> Option<Arene> {
    static ARENE: OnceLock<Option<Arene>> = OnceLock::new();
    *ARENE.get_or_init(|| {
        let config = common::icebreaker_arene::config();
        let taille = sim.get_size().map(|e| e as i32);
        let portee = (config.rayon_terrain / TerrainChunkSize::RECT_SIZE.x as f32).ceil() as i32;
        let (troncon, alt) = choisir(taille, config.recherche, portee, |pos| {
            sim.get(pos).map(|c| (c.alt, c.is_underwater()))
        })?;
        Some(Arene {
            centre: TerrainChunkSize::center_wpos(troncon),
            alt: alt.round() as i32,
        })
    })
}

/// Le tronçon le plus plat à `recherche` tronçons au plus du centre d'une carte de `taille` :
/// celui dont les voisins (à `portee` tronçons) s'écartent le moins de sa hauteur, tous hors
/// d'eau ; à égalité, le plus proche du centre. `terrain` donne la hauteur d'un tronçon et s'il
/// est sous l'eau.
pub fn choisir(
    taille: Vec2<i32>,
    recherche: i32,
    portee: i32,
    terrain: impl Fn(Vec2<i32>) -> Option<(f32, bool)>,
) -> Option<(Vec2<i32>, f32)> {
    let centre = taille / 2;
    let voisins = |pos: Vec2<i32>| {
        (-portee..=portee).flat_map(move |y| (-portee..=portee).map(move |x| pos + Vec2::new(x, y)))
    };
    (-recherche..=recherche)
        .flat_map(|y| (-recherche..=recherche).map(move |x| centre + Vec2::new(x, y)))
        .filter_map(|pos| {
            let (alt, _) = terrain(pos)?;
            let mut ecart = 0.0f32;
            for voisin in voisins(pos) {
                let (a, sous_eau) = terrain(voisin)?;
                if sous_eau {
                    return None;
                }
                ecart = ecart.max((a - alt).abs());
            }
            Some((pos, alt, ecart))
        })
        .min_by(|a, b| {
            a.2.total_cmp(&b.2)
                .then_with(|| a.0.distance_squared(centre).cmp(&b.0.distance_squared(centre)))
        })
        .map(|(pos, alt, _)| (pos, alt))
}

/// Ce que devient une colonne à `distance` du centre du plateau.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Colonne {
    /// Hors du plateau : le terrain du monde.
    Dehors,
    Sol,
    /// L'anneau qui marque le bord de l'arène.
    Bord,
}

pub fn colonne(distance: f32, config: &AreneConfig) -> Colonne {
    if distance > config.rayon_terrain {
        Colonne::Dehors
    } else if (distance - config.rayon).abs() <= 0.75 {
        Colonne::Bord
    } else {
        Colonne::Sol
    }
}

/// Nivelle la part du plateau qui tombe dans ce tronçon (coin `chunk_wpos2d`) : de l'air
/// au-dessus du sol, de la terre dessous.
pub fn niveler(chunk: &mut TerrainChunk, chunk_wpos2d: Vec2<i32>, arene: &Arene, config: &AreneConfig) {
    let taille = TerrainChunkSize::RECT_SIZE.map(|e| e as i32);
    let bloc = |(r, g, b): (u8, u8, u8)| Block::new(BlockKind::Grass, Rgb::new(r, g, b));
    let terre = Block::new(BlockKind::Earth, Rgb::new(92, 72, 52));
    for y in 0..taille.y {
        for x in 0..taille.x {
            let offset = (chunk_wpos2d + Vec2::new(x, y) - arene.centre).map(|e| e as f32);
            let dessus = match colonne(offset.magnitude(), config) {
                Colonne::Dehors => continue,
                Colonne::Sol => bloc(config.couleur_sol),
                Colonne::Bord => bloc(config.couleur_bord),
            };
            for z in arene.alt..arene.alt + config.hauteur_libre {
                let _ = chunk.set(Vec3::new(x, y, z), Block::empty());
            }
            for z in arene.alt - config.profondeur..arene.alt - 1 {
                let _ = chunk.set(Vec3::new(x, y, z), terre);
            }
            let _ = chunk.set(Vec3::new(x, y, arene.alt - 1), dessus);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AreneConfig { common::icebreaker_arene::config() }

    #[test]
    fn arene_prend_le_troncon_le_plus_plat() {
        // Une pente partout, sauf un carré plat autour de (20, 18).
        let plat = Vec2::new(20, 18);
        let terrain = |pos: Vec2<i32>| {
            let d = (pos - plat).map(i32::abs).reduce_max();
            Some((if d <= 2 { 100.0 } else { 100.0 + pos.x as f32 * 7.0 }, false))
        };
        assert_eq!(choisir(Vec2::new(32, 32), 8, 2, terrain), Some((plat, 100.0)));
    }

    #[test]
    fn arene_evite_l_eau() {
        // Tout est plat, mais l'ouest de la carte est sous l'eau : à égalité, le plus proche du
        // centre dont aucun voisin n'est mouillé.
        let terrain = |pos: Vec2<i32>| Some((50.0, pos.x < 17));
        assert_eq!(choisir(Vec2::new(32, 32), 8, 2, terrain), Some((Vec2::new(19, 16), 50.0)));
        // Tout sous l'eau : pas de plateau.
        assert_eq!(choisir(Vec2::new(32, 32), 8, 2, |_| Some((50.0, true))), None);
    }

    #[test]
    fn arene_colonnes() {
        let c = config();
        assert_eq!(colonne(0.0, &c), Colonne::Sol);
        assert_eq!(colonne(c.rayon, &c), Colonne::Bord);
        assert_eq!(colonne(c.rayon_terrain - 0.5, &c), Colonne::Sol);
        assert_eq!(colonne(c.rayon_terrain + 0.5, &c), Colonne::Dehors);
    }

    #[test]
    fn arene_plateau_plat() {
        use common::{
            terrain::TerrainChunkMeta,
            vol::ReadVol,
        };
        let c = config();
        let arene = Arene { centre: Vec2::new(16, 16), alt: 40 };
        let pierre = Block::new(BlockKind::Rock, Rgb::new(100, 100, 100));
        // Un tronçon de pierre jusqu'à 70 : plus haut que le plateau.
        let mut chunk = TerrainChunk::new(0, pierre, Block::empty(), TerrainChunkMeta::void());
        for z in 0..70 {
            for y in 0..32 {
                for x in 0..32 {
                    let _ = chunk.set(Vec3::new(x, y, z), pierre);
                }
            }
        }
        niveler(&mut chunk, Vec2::zero(), &arene, &c);
        for (x, y) in [(0, 0), (16, 16), (31, 5)] {
            assert!(chunk.get(Vec3::new(x, y, 39)).unwrap().is_filled(), "sol en {x},{y}");
            assert!(!chunk.get(Vec3::new(x, y, 40)).unwrap().is_filled(), "air en {x},{y}");
            assert!(!chunk.get(Vec3::new(x, y, 69)).unwrap().is_filled(), "relief coupé en {x},{y}");
        }
    }
}
