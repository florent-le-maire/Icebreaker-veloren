//! Icebreaker: a dungeon plan, one character per cell over several levels, in
//! the text format of the Icebreaker project (`docs/icebreaker/design/donjon.md`,
//! « Plans de donjon »).

use std::collections::BTreeMap;

/// A cell of a plan: `x` grows eastward, `y` southward (the rows of the text),
/// `level` upward (0 = the entrance level, negative = below).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cell {
    pub level: i32,
    pub y: i32,
    pub x: i32,
}

impl Cell {
    pub const fn new(x: i32, y: i32, level: i32) -> Self { Self { level, y, x } }

    pub const fn offset(self, dx: i32, dy: i32, dlevel: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.level + dlevel)
    }
}

/// The four horizontal directions, as (dx, dy).
pub const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Direction toward which stairs climb (`^ > v <`), as (dx, dy).
pub fn stairs_direction(symbol: char) -> Option<(i32, i32)> {
    match symbol {
        '^' => Some((0, -1)),
        '>' => Some((1, 0)),
        'v' => Some((0, 1)),
        '<' => Some((-1, 0)),
        _ => None,
    }
}

/// Symbol of stairs climbing toward (dx, dy).
pub fn stairs_symbol(dx: i32, dy: i32) -> char {
    match (dx, dy) {
        (0, -1) => '^',
        (1, 0) => '>',
        (0, 1) => 'v',
        _ => '<',
    }
}

/// Creature markers (lowercase awake, uppercase asleep): brute, archer,
/// lookout, chief, boss, and the generic letters.
pub fn is_creature_marker(symbol: char) -> bool {
    "bagcxefijklnpqrtuwyz".contains(symbol.to_ascii_lowercase())
}

/// A cell one can stand on.
pub fn is_walkable(symbol: char) -> bool {
    matches!(symbol, '.' | 'S' | '$' | 'm' | 'D' | 'O' | '*' | 'H' | '~') || is_creature_marker(symbol)
}

/// A cell one can stand on without an obstacle (pillars are obstacles).
pub fn is_open_floor(symbol: char) -> bool { is_walkable(symbol) && symbol != 'O' }

/// A multi-level plan. Cells absent from the map are empty (the rock around).
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub cells: BTreeMap<Cell, char>,
}

impl Plan {
    pub fn get(&self, cell: Cell) -> char { self.cells.get(&cell).copied().unwrap_or(' ') }

    pub fn is_free(&self, cell: Cell) -> bool { !self.cells.contains_key(&cell) }

    /// Reads a plan: sections `[étage N]` (or a single level 0 without
    /// section), lines starting with `;` or `@` ignored, all sections aligned
    /// on the same top-left corner. Spaces are empty cells.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut cells = BTreeMap::new();
        let mut level = 0;
        let mut row = 0;
        let mut seen_levels = Vec::new();
        for line in text.lines() {
            let line = line.trim_end_matches('\r');
            if line.starts_with(';') || line.starts_with('@') {
                continue;
            }
            if let Some(header) = line.strip_prefix('[') {
                let number = header
                    .strip_suffix(']')
                    .and_then(|h| h.trim().strip_prefix("étage"))
                    .and_then(|n| n.trim().parse::<i32>().ok())
                    .ok_or_else(|| format!("en-tête de section illisible : « {line} »"))?;
                if seen_levels.contains(&number) {
                    return Err(format!("étage {number} en double"));
                }
                seen_levels.push(number);
                level = number;
                row = 0;
                continue;
            }
            for (x, symbol) in line.chars().enumerate() {
                if symbol != ' ' {
                    cells.insert(Cell::new(x as i32, row, level), symbol);
                }
            }
            row += 1;
        }
        if cells.is_empty() {
            return Err("plan vide".into());
        }
        Ok(Self { cells })
    }

    /// The plan as text, highest level first.
    pub fn to_text(&self) -> String {
        let Some((min_x, max_x, min_y, max_y)) = self.bounds() else {
            return String::new();
        };
        let levels = self.cells.keys().map(|c| c.level);
        let (low, high) = (levels.clone().min().unwrap_or(0), levels.max().unwrap_or(0));
        let mut text = String::new();
        for level in (low..=high).rev() {
            text.push_str(&format!("[étage {level}]\n"));
            for y in min_y..=max_y {
                let row: String = (min_x..=max_x)
                    .map(|x| self.get(Cell::new(x, y, level)))
                    .collect();
                text.push_str(row.trim_end());
                text.push('\n');
            }
        }
        text
    }

    /// (min x, max x, min y, max y) of the non-empty cells.
    pub fn bounds(&self) -> Option<(i32, i32, i32, i32)> {
        let xs = self.cells.keys().map(|c| c.x);
        let ys = self.cells.keys().map(|c| c.y);
        Some((xs.clone().min()?, xs.max()?, ys.clone().min()?, ys.max()?))
    }

    /// Where one can go from `cell` in one move: walk to a neighbour, climb
    /// stairs (only from their foot), climb a ladder through the hole above
    /// it, fall through a hole down to the first thing to stand on.
    pub fn moves(&self, cell: Cell) -> Vec<Cell> {
        let mut moves = Vec::new();
        let here = self.get(cell);
        if let Some((dx, dy)) = stairs_direction(here) {
            // On stairs: down to their foot, up to the landing.
            moves.push(cell.offset(-dx, -dy, 0));
            moves.push(cell.offset(dx, dy, 1));
            return moves;
        }
        if here == 'H' {
            let hole = cell.offset(0, 0, 1);
            if self.get(hole) == '_' {
                for (dx, dy) in DIRECTIONS {
                    if is_walkable(self.get(hole.offset(dx, dy, 0))) {
                        moves.push(hole.offset(dx, dy, 0));
                    }
                }
            }
        }
        for (dx, dy) in DIRECTIONS {
            let next = cell.offset(dx, dy, 0);
            let symbol = self.get(next);
            if is_walkable(symbol) {
                moves.push(next);
            } else if stairs_direction(symbol) == Some((dx, dy)) {
                moves.push(next);
            } else if symbol == '_' {
                if let Some(landing) = self.fall(next) {
                    moves.push(landing);
                }
            }
        }
        moves
    }

    /// Where one lands falling into the hole `cell`.
    fn fall(&self, mut cell: Cell) -> Option<Cell> {
        while self.get(cell) == '_' {
            cell = cell.offset(0, 0, -1);
        }
        let symbol = self.get(cell);
        (is_walkable(symbol) || stairs_direction(symbol).is_some()).then_some(cell)
    }

    /// Cells reachable from `start`.
    pub fn reachable(&self, start: Cell) -> std::collections::BTreeSet<Cell> {
        let mut seen = std::collections::BTreeSet::from([start]);
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(cell) = queue.pop_front() {
            for next in self.moves(cell) {
                if seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_levels_and_moves() {
        let plan = Plan::parse(
            "; test\n[étage 1]\n#####\n#_..#\n#####\n\n[étage 0]\n#####\n#.>.#\n#####\n",
        )
        .unwrap();
        assert_eq!(plan.get(Cell::new(2, 1, 0)), '>');
        let reached = plan.reachable(Cell::new(1, 1, 0));
        // Up the stairs to the landing, and back down through the hole.
        assert!(reached.contains(&Cell::new(3, 1, 1)));
        assert!(plan.reachable(Cell::new(3, 1, 1)).contains(&Cell::new(1, 1, 0)));
        assert!(plan.to_text().starts_with("[étage 1]\n#####\n#_..#"));
    }
}
