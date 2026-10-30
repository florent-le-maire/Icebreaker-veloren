//! Icebreaker: hand-written rooms (`assets/icebreaker/donjons/salles/`), one
//! per file, in the plan format with a `@roles:` line, possible doors `+` in
//! the outer walls and possible pits `*` in the floor
//! (`docs/icebreaker/design/generation.md`, « Salles préfaites »).

use super::plan::{self, Cell, DIRECTIONS, Plan};
use serde::Deserialize;
use std::collections::BTreeSet;

/// The part a room plays in the life of the dungeon.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoomRole {
    Entrance,
    /// A room on the way, on the main path.
    Hall,
    GuardRoom,
    Barracks,
    Storage,
    Kitchen,
    Nest,
    Treasure,
    ChiefLair,
    Prison,
    Vault,
    Sanctum,
    /// The arena of a boss; the room tells which bosses (`@boss:`).
    BossLair,
}

impl RoomRole {
    pub const ALL: [RoomRole; 13] = [
        RoomRole::Entrance,
        RoomRole::Hall,
        RoomRole::GuardRoom,
        RoomRole::Barracks,
        RoomRole::Storage,
        RoomRole::Kitchen,
        RoomRole::Nest,
        RoomRole::Treasure,
        RoomRole::ChiefLair,
        RoomRole::Prison,
        RoomRole::Vault,
        RoomRole::Sanctum,
        RoomRole::BossLair,
    ];

    /// The name of the role in the data files (`guardRoom`, `barracks`…).
    pub fn name(self) -> &'static str {
        match self {
            RoomRole::Entrance => "entrance",
            RoomRole::Hall => "hall",
            RoomRole::GuardRoom => "guardRoom",
            RoomRole::Barracks => "barracks",
            RoomRole::Storage => "storage",
            RoomRole::Kitchen => "kitchen",
            RoomRole::Nest => "nest",
            RoomRole::Treasure => "treasure",
            RoomRole::ChiefLair => "chiefLair",
            RoomRole::Prison => "prison",
            RoomRole::Vault => "vault",
            RoomRole::Sanctum => "sanctum",
            RoomRole::BossLair => "bossLair",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.name().eq_ignore_ascii_case(name))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConnectorKind {
    /// `+`: a door in an outer wall.
    Door,
    /// `*`: a pit in the floor, toward a lower room.
    Pit,
}

/// A connector of a room: `inside` is the cell of the room one goes through,
/// `outside` the cell out of the room the generator must reach (in front of
/// the door, or under the pit).
#[derive(Copy, Clone, Debug)]
pub struct Connector {
    pub kind: ConnectorKind,
    pub cell: Cell,
    pub inside: Cell,
    pub outside: Cell,
}

impl Connector {
    /// The direction out of a door, as (dx, dy).
    pub fn direction(&self) -> (i32, i32) {
        (self.outside.x - self.cell.x, self.outside.y - self.cell.y)
    }
}

/// A hand-written room.
#[derive(Clone, Debug)]
pub struct RoomTemplate {
    pub id: String,
    pub roles: Vec<RoomRole>,
    pub bosses: Vec<String>,
    pub plan: Plan,
    pub connectors: Vec<Connector>,
}

fn describe(c: Cell) -> String {
    format!("étage {}, ligne {}, colonne {}", c.level, c.y + 1, c.x + 1)
}

impl RoomTemplate {
    /// Reads and checks a room; all its errors in one message.
    pub fn parse(id: &str, text: &str) -> Result<Self, String> {
        let mut errors = Vec::new();
        let mut roles = Vec::new();
        let mut bosses = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let Some(setting) = line.strip_prefix('@') else {
                continue;
            };
            let (key, value) = setting.split_once(':').unwrap_or((setting, ""));
            let names = value.split(',').map(str::trim).filter(|n| !n.is_empty());
            match key.trim() {
                "roles" => {
                    for name in names {
                        match RoomRole::parse(name) {
                            Some(role) => roles.push(role),
                            None => errors.push(format!("rôle inconnu « {name} »")),
                        }
                    }
                },
                "boss" => bosses.extend(names.map(String::from)),
                key => errors.push(format!("ligne {} : réglage inconnu « @{key} »", i + 1)),
            }
        }
        if roles.is_empty() && errors.is_empty() {
            errors.push("aucun rôle (ligne « @roles: … »)".into());
        }
        if roles.contains(&RoomRole::BossLair) != !bosses.is_empty() {
            errors.push(
                "un antre de boss (bossLair) déclare les boss qu'il accueille (« @boss: … »), et \
                 seul un antre le fait"
                    .into(),
            );
        }

        let plan = Plan::parse(text).map_err(|e| format!("salle « {id} » : {e}"))?;
        let mut connectors = Vec::new();
        for (&cell, &symbol) in &plan.cells {
            let place = format!("connecteur ({})", describe(cell));
            match symbol {
                '*' => {
                    let below = cell.offset(0, 0, -1);
                    if !plan.is_free(below) {
                        errors.push(format!(
                            "{place} : sous une fosse '*', la case doit être hors de la salle (vide)"
                        ));
                    }
                    connectors.push(Connector {
                        kind: ConnectorKind::Pit,
                        cell,
                        inside: cell,
                        outside: below,
                    });
                },
                '+' => {
                    let outward: Vec<_> = DIRECTIONS
                        .into_iter()
                        .filter(|(dx, dy)| plan.is_free(cell.offset(*dx, *dy, 0)))
                        .collect();
                    let [(dx, dy)] = outward[..] else {
                        errors.push(format!(
                            "{place} : une porte '+' doit être dans un mur extérieur, avec le dehors \
                             d'un seul côté"
                        ));
                        continue;
                    };
                    let inside = cell.offset(-dx, -dy, 0);
                    if !plan::is_walkable(plan.get(inside)) {
                        errors.push(format!("{place} : il faut du sol juste à l'intérieur de la porte"));
                    }
                    connectors.push(Connector {
                        kind: ConnectorKind::Door,
                        cell,
                        inside,
                        outside: cell.offset(dx, dy, 0),
                    });
                },
                _ => {},
            }
        }
        if connectors.iter().all(|c| c.kind != ConnectorKind::Door) {
            errors.push("il faut au moins un connecteur de porte '+'".into());
        }
        let has_spawn = plan.cells.values().any(|s| *s == 'S');
        if roles.contains(&RoomRole::Entrance) != has_spawn {
            errors.push(
                "les points d'apparition 'S' sont réservés aux salles d'entrée, et une entrée en a au \
                 moins un"
                    .into(),
            );
        }

        // Navigable: from each connector, all the others; from the first door, the whole room.
        if errors.is_empty() {
            let first = connectors.iter().find(|c| c.kind == ConnectorKind::Door).unwrap();
            let reached = plan.reachable(first.inside);
            for (&cell, &symbol) in &plan.cells {
                let practicable = plan::is_walkable(symbol) || plan::stairs_direction(symbol).is_some();
                if practicable && !reached.contains(&cell) {
                    errors.push(format!(
                        "case inaccessible depuis le connecteur ({}) : {}",
                        describe(first.cell),
                        describe(cell)
                    ));
                }
            }
            for from in &connectors {
                let reached = plan.reachable(from.inside);
                for to in connectors.iter().filter(|to| !reached.contains(&to.inside)) {
                    errors.push(format!(
                        "impossible d'aller du connecteur ({}) au connecteur ({})",
                        describe(from.cell),
                        describe(to.cell)
                    ));
                }
            }
        }

        if !errors.is_empty() {
            errors.dedup();
            return Err(format!("salle « {id} » invalide : {}.", errors.join(" ; ")));
        }
        Ok(Self {
            id: id.to_string(),
            roles,
            bosses,
            plan,
            connectors,
        })
    }

    pub fn doors(&self) -> impl Iterator<Item = &Connector> {
        self.connectors.iter().filter(|c| c.kind == ConnectorKind::Door)
    }

    pub fn door_count(&self) -> usize { self.doors().count() }
}

/// The hand-written rooms, sorted by id (a seed gives the same dungeon
/// whatever the order the files are read in).
#[derive(Clone, Debug)]
pub struct RoomCatalog {
    pub rooms: Vec<RoomTemplate>,
}

impl RoomCatalog {
    /// Reads the rooms given as (id, text); the errors of all rooms in one message.
    pub fn parse(files: impl IntoIterator<Item = (String, String)>) -> Result<Self, String> {
        let mut rooms = Vec::new();
        let mut errors = Vec::new();
        for (id, text) in files {
            match RoomTemplate::parse(&id, &text) {
                Ok(room) => rooms.push(room),
                Err(e) => errors.push(e),
            }
        }
        let mut ids = BTreeSet::new();
        for room in &rooms {
            if !ids.insert(room.id.clone()) {
                errors.push(format!("salle « {} » en double", room.id));
            }
        }
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }
        rooms.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self { rooms })
    }

    /// Reads `assets/icebreaker/donjons/salles/*.txt`.
    pub fn load() -> Result<Self, String> {
        let dir = super::data_path("salles");
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("{} : {e}", dir.display()))?;
        let mut files = Vec::new();
        for entry in entries {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| e == "txt") {
                let id = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("{} : {e}", path.display()))?;
                files.push((id, text));
            }
        }
        Self::parse(files)
    }

    pub fn with_role(&self, role: RoomRole) -> Vec<&RoomTemplate> {
        self.rooms.iter().filter(|r| r.roles.contains(&role)).collect()
    }

    pub fn get(&self, id: &str) -> Option<&RoomTemplate> { self.rooms.iter().find(|r| r.id == id) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rooms_are_valid() {
        let catalog = RoomCatalog::load().unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(catalog.rooms.len(), 17);
        // Every role of a branch or of the main path has a room.
        for role in RoomRole::ALL {
            assert!(!catalog.with_role(role).is_empty(), "aucune salle pour {}", role.name());
        }
        // A room in the middle of a branch or of the main path needs two doors.
        for role in [RoomRole::Hall, RoomRole::GuardRoom, RoomRole::Barracks, RoomRole::Kitchen] {
            assert!(
                catalog.with_role(role).iter().any(|r| r.door_count() >= 2),
                "{} : aucune salle à deux portes",
                role.name()
            );
        }
    }

    #[test]
    fn errors_are_explained() {
        let e = RoomTemplate::parse("x", "@roles: hall\n#####\n#.#.#\n#+###\n").unwrap_err();
        assert!(e.contains("case inaccessible"), "{e}");
        let e = RoomTemplate::parse("x", "@roles: dragon\n###\n+.#\n###\n").unwrap_err();
        assert!(e.contains("rôle inconnu"), "{e}");
        let e = RoomTemplate::parse("x", "@roles: hall\n###\n#S+\n###\n").unwrap_err();
        assert!(e.contains("apparition"), "{e}");
        let e = RoomTemplate::parse("x", "@roles: hall\n###\n#.#\n###\n").unwrap_err();
        assert!(e.contains("au moins un connecteur"), "{e}");
    }

    #[test]
    fn connectors_point_outward() {
        let catalog = RoomCatalog::load().unwrap();
        let entrance = catalog.get("entree").unwrap();
        assert_eq!(entrance.door_count(), 3);
        for door in entrance.doors() {
            assert!(entrance.plan.is_free(door.outside));
            assert!(plan::is_walkable(entrance.plan.get(door.inside)));
        }
        let nest = catalog.get("nid").unwrap();
        assert!(nest.connectors.iter().any(|c| c.kind == ConnectorKind::Pit));
    }
}
