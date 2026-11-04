//! Icebreaker: the cave (« Grotte », `docs/icebreaker/PLAN-GROTTE.md`,
//! jalon K): a place of natural pockets and bores dug in the rock, drawn by
//! the `IcebreakerGrotte` plot (`site/plot/icebreaker_grotte.rs`). No cells
//! nor rooms here: the layout is in blocks.

use crate::util::{FastNoise, Sampler};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::BTreeMap;
use vek::*;

/// `assets/icebreaker/grottes/grotte.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub entree: EntranceSettings,
    pub poche: PocketSettings,
    pub reseau: NetworkSettings,
    pub relief: ReliefSettings,
    /// The moods of the cave (Icebreaker K.3), by name; one is drawn per cave.
    pub ambiances: BTreeMap<String, Ambiance>,
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

/// The relief (Icebreaker K.3): how uneven the floors and the walls are.
#[derive(Clone, Debug, Deserialize)]
pub struct ReliefSettings {
    /// Height of the bumps of a pocket's floor, in blocks.
    pub bosses: f32,
    /// How far the walls of a pocket wander, as a share of its radius.
    pub rugosite: f32,
    /// Height of stalagmites and stalactites, in blocks (min, max).
    pub hauteur_concretions: (i32, i32),
}

/// A mood of the cave (Icebreaker K.3): what its floors, walls, concretions
/// and sprites are made of.
#[derive(Clone, Debug, Deserialize)]
pub struct Ambiance {
    /// Colour of the rubble floors.
    pub roche: (u8, u8, u8),
    /// Colour of the glowing rock, and its share among the floor blocks.
    pub lueur: (u8, u8, u8),
    pub lueur_chance: f32,
    /// Sprites on the floors, with a chance per block.
    pub sol: Vec<(String, f32)>,
    /// Sprites under the ceilings, with a chance per block.
    pub plafond: Vec<(String, f32)>,
    /// Stalagmites, stalactites and glowing crystals per pocket (min, max).
    pub stalagmites: (i32, i32),
    pub stalactites: (i32, i32),
    pub cristaux: (i32, i32),
}

impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let settings: Self = ron::from_str(text).map_err(|e| format!("grotte.ron illisible : {e}"))?;
        let mut errors = Vec::new();
        if settings.entree.profondeur < 4 || settings.entree.longueur < 4 || settings.entree.rayon < 1.0 {
            errors.push("entree : profondeur et longueur d'au moins 4 blocs, rayon d'au moins 1".to_string());
        }
        let n = &settings.reseau;
        let mut ranges = vec![
            ("poche.rayon", settings.poche.rayon, 2),
            ("poche.hauteur", settings.poche.hauteur, 2),
            ("reseau.poches", n.poches, 1),
            ("reseau.ecart", n.ecart, 2),
            ("reseau.rayon_boyau", n.rayon_boyau, 1),
            ("reseau.boucles", n.boucles, 0),
            ("reseau.culs_de_sac", n.culs_de_sac, 0),
            ("reseau.longueur_cul_de_sac", n.longueur_cul_de_sac, 2),
            ("relief.hauteur_concretions", settings.relief.hauteur_concretions, 1),
        ];
        for ambiance in settings.ambiances.values() {
            ranges.push(("stalagmites", ambiance.stalagmites, 0));
            ranges.push(("stalactites", ambiance.stalactites, 0));
            ranges.push(("cristaux", ambiance.cristaux, 0));
        }
        for (name, (low, high), least) in ranges {
            if low < least || high < low {
                errors.push(format!("{name} : (min, max) avec {least} ≤ min ≤ max"));
            }
        }
        if n.niveaux < 1 || n.ecart_niveau < 4 || n.rayon_cul_de_sac < 1.0 {
            errors.push("reseau : au moins 1 niveau, ecart_niveau ≥ 4, rayon_cul_de_sac ≥ 1".into());
        }
        if settings.relief.bosses < 0.0 || !(0.0..=0.5).contains(&settings.relief.rugosite) {
            errors.push("relief : bosses ≥ 0, rugosite entre 0 et 0,5".into());
        }
        if settings.ambiances.is_empty() {
            errors.push("ambiances : au moins une".into());
        }
        for (name, ambiance) in &settings.ambiances {
            for (sprite, chance) in ambiance.sol.iter().chain(&ambiance.plafond) {
                if super::dungeon::settings::sprite_kind(sprite).is_none() {
                    errors.push(format!("ambiances.{name} : sprite inconnu « {sprite} »"));
                }
                if !(0.0..=1.0).contains(chance) {
                    errors.push(format!("ambiances.{name} : chance de « {sprite} » entre 0 et 1"));
                }
            }
            if !(0.0..=1.0).contains(&ambiance.lueur_chance) {
                errors.push(format!("ambiances.{name} : lueur_chance entre 0 et 1"));
            }
        }
        if errors.is_empty() { Ok(settings) } else { Err(errors.join(" ; ")) }
    }

    pub fn load() -> Result<Self, String> {
        let path = common::assets::ASSETS_PATH.join("icebreaker/grottes/grotte.ron");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))?;
        Self::parse(&text)
    }
}

/// The bumps of the floors: 0 to `bosses` blocks, smooth over a few blocks.
pub fn bump(xy: Vec2<i32>, bosses: f32) -> i32 {
    (FastNoise::new(731).get(xy.with_z(0).as_::<f64>() / 6.0) * bosses).round() as i32
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

    /// The height of the floor at `xy` (rubble over the bottom of the
    /// ellipsoid, Icebreaker K.3): the first solid block from below.
    pub fn floor_z(&self, xy: Vec2<i32>, bosses: f32) -> i32 { self.center.z - self.radius.z + bump(xy, bosses) }

    /// Squared normalised horizontal distance of `xy` from the centre (1 at the rim).
    pub fn rim(&self, xy: Vec2<i32>) -> f32 {
        let d = (xy - self.center.xy()).as_::<f32>();
        (d.x / self.radius.x as f32).powi(2) + (d.y / self.radius.y as f32).powi(2)
    }

    /// The height of the ceiling at `xy` (the last air block), if the pocket
    /// is open there.
    pub fn ceiling_z(&self, xy: Vec2<i32>) -> Option<i32> {
        let rim = self.rim(xy);
        (rim < 1.0).then(|| self.center.z + (self.radius.z as f32 * (1.0 - rim).sqrt()).floor() as i32 - 1)
    }
}

/// A bore: a tube of air from `a` to `b`.
#[derive(Clone, Debug)]
pub struct Bore {
    pub a: Vec3<i32>,
    pub b: Vec3<i32>,
    pub radius: f32,
}

/// A concretion of a pocket (Icebreaker K.3).
#[derive(Clone, Debug)]
pub struct Speleothem {
    /// On the floor (stalagmite, crystal) or under the ceiling (stalactite).
    pub base: Vec3<i32>,
    pub height: i32,
    pub kind: SpeleothemKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpeleothemKind {
    Stalagmite,
    Stalactite,
    Crystal,
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
    pub speleothems: Vec<Speleothem>,
    /// The mood drawn, and what it is made of.
    pub ambiance: String,
    pub look: Ambiance,
    pub relief: ReliefSettings,
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
            let margin = (pocket.radius.x.max(pocket.radius.y) as f32 * self.relief.rugosite).ceil() as i32 + 2;
            bounds.expand_to_contain(Aabb {
                min: pocket.aabb().min - margin,
                max: pocket.aabb().max + margin,
            });
        }
        for bore in self.bores.iter().chain(&self.dead_ends) {
            let r = bore.radius.ceil() as i32 + 2;
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
const PLACEMENT_TRIES: usize = 80;

/// Blocks along for one block down, at least: a slope one walks back up
/// (one-block steps).
const SLOPE: f32 = 1.6;

/// Whether `candidate` would cut through one of `existing` out of the pockets
/// (two tubes crossing leave a pit in one of them).
fn crosses(existing: &[Bore], pockets: &[Pocket], candidate: &Bore) -> bool {
    let in_pocket = |p: Vec3<i32>| {
        pockets.iter().any(|pocket| {
            let aabb = pocket.aabb();
            Aabb {
                min: aabb.min - 2,
                max: aabb.max + 2,
            }
            .contains_point(p)
        })
    };
    let samples = |bore: &Bore| {
        let n = ((bore.b - bore.a).as_::<f32>().magnitude().ceil() as i32).max(1);
        (0..=n)
            .map(move |k| {
                (bore.a.as_::<f32>() + (bore.b - bore.a).as_::<f32>() * (k as f32 / n as f32)).map(|e| e.round() as i32)
            })
            .filter(|p| !in_pocket(*p))
            .collect::<Vec<_>>()
    };
    // Through a pocket it does not come from or go to: a pit in that pocket.
    let margin = |pocket: &Pocket| Aabb {
        min: pocket.aabb().min - 3,
        max: pocket.aabb().max + 3,
    };
    let own = |pocket: &Pocket| {
        let aabb = margin(pocket);
        aabb.contains_point(candidate.a) || aabb.contains_point(candidate.b)
    };
    let n = ((candidate.b - candidate.a).as_::<f32>().magnitude().ceil() as i32).max(1);
    let through = (0..=n).any(|k| {
        let p = (candidate.a.as_::<f32>() + (candidate.b - candidate.a).as_::<f32>() * (k as f32 / n as f32))
            .map(|e| e.round() as i32);
        pockets.iter().any(|pocket| !own(pocket) && margin(pocket).contains_point(p))
    });
    if through {
        return true;
    }
    let mine = samples(candidate);
    existing.iter().any(|other| {
        let least = candidate.radius + other.radius + 2.0;
        samples(other)
            .iter()
            .any(|p| mine.iter().any(|q| (*p - *q).as_::<f32>().magnitude() < least))
    })
}

/// Generates a cave whose mouth is at `mouth` (a surface position): the
/// entrance pocket under the mouth, then the others, each hung on a pocket
/// of the level above (a tree), plus loops and dead ends; `ambiance` names
/// the mood wanted (drawn by the seed if `None` or unknown).
pub fn generate(
    settings: &Settings,
    seed: u64,
    mouth: Vec3<i32>,
    ambiance: Option<&str>,
    surface: &dyn Fn(Vec2<i32>) -> i32,
) -> Grotte {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    // Everything stays under the ground (the terrain may fall away from the mouth).
    let underground = |bore: &Bore| {
        let n = ((bore.b - bore.a).as_::<f32>().magnitude() / 3.0).ceil().max(1.0) as i32;
        (0..=n).all(|k| {
            let p = bore.a.as_::<f32>() + (bore.b - bore.a).as_::<f32>() * (k as f32 / n as f32);
            p.z + bore.radius + 3.0 < surface(p.xy().map(|e| e.round() as i32)) as f32
        })
    };
    let roll = |rng: &mut ChaCha8Rng, (low, high): (i32, i32)| rng.random_range(low..=high);
    let radius = |rng: &mut ChaCha8Rng| {
        Vec3::new(
            roll(rng, settings.poche.rayon),
            roll(rng, settings.poche.rayon),
            roll(rng, settings.poche.hauteur),
        )
    };
    let names: Vec<&String> = settings.ambiances.keys().collect();
    let ambiance = ambiance
        .filter(|a| settings.ambiances.contains_key(*a))
        .map(String::from)
        .unwrap_or_else(|| names[rng.random_range(0..names.len())].clone());
    let look = settings.ambiances[&ambiance].clone();

    // The entrance runs toward the highest ground (the cave stays under it).
    let dirs = [Vec2::new(1, 0), Vec2::new(-1, 0), Vec2::new(0, 1), Vec2::new(0, -1)];
    let start = rng.random_range(0..4);
    let dir = (0..4)
        .map(|i| dirs[(start + i) % 4])
        .max_by_key(|d| surface(mouth.xy() + *d * settings.entree.longueur))
        .unwrap_or(dirs[0]);
    // The entrance pocket: under the mouth, under the ground even where the
    // terrain falls away, and far enough for the entrance bore to be gentle
    // (the farther it is, the lower the ground may be there: until it settles).
    let first_radius = radius(&mut rng);
    let mut longueur = settings.entree.longueur;
    let mut first_center = mouth;
    for _ in 0..8 {
        let xy = mouth.xy() + dir * longueur;
        let z = (mouth.z - settings.entree.profondeur).min(surface(xy) - first_radius.z - 8);
        first_center = xy.with_z(z);
        let drop = (mouth.z + 2 - (z - first_radius.z + 1)) as f32;
        let needed = longueur.max((SLOPE * drop).ceil() as i32 + 4);
        if needed == longueur {
            break;
        }
        longueur = needed;
    }
    let first = Pocket {
        center: first_center,
        radius: first_radius,
        level: 0,
    };
    // Bores run at floor level, so that one walks from a pocket into them: they
    // start at the rim of the pocket, toward where they go, their floor level
    // with the pocket's (a tube of radius r clears r - 1 blocks under its
    // axis). The entrance bore starts a little before the mouth, above the
    // ground: it opens to the sky.
    let rim = |pocket: &Pocket, toward: Vec2<i32>, radius: f32| {
        let d = (toward - pocket.center.xy()).as_::<f32>().try_normalized().unwrap_or(Vec2::unit_x());
        let along = 1.0 / ((d.x / pocket.radius.x as f32).powi(2) + (d.y / pocket.radius.y as f32).powi(2)).sqrt();
        // On the flat floor of the pocket (0.92 of its radius), a little inside.
        let xy = pocket.center.xy().as_::<f32>() + d * (along * 0.86);
        xy.map(|e| e.round() as i32).with_z(pocket.floor().z + radius.ceil() as i32 - 1)
    };
    let mut bores = vec![Bore {
        a: (mouth.xy() - dir * 3).with_z(mouth.z + 2),
        b: rim(&first, mouth.xy(), settings.entree.rayon),
        radius: settings.entree.rayon,
    }];
    // A slope one walks up: at least `SLOPE` blocks along for one block down.
    let gentle = |a: Vec3<i32>, b: Vec3<i32>| (a.xy() - b.xy()).as_::<f32>().magnitude() >= SLOPE * (a.z - b.z).abs() as f32;
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
            let drop = if level > pockets[parent].level {
                net.ecart_niveau + rng.random_range(-3..=3)
            } else {
                rng.random_range(-4..=4)
            };
            let center = (pockets[parent].center.xy().as_::<f32>() + offset)
                .map(|e| e.round() as i32)
                .with_z(pockets[parent].center.z - drop);
            // Under the ground here, with rock above.
            let center = center.with_z(center.z.min(surface(center.xy()) - radius.z - 8));
            let candidate = Pocket { center, radius, level };
            // Away from every other pocket, and never above the entrance pocket.
            let clear = pockets.iter().all(|p| {
                let d = (p.center.xy() - center.xy()).as_::<f32>().magnitude();
                d >= (p.radius.x.max(p.radius.y) + radius.x.max(radius.y) + 4) as f32
                    || (p.center.z - center.z).abs() > p.radius.z + radius.z + 6
            });
            if !(clear && center.z + radius.z < mouth.z - 6) {
                continue;
            }
            // Its bore: a gentle slope, crossing no other bore.
            let bore_radius = roll(&mut rng, net.rayon_boyau) as f32;
            let bore = Bore {
                a: rim(&pockets[parent], candidate.center.xy(), bore_radius),
                b: rim(&candidate, pockets[parent].center.xy(), bore_radius),
                radius: bore_radius,
            };
            if gentle(bore.a, bore.b) && underground(&bore) && !crosses(&bores, &pockets, &bore) {
                placed = Some((parent, candidate, bore));
                break;
            }
        }
        let Some((parent, pocket, bore)) = placed else { continue };
        bores.push(bore);
        links.push((parent, pockets.len()));
        pockets.push(pocket);
    }

    // Loops: between two pockets of the same level or of neighbouring levels,
    // not yet linked, the closest first; a gentle slope, crossing nothing.
    let mut loops = 0;
    for _ in 0..roll(&mut rng, net.boucles).max(0) {
        let mut pairs: Vec<(i32, usize, usize)> = Vec::new();
        for a in 0..pockets.len() {
            for b in a + 1..pockets.len() {
                if links.contains(&(a, b)) || links.contains(&(b, a)) || (pockets[a].level - pockets[b].level).abs() > 1 {
                    continue;
                }
                let d = (pockets[a].center - pockets[b].center).map(|e| e.abs()).sum();
                pairs.push((d, a, b));
            }
        }
        pairs.sort();
        let radius = roll(&mut rng, net.rayon_boyau) as f32;
        let found = pairs.iter().find_map(|(_, a, b)| {
            let bore = Bore {
                a: rim(&pockets[*a], pockets[*b].center.xy(), radius),
                b: rim(&pockets[*b], pockets[*a].center.xy(), radius),
                radius,
            };
            (gentle(bore.a, bore.b) && underground(&bore) && !crosses(&bores, &pockets, &bore)).then_some((*a, *b, bore))
        });
        let Some((a, b, bore)) = found else { break };
        bores.push(bore);
        links.push((a, b));
        loops += 1;
    }

    // Dead ends: narrow tubes out of each pocket, slightly rising or falling,
    // crossing nothing.
    let mut dead_ends: Vec<Bore> = Vec::new();
    for pocket in &pockets {
        for _ in 0..roll(&mut rng, net.culs_de_sac).max(0) {
            for _ in 0..10 {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let length = roll(&mut rng, net.longueur_cul_de_sac);
                let start = pocket.center.xy().as_::<f32>()
                    + Vec2::new(angle.cos(), angle.sin()) * (pocket.radius.x.min(pocket.radius.y) as f32 - 2.0);
                let end = start + Vec2::new(angle.cos(), angle.sin()) * length as f32;
                let z = pocket.floor().z + net.rayon_cul_de_sac.ceil() as i32 - 1;
                let bore = Bore {
                    a: start.map(|e| e.round() as i32).with_z(z),
                    b: end.map(|e| e.round() as i32).with_z(z + rng.random_range(-2..=3)),
                    radius: net.rayon_cul_de_sac,
                };
                let others: Vec<Bore> = bores.iter().chain(&dead_ends).cloned().collect();
                if underground(&bore) && !crosses(&others, &pockets, &bore) {
                    dead_ends.push(bore);
                    break;
                }
            }
        }
    }

    // Concretions (K.3): in each pocket, away from its middle and from the
    // ways in (the bores and dead ends), on the floor or under the ceiling.
    let mut speleothems = Vec::new();
    let relief = &settings.relief;
    for pocket in &pockets {
        let ways: Vec<f32> = bores
            .iter()
            .chain(&dead_ends)
            .filter_map(|bore| {
                let (from, to) = if pocket.aabb().contains_point(bore.b) {
                    (pocket.center, bore.a)
                } else if pocket.aabb().contains_point(bore.a) {
                    (pocket.center, bore.b)
                } else {
                    return None;
                };
                let d = (to.xy() - from.xy()).as_::<f32>();
                Some(d.y.atan2(d.x))
            })
            .collect();
        let kinds = [
            (SpeleothemKind::Stalagmite, look.stalagmites),
            (SpeleothemKind::Stalactite, look.stalactites),
            (SpeleothemKind::Crystal, look.cristaux),
        ];
        for (kind, range) in kinds {
            for _ in 0..roll(&mut rng, range).max(0) {
                for _ in 0..10 {
                    let angle = rng.random_range(0.0..std::f32::consts::TAU);
                    let away = ways.iter().all(|w| {
                        let diff = (angle - w).rem_euclid(std::f32::consts::TAU);
                        diff.min(std::f32::consts::TAU - diff) > 0.7
                    });
                    if !away {
                        continue;
                    }
                    let reach = rng.random_range(0.45..0.8);
                    let xy = pocket.center.xy()
                        + Vec2::new(
                            (angle.cos() * reach * pocket.radius.x as f32).round() as i32,
                            (angle.sin() * reach * pocket.radius.y as f32).round() as i32,
                        );
                    // A stalactite stops above the middle of the pocket: the bores pass there.
                    let height = roll(&mut rng, relief.hauteur_concretions).min(pocket.radius.z - 1).max(1);
                    let base = match kind {
                        SpeleothemKind::Stalactite => match pocket.ceiling_z(xy) {
                            Some(z) => xy.with_z(z),
                            None => continue,
                        },
                        _ => xy.with_z(pocket.floor_z(xy, relief.bosses) + 1),
                    };
                    speleothems.push(Speleothem { base, height, kind });
                    break;
                }
            }
        }
    }

    let levels = pockets.iter().map(|p| p.level).max().unwrap_or(0) + 1;
    Grotte {
        mouth,
        dir,
        summary: format!(
            "grotte {ambiance}, graine {seed} : {} poche(s) sur {levels} niveau(x), {} boyau(x) dont {loops} boucle(s), {} \
             cul(s)-de-sac, {} concrétion(s), fond en {:?}",
            pockets.len(),
            bores.len(),
            dead_ends.len(),
            speleothems.len(),
            pockets.iter().min_by_key(|p| p.center.z).map(|p| p.center)
        ),
        pockets,
        bores,
        dead_ends,
        speleothems,
        ambiance,
        look,
        relief: relief.clone(),
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
        let grotte = generate(&settings, 7, mouth, None, &|_| mouth.z);
        let pocket = &grotte.pockets[0];
        assert!(pocket.center.z + pocket.radius.z < mouth.z, "la poche n'est pas sous terre");
        assert_eq!(grotte.bores[0].a.z, mouth.z + 2, "l'entrée ne s'ouvre pas au ciel");
        assert!(grotte.bounds().contains_point(pocket.center));
        // Same seed, same cave.
        assert_eq!(generate(&settings, 7, mouth, None, &|_| mouth.z).summary, grotte.summary);
    }

    /// Icebreaker K.2: the network holds enough pockets, on several levels,
    /// all linked to the entrance, none overlapping.
    #[test]
    fn the_network_is_a_cave() {
        let settings = Settings::load().unwrap();
        let mouth = Vec3::new(1000, 1000, 200);
        for seed in 0..20 {
            let grotte = generate(&settings, seed, mouth, None, &|_| mouth.z);
            let n = grotte.pockets.len();
            assert!(n >= settings.reseau.poches.0 as usize, "graine {seed} : {n} poches");
            let levels = grotte.pockets.iter().map(|p| p.level).max().unwrap() + 1;
            assert!(levels >= 2, "graine {seed} : un seul niveau");
            assert!(grotte.deepest().center.z < grotte.pockets[0].center.z - settings.reseau.ecart_niveau / 2);
            // Linked: from the entrance pocket, every pocket is reached through the bores.
            let index_of = |p: Vec3<i32>| {
                grotte.pockets.iter().position(|q| q.rim(p.xy()) <= 1.3 && (p.z - q.center.z).abs() <= q.radius.z + 4)
            };
            let mut reached = vec![false; n];
            reached[0] = true;
            loop {
                let mut changed = false;
                for bore in &grotte.bores[1..] {
                    if let (Some(a), Some(b)) = (index_of(bore.a), index_of(bore.b))
                        && reached[a] != reached[b]
                    {
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

    /// Icebreaker K.3: every mood can be asked for, and the concretions stand
    /// inside their pocket.
    #[test]
    fn moods_and_concretions() {
        let settings = Settings::load().unwrap();
        let mouth = Vec3::new(1000, 1000, 200);
        for name in settings.ambiances.keys() {
            let grotte = generate(&settings, 3, mouth, Some(name), &|_| mouth.z);
            assert_eq!(&grotte.ambiance, name);
            assert!(!grotte.speleothems.is_empty(), "{name} : aucune concrétion");
            for s in &grotte.speleothems {
                let inside = grotte.pockets.iter().any(|p| p.rim(s.base.xy()) < 1.0 && (s.base.z - p.center.z).abs() <= p.radius.z + 1);
                assert!(inside, "{name} : concrétion hors des poches en {:?}", s.base);
            }
        }
        assert!(settings.ambiances.contains_key(&generate(&settings, 11, mouth, Some("lune"), &|_| mouth.z).ambiance));
    }
}
