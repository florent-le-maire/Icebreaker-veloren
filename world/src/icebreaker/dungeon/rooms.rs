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
    /// Icebreaker F.19 : the room of the portal back to the village, and the room of the altar that raises the
    /// dead. Each tells where its object stands by a `@manque:` of the reserved label (`special_label`).
    Portal,
    Altar,
    /// Icebreaker F.31 : the safe room where the party arrives, under the ground (no tunnel to the surface any
    /// more), off the entrance.
    SafeRoom,
}

impl RoomRole {
    /// Icebreaker F.19 : the roles whose room holds an object of the game (portal, altar).
    pub const SPECIAL: [RoomRole; 2] = [RoomRole::Portal, RoomRole::Altar];

    /// Icebreaker F.19 : the label of the `@manque:` that places the object of a special room.
    pub fn special_label(self) -> Option<&'static str> {
        match self {
            RoomRole::Portal => Some("portail"),
            RoomRole::Altar => Some("autel de résurrection"),
            _ => None,
        }
    }

    pub const ALL: [RoomRole; 16] = [
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
        RoomRole::Portal,
        RoomRole::Altar,
        RoomRole::SafeRoom,
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
            RoomRole::Portal => "portal",
            RoomRole::Altar => "altar",
            RoomRole::SafeRoom => "safeRoom",
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

/// Icebreaker F.14 : blocks per cell and per level the furniture of the rooms is written for (`@meuble:` counts in
/// blocks). The same as `render.cell_size` and `render.level_height` of `generation.ron` (a test says so).
pub const FURNITURE_CELL: i32 = 7;
pub const FURNITURE_LEVEL: i32 = 7;

/// Icebreaker F.14 : a piece of furniture placed by hand in a room (`@meuble: Barrel 12 8 0 2 miroir`, written
/// by the rooms workshop): a sprite of Veloren, its block from the corner of the room (x east, y south, z 0 = the
/// block standing on the floor of level 0; a level is `FURNITURE_LEVEL` blocks higher), its orientation (0 to 7,
/// eighths of a turn) and whether its model is mirrored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Furniture {
    pub sprite: String,
    pub pos: (i32, i32, i32),
    pub ori: u8,
    pub mirror: bool,
}

/// Icebreaker F.14 : a model the room wants and Veloren doesn't have (`@manque: "trône" 30 20 0 3 3 5`): its
/// name, the block of its lowest corner, its size in blocks, and the way it faces (0 to 7 like a sprite, 0 if not
/// said: `@manque: "statue" 30 20 0 2 2 5 6`). Nothing is drawn in the game; it is the list of models to make
/// (F.13), each with the place and the direction the model must have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Missing {
    pub label: String,
    pub pos: (i32, i32, i32),
    pub size: (i32, i32, i32),
    pub ori: u8,
}

/// `Barrel 12 8 0 2 miroir`.
fn parse_furniture(value: &str) -> Result<Furniture, String> {
    let mut words = value.split_whitespace();
    let sprite = words.next().ok_or("il manque le sprite")?.to_string();
    if super::settings::sprite_kind(&sprite).is_none() {
        return Err(format!("sprite inconnu « {sprite} »"));
    }
    let mut number = |name: &str| -> Result<i32, String> {
        words.next().and_then(|w| w.parse().ok()).ok_or(format!("il manque {name} (un nombre entier)"))
    };
    let pos = (number("x")?, number("y")?, number("z")?);
    let ori = number("l'orientation")?;
    if !(0..8).contains(&ori) {
        return Err(format!("orientation {ori} : de 0 à 7"));
    }
    let mirror = match words.next() {
        None => false,
        Some("miroir") => true,
        Some(other) => return Err(format!("« {other} » en trop (seul « miroir » peut suivre)")),
    };
    Ok(Furniture { sprite, pos, ori: ori as u8, mirror })
}

/// `"trône" 30 20 0 3 3 5`.
fn parse_missing(value: &str) -> Result<Missing, String> {
    let rest = value.trim().strip_prefix('"').ok_or("le libellé se met entre guillemets")?;
    let (label, rest) = rest.split_once('"').ok_or("guillemet fermant absent")?;
    let numbers: Vec<i32> = rest.split_whitespace().filter_map(|w| w.parse().ok()).collect();
    let (x, y, z, sx, sy, sz, ori) = match numbers[..] {
        [x, y, z, sx, sy, sz] => (x, y, z, sx, sy, sz, 0),
        [x, y, z, sx, sy, sz, ori] => (x, y, z, sx, sy, sz, ori),
        _ => return Err("après le libellé : x y z, puis largeur, profondeur, hauteur, puis le sens (0 à 7)".into()),
    };
    if label.trim().is_empty() || sx < 1 || sy < 1 || sz < 1 {
        return Err("libellé vide ou taille nulle".into());
    }
    if !(0..8).contains(&ori) {
        return Err(format!("sens {ori} : de 0 à 7"));
    }
    Ok(Missing { label: label.trim().to_string(), pos: (x, y, z), size: (sx, sy, sz), ori: ori as u8 })
}

/// The cell of the plan a block of furniture is in, and its height above the floor of that cell.
fn furniture_cell(plan: &Plan, (x, y, z): (i32, i32, i32)) -> (Cell, i32) {
    let (min_x, _, min_y, _) = plan.bounds().unwrap_or((0, 0, 0, 0));
    let cell = Cell::new(
        min_x + x.div_euclid(FURNITURE_CELL),
        min_y + y.div_euclid(FURNITURE_CELL),
        z.div_euclid(FURNITURE_LEVEL),
    );
    (cell, z.rem_euclid(FURNITURE_LEVEL))
}

/// What is wrong with a block of furniture: out of the floor of the room, in the ceiling, or in the middle of a
/// cell that must stay free (a creature, a chest, a ladder, the way in from a door).
fn furniture_error(plan: &Plan, pos: (i32, i32, i32)) -> Option<&'static str> {
    let (cell, height) = furniture_cell(plan, pos);
    let symbol = plan.get(cell);
    if !plan::is_walkable(symbol) || matches!(symbol, 'D' | 'O' | '*' | '~') {
        return Some("hors du sol de la salle (mur, porte, pilier, fosse ou vide)");
    }
    if height > FURNITURE_LEVEL - 2 {
        return Some("dans le plafond");
    }
    let middle = FURNITURE_CELL / 2;
    let centered = (pos.0.rem_euclid(FURNITURE_CELL) - middle).abs() <= 1
        && (pos.1.rem_euclid(FURNITURE_CELL) - middle).abs() <= 1
        && height < 3;
    let reserved = symbol != '.' && !plan::is_decor_symbol(symbol);
    let doorway = DIRECTIONS.into_iter().any(|(dx, dy)| plan.get(cell.offset(dx, dy, 0)) == '+');
    if centered && reserved {
        return Some("au milieu d'une case prise (créature, coffre, départ, échelle)");
    }
    if centered && doorway {
        return Some("au milieu de la case devant une porte : le passage doit rester libre");
    }
    None
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
    /// Icebreaker F.14 : the furniture placed by hand (`@meuble:`) and the models it lacks (`@manque:`). A room
    /// with furniture gets no decor drawn by its role.
    pub furniture: Vec<Furniture>,
    pub missing: Vec<Missing>,
    /// Icebreaker F.28 : the floor, walls and ceiling it asks for (`@sol:`, `@murs:`, `@plafond:`, names of
    /// `donjons/materiaux.ron`) ; what it leaves out comes from the theme of the dungeon.
    pub habillage: super::materiaux::HabillageSalle,
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
        let mut furniture = Vec::new();
        let mut missing = Vec::new();
        let mut habillage = super::materiaux::HabillageSalle::default();
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
                "meuble" => match parse_furniture(value) {
                    Ok(f) => furniture.push((i + 1, f)),
                    Err(e) => errors.push(format!("ligne {} : « @meuble » : {e}", i + 1)),
                },
                "manque" => match parse_missing(value) {
                    Ok(m) => missing.push(m),
                    Err(e) => errors.push(format!("ligne {} : « @manque » : {e}", i + 1)),
                },
                "sol" => habillage.sol = Some(value.trim().to_lowercase()).filter(|v| !v.is_empty()),
                "murs" => habillage.murs = Some(value.trim().to_lowercase()).filter(|v| !v.is_empty()),
                "plafond" => habillage.plafond = Some(value.trim().to_lowercase()).filter(|v| !v.is_empty()),
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
        for (line, f) in &furniture {
            if let Some(e) = furniture_error(&plan, f.pos) {
                errors.push(format!("ligne {line} : « @meuble {} » {e}", f.sprite));
            }
        }
        // Icebreaker F.19 : a special room places its object once, on its floor; the label is kept for it.
        for role in RoomRole::SPECIAL {
            let label = role.special_label().unwrap_or_default();
            let placed: Vec<&Missing> = missing.iter().filter(|m| m.label == label).collect();
            if roles.contains(&role) {
                match placed[..] {
                    [m] => {
                        let middle = (m.pos.0 + m.size.0 / 2, m.pos.1 + m.size.1 / 2, m.pos.2);
                        if let Some(e) = furniture_error(&plan, middle) {
                            errors.push(format!("« @manque \"{label}\" » {e}"));
                        }
                    },
                    _ => errors.push(format!(
                        "une salle « {} » place son objet une fois (« @manque: \"{label}\" x y z l p h sens »)",
                        role.name()
                    )),
                }
            } else if !placed.is_empty() {
                errors.push(format!("le libellé « {label} » est réservé aux salles de rôle « {} »", role.name()));
            }
        }
        let furniture = furniture.into_iter().map(|(_, f)| f).collect();
        Self::from_plan(id, roles, bosses, errors, plan).map(|room| Self {
            themes,
            weight,
            function,
            furniture,
            missing,
            habillage,
            ..room
        })
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
            furniture: Vec::new(),
            missing: Vec::new(),
            habillage: Default::default(),
            plan,
            connectors,
        })
    }

    pub fn doors(&self) -> impl Iterator<Item = &Connector> {
        self.connectors.iter().filter(|c| c.kind == ConnectorKind::Door)
    }

    pub fn door_count(&self) -> usize { self.doors().count() }

    /// Icebreaker F.31 : the open floor cell of the lowest level nearest to the middle of the room (where one
    /// arrives in the safe room, or in a room alone).
    pub fn middle(&self) -> Cell {
        let level = self.plan.cells.keys().map(|c| c.level).min().unwrap_or(0);
        let floor: Vec<Cell> = self
            .plan
            .cells
            .iter()
            .filter(|(c, s)| c.level == level && plan::is_open_floor(**s))
            .map(|(c, _)| *c)
            .collect();
        let n = floor.len().max(1) as i32;
        let (mx, my) = (floor.iter().map(|c| 2 * c.x).sum::<i32>() / n, floor.iter().map(|c| 2 * c.y).sum::<i32>() / n);
        floor
            .into_iter()
            .min_by_key(|c| ((2 * c.x - mx).pow(2) + (2 * c.y - my).pow(2), c.x, c.y))
            .unwrap_or(Cell::new(0, 0, level))
    }

    /// Icebreaker F.19 : where the object of a special room stands (none: not a room of this role).
    pub fn special(&self, role: RoomRole) -> Option<&Missing> {
        let label = role.special_label()?;
        self.roles.contains(&role).then(|| self.missing.iter().find(|m| m.label == label)).flatten()
    }

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
        // Icebreaker F.14 : the furniture turns with the room, in blocks; a mirror turns a model the other way
        // and flips it, a quarter turn adds two eighths.
        let mut furniture = self.furniture.clone();
        let mut missing = self.missing.clone();
        if orientation >= 4 {
            let blocks = width * FURNITURE_CELL;
            for f in &mut furniture {
                f.pos.0 = blocks - 1 - f.pos.0;
                f.ori = (8 - f.ori) % 8;
                f.mirror = !f.mirror;
            }
            for m in &mut missing {
                m.pos.0 = blocks - m.pos.0 - m.size.0;
                m.ori = (8 - m.ori) % 8;
            }
        }
        for turn in 0..orientation % 4 {
            // The depth of the room before this turn.
            let blocks = (if turn % 2 == 0 { depth } else { width }) * FURNITURE_CELL;
            for f in &mut furniture {
                f.pos = (blocks - 1 - f.pos.1, f.pos.0, f.pos.2);
                f.ori = (f.ori + 2) % 8;
            }
            for m in &mut missing {
                m.pos = (blocks - m.pos.1 - m.size.1, m.pos.0, m.pos.2);
                m.size = (m.size.1, m.size.0, m.size.2);
                m.ori = (m.ori + 2) % 8;
            }
        }
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
            furniture,
            missing,
            habillage: self.habillage.clone(),
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
        assert_eq!(catalog.rooms.len(), 33);
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

    const FURNISHED: &str = "@roles: hall\n@meuble: Barrel 8 7 0 2\n@meuble: BenchWoodEnd 12 9 1 3 miroir\n@manque: \
                             \"trône\" 14 10 0 3 2 5\n#####\n+...+\n#...#\n#####\n";

    /// Icebreaker F.14 : the furniture placed by hand is read, and what is wrong with it is said.
    #[test]
    fn furniture_is_read_and_checked() {
        let settings = super::super::Settings::load().unwrap();
        assert_eq!((settings.render.cell_size, settings.render.level_height), (FURNITURE_CELL, FURNITURE_LEVEL));
        let room = RoomTemplate::parse("x", FURNISHED).unwrap();
        assert_eq!(room.furniture, [
            Furniture { sprite: "Barrel".into(), pos: (8, 7, 0), ori: 2, mirror: false },
            Furniture { sprite: "BenchWoodEnd".into(), pos: (12, 9, 1), ori: 3, mirror: true },
        ]);
        assert_eq!(room.missing, [Missing { label: "trône".into(), pos: (14, 10, 0), size: (3, 2, 5), ori: 0 }]);
        // The way a missing model faces, after its size; it turns with the room.
        let statue = RoomTemplate::parse("x", &FURNISHED.replace("3 2 5\n", "3 2 5 6\n")).unwrap();
        assert_eq!(statue.missing[0].ori, 6);
        assert_eq!(statue.oriented(1).unwrap().missing[0].ori, 0);
        assert_eq!(statue.oriented(4).unwrap().missing[0].ori, 2);
        let wrong = |line: &str| {
            RoomTemplate::parse("x", &format!("@roles: hall\n{line}\n#####\n+.b.+\n#...#\n#####\n")).unwrap_err()
        };
        assert!(wrong("@meuble: Tonneau 8 7 0 0").contains("sprite inconnu"));
        assert!(wrong("@meuble: Barrel 8 7 0 9").contains("de 0 à 7"));
        assert!(wrong("@meuble: Barrel 3 7 0 0").contains("hors du sol"));
        assert!(wrong("@meuble: Barrel 8 7 6 0").contains("plafond"));
        assert!(wrong("@meuble: Barrel 17 10 0 0").contains("case prise"), "au milieu de la case de la brute");
        assert!(wrong("@meuble: Barrel 10 10 0 0").contains("devant une porte"));
        assert!(wrong("@manque: trône 1 1 0 1 1 1").contains("guillemets"));
        assert!(wrong("@manque: \"trône\" 8 8 0 1 1 1 9").contains("de 0 à 7"));
    }

    /// Icebreaker F.19 : a special room places its object once, on its floor, and it turns with the room.
    #[test]
    fn special_rooms_place_their_object() {
        let room = |lines: &str| {
            RoomTemplate::parse("x", &format!("@roles: portal\n{lines}#####\n+...+\n#...#\n#####\n"))
        };
        let portal = room("@manque: \"portail\" 15 10 0 5 1 6 2\n").unwrap();
        let placed = portal.special(RoomRole::Portal).unwrap();
        assert_eq!((placed.pos, placed.ori), ((15, 10, 0), 2));
        assert!(portal.special(RoomRole::Altar).is_none());
        assert_eq!(portal.oriented(1).unwrap().special(RoomRole::Portal).unwrap().ori, 4);
        assert!(room("").unwrap_err().contains("une fois"));
        let twice = "@manque: \"portail\" 15 10 0 5 1 6\n@manque: \"portail\" 15 12 0 5 1 6\n";
        assert!(room(twice).unwrap_err().contains("une fois"));
        assert!(room("@manque: \"portail\" 1 1 0 3 1 6\n").unwrap_err().contains("hors du sol"));
        let hall = "@roles: hall\n@manque: \"portail\" 15 10 0 5 1 6\n#####\n+...+\n#...#\n#####\n";
        let e = RoomTemplate::parse("x", hall).unwrap_err();
        assert!(e.contains("réservé"), "{e}");
        let catalog = RoomCatalog::load().unwrap();
        for role in RoomRole::SPECIAL {
            for room in catalog.with_role(role) {
                assert!(room.special(role).is_some(), "{}", room.id);
            }
        }
    }

    /// Icebreaker F.14 : the furniture turns with its room: it stays on the floor in every orientation, four
    /// quarter turns and two mirrors give the room back, and a piece against a wall still faces the room.
    #[test]
    fn furniture_turns_with_the_room() {
        let room = RoomTemplate::parse("x", FURNISHED).unwrap();
        for orientation in 0..8 {
            let turned = room.oriented(orientation).unwrap();
            for f in &turned.furniture {
                assert_eq!(furniture_error(&turned.plan, f.pos), None, "orientation {orientation} : {f:?}");
            }
        }
        let mut turned = room.clone();
        for _ in 0..4 {
            turned = turned.oriented(1).unwrap();
        }
        assert_eq!((&turned.furniture, &turned.missing), (&room.furniture, &room.missing));
        let mirrored = room.oriented(4).unwrap().oriented(4).unwrap();
        assert_eq!((&mirrored.furniture, &mirrored.missing), (&room.furniture, &room.missing));
        // The barrel is on the first row of blocks of the room, facing +y (ori 2): after a quarter turn it is
        // on the last column, facing -x (ori 4).
        let quarter = room.oriented(1).unwrap();
        assert_eq!(quarter.furniture[0], Furniture { sprite: "Barrel".into(), pos: (4 * 7 - 1 - 7, 8, 0), ori: 4, mirror: false });
        let flipped = room.oriented(4).unwrap();
        assert_eq!(flipped.furniture[1], Furniture { sprite: "BenchWoodEnd".into(), pos: (5 * 7 - 1 - 12, 9, 1), ori: 5, mirror: false });
        assert_eq!(flipped.missing[0].pos, (5 * 7 - 14 - 3, 10, 0));
        // The rooms of the game, in every orientation.
        for room in RoomCatalog::load().unwrap().with_orientations(8).unwrap().rooms {
            for f in &room.furniture {
                assert_eq!(furniture_error(&room.plan, f.pos), None, "{} : {f:?}", room.id);
            }
        }
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
