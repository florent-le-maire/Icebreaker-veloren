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

impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let settings: Self = ron::from_str(text).map_err(|e| format!("grotte.ron illisible : {e}"))?;
        let mut errors = Vec::new();
        if settings.entree.profondeur < 4 || settings.entree.longueur < 4 || settings.entree.rayon < 1.0 {
            errors.push("entree : profondeur et longueur d'au moins 4 blocs, rayon d'au moins 1".to_string());
        }
        for (name, (low, high)) in [("rayon", settings.poche.rayon), ("hauteur", settings.poche.hauteur)] {
            if low < 2 || high < low {
                errors.push(format!("poche.{name} : (min, max) avec 2 ≤ min ≤ max"));
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

/// A pocket: an ellipsoid of air around `center`.
#[derive(Clone, Debug)]
pub struct Pocket {
    pub center: Vec3<i32>,
    pub radius: Vec3<i32>,
}

impl Pocket {
    pub fn aabb(&self) -> Aabb<i32> {
        Aabb {
            min: self.center - self.radius,
            max: self.center + self.radius,
        }
    }
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
    pub bores: Vec<Bore>,
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
        for bore in &self.bores {
            let r = bore.radius.ceil() as i32;
            bounds.expand_to_contain_point(bore.a - r);
            bounds.expand_to_contain_point(bore.a + r);
            bounds.expand_to_contain_point(bore.b - r);
            bounds.expand_to_contain_point(bore.b + r);
        }
        bounds
    }
}

/// Generates a cave whose mouth is at `mouth` (a surface position): for now
/// (K.1) one pocket under the mouth and the bore that leads to it.
pub fn generate(settings: &Settings, seed: u64, mouth: Vec3<i32>) -> Grotte {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let dir = [Vec2::new(1, 0), Vec2::new(-1, 0), Vec2::new(0, 1), Vec2::new(0, -1)][rng.random_range(0..4)];
    let roll = |rng: &mut ChaCha8Rng, (low, high): (i32, i32)| rng.random_range(low..=high);
    let radius = Vec3::new(
        roll(&mut rng, settings.poche.rayon),
        roll(&mut rng, settings.poche.rayon),
        roll(&mut rng, settings.poche.hauteur),
    );
    let center = (mouth.xy() + dir * settings.entree.longueur).with_z(mouth.z - settings.entree.profondeur);
    let pocket = Pocket { center, radius };
    let bore = Bore {
        a: mouth,
        b: center,
        radius: settings.entree.rayon,
    };
    Grotte {
        mouth,
        dir,
        summary: format!("grotte, graine {seed} : 1 poche de rayon {radius:?} en {center:?}"),
        pockets: vec![pocket],
        bores: vec![bore],
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
}
