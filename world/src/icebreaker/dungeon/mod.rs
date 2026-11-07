//! Icebreaker: our dungeon generator. A dungeon is first a plan of cells over
//! several levels (`plan`), then voxels drawn by the `IcebreakerDungeon` plot
//! (`site/plot/icebreaker_dungeon.rs`).

pub mod assemble;
pub mod graph;
pub mod plan;
pub mod populate;
pub mod rooms;
pub mod settings;
pub mod shapes;

use super::butin::Butin;
use plan::{Cell, Plan};
use rooms::RoomRole;
use std::collections::BTreeMap;
pub use settings::{Mission, RenderSettings, Settings};

/// Where the dungeon data lives, under `assets/`.
const DATA_DIR: &str = "icebreaker/donjons";

/// The path of a data file of the generator.
pub fn data_path(name: &str) -> std::path::PathBuf {
    common::assets::ASSETS_PATH.join(DATA_DIR).join(name)
}

/// Reads a data file of the generator.
pub fn read_data(name: &str) -> Result<String, String> {
    let path = data_path(name);
    std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))
}

/// The tunnel from the surface down to a door of the entrance room: its cells
/// at the entrance level, from the door outward, and its direction.
#[derive(Clone, Debug)]
pub struct Tunnel {
    pub cells: Vec<Cell>,
    pub dir: (i32, i32),
}

/// A generated dungeon, ready to be drawn.
#[derive(Clone, Debug)]
pub struct Dungeon {
    pub plan: Plan,
    pub tunnel: Tunnel,
    /// What the dungeon is made of, for the logs.
    pub summary: String,
    /// Who lives there.
    pub faction: settings::Faction,
    /// Locked doors (door cells), and the cells where their keys lie, with
    /// the key of each.
    pub locks: Vec<(Cell, settings::KeySettings)>,
    pub keys: Vec<(Cell, settings::KeySettings)>,
    /// How its asleep creatures (uppercase markers) wake up.
    pub sleep: settings::SleepSettings,
    /// Lights on free floor cells: a torch on the wall toward (dx, dy), or a
    /// brazier (`None`).
    pub lights: Vec<(Cell, Option<(i32, i32)>)>,
    /// The decor of the rooms (Icebreaker F.6).
    pub decor: Vec<Decor>,
    /// Icebreaker B.5: the loot settings, and the tier of each chest and
    /// creature of the plan.
    pub butin: Butin,
    pub paliers: BTreeMap<Cell, usize>,
}

/// Icebreaker B.5: the tier of each chest and creature of `plan` by its
/// depth, the passages from the entrance to its room (in a corridor, the
/// nearest room).
fn place_tiers(
    plan: &Plan,
    rooms: &[assemble::PlacedRoom],
    graph: &graph::MissionGraph,
    butin: &Butin,
) -> BTreeMap<Cell, usize> {
    // Passages from the entrance (room 0) to each room, either way.
    let mut distances = vec![usize::MAX; graph.rooms.len()];
    distances[0] = 0;
    let mut queue = std::collections::VecDeque::from([0]);
    while let Some(room) = queue.pop_front() {
        for p in graph.passages.iter().filter(|p| p.from == room || p.to == room) {
            let next = p.other(room);
            if distances[next] == usize::MAX {
                distances[next] = distances[room] + 1;
                queue.push_back(next);
            }
        }
    }
    let distances: Vec<usize> = distances.into_iter().map(|d| if d == usize::MAX { 0 } else { d }).collect();
    let depths = distances.iter().max().map_or(1, |d| d + 1);
    let room_of: BTreeMap<Cell, usize> = rooms
        .iter()
        .flat_map(|r| r.footprint().map(move |c| (c, r.graph_room)))
        .collect();
    plan.cells
        .iter()
        .filter(|(_, s)| **s == '$' || populate::kind_of(**s).is_some())
        .map(|(cell, _)| {
            let room = room_of.get(cell).copied().or_else(|| {
                room_of
                    .iter()
                    .min_by_key(|(c, _)| {
                        (c.x - cell.x).abs() + (c.y - cell.y).abs() + 4 * (c.level - cell.level).abs()
                    })
                    .map(|(_, r)| *r)
            });
            let depth = room.and_then(|r| distances.get(r).copied()).unwrap_or(0);
            (*cell, butin.palier(depth, depths))
        })
        .collect()
}

/// The lights of a dungeon by its inhabitants (Icebreaker G.10): on the free
/// floor cells (`.`, not a key) of each room, with the chance of its role,
/// lower without creatures; fewer in the corridors. A cell along a wall gets
/// a torch on that wall `wall_torches` of the time, otherwise a brazier.
fn place_lights(
    plan: &Plan,
    rooms: &[(&str, Vec<Cell>)],
    keys: &[(Cell, settings::KeySettings)],
    lighting: &settings::LightingSettings,
    seed: u64,
) -> Vec<(Cell, Option<(i32, i32)>)> {
    use rand::{RngExt, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0x11_6874);
    let mut chances = std::collections::BTreeMap::new();
    for (role, cells) in rooms {
        let inhabited = cells.iter().any(|c| populate::kind_of(plan.get(*c)).is_some());
        let chance = lighting.chance(Some(role), inhabited);
        chances.extend(cells.iter().map(|c| (*c, chance)));
    }
    let mut lights = Vec::new();
    for (cell, symbol) in &plan.cells {
        // Icebreaker F.2: nothing on a landing or at the foot of stairs.
        let by_stairs = plan::DIRECTIONS.into_iter().any(|(dx, dy)| {
            let s = plan.get(cell.offset(dx, dy, 0));
            s == '_' || plan::stairs_direction(s).is_some()
        });
        if *symbol != '.' || by_stairs || keys.iter().any(|(k, _)| k == cell) {
            continue;
        }
        let chance = chances.get(cell).copied().unwrap_or_else(|| lighting.chance(None, false));
        if !rng.random_bool(chance.clamp(0.0, 1.0) as f64) {
            continue;
        }
        let walls: Vec<(i32, i32)> = plan::DIRECTIONS
            .into_iter()
            .filter(|(dx, dy)| plan.get(cell.offset(*dx, *dy, 0)) == '#')
            .collect();
        let torch = !walls.is_empty() && rng.random_bool(lighting.wall_torches.clamp(0.0, 1.0) as f64);
        lights.push((*cell, torch.then(|| walls[rng.random_range(0..walls.len())])));
    }
    lights
}

/// A piece of decor of the dungeon (Icebreaker F.6): a sprite (by name) in a
/// cell, against the wall toward (dx, dy), in the middle (with a sprite on
/// the four blocks around, or not), or scattered over the floor blocks.
#[derive(Clone, Debug)]
pub struct Decor {
    pub cell: Cell,
    pub sprite: String,
    pub place: DecorPlacement,
}

#[derive(Clone, Debug)]
pub enum DecorPlacement {
    Wall((i32, i32)),
    Center { around: Option<String> },
    Scatter { chance: f32 },
}

/// The decor of a dungeon by the role of its rooms (Icebreaker F.6), on the
/// free floor cells (`.`, not `taken`: keys, lights) that are not next to a
/// door, stairs or a hole; the corridors as one more room.
fn place_decor(
    plan: &Plan,
    rooms: &[(&str, Vec<Cell>)],
    taken: &std::collections::BTreeSet<Cell>,
    decor: &settings::DecorSettings,
    seed: u64,
) -> Vec<Decor> {
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed ^ 0xDEC0_12);
    let in_room: std::collections::BTreeSet<Cell> = rooms.iter().flat_map(|(_, c)| c.iter().copied()).collect();
    let usable = |cell: &Cell| {
        plan.get(*cell) == '.'
            && !taken.contains(cell)
            && plan::DIRECTIONS.into_iter().all(|(dx, dy)| {
                let s = plan.get(cell.offset(dx, dy, 0));
                s != 'D' && s != '_' && plan::stairs_direction(s).is_none()
            })
    };
    let corridors: Vec<Cell> = plan.cells.keys().filter(|c| !in_room.contains(c)).copied().collect();
    let mut placed = Vec::new();
    let empty = Vec::new();
    let sets = rooms
        .iter()
        .map(|(role, cells)| (decor.roles.get(*role).unwrap_or(&empty), cells))
        .chain(std::iter::once((&decor.corridors, &corridors)));
    for (items, cells) in sets {
        let mut free: Vec<Cell> = cells.iter().filter(|c| usable(c)).copied().collect();
        graph::shuffle(&mut free, &mut rng);
        let wall_of = |cell: &Cell| {
            plan::DIRECTIONS.into_iter().find(|(dx, dy)| plan.get(cell.offset(*dx, *dy, 0)) == '#')
        };
        for item in items {
            match item.at {
                settings::DecorPlace::Scatter => {
                    placed.extend(free.iter().map(|cell| Decor {
                        cell: *cell,
                        sprite: item.sprite.clone(),
                        place: DecorPlacement::Scatter { chance: item.chance },
                    }));
                },
                at => {
                    for _ in 0..item.count.roll(&mut rng).max(0) {
                        // A wall item wants a wall; a centre item prefers open floor.
                        let wanted = |c: &Cell| (at == settings::DecorPlace::Wall) == wall_of(c).is_some();
                        let Some(index) = free.iter().position(wanted).or_else(|| {
                            (at == settings::DecorPlace::Center && !free.is_empty()).then_some(0)
                        }) else {
                            break;
                        };
                        let cell = free.swap_remove(index);
                        let place = match wall_of(&cell) {
                            Some(wall) if at == settings::DecorPlace::Wall => DecorPlacement::Wall(wall),
                            _ => DecorPlacement::Center { around: item.around.clone() },
                        };
                        placed.push(Decor {
                            cell,
                            sprite: item.sprite.clone(),
                            place,
                        });
                    }
                },
            }
        }
    }
    placed
}

/// Marks the tunnel in front of `door` (a wall cell of a room, opening toward
/// `dir`), `length` cells long, and opens the door. The tunnel climbs to the
/// surface, so the cells above it are kept free too (`~` at the entrance level,
/// `%` above it and along it: cells the plot leaves to the terrain).
/// Returns `None` if the cells are not free.
pub fn reserve_tunnel(plan: &mut Plan, door: Cell, dir: (i32, i32), length: i32) -> Option<Tunnel> {
    let cells: Vec<Cell> = (1..=length)
        .map(|k| door.offset(dir.0 * k, dir.1 * k, 0))
        .collect();
    let (side_x, side_y) = (dir.1, dir.0);
    let footprint = cells
        .iter()
        .flat_map(|c| [*c, c.offset(0, 0, 1)])
        .collect::<Vec<_>>();
    let sides = footprint
        .iter()
        .flat_map(|c| [c.offset(side_x, side_y, 0), c.offset(-side_x, -side_y, 0)])
        .collect::<Vec<_>>();
    if footprint.iter().chain(&sides).any(|c| !plan.is_free(*c)) {
        return None;
    }
    for cell in footprint {
        plan.cells.insert(cell, if cell.level == door.level { '~' } else { '%' });
    }
    for cell in sides {
        plan.cells.insert(cell, '%');
    }
    plan.cells.insert(door, 'D');
    Some(Tunnel { cells, dir })
}

/// Closes the connectors left unused: a door stays a wall, a pit stays floor.
pub fn close_connectors(plan: &mut Plan) {
    for symbol in plan.cells.values_mut() {
        match *symbol {
            '+' => *symbol = '#',
            '*' => *symbol = '.',
            _ => {},
        }
    }
}

/// A single room, its tunnel through the first door that has room for it.
pub fn single_room(settings: &Settings, room: &rooms::RoomTemplate) -> Result<Dungeon, String> {
    let mut plan = room.plan.clone();
    let tunnel = room
        .doors()
        .find_map(|door| {
            let mut tried = plan.clone();
            let tunnel =
                reserve_tunnel(&mut tried, door.cell, door.direction(), settings.render.tunnel_cells)?;
            plan = tried;
            Some(tunnel)
        })
        .ok_or("pas de place pour le tunnel d'entrée")?;
    close_connectors(&mut plan);
    let role = room.roles.first().map_or("hall", |r| r.name());
    let cells: Vec<Cell> = room.plan.cells.keys().copied().collect();
    let rooms = [(role, cells)];
    let lights = place_lights(&plan, &rooms, &[], &settings.lighting, 0);
    let taken = lights.iter().map(|(c, _)| *c).collect();
    let decor = place_decor(&plan, &rooms, &taken, &settings.decor, 0);
    Ok(Dungeon {
        plan,
        tunnel,
        summary: format!("salle seule : {}", room.id),
        faction: settings
            .factions
            .get(&settings.default_faction)
            .cloned()
            .ok_or("faction par défaut inconnue")?,
        locks: Vec::new(),
        keys: Vec::new(),
        sleep: settings.sleep.clone(),
        lights,
        decor,
        butin: Butin::load()?,
        paliers: BTreeMap::new(),
    })
}

/// Seeds tried one after the other when a dungeon can't be assembled.
const SEED_TRIES: u64 = 10;

/// Generates the dungeon of a mission: its graph, then the assembly of the
/// rooms. A seed whose dungeon can't be assembled gives way to the next one.
pub fn generate(settings: &Settings, mission: &Mission) -> Result<Dungeon, String> {
    let catalog = settings.catalog(mission.seed)?;
    // A single room to look at (`ICEBREAKER_DONJON_SALLE`, `-Salle` of the launcher).
    if let Ok(id) = std::env::var("ICEBREAKER_DONJON_SALLE")
        && !id.is_empty()
    {
        let room = catalog.get(&id).ok_or(format!("salle inconnue : {id}"))?;
        return single_room(settings, room);
    }

    let resolved = settings.resolve(mission)?;
    let missing: Vec<&str> = std::iter::once(RoomRole::Entrance)
        .chain([resolved.goal, resolved.fallback, RoomRole::Hall, RoomRole::GuardRoom])
        .chain(resolved.weights.keys().copied())
        .filter(|role| catalog.with_role(*role).is_empty())
        .map(RoomRole::name)
        .collect();
    if !missing.is_empty() {
        return Err(format!("aucune salle pour : {}", missing.join(", ")));
    }
    // Each lock has its own key (`keys` of the settings, in the order of the locks).
    let with_keys = |cells: &[(Cell, usize)]| {
        cells
            .iter()
            .map(|(cell, lock)| {
                let key = settings.keys.get(*lock).ok_or(format!("pas de clé pour le verrou {lock}"))?;
                Ok((*cell, key.clone()))
            })
            .collect::<Result<Vec<_>, String>>()
    };
    let butin = Butin::load()?;
    let mut failures = Vec::new();
    for seed in mission.seed..mission.seed + SEED_TRIES {
        let graph = graph::generate(&resolved, seed);
        let assembled = assemble::assemble(
            &graph,
            &catalog,
            seed,
            &settings.assembly,
            settings.render.tunnel_cells,
            |failure| failures.push(format!("graine {seed}, {failure}")),
        );
        if let Some(mut assembled) = assembled {
            let (creatures, chests) = populate::populate(&mut assembled, &graph, &resolved, seed);
            let rooms: Vec<String> = assembled
                .rooms
                .iter()
                .map(|r| format!("{} {}", graph.rooms[r.graph_room].role.name(), r.template.id))
                .collect();
            let keys = with_keys(&assembled.keys)?;
            let room_cells: Vec<(&str, Vec<Cell>)> = assembled
                .rooms
                .iter()
                .map(|r| (graph.rooms[r.graph_room].role.name(), r.footprint().collect()))
                .collect();
            let lights = place_lights(&assembled.plan, &room_cells, &keys, &settings.lighting, seed);
            let taken = keys.iter().map(|(c, _)| *c).chain(lights.iter().map(|(c, _)| *c)).collect();
            let decor = place_decor(&assembled.plan, &room_cells, &taken, &settings.decor, seed);
            // Icebreaker B.5: the tier of each chest and creature.
            let paliers = place_tiers(&assembled.plan, &assembled.rooms, &graph, &butin);
            let chests_by_tier: Vec<String> = (1..=butin.paliers())
                .map(|p| {
                    paliers
                        .iter()
                        .filter(|(c, t)| **t == p && assembled.plan.get(**c) == '$')
                        .count()
                        .to_string()
                })
                .collect();
            return Ok(Dungeon {
                plan: assembled.plan,
                tunnel: assembled.tunnel,
                summary: format!(
                    "{} {} ({}), graine {seed} ({} tentative(s)), {creatures} créature(s), {chests} \
                     coffre(s) ({} par palier) ; salles : {}\n{}",
                    mission.size,
                    mission.objective,
                    resolved.modifiers.join(", "),
                    assembled.attempts,
                    chests_by_tier.join(" / "),
                    rooms.join(", "),
                    graph.describe()
                ),
                faction: resolved.faction.clone(),
                locks: with_keys(&assembled.locks)?,
                keys,
                sleep: settings.sleep.clone(),
                lights,
                decor,
                butin,
                paliers,
            });
        }
    }
    Err(format!("aucun donjon assemblé :\n{}", failures.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::rooms::RoomCatalog;
    use std::collections::BTreeSet;

    #[test]
    fn settings_load() { Settings::load().unwrap(); }

    #[test]
    fn threat_changes_the_dungeon() {
        let settings = Settings::load().unwrap();
        // (cells, creatures) over a few seeds.
        let measure = |threat: i32| {
            (0..20).fold((0, 0), |(cells, creatures), seed| {
                let mission = Mission {
                    seed,
                    threat: Some(threat),
                    ..Mission::default()
                };
                let dungeon = generate(&settings, &mission).unwrap();
                let plan = &dungeon.plan;
                let people = plan.cells.values().filter(|s| populate::kind_of(**s).is_some()).count();
                (cells + plan.cells.len(), creatures + people)
            })
        };
        let (low, high) = (measure(1), measure(5));
        assert!(high.0 > low.0, "menace 5 pas plus grande : {low:?} / {high:?}");
        assert!(high.1 > 2 * low.1, "menace 5 pas plus peuplée : {low:?} / {high:?}");
    }

    #[test]
    fn every_parameter_resolves() {
        let settings = Settings::load().unwrap();
        let p = &settings.parameters;
        for region in p.regions.keys() {
            for season in p.seasons.keys() {
                for objective in settings.goal_roles.keys() {
                    let mission = Mission {
                        region: Some(region.clone()),
                        season: Some(season.clone()),
                        objective: objective.clone(),
                        density: Some("high".into()),
                        ..Mission::default()
                    };
                    let dungeon = generate(&settings, &mission).unwrap_or_else(|e| panic!("{mission:?} : {e}"));
                    assert!(!dungeon.plan.cells.is_empty());
                }
            }
        }
        let unknown = Mission {
            region: Some("lune".into()),
            ..Mission::default()
        };
        assert!(settings.resolve(&unknown).unwrap_err().contains("région inconnue"));
    }

    #[test]
    fn faction_creatures_exist() {
        let settings = Settings::load().unwrap();
        for (name, faction) in &settings.factions {
            for asset in faction.creatures.values().flatten() {
                let path = common::assets::ASSETS_PATH.join(asset.replace('.', "/") + ".ron");
                assert!(path.exists(), "faction {name} : {asset} introuvable");
            }
        }
    }

    #[test]
    fn lights_follow_the_inhabitants() {
        let settings = Settings::load().unwrap();
        // Three rooms of 20 × 20 floor cells, side by side, walls around: a nest
        // and a guard room with a creature each, an empty guard room.
        let mut plan = Plan::default();
        let mut rooms = Vec::new();
        for (r, role) in ["nest", "guardRoom", "guardRoom"].into_iter().enumerate() {
            let x0 = r as i32 * 22;
            let mut cells = Vec::new();
            for y in -1..=20 {
                for x in x0 - 1..=x0 + 20 {
                    let inside = (0..20).contains(&y) && (x0..x0 + 20).contains(&x);
                    let cell = Cell::new(x, y, 0);
                    plan.cells.insert(cell, if inside { '.' } else { '#' });
                    cells.push(cell);
                }
            }
            if r < 2 {
                plan.cells.insert(Cell::new(x0 + 5, 5, 0), 'b');
            }
            rooms.push((role, cells));
        }
        let lights = place_lights(&plan, &rooms, &[], &settings.lighting, 7);
        let in_room = |r: i32| lights.iter().filter(|(c, _)| c.x / 22 == r).count();
        assert_eq!(in_room(0), 0, "un nid éclairé");
        assert!(in_room(1) > 2 * in_room(2), "salle habitée {} / vide {}", in_room(1), in_room(2));
        for (cell, torch) in &lights {
            assert_eq!(plan.get(*cell), '.');
            if let Some((dx, dy)) = torch {
                assert_eq!(plan.get(cell.offset(*dx, *dy, 0)), '#', "torche loin du mur");
            }
        }
        assert!(lights.iter().any(|(_, t)| t.is_some()) && lights.iter().any(|(_, t)| t.is_none()));
    }

    #[test]
    fn each_locked_door_has_its_own_key() {
        let settings = Settings::load().unwrap();
        for key in &settings.keys {
            for asset in [&key.item, &key.loot] {
                let path = common::assets::ASSETS_PATH.join(asset.replace('.', "/") + ".ron");
                assert!(path.exists(), "clé : {asset} introuvable");
            }
        }
        let mut locked = 0;
        for size in ["small", "medium", "large"] {
            for seed in 0..30 {
                let mission = Mission {
                    seed,
                    size: size.into(),
                    ..Mission::default()
                };
                let dungeon = generate(&settings, &mission).unwrap();
                let items: BTreeSet<_> = dungeon.locks.iter().map(|(_, k)| &k.item).collect();
                assert_eq!(items.len(), dungeon.locks.len(), "{size} {seed} : deux portes, même clé");
                let keys: BTreeSet<_> = dungeon.keys.iter().map(|(_, k)| &k.item).collect();
                assert_eq!(items, keys, "{size} {seed} : clés et portes ne vont pas ensemble");
                for (cell, _) in &dungeon.locks {
                    assert_eq!(dungeon.plan.get(*cell), 'D', "{size} {seed} : porte fermée hors d'une porte");
                }
                for (cell, _) in &dungeon.keys {
                    assert_eq!(dungeon.plan.get(*cell), '.', "{size} {seed} : clé hors du sol libre");
                }
                locked += dungeon.locks.len();
            }
        }
        assert!(locked > 0, "aucune porte fermée en 90 donjons");
    }

    #[test]
    fn every_room_alone_reaches_its_tunnel() {
        let settings = Settings::load().unwrap();
        let catalog = RoomCatalog::load().unwrap().with_orientations(8).unwrap();
        for room in &catalog.rooms {
            let dungeon = single_room(&settings, room).unwrap();
            let top = *dungeon.tunnel.cells.last().unwrap();
            let reached = dungeon.plan.reachable(top);
            for (cell, symbol) in &dungeon.plan.cells {
                if plan::is_walkable(*symbol) {
                    assert!(reached.contains(cell), "{} : {cell:?} inaccessible", room.id);
                }
            }
        }
    }

    /// Icebreaker F.6: the decor follows the role of the room and stays out
    /// of the way.
    #[test]
    fn decor_follows_the_roles() {
        let settings = Settings::load().unwrap();
        // Three rooms of 12 × 12 floor cells side by side, walls around, a door
        // in the south wall of each: a kitchen, a nest, a storage.
        let mut plan = Plan::default();
        let mut rooms = Vec::new();
        for (r, role) in ["kitchen", "nest", "storage"].into_iter().enumerate() {
            let x0 = r as i32 * 14;
            let mut cells = Vec::new();
            for y in -1..=12 {
                for x in x0 - 1..=x0 + 12 {
                    let inside = (0..12).contains(&y) && (x0..x0 + 12).contains(&x);
                    let cell = Cell::new(x, y, 0);
                    plan.cells.insert(cell, if inside { '.' } else { '#' });
                    cells.push(cell);
                }
            }
            plan.cells.insert(Cell::new(x0 + 6, 12, 0), 'D');
            plan.cells.insert(Cell::new(x0 + 2, 2, 0), 'b');
            rooms.push((role, cells));
        }
        let taken = std::collections::BTreeSet::from([Cell::new(5, 5, 0)]);
        let decor = place_decor(&plan, &rooms, &taken, &settings.decor, 7);
        let of = |r: i32| decor.iter().filter(move |d| d.cell.x / 14 == r);
        assert!(of(0).any(|d| d.sprite == "CookingPot" && matches!(d.place, DecorPlacement::Center { .. })));
        assert!(of(0).any(|d| matches!(&d.place, DecorPlacement::Center { around: Some(c) } if c == "ChairWoodWoodland")));
        assert!(of(1).any(|d| d.sprite == "Bones" && matches!(d.place, DecorPlacement::Scatter { .. })));
        assert!(of(2).filter(|d| d.sprite == "Barrel").count() >= 3, "réserve sans tonneaux");
        for d in &decor {
            assert_eq!(plan.get(d.cell), '.', "{d:?} hors du sol");
            assert!(!taken.contains(&d.cell), "{d:?} sur une case prise");
            for (dx, dy) in plan::DIRECTIONS {
                assert_ne!(plan.get(d.cell.offset(dx, dy, 0)), 'D', "{d:?} devant une porte");
            }
            if let DecorPlacement::Wall((dx, dy)) = d.place {
                assert_eq!(plan.get(d.cell.offset(dx, dy, 0)), '#', "{d:?} loin du mur");
            }
        }
        // Never two pieces of furniture in one cell.
        let furniture: Vec<Cell> = decor
            .iter()
            .filter(|d| !matches!(d.place, DecorPlacement::Scatter { .. }))
            .map(|d| d.cell)
            .collect();
        let distinct: std::collections::BTreeSet<Cell> = furniture.iter().copied().collect();
        assert_eq!(furniture.len(), distinct.len());
    }

    /// Icebreaker B.5: every chest and creature has a tier; the chests of the
    /// entrance side are of the first tier, those of the far end of the last
    /// one, and the chief never stands among the first.
    #[test]
    fn loot_gets_richer_with_depth() {
        let settings = Settings::load().unwrap();
        let mut chests = [0; 3];
        for seed in 0..30 {
            let mission = Mission { seed, ..Mission::default() };
            let dungeon = generate(&settings, &mission).unwrap();
            for (cell, symbol) in &dungeon.plan.cells {
                if *symbol == '$' || populate::kind_of(*symbol).is_some() {
                    let palier = *dungeon.paliers.get(cell).unwrap_or_else(|| panic!("graine {seed} : {cell:?} sans palier"));
                    if *symbol == '$' {
                        chests[palier - 1] += 1;
                    }
                    if symbol.eq_ignore_ascii_case(&'c') {
                        assert!(palier > 1, "graine {seed} : chef au palier 1");
                    }
                }
            }
        }
        assert!(chests.iter().all(|n| *n > 0), "coffres par palier : {chests:?}");
    }
}
