//! Icebreaker: the cave (« Grotte », `docs/icebreaker/PLAN-GROTTE.md`,
//! jalon K): a place of natural pockets and bores dug in the rock, drawn by
//! the `IcebreakerGrotte` plot (`site/plot/icebreaker_grotte.rs`). No cells
//! nor rooms here: the layout is in blocks.

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use vek::*;

/// `assets/icebreaker/grottes/grotte.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub entree: EntranceSettings,
    pub poche: PocketSettings,
    pub reseau: NetworkSettings,
}

/// The entrance: a bore from the mouth, at the surface, down to the first pocket.
#[derive(Clone, Debug, Deserialize)]
pub struct EntranceSettings {
    /// Blocks between the surface and the centre of the first pocket.
    pub profondeur: i32,
    /// Horizontal blocks between the mouth and the centre of the first pocket.
    pub longueur: i32,
    /// Radius of the bore, in blocks.
    pub rayon: f32,
}

/// A pocket (a chamber): an ellipsoid.
#[derive(Clone, Debug, Deserialize)]
pub struct PocketSettings {
    /// Horizontal radius, in blocks (min, max).
    pub rayon: (i32, i32),
    /// Half-height, in blocks (min, max).
    pub hauteur: (i32, i32),
}

/// The network (Icebreaker K.2): pockets at several depths, linked by bores
/// (a tree from the entrance, plus loops), with dead ends out of the pockets.
#[derive(Clone, Debug, Deserialize)]
pub struct NetworkSettings {
    /// Pockets in all, entrance pocket included (min, max).
    pub poches: (i32, i32),
    /// Depth levels, the entrance pocket at the first.
    pub niveaux: i32,
    /// Blocks down from one level to the next.
    pub ecart_niveau: i32,
    /// Horizontal blocks between the walls of a pocket and of its parent (min, max).
    pub ecart: (i32, i32),
    /// Radius of a bore between two pockets, in blocks (min, max).
    pub rayon_boyau: (i32, i32),
    /// Extra bores between pockets already placed (min, max).
    pub boucles: (i32, i32),
    /// Dead ends out of each pocket (min, max), their length (min, max) and radius.
    pub culs_de_sac: (i32, i32),
    pub longueur_cul_de_sac: (i32, i32),
    pub rayon_cul_de_sac: f32,
}

impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let settings: Self = ron::from_str(text).map_err(|e| format!("grotte.ron illisible : {e}"))?;
        let mut errors = Vec::new();
        if settings.entree.profondeur < 4 || settings.entree.longueur < 4 || settings.entree.rayon < 1.0 {
            errors.push("entree : profondeur et longueur d'au moins 4 blocs, rayon d'au moins 1".to_string());
        }
        let n = &settings.reseau;
        let ranges = [
            ("poche.rayon", settings.poche.rayon, 2),
            ("poche.hauteur", settings.poche.hauteur, 2),
            ("reseau.poches", n.poches, 1),
            ("reseau.ecart", n.ecart, 2),
            ("reseau.rayon_boyau", n.rayon_boyau, 1),
            ("reseau.boucles", n.boucles, 0),
            ("reseau.culs_de_sac", n.culs_de_sac, 0),
            ("reseau.longueur_cul_de_sac", n.longueur_cul_de_sac, 2),
        ];
        for (name, (low, high), least) in ranges {
            if low < least || high < low {
                errors.push(format!("{name} : (min, max) avec {least} ≤ min ≤ max"));
            }
        }
        if n.niveaux < 1 || n.ecart_niveau < 4 || n.rayon_cul_de_sac < 1.0 {
            errors.push("reseau : au moins 1 niveau, ecart_niveau ≥ 4, rayon_cul_de_sac ≥ 1".into());
        }
        if errors.is_empty() { Ok(settings) } else { Err(errors.join(" ; ")) }
    }

    pub fn load() -> Result<Self, String> {
        let path = common::assets::ASSETS_PATH.join("icebreaker/grottes/grotte.ron");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))?;
        Self::parse(&text)
    }
}

/// A pocket: an ellipsoid of air around `center`, at depth `level` (0 = the
/// entrance pocket).
#[derive(Clone, Debug)]
pub struct Pocket {
    pub center: Vec3<i32>,
    pub radius: Vec3<i32>,
    pub level: i32,
}

impl Pocket {
    pub fn aabb(&self) -> Aabb<i32> {
        Aabb {
            min: self.center - self.radius,
            max: self.center + self.radius,
        }
    }

    /// Where one stands in the middle of the pocket: on its floor.
    pub fn floor(&self) -> Vec3<i32> { self.center.with_z(self.center.z - self.radius.z + 1) }
}

/// A bore: a tube of air from `a` to `b`.
#[derive(Clone, Debug)]
pub struct Bore {
    pub a: Vec3<i32>,
    pub b: Vec3<i32>,
    pub radius: f32,
}

/// A generated cave, ready to be drawn.
#[derive(Clone, Debug)]
pub struct Grotte {
    /// The mouth, at the surface.
    pub mouth: Vec3<i32>,
    /// Which way the entrance bore goes from the mouth (horizontal, unit).
    pub dir: Vec2<i32>,
    pub pockets: Vec<Pocket>,
    /// The bores between the pockets (the entrance bore first).
    pub bores: Vec<Bore>,
    /// Dead ends out of the pockets: `b` is their end, the hole the beasts
    /// come from.
    pub dead_ends: Vec<Bore>,
    pub summary: String,
}

impl Grotte {
    /// The blocks the cave may touch.
    pub fn bounds(&self) -> Aabb<i32> {
        let mut bounds = Aabb {
            min: self.mouth,
            max: self.mouth,
        };
        for pocket in &self.pockets {
            bounds.expand_to_contain(pocket.aabb());
        }
        for bore in self.bores.iter().chain(&self.dead_ends) {
            let r = bore.radius.ceil() as i32;
            bounds.expand_to_contain_point(bore.a - r);
            bounds.expand_to_contain_point(bore.a + r);
            bounds.expand_to_contain_point(bore.b - r);
            bounds.expand_to_contain_point(bore.b + r);
        }
        bounds
    }

    /// The deepest pocket (where the chief lives).
    pub fn deepest(&self) -> &Pocket {
        self.pockets.iter().min_by_key(|p| p.center.z).expect("a cave has pockets")
    }
}

/// Tries to place a pocket before giving up on it.
const PLACEMENT_TRIES: usize = 40;

/// Generates a cave whose mouth is at `mouth` (a surface position): the
/// entrance pocket under the mouth, then the others, each hung on a pocket
/// of the level above (a tree), plus loops and dead ends.
pub fn generate(settings: &Settings, seed: u64, mouth: Vec3<i32>) -> Grotte {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let roll = |rng: &mut ChaCha8Rng, (low, high): (i32, i32)| rng.random_range(low..=high);
    let radius = |rng: &mut ChaCha8Rng| {
        Vec3::new(
            roll(rng, settings.poche.rayon),
            roll(rng, settings.poche.rayon),
            roll(rng, settings.poche.hauteur),
        )
    };
    let dir = [Vec2::new(1, 0), Vec2::new(-1, 0), Vec2::new(0, 1), Vec2::new(0, -1)][rng.random_range(0..4)];
    let first = Pocket {
        center: (mouth.xy() + dir * settings.entree.longueur).with_z(mouth.z - settings.entree.profondeur),
        radius: radius(&mut rng),
        level: 0,
    };
    let mut bores = vec![Bore {
        a: mouth,
        b: first.center,
        radius: settings.entree.rayon,
    }];
    let mut pockets = vec![first];
    let mut links: Vec<(usize, usize)> = Vec::new();

    // The tree: pocket i goes one level deeper every `levels / count` pockets.
    let net = &settings.reseau;
    let count = roll(&mut rng, net.poches).max(1);
    for i in 1..count {
        let level = ((i * net.niveaux) / count).min(net.niveaux - 1);
        let parents: Vec<usize> = (0..pockets.len()).filter(|p| pockets[*p].level == level - 1).collect();
        let parents = if parents.is_empty() { (0..pockets.len()).collect() } else { parents };
        let mut placed = None;
        for _ in 0..PLACEMENT_TRIES {
            let parent = parents[rng.random_range(0..parents.len())];
            let radius = radius(&mut rng);
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let gap = roll(&mut rng, net.ecart);
            let distance = pockets[parent].radius.x.max(pockets[parent].radius.y) + radius.x.max(radius.y) + gap;
            let offset = Vec2::new(angle.cos(), angle.sin()) * distance as f32;
            let drop = if level > pockets[parent].level { net.ecart_niveau + rng.random_range(-3..=3) } else { rng.random_range(-4..=4) };
            let center = (pockets[parent].center.xy().as_::<f32>() + offset).map(|e| e.round() as i32).with_z(pockets[parent].center.z - drop);
            let candidate = Pocket { center, radius, level };
            // Away from every other pocket, and never above the entrance pocket.
            let clear = pockets.iter().all(|p| {
                let d = (p.center.xy() - center.xy()).as_::<f32>().magnitude();
                d >= (p.radius.x.max(p.radius.y) + radius.x.max(radius.y) + 4) as f32 || (p.center.z - center.z).abs() > p.radius.z + radius.z + 6
            });
            if clear && center.z + radius.z < mouth.z - 6 {
                placed = Some((parent, candidate));
                break;
            }
        }
        let Some((parent, pocket)) = placed else { continue };
        bores.push(Bore {
            a: pockets[parent].center,
            b: pocket.center,
            radius: roll(&mut rng, net.rayon_boyau) as f32,
        });
        links.push((parent, pockets.len()));
        pockets.push(pocket);
    }

    // Loops: between two pockets not yet linked, close to each other.
    let mut loops = 0;
    for _ in 0..roll(&mut rng, net.boucles).max(0) {
        let mut pairs: Vec<(i32, usize, usize)> = Vec::new();
        for a in 0..pockets.len() {
            for b in a + 1..pockets.len() {
                if links.contains(&(a, b)) || links.contains(&(b, a)) {
                    continue;
                }
                let d = (pockets[a].center - pockets[b].center).map(|e| e.abs()).sum();
                pairs.push((d, a, b));
            }
        }
        pairs.sort();
        let Some((_, a, b)) = pairs.first().copied() else { break };
        bores.push(Bore {
            a: pockets[a].center,
            b: pockets[b].center,
            radius: roll(&mut rng, net.rayon_boyau) as f32,
        });
        links.push((a, b));
        loops += 1;
    }

    // Dead ends: narrow tubes out of each pocket, slightly rising or falling.
    let mut dead_ends = Vec::new();
    for pocket in &pockets {
        for _ in 0..roll(&mut rng, net.culs_de_sac).max(0) {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let length = roll(&mut rng, net.longueur_cul_de_sac);
            let start = pocket.center.xy().as_::<f32>() + Vec2::new(angle.cos(), angle.sin()) * (pocket.radius.x.min(pocket.radius.y) as f32 - 2.0);
            let end = start + Vec2::new(angle.cos(), angle.sin()) * length as f32;
            let z = pocket.floor().z + 1;
            dead_ends.push(Bore {
                a: start.map(|e| e.round() as i32).with_z(z),
                b: end.map(|e| e.round() as i32).with_z(z + rng.random_range(-2..=3)),
                radius: net.rayon_cul_de_sac,
            });
        }
    }

    let levels = pockets.iter().map(|p| p.level).max().unwrap_or(0) + 1;
    Grotte {
        mouth,
        dir,
        summary: format!(
            "grotte, graine {seed} : {} poche(s) sur {levels} niveau(x), {} boyau(x) dont {loops} boucle(s), {} cul(s)-de-sac, fond en {:?}",
            pockets.len(),
            bores.len(),
            dead_ends.len(),
            pockets.iter().min_by_key(|p| p.center.z).map(|p| p.center)
        ),
        pockets,
        bores,
        dead_ends,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_load() { Settings::load().unwrap(); }

    #[test]
    fn a_pocket_lies_under_the_mouth() {
        let settings = Settings::load().unwrap();
        let mouth = Vec3::new(1000, 1000, 200);
        let grotte = generate(&settings, 7, mouth);
        let pocket = &grotte.pockets[0];
        assert!(pocket.center.z + pocket.radius.z < mouth.z, "la poche n'est pas sous terre");
        assert_eq!(grotte.bores[0].a, mouth);
        assert!(grotte.bounds().contains_point(pocket.center));
        // Same seed, same cave.
        assert_eq!(generate(&settings, 7, mouth).summary, grotte.summary);
    }

    /// Icebreaker K.2: the network holds enough pockets, on several levels,
    /// all linked to the entrance, none overlapping.
    #[test]
    fn the_network_is_a_cave() {
        let settings = Settings::load().unwrap();
        let mouth = Vec3::new(1000, 1000, 200);
        for seed in 0..20 {
            let grotte = generate(&settings, seed, mouth);
            let n = grotte.pockets.len();
            assert!(n >= settings.reseau.poches.0 as usize, "graine {seed} : {n} poches");
            let levels = grotte.pockets.iter().map(|p| p.level).max().unwrap() + 1;
            assert!(levels >= 2, "graine {seed} : un seul niveau");
            assert!(grotte.deepest().center.z < grotte.pockets[0].center.z - settings.reseau.ecart_niveau / 2);
            // Linked: from the entrance pocket, every pocket is reached through the bores.
            let index_of = |p: Vec3<i32>| grotte.pockets.iter().position(|q| q.center == p);
            let mut reached = vec![false; n];
            reached[0] = true;
            loop {
                let mut changed = false;
                for bore in &grotte.bores[1..] {
                    if let (Some(a), Some(b)) = (index_of(bore.a), index_of(bore.b)) && reached[a] != reached[b] {
                        reached[a] = true;
                        reached[b] = true;
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }
            assert!(reached.iter().all(|r| *r), "graine {seed} : une poche sans boyau");
            for (i, a) in grotte.pockets.iter().enumerate() {
                for b in &grotte.pockets[i + 1..] {
                    let d = (a.center.xy() - b.center.xy()).as_::<f32>().magnitude();
                    let apart = d >= (a.radius.x.max(a.radius.y) + b.radius.x.max(b.radius.y)) as f32
                        || (a.center.z - b.center.z).abs() > a.radius.z + b.radius.z;
                    assert!(apart, "graine {seed} : deux poches se chevauchent");
                }
            }
            assert!(!grotte.dead_ends.is_empty());
        }
    }
}
