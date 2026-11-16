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
    /// Icebreaker : the styles of dungeon the room fits (`@theme:`, none = any) and its weight in the draw
    /// (`@poids:`, 1 by default).
    pub themes: Vec<String>,
    pub weight: f64,
    /// Icebreaker : what the room is for (`@fonction:`, an entry of `donjons/pieces.ron`; none for entrances and
    /// halls). Finer than the role: the role says who lives there, the function what the room looks like.
    pub function: Option<String>,
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
        let mut themes = Vec::new();
        let mut weight = 1.0;
        let mut function = None;
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
                "theme" => themes.extend(names.map(|n| n.to_lowercase())),
                "fonction" => function = Some(value.trim().to_lowercase()).filter(|f| !f.is_empty()),
                "poids" => match value.trim().parse::<f64>() {
                    Ok(w) if w > 0.0 => weight = w,
                    _ => errors.push(format!("ligne {} : « @poids » doit être un nombre positif", i + 1)),
                },
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
        Self::from_plan(id, roles, bosses, errors, plan).map(|room| Self { themes, weight, function, ..room })
    }

    /// Checks a plan and finds its connectors (`errors`: those of the header,
    /// already found).
    pub(super) fn from_plan(
        id: &str,
        roles: Vec<RoomRole>,
        bosses: Vec<String>,
        mut errors: Vec<String>,
        plan: Plan,
    ) -> Result<Self, String> {
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
            themes: Vec::new(),
            weight: 1.0,
            function: None,
            plan,
            connectors,
        })
    }

    pub fn doors(&self) -> impl Iterator<Item = &Connector> {
        self.connectors.iter().filter(|c| c.kind == ConnectorKind::Door)
    }

    pub fn door_count(&self) -> usize { self.doors().count() }

    /// The room turned (Icebreaker F.2, `RoomOrientation.Transform` of the
    /// project): 0 to 3 = quarter turns clockwise (seen from above, north up),
    /// 4 to 7 = the same after a left-right mirror. Stairs turn with the room;
    /// all the levels turn around the same frame. Its id becomes `id:n`.
    pub fn oriented(&self, orientation: u8) -> Result<Self, String> {
        if orientation == 0 {
            return Ok(self.clone());
        }
        let (min_x, max_x, min_y, max_y) = self.plan.bounds().ok_or("salle vide")?;
        let (mut width, mut depth) = (max_x - min_x + 1, max_y - min_y + 1);
        let mut cells: Vec<(Cell, char)> = self
            .plan
            .cells
            .iter()
            .map(|(c, s)| (Cell::new(c.x - min_x, c.y - min_y, c.level), *s))
            .collect();
        if orientation >= 4 {
            for (c, s) in &mut cells {
                c.x = width - 1 - c.x;
                *s = match *s {
                    '<' => '>',
                    '>' => '<',
                    s => s,
                };
            }
        }
        for _ in 0..orientation % 4 {
            for (c, s) in &mut cells {
                *c = Cell::new(depth - 1 - c.y, c.x, c.level);
                *s = match *s {
                    '^' => '>',
                    '>' => 'v',
                    'v' => '<',
                    '<' => '^',
                    s => s,
                };
            }
            std::mem::swap(&mut width, &mut depth);
        }
        let plan = Plan {
            cells: cells.into_iter().collect(),
        };
        Self::from_plan(&format!("{}:{orientation}", self.id), self.roles.clone(), self.bosses.clone(), Vec::new(), plan).map(|room| Self {
            themes: self.themes.clone(),
            weight: self.weight,
            function: self.function.clone(),
            ..room
        })
    }
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

    /// Icebreaker : the themes the rooms name (sorted, so a seed draws the same one whatever the files).
    pub fn themes(&self) -> BTreeSet<String> {
        self.rooms.iter().flat_map(|r| r.themes.iter().cloned()).collect()
    }

    pub fn with_role(&self, role: RoomRole) -> Vec<&RoomTemplate> {
        self.rooms.iter().filter(|r| r.roles.contains(&role)).collect()
    }

    /// The rooms in every orientation asked (Icebreaker F.2, `orientations` of
    /// the settings): 1 = as drawn, 2 = and mirrored, 4 = the quarter turns,
    /// 8 = turns and mirrors. The drawn room keeps its id.
    pub fn with_orientations(mut self, count: u8) -> Result<Self, String> {
        let orientations: &[u8] = match count {
            1 => &[0],
            2 => &[0, 4],
            4 => &[0, 1, 2, 3],
            8 => &[0, 1, 2, 3, 4, 5, 6, 7],
            _ => return Err(format!("variety.orientations : 1, 2, 4 ou 8, pas {count}")),
        };
        let mut rooms = Vec::new();
        for room in &self.rooms {
            for orientation in orientations {
                rooms.push(room.oriented(*orientation)?);
            }
        }
        rooms.sort_by(|a, b| a.id.cmp(&b.id));
        self.rooms = rooms;
        Ok(self)
    }

    pub fn get(&self, id: &str) -> Option<&RoomTemplate> { self.rooms.iter().find(|r| r.id == id) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rooms_are_valid() {
        let catalog = RoomCatalog::load().unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(catalog.rooms.len(), 30);
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

    /// Icebreaker : the themes and the weight of a room are read, and survive its turns.
    #[test]
    fn themes_and_weights_are_read() {
        let room = RoomTemplate::parse("x", "@roles: hall\n@theme: Culte, militaire\n@poids: 2.5\n###\n+.+\n###\n").unwrap();
        assert_eq!(room.themes, ["culte", "militaire"]);
        assert_eq!(room.weight, 2.5);
        assert_eq!(room.oriented(1).unwrap().themes, room.themes);
        let kitchen = RoomTemplate::parse("x", "@roles: kitchen\n@fonction: Cuisine\n###\n+.+\n###\n").unwrap();
        assert_eq!(kitchen.function.as_deref(), Some("cuisine"));
        assert_eq!(kitchen.oriented(5).unwrap().function, kitchen.function);
        let e = RoomTemplate::parse("x", "@roles: hall\n@poids: lourd\n###\n+.+\n###\n").unwrap_err();
        assert!(e.contains("@poids"), "{e}");
        let catalog = RoomCatalog::load().unwrap();
        assert!(catalog.themes().len() >= 3, "{:?}", catalog.themes());
    }

    /// Icebreaker : the decor drawn in the rooms never blocks a door or stairs, and a wall piece has its wall.
    #[test]
    fn drawn_decor_stays_out_of_the_way() {
        let catalog = RoomCatalog::load().unwrap();
        let settings = super::super::settings::Settings::load().unwrap();
        let mut drawn = 0;
        for room in &catalog.rooms {
            for (&cell, &symbol) in &room.plan.cells {
                if !plan::is_decor_symbol(symbol) {
                    continue;
                }
                drawn += 1;
                let item = settings
                    .decor
                    .symbols
                    .get(&symbol)
                    .unwrap_or_else(|| panic!("salle « {} » : symbole de décor « {symbol} » sans sprite", room.id));
                for connector in &room.connectors {
                    assert_ne!(connector.inside, cell, "salle « {} » : décor devant une porte", room.id);
                }
                for (dx, dy) in plan::DIRECTIONS {
                    let next = room.plan.get(cell.offset(dx, dy, 0));
                    assert!(
                        plan::stairs_direction(next).is_none(),
                        "salle « {} » : décor contre un escalier ({}, {})",
                        room.id,
                        cell.x,
                        cell.y
                    );
                }
                if item.at == super::super::settings::DecorPlace::Wall {
                    assert!(
                        plan::DIRECTIONS.into_iter().any(|(dx, dy)| room.plan.get(cell.offset(dx, dy, 0)) == '#'),
                        "salle « {} » : « {symbol} » veut un mur ({}, {})",
                        room.id,
                        cell.x,
                        cell.y
                    );
                }
            }
        }
        assert!(drawn > 100, "{drawn} décors dessinés seulement");
    }
}
