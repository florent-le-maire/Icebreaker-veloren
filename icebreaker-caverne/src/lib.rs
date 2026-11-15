//! Icebreaker: the shape of the caves (jalon U,
//! `docs/icebreaker/PLAN-CAVERNE.md`). From a seed and the settings of a
//! profile (`Forme`), `generer` digs a cave in a grid of blocks around its
//! mouth: wide flat zones with stalactites, chaotic zones full of bumps and
//! holes, galleries and shafts between them, and a trail one can always walk
//! from the mouth to every zone.
//!
//! No dependency on Veloren: the game (`veloren-world`, the `Grotte` place)
//! and the workshop page (`web/index.html`, through WebAssembly) run the same
//! code.

pub mod bruit;
pub mod grille;
pub mod maillage;
#[cfg(target_arch = "wasm32")] mod wasm;

use bruit::{Alea, bruit2, bruit3, fbm3};
pub use grille::Grille;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    f32::consts::{PI, TAU},
};

/// A setting of `Forme`, as the workshop shows it.
pub struct Champ {
    pub cle: &'static str,
    pub groupe: &'static str,
    pub libelle: &'static str,
    pub min: f32,
    pub max: f32,
    pub pas: f32,
    pub defaut: f32,
}

/// Declares `Forme` (every setting is a number), its defaults and the table
/// the workshop builds its sliders from: key: default, min, max, step, label.
macro_rules! forme {
    ($( $groupe:literal { $( $cle:ident : $def:expr, $min:expr, $max:expr, $pas:expr, $lib:literal; )* } )*) => {
        /// The settings of a cave shape (a profile of
        /// `assets/icebreaker/grottes/formes.ron`).
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
    "Général" {
        zones: 8.0, 2.0, 16.0, 1.0, "Nombre de zones";
        verticalite: 0.3, 0.0, 1.0, 0.01, "Verticalité (0 : étalée, 1 : gouffre)";
        ecart: 0.0, -0.6, 1.5, 0.05, "Écart entre zones (négatif : elles se recouvrent)";
        variation: 0.25, 0.0, 0.6, 0.01, "Variation des tailles";
        part_plaine: 0.5, 0.0, 1.0, 0.01, "Part de zones plates (le reste : chaotiques)";
        niches: 1.0, 0.0, 3.0, 0.1, "Niches par zone (coffres)";
        boucles: 1.0, 0.0, 4.0, 1.0, "Boucles";
        emprise: 250.0, 80.0, 400.0, 10.0, "Emprise (blocs de chaque côté de l'entrée)";
        profondeur: 160.0, 40.0, 320.0, 10.0, "Profondeur maximale";
    }
    "Entrée" {
        entree_longueur: 30.0, 10.0, 80.0, 1.0, "Longueur de l'entrée";
        entree_profondeur: 18.0, 6.0, 60.0, 1.0, "Roche au-dessus de la première zone";
    }
    "Zone plate" {
        plaine_rayon: 50.0, 15.0, 110.0, 1.0, "Rayon";
        plaine_hauteur: 30.0, 5.0, 90.0, 1.0, "Hauteur sous plafond";
        plaine_contour: 0.3, 0.0, 0.7, 0.01, "Contour irrégulier";
        plaine_sol: 2.0, 0.0, 8.0, 0.5, "Bosses du sol";
        plaine_plafond: 8.0, 0.0, 24.0, 0.5, "Relief du plafond";
        plaine_stalactites: 0.5, 0.0, 1.0, 0.01, "Stalactites";
        plaine_stalagmites: 0.2, 0.0, 1.0, 0.01, "Stalagmites";
        plaine_piliers: 0.2, 0.0, 1.0, 0.01, "Piliers";
    }
    "Zone chaotique" {
        chaos_rayon: 30.0, 10.0, 70.0, 1.0, "Rayon";
        chaos_hauteur: 60.0, 10.0, 140.0, 1.0, "Hauteur";
        chaos_bruit: 0.7, 0.0, 1.5, 0.01, "Bosses et trous (0 : bulle lisse)";
        chaos_echelle: 16.0, 5.0, 40.0, 1.0, "Taille des bosses";
        chaos_detail: 0.5, 0.0, 1.0, 0.01, "Petit relief";
        chaos_stalactites: 0.2, 0.0, 1.0, 0.01, "Stalactites";
    }
    "Concrétions" {
        stalactite_longueur: 14.0, 2.0, 40.0, 1.0, "Longueur des stalactites";
    }
    "Galeries" {
        galerie_largeur: 8.0, 2.0, 16.0, 0.5, "Demi-largeur";
        galerie_hauteur: 18.0, 4.0, 40.0, 1.0, "Hauteur";
        galerie_sinuosite: 0.5, 0.0, 1.0, 0.01, "Sinuosité";
        galerie_bruit: 0.6, 0.0, 1.0, 0.01, "Parois irrégulières";
    }
    "Descentes" {
        puits_rayon: 14.0, 7.0, 30.0, 1.0, "Rayon des puits";
        puits_longueur: 25.0, 0.0, 80.0, 1.0, "Chute entre deux zones";
        puits_bruit: 0.5, 0.0, 1.0, 0.01, "Parois irrégulières";
        lacets_largeur: 44.0, 20.0, 80.0, 1.0, "Largeur des lacets (descente en pente)";
        puits_part: 0.2, 0.0, 1.0, 0.01, "Part des descentes en puits (le reste : pentes en lacets)";
    }
    "Sentier" {
        sentier_largeur: 2.0, 1.0, 4.0, 0.5, "Demi-largeur";
        sentier_pente: 2.0, 1.5, 4.0, 0.1, "Blocs parcourus pour un bloc de dénivelé";
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

/// `assets/icebreaker/grottes/formes.ron`: the profiles, by name. The seed of
/// a cave draws one: this is what makes a cave a pit, a plain or a maze.
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

    /// The profile of a cave: the one asked for, or one drawn by the seed.
    pub fn tirer(&self, graine: u64, voulu: Option<&str>) -> (&str, &Forme) {
        if let Some((nom, profil)) = voulu.and_then(|v| self.profils.get_key_value(v)) {
            return (nom, &profil.forme);
        }
        let total: f32 = self.profils.values().map(|p| p.poids.max(0.0)).sum();
        let mut tirage = Alea::new(graine ^ 0x70f1).unite() * total;
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
    /// Wide and low, a nearly flat floor, stalactites.
    Plaine,
    /// Bumps, holes and bridges of rock.
    Chaos,
    /// A small side pocket (a chest, a lone beast).
    Niche,
}

/// A zone of the cave: where it was dug (the walls wander around).
#[derive(Clone, Debug)]
pub struct Zone {
    pub centre: [i32; 3],
    /// Horizontal radii and half-height, before the noise.
    pub rayon: [i32; 3],
    pub genre: Genre,
    /// The zone it hangs on (none: the first one, on the entrance).
    pub parent: Option<usize>,
    /// Where the trail passes: a block one stands in.
    pub ancre: [i32; 3],
    /// Blocks walked along the trail from the mouth.
    pub distance: f32,
    /// The blocks one stands in here, all reached on foot from the mouth.
    pub sols: Vec<[i32; 3]>,
}

impl Zone {
    /// Whether `p` is in the zone (its shape before the noise, grown by `marge`).
    pub fn contient(&self, p: [i32; 3], marge: f32) -> bool {
        let q = [0, 1, 2].map(|i| (p[i] - self.centre[i]) as f32 / self.rayon[i].max(1) as f32);
        q[0] * q[0] + q[1] * q[1] + q[2] * q[2] < marge * marge
    }
}

/// A way between two zones (or from the mouth to the first one): the trail,
/// in a gallery or winding down a shaft.
#[derive(Clone, Debug)]
pub struct Lien {
    pub de: Option<usize>,
    pub vers: usize,
    /// The trail: where the feet are, a point every third of a block.
    pub points: Vec<[f32; 3]>,
    /// The shaft the trail winds down, if the way is steep: centre, radius,
    /// and the indices of the points where the trail enters and leaves it.
    pub puits: Option<([f32; 2], f32, usize, usize)>,
}

/// A generated cave, in blocks relative to its mouth (0, 0, 0: where one
/// stands at the surface, z up).
#[derive(Clone)]
pub struct Caverne {
    pub grille: Grille,
    pub zones: Vec<Zone>,
    pub liens: Vec<Lien>,
    /// Which way the entrance goes from the mouth (horizontal, unit).
    pub dir: [i32; 2],
    /// Where the trail starts, at the mouth.
    pub depart: [i32; 3],
    /// Whether every zone is reached on foot from the mouth, and back.
    pub praticable: bool,
    /// Seeds tried for a whole trail (1: the first one was right).
    pub essais: u32,
    pub resume: String,
}

impl Caverne {
    /// The zone at the end of the cave: the farthest from the mouth along the
    /// trail (niches aside).
    pub fn fond(&self) -> usize {
        (0..self.zones.len())
            .filter(|i| self.zones[*i].genre != Genre::Niche)
            .max_by(|a, b| self.zones[*a].distance.total_cmp(&self.zones[*b].distance))
            .unwrap_or(0)
    }

    /// How far a zone is in the cave: 0 at the entrance, 1 at the end.
    pub fn avancee(&self, zone: usize) -> f32 {
        let fond = self.zones[self.fond()].distance.max(1.0);
        (self.zones[zone].distance / fond).clamp(0.0, 1.0)
    }
}

/// Seeds tried for a cave whose trail is whole.
const ESSAIS: u64 = 12;

/// Rock kept over the top of a zone, under the surface.
const COUVERTURE: i32 = 6;

/// Generates the cave of `graine`. `surface(x, y)` is the height of the
/// ground around the mouth (0 at the mouth): the zones stay under it; where
/// the ground falls away from the mouth, the cave may go as much deeper than
/// `profondeur`, but never under `plancher` (the bottom of the world).
pub fn generer(forme: &Forme, graine: u64, surface: &dyn Fn(i32, i32) -> i32, plancher: i32) -> Caverne {
    let forme = forme.bornee();
    let mut dernier = None;
    for essai in 0..ESSAIS {
        let mut caverne = essayer(&forme, graine.wrapping_add(essai.wrapping_mul(0x9e37_79b9_7f4a_7c15)), surface, plancher);
        caverne.essais = essai as u32 + 1;
        if caverne.praticable {
            return caverne;
        }
        dernier = Some(caverne);
    }
    dernier.expect("ESSAIS is at least 1")
}

fn essayer(f: &Forme, graine: u64, surface: &dyn Fn(i32, i32) -> i32, plancher: i32) -> Caverne {
    let mut rng = Alea::new(graine);
    let s = (graine as u32) ^ ((graine >> 32) as u32);
    let e = f.emprise as i32;
    // How far the ground falls around the mouth: the cave may go as much deeper.
    let mut creux = 0;
    for x in (-e..=e).step_by(16) {
        for y in (-e..=e).step_by(16) {
            creux = creux.min(surface(x, y));
        }
    }
    let prof = (f.profondeur as i32 + (-creux).min(150)).min(-plancher.max(-100_000) - 8).max(24);
    let mut grille = Grille::new([-e, -e, -prof - 8], [e, e, 10]);

    // The entrance runs toward the highest ground (the cave stays under it).
    let dirs = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let l0 = f.entree_longueur as i32;
    let start = rng.indice(4);
    let dir = (0..4)
        .map(|i| dirs[(start + i) % 4])
        .max_by_key(|d| surface(d[0] * l0, d[1] * l0))
        .unwrap_or(dirs[0]);

    // How far a zone may reach past its radii (the noise of its walls).
    let debord = |genre: Genre| match genre {
        Genre::Plaine => [1.0 + f.plaine_contour, 1.0 + f.plaine_contour, 1.3],
        Genre::Chaos => [1.0 + f.chaos_bruit; 3],
        Genre::Niche => [1.3; 3],
    };
    let tailles = |rng: &mut Alea, genre: Genre| {
        let mut var = |v: f32| (v * (1.0 + f.variation * rng.entre(-1.0, 1.0))).round().max(3.0) as i32;
        match genre {
            Genre::Plaine => [var(f.plaine_rayon), var(f.plaine_rayon), var(f.plaine_hauteur / 2.0)],
            Genre::Chaos => [var(f.chaos_rayon), var(f.chaos_rayon), var(f.chaos_hauteur / 2.0)],
            Genre::Niche => [var(6.5), var(6.5), 4],
        }
    };
    // The lowest ground over a zone.
    let sous_terre = |centre: [i32; 3], rayon: [i32; 3]| {
        let mut low = i32::MAX;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
            low = low.min(surface(centre[0] + dx * rayon[0], centre[1] + dy * rayon[1]));
        }
        low - COUVERTURE - rayon[2]
    };
    let dedans = |centre: [i32; 3], rayon: [i32; 3], genre: Genre| {
        let d = debord(genre);
        let ext = [0, 1, 2].map(|i| (rayon[i] as f32 * d[i]).ceil() as i32 + 3);
        centre[0].abs() + ext[0] < e && centre[1].abs() + ext[1] < e && centre[2] - ext[2] > -prof - 6
    };
    let libre = |zones: &[Zone], sauf: usize, centre: [i32; 3], rayon: [i32; 3]| {
        zones.iter().enumerate().all(|(i, z)| {
            let d = ((z.centre[0] - centre[0]) as f32).hypot((z.centre[1] - centre[1]) as f32);
            i == sauf
                || d >= (z.rayon[0].max(z.rayon[1]) + rayon[0].max(rayon[1]) + 6) as f32
                || (z.centre[2] - centre[2]).abs() > z.rayon[2] + rayon[2] + 6
        })
    };
    let genre_tire = |rng: &mut Alea| if rng.chance(f.part_plaine) { Genre::Plaine } else { Genre::Chaos };

    // The first zone, at the end of the entrance: small enough to hold in the area.
    let genre = genre_tire(&mut rng);
    let mut rayon = tailles(&mut rng, genre);
    let most = (((e - l0 - 12) as f32 / (1.0 + 2.0 * debord(genre)[0])) as i32).max(6);
    rayon[0] = rayon[0].min(most);
    rayon[1] = rayon[1].min(most);
    let reach = l0 + (rayon[0] + rayon[1]) / 2;
    let mut centre = [dir[0] * reach, dir[1] * reach, -(f.entree_profondeur as i32) - rayon[2]];
    centre[2] = centre[2].min(sous_terre(centre, rayon));
    let mut zones = vec![Zone {
        centre,
        rayon,
        genre,
        parent: None,
        ancre: centre,
        distance: 0.0,
        sols: Vec::new(),
    }];
    // The directions of the ways out of each zone: they stay apart (two
    // trails that cross at different heights would cut each other).
    let mut azimuts: Vec<Vec<f32>> = vec![vec![(-dir[1] as f32).atan2(-dir[0] as f32)]];
    const ECART_ANGLE: f32 = 0.6;

    // The others: each hangs on a zone already there, beside it or under it
    // (`verticalite`), touching it or not (`ecart`).
    let count = f.zones.round() as usize;
    for _ in 1..count {
        for _ in 0..60 {
            let parent = if rng.chance(0.65) { zones.len() - 1 } else { rng.indice(zones.len()) };
            let genre = genre_tire(&mut rng);
            let rayon = tailles(&mut rng, genre);
            let p = &zones[parent];
            let pitch = (f.verticalite * 90.0 + rng.entre(-20.0, 20.0)).clamp(0.0, 90.0).to_radians();
            let az = rng.entre(0.0, TAU);
            let horiz = pitch.cos() * ((p.rayon[0] + p.rayon[1] + rayon[0] + rayon[1]) as f32 / 2.0) * (1.0 + f.ecart);
            let mut drop = pitch.sin()
                * ((p.rayon[2] + rayon[2]) as f32 * (1.0 + f.ecart.max(0.0)) + f.puits_longueur * rng.entre(0.6, 1.4));
            if pitch < 0.2 && rng.chance(0.35) {
                drop = -drop;
            }
            // Icebreaker U.6: far enough aside for a slope with switchbacks,
            // most of the time; otherwise right under, down a shaft.
            let lacets = drop.abs() > 12.0 && !rng.chance(f.puits_part);
            let horiz = if lacets { horiz.max(1.1 * drop.abs() + 24.0) } else { horiz };
            let mut centre = [
                p.centre[0] + (az.cos() * horiz).round() as i32,
                p.centre[1] + (az.sin() * horiz).round() as i32,
                p.centre[2] - drop.round() as i32,
            ];
            centre[2] = centre[2].min(sous_terre(centre, rayon));
            // Under its parent, it has no direction: a shaft will choose one.
            let aside = horiz >= 10.0;
            if dedans(centre, rayon, genre)
                && libre(&zones, parent, centre, rayon)
                && (!aside || ecart_angle(&azimuts[parent], az) >= ECART_ANGLE)
            {
                zones.push(Zone {
                    centre,
                    rayon,
                    genre,
                    parent: Some(parent),
                    ancre: centre,
                    distance: 0.0,
                    sols: Vec::new(),
                });
                if aside {
                    azimuts[parent].push(az);
                }
                azimuts.push(if aside { vec![az + PI] } else { Vec::new() });
                break;
            }
        }
    }
    // Loops: between the closest zones not linked yet.
    let mut boucles: Vec<(usize, usize)> = Vec::new();
    let mut paires: Vec<(i32, usize, usize)> = Vec::new();
    for a in 0..zones.len() {
        for b in a + 1..zones.len() {
            if zones[a].parent == Some(b) || zones[b].parent == Some(a) {
                continue;
            }
            let d = [0, 1, 2].map(|i| (zones[a].centre[i] - zones[b].centre[i]).abs());
            let near = (zones[a].rayon[0].max(zones[a].rayon[1]) + zones[b].rayon[0].max(zones[b].rayon[1])) * 2;
            if d[0].max(d[1]) < near {
                paires.push((d[0] + d[1] + d[2], a, b));
            }
        }
    }
    paires.sort();
    for (_, a, b) in paires {
        let d = [0, 1].map(|i| (zones[b].centre[i] - zones[a].centre[i]) as f32);
        let az = d[1].atan2(d[0]);
        if boucles.len() < f.boucles.round() as usize
            && ecart_angle(&azimuts[a], az) >= ECART_ANGLE
            && ecart_angle(&azimuts[b], az + PI) >= ECART_ANGLE
        {
            azimuts[a].push(az);
            azimuts[b].push(az + PI);
            boucles.push((a, b));
        }
    }
    // Niches: small pockets beside the zones.
    let principales = zones.len();
    for parent in 0..principales {
        let n = f.niches.floor() as usize + rng.chance(f.niches.fract()) as usize;
        for _ in 0..n {
            for _ in 0..12 {
                let rayon = tailles(&mut rng, Genre::Niche);
                let p = &zones[parent];
                let az = rng.entre(0.0, TAU);
                let d = (p.rayon[0] + p.rayon[1]) as f32 / 2.0 * debord(p.genre)[0].min(1.3)
                    + rayon[0] as f32
                    + rng.entre(6.0, 14.0);
                let mut centre = [
                    p.centre[0] + (az.cos() * d).round() as i32,
                    p.centre[1] + (az.sin() * d).round() as i32,
                    p.centre[2] - p.rayon[2] + rayon[2] + rng.entre(-3.0, 3.0).round() as i32,
                ];
                centre[2] = centre[2].min(sous_terre(centre, rayon));
                if dedans(centre, rayon, Genre::Niche)
                    && libre(&zones, parent, centre, rayon)
                    && ecart_angle(&azimuts[parent], az) >= ECART_ANGLE
                {
                    zones.push(Zone {
                        centre,
                        rayon,
                        genre: Genre::Niche,
                        parent: Some(parent),
                        ancre: centre,
                        distance: 0.0,
                        sols: Vec::new(),
                    });
                    azimuts[parent].push(az);
                    azimuts.push(vec![az + PI]);
                    break;
                }
            }
        }
    }

    // Dig the zones, then find where the trail passes in each.
    for zone in &zones {
        match zone.genre {
            Genre::Plaine => creuser_plaine(&mut grille, zone, f, s),
            Genre::Chaos => creuser_chaos(&mut grille, zone, f.chaos_bruit, f.chaos_echelle, f.chaos_detail, s),
            Genre::Niche => creuser_chaos(&mut grille, zone, 0.3, 7.0, 0.5, s),
        }
    }
    for zone in &mut zones {
        zone.ancre = ancre(&grille, zone);
    }

    // The ways: from the mouth to the first zone, from each zone to its
    // parent, and the loops.
    let reel = |p: [i32; 3]| p.map(|e| e as f32);
    let bouche = [-2.0 * dir[0] as f32, -2.0 * dir[1] as f32, 0.0];
    let mut liens = Vec::new();
    let vers = |p: [f32; 3], c: [f32; 2]| (c[1] - p[1]).atan2(c[0] - p[0]);
    for (i, zone) in zones.iter().enumerate() {
        let (de, a) = match zone.parent {
            Some(parent) => (Some(parent), reel(zones[parent].ancre)),
            None => (None, bouche),
        };
        let b = reel(zone.ancre);
        let (points, puits) = tracer(a, b, f, &mut rng, de.map_or(&[], |p| &azimuts[p]), &azimuts[i]);
        if let Some((c, ..)) = puits {
            if let Some(parent) = de {
                azimuts[parent].push(vers(a, c));
            }
            azimuts[i].push(vers(b, c));
        }
        liens.push(Lien { de, vers: i, points, puits });
    }
    for i in 0..zones.len() {
        let from = zones[i].parent.map_or(0.0, |p| zones[p].distance);
        zones[i].distance = from + longueur(&liens[i].points);
    }
    for (a, b) in boucles {
        let (pa, pb) = (reel(zones[a].ancre), reel(zones[b].ancre));
        let (points, puits) = tracer(pa, pb, f, &mut rng, &azimuts[a], &azimuts[b]);
        if let Some((c, ..)) = puits {
            azimuts[a].push(vers(pa, c));
            azimuts[b].push(vers(pb, c));
        }
        liens.push(Lien { de: Some(a), vers: b, points, puits });
    }
    for lien in &liens {
        let (debut, fin) = lien.puits.map_or((lien.points.len(), lien.points.len()), |(_, _, debut, fin)| (debut, fin));
        // Switchbacks (a way much longer than it goes far): dug high, the legs open on each other as ledges on a slope.
        let droit = lien.points.first().zip(lien.points.last()).map_or(0.0, |(p, q)| (q[0] - p[0]).hypot(q[1] - p[1]));
        let ample = if lien.puits.is_none() && longueur(&lien.points) > 1.6 * droit { 2.2 } else { 1.0 };
        creuser_galerie(&mut grille, &lien.points[..debut], f, s, surface, ample);
        creuser_galerie(&mut grille, &lien.points[fin..], f, s, surface, ample);
        if let Some((centre, rayon, ..)) = lien.puits {
            let zs = lien.points[debut..fin].iter().map(|p| p[2]);
            let (low, high) = zs.fold((f32::MAX, f32::MIN), |(l, h), z| (l.min(z), h.max(z)));
            // Under the ground: a shaft near the mouth does not open to the sky.
            let haut = (high.ceil() as i32 + 5).min(surface(centre[0] as i32, centre[1] as i32) - 3);
            creuser_puits(&mut grille, centre, rayon, low.floor() as i32, haut, f, s);
        }
    }

    // Concretions, then the trail over everything: it is always whole.
    for zone in &zones {
        concretions(&mut grille, zone, f, &mut rng);
    }
    for lien in &liens {
        poser_sentier(&mut grille, &lien.points, f.sentier_largeur);
    }
    // A way that leaves a zone lowers or raises the trail around its start by a
    // block or two: the mouth and the anchors are where one stands there now.
    let poser = |g: &Grille, p: [i32; 3]| {
        [0, -1, 1, -2, 2, -3, 3].iter().map(|dz| [p[0], p[1], p[2] + dz]).find(|q| debout(g, q[0], q[1], q[2])).unwrap_or(p)
    };
    let depart = poser(&grille, [bouche[0] as i32, bouche[1] as i32, 0]);
    for zone in &mut zones {
        zone.ancre = poser(&grille, zone.ancre);
    }
    grille.garder_relie(depart);

    // What one reaches on foot, and back: steps of one block, both ways.
    let pieds = marcher(&grille, depart);
    let praticable = zones.iter().all(|z| pieds.binary_search(&z.ancre).is_ok());
    for p in &pieds {
        if let Some(zone) = zones.iter_mut().find(|z| z.ancre == *p || z.contient(*p, 1.25)) {
            zone.sols.push(*p);
        }
    }

    let n = |genre: Genre| zones.iter().filter(|z| z.genre == genre).count();
    let bas = zones.iter().map(|z| z.centre[2] - z.rayon[2]).min().unwrap_or(0);
    let large = zones
        .iter()
        .map(|z| (z.centre[0].abs() + z.rayon[0]).max(z.centre[1].abs() + z.rayon[1]))
        .max()
        .unwrap_or(0);
    let resume = format!(
        "{} zone(s) plate(s), {} chaotique(s), {} niche(s), {} puits ; dénivelé {} blocs, étendue {} blocs, {} blocs \
         d'air, {} sols à pied ; sentier {}",
        n(Genre::Plaine),
        n(Genre::Chaos),
        n(Genre::Niche),
        liens.iter().filter(|l| l.puits.is_some()).count(),
        -bas,
        large,
        grille.volume(),
        pieds.len(),
        if praticable { "entier" } else { "COUPÉ" },
    );
    Caverne {
        grille,
        zones,
        liens,
        dir,
        depart,
        praticable,
        essais: 1,
        resume,
    }
}

/// A wide low zone: an irregular outline, a floor with gentle bumps that
/// rises toward the rim, a ceiling with its own relief.
fn creuser_plaine(g: &mut Grille, zone: &Zone, f: &Forme, s: u32) {
    let [rx, ry, h] = zone.rayon.map(|e| e as f32);
    let [cx, cy, cz] = zone.centre;
    let rm = (rx + ry) / 2.0;
    let ext = 1.0 + f.plaine_contour;
    let base = (cz - zone.rayon[2]) as f32;
    let (wx, wy) = ((rx * ext) as i32 + 2, (ry * ext) as i32 + 2);
    for y in cy - wy..=cy + wy {
        for x in cx - wx..=cx + wx {
            let (fx, fy) = (x as f32, y as f32);
            let wander = 0.7 * bruit2(fx / (rm * 0.5), fy / (rm * 0.5), s.wrapping_add(11))
                + 0.3 * bruit2(fx / (rm * 0.15), fy / (rm * 0.15), s.wrapping_add(12));
            let edge = (((x - cx) as f32 / rx).hypot((y - cy) as f32 / ry) + f.plaine_contour * wander).max(0.0);
            if edge >= 1.0 {
                continue;
            }
            let open = (1.0 - edge.powi(4)).sqrt();
            let sol = base + f.plaine_sol * 0.5 * bruit2(fx / 10.0, fy / 10.0, s.wrapping_add(13)) + (1.0 - open) * h * 0.7;
            let relief = bruit2(fx / 16.0, fy / 16.0, s.wrapping_add(14)) + 0.5 * bruit2(fx / 5.0, fy / 5.0, s.wrapping_add(15));
            let plafond = base + 2.0 * h * open + f.plaine_plafond * 0.5 * relief;
            for z in sol.round() as i32 + 1..=plafond.round() as i32 {
                g.creuser(x, y, z);
            }
        }
    }
}

/// A bubble whose walls wander with a 3D noise: with a strong noise, bumps,
/// holes and bridges all through it. The noise is flattened: it leaves ledges.
fn creuser_chaos(g: &mut Grille, zone: &Zone, bruit: f32, echelle: f32, detail: f32, s: u32) {
    let r = zone.rayon.map(|e| e as f32);
    let [cx, cy, cz] = zone.centre;
    let rmin = r[0].min(r[1]).min(r[2]);
    let a = bruit * rmin;
    let w = r.map(|e| (e * (1.0 + bruit)) as i32 + 1);
    for z in cz - w[2]..=cz + w[2] {
        for y in cy - w[1]..=cy + w[1] {
            for x in cx - w[0]..=cx + w[0] {
                let q = [(x - cx) as f32 / r[0], (y - cy) as f32 / r[1], (z - cz) as f32 / r[2]];
                let d = ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 1.0) * rmin;
                if d > a {
                    continue;
                }
                let p = [x as f32 / echelle, y as f32 / echelle, z as f32 / (echelle * 0.6)];
                if d < -a || d + a * fbm3(p, s.wrapping_add(21), detail) < 0.0 {
                    g.creuser(x, y, z);
                }
            }
        }
    }
}

/// A shaft: a vertical cylinder with wandering walls.
fn creuser_puits(g: &mut Grille, centre: [f32; 2], rayon: f32, bas: i32, haut: i32, f: &Forme, s: u32) {
    let a = f.puits_bruit * rayon * 0.5;
    let w = (rayon + a) as i32 + 1;
    let (cx, cy) = (centre[0].round() as i32, centre[1].round() as i32);
    for z in bas..=haut {
        for y in cy - w..=cy + w {
            for x in cx - w..=cx + w {
                let d = (x as f32 - centre[0]).hypot(y as f32 - centre[1]) - rayon;
                if d > a {
                    continue;
                }
                let p = [x as f32 / 12.0, y as f32 / 12.0, z as f32 / 20.0];
                if d < -a || d + a * fbm3(p, s.wrapping_add(31), 0.5) < 0.0 {
                    g.creuser(x, y, z);
                }
            }
        }
    }
}

/// A gallery along a trail: wider than high, its width changing along the
/// way, its walls roughened by the noise.
fn creuser_galerie(g: &mut Grille, points: &[[f32; 3]], f: &Forme, s: u32, surface: &dyn Fn(i32, i32) -> i32, ample: f32) {
    let hg = f.galerie_hauteur / 2.0 * ample;
    for (i, p) in points.iter().enumerate().step_by(4) {
        let w = f.galerie_largeur * (0.75 + 0.5 * bruit2(i as f32 / 60.0, 0.0, s.wrapping_add(41)));
        let c = [p[0], p[1], p[2] + hg - 1.0];
        let (wi, hi) = (w as i32 + 1, hg as i32 + 1);
        let (cx, cy, cz) = (c[0].round() as i32, c[1].round() as i32, c[2].round() as i32);
        for z in cz - hi..=cz + hi {
            for y in cy - wi..=cy + wi {
                for x in cx - wi..=cx + wi {
                    let q = [(x as f32 - c[0]) / w, (y as f32 - c[1]) / w, (z as f32 - c[2]) / hg];
                    let l = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
                    if l >= 1.0 {
                        continue;
                    }
                    let rough = bruit3([x as f32 / 7.0, y as f32 / 7.0, z as f32 / 7.0], s.wrapping_add(42));
                    // Under the ground: near the mouth, only the trail cuts the surface.
                    if l + f.galerie_bruit * 0.45 * rough < 1.0 && z < surface(x, y) - 1 {
                        g.creuser(x, y, z);
                    }
                }
            }
        }
    }
}

/// Where the trail passes in a zone: on its floor, as near its centre as
/// there is air.
fn ancre(g: &Grille, zone: &Zone) -> [i32; 3] {
    let [cx, cy, cz] = zone.centre;
    let h = zone.rayon[2];
    let rings = zone.rayon[0].min(zone.rayon[1]) / 4;
    for ring in 0..=rings {
        let r = ring * 2;
        let spots: &[(i32, i32)] = if ring == 0 {
            &[(0, 0)]
        } else {
            &[(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)]
        };
        for (dx, dy) in spots {
            let (x, y) = (cx + dx * r, cy + dy * r);
            for dz in 0..=h {
                for z in [cz - dz, cz + dz] {
                    if g.air(x, y, z) {
                        let mut feet = z;
                        while g.air(x, y, feet - 1) && feet > cz - h - 2 {
                            feet -= 1;
                        }
                        return [x, y, feet];
                    }
                }
            }
        }
    }
    zone.centre
}

fn longueur(points: &[[f32; 3]]) -> f32 { points.windows(2).map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1])).sum() }

/// Gives the points of a flat path their height: level over `paliers` blocks at
/// each end (the ways that meet at an anchor meet level), an even slope from
/// `za` to `zb` between.
fn monter(plat: Vec<[f32; 2]>, za: f32, zb: f32, paliers: (f32, f32)) -> Vec<[f32; 3]> {
    let mut along = 0.0;
    let mut alongs = Vec::with_capacity(plat.len());
    for (i, p) in plat.iter().enumerate() {
        if i > 0 {
            along += (p[0] - plat[i - 1][0]).hypot(p[1] - plat[i - 1][1]);
        }
        alongs.push(along);
    }
    let slope = (along - paliers.0 - paliers.1).max(0.001);
    plat.iter()
        .zip(alongs)
        .map(|(p, l)| [p[0], p[1], za + (zb - za) * ((l - paliers.0) / slope).clamp(0.0, 1.0)])
        .collect()
}

/// The smallest angle between `az` and the directions of `autour`.
fn ecart_angle(autour: &[f32], az: f32) -> f32 {
    autour
        .iter()
        .map(|o| {
            let g = (az - o).rem_euclid(TAU);
            g.min(TAU - g)
        })
        .fold(PI, f32::min)
}

/// A trail, and the shaft it winds down if any: its centre, its radius, and
/// the indices of the points where the trail enters and leaves it.
type Trace = (Vec<[f32; 3]>, Option<([f32; 2], f32, usize, usize)>);

/// The trail from `a` to `b`, never steeper than `sentier_pente`: a winding
/// way if it is long enough for the drop. Otherwise it walks to a shaft dug
/// aside, winds down along its wall, and walks from its foot to `b`. The
/// shaft stands away from both ends, in the direction the freest of the
/// other ways out of them (`autour_a`, `autour_b`): two trails that cross at
/// different heights would cut each other.
fn tracer(a: [f32; 3], b: [f32; 3], f: &Forme, rng: &mut Alea, autour_a: &[f32], autour_b: &[f32]) -> Trace {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let d = dx.hypot(dy);
    let palier = f.sentier_largeur + 3.0;
    let besoin = f.sentier_pente * (b[2] - a[2]).abs() + 1.0;
    let (a1, a2) = (rng.entre(-0.35, 0.35), rng.entre(-0.35, 0.35));
    let sens = if rng.chance(0.5) { 1.0 } else { -1.0 };
    let tour = rng.entre(0.0, TAU);
    let etendue = |plat: &[[f32; 2]]| -> f32 { plat.windows(2).map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1])).sum() };
    if d >= 4.0 {
        let n = (d * 3.0).ceil() as usize;
        for amp in [1.0, 1.7, 2.6] {
            // It leaves and arrives straight: the ways out of a zone stay apart.
            let plat: Vec<[f32; 2]> = (0..=n)
                .map(|i| {
                    let t = i as f32 / n as f32;
                    let o = d * f.galerie_sinuosite * amp * (PI * t).sin().powi(2) * (a1 + a2 * (TAU * t).sin());
                    [a[0] + dx * t - dy / d * o, a[1] + dy * t + dx / d * o]
                })
                .collect();
            if etendue(&plat) - 2.0 * palier >= besoin {
                return (monter(plat, a[2], b[2], (palier, palier)), None);
            }
        }
    }
    // Steeper: switchbacks down a slope (Icebreaker U.6), the trail swinging
    // from side to side as it goes toward `b`. Each turn is level: there the
    // two legs run side by side, and would cut each other if they sloped.
    // Between the turns the legs are `espacement` apart, one above the other
    // as the ledges of a cliff.
    let espacement = 2.0 * f.sentier_largeur + 8.0;
    const VIRAGE: f32 = 0.75;
    // Narrow first; wider if the drop asks for it.
    let essais = (2..=((d / espacement) as usize).min(12)).flat_map(|n| [(n, f.lacets_largeur), (n, f.lacets_largeur * 1.5)]);
    for (n, largeur) in essais {
        let wave = |t: f32| (n as f32 * PI * t).sin();
        let count = ((n as f32 * largeur + d) * 3.0).ceil() as usize;
        let plat: Vec<[f32; 2]> = (0..=count)
            .map(|i| {
                let t = i as f32 / count as f32;
                // Straight out of the zones (their other ways are near), then
                // the full swing; a slight wander along the legs.
                let swing = ((t.min(1.0 - t) * d - 8.0) / 14.0).clamp(0.0, 1.0);
                let o = sens * largeur / 2.0 * wave(t) * swing * (1.0 + 0.25 * a1 * (7.0 * PI * t).sin());
                [a[0] + dx * t - dy / d * o, a[1] + dy * t + dx / d * o]
            })
            .collect();
        // The slope only where the legs are apart: lengths walked on a slope.
        let mut pente = vec![0.0; plat.len()];
        for i in 1..plat.len() {
            let step = (plat[i][0] - plat[i - 1][0]).hypot(plat[i][1] - plat[i - 1][1]);
            let turn = wave(i as f32 / count as f32).abs() > VIRAGE;
            pente[i] = pente[i - 1] + if turn { 0.0 } else { step };
        }
        let total = pente[plat.len() - 1] - 2.0 * palier;
        if total >= besoin {
            let points = plat
                .iter()
                .zip(&pente)
                .map(|(p, l)| [p[0], p[1], a[2] + (b[2] - a[2]) * ((l - palier) / total).clamp(0.0, 1.0)])
                .collect();
            return (points, None);
        }
    }
    // Too steep: a shaft, `recul` blocks away from the ends of the trail.
    let rh = (f.puits_rayon - f.sentier_largeur - 2.0).max(4.0);
    let recul = rh + palier + 4.0;
    let c = if d > 2.0 * recul + 4.0 {
        // Far apart: on the way, near `b`.
        [b[0] - dx / d * recul, b[1] - dy / d * recul]
    } else {
        // Nearly one above the other: aside, where no other way goes.
        let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
        let mut best = (f32::MIN, mid);
        for i in 0..16 {
            let az = tour + i as f32 * TAU / 16.0;
            let c = [mid[0] + az.cos() * recul, mid[1] + az.sin() * recul];
            let (ca, cb) = ((c[0] - a[0]).hypot(c[1] - a[1]), (c[0] - b[0]).hypot(c[1] - b[1]));
            let mut score = ecart_angle(autour_a, (c[1] - a[1]).atan2(c[0] - a[0]))
                .min(ecart_angle(autour_b, (c[1] - b[1]).atan2(c[0] - b[0])));
            if ca.min(cb) < rh + palier {
                score -= 10.0;
            }
            if c[0].abs().max(c[1].abs()) + f.puits_rayon + 6.0 > f.emprise {
                score -= 100.0;
            }
            if score > best.0 {
                best = (score, c);
            }
        }
        best.1
    };
    let droit = |p: [f32; 2], q: [f32; 2]| -> Vec<[f32; 2]> {
        let n = (((q[0] - p[0]).hypot(q[1] - p[1]) * 3.0).ceil() as usize).max(1);
        (0..n)
            .map(|i| {
                let t = i as f32 / n as f32;
                [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]
            })
            .collect()
    };
    let (ta, tb) = ((a[1] - c[1]).atan2(a[0] - c[0]), (b[1] - c[1]).atan2(b[0] - c[0]));
    let sur = |t: f32| [c[0] + rh * t.cos(), c[1] + rh * t.sin()];
    let base = ((tb - ta) * sens).rem_euclid(TAU);
    for k in 0..80 {
        let total = base + TAU * k as f32;
        let mut plat = droit([a[0], a[1]], sur(ta));
        let debut = plat.len();
        let n = ((total * rh * 3.0).ceil() as usize).max(1);
        plat.extend((0..n).map(|i| sur(ta + sens * total * i as f32 / n as f32)));
        let fin = plat.len();
        plat.extend(droit(sur(tb), [b[0], b[1]]));
        plat.push([b[0], b[1]]);
        if etendue(&plat) - 2.0 * palier >= besoin || k == 79 {
            let rayon = rh + f.sentier_largeur + 2.0;
            return (monter(plat, a[2], b[2], (palier, palier)), Some((c, rayon, debut, fin)));
        }
    }
    unreachable!("the loop returns at its last turn")
}

/// Stalactites, stalagmites and pillars of a zone, where its ceiling is high enough.
fn concretions(g: &mut Grille, zone: &Zone, f: &Forme, rng: &mut Alea) {
    let (stalactites, stalagmites, piliers) = match zone.genre {
        Genre::Plaine => (f.plaine_stalactites, f.plaine_stalagmites, f.plaine_piliers),
        Genre::Chaos => (f.chaos_stalactites, f.chaos_stalactites * 0.5, 0.0),
        Genre::Niche => return,
    };
    let [rx, ry, h] = zone.rayon.map(|e| e as f32);
    let [cx, cy, cz] = zone.centre;
    let aire = PI * rx * ry;
    let reach = (h * 2.2) as i32 + 4;
    let disc = |g: &mut Grille, x: i32, y: i32, z: i32, r: f32| {
        let w = r.ceil() as i32;
        for dy in -w..=w {
            for dx in -w..=w {
                if ((dx * dx + dy * dy) as f32) <= r * r {
                    g.remplir(x + dx, y + dy, z, 2);
                }
            }
        }
    };
    for (nature, densite, par) in [(0, stalactites, 25.0), (1, stalagmites, 40.0), (2, piliers, 400.0)] {
        for _ in 0..(densite * aire / par) as usize {
            let (ang, rr) = (rng.entre(0.0, TAU), rng.unite().sqrt() * 0.92);
            let (x, y) = (cx + (ang.cos() * rr * rx) as i32, cy + (ang.sin() * rr * ry) as i32);
            let (size, thick) = (rng.entre(0.35, 1.0), rng.entre(1.5, 3.5));
            // The ceiling: the first air from above; the floor under it.
            let mut plafond = cz + reach;
            while plafond > cz - reach && !g.air(x, y, plafond) {
                plafond -= 1;
            }
            if plafond <= cz - reach {
                continue;
            }
            let mut sol = plafond;
            while g.air(x, y, sol - 1) && sol > cz - reach {
                sol -= 1;
            }
            let libre = plafond - sol + 1;
            match nature {
                2 if libre <= 70 => {
                    for k in 0..libre {
                        let t = k as f32 / (libre - 1).max(1) as f32;
                        disc(g, x, y, sol + k, thick * (0.6 + 0.8 * (2.0 * t - 1.0).powi(2)));
                    }
                },
                0 | 1 => {
                    let share = if nature == 0 { 1.0 } else { 0.7 };
                    let len = ((f.stalactite_longueur * share * size) as i32).min(libre - 4);
                    if len < 2 {
                        continue;
                    }
                    let foot = (1.0 + len as f32 * 0.16).min(3.5);
                    for k in 0..len {
                        let z = if nature == 0 { plafond - k } else { sol + k };
                        disc(g, x, y, z, foot * (1.0 - k as f32 / len as f32) + 0.35);
                    }
                },
                _ => {},
            }
        }
    }
}

/// The trail: rock under the feet, air for the body, all along the way.
fn poser_sentier(g: &mut Grille, points: &[[f32; 3]], largeur: f32) {
    let w = largeur.ceil() as i32;
    for p in points {
        let (cx, cy, z) = (p[0].round() as i32, p[1].round() as i32, p[2].round() as i32);
        for dy in -w..=w {
            for dx in -w..=w {
                if (dx * dx + dy * dy) as f32 > largeur * largeur + 0.5 {
                    continue;
                }
                g.remplir(cx + dx, cy + dy, z - 2, 0);
                g.remplir(cx + dx, cy + dy, z - 1, 1);
                for k in 0..4 {
                    g.creuser(cx + dx, cy + dy, z + k);
                }
            }
        }
    }
}

/// Whether one stands in this block: two blocks of air over rock.
pub fn debout(g: &Grille, x: i32, y: i32, z: i32) -> bool { g.air(x, y, z) && g.air(x, y, z + 1) && !g.air(x, y, z - 1) }

/// The blocks one stands in, walking from `depart` with steps of one block up
/// or down (so that one walks back), sorted.
pub fn marcher(g: &Grille, depart: [i32; 3]) -> Vec<[i32; 3]> {
    let mut seen = std::collections::BTreeSet::new();
    let mut stack = Vec::new();
    if debout(g, depart[0], depart[1], depart[2]) {
        seen.insert(depart);
        stack.push(depart);
    }
    while let Some([x, y, z]) = stack.pop() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for dz in [0, 1, -1] {
                let next = [x + dx, y + dy, z + dz];
                // Up a step: room for the head over where one stands.
                if debout(g, next[0], next[1], next[2]) && (dz < 1 || g.air(x, y, z + 2)) && (dz > -1 || g.air(next[0], next[1], z + 1)) {
                    if seen.insert(next) {
                        stack.push(next);
                    }
                    break;
                }
            }
        }
    }
    seen.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formes() -> Formes {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/icebreaker/grottes/formes.ron");
        Formes::lire(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn same_seed_same_cave() {
        let forme = Forme::default();
        let (a, b) = (generer(&forme, 7, &|_, _| 0, i32::MIN), generer(&forme, 7, &|_, _| 0, i32::MIN));
        assert_eq!(a.resume, b.resume);
        assert_eq!(a.grille.cellules_air().collect::<Vec<_>>(), b.grille.cellules_air().collect::<Vec<_>>());
        assert_ne!(a.resume, generer(&forme, 8, &|_, _| 0, i32::MIN).resume);
    }

    /// Every profile, over a few seeds: every zone is reached on foot from
    /// the mouth, the cave stays under the ground and in its area.
    #[test]
    fn every_profile_is_walked() {
        let formes = formes();
        for (nom, profil) in &formes.profils {
            for graine in 0..6 {
                let c = generer(&profil.forme, graine, &|_, _| 0, i32::MIN);
                assert!(c.praticable, "{nom}, graine {graine} : {}", c.resume);
                assert!(c.zones.len() >= 2, "{nom}, graine {graine} : {}", c.resume);
                for zone in &c.zones {
                    assert!(!zone.sols.is_empty(), "{nom}, graine {graine} : zone sans sol en {:?}", zone.centre);
                    assert!(zone.centre[2] + zone.rayon[2] <= -COUVERTURE, "{nom}, graine {graine} : zone hors de terre");
                }
                let (min, max) = (c.grille.min, c.grille.max());
                for i in c.grille.cellules_air() {
                    let p = c.grille.position(i);
                    assert!((0..3).all(|k| p[k] > min[k] && p[k] < max[k] - 1));
                }
            }
        }
    }

    /// The profiles differ: the pit is deep and narrow, the plain wide and shallow.
    #[test]
    fn the_seed_draws_different_caves() {
        let formes = formes();
        let depth = |nom: &str| {
            let c = generer(&formes.profils[nom].forme, 3, &|_, _| 0, i32::MIN);
            -c.zones.iter().map(|z| z.centre[2]).min().unwrap()
        };
        assert!(depth("gouffre") > 2 * depth("plaine"), "gouffre {} / plaine {}", depth("gouffre"), depth("plaine"));
        let drawn: std::collections::BTreeSet<&str> = (0..40).map(|g| formes.tirer(g, None).0).collect();
        assert_eq!(drawn.len(), formes.profils.len(), "profils tirés : {drawn:?}");
        assert_eq!(formes.tirer(1, Some("gouffre")).0, "gouffre");
    }

    #[test]
    fn settings_round_trip() {
        let forme = Forme { zones: 11.0, verticalite: 0.85, ..Forme::default() };
        assert_eq!(Forme::depuis_valeurs(&forme.valeurs()), forme);
        assert_eq!(ron::from_str::<Forme>(&forme.vers_ron()).unwrap(), forme);
        assert_eq!(Forme { zones: 99.0, ..Forme::default() }.bornee().zones, 16.0);
    }
}
