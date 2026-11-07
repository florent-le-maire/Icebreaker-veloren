//! Icebreaker: the settings of the dungeon generator
//! (`assets/icebreaker/donjons/generation.ron`), and what a mission makes of
//! them.

use super::rooms::RoomRole;
use rand::RngExt;
use serde::Deserialize;
use std::collections::BTreeMap;

/// An inclusive range `(min, max)`.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub struct Range(pub i32, pub i32);

impl Range {
    pub fn roll(self, rng: &mut impl RngExt) -> i32 { rng.random_range(self.0..=self.1) }

    pub fn valid(self, lowest: i32) -> bool { lowest <= self.0 && self.0 <= self.1 }
}

/// How a plan becomes voxels.
#[derive(Clone, Debug, Deserialize)]
pub struct RenderSettings {
    /// Blocks per cell, horizontally.
    pub cell_size: i32,
    /// Blocks per level (floor included). Stairs climb one level over one
    /// cell, one block per step: at most `cell_size`.
    pub level_height: i32,
    /// Cells of the tunnel from the surface down to the entrance room.
    pub tunnel_cells: i32,
    pub wall_color: (u8, u8, u8),
    pub floor_color: (u8, u8, u8),
}

/// The shape of the mission graph for a size of dungeon. Counts of branches,
/// locks, loops and drops are maximums.
#[derive(Clone, Debug, Deserialize)]
pub struct GraphShape {
    /// Rooms of the main path, entrance and goal included.
    pub main_path: Range,
    /// Levels the main path goes through.
    pub levels: Range,
    pub branches: Range,
    pub branch_length: Range,
    pub max_branches_per_room: usize,
    pub locks: Range,
    pub loops: Range,
    pub drops: Range,
}

/// What a role of the branches requires.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct RoleRule {
    /// Roles the room the branch starts from may have (empty: any).
    pub next_to: Vec<String>,
    /// 1: hooked on the main path; 2: at least one room in between.
    pub min_distance_from_main_path: i32,
    /// Rooms of this role at most per dungeon.
    pub max: Option<usize>,
}

/// Rules that make believable inhabitants.
#[derive(Clone, Debug, Deserialize)]
pub struct Rules {
    /// The room before each locked door is a guard room.
    pub guard_before_locks: bool,
    /// Role taken when no other one respects the rules.
    pub fallback: String,
    pub roles: BTreeMap<String, RoleRule>,
}

/// The settings of the generator.
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub render: RenderSettings,
    pub sizes: BTreeMap<String, GraphShape>,
    /// Roles of the rooms of the branches; a repeated role is more frequent.
    pub branch_roles: Vec<String>,
    pub rules: Rules,
    /// The room at the end of the main path, by objective.
    pub goal_roles: BTreeMap<String, String>,
    pub assembly: super::assemble::AssemblySettings,
    /// People and loot by role of room (absent: an empty room).
    pub population: BTreeMap<String, RoomPopulation>,
    /// People of the corridors.
    pub corridors: CorridorPopulation,
    /// Who lives in the dungeon: Veloren creatures for each kind of the plans.
    pub factions: BTreeMap<String, Faction>,
    pub default_faction: String,
    /// What region, season, threat, enemies and objective change.
    pub parameters: Parameters,
    /// The keys of the locked doors, one for each lock of a dungeon (in the
    /// order of the locks).
    pub keys: Vec<KeySettings>,
    /// How asleep creatures wake up.
    pub sleep: SleepSettings,
    /// How lit the rooms are, by their inhabitants.
    pub lighting: LightingSettings,
    /// Variety of the rooms (Icebreaker F.2).
    pub variety: VarietySettings,
    /// Decor by role of room (Icebreaker F.6).
    pub decor: DecorSettings,
    /// Where the dungeon may lie in the world (Icebreaker F.8).
    pub placement: PlacementSettings,
}

/// Icebreaker F.8: a dungeon that goes past the edge of the world cannot be
/// finished (the chunks there are never generated). Its entrance is placed
/// `entrance_margin` chunks at least from the edges, and a dungeon that comes
/// closer than `edge_margin` chunks to an edge is generated again with another
/// seed, `fit_tries` times at most.
#[derive(Clone, Debug, Deserialize)]
pub struct PlacementSettings {
    pub entrance_margin: i32,
    pub edge_margin: i32,
    pub fit_tries: u64,
    /// Icebreaker I.11: chunks kept around the entrance, on each side, for
    /// the dungeon and the ones drawn again in its place.
    pub emprise: i32,
}

/// Variety of the rooms (Icebreaker F.2 and F.5): the orientations of each drawn
/// room (1 = as drawn, 2 = and mirrored, 4 = quarter turns, 8 = both), and the
/// generated rooms of varied shapes.
#[derive(Clone, Debug, Deserialize)]
pub struct VarietySettings {
    pub orientations: u8,
    pub shapes: super::shapes::ShapeSettings,
}

/// Lights of the dungeon: the chance that a free floor cell holds one, by role
/// of room (`default` for the others), in corridors, and how much less in a
/// room without creatures; the share of wall torches (the rest are braziers).
#[derive(Clone, Debug, Deserialize)]
pub struct LightingSettings {
    pub roles: BTreeMap<String, f32>,
    pub default: f32,
    pub corridors: f32,
    pub empty_factor: f32,
    pub wall_torches: f32,
}

impl LightingSettings {
    /// The chance of a light on a free floor cell of a room of this role
    /// (`None`: a corridor) with or without creatures.
    pub fn chance(&self, role: Option<&str>, inhabited: bool) -> f32 {
        let Some(role) = role else { return self.corridors };
        let chance = self.roles.get(role).copied().unwrap_or(self.default);
        if inhabited { chance } else { chance * self.empty_factor }
    }
}

/// How asleep creatures wake up (distances in blocks, time in seconds).
#[derive(Clone, Debug, Deserialize)]
pub struct SleepSettings {
    pub wake_radius: f32,
    pub wake_radius_sneaking: f32,
    pub alert_radius: f32,
    pub zzz_interval: f64,
}

/// A key of a locked door: the Veloren item its keyhole consumes, and the loot
/// table of the chest holding it.
#[derive(Clone, Debug, Deserialize)]
pub struct KeySettings {
    pub item: String,
    pub loot: String,
}

/// People and loot of a room: creatures by kind (`brute`, `archer`,
/// `lookout`, `chief`, `boss`), chests, awake or asleep.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct RoomPopulation {
    pub creatures: BTreeMap<String, Range>,
    pub chests: Range,
    pub awake: bool,
}

impl Default for RoomPopulation {
    fn default() -> Self {
        Self {
            creatures: BTreeMap::new(),
            chests: Range(0, 0),
            awake: true,
        }
    }
}

/// People of the corridors (Icebreaker F.4): for each `per_cells` cells of
/// corridor, these creatures (by kind).
#[derive(Clone, Debug, Deserialize)]
pub struct CorridorPopulation {
    pub creatures: BTreeMap<String, Range>,
    pub per_cells: usize,
}

/// The Veloren creatures (entity configs) of each kind, and the chest sprite.
#[derive(Clone, Debug, Deserialize)]
pub struct Faction {
    pub creatures: BTreeMap<String, Vec<String>>,
    pub chest: String,
}

fn role(name: &str, errors: &mut Vec<String>) -> RoomRole {
    RoomRole::parse(name).unwrap_or_else(|| {
        errors.push(format!("rôle inconnu « {name} »"));
        RoomRole::GuardRoom
    })
}

impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let settings: Self =
            ron::from_str(text).map_err(|e| format!("generation.ron illisible : {e}"))?;
        let mut errors = Vec::new();
        let render = &settings.render;
        if render.cell_size < 3 || render.level_height < 3 || render.level_height > render.cell_size
        {
            errors.push("render : il faut 3 ≤ level_height ≤ cell_size".to_string());
        }
        let placement = &settings.placement;
        if placement.entrance_margin < 0 || placement.edge_margin < 0 || placement.fit_tries < 1 || placement.emprise < 1 {
            errors.push("placement : marges positives, fit_tries et emprise au moins 1".into());
        }
        if render.tunnel_cells < 1 {
            errors.push("render : tunnel_cells doit être au moins 1".into());
        }
        if ![1, 2, 4, 8].contains(&settings.variety.orientations) {
            errors.push("variety : orientations vaut 1, 2, 4 ou 8".into());
        }
        errors.extend(settings.variety.shapes.errors().into_iter().map(|e| format!("variety.shapes : {e}")));
        errors.extend(settings.decor.errors().into_iter().map(|e| format!("decor : {e}")));
        for (name, shape) in &settings.sizes {
            let ranges = [
                ("main_path", shape.main_path, 2),
                ("levels", shape.levels, 1),
                ("branches", shape.branches, 0),
                ("branch_length", shape.branch_length, 1),
                ("locks", shape.locks, 0),
                ("loops", shape.loops, 0),
                ("drops", shape.drops, 0),
            ];
            for (field, range, lowest) in ranges {
                if !range.valid(lowest) {
                    errors.push(format!("sizes.{name}.{field} : il faut {lowest} ≤ min ≤ max"));
                }
            }
            if shape.max_branches_per_room == 0 {
                errors.push(format!("sizes.{name}.max_branches_per_room : au moins 1"));
            }
            if shape.locks.1.max(0) as usize > settings.keys.len() {
                errors.push(format!("sizes.{name}.locks : plus de verrous que de clés (keys)"));
            }
        }
        let items: std::collections::BTreeSet<&String> = settings.keys.iter().map(|k| &k.item).collect();
        if items.len() < settings.keys.len() {
            errors.push("keys : une clé différente par porte, deux fois le même objet".into());
        }
        for name in settings
            .branch_roles
            .iter()
            .chain(std::iter::once(&settings.rules.fallback))
            .chain(settings.rules.roles.keys())
            .chain(settings.rules.roles.values().flat_map(|r| &r.next_to))
            .chain(settings.goal_roles.values())
        {
            role(name, &mut errors);
        }
        for name in settings.lighting.roles.keys() {
            role(name, &mut errors);
        }
        for (name, population) in &settings.population {
            role(name, &mut errors);
            for (kind, range) in &population.creatures {
                if super::populate::KINDS.iter().all(|(k, _)| k != kind) {
                    errors.push(format!("population.{name} : sorte de créature inconnue « {kind} »"));
                }
                if !range.valid(0) {
                    errors.push(format!("population.{name}.{kind} : il faut 0 ≤ min ≤ max"));
                }
            }
        }
        if settings.corridors.per_cells == 0 {
            errors.push("corridors : per_cells doit être au moins 1".into());
        }
        for (name, faction) in &settings.factions {
            for (kind, _) in super::populate::KINDS {
                if faction.creatures.get(kind).is_none_or(Vec::is_empty) {
                    errors.push(format!("factions.{name} : aucune créature « {kind} »"));
                }
            }
        }
        let p = &settings.parameters;
        let tables = [
            ("regions", &p.regions),
            ("seasons", &p.seasons),
            ("threats", &p.threats),
            ("densities", &p.densities),
            ("objectives", &p.objectives),
        ];
        for (table, modifiers) in tables {
            for (key, m) in modifiers {
                for name in m.rooms.keys() {
                    role(name, &mut errors);
                }
                for kind in m.creatures.keys() {
                    if super::populate::KINDS.iter().all(|(k, _)| k != kind) {
                        errors.push(format!("parameters.{table}.{key} : sorte de créature inconnue « {kind} »"));
                    }
                }
                if m.creature_factor < 0.0 || m.chests < 0.0 || m.rooms.values().chain(m.creatures.values()).any(|f| *f < 0.0) {
                    errors.push(format!("parameters.{table}.{key} : les facteurs doivent être ≥ 0"));
                }
            }
        }
        if !p.threats.contains_key(&p.default_threat.to_string()) {
            errors.push(format!("parameters.default_threat ({}) absente de threats", p.default_threat));
        }
        if !p.densities.contains_key(&p.default_density) {
            errors.push(format!("parameters.default_density ({}) absente de densities", p.default_density));
        }
        if !settings.factions.contains_key(&settings.default_faction) {
            errors.push(format!("default_faction : faction inconnue « {} »", settings.default_faction));
        }
        if errors.is_empty() {
            Ok(settings)
        } else {
            Err(format!("generation.ron : {}", errors.join(" ; ")))
        }
    }

    pub fn load() -> Result<Self, String> { Self::parse(&super::read_data("generation.ron")?) }

    /// The rooms a dungeon of `seed` is built from: the drawn ones in every
    /// orientation, and the generated ones of this seed (Icebreaker F.2, F.5).
    pub fn catalog(&self, seed: u64) -> Result<super::rooms::RoomCatalog, String> {
        super::rooms::RoomCatalog::load()?
            .with_orientations(self.variety.orientations)?
            .with_shapes(&self.variety.shapes, seed)
    }

    /// The modifiers of a mission: its region, season, threat, number of
    /// enemies and objective. Unknown names are errors.
    fn modifiers(&self, mission: &Mission) -> Result<Vec<(String, &Modifier)>, String> {
        fn find<'a>(
            table: &'a BTreeMap<String, Modifier>,
            key: &str,
            what: &str,
        ) -> Result<(String, &'a Modifier), String> {
            table.get(key).map(|m| (key.to_string(), m)).ok_or_else(|| {
                format!(
                    "{what} inconnue : {key} (connues : {})",
                    table.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            })
        }
        let p = &self.parameters;
        let mut modifiers = Vec::new();
        if let Some(region) = &mission.region {
            modifiers.push(find(&p.regions, region, "région")?);
        }
        if let Some(season) = &mission.season {
            modifiers.push(find(&p.seasons, season, "saison")?);
        }
        let threat = mission.threat.unwrap_or(p.default_threat).to_string();
        modifiers.push(find(&p.threats, &threat, "menace")?);
        let density = mission.density.as_deref().unwrap_or(&p.default_density);
        modifiers.push(find(&p.densities, density, "quantité d'ennemis")?);
        if let Some(objective) = p.objectives.get(&mission.objective) {
            modifiers.push((mission.objective.clone(), objective));
        }
        Ok(modifiers)
    }

    /// The settings of a mission: the effects of its region, season, threat,
    /// number of enemies and objective applied to the shape of the graph, the
    /// weights of the roles of the branches and the people. Shifts add up,
    /// factors multiply; the chief and the boss stay unique.
    pub fn resolve(&self, mission: &Mission) -> Result<Resolved, String> {
        let mut errors = Vec::new();
        let modifiers = self.modifiers(mission)?;
        let shape = self.sizes.get(&mission.size).cloned().ok_or_else(|| {
            format!(
                "taille de donjon inconnue : {} (tailles : {})",
                mission.size,
                self.sizes.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?;
        let goal = self.goal_roles.get(&mission.objective).ok_or_else(|| {
            format!(
                "objectif inconnu : {} (objectifs : {})",
                mission.objective,
                self.goal_roles.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?;
        let goal = role(goal, &mut errors);
        let shift = |range: Range, delta: i32, lowest: i32| {
            Range((range.0 + delta).max(lowest), (range.1 + delta).max(lowest))
        };
        let sum = |f: fn(&Modifier) -> i32| modifiers.iter().map(|(_, m)| f(m)).sum::<i32>();
        let mut shape = shape;
        shape.main_path = shift(shape.main_path, sum(|m| m.main_path), 2);
        let levels = shift(shape.levels, sum(|m| m.levels), 1);
        // At least one room per level.
        shape.levels = Range(levels.0.min(shape.main_path.0), levels.1.min(shape.main_path.0));
        shape.branches = shift(shape.branches, sum(|m| m.branches), 0);

        let mut weights = BTreeMap::new();
        for name in &self.branch_roles {
            *weights.entry(role(name, &mut errors)).or_insert(0.0) += 1.0;
        }
        for (role, weight) in weights.iter_mut() {
            for (_, m) in &modifiers {
                *weight *= m.rooms.get(role.name()).copied().unwrap_or(1.0);
            }
        }
        let creature_factor: f64 = modifiers.iter().map(|(_, m)| m.creature_factor).product();
        let chest_factor: f64 = modifiers.iter().map(|(_, m)| m.chests).product();
        let scale = |range: Range, factor: f64| {
            Range((range.0 as f64 * factor).round() as i32, (range.1 as f64 * factor).round() as i32)
        };
        let rules = self
            .rules
            .roles
            .iter()
            .map(|(name, rule)| {
                let next_to = rule.next_to.iter().map(|n| role(n, &mut errors)).collect();
                (role(name, &mut errors), ResolvedRule {
                    next_to,
                    min_distance_from_main_path: rule.min_distance_from_main_path.max(1),
                    max: rule.max,
                })
            })
            .collect();
        let fallback = role(&self.rules.fallback, &mut errors);
        let population = self
            .population
            .iter()
            .map(|(name, population)| {
                let mut population = population.clone();
                for (kind, range) in population.creatures.iter_mut() {
                    if kind != "chief" && kind != "boss" {
                        let factor = modifiers.iter().fold(creature_factor, |f, (_, m)| {
                            f * m.creatures.get(kind).copied().unwrap_or(1.0)
                        });
                        *range = scale(*range, factor);
                    }
                }
                population.chests = scale(population.chests, chest_factor);
                (role(name, &mut errors), population)
            })
            .collect();
        let mut corridors = self.corridors.clone();
        for (kind, range) in corridors.creatures.iter_mut() {
            let factor = modifiers
                .iter()
                .fold(creature_factor, |f, (_, m)| f * m.creatures.get(kind).copied().unwrap_or(1.0));
            *range = scale(*range, factor);
        }
        let names: Vec<String> = modifiers
            .iter()
            .map(|(key, m)| if m.name.is_empty() { key.clone() } else { m.name.clone() })
            .filter(|n| !n.is_empty())
            .collect();
        let faction_name = mission.faction.clone().unwrap_or_else(|| self.default_faction.clone());
        let faction = self.factions.get(&faction_name).cloned().ok_or_else(|| {
            format!(
                "faction inconnue : {faction_name} (factions : {})",
                self.factions.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?;
        if !errors.is_empty() {
            return Err(errors.join(" ; "));
        }
        Ok(Resolved {
            shape,
            goal,
            weights,
            rules,
            guard_before_locks: self.rules.guard_before_locks,
            fallback,
            population,
            corridors,
            faction,
            modifiers: names,
        })
    }
}

/// What a parameter of the mission changes. `main_path`, `levels`,
/// `branches`: rooms of the main path, levels, branches in more (or less);
/// `rooms`: factor of the weight of a role of the branches (0: never);
/// `creature_factor`: factor of the number of creatures (the chief stays
/// unique); `creatures`: factor by kind; `chests`: factor of the chests.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Modifier {
    /// Shown in the summary of the dungeon (empty: its key).
    pub name: String,
    pub main_path: i32,
    pub levels: i32,
    pub branches: i32,
    pub rooms: BTreeMap<String, f64>,
    pub creature_factor: f64,
    pub creatures: BTreeMap<String, f64>,
    pub chests: f64,
}

impl Default for Modifier {
    fn default() -> Self {
        Self {
            name: String::new(),
            main_path: 0,
            levels: 0,
            branches: 0,
            rooms: BTreeMap::new(),
            creature_factor: 1.0,
            creatures: BTreeMap::new(),
            chests: 1.0,
        }
    }
}

/// `generation.ron` → `parameters`.
#[derive(Clone, Debug, Deserialize)]
pub struct Parameters {
    pub regions: BTreeMap<String, Modifier>,
    pub seasons: BTreeMap<String, Modifier>,
    /// From 1 (low) to 5 (extreme), as text keys.
    pub threats: BTreeMap<String, Modifier>,
    pub default_threat: i32,
    pub densities: BTreeMap<String, Modifier>,
    pub default_density: String,
    pub objectives: BTreeMap<String, Modifier>,
}

/// What the generator is asked for. The same mission always gives the same
/// dungeon.
#[derive(Clone, Debug)]
pub struct Mission {
    pub seed: u64,
    /// `small`, `medium`, `large`.
    pub size: String,
    /// `killChief`, `rescue`, `loot`, `clear`, `explore`, `bossLair`.
    pub objective: String,
    /// Who lives there (`factions` of `generation.ron`; none: the default one).
    pub faction: Option<String>,
    /// `forest`, `mountain`, `swamp` (none: no effect).
    pub region: Option<String>,
    /// `spring`, `summer`, `autumn`, `winter` (none: no effect).
    pub season: Option<String>,
    /// 1 to 5 (none: the default threat).
    pub threat: Option<i32>,
    /// Number of enemies: `low`, `medium`, `high` (none: the default one).
    pub density: Option<String>,
}

impl Default for Mission {
    fn default() -> Self {
        Self {
            seed: 0,
            size: "medium".into(),
            objective: "killChief".into(),
            faction: None,
            region: None,
            season: None,
            threat: None,
            density: None,
        }
    }
}

impl Mission {
    /// The mission given by the `ICEBREAKER_DONJON_*` variables (set by the
    /// launcher: `-Taille`, `-Objectif`, `-Region`, `-Saison`, `-Menace`,
    /// `-Ennemis`, `-Faction`); unset or empty: the default.
    pub fn from_env(seed: u64) -> Result<Self, String> {
        let var = |name: &str| {
            std::env::var(format!("ICEBREAKER_DONJON_{name}"))
                .ok()
                .filter(|v| !v.is_empty())
        };
        let default = Self::default();
        let threat = match var("MENACE") {
            Some(t) => Some(t.parse().map_err(|_| format!("menace illisible : {t}"))?),
            None => None,
        };
        Ok(Self {
            seed,
            size: var("TAILLE").unwrap_or(default.size),
            objective: var("OBJECTIF").unwrap_or(default.objective),
            faction: var("FACTION"),
            region: var("REGION"),
            season: var("SAISON"),
            threat,
            density: var("ENNEMIS"),
        })
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedRule {
    pub next_to: Vec<RoomRole>,
    pub min_distance_from_main_path: i32,
    pub max: Option<usize>,
}

/// The settings of one mission.
#[derive(Clone, Debug)]
pub struct Resolved {
    pub shape: GraphShape,
    pub goal: RoomRole,
    /// Weight of each role of the branches.
    pub weights: BTreeMap<RoomRole, f64>,
    pub rules: BTreeMap<RoomRole, ResolvedRule>,
    pub guard_before_locks: bool,
    pub fallback: RoomRole,
    pub population: BTreeMap<RoomRole, RoomPopulation>,
    pub corridors: CorridorPopulation,
    pub faction: Faction,
    /// Names of the parameters applied, for the summary.
    pub modifiers: Vec<String>,
}

/// Decor by role of room (Icebreaker F.6): sprites of Veloren placed in the
/// rooms and the corridors.
#[derive(Clone, Debug, Deserialize)]
pub struct DecorSettings {
    pub roles: BTreeMap<String, Vec<DecorItem>>,
    pub corridors: Vec<DecorItem>,
}

/// Where a piece of decor goes: against a wall (facing the room), in the
/// middle of a cell, or scattered over the floor blocks (`chance` per block).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecorPlace {
    Wall,
    Center,
    Scatter,
}

/// A piece of decor: a sprite of Veloren (its name), where it goes, how many
/// per room (`count`, for `wall` and `center`) or how often per floor block
/// (`chance`, for `scatter`); `around`: a sprite on the four blocks around
/// (chairs around a table).
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct DecorItem {
    pub sprite: String,
    pub at: DecorPlace,
    pub count: Range,
    pub chance: f32,
    pub around: Option<String>,
}

impl Default for DecorItem {
    fn default() -> Self {
        Self {
            sprite: String::new(),
            at: DecorPlace::Center,
            count: Range(1, 1),
            chance: 0.0,
            around: None,
        }
    }
}

impl DecorSettings {
    pub fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let items = self.roles.iter().flat_map(|(role, items)| items.iter().map(move |i| (role.as_str(), i)));
        for (role, item) in items.chain(self.corridors.iter().map(|i| ("corridors", i))) {
            for name in std::iter::once(&item.sprite).chain(&item.around) {
                if sprite_kind(name).is_none() {
                    errors.push(format!("{role} : sprite inconnu « {name} »"));
                }
            }
            if item.at == DecorPlace::Scatter && !(0.0..=1.0).contains(&item.chance) {
                errors.push(format!("{role} : chance de « {} » entre 0 et 1", item.sprite));
            }
        }
        for role in self.roles.keys() {
            if super::rooms::RoomRole::parse(role).is_none() {
                errors.push(format!("rôle inconnu : {role}"));
            }
        }
        errors
    }
}

/// The sprite of Veloren named in the data.
pub fn sprite_kind(name: &str) -> Option<common::terrain::SpriteKind> { ron::from_str(name).ok() }
