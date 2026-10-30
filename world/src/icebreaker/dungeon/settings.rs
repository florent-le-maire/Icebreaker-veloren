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
    /// Who lives in the dungeon: Veloren creatures for each kind of the plans.
    pub factions: BTreeMap<String, Faction>,
    pub default_faction: String,
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
        if render.tunnel_cells < 1 {
            errors.push("render : tunnel_cells doit être au moins 1".into());
        }
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
        for (name, faction) in &settings.factions {
            for (kind, _) in super::populate::KINDS {
                if faction.creatures.get(kind).is_none_or(Vec::is_empty) {
                    errors.push(format!("factions.{name} : aucune créature « {kind} »"));
                }
            }
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

    /// The settings of a mission (task G.6 adds region, season, threat…).
    pub fn resolve(&self, mission: &Mission) -> Result<Resolved, String> {
        let mut errors = Vec::new();
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
        let mut weights = BTreeMap::new();
        for name in &self.branch_roles {
            *weights.entry(role(name, &mut errors)).or_insert(0.0) += 1.0;
        }
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
            .map(|(name, population)| (role(name, &mut errors), population.clone()))
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
            faction,
        })
    }
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
}

impl Default for Mission {
    fn default() -> Self {
        Self {
            seed: 0,
            size: "medium".into(),
            objective: "killChief".into(),
            faction: None,
        }
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
    pub faction: Faction,
}
