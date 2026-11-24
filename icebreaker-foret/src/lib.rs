//! Icebreaker: the shape of the forests (jalon FO,
//! `docs/icebreaker/PLAN-FORET.md`). From a seed and the settings of a profile
//! (`Forme`), `generer` grows a forest in a grid of blocks around its middle:
//! clearings linked by trails on the ground, and over them platforms around
//! the trunks, linked by bridges, with stairs and ladders to go up. One can
//! always walk from the clearing one arrives in to every clearing and every
//! platform: who falls from a bridge lands on the ground and climbs back.
//!
//! No dependency on Veloren, as `icebreaker-caverne` (whose noise and preset
//! writing it uses): the game and the workshop page (`web/index.html`, through
//! WebAssembly) run the same code.

pub mod grille;
pub mod maillage;
pub mod presets;
#[cfg(target_arch = "wasm32")] mod wasm;

pub use grille::Grille;
use grille::{BOIS, BUISSON, CLAIRIERE, ECHELLE, FEUILLES, HERBE, PLANCHE, RACINE, RAMBARDE, SENTE, SENTIER};
pub use icebreaker_caverne::Champ;
use icebreaker_caverne::bruit::{Alea, bruit2, bruit3};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, VecDeque},
    f32::consts::{PI, TAU},
};

/// Declares `Forme` (every setting is a number), its defaults and the table
/// the workshop builds its sliders from: key: default, min, max, step, label.
/// (The same as `forme!` of `icebreaker-caverne`.)
macro_rules! forme {
    ($( $groupe:literal { $( $cle:ident : $def:expr, $min:expr, $max:expr, $pas:expr, $lib:literal; )* } )*) => {
        /// The settings of a forest shape (a profile of
        /// `assets/icebreaker/forets/formes.ron`).
        #[derive(Clone, Debug, PartialEq, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct Forme { $($( pub $cle: f32, )*)* }

        impl Default for Forme {
            fn default() -> Self { Self { $($( $cle: $def, )*)* } }
        }

        pub const CHAMPS: &[Champ] = &[ $($( Champ {
            cle: stringify!($cle), groupe: $groupe, libelle: $lib, min: $min, max: $max, pas: $pas, defaut: $def,
        }, )*)* ];

        impl Forme {
            /// The settings in the order of `CHAMPS`.
            pub fn valeurs(&self) -> Vec<f32> { vec![ $($( self.$cle, )*)* ] }

            /// From values in the order of `CHAMPS` (the missing ones keep their default).
            pub fn depuis_valeurs(valeurs: &[f32]) -> Self {
                let mut forme = Self::default();
                let mut valeurs = valeurs.iter();
                $($( if let Some(v) = valeurs.next() { forme.$cle = *v; } )*)*
                forme
            }
        }
    };
}

forme! {
    "Forêt" {
        emprise: 90.0, 50.0, 200.0, 10.0, "Emprise (blocs de chaque côté du centre)";
        densite: 0.5, 0.0, 1.0, 0.01, "Densité des arbres";
        arbre_rayon: 3.0, 2.0, 6.0, 0.5, "Rayon des troncs";
        arbre_hauteur: 45.0, 25.0, 80.0, 1.0, "Hauteur des arbres";
        variation: 0.3, 0.0, 0.6, 0.01, "Variation des tailles";
        geants: 0.08, 0.0, 0.5, 0.01, "Part de très grands arbres";
        feuillage: 1.0, 0.4, 1.6, 0.05, "Ampleur du feuillage (moins : plus de lumière au sol)";
    }
    "Clairières" {
        clairieres: 5.0, 2.0, 12.0, 1.0, "Nombre de clairières";
        clairiere_rayon: 14.0, 8.0, 30.0, 1.0, "Rayon";
        clairiere_ecart: 20.0, 5.0, 80.0, 1.0, "Écart minimal entre deux clairières";
    }
    "Sentiers" {
        sentier_largeur: 2.0, 1.0, 4.0, 0.5, "Demi-largeur";
        sentier_detours: 0.4, 0.0, 1.0, 0.01, "Détours";
        boucles: 1.0, 0.0, 4.0, 1.0, "Boucles (sentiers en plus)";
    }
    "Étage haut" {
        porteurs: 0.35, 0.0, 1.0, 0.01, "Part d'arbres à plateforme";
        plateforme_hauteur: 16.0, 8.0, 40.0, 1.0, "Hauteur du premier niveau";
        niveaux: 2.0, 1.0, 3.0, 1.0, "Nombre de niveaux";
        niveau_ecart: 10.0, 7.0, 20.0, 1.0, "Écart entre deux niveaux";
        plateforme_rayon: 5.0, 4.0, 10.0, 1.0, "Largeur des plateformes autour du tronc";
    }
    "Ponts" {
        ponts: 0.6, 0.0, 1.0, 0.01, "Part de ponts (0 : aucun, 1 : tous les ponts possibles)";
        pont_longueur: 30.0, 8.0, 60.0, 1.0, "Longueur maximale";
        pont_largeur: 1.0, 0.0, 2.0, 1.0, "Demi-largeur (0 : une planche)";
        pont_pente: 0.25, 0.0, 0.5, 0.01, "Pente maximale";
        pont_creux: 1.5, 0.0, 4.0, 0.5, "Creux au milieu (pont de corde)";
        ponts_par_plateforme: 3.0, 1.0, 5.0, 1.0, "Ponts par plateforme, au plus";
        garde_corps: 1.0, 0.0, 1.0, 0.05, "Garde-corps (0 : aucun, 1 : continu)";
    }
    "Montées" {
        montees: 0.25, 0.0, 1.0, 0.01, "Part d'arbres à plateforme avec une montée depuis le sol";
        echelles: 0.3, 0.0, 1.0, 0.01, "Part d'échelles (le reste : escaliers en spirale)";
    }
    "Sous-bois" {
        buissons: 0.3, 0.0, 1.0, 0.01, "Buissons";
    }
}

impl Forme {
    /// Every setting within its bounds.
    pub fn bornee(&self) -> Self {
        let valeurs: Vec<f32> = self
            .valeurs()
            .iter()
            .zip(CHAMPS)
            .map(|(v, c)| if v.is_finite() { v.clamp(c.min, c.max) } else { c.defaut })
            .collect();
        Self::depuis_valeurs(&valeurs)
    }

    /// The settings as a block of `formes.ron`.
    pub fn vers_ron(&self) -> String {
        let mut text = String::from("(\n");
        let mut groupe = "";
        for (champ, valeur) in CHAMPS.iter().zip(self.valeurs()) {
            if champ.groupe != groupe {
                groupe = champ.groupe;
                text += &format!("    // {groupe}\n");
            }
            text += &format!("    {}: {:?},\n", champ.cle, valeur);
        }
        text + ")"
    }
}

/// A profile: a shape and its weight in the draw.
#[derive(Clone, Debug, Deserialize)]
pub struct Profil {
    pub poids: f32,
    pub forme: Forme,
}

/// `assets/icebreaker/forets/formes.ron`: the profiles, by name.
#[derive(Clone, Debug, Deserialize)]
pub struct Formes {
    pub profils: BTreeMap<String, Profil>,
}

impl Formes {
    pub fn lire(text: &str) -> Result<Self, String> {
        let formes: Self = ron::from_str(text).map_err(|e| format!("formes.ron illisible : {e}"))?;
        if formes.profils.is_empty() || formes.profils.values().all(|p| p.poids <= 0.0) {
            return Err("formes.ron : au moins un profil de poids > 0".into());
        }
        Ok(formes)
    }

    /// The profile of a forest: the one asked for, or one drawn by the seed.
    pub fn tirer(&self, graine: u64, voulu: Option<&str>) -> (&str, &Forme) {
        if let Some((nom, profil)) = voulu.and_then(|v| self.profils.get_key_value(v)) {
            return (nom, &profil.forme);
        }
        let total: f32 = self.profils.values().map(|p| p.poids.max(0.0)).sum();
        let mut tirage = Alea::new(graine ^ 0xf0e7).unite() * total;
        for (nom, profil) in &self.profils {
            tirage -= profil.poids.max(0.0);
            if tirage < 0.0 && profil.poids > 0.0 {
                return (nom, &profil.forme);
            }
        }
        let (nom, profil) = self.profils.iter().rfind(|(_, p)| p.poids > 0.0).expect("checked by lire");
        (nom, &profil.forme)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Genre {
    /// Where one arrives, at the edge of the forest.
    Arrivee,
    Camp,
    Nid,
    /// The farthest from the arrival along the trails: the chief.
    Fond,
}

#[derive(Clone, Debug)]
pub struct Clairiere {
    /// Its middle, a block one stands in.
    pub centre: [i32; 3],
    pub rayon: i32,
    pub genre: Genre,
    /// Blocks walked along the trails from the arrival.
    pub distance: f32,
}

#[derive(Clone, Debug)]
pub struct Arbre {
    /// The middle of the trunk, on the ground.
    pub pied: [i32; 3],
    pub rayon: i32,
    /// From the foot to the top of the trunk (the leaves go higher).
    pub hauteur: i32,
    pub geant: bool,
    /// Where one stands on each of its platforms, the lowest first (none: a
    /// plain tree). Two platforms of a tree are always linked.
    pub niveaux: Vec<i32>,
    /// How far its platforms go from the middle of the trunk.
    pub plateforme: i32,
    /// Whether one climbs to its first platform from the ground.
    pub montee: bool,
    /// Its ways up are ladders (stairs winding around the trunk otherwise).
    pub echelle: bool,
    /// Whether each platform is reached on foot from the arrival.
    pub atteints: Vec<bool>,
}

/// A bridge between the platforms `niveau` of two trees: where its deck starts
/// and ends (where the feet are).
#[derive(Clone, Debug)]
pub struct Pont {
    pub de: usize,
    pub vers: usize,
    pub niveau: usize,
    pub a: [f32; 3],
    pub b: [f32; 3],
}

/// A generated forest, in blocks around its middle (0, 0), z up; the height 0
/// is the one `surface` gives.
#[derive(Clone)]
pub struct Foret {
    pub grille: Grille,
    pub arbres: Vec<Arbre>,
    pub clairieres: Vec<Clairiere>,
    /// The trails between clearings, a point per block.
    pub sentiers: Vec<Vec<[f32; 2]>>,
    pub ponts: Vec<Pont>,
    /// Where one arrives: the middle of the first clearing.
    pub depart: [i32; 3],
    /// Whether every clearing and every platform is reached on foot from the
    /// arrival (so that one always climbs back after a fall).
    pub praticable: bool,
    /// Seeds tried for that (1: the first one was right).
    pub essais: u32,
    pub resume: String,
}

impl Foret {
    /// The clearing at the end of the forest.
    pub fn fond(&self) -> usize { self.clairieres.iter().position(|c| c.genre == Genre::Fond).unwrap_or(0) }

    pub fn plateformes(&self) -> usize { self.arbres.iter().map(|a| a.niveaux.len()).sum() }
}

/// Seeds tried for a forest one walks whole.
const ESSAIS: u64 = 12;
/// Blocks kept around the forest in the grid, for the leaves at its edge.
const MARGE: i32 = 12;
/// Blocks walked around the trunk for a block of height, on a stair.
const PAS: f32 = 2.0;

/// Generates the forest of `graine`. `surface(x, y)` is the height of the
/// ground (the lowest block over it) around the middle of the forest.
pub fn generer(forme: &Forme, graine: u64, surface: &dyn Fn(i32, i32) -> i32) -> Foret {
    let forme = forme.bornee();
    let mut derniere = None;
    for essai in 0..ESSAIS {
        let mut foret = essayer(&forme, graine.wrapping_add(essai.wrapping_mul(0x9e37_79b9_7f4a_7c15)), surface);
        foret.essais = essai as u32 + 1;
        if foret.praticable {
            return foret;
        }
        derniere = Some(foret);
    }
    derniere.expect("ESSAIS is at least 1")
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() }

/// How far `p` is from the segment `a`-`b`.
fn dist_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (ux, uy) = (b[0] - a[0], b[1] - a[1]);
    let t = (((p[0] - a[0]) * ux + (p[1] - a[1]) * uy) / (ux * ux + uy * uy).max(1e-6)).clamp(0.0, 1.0);
    dist(p, [a[0] + ux * t, a[1] + uy * t])
}

/// Whether the segments `a`-`b` and `c`-`d` cross away from their ends.
fn croisent(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    let cote = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
    cote(a, b, c) * cote(a, b, d) < 0.0 && cote(c, d, a) * cote(c, d, b) < 0.0
}

/// Every column within `r` of `c`.
fn disque(c: [f32; 2], r: f32, mut f: impl FnMut(i32, i32)) {
    let lim = (r + 0.5) * (r + 0.5);
    for y in (c[1] - r - 1.0).floor() as i32..=(c[1] + r + 1.0).ceil() as i32 {
        for x in (c[0] - r - 1.0).floor() as i32..=(c[0] + r + 1.0).ceil() as i32 {
            if (x as f32 - c[0]).powi(2) + (y as f32 - c[1]).powi(2) <= lim {
                f(x, y);
            }
        }
    }
}

fn xy(p: [i32; 3]) -> [f32; 2] { [p[0] as f32, p[1] as f32] }

/// Whether one stands in this block: room for the body, and something under
/// the feet (or a ladder to hold).
pub fn debout(g: &Grille, x: i32, y: i32, z: i32) -> bool {
    g.passable(x, y, z) && g.passable(x, y, z + 1) && (!g.passable(x, y, z - 1) || g.bloc(x, y, z) == ECHELLE)
}

/// The blocks one stands in, walking from `depart`: steps of one block up or
/// down, ladders climbed. Every move can be walked back.
pub struct Pieds(Vec<u64>);

impl Pieds {
    pub fn contient(&self, g: &Grille, p: [i32; 3]) -> bool {
        g.indice(p[0], p[1], p[2]).is_some_and(|i| self.0[i >> 6] >> (i & 63) & 1 == 1)
    }
}

pub fn marcher(g: &Grille, depart: [i32; 3]) -> Pieds {
    let mut vus = vec![0u64; g.nombre().div_ceil(64)];
    let mut file = VecDeque::new();
    let mut voir = |p: [i32; 3], file: &mut VecDeque<[i32; 3]>| {
        if let Some(i) = g.indice(p[0], p[1], p[2])
            && vus[i >> 6] >> (i & 63) & 1 == 0
        {
            vus[i >> 6] |= 1 << (i & 63);
            file.push_back(p);
        }
    };
    if debout(g, depart[0], depart[1], depart[2]) {
        voir(depart, &mut file);
    }
    while let Some([x, y, z]) = file.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for dz in [0, 1, -1] {
                let (nx, ny, nz) = (x + dx, y + dy, z + dz);
                // Room for the head while stepping up or down.
                let tete = match dz {
                    1 => g.passable(x, y, z + 2),
                    -1 => g.passable(nx, ny, z + 1),
                    _ => true,
                };
                if tete && debout(g, nx, ny, nz) {
                    voir([nx, ny, nz], &mut file);
                }
            }
        }
        if (g.bloc(x, y, z) == ECHELLE || g.bloc(x, y, z + 1) == ECHELLE) && debout(g, x, y, z + 1) {
            voir([x, y, z + 1], &mut file);
        }
        if g.bloc(x, y, z - 1) == ECHELLE && debout(g, x, y, z - 1) {
            voir([x, y, z - 1], &mut file);
        }
    }
    Pieds(vus)
}

/// A stair winding around the trunk from `z0` to `z1` (where one stands),
/// starting at the angle `depart`; the angle it ends at.
fn escalier(g: &mut Grille, arbre: &Arbre, z0: i32, z1: i32, depart: f32) -> f32 {
    let (cx, cy, r) = (arbre.pied[0], arbre.pied[1], arbre.rayon);
    let milieu = r as f32 + 1.5;
    let fin = (z1 - z0) as f32 * PAS / milieu;
    let (dedans, dehors) = ((r as f32 + 0.5).powi(2), (r as f32 + 2.5).powi(2));
    for dy in -(r + 3)..=r + 3 {
        for dx in -(r + 3)..=r + 3 {
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 <= dedans || d2 > dehors {
                continue;
            }
            let mut angle = ((dy as f32).atan2(dx as f32) - depart).rem_euclid(TAU);
            while angle <= fin + 0.3 {
                let zs = (z0 + (angle * milieu / PAS).floor() as i32).min(z1);
                g.poser(cx + dx, cy + dy, zs - 1, PLANCHE);
                // Room over the step: through the platform it comes up in, too.
                for z in zs..zs + 4 {
                    g.degager(cx + dx, cy + dy, z, true);
                }
                angle += TAU;
            }
        }
    }
    depart + fin
}

/// A ladder against the trunk, from `z0` to `z1` (where one stands).
fn echelle(g: &mut Grille, arbre: &Arbre, z0: Option<i32>, z1: i32, cote: (i32, i32)) {
    let (x, y) = (arbre.pied[0] + cote.0 * (arbre.rayon + 1), arbre.pied[1] + cote.1 * (arbre.rayon + 1));
    for z in z0.unwrap_or_else(|| g.sol(x, y))..=z1 {
        g.poser(x, y, z, ECHELLE);
    }
    g.degager(x, y, z1 + 1, true);
    g.degager(x, y, z1 + 2, true);
}

const COTES: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

fn essayer(f: &Forme, graine: u64, surface: &dyn Fn(i32, i32) -> i32) -> Foret {
    let mut rng = Alea::new(graine);
    let s = (graine as u32) ^ ((graine >> 32) as u32);
    let e = f.emprise;
    let bord = e as i32 + MARGE;
    let cote = (2 * bord + 1) as usize;
    let col = |x: i32, y: i32| (x.abs() <= bord && y.abs() <= bord).then(|| (y + bord) as usize * cote + (x + bord) as usize);
    let mut nature = vec![HERBE; cote * cote];
    // Where no tree nor bush grows: clearings, trails and around them.
    let mut degage = vec![false; cote * cote];

    // The clearings: the arrival at the edge, the others anywhere, apart.
    let mut ronds: Vec<([f32; 2], f32)> = Vec::new();
    let rayon_clairiere =
        |rng: &mut Alea| (f.clairiere_rayon * (1.0 + f.variation * rng.entre(-1.0, 1.0))).clamp(6.0, e * 0.35);
    let r0 = rayon_clairiere(&mut rng);
    let a0 = rng.entre(0.0, TAU);
    let d0 = e - r0 - 4.0;
    ronds.push(([d0 * a0.cos(), d0 * a0.sin()], r0));
    for _ in 1..f.clairieres as usize {
        for _ in 0..200 {
            let r = rayon_clairiere(&mut rng);
            let (d, a) = ((e - r - 4.0) * rng.unite().sqrt(), rng.entre(0.0, TAU));
            let p = [d * a.cos(), d * a.sin()];
            if ronds.iter().all(|(q, rq)| dist(p, *q) >= r + rq + f.clairiere_ecart) {
                ronds.push((p, r));
                break;
            }
        }
    }
    if ronds.len() < 2 {
        let (p, r) = ronds[0];
        ronds.push(([-p[0], -p[1]], r));
    }
    let n = ronds.len();

    // The trails: the shortest tree over the clearings, then a few loops.
    let mut aretes: Vec<(usize, usize)> = Vec::new();
    let mut reliee = vec![false; n];
    reliee[0] = true;
    for _ in 1..n {
        let mut best: Option<(f32, usize, usize)> = None;
        for i in (0..n).filter(|i| reliee[*i]) {
            for j in (0..n).filter(|j| !reliee[*j]) {
                let d = dist(ronds[i].0, ronds[j].0);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, i, j));
                }
            }
        }
        let (_, i, j) = best.expect("a clearing is left");
        reliee[j] = true;
        aretes.push((i, j));
    }
    let mut autres: Vec<(f32, usize, usize)> = (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
        .filter(|(i, j)| !aretes.contains(&(*i, *j)) && !aretes.contains(&(*j, *i)))
        .map(|(i, j)| (dist(ronds[i].0, ronds[j].0), i, j))
        .collect();
    autres.sort_by(|a, b| a.0.total_cmp(&b.0));
    aretes.extend(autres.iter().take(f.boucles as usize).map(|(_, i, j)| (*i, *j)));

    let mut sentiers = Vec::new();
    let mut longueurs = Vec::new();
    for (k, (i, j)) in aretes.iter().enumerate() {
        let (a, b) = (ronds[*i].0, ronds[*j].0);
        let l = dist(a, b).max(1.0);
        let (ux, uy) = ((b[0] - a[0]) / l, (b[1] - a[1]) / l);
        let amp = f.sentier_detours * (l * 0.3).min(30.0);
        let pas = l.ceil();
        let points: Vec<[f32; 2]> = (0..=pas as usize)
            .map(|q| {
                let t = q as f32 / pas;
                let o = amp * bruit2(t * l / 35.0 + k as f32 * 17.3, k as f32 * 5.1, s) * (PI * t).sin();
                let p = [a[0] + ux * l * t - uy * o, a[1] + uy * l * t + ux * o];
                // The trail stays in the forest.
                let d = dist(p, [0.0, 0.0]);
                if d > e { [p[0] * e / d, p[1] * e / d] } else { p }
            })
            .collect();
        longueurs.push(points.windows(2).map(|w| dist(w[0], w[1])).sum::<f32>());
        for p in &points {
            disque(*p, f.sentier_largeur + 2.0, |x, y| {
                if let Some(c) = col(x, y) {
                    degage[c] = true;
                }
            });
            disque(*p, f.sentier_largeur, |x, y| {
                if let Some(c) = col(x, y) {
                    nature[c] = SENTIER;
                }
            });
        }
        sentiers.push(points);
    }
    for (p, r) in &ronds {
        disque(*p, *r + 1.0, |x, y| {
            if let Some(c) = col(x, y) {
                degage[c] = true;
            }
        });
        disque(*p, *r, |x, y| {
            if let Some(c) = col(x, y) {
                nature[c] = CLAIRIERE;
            }
        });
    }

    // How far each clearing is from the arrival along the trails.
    let mut distances = vec![f32::INFINITY; n];
    distances[0] = 0.0;
    for _ in 0..n {
        for (k, (i, j)) in aretes.iter().enumerate() {
            distances[*j] = distances[*j].min(distances[*i] + longueurs[k]);
            distances[*i] = distances[*i].min(distances[*j] + longueurs[k]);
        }
    }
    let fond = (1..n).max_by(|a, b| distances[*a].total_cmp(&distances[*b])).unwrap_or(0);
    let mut clairieres: Vec<Clairiere> = (0..n)
        .map(|i| {
            let (p, r) = ronds[i];
            let (x, y) = (p[0].round() as i32, p[1].round() as i32);
            let genre = match i {
                0 => Genre::Arrivee,
                _ if i == fond => Genre::Fond,
                _ if rng.chance(0.5) => Genre::Nid,
                _ => Genre::Camp,
            };
            Clairiere { centre: [x, y, surface(x, y)], rayon: r as i32, genre, distance: distances[i] }
        })
        .collect();
    clairieres[0].genre = Genre::Arrivee;

    // The trees: thrown in the forest, apart from each other by the density,
    // out of the clearings and the trails.
    let mut arbres: Vec<Arbre> = Vec::new();
    let ecart = 16.0 - 14.0 * f.densite;
    let jets = ((4.0 * e * e / (ecart + 2.0 * f.arbre_rayon).powi(2)) * 8.0).min(30_000.0) as usize;
    for _ in 0..jets {
        let (d, a) = (e * rng.unite().sqrt(), rng.entre(0.0, TAU));
        let p = [(d * a.cos()).round(), (d * a.sin()).round()];
        let geant = rng.chance(f.geants);
        let taille = 1.0 + f.variation * rng.entre(-1.0, 1.0);
        let r = (f.arbre_rayon * taille * if geant { 1.7 } else { 1.0 }).round().clamp(2.0, 10.0);
        let h = (f.arbre_hauteur * taille * if geant { 1.4 } else { 1.0 }).round();
        if arbres.iter().any(|t| dist(p, xy(t.pied)) < r + t.rayon as f32 + ecart) {
            continue;
        }
        let mut libre = true;
        disque(p, r + 2.0, |x, y| libre &= col(x, y).is_some_and(|c| !degage[c]));
        if !libre {
            continue;
        }
        let (x, y) = (p[0] as i32, p[1] as i32);
        arbres.push(Arbre {
            pied: [x, y, surface(x, y)],
            rayon: r as i32,
            hauteur: h as i32,
            geant,
            niveaux: Vec::new(),
            plateforme: 0,
            montee: false,
            echelle: false,
            atteints: Vec::new(),
        });
    }

    // The trees that carry platforms, and how many levels each.
    for arbre in &mut arbres {
        let porte = rng.chance(f.porteurs);
        let etages = 1 + rng.indice(f.niveaux as usize);
        let echelle = rng.chance(f.echelles);
        if !porte {
            continue;
        }
        arbre.niveaux = (0..etages)
            .map(|k| arbre.pied[2] + (f.plateforme_hauteur + k as f32 * f.niveau_ecart) as i32)
            .collect();
        arbre.plateforme = arbre.rayon + f.plateforme_rayon as i32;
        arbre.hauteur = arbre.hauteur.max(arbre.niveaux[etages - 1] - arbre.pied[2] + 12);
        arbre.echelle = echelle;
    }
    let porteurs: Vec<usize> = (0..arbres.len()).filter(|i| !arbres[*i].niveaux.is_empty()).collect();

    // The bridges: between two platforms of the same level, close enough,
    // with no trunk in the way; the shortest first, and none crosses another.
    let demi = f.pont_largeur;
    let mut candidats: Vec<(f32, usize, usize, usize)> = Vec::new();
    for (a, &i) in porteurs.iter().enumerate() {
        for &j in &porteurs[a + 1..] {
            let (ti, tj) = (&arbres[i], &arbres[j]);
            let trou = dist(xy(ti.pied), xy(tj.pied)) - (ti.plateforme + tj.plateforme) as f32;
            if !(2.0..=f.pont_longueur).contains(&trou) {
                continue;
            }
            for k in 0..ti.niveaux.len().min(tj.niveaux.len()) {
                if (ti.niveaux[k] - tj.niveaux[k]).abs() as f32 <= f.pont_pente * trou {
                    candidats.push((trou, i, j, k));
                }
            }
        }
    }
    candidats.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut ponts: Vec<Pont> = Vec::new();
    let mut charge: Vec<[u8; 3]> = vec![[0; 3]; arbres.len()];
    let mut groupe: Vec<usize> = (0..arbres.len()).collect();
    fn racine(groupe: &mut [usize], mut i: usize) -> usize {
        while groupe[i] != i {
            groupe[i] = groupe[groupe[i]];
            i = groupe[i];
        }
        i
    }
    for (trou, i, j, k) in candidats {
        // Drawn for every candidate: moving a slider does not reshuffle the rest.
        let tirage = rng.unite();
        let (ti, tj) = (&arbres[i], &arbres[j]);
        let (pi, pj) = (xy(ti.pied), xy(tj.pied));
        if tirage >= f.ponts
            || charge[i][k].max(charge[j][k]) >= f.ponts_par_plateforme as u8
            || ponts.iter().any(|p| croisent(pi, pj, xy(arbres[p.de].pied), xy(arbres[p.vers].pied)))
            || arbres.iter().enumerate().any(|(t, autre)| {
                t != i && t != j && dist_segment(xy(autre.pied), pi, pj) < autre.rayon.max(autre.plateforme) as f32 + demi + 2.0
            })
        {
            continue;
        }
        let l = dist(pi, pj);
        let (ux, uy) = ((pj[0] - pi[0]) / l, (pj[1] - pi[1]) / l);
        let (da, db) = (ti.plateforme as f32 - 0.5, tj.plateforme as f32 - 0.5);
        ponts.push(Pont {
            de: i,
            vers: j,
            niveau: k,
            a: [pi[0] + ux * da, pi[1] + uy * da, ti.niveaux[k] as f32],
            b: [pj[0] - ux * db, pj[1] - uy * db, tj.niveaux[k] as f32],
        });
        let _ = trou;
        charge[i][k] += 1;
        charge[j][k] += 1;
        let (a, b) = (racine(&mut groupe, i), racine(&mut groupe, j));
        groupe[a] = b;
    }

    // The ways up from the ground: some by the draw, and one at least for each
    // group of platforms linked by bridges.
    for &i in &porteurs {
        arbres[i].montee = rng.chance(f.montees);
    }
    for &i in &porteurs {
        let g = racine(&mut groupe, i);
        if !porteurs.iter().any(|j| arbres[*j].montee && racine(&mut groupe, *j) == g) {
            arbres[i].montee = true;
        }
    }

    // The blocks. First the trunks, their roots and their leaves.
    let houppier = |t: &Arbre| (t.rayon as f32 * 1.6 + 4.0) * f.feuillage;
    let haut = arbres.iter().map(|t| t.pied[2] + t.hauteur + houppier(t) as i32 + 2).max().unwrap_or(0);
    let mut g = Grille::new(bord, surface, nature, haut);
    let mut tronc = vec![false; cote * cote];
    for t in &arbres {
        let (c, r) = (xy(t.pied), t.rayon as f32);
        disque(c, r + 1.0, |x, y| {
            if let Some(i) = col(x, y) {
                tronc[i] = true;
            }
        });
        for (large, hauteur) in [(2.0, 1), (1.0, 3)] {
            disque(c, r + large, |x, y| {
                for z in g.sol(x, y).min(t.pied[2])..t.pied[2] + hauteur {
                    g.poser(x, y, z, RACINE);
                }
            });
        }
        disque(c, r, |x, y| {
            for z in g.sol(x, y).min(t.pied[2])..=t.pied[2] + t.hauteur {
                g.poser(x, y, z, BOIS);
            }
        });
    }
    for t in &arbres {
        let cr = houppier(t);
        let sommet = [t.pied[0] as f32, t.pied[1] as f32, (t.pied[2] + t.hauteur) as f32];
        for boule in 0..3 + rng.indice(3) {
            let (a, d) = (rng.entre(0.0, TAU), if boule == 0 { 0.0 } else { cr * rng.entre(0.3, 0.8) });
            let r = cr * if boule == 0 { 1.0 } else { rng.entre(0.6, 0.9) };
            let c = [sommet[0] + d * a.cos(), sommet[1] + d * a.sin(), sommet[2] - cr * rng.entre(0.0, 0.5)];
            let lim = r.ceil() as i32 + 1;
            for dz in -lim..=lim {
                for dy in -lim..=lim {
                    for dx in -lim..=lim {
                        let (x, y, z) = (c[0] as i32 + dx, c[1] as i32 + dy, c[2] as i32 + dz);
                        // Flattened: wider than high.
                        let q = ((dx * dx + dy * dy) as f32 + (dz as f32 / 0.7).powi(2)).sqrt() / r;
                        if q < 1.15
                            && g.bloc(x, y, z) == grille::AIR
                            && (q < 0.75 || q < 1.0 + 0.15 * bruit3([x as f32 / 5.0, y as f32 / 5.0, z as f32 / 5.0], s))
                        {
                            g.poser(x, y, z, FEUILLES);
                        }
                    }
                }
            }
        }
    }

    // The platforms, then the decks of the bridges, then their rails (never
    // where one walks).
    let mut tablier = vec![false; cote * cote];
    for &i in &porteurs {
        let t = &arbres[i];
        for z in &t.niveaux {
            disque(xy(t.pied), t.plateforme as f32, |x, y| {
                g.poser(x, y, z - 1, PLANCHE);
                for dz in 0..4 {
                    g.degager(x, y, z + dz, false);
                }
                if let Some(c) = col(x, y) {
                    tablier[c] = true;
                }
            });
        }
    }
    // Where the feet are along a bridge: a point every third of a block.
    let le_long = |p: &Pont| {
        let l = dist([p.a[0], p.a[1]], [p.b[0], p.b[1]]).max(1.0);
        let creux = f.pont_creux.min(l / 8.0);
        let pas = (l * 3.0).ceil() as usize;
        let (a, b) = (p.a, p.b);
        (0..=pas).map(move |q| {
            let t = q as f32 / pas as f32;
            let z = a[2] + (b[2] - a[2]) * t - creux * 4.0 * t * (1.0 - t);
            ([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t], z.round() as i32, [(b[1] - a[1]) / l, (a[0] - b[0]) / l])
        })
    };
    for p in &ponts {
        for (c, z, travers) in le_long(p) {
            for o in (-2 * demi as i32)..=(2 * demi as i32) {
                let o = o as f32 / 2.0;
                let (x, y) = ((c[0] + travers[0] * o).round() as i32, (c[1] + travers[1] * o).round() as i32);
                g.poser(x, y, z - 1, PLANCHE);
                for dz in 0..4 {
                    g.degager(x, y, z + dz, false);
                }
                if let Some(i) = col(x, y) {
                    tablier[i] = true;
                }
            }
        }
    }
    if f.garde_corps > 0.0 {
        for p in &ponts {
            for (c, z, travers) in le_long(p) {
                for o in [-demi - 1.0, demi + 1.0] {
                    let (x, y) = ((c[0] + travers[0] * o).round() as i32, (c[1] + travers[1] * o).round() as i32);
                    // A post by block, kept by the setting.
                    let garde = ((x * 7 + y * 13).rem_euclid(20) as f32) < f.garde_corps * 20.0;
                    if garde && col(x, y).is_some_and(|i| !tablier[i]) && g.bloc(x, y, z - 1) == grille::AIR && g.bloc(x, y, z) == grille::AIR {
                        g.poser(x, y, z - 1, PLANCHE);
                        g.poser(x, y, z, RAMBARDE);
                    }
                }
            }
        }
    }

    // The ways up: from the ground where the tree has one, and between two
    // platforms of a tree, always.
    let mut pieds_de_montee: Vec<[i32; 2]> = Vec::new();
    for &i in &porteurs {
        let t = arbres[i].clone();
        let cote_echelle = COTES[rng.indice(4)];
        let mut angle = rng.entre(0.0, TAU);
        if t.montee {
            let (dx, dy) = if t.echelle { (cote_echelle.0 as f32, cote_echelle.1 as f32) } else { (angle.cos(), angle.sin()) };
            let d = t.rayon as f32 + 4.0;
            pieds_de_montee.push([t.pied[0] + (dx * d).round() as i32, t.pied[1] + (dy * d).round() as i32]);
            disque(xy(t.pied), t.rayon as f32 + 5.0, |x, y| {
                if let Some(c) = col(x, y) {
                    degage[c] = true;
                }
            });
        }
        let mut bas = t.montee.then_some(None);
        for z in &t.niveaux {
            if let Some(depuis) = bas {
                if t.echelle {
                    echelle(&mut g, &t, depuis, *z, cote_echelle);
                } else {
                    // From the ground: started lower, for a ground that slopes.
                    angle = escalier(&mut g, &t, depuis.unwrap_or(t.pied[2] - 2), *z, angle);
                }
            }
            bas = Some(Some(*z));
        }
    }

    // A thin trail from the trails to the foot of each way up, around the trunks.
    let mut parent = vec![u32::MAX; cote * cote];
    let mut file: VecDeque<usize> = (0..cote * cote)
        .filter(|c| matches!(g.nature((c % cote) as i32 - bord, (c / cote) as i32 - bord), SENTIER | CLAIRIERE))
        .collect();
    for c in &file {
        parent[*c] = *c as u32;
    }
    while let Some(c) = file.pop_front() {
        let (x, y) = ((c % cote) as i32 - bord, (c / cote) as i32 - bord);
        for (dx, dy) in COTES {
            if let Some(v) = col(x + dx, y + dy)
                && parent[v] == u32::MAX
                && !tronc[v]
            {
                parent[v] = c as u32;
                file.push_back(v);
            }
        }
    }
    for [x, y] in pieds_de_montee {
        // The foot, or a free column next to it.
        let depart = (0..9).filter_map(|k| col(x + k % 3 - 1, y + k / 3 - 1)).find(|c| parent[*c] != u32::MAX);
        let mut c = match depart {
            Some(c) => c,
            None => continue,
        };
        while parent[c] != c as u32 {
            let (x, y) = ((c % cote) as i32 - bord, (c / cote) as i32 - bord);
            if g.nature(x, y) == HERBE {
                g.marquer(x, y, SENTE);
            }
            for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(v) = col(x + dx, y + dy) {
                    degage[v] = true;
                }
            }
            c = parent[c] as usize;
        }
    }

    // The bushes, in patches.
    if f.buissons > 0.0 {
        for y in -bord..=bord {
            for x in -bord..=bord {
                let c = col(x, y).expect("in the box");
                if degage[c] || tronc[c] || dist([x as f32, y as f32], [0.0, 0.0]) > e {
                    continue;
                }
                let v = 0.5 + 0.5 * bruit2(x as f32 / 9.0, y as f32 / 9.0, s ^ 0xb015);
                let grain = (x * 31 + y * 17).rem_euclid(7);
                if v < f.buissons * 0.8 && grain != 0 {
                    let z = g.sol(x, y);
                    for dz in 0..1 + (grain > 4) as i32 {
                        if g.bloc(x, y, z + dz) == grille::AIR {
                            g.poser(x, y, z + dz, BUISSON);
                        }
                    }
                }
            }
        }
    }

    // On foot from the arrival: every clearing, every platform.
    let depart = clairieres[0].centre;
    let pieds = marcher(&g, depart);
    let mut manques = clairieres.iter().filter(|c| !pieds.contient(&g, c.centre)).count();
    for arbre in &mut arbres {
        let (c, r, p) = (arbre.pied, arbre.rayon + 3, arbre.plateforme);
        arbre.atteints = arbre
            .niveaux
            .iter()
            .map(|z| {
                // One block of its outer ring is enough: the ring goes around.
                let mut atteint = false;
                disque(xy(c), p as f32, |x, y| {
                    atteint |= (x - c[0]).pow(2) + (y - c[1]).pow(2) >= r * r && pieds.contient(&g, [x, y, *z]);
                });
                atteint
            })
            .collect();
        manques += arbre.atteints.iter().filter(|a| !**a).count();
    }
    let resume = format!(
        "{} arbres (dont {} très grands), {} clairières, {} plateformes, {} ponts, {} montées{}",
        arbres.len(),
        arbres.iter().filter(|t| t.geant).count(),
        clairieres.len(),
        arbres.iter().map(|t| t.niveaux.len()).sum::<usize>(),
        ponts.len(),
        arbres.iter().filter(|t| t.montee).count(),
        if manques == 0 { String::new() } else { format!(" ; {manques} lieu(x) COUPÉ(S) du reste") },
    );
    Foret {
        grille: g,
        arbres,
        clairieres,
        sentiers,
        ponts,
        depart,
        praticable: manques == 0,
        essais: 1,
        resume,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formes() -> Formes { Formes::lire(include_str!("../../assets/icebreaker/forets/formes.ron")).unwrap() }

    fn petite() -> Forme { Forme { emprise: 60.0, clairieres: 3.0, clairiere_rayon: 9.0, ..Forme::default() } }

    #[test]
    fn same_seed_same_forest() {
        let (a, b) = (generer(&petite(), 7, &|_, _| 0), generer(&petite(), 7, &|_, _| 0));
        assert!(a.grille == b.grille && a.resume == b.resume);
        assert!(generer(&petite(), 8, &|_, _| 0).grille != a.grille);
    }

    /// Every clearing and every platform of every profile is reached on foot.
    #[test]
    fn every_profile_is_walked() {
        for (nom, profil) in &formes().profils {
            for graine in 0..4 {
                let foret = generer(&profil.forme, graine, &|_, _| 0);
                assert!(foret.praticable, "{nom}, graine {graine} : {}", foret.resume);
                assert!(foret.clairieres.len() >= 2 && !foret.arbres.is_empty(), "{nom}, graine {graine}");
            }
        }
    }

    /// On a ground that slopes and rolls, too.
    #[test]
    fn a_forest_on_a_slope_is_walked() {
        let pente = |x: i32, y: i32| x / 12 + ((y as f32 / 15.0).sin() * 2.0) as i32;
        for graine in 0..4 {
            let foret = generer(&petite(), graine, &pente);
            assert!(foret.praticable, "graine {graine} : {}", foret.resume);
        }
    }

    #[test]
    fn the_upper_floor_follows_its_settings() {
        // No carrying tree: a forest on the ground only.
        let sol = generer(&Forme { porteurs: 0.0, ..petite() }, 3, &|_, _| 0);
        assert!(sol.praticable && sol.plateformes() == 0 && sol.ponts.is_empty());
        // No bridge: every tree with a platform has its own way up.
        let seuls = generer(&Forme { ponts: 0.0, porteurs: 0.6, ..petite() }, 3, &|_, _| 0);
        assert!(seuls.praticable && seuls.ponts.is_empty() && seuls.plateformes() > 0);
        assert!(seuls.arbres.iter().all(|t| t.niveaux.is_empty() || t.montee));
        // Every bridge: more of them, and fewer ways up are needed.
        let tous = generer(&Forme { ponts: 1.0, porteurs: 0.6, montees: 0.0, ..petite() }, 3, &|_, _| 0);
        let peu = generer(&Forme { ponts: 0.3, porteurs: 0.6, montees: 0.0, ..petite() }, 3, &|_, _| 0);
        assert!(tous.praticable && peu.praticable);
        assert!(tous.ponts.len() > peu.ponts.len());
        let montees = |f: &Foret| f.arbres.iter().filter(|t| t.montee).count();
        assert!(montees(&tous) < montees(&peu));
    }

    #[test]
    fn settings_round_trip() {
        let forme = Forme { ponts: 0.85, niveaux: 3.0, ..Forme::default() };
        let relue: Forme = ron::from_str(&forme.vers_ron()).unwrap();
        assert_eq!(relue, forme);
        assert_eq!(Forme::depuis_valeurs(&forme.valeurs()), forme);
        assert_eq!(Forme { emprise: 9000.0, densite: f32::NAN, ..Forme::default() }.bornee(), Forme {
            emprise: 200.0,
            ..Forme::default()
        });
    }
}
