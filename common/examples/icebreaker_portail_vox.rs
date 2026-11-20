//! Icebreaker F.21 : écrit les modèles du portail (éteint, allumé), de l'autel de résurrection et du
//! cristal d'un mort, d'après les images de Flo (2026-11-20) :
//! `assets/voxygen/voxel/object/icebreaker_portail.vox`, `icebreaker_portail_actif.vox`,
//! `icebreaker_autel.vox` et `assets/voxygen/voxel/item/icebreaker/cristal.vox`. Les fichiers se
//! retouchent ensuite dans MagicaVoxel ; relancer ce programme les écrase.
//!
//! `cargo run -p veloren-common --example icebreaker_portail_vox [dossier]` depuis la racine du
//! dépôt ; avec un dossier, il y écrit aussi une image de contrôle par modèle (face et dessus).
//!
//! 11 voxels font un bloc (échelle des objets). L'avant est vers +y. Le portail fait 5 blocs de
//! large et un peu moins de 5 de haut, l'autel 3 × 2 × 1 blocs.

use dot_vox::{Color, DotVoxData, Model, Size, Voxel};
use std::{collections::BTreeMap, fs::File, io::BufWriter, io::Write};

// Indices de palette (dans le fichier : indice + 1). Veloren lit 8 à 12 comme brillant et 13 à 15
// comme lumineux (`common/src/figure/cell.rs`) ; la pierre reste au-dessus de 21.
const BRILLANT_CLAIR: u8 = 8;
const BRILLANT: u8 = 9;
const BRILLANT_SOMBRE: u8 = 10;
const LUEUR: u8 = 13;
const LUEUR_PALE: u8 = 14;
const LUEUR_CHEVRON: u8 = 15;
const PIERRE: u8 = 30; // 30 à 33, du clair au sombre
const BANDE: u8 = 34;
const CHEVRON: u8 = 35;
const ORANGE: u8 = 36;
const GRES: u8 = 40; // 40 à 42
const GRAVE: u8 = 43;

fn couleur(indice: u8) -> [u8; 3] {
    match indice {
        BRILLANT_CLAIR => [176, 238, 246],
        BRILLANT => [86, 204, 226],
        BRILLANT_SOMBRE => [44, 156, 196],
        LUEUR => [70, 130, 235],
        LUEUR_PALE => [190, 225, 255],
        LUEUR_CHEVRON => [255, 150, 60],
        30 => [132, 130, 126],
        31 => [120, 118, 114],
        32 => [108, 106, 103],
        33 => [96, 94, 92],
        BANDE => [70, 69, 70],
        CHEVRON => [86, 82, 80],
        ORANGE => [196, 84, 44],
        40 => [152, 142, 126],
        41 => [140, 130, 115],
        42 => [126, 117, 104],
        GRAVE => [94, 86, 76],
        _ => [0, 0, 0],
    }
}

/// Un nombre stable par voxel, pour varier la teinte de la pierre.
fn bruit(x: i32, y: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ (z as u32).wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 13)
}

/// Un modèle : ses voxels par position, depuis le coin (0, 0, 0).
struct Modele {
    taille: (i32, i32, i32),
    voxels: BTreeMap<(i32, i32, i32), u8>,
}

impl Modele {
    /// Remplit le modèle : `voxel(x, y, z)` dit l'indice de chaque voxel, ou rien s'il est vide.
    fn rempli(taille: (i32, i32, i32), voxel: impl Fn(i32, i32, i32) -> Option<u8>) -> Self {
        let mut voxels = BTreeMap::new();
        for x in 0..taille.0 {
            for y in 0..taille.1 {
                for z in 0..taille.2 {
                    if let Some(i) = voxel(x, y, z) {
                        voxels.insert((x, y, z), i);
                    }
                }
            }
        }
        Self { taille, voxels }
    }

    fn ecrire(&self, chemin: &str) {
        let palette = (0..=255u8)
            .map(|i| {
                let [r, g, b] = couleur(i);
                Color { r, g, b, a: 255 }
            })
            .collect();
        let voxels = self.voxels.iter().map(|((x, y, z), i)| Voxel { x: *x as u8, y: *y as u8, z: *z as u8, i: *i }).collect();
        let data = DotVoxData {
            version: 150,
            index_map: (0..=255).collect(),
            models: vec![Model {
                size: Size { x: self.taille.0 as u32, y: self.taille.1 as u32, z: self.taille.2 as u32 },
                voxels,
            }],
            palette,
            materials: Vec::new(),
            scenes: Vec::new(),
            layers: Vec::new(),
        };
        let mut fichier = BufWriter::new(File::create(chemin).expect("lancer depuis la racine du dépôt"));
        data.write_vox(&mut fichier).expect("écriture du .vox");
        println!("{chemin} : {} × {} × {}, {} voxels", self.taille.0, self.taille.1, self.taille.2, self.voxels.len());
    }

    /// L'image de contrôle : la face avant (vue depuis +y) à gauche, le dessus à droite ; les voxels
    /// du fond sont assombris.
    fn image(&self, chemin: &str) {
        const PX: i32 = 6;
        let (sx, sy, sz) = self.taille;
        let (largeur, hauteur) = ((sx * 2 + 2) * PX, sy.max(sz) * PX);
        let mut pixels = vec![[24u8, 24, 28]; (largeur * hauteur) as usize];
        let mut carre = |cx: i32, cy: i32, c: [u8; 3]| {
            for dx in 0..PX {
                for dy in 0..PX {
                    pixels[((cy * PX + dy) * largeur + cx * PX + dx) as usize] = c;
                }
            }
        };
        let teinte = |i: u8, recul: f32| couleur(i).map(|c| (c as f32 * (1.0 - 0.5 * recul)) as u8);
        for x in 0..sx {
            for z in 0..sz {
                if let Some(y) = (0..sy).rev().find(|y| self.voxels.contains_key(&(x, *y, z))) {
                    carre(x, sz - 1 - z, teinte(self.voxels[&(x, y, z)], (sy - 1 - y) as f32 / sy as f32));
                }
            }
            for y in 0..sy {
                if let Some(z) = (0..sz).rev().find(|z| self.voxels.contains_key(&(x, y, *z))) {
                    carre(sx + 2 + x, sy - 1 - y, teinte(self.voxels[&(x, y, z)], (sz - 1 - z) as f32 / sz as f32));
                }
            }
        }
        ecrire_png(chemin, largeur as u32, hauteur as u32, &pixels);
    }
}

/// Un PNG sans compression (assez pour une image de contrôle).
fn ecrire_png(chemin: &str, largeur: u32, hauteur: u32, pixels: &[[u8; 3]]) {
    fn crc(octets: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for o in octets {
            c ^= *o as u32;
            for _ in 0..8 {
                c = if c & 1 == 1 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
        !c
    }
    fn bloc(sortie: &mut Vec<u8>, nom: &[u8; 4], contenu: &[u8]) {
        sortie.extend((contenu.len() as u32).to_be_bytes());
        let mut corps = nom.to_vec();
        corps.extend(contenu);
        sortie.extend(&corps);
        sortie.extend(crc(&corps).to_be_bytes());
    }
    let mut brut = Vec::new();
    for ligne in pixels.chunks(largeur as usize) {
        brut.push(0);
        brut.extend(ligne.iter().flatten());
    }
    let (mut a, mut b) = (1u32, 0u32);
    for o in &brut {
        a = (a + *o as u32) % 65521;
        b = (b + a) % 65521;
    }
    let mut zlib = vec![0x78, 0x01];
    let morceaux: Vec<&[u8]> = brut.chunks(65535).collect();
    for (n, morceau) in morceaux.iter().enumerate() {
        zlib.push(u8::from(n + 1 == morceaux.len()));
        zlib.extend((morceau.len() as u16).to_le_bytes());
        zlib.extend((!(morceau.len() as u16)).to_le_bytes());
        zlib.extend(*morceau);
    }
    zlib.extend(((b << 16) | a).to_be_bytes());
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut entete = Vec::new();
    entete.extend(largeur.to_be_bytes());
    entete.extend(hauteur.to_be_bytes());
    entete.extend([8, 2, 0, 0, 0]);
    bloc(&mut png, b"IHDR", &entete);
    bloc(&mut png, b"IDAT", &zlib);
    bloc(&mut png, b"IEND", &[]);
    File::create(chemin).and_then(|mut f| f.write_all(&png)).expect("écriture de l'image");
}

/// Le portail : un anneau de pierre debout, à neuf chevrons, enfoncé dans un socle à deux marches ;
/// allumé, une surface lumineuse le ferme et ses chevrons brillent.
fn portail(actif: bool) -> Modele {
    const TAILLE: (i32, i32, i32) = (59, 33, 54);
    // Le centre de l'anneau : son bord intérieur affleure le haut du socle (4 voxels).
    const CENTRE_Z: f32 = 24.0;
    const DEDANS: f32 = 20.5;
    const DEHORS: f32 = 27.5;
    Modele::rempli(TAILLE, |vx, vy, vz| {
        let (x, y, z) = ((vx - TAILLE.0 / 2) as f32, (vy - TAILLE.1 / 2) as f32, vz as f32);
        let ton = PIERRE + (bruit(vx, vy, vz) % 4) as u8;
        let rayon = (x * x + (z - CENTRE_Z).powi(2)).sqrt();
        // L'angle depuis le haut de l'anneau, et l'écart au chevron le plus proche (un tous les 40°).
        let angle = x.atan2(z - CENTRE_Z).to_degrees();
        let ecart = (angle.rem_euclid(40.0) - 20.0).abs() - 20.0;
        let arc = ecart.abs().to_radians() * rayon;
        // Les chevrons : un bloc qui dépasse de l'anneau, un V orange pointé vers le centre.
        if (23.0..=29.6).contains(&rayon) && arc <= 4.2 && y.abs() <= 4.0 {
            let v = (25.0..=29.0).contains(&rayon) && arc <= (rayon - 24.4) * 0.55 && y.abs() >= 3.0;
            return Some(if !v {
                CHEVRON
            } else if actif {
                LUEUR_CHEVRON
            } else {
                ORANGE
            });
        }
        if (DEDANS..=DEHORS).contains(&rayon) {
            // La bande intérieure, en retrait, gravée de signes ; le reste de l'anneau, rayé de joints.
            if rayon <= 22.6 {
                let signe = (angle.rem_euclid(10.0) < 5.0) != (rayon < 21.6);
                return (y.abs() <= 2.0).then_some(if signe { BANDE } else { 33 });
            }
            if y.abs() <= 3.0 {
                let joint = angle.rem_euclid(20.0) < 1.6 && rayon < 25.6;
                return Some(if joint { BANDE } else { ton });
            }
        }
        // Le socle : deux marches, dallées.
        let dalle = |bord: bool| if bord { 33 } else { ton };
        if vz <= 1 && x.abs() <= 29.0 && y.abs() <= 16.0 {
            return Some(dalle(vz == 1 && (vx % 10 == 4 || vy % 11 == 5)));
        }
        if (2..=3).contains(&vz) && x.abs() <= 26.0 && y.abs() <= 11.0 {
            return Some(dalle(vz == 3 && (vx % 10 == 9 || vy % 11 == 0)));
        }
        // La surface : des rides claires sur un fond bleu.
        if actif && rayon < DEDANS && y == 0.0 && vz >= 4 {
            let ride = (x * 0.33 + (z * 0.29).sin() * 1.6).sin() + (z * 0.47 + x * 0.12).sin();
            let eclat = bruit(vx, 0, vz) % 9 == 0;
            return Some(if ride > 0.9 || eclat { LUEUR_PALE } else { LUEUR });
        }
        None
    })
}

/// L'autel : un coffre de pierre sur une plinthe, une frise sur ses flancs, une dalle qui déborde,
/// et un disque gravé au milieu du dessus.
fn autel() -> Modele {
    const TAILLE: (i32, i32, i32) = (33, 22, 12);
    Modele::rempli(TAILLE, |x, y, z| {
        let ton = GRES + (bruit(x, y, z) % 3) as u8;
        // À combien de voxels du bord du modèle.
        let bord = x.min(TAILLE.0 - 1 - x).min(y).min(TAILLE.1 - 1 - y);
        match z {
            0..=1 => Some(if z == 1 && bord == 0 { GRAVE } else { 42 }),
            2..=8 => {
                if bord < 1 {
                    return None;
                }
                // La frise : des losanges sur les flancs, entre deux filets.
                let flanc = bord == 1;
                let le_long = if x == 1 || x == TAILLE.0 - 2 { y } else { x };
                let frise = match z {
                    3 | 7 => true,
                    4..=6 => (le_long % 4 - 2).abs() + (z - 5).abs() <= 1,
                    _ => false,
                };
                Some(if flanc && frise { GRAVE } else { ton })
            },
            9..=11 => {
                if z == 11 && bord < 1 {
                    return None;
                }
                if z < 11 {
                    return Some(if z == 9 && bord == 0 { GRAVE } else { ton });
                }
                // Le dessus : un cadre gravé, et le disque (un anneau, des rayons, un cœur).
                let (dx, dy) = (x as f32 - 16.0, y as f32 - 10.5);
                let rayon = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx).to_degrees().rem_euclid(45.0);
                let grave = bord == 3
                    || (6.6..=7.8).contains(&rayon)
                    || rayon < 1.6
                    || ((2.6..=5.6).contains(&rayon) && !(8.0..=37.0).contains(&angle));
                Some(if grave { GRAVE } else { ton })
            },
            _ => None,
        }
    })
}

/// Le cristal : un prisme à six pans, bleu aigue-marine, coupé en biseau au sommet.
fn cristal() -> Modele {
    const TAILLE: (i32, i32, i32) = (7, 7, 14);
    Modele::rempli(TAILLE, |vx, vy, vz| {
        let (x, y) = (vx - 3, vy - 3);
        let pans = x.abs() <= 3 && y.abs() <= 3 && x.abs() + y.abs() <= 4;
        // Le sommet en biseau (plus haut à gauche), la base un peu ébréchée.
        let sommet = 12 - (x + 3) / 2 - i32::from(y.abs() == 3);
        let base = i32::from(x.abs() + y.abs() == 4 && (x + y) % 2 == 0);
        if !pans || vz > sommet || vz < base {
            return None;
        }
        let arete = x.abs() + y.abs() == 4 || vz == sommet;
        Some(if arete && x <= 0 {
            BRILLANT_CLAIR
        } else if x > 1 || (vz + x) % 7 == 0 {
            BRILLANT_SOMBRE
        } else {
            BRILLANT
        })
    })
}

fn main() {
    let images = std::env::args().nth(1);
    let modeles = [
        ("assets/voxygen/voxel/object/icebreaker_portail.vox", portail(false)),
        ("assets/voxygen/voxel/object/icebreaker_portail_actif.vox", portail(true)),
        ("assets/voxygen/voxel/object/icebreaker_autel.vox", autel()),
        ("assets/voxygen/voxel/item/icebreaker/cristal.vox", cristal()),
    ];
    std::fs::create_dir_all("assets/voxygen/voxel/item/icebreaker").expect("lancer depuis la racine du dépôt");
    for (chemin, modele) in &modeles {
        modele.ecrire(chemin);
        if let Some(dossier) = &images {
            let nom = chemin.rsplit('/').next().unwrap_or_default().replace(".vox", ".png");
            modele.image(&format!("{dossier}/{nom}"));
        }
    }
}
