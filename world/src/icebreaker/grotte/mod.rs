//! Icebreaker: the cave (« Grotte », `docs/icebreaker/PLAN-GROTTE.md`,
//! jalon K), drawn by the `IcebreakerGrotte` plot
//! (`site/plot/icebreaker_grotte.rs`). Its shape comes from the
//! `icebreaker-caverne` generator (jalon U, `PLAN-CAVERNE.md`: zones, shafts
//! and a trail, dug in a grid of blocks); here is what lives in it: the mood,
//! the ores, the inhabitants, the chests.

use super::dungeon::settings::sprite_kind;
use common::vol::RectVolSize;
use hashbrown::HashSet;
use icebreaker_caverne::{
    Caverne, Forme, Formes, Genre,
    filon::{self, Gisement},
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};
use vek::*;

/// `assets/icebreaker/grottes/grotte.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    /// Icebreaker I.11: chunks kept around the mouth, on each side, for the
    /// cave and the ones drawn again in its place.
    pub emprise: i32,
    /// Height of the glowing crystals, in blocks (min, max).
    pub hauteur_cristaux: (i32, i32),
    /// The ores (Icebreaker K.4).
    pub filons: VeinSettings,
    /// The inhabitants (Icebreaker K.5).
    pub population: PopulationSettings,
    /// The moods of the cave (Icebreaker K.3), by name; one is drawn per cave.
    pub ambiances: BTreeMap<String, Ambiance>,
    /// Icebreaker H.4: the people of a race (one per cave, drawn from the
    /// seed among `especes`), and their chief at the bottom.
    pub habitants: Vec<String>,
    pub chef: String,
    pub especes: Vec<String>,
    /// Icebreaker B.5: the loot by depth (`butin.ron`), read by `load`.
    #[serde(skip)]
    pub butin: Option<super::butin::Butin>,
    /// Icebreaker U.4: the shapes (`formes.ron`), read by `load`.
    #[serde(skip)]
    pub formes: Option<Formes>,
}

/// The veins of ore (Icebreaker U.19, `design/filons.md`): how many; their ore,
/// shape and hardness are in `minerais.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct VeinSettings {
    /// The levels of the cave, from the entrance to its end: the level of a
    /// zone picks its ores (`poids` in `minerais.ron`) and its loot.
    pub niveaux: i32,
    /// How many veins a zone holds.
    pub veines: VeineSettings,
    /// Icebreaker U.15: the ore room.
    pub salle: SalleSettings,
}

/// Icebreaker U.15, U.19: the ore room, one per cave behind a wall: how many
/// veins cover its walls (their ores: `salle` in `minerais.ron`).
#[derive(Clone, Debug, Deserialize)]
pub struct SalleSettings {
    pub filons: i32,
}

/// Icebreaker U.14, U.19: how many veins a zone holds, by level (the last
/// value for the deeper levels); their ore and shape: `minerais.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct VeineSettings {
    pub par_zone: Vec<i32>,
}

impl VeineSettings {
    pub fn count(&self, level: i32) -> i32 {
        let list = &self.par_zone;
        list.get((level.max(0) as usize).min(list.len().saturating_sub(1))).copied().unwrap_or(0)
    }
}

/// Icebreaker U.19: a vein of ore, blocks in the rock (`rock`) and bumps in
/// the air (`relief`), in the world; `ore` is its place in `minerais.ron`.
#[derive(Clone, Debug)]
pub struct Vein {
    pub ore: usize,
    pub rock: Vec<Vec3<i32>>,
    pub relief: Vec<Vec3<i32>>,
    pub in_ore_room: bool,
}

impl Vein {
    pub fn blocks(&self) -> impl Iterator<Item = &Vec3<i32>> { self.rock.iter().chain(&self.relief) }

    pub fn len(&self) -> usize { self.rock.len() + self.relief.len() }
}

/// The inhabitants (Icebreaker K.5): how many, and where the chests lie.
#[derive(Clone, Debug, Deserialize)]
pub struct PopulationSettings {
    /// Creatures per zone (min, max), at the entrance...
    pub par_poche: (i32, i32),
    /// ...plus this many per level further in the cave.
    pub par_niveau: i32,
    /// Chance of a creature in a niche, and of a chest there.
    pub trous: f32,
    pub coffres: f32,
    /// The chest sprite.
    pub coffre: String,
    /// Icebreaker H.4: share of the lone monsters among the creatures of the
    /// zones (the niches hold only lone monsters).
    pub part_solitaires: f32,
    /// Icebreaker U.8: no creature within this many blocks of where one
    /// arrives (the safe room).
    pub calme: i32,
}

/// A mood of the cave (Icebreaker K.3): what its rock and its sprites are.
#[derive(Clone, Debug, Deserialize)]
pub struct Ambiance {
    /// Colour of the rock of the cave.
    pub roche: (u8, u8, u8),
    /// Colour of the glowing rock, and its share among the blocks of the walls.
    pub lueur: (u8, u8, u8),
    pub lueur_chance: f32,
    /// Sprites on the floors, with a chance per block.
    pub sol: Vec<(String, f32)>,
    /// Sprites under the ceilings, with a chance per block.
    pub plafond: Vec<(String, f32)>,
    /// Glowing crystals per zone (min, max).
    pub cristaux: (i32, i32),
    /// Icebreaker H.4: the lone monsters living here (entities of Veloren, of
    /// no race).
    pub solitaires: Vec<String>,
}

impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let settings: Self = ron::from_str(text).map_err(|e| format!("grotte.ron illisible : {e}"))?;
        let mut errors = Vec::new();
        if settings.emprise < 3 {
            errors.push("emprise : au moins 3 tronçons".to_string());
        }
        let mut ranges = vec![("hauteur_cristaux", settings.hauteur_cristaux, 1)];
        for ambiance in settings.ambiances.values() {
            ranges.push(("cristaux", ambiance.cristaux, 0));
        }
        for (name, (low, high), least) in ranges {
            if low < least || high < low {
                errors.push(format!("{name} : (min, max) avec {least} ≤ min ≤ max"));
            }
        }
        let veins = &settings.filons;
        if veins.niveaux < 1 {
            errors.push("filons : niveaux ≥ 1".into());
        }
        let v = &veins.veines;
        if v.par_zone.is_empty() || v.par_zone.iter().any(|n| *n < 0) {
            errors.push("filons.veines : par_zone (≥ 0), au moins une valeur".into());
        }
        if veins.salle.filons < 0 {
            errors.push("filons.salle : filons ≥ 0".into());
        }
        if settings.ambiances.is_empty() {
            errors.push("ambiances : au moins une".into());
        }
        for (name, ambiance) in &settings.ambiances {
            for (sprite, chance) in ambiance.sol.iter().chain(&ambiance.plafond) {
                if sprite_kind(sprite).is_none() {
                    errors.push(format!("ambiances.{name} : sprite inconnu « {sprite} »"));
                }
                if !(0.0..=1.0).contains(chance) {
                    errors.push(format!("ambiances.{name} : chance de « {sprite} » entre 0 et 1"));
                }
            }
            if !(0.0..=1.0).contains(&ambiance.lueur_chance) {
                errors.push(format!("ambiances.{name} : lueur_chance entre 0 et 1"));
            }
            if ambiance.solitaires.is_empty() {
                errors.push(format!("ambiances.{name} : solitaires (au moins un)"));
            }
        }
        let pop = &settings.population;
        if pop.par_poche.0 < 0
            || pop.par_poche.1 < pop.par_poche.0
            || pop.par_niveau < 0
            || !(0.0..=1.0).contains(&pop.trous)
            || !(0.0..=1.0).contains(&pop.coffres)
            || !(0.0..=1.0).contains(&pop.part_solitaires)
        {
            errors.push("population : par_poche (min, max) avec 0 ≤ min ≤ max, par_niveau ≥ 0, trous et coffres entre 0 et 1".into());
        }
        if sprite_kind(&pop.coffre).is_none() {
            errors.push(format!("population : coffre inconnu « {} »", pop.coffre));
        }
        if settings.habitants.is_empty() || settings.chef.is_empty() {
            errors.push("habitants (au moins un) et chef".into());
        }
        for espece in &settings.especes {
            if espece.parse::<common::npc::NpcBody>().is_err() {
                errors.push(format!("especes : espèce inconnue « {espece} »"));
            }
        }
        if errors.is_empty() { Ok(settings) } else { Err(errors.join(" ; ")) }
    }

    pub fn load() -> Result<Self, String> {
        let read = |name: &str| {
            let path = common::assets::ASSETS_PATH.join("icebreaker/grottes").join(name);
            std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))
        };
        let mut settings = Self::parse(&read("grotte.ron")?)?;
        settings.butin = Some(super::butin::Butin::load()?);
        settings.formes = Some(Formes::lire(&read("formes.ron")?)?);
        Ok(settings)
    }
}

/// An inhabitant of the cave (Icebreaker K.5): where it appears (the block
/// its feet are in) and which entity of Veloren it is.
#[derive(Clone, Debug)]
pub struct Inhabitant {
    pub pos: Vec3<i32>,
    pub asset: String,
    /// The leader: killing it wins the mission (G.7).
    pub chief: bool,
    /// Icebreaker B.14: the level of its zone, and the loot table it drops
    /// (`butin.ron`; `None`: its own, from Veloren).
    pub level: i32,
    pub loot: Option<String>,
}

/// A generated cave, ready to be drawn.
#[derive(Clone)]
pub struct Grotte {
    /// The origin of the cave, at the surface (the cave has no mouth there
    /// any more, Icebreaker U.8: its shape is in blocks relative to this).
    pub mouth: Vec3<i32>,
    /// Where one arrives: in the safe room, a block one stands in.
    pub arrival: Vec3<i32>,
    /// Icebreaker U.4: its shape, in blocks relative to the mouth, and the
    /// profile of `formes.ron` it was drawn with.
    pub caverne: Arc<Caverne>,
    pub profil: String,
    /// Icebreaker U.11: the seed it was generated from (shown with `profil`
    /// to the party that arrives: the same in the workshop gives its shape).
    pub seed: u64,
    /// The level of each zone of `caverne`: 0 at the entrance, `niveaux - 1`
    /// at the end.
    pub levels: Vec<i32>,
    pub niveaux: i32,
    /// The blocks the cave touches.
    pub bounds: Aabb<i32>,
    /// Glowing crystals: the block they stand in, and their height.
    pub crystals: Vec<(Vec3<i32>, i32)>,
    /// The mood drawn, and what it is made of.
    pub ambiance: String,
    pub look: Ambiance,
    pub filons: VeinSettings,
    /// Icebreaker U.19: the veins (bands of ore blocks in the rock, those of
    /// the ore room among them), and the ore of each block (its place in
    /// `minerais.ron`).
    pub veins: Vec<Vein>,
    pub vein_blocks: Arc<hashbrown::HashMap<Vec3<i32>, usize>>,
    /// The inhabitants (Icebreaker K.5), the chief among them.
    pub inhabitants: Vec<Inhabitant>,
    /// Icebreaker H.4: the species of its people (none: their own bodies).
    pub species: Option<String>,
    /// Where the chests lie, and their sprite.
    pub chests: Vec<Vec3<i32>>,
    /// Icebreaker B.5: the tier of each chest (same order), by the level of
    /// its zone, and the loot settings.
    pub chest_tiers: Vec<usize>,
    pub butin: Option<super::butin::Butin>,
    pub chest: String,
    pub summary: String,
}

impl Grotte {
    /// The blocks the cave may touch.
    pub fn bounds(&self) -> Aabb<i32> { self.bounds }

    /// A block of the shape, in the world.
    pub fn monde(&self, p: [i32; 3]) -> Vec3<i32> { self.mouth + Vec3::from(p) }

    /// Whether the cave is hollow at `wpos`.
    pub fn air(&self, wpos: Vec3<i32>) -> bool {
        let p = wpos - self.mouth;
        self.caverne.grille.air(p.x, p.y, p.z)
    }

    /// Icebreaker U.13: whether `wpos` is rock of a wall to break (a way
    /// between two zones closed by the generator).
    pub fn wall(&self, wpos: Vec3<i32>) -> bool {
        let p = wpos - self.mouth;
        self.caverne.grille.mur(p.x, p.y, p.z)
    }

    /// Whether `wpos` is in the safe room.
    pub fn in_safe_room(&self, wpos: Vec3<i32>) -> bool {
        let p = (wpos - self.mouth).into_array();
        self.caverne.abri.is_some_and(|i| self.caverne.zones[i].contient(p, 1.3))
    }

    /// The level of the zone `wpos` is in (the entrance level out of them all).
    pub fn level_at(&self, wpos: Vec3<i32>) -> i32 {
        let p = (wpos - self.mouth).into_array();
        self.caverne
            .zones
            .iter()
            .position(|z| z.contient(p, 1.25))
            .map_or(0, |i| self.levels[i])
    }
}


/// Icebreaker U.19: the ores of `minerais.ron` as the placing of the veins
/// (`icebreaker_caverne::filon`) sees them, in the same order.
pub fn gisements(minerais: &common::icebreaker_minerais::Minerais) -> Vec<Gisement> {
    minerais
        .liste()
        .iter()
        .map(|(_, m)| Gisement {
            longueur: m.filon.longueur,
            epaisseur: m.filon.epaisseur,
            profondeur: m.filon.profondeur,
            sinuosite: m.filon.sinuosite,
            relief: m.filon.relief,
            poids: m.poids.clone(),
            salle: m.salle,
        })
        .collect()
}

/// Generates a cave whose mouth is at `mouth` (a surface position).
/// `ambiance` names the mood wanted and `profil` the shape (drawn by the seed
/// if `None` or unknown); `surface` gives the height of the ground.
pub fn generate(
    settings: &Settings,
    seed: u64,
    mouth: Vec3<i32>,
    ambiance: Option<&str>,
    profil: Option<&str>,
    surface: &dyn Fn(Vec2<i32>) -> i32,
) -> Grotte {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let roll = |rng: &mut ChaCha8Rng, (low, high): (i32, i32)| rng.random_range(low..=high);
    let names: Vec<&String> = settings.ambiances.keys().collect();
    let ambiance = ambiance
        .filter(|a| settings.ambiances.contains_key(*a))
        .map(String::from)
        .unwrap_or_else(|| names[rng.random_range(0..names.len())].clone());
    let look = settings.ambiances[&ambiance].clone();

    // The shape (Icebreaker U.4): a profile of `formes.ron`, within the area
    // kept for the cave.
    let (profil, mut forme) = match &settings.formes {
        Some(formes) => {
            let (nom, forme) = formes.tirer(seed, profil);
            (nom.to_string(), forme.clone())
        },
        None => ("défaut".to_string(), Forme::default()),
    };
    let chunk = common::terrain::TerrainChunkSize::RECT_SIZE.x as i32;
    forme.emprise = forme.emprise.min((settings.emprise * chunk - 8) as f32);
    // Not under the bottom of the world.
    let caverne =
        icebreaker_caverne::generer(&forme, seed, &|x, y| surface(mouth.xy() + Vec2::new(x, y)) - mouth.z, 8 - mouth.z);
    let monde = |p: [i32; 3]| mouth + Vec3::from(p);
    let zones = &caverne.zones;

    // Levels: by how far the zone is along the trail; a niche has the level
    // of the zone it opens on.
    let niveaux = settings.filons.niveaux.max(1);
    let levels: Vec<i32> = filon::niveaux_des_zones(&caverne, niveaux);

    // Inhabitants (K.5): the chief at the end of the cave; creatures in every
    // zone, more of them further in; a lone monster and a chest in some
    // niches. All of them stand where one walks from the mouth.
    let pop = &settings.population;
    let lone = |rng: &mut ChaCha8Rng| look.solitaires[rng.random_range(0..look.solitaires.len())].clone();
    let fond = caverne.fond();
    let mut taken: HashSet<[i32; 3]> = HashSet::from([zones[fond].ancre]);
    let mut inhabitants = vec![Inhabitant {
        pos: monde(zones[fond].ancre),
        asset: settings.chef.clone(),
        chief: true,
        level: levels[fond],
        loot: None,
    }];
    let mut chests = Vec::new();
    let mut chest_levels = Vec::new();
    // Icebreaker U.8: no one near where the party arrives.
    let quiet = |p: [i32; 3]| (0..3).map(|k| ((p[k] - caverne.depart[k]) as i64).pow(2)).sum::<i64>() >= (pop.calme as i64).pow(2);
    for (i, zone) in zones.iter().enumerate() {
        // No one in the safe room, nor in the ore room (U.15): its wall is its guard.
        if matches!(zone.genre, Genre::Abri | Genre::Minerai) {
            continue;
        }
        if zone.genre == Genre::Niche {
            if !quiet(zone.ancre) {
                continue;
            }
            if rng.random_bool(pop.trous as f64) && taken.insert(zone.ancre) {
                let asset = lone(&mut rng);
                inhabitants.push(Inhabitant {
                    pos: monde(zone.ancre),
                    asset,
                    chief: false,
                    level: levels[i],
                    loot: None,
                });
            }
            // The chest at the back of the niche: as far from its way in as one stands.
            let back = zone
                .sols
                .iter()
                .filter(|p| !taken.contains(*p) && **p != zone.ancre)
                .max_by_key(|p| (0..3).map(|k| (p[k] - zone.ancre[k]).pow(2)).sum::<i32>());
            if let Some(back) = back
                && rng.random_bool(pop.coffres as f64)
            {
                taken.insert(*back);
                chests.push(monde(*back));
                chest_levels.push(levels[i].max(0) as usize);
            }
            continue;
        }
        if zone.sols.is_empty() {
            continue;
        }
        let count = roll(&mut rng, pop.par_poche).max(0) + levels[i] * pop.par_niveau;
        for _ in 0..count {
            for _ in 0..10 {
                let p = zone.sols[rng.random_range(0..zone.sols.len())];
                // Away from the walls: an ore may lie at their foot (a solid sprite).
                let open = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().all(|(dx, dy)| caverne.grille.air(p[0] + dx, p[1] + dy, p[2]));
                if !open || !quiet(p) || !taken.insert(p) {
                    continue;
                }
                let asset = if rng.random_bool(pop.part_solitaires as f64) {
                    lone(&mut rng)
                } else {
                    settings.habitants[rng.random_range(0..settings.habitants.len())].clone()
                };
                inhabitants.push(Inhabitant {
                    pos: monde(p),
                    asset,
                    chief: false,
                    level: levels[i],
                    loot: None,
                });
                break;
            }
        }
    }
    // The veins (Icebreaker U.19, `design/filons.md`): bands of ore blocks in
    // the rock of the walls, shaped by their ore (`minerais.ron`), placed by
    // `icebreaker_caverne::filon` (the workshop page shows the same).
    let minerais = common::icebreaker_minerais::minerais();
    let gisements = gisements(minerais);
    let poses = filon::poser(
        &caverne,
        &gisements,
        &levels,
        &settings.filons.veines.par_zone,
        settings.filons.salle.filons,
        &|p| taken.contains(&p),
        rng.random::<u64>(),
    );
    let veins: Vec<Vein> = poses
        .iter()
        .map(|p| Vein {
            ore: p.minerai,
            rock: p.filon.roche.iter().map(|q| monde(*q)).collect(),
            relief: p.filon.relief.iter().map(|q| monde(*q)).collect(),
            in_ore_room: p.salle,
        })
        .collect();
    // No crystal grows on a bump of ore.
    for pose in &poses {
        taken.extend(pose.filon.relief.iter().copied());
    }
    let vein_blocks: hashbrown::HashMap<Vec3<i32>, usize> =
        veins.iter().flat_map(|v| v.blocks().map(move |p| (*p, v.ore))).collect();

    // Glowing crystals (K.3): on the floors, off the trail.
    let mut crystals = Vec::new();
    for zone in zones.iter().filter(|z| !matches!(z.genre, Genre::Niche | Genre::Abri | Genre::Minerai) && !z.sols.is_empty()) {
        for _ in 0..roll(&mut rng, look.cristaux).max(0) {
            for _ in 0..10 {
                let p = zone.sols[rng.random_range(0..zone.sols.len())];
                let trail = (-3..=3).any(|dx| (-3..=3).any(|dy| caverne.grille.sentier(p[0] + dx, p[1] + dy, p[2] - 1)));
                // Off the trail, and no one stands where the spike grows.
                let near = taken.iter().any(|q| (q[0] - p[0]).abs() <= 3 && (q[1] - p[1]).abs() <= 3 && (q[2] - p[2]).abs() <= 8);
                if trail || near || !taken.insert(p) {
                    continue;
                }
                crystals.push((monde(p), roll(&mut rng, settings.hauteur_cristaux)));
                break;
            }
        }
    }

    let species = crate::icebreaker::races::species_of(&settings.especes, seed).cloned();
    let butin = settings.butin.clone();
    let chest_tiers: Vec<usize> = chest_levels
        .iter()
        .map(|l| butin.as_ref().map_or(1, |b| b.palier(*l, niveaux as usize)))
        .collect();
    if let Some(b) = &butin {
        for i in &mut inhabitants {
            let kind = if i.chief { "chief" } else { "" };
            i.loot = Some(b.creature(kind, b.palier(i.level.max(0) as usize, niveaux as usize)).to_string());
        }
    }
    let tiers = butin.as_ref().map_or(1, |b| b.paliers());
    let chests_by_tier: Vec<String> = (1..=tiers)
        .map(|p| chest_tiers.iter().filter(|t| **t == p).count().to_string())
        .collect();

    // The blocks it touches: its air, and the rock around it.
    let mut bounds = Aabb { min: mouth, max: mouth };
    for i in caverne.grille.cellules_air() {
        bounds.expand_to_contain_point(monde(caverne.grille.position(i)));
    }
    bounds.min -= 2;
    bounds.max += 3;

    Grotte {
        mouth,
        arrival: monde(caverne.depart),
        seed,
        summary: format!(
            "grotte {ambiance}, forme {profil}, race {}, graine {seed} : {} ; {} habitant(s), {} filon(s) dont {} dans la salle au minerai ({} blocs de minerai), {} cristal(aux), {} coffre(s) \
             ({} par palier), fond en {:?}, un filon en {:?}",
            species.as_deref().unwrap_or("des fichiers"),
            caverne.resume,
            inhabitants.len(),
            veins.len(),
            veins.iter().filter(|v| v.in_ore_room).count(),
            vein_blocks.len(),
            crystals.len(),
            chests.len(),
            chests_by_tier.join(" / "),
            monde(zones[fond].ancre),
            // Icebreaker U.24 : où aller voir un filon (une bosse dans le vide, sinon un bloc dans la roche).
            veins.iter().find_map(|v| v.relief.first()).or_else(|| veins.iter().find_map(|v| v.rock.first())),
        ),
        levels,
        niveaux,
        bounds,
        crystals,
        profil,
        ambiance,
        look,
        filons: settings.filons.clone(),
        veins,
        vein_blocks: Arc::new(vein_blocks),
        inhabitants,
        species,
        chests,
        chest_tiers,
        butin,
        chest: settings.population.coffre.clone(),
        caverne: Arc::new(caverne),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUTH: Vec3<i32> = Vec3::new(1000, 1000, 200);

    fn cave(settings: &Settings, seed: u64) -> Grotte { generate(settings, seed, MOUTH, None, None, &|_| MOUTH.z) }

    #[test]
    fn settings_load() { Settings::load().unwrap(); }

    /// Icebreaker U.4: the cave lies under its mouth, every zone is walked to
    /// from it, and the same seed gives the same cave.
    #[test]
    fn a_cave_lies_under_the_mouth() {
        let settings = Settings::load().unwrap();
        for seed in 0..6 {
            let grotte = cave(&settings, seed);
            assert!(grotte.caverne.praticable, "graine {seed} : {}", grotte.summary);
            for zone in &grotte.caverne.zones {
                assert!(zone.centre[2] + zone.rayon[2] < 0, "graine {seed} : une zone n'est pas sous terre");
                assert!(grotte.bounds().contains_point(grotte.monde(zone.ancre)));
            }
            assert!(grotte.bounds().contains_point(MOUTH));
            // Icebreaker U.8: one arrives in the safe room, with no one around, far from the chief.
            assert!(grotte.caverne.abri.is_some(), "graine {seed} : pas de salle sûre");
            assert!(grotte.air(grotte.arrival) && !grotte.air(grotte.arrival - Vec3::unit_z()), "graine {seed} : arrivée dans la roche");
            assert!(grotte.in_safe_room(grotte.arrival));
            let calme = settings.population.calme as f32;
            for i in &grotte.inhabitants {
                let d = i.pos.as_::<f32>().distance(grotte.arrival.as_::<f32>());
                assert!(d >= calme, "graine {seed} : {} à {d} blocs de l'arrivée", i.asset);
                assert!(!i.chief || d >= 60.0, "graine {seed} : le chef est à {d} blocs de l'arrivée");
            }
            assert_eq!(cave(&settings, seed).summary, grotte.summary);
        }
    }

    /// Every shape of `formes.ron` can be asked for, and holds in the area kept.
    #[test]
    fn every_shape_holds_in_its_area() {
        let settings = Settings::load().unwrap();
        let kept = settings.emprise * 32;
        for name in settings.formes.as_ref().unwrap().profils.keys() {
            let grotte = generate(&settings, 5, MOUTH, None, Some(name), &|_| MOUTH.z);
            assert_eq!(&grotte.profil, name);
            let b = grotte.bounds();
            assert!(
                b.min.x > MOUTH.x - kept && b.max.x < MOUTH.x + kept && b.min.y > MOUTH.y - kept && b.max.y < MOUTH.y + kept,
                "{name} : {b:?} hors de l'emprise"
            );
        }
    }

    /// Icebreaker K.3: every mood can be asked for.
    #[test]
    fn moods() {
        let settings = Settings::load().unwrap();
        for name in settings.ambiances.keys() {
            assert_eq!(&generate(&settings, 3, MOUTH, Some(name), None, &|_| MOUTH.z).ambiance, name);
        }
        assert!(settings.ambiances.contains_key(&generate(&settings, 11, MOUTH, Some("lune"), None, &|_| MOUTH.z).ambiance));
    }

    /// Icebreaker U.19: from the second level on, every zone has a vein: a
    /// band of ore blocks in the rock of a wall, bumps in the air, never on
    /// the trail nor in the safe room.
    #[test]
    fn veins_lie_in_the_walls() {
        let settings = Settings::load().unwrap();
        let minerais = common::icebreaker_minerais::minerais();
        let mut ores = std::collections::HashSet::new();
        for seed in 0..6 {
            let grotte = cave(&settings, seed);
            let c = &grotte.caverne;
            for (i, zone) in c.zones.iter().enumerate() {
                if matches!(zone.genre, Genre::Niche | Genre::Abri | Genre::Minerai) || grotte.levels[i] < 1 {
                    continue;
                }
                let near = |p: &Vec3<i32>| zone.sols.iter().any(|s| (0..3).all(|k| (s[k] - (p[k] - MOUTH[k])).abs() <= 3));
                let inside = grotte.veins.iter().filter(|v| !v.in_ore_room && v.rock.iter().any(near)).count();
                assert!(inside >= 1, "graine {seed} : zone {i} (niveau {}) sans filon", grotte.levels[i]);
            }
            let mut total = 0;
            for vein in &grotte.veins {
                let m = minerais.get(vein.ore).unwrap();
                assert!(vein.rock.len() >= (m.filon.longueur.0 as usize).min(4), "graine {seed} : filon de {} bloc(s)", vein.rock.len());
                ores.insert(vein.ore);
                for p in &vein.rock {
                    assert!(!grotte.air(*p), "graine {seed} : minerai dans l'air");
                    let q = *p - MOUTH;
                    assert!(!c.grille.sentier(q.x, q.y, q.z) && !grotte.wall(*p), "graine {seed} : filon sur le sentier ou un mur");
                }
                for p in &vein.relief {
                    assert!(grotte.air(*p), "graine {seed} : bosse dans la roche");
                    let q = *p - MOUTH;
                    let trail = (-2..=2).any(|dx| (-2..=2).any(|dy| (1..=3).any(|dz| c.grille.sentier(q.x + dx, q.y + dy, q.z - dz))));
                    assert!(!trail, "graine {seed} : bosse sur le sentier");
                    assert!(grotte.inhabitants.iter().all(|i| i.pos != *p) && grotte.chests.iter().all(|c| c != p));
                }
                for p in vein.blocks() {
                    assert!(!grotte.in_safe_room(*p) && grotte.vein_blocks.get(p) == Some(&vein.ore));
                }
                total += vein.len();
            }
            assert_eq!(total, grotte.vein_blocks.len(), "graine {seed} : deux filons sur un bloc");
        }
        assert!(ores.len() >= 6, "{ores:?}");
    }

    /// Icebreaker U.15, U.19: every cave has an ore room behind a wall, its
    /// walls covered with veins, with no one in it; over a few caves, common
    /// ores and rare ones.
    #[test]
    fn the_ore_room_is_full_behind_its_wall() {
        let settings = Settings::load().unwrap();
        let minerais = common::icebreaker_minerais::minerais();
        let mut found = std::collections::BTreeSet::new();
        for seed in 0..6 {
            let grotte = cave(&settings, seed);
            let c = &grotte.caverne;
            let m = c.minerai.unwrap_or_else(|| panic!("graine {seed} : pas de salle au minerai"));
            assert!(c.fermees.contains(&m), "graine {seed} : la salle au minerai est ouverte");
            let room: Vec<&Vein> = grotte.veins.iter().filter(|v| v.in_ore_room).collect();
            let blocks: usize = room.iter().map(|v| v.len()).sum();
            assert!(room.len() * 2 >= settings.filons.salle.filons as usize, "graine {seed} : {} filon(s) dans la salle", room.len());
            let richest = grotte.veins.iter().filter(|v| !v.in_ore_room).map(|v| v.len()).max().unwrap_or(0);
            assert!(blocks > 2 * richest, "graine {seed} : la salle n'est pas plus riche qu'un filon");
            for vein in &room {
                for p in vein.blocks() {
                    assert!(c.zones[m].contient((*p - MOUTH).into_array(), 1.6), "graine {seed} : minerai hors de la salle");
                }
                found.insert(minerais.liste()[vein.ore].0.clone());
            }
            let inside = |p: Vec3<i32>| c.zones[m].contient((p - MOUTH).into_array(), 1.3);
            assert!(grotte.inhabitants.iter().all(|i| !inside(i.pos)) && grotte.chests.iter().all(|p| !inside(*p)), "graine {seed}");
            assert_eq!(grotte.levels[m], grotte.levels[c.zones[m].parent.unwrap()]);
        }
        assert!(found.contains("fer") && found.len() >= 5, "{found:?}");
    }

    /// Icebreaker B.5: each chest has a tier (the level of its zone); over a
    /// few caves, chests of every tier.
    #[test]
    fn chests_get_richer_with_depth() {
        let settings = Settings::load().unwrap();
        assert!(settings.butin.is_some());
        let mut by_tier = [0; 3];
        for seed in 0..20 {
            let grotte = cave(&settings, seed);
            assert_eq!(grotte.chest_tiers.len(), grotte.chests.len());
            for (chest, tier) in grotte.chests.iter().zip(&grotte.chest_tiers) {
                assert!(grotte.air(*chest), "graine {seed} : coffre dans la roche");
                by_tier[tier - 1] += 1;
            }
        }
        assert!(by_tier.iter().all(|n| *n > 0), "coffres par palier : {by_tier:?}");
    }

    /// Icebreaker B.14: every inhabitant drops a table of `butin.ron`, the
    /// chief its own, the others that of the level of their zone.
    #[test]
    fn inhabitants_drop_the_loot_of_their_depth() {
        let settings = Settings::load().unwrap();
        let butin = settings.butin.clone().unwrap();
        for seed in 0..6 {
            let grotte = cave(&settings, seed);
            for i in &grotte.inhabitants {
                let expected = if i.chief {
                    butin.chef.as_str()
                } else {
                    butin.creature("", butin.palier(i.level as usize, grotte.niveaux as usize))
                };
                assert_eq!(i.loot.as_deref(), Some(expected), "graine {seed} : {i:?}");
            }
            let deepest = grotte.inhabitants.iter().filter(|i| !i.chief).map(|i| i.level).max().unwrap();
            assert!(deepest > 0, "graine {seed} : personne au fond");
        }
    }

    /// Icebreaker K.5: one chief at the end of the cave; everyone stands in
    /// the cave, no two at the same place.
    #[test]
    fn the_cave_is_inhabited() {
        use common::assets::AssetExt;
        let settings = Settings::load().unwrap();
        let mut total = 0;
        for seed in 1..=10 {
            let grotte = cave(&settings, seed);
            let chiefs: Vec<&Inhabitant> = grotte.inhabitants.iter().filter(|i| i.chief).collect();
            assert_eq!(chiefs.len(), 1, "graine {seed} : un seul chef");
            let fond = &grotte.caverne.zones[grotte.caverne.fond()];
            assert_eq!(chiefs[0].pos, grotte.monde(fond.ancre), "graine {seed} : le chef n'est pas au fond");
            let mut places = std::collections::HashSet::new();
            for i in &grotte.inhabitants {
                assert!(grotte.air(i.pos) && !grotte.air(i.pos - Vec3::unit_z()), "graine {seed} : {i:?} n'est pas debout");
                assert!(places.insert(i.pos), "graine {seed} : deux habitants en {:?}", i.pos);
            }
            assert!(grotte.inhabitants.len() >= 10, "graine {seed} : {} habitant(s) seulement", grotte.inhabitants.len());
            total += grotte.inhabitants.len();
        }
        let mean = total as f32 / 10.0;
        assert!((15.0..=50.0).contains(&mean), "{mean} habitants par grotte en moyenne");
        // Every creature named in the data is an entity of Veloren.
        let people = settings.habitants.iter().chain(std::iter::once(&settings.chef));
        for asset in settings.ambiances.values().flat_map(|a| &a.solitaires).chain(people) {
            common::assets::Ron::<common::generation::EntityConfig>::load(asset).unwrap_or_else(|e| panic!("{asset} : {e}"));
        }
    }

    // Icebreaker H.4: one race per cave, its chief at the bottom, lone monsters too.
    #[test]
    fn one_race_per_cave() {
        let settings = Settings::load().unwrap();
        for seed in 1..=6 {
            let grotte = cave(&settings, seed);
            let species = grotte.species.clone().expect("une race");
            assert!(settings.especes.contains(&species));
            assert!(grotte.summary.contains(&format!("race {species}")));
            let chief = grotte.inhabitants.iter().find(|i| i.chief).unwrap();
            assert_eq!(chief.asset, settings.chef);
            let people = grotte.inhabitants.iter().filter(|i| settings.habitants.contains(&i.asset)).count();
            let lone = grotte.inhabitants.iter().filter(|i| grotte.look.solitaires.contains(&i.asset)).count();
            assert!(people > 0 && lone > 0, "graine {seed} : {people} de la race, {lone} solitaire(s)");
            assert_eq!(people + lone + 1, grotte.inhabitants.len());
        }
        // Spiders live in every mood.
        for (name, ambiance) in &settings.ambiances {
            assert!(ambiance.solitaires.iter().any(|s| s.ends_with("cave_spider")), "{name} : pas d'araignée");
        }
    }
}
