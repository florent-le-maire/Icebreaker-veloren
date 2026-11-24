//! Icebreaker F.23 : un petit langage pour écrire nos modèles voxel par du code.
//!
//! Un [`Modele`] est une grille de voxels (11 voxels font un bloc, comme les sprites de Veloren) remplie par des
//! formes (boîte, cylindre, ellipsoïde, ligne) dans une [`Matiere`]. Chaque matière a plusieurs tons ; le ton d'un
//! voxel vient d'un bruit fixe de sa position, donc deux lancements donnent le même fichier. Le modèle s'écrit en
//! `.vox` (relu par Veloren, retouchable dans MagicaVoxel) et en image de contrôle (vue de trois quarts).
//!
//! Indices de palette lus par Veloren (`common/src/figure/cell.rs`) : 8 à 12 brillent (métal, vernis), 13 à 15
//! luisent (braises), 16 à 21 sont creux ; nos matières mates commencent à 22.
//!
//! Repère : x vers la droite du modèle, y vers l'arrière, z vers le haut ; l'avant du modèle regarde -y, comme les meubles
//! de Veloren (l'étagère `shelf-0.vox` a son dos plein en y max : exemple `cotes`) ; il se tourne donc dans l'atelier
//! des salles comme une étagère.

pub mod cuisine;
pub mod filons;
pub mod trone;

use std::collections::BTreeMap;

/// Les matières, chacune avec ses tons dans la palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matiere {
    /// Briques rouges posées en quinconce, joints sombres (le motif vient de la position).
    Briques,
    Joint,
    Pierre,
    BoisClair,
    BoisSombre,
    /// Bois noirci par le feu.
    Suie,
    Fer,
    FerSombre,
    Cuivre,
    Pain,
    Mie,
    Viande,
    Gras,
    Jambon,
    Volaille,
    Carotte,
    Chou,
    Oignon,
    Ail,
    Herbe,
    TerreCuite,
    /// Faïence vernie (écuelles, pichets).
    Faience,
    Cendre,
    Braise,
    Flamme,
    Corde,
    Toile,
    Grain,
    Os,
    // Icebreaker F.37 : la salle du trône.
    Marbre,
    MarbreSombre,
    PierreRose,
    Pourpre,
    PourpreSombre,
    Or,
    VerreBleu,
    VerreVert,
    Cire,
    Vigne,
    // Icebreaker U.23 : les filons. Minerai, Eclat et les deux Gemme prennent la teinte du modèle (`teinter`).
    Minerai,
    Eclat,
    Gemme,
    GemmeCoeur,
    Gangue,
}

impl Matiere {
    /// Les indices de palette de la matière, du plus clair au plus sombre.
    fn tons(self) -> &'static [u8] {
        use Matiere::*;
        match self {
            Fer => &[8, 9],
            FerSombre => &[9, 10],
            Cuivre => &[11],
            Faience => &[12, 22],
            Braise => &[13, 14],
            Flamme => &[15],
            Briques => &[23, 24, 25, 26],
            Joint => &[27],
            Pierre => &[28, 29, 30, 31],
            BoisClair => &[32, 33, 34],
            BoisSombre => &[35, 36, 37],
            Suie => &[38, 39],
            Pain => &[40, 41],
            Mie => &[42],
            Viande => &[43, 44],
            Gras => &[45],
            Jambon => &[46, 47],
            Volaille => &[48, 49],
            Carotte => &[50],
            Chou => &[51, 52],
            Oignon => &[53],
            Ail => &[54],
            Herbe => &[55, 56],
            TerreCuite => &[57, 58],
            Cendre => &[59, 60],
            Corde => &[61],
            Toile => &[62, 63],
            Grain => &[64, 65],
            Os => &[66],
            Marbre => &[67, 68, 69],
            MarbreSombre => &[71, 72],
            PierreRose => &[73, 74, 75],
            Pourpre => &[76, 77],
            PourpreSombre => &[78],
            Or => &[79, 80],
            VerreBleu => &[81],
            VerreVert => &[82],
            Cire => &[83, 84],
            Vigne => &[85, 86],
            Minerai => &[8, 9, 10],
            Eclat => &[11],
            Gemme => &[13, 14],
            GemmeCoeur => &[15],
            Gangue => &[87, 88, 89],
        }
    }
}

/// La couleur de chaque indice de palette.
pub fn couleur(indice: u8) -> [u8; 3] {
    match indice {
        8 => [168, 170, 176],  // fer clair
        9 => [118, 120, 128],  // fer
        10 => [70, 72, 80],    // fer sombre
        11 => [184, 108, 62],  // cuivre
        12 => [214, 204, 176], // faïence vernie
        13 => [214, 70, 24],   // braise rouge
        14 => [250, 140, 40],  // braise orange
        15 => [255, 214, 110], // flamme
        22 => [182, 168, 140], // faïence mate
        23 => [150, 70, 50],
        24 => [138, 62, 45],
        25 => [162, 82, 58],
        26 => [124, 56, 42],
        27 => [70, 60, 54], // joint
        28 => [128, 124, 118],
        29 => [116, 112, 106],
        30 => [104, 100, 96],
        31 => [92, 89, 86],
        32 => [170, 124, 78],
        33 => [156, 112, 70],
        34 => [142, 100, 62],
        35 => [100, 66, 40],
        36 => [88, 58, 36],
        37 => [76, 50, 32],
        38 => [44, 36, 32],
        39 => [32, 27, 25],
        40 => [176, 112, 52], // croûte
        41 => [158, 96, 42],
        42 => [232, 206, 150], // mie
        43 => [166, 48, 44],   // viande
        44 => [140, 36, 36],
        45 => [236, 214, 196], // gras
        46 => [128, 62, 40],   // jambon fumé
        47 => [108, 50, 34],
        48 => [222, 186, 140], // volaille
        49 => [204, 164, 116],
        50 => [226, 118, 34], // carotte
        51 => [104, 150, 64], // chou
        52 => [84, 128, 52],
        53 => [196, 148, 88],  // oignon
        54 => [236, 230, 210], // ail
        55 => [86, 120, 52],   // herbes
        56 => [110, 138, 60],
        57 => [168, 88, 56], // terre cuite
        58 => [146, 74, 48],
        59 => [120, 116, 110], // cendre
        60 => [84, 82, 80],
        61 => [176, 150, 104], // corde
        62 => [196, 176, 136], // toile
        63 => [178, 158, 120],
        64 => [218, 196, 140], // grain
        65 => [200, 176, 118],
        66 => [230, 222, 200], // os
        // Icebreaker F.37 : la salle du trône.
        67 => [200, 192, 174], // marbre
        68 => [190, 182, 165],
        69 => [181, 173, 157],
        70 => [140, 132, 122], // veine du marbre
        71 => [88, 66, 52],    // marbre sombre (cadres, moulures)
        72 => [76, 57, 45],
        73 => [178, 142, 122], // pierre rose (fût des piliers)
        74 => [168, 132, 113],
        75 => [188, 152, 130],
        76 => [130, 28, 36], // pourpre (tapis, coussin)
        77 => [116, 24, 32],
        78 => [80, 16, 24],   // pourpre sombre
        79 => [204, 164, 70], // or (mat)
        80 => [182, 142, 56],
        81 => [56, 88, 170],   // verre bleu
        82 => [54, 126, 84],   // verre vert
        83 => [234, 226, 202], // cire
        84 => [218, 208, 182],
        85 => [72, 82, 60], // vigne peinte
        86 => [60, 70, 50],
        // Icebreaker U.23 : la gangue des filons.
        87 => [78, 74, 72],
        88 => [66, 63, 62],
        89 => [55, 53, 55],
        _ => [0, 0, 0],
    }
}

/// Un nombre fixe par voxel, pour choisir le ton.
pub fn bruit(x: i32, y: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ (z as u32).wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 13)
}

/// Un point de voxel.
pub type P = (i32, i32, i32);

/// Un modèle : sa taille en voxels, son ancre et ses voxels.
pub struct Modele {
    /// Nom du fichier (`<id>.vox`).
    pub id: &'static str,
    pub taille: P,
    /// Le point du modèle (en voxels) qui tombe au centre du bas du bloc où l'on pose le sprite : l'offset du
    /// manifeste est son opposé.
    pub ancre: (f32, f32, f32),
    voxels: BTreeMap<P, u8>,
    /// Icebreaker U.23 : les indices de palette dont ce modèle change la couleur (un minerai, sa teinte).
    teintes: BTreeMap<u8, [u8; 3]>,
}

impl Modele {
    pub fn new(id: &'static str, taille: P, ancre: (f32, f32, f32)) -> Self {
        Self { id, taille, ancre, voxels: BTreeMap::new(), teintes: BTreeMap::new() }
    }

    /// Icebreaker U.23 : donne à l'indice de palette `indice` la couleur `rvb` dans ce modèle seulement.
    pub fn teinter(&mut self, indice: u8, rvb: [u8; 3]) { self.teintes.insert(indice, rvb); }

    /// La couleur d'un indice de palette dans ce modèle.
    pub fn couleur(&self, indice: u8) -> [u8; 3] {
        self.teintes.get(&indice).copied().unwrap_or_else(|| couleur(indice))
    }

    /// L'offset à écrire dans `sprite_manifest.ron`.
    pub fn offset(&self) -> (f32, f32, f32) { (0.0 - self.ancre.0, 0.0 - self.ancre.1, 0.0 - self.ancre.2) }

    pub fn nombre_de_voxels(&self) -> usize { self.voxels.len() }

    pub fn voxel(&self, p: P) -> Option<u8> { self.voxels.get(&p).copied() }

    pub fn voxels(&self) -> impl Iterator<Item = (P, u8)> + '_ { self.voxels.iter().map(|(p, i)| (*p, *i)) }

    fn dedans(&self, (x, y, z): P) -> bool {
        (0..self.taille.0).contains(&x) && (0..self.taille.1).contains(&y) && (0..self.taille.2).contains(&z)
    }

    /// Le ton de la matière en ce point.
    pub fn ton(m: Matiere, (x, y, z): P) -> u8 {
        let tons = m.tons();
        match m {
            Matiere::Briques => {
                // Rangs de 3 voxels (2 de brique, 1 de joint), briques de 5 + 1 joint, décalées d'un rang à l'autre.
                // Le long du mur = x + y, pour que le motif suive les deux faces.
                let rang = z.div_euclid(3);
                let le_long = x + y + if rang % 2 == 0 { 0 } else { 3 };
                if z.rem_euclid(3) == 2 || le_long.rem_euclid(6) == 5 {
                    return Matiere::Joint.tons()[0];
                }
                let brique = bruit(le_long.div_euclid(6), rang, 7);
                tons[(brique as usize) % tons.len()]
            },
            // Le bois : un ton par fibre (le long de z ou de la plus grande dimension, la fibre change peu).
            // Icebreaker F.37 : le marbre a des veines rares.
            Matiere::Marbre if bruit(x, y, z) % 13 == 0 => 70,
            Matiere::BoisClair | Matiere::BoisSombre => tons[(bruit(x, y, z.div_euclid(4)) as usize) % tons.len()],
            _ => tons[(bruit(x, y, z) as usize) % tons.len()],
        }
    }

    /// Pose un voxel (hors du modèle : ignoré).
    pub fn mettre(&mut self, p: P, m: Matiere) {
        if self.dedans(p) {
            self.voxels.insert(p, Self::ton(m, p));
        }
    }

    pub fn enlever(&mut self, p: P) { self.voxels.remove(&p); }

    /// Une boîte pleine, bornes comprises.
    pub fn boite(&mut self, a: P, b: P, m: Matiere) {
        for x in a.0.min(b.0)..=a.0.max(b.0) {
            for y in a.1.min(b.1)..=a.1.max(b.1) {
                for z in a.2.min(b.2)..=a.2.max(b.2) {
                    self.mettre((x, y, z), m);
                }
            }
        }
    }

    /// Vide une boîte, bornes comprises.
    pub fn creuser(&mut self, a: P, b: P) {
        for x in a.0.min(b.0)..=a.0.max(b.0) {
            for y in a.1.min(b.1)..=a.1.max(b.1) {
                for z in a.2.min(b.2)..=a.2.max(b.2) {
                    self.enlever((x, y, z));
                }
            }
        }
    }

    /// Un cylindre debout (axe z), de centre `(cx, cy)` (au demi-voxel près), de `z0` à `z1` compris.
    pub fn cylindre(&mut self, (cx, cy): (f32, f32), rayon: f32, z0: i32, z1: i32, m: Matiere) {
        self.cylindre_axe(Axe::Z, (cx, cy), rayon, z0, z1, m);
    }

    /// Un cylindre le long d'un axe ; `centre` est la position sur les deux autres axes (dans l'ordre x, y, z).
    pub fn cylindre_axe(&mut self, axe: Axe, centre: (f32, f32), rayon: f32, de: i32, a: i32, m: Matiere) {
        self.cyl(axe, centre, rayon, de, a, Some(m));
    }

    /// Vide un cylindre debout (l'intérieur d'une écuelle, d'une marmite).
    pub fn percer(&mut self, (cx, cy): (f32, f32), rayon: f32, z0: i32, z1: i32) {
        self.cyl(Axe::Z, (cx, cy), rayon, z0, z1, None);
    }

    fn cyl(&mut self, axe: Axe, centre: (f32, f32), rayon: f32, de: i32, a: i32, m: Option<Matiere>) {
        let r = rayon.ceil() as i32 + 1;
        for t in de.min(a)..=de.max(a) {
            for u in (centre.0 as i32 - r)..=(centre.0 as i32 + r) {
                for v in (centre.1 as i32 - r)..=(centre.1 as i32 + r) {
                    let (du, dv) = (u as f32 + 0.5 - centre.0, v as f32 + 0.5 - centre.1);
                    if du * du + dv * dv <= rayon * rayon {
                        match m {
                            Some(m) => self.mettre(axe.point(t, u, v), m),
                            None => self.enlever(axe.point(t, u, v)),
                        }
                    }
                }
            }
        }
    }

    /// Un ellipsoïde plein de centre `c` et de demi-axes `r` (en voxels).
    pub fn ellipsoide(&mut self, c: (f32, f32, f32), r: (f32, f32, f32), m: Matiere) {
        for x in (c.0 - r.0).floor() as i32..=(c.0 + r.0).ceil() as i32 {
            for y in (c.1 - r.1).floor() as i32..=(c.1 + r.1).ceil() as i32 {
                for z in (c.2 - r.2).floor() as i32..=(c.2 + r.2).ceil() as i32 {
                    let d = ((x as f32 + 0.5 - c.0) / r.0).powi(2)
                        + ((y as f32 + 0.5 - c.1) / r.1).powi(2)
                        + ((z as f32 + 0.5 - c.2) / r.2).powi(2);
                    if d <= 1.0 {
                        self.mettre((x, y, z), m);
                    }
                }
            }
        }
    }

    /// Une ligne de voxels de `a` à `b` compris.
    pub fn ligne(&mut self, a: P, b: P, m: Matiere) {
        let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max((b.2 - a.2).abs()).max(1);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let p = |u: i32, v: i32| (u as f32 + (v - u) as f32 * t).round() as i32;
            self.mettre((p(a.0, b.0), p(a.1, b.1), p(a.2, b.2)), m);
        }
    }

    /// Le fichier `.vox`.
    pub fn vox(&self) -> Vec<u8> {
        use dot_vox::{Color, DotVoxData, Model, Size, Voxel};
        let palette = (0..=255u8)
            .map(|i| {
                let [r, g, b] = self.couleur(i);
                Color { r, g, b, a: 255 }
            })
            .collect();
        let voxels = self
            .voxels
            .iter()
            .map(|((x, y, z), i)| Voxel { x: *x as u8, y: *y as u8, z: *z as u8, i: *i })
            .collect();
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
        let mut octets = Vec::new();
        data.write_vox(&mut octets).expect("écriture en mémoire");
        octets
    }

    /// L'image de contrôle : à gauche le modèle vu de trois quarts (depuis l'avant -y, la droite +x et le haut),
    /// à droite vu de face. Chaque voxel est un petit cube ombré.
    pub fn image(&self, px: u32) -> image::RgbImage {
        let (sx, sy, sz) = self.taille;
        let w = px as f32;
        // Trois quarts : écran x = (x + y) w, écran y = (x - y) w/2 - z w (le haut de l'image est petit).
        let proj = |x: f32, y: f32, z: f32| ((x + y) * w, (x - y) * w / 2.0 - z * w);
        let coins = [(0, 0, 0), (sx, 0, 0), (0, sy, 0), (sx, sy, 0), (0, 0, sz), (sx, 0, sz), (0, sy, sz), (sx, sy, sz)]
            .map(|(x, y, z)| proj(x as f32, y as f32, z as f32));
        let (min_x, max_x) = coins.iter().fold((f32::MAX, f32::MIN), |(a, b), c| (a.min(c.0), b.max(c.0)));
        let (min_y, max_y) = coins.iter().fold((f32::MAX, f32::MIN), |(a, b), c| (a.min(c.1), b.max(c.1)));
        let marge = 2.0 * w;
        let iso_l = max_x - min_x + 2.0 * marge;
        let iso_h = max_y - min_y + 2.0 * marge;
        let face_l = sx as f32 * w + marge;
        let face_h = sz as f32 * w + 2.0 * marge;
        let mut img = image::RgbImage::from_pixel(
            (iso_l + face_l) as u32,
            iso_h.max(face_h) as u32,
            image::Rgb([30, 30, 34]),
        );
        let ombre = |c: [u8; 3], f: f32| image::Rgb(c.map(|v| (v as f32 * f).min(255.0) as u8));
        let (ox, oy) = (marge - min_x, marge - min_y);
        let ecran = |x: i32, y: i32, z: i32| {
            let (a, b) = proj(x as f32, y as f32, z as f32);
            (a + ox, b + oy)
        };

        // Du plus loin au plus proche : la profondeur vers l'œil est x - y, puis le bas avant le haut.
        let mut tries: Vec<_> = self.voxels.iter().collect();
        tries.sort_by_key(|((x, y, z), _)| (x - y, *z, *x));
        for (&(x, y, z), &i) in tries {
            let c = self.couleur(i);
            let plein = |p: P| self.voxels.contains_key(&p);
            if !plein((x, y - 1, z)) {
                quad(&mut img, [ecran(x, y, z), ecran(x + 1, y, z), ecran(x + 1, y, z + 1), ecran(x, y, z + 1)], ombre(c, 0.8));
            }
            if !plein((x + 1, y, z)) {
                quad(&mut img, [ecran(x + 1, y, z), ecran(x + 1, y + 1, z), ecran(x + 1, y + 1, z + 1), ecran(x + 1, y, z + 1)], ombre(c, 0.6));
            }
            if !plein((x, y, z + 1)) {
                quad(&mut img, [ecran(x, y, z + 1), ecran(x + 1, y, z + 1), ecran(x + 1, y + 1, z + 1), ecran(x, y + 1, z + 1)], ombre(c, 1.05));
            }
        }

        // De face (depuis -y) : le premier voxel rencontré, assombri avec la profondeur.
        let (fx0, fy0, p) = (iso_l as i32, marge as i32, px as i32);
        for x in 0..sx {
            for z in 0..sz {
                if let Some(y) = (0..sy).find(|y| self.voxels.contains_key(&(x, *y, z))) {
                    let c = ombre(self.couleur(self.voxels[&(x, y, z)]), 1.0 - 0.5 * y as f32 / sy as f32);
                    for d in 0..p {
                        for e in 0..p {
                            put(&mut img, fx0 + x * p + d, fy0 + (sz - 1 - z) * p + e, c);
                        }
                    }
                }
            }
        }
        img
    }
}

fn put(img: &mut image::RgbImage, x: i32, y: i32, c: image::Rgb<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
        img.put_pixel(x as u32, y as u32, c);
    }
}

/// Remplit un quadrilatère convexe (coins dans l'ordre).
fn quad(img: &mut image::RgbImage, q: [(f32, f32); 4], c: image::Rgb<u8>) {
    let (x0, x1) = q.iter().fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.0), b.max(p.0)));
    let (y0, y1) = q.iter().fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.1), b.max(p.1)));
    for py in y0.floor() as i32..=y1.ceil() as i32 {
        for px in x0.floor() as i32..=x1.ceil() as i32 {
            let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
            let signes: Vec<f32> = (0..4)
                .map(|k| {
                    let (a, b) = (q[k], q[(k + 1) % 4]);
                    (b.0 - a.0) * (fy - a.1) - (b.1 - a.1) * (fx - a.0)
                })
                .collect();
            if signes.iter().all(|s| *s >= -0.01) || signes.iter().all(|s| *s <= 0.01) {
                put(img, px, py, c);
            }
        }
    }
}

/// L'axe d'un cylindre.
#[derive(Clone, Copy)]
pub enum Axe {
    X,
    Y,
    Z,
}

impl Axe {
    /// Le point dont la coordonnée le long de l'axe est `t`, et les deux autres `u`, `v` (dans l'ordre x, y, z).
    fn point(self, t: i32, u: i32, v: i32) -> P {
        match self {
            Axe::X => (t, u, v),
            Axe::Y => (u, t, v),
            Axe::Z => (u, v, t),
        }
    }
}

/// Tous nos modèles.
pub fn tous() -> Vec<Modele> { cuisine::modeles().into_iter().chain(trone::modeles()).chain(filons::modeles()).collect() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_modele_tient_dans_sa_taille_et_n_est_pas_vide() {
        for m in tous() {
            assert!(m.nombre_de_voxels() > 0, "{} est vide", m.id);
            assert!(m.taille.0 <= 256 && m.taille.1 <= 256 && m.taille.2 <= 256, "{} trop grand", m.id);
            for ((x, y, z), i) in m.voxels() {
                assert!(m.dedans((x, y, z)), "{} : voxel hors du modèle", m.id);
                assert!(!(16..22).contains(&i), "{} : indice creux {i}", m.id);
            }
        }
    }

    #[test]
    fn deux_lancements_donnent_les_memes_octets() {
        for (a, b) in tous().iter().zip(tous().iter()) {
            assert_eq!(a.vox(), b.vox(), "{}", a.id);
        }
    }

    #[test]
    fn le_vox_se_relit() {
        for m in tous() {
            let data = dot_vox::load_bytes(&m.vox()).expect("relecture");
            assert_eq!(data.models[0].voxels.len(), m.nombre_de_voxels(), "{}", m.id);
        }
    }

    #[test]
    fn les_identifiants_sont_uniques() {
        let mut ids: Vec<_> = tous().iter().map(|m| m.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), tous().len());
    }
}
