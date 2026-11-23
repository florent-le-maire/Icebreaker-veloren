//! Icebreaker F.46: the outline of the plan of a generated dungeon (`assembly.silhouette` of `generation.ron`, or a
//! profile of `formes.ron`), an idea taken from the dungeon layouts of donjon.bin.sh (not its code): rooms and
//! corridors stay inside a shape `etendue` cells wide on each side of its middle, the same on every level. `Libre`
//! is no outline at all: the dungeon grows where it finds room, as before.

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Silhouette {
    #[default]
    Libre,
    Carre,
    /// Half as deep as wide.
    Rectangle,
    /// Two arms a third of the square wide.
    Croix,
    Rond,
    /// A square without its middle: one walks around.
    Couronne,
    /// The two diagonals of the square.
    Diagonales,
    Losange,
    Hexagone,
}

impl Silhouette {
    pub const TOUTES: [Self; 9] = [
        Self::Libre,
        Self::Carre,
        Self::Rectangle,
        Self::Croix,
        Self::Rond,
        Self::Couronne,
        Self::Diagonales,
        Self::Losange,
        Self::Hexagone,
    ];
    /// The names of `TOUTES`, as written in the data.
    pub const NOMS: &'static [&'static str] =
        &["Libre", "Carre", "Rectangle", "Croix", "Rond", "Couronne", "Diagonales", "Losange", "Hexagone"];

    pub fn nom(self) -> &'static str { Self::NOMS[Self::TOUTES.iter().position(|s| *s == self).unwrap_or(0)] }

    /// Whether the cell `(x, y)`, counted from the middle of the outline, is inside it (`r`: cells from the
    /// middle to the edge). Whole numbers only: the same on every machine.
    pub fn dedans(self, x: i32, y: i32, r: i32) -> bool {
        let (ax, ay) = (x.abs(), y.abs());
        let carre = ax <= r && ay <= r;
        match self {
            Self::Libre => true,
            Self::Carre => carre,
            Self::Rectangle => ax <= r && 2 * ay <= r,
            Self::Croix => carre && (3 * ax <= r || 3 * ay <= r),
            Self::Rond => ax * ax + ay * ay <= r * r,
            Self::Couronne => carre && 5 * ax.max(ay) >= 2 * r,
            Self::Diagonales => carre && 2 * (ax - ay).abs() <= r,
            Self::Losange => ax + ay <= r,
            Self::Hexagone => 100 * ay <= 87 * r && 100 * ax + 58 * ay <= 100 * r,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlines_are_symmetric_and_within_their_square() {
        let r = 24;
        for s in Silhouette::TOUTES {
            assert_eq!(ron::from_str::<Silhouette>(s.nom()).unwrap(), s);
            let mut inside = 0;
            for x in -r - 3..=r + 3 {
                for y in -r - 3..=r + 3 {
                    let d = s.dedans(x, y, r);
                    assert_eq!(d, s.dedans(-x, y, r), "{s:?}");
                    assert_eq!(d, s.dedans(x, -y, r), "{s:?}");
                    if s != Silhouette::Libre {
                        assert!(!d || (x.abs() <= r && y.abs() <= r), "{s:?} sort de son carré");
                    }
                    inside += d as i32;
                }
            }
            // Room for a dungeon: a fifth of the square at least.
            assert!(5 * inside >= (2 * r + 1) * (2 * r + 1), "{s:?} : {inside} cases");
        }
        assert!(!Silhouette::Couronne.dedans(0, 0, r));
        assert!(Silhouette::Croix.dedans(r, 0, r) && !Silhouette::Croix.dedans(r, r, r));
    }
}
