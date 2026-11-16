//! Icebreaker FT.1 : écrit le modèle du fantôme du village
//! (`assets/voxygen/voxel/object/icebreaker_fantome.vox`). Le fichier se retouche ensuite dans
//! MagicaVoxel ; relancer ce programme l'écrase.
//!
//! `cargo run -p veloren-common --example icebreaker_fantome_vox` depuis la racine du dépôt.
//!
//! 11 voxels font un bloc (échelle des objets) : 21 voxels de haut, soit un humain. L'avant est
//! vers +y.

use dot_vox::{Color, DotVoxData, Model, Size, Voxel};
use std::{fs::File, io::BufWriter};

const LARGEUR: i32 = 15;
const PROFONDEUR: i32 = 11;
const HAUTEUR: i32 = 21;
/// Hauteur des épaules : au-dessus, la tête en dôme ; au-dessous, le drapé.
const EPAULES: f32 = 14.0;

// Indices de palette (dans le fichier : indice + 1). Veloren lit 13 à 15 comme « lumineux » et
// 8 à 21 comme matières spéciales : le drapé reste au-dessus.
const DRAPE: u8 = 30; // 30 (haut, clair) à 35 (bas, soutenu)
const YEUX: u8 = 14;
const BOUCHE: u8 = 40;

fn couleur(indice: u8) -> Option<[u8; 3]> {
    match indice {
        30 => Some([198, 230, 255]),
        31 => Some([176, 216, 255]),
        32 => Some([152, 200, 252]),
        33 => Some([128, 182, 246]),
        34 => Some([106, 164, 238]),
        35 => Some([86, 144, 228]),
        YEUX => Some([210, 245, 255]),
        BOUCHE => Some([32, 54, 120]),
        _ => None,
    }
}

/// L'indice du voxel (x, y, z), ou rien s'il est vide. `x` et `y` sont comptés depuis l'axe.
fn voxel(x: f32, y: f32, z: f32) -> Option<u8> {
    let (rx, ry) = (5.3, 4.3);
    let dedans = if z >= EPAULES {
        // La tête : un demi-ellipsoïde.
        let rz = HAUTEUR as f32 - EPAULES - 0.2;
        (x / rx).powi(2) + (y / ry).powi(2) + ((z - EPAULES) / rz).powi(2) <= 1.0
    } else {
        // Le drapé : s'évase un peu vers le bas, et finit en pointes.
        let evase = 1.0 + (EPAULES - z) * 0.012;
        let angle = y.atan2(x);
        let ourlet = 1.5 + 1.5 * (angle * 6.0).cos();
        (x / (rx * evase)).powi(2) + (y / (ry * evase)).powi(2) <= 1.0 && z >= ourlet
    };
    // Deux petits bras tendus vers l'avant.
    let bras = (5.0..7.5).contains(&x.abs()) && (0.0..4.0).contains(&y) && (9.0..11.0).contains(&z);
    if !dedans && !bras {
        return None;
    }
    // Le visage, sur la face avant.
    if y >= 2.0 && dedans {
        if (1.0..3.0).contains(&x.abs()) && (15.0..17.0).contains(&z) {
            return Some(YEUX);
        }
        if x.abs() < 1.0 && (12.0..14.0).contains(&z) {
            return Some(BOUCHE);
        }
    }
    // Du clair en haut au soutenu en bas.
    let rang = ((HAUTEUR as f32 - 1.0 - z) / HAUTEUR as f32 * 6.0) as u8;
    Some(DRAPE + rang.min(5))
}

fn main() {
    let mut voxels = Vec::new();
    for x in 0..LARGEUR {
        for y in 0..PROFONDEUR {
            for z in 0..HAUTEUR {
                let dx = x as f32 - (LARGEUR - 1) as f32 / 2.0;
                let dy = y as f32 - (PROFONDEUR - 1) as f32 / 2.0;
                if let Some(i) = voxel(dx, dy, z as f32) {
                    voxels.push(Voxel {
                        x: x as u8,
                        y: y as u8,
                        z: z as u8,
                        i,
                    });
                }
            }
        }
    }
    let palette = (0..=255u8)
        .map(|i| {
            let [r, g, b] = couleur(i).unwrap_or([0, 0, 0]);
            Color { r, g, b, a: 255 }
        })
        .collect();
    let data = DotVoxData {
        version: 150,
        index_map: (0..=255).collect(),
        models: vec![Model {
            size: Size {
                x: LARGEUR as u32,
                y: PROFONDEUR as u32,
                z: HAUTEUR as u32,
            },
            voxels,
        }],
        palette,
        materials: Vec::new(),
        scenes: Vec::new(),
        layers: Vec::new(),
    };
    let chemin = "assets/voxygen/voxel/object/icebreaker_fantome.vox";
    let mut fichier = BufWriter::new(File::create(chemin).expect("lancer depuis la racine du dépôt"));
    data.write_vox(&mut fichier).expect("écriture du .vox");
    println!("{chemin} : {} voxels", data.models[0].voxels.len());
    // Vue de face (ce qu'on voit depuis +y), pour vérifier la forme sans lancer le jeu.
    for z in (0..HAUTEUR).rev() {
        let ligne: String = (0..LARGEUR)
            .map(|x| {
                let devant = data.models[0]
                    .voxels
                    .iter()
                    .filter(|v| v.x as i32 == x && v.z as i32 == z)
                    .max_by_key(|v| v.y);
                match devant.map(|v| v.i) {
                    None => ' ',
                    Some(YEUX) => 'o',
                    Some(BOUCHE) => '_',
                    Some(_) => '#',
                }
            })
            .collect();
        println!("{ligne}");
    }
}
