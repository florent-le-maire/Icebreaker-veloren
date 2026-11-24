//! The blocks of a forest: what stands over the ground (one byte per block),
//! the height of the ground and what it is (grass, trail, clearing) by column.

/// What a block is.
pub const AIR: u8 = 0;
/// A trunk: never replaced nor cleared.
pub const BOIS: u8 = 1;
/// The roots at the foot of a trunk.
pub const RACINE: u8 = 2;
pub const FEUILLES: u8 = 3;
/// A platform, the deck of a bridge, a step.
pub const PLANCHE: u8 = 4;
/// The rail of a bridge.
pub const RAMBARDE: u8 = 5;
/// A ladder: one climbs in it.
pub const ECHELLE: u8 = 6;
pub const BUISSON: u8 = 7;
/// Icebreaker FO.9: a lantern, hung where no one walks.
pub const LANTERNE: u8 = 8;
/// Icebreaker FO.9: a liana hanging under the leaves: one walks through it.
pub const LIANE: u8 = 9;

/// What the ground of a column is.
pub const HERBE: u8 = 0;
/// A trail between two clearings.
pub const SENTIER: u8 = 1;
pub const CLAIRIERE: u8 = 2;
/// A thin trail from the trails to the foot of a way up.
pub const SENTE: u8 = 3;

/// A box of blocks around the middle of the forest (0, 0), z up. Under
/// `sol(x, y)` is the ground; out of the box, nothing one can walk in.
#[derive(Clone, PartialEq)]
pub struct Grille {
    pub min: [i32; 3],
    pub taille: [i32; 3],
    blocs: Vec<u8>,
    sol: Vec<i32>,
    nature: Vec<u8>,
}

impl Grille {
    /// A box of `bord` blocks on each side of the middle, up to `haut`;
    /// `nature` is what the ground of each column is, row by row.
    pub fn new(bord: i32, surface: &dyn Fn(i32, i32) -> i32, nature: Vec<u8>, haut: i32) -> Self {
        let cote = 2 * bord + 1;
        let sol: Vec<i32> = (0..cote * cote).map(|i| surface(i % cote - bord, i / cote - bord)).collect();
        let bas = sol.iter().copied().min().unwrap_or(0);
        let sommet = haut.max(sol.iter().copied().max().unwrap_or(0) + 8);
        let taille = [cote, cote, sommet - bas];
        assert_eq!(nature.len(), sol.len());
        Self {
            min: [-bord, -bord, bas],
            taille,
            blocs: vec![AIR; taille[0] as usize * taille[1] as usize * taille[2] as usize],
            sol,
            nature,
        }
    }

    pub fn max(&self) -> [i32; 3] { [0, 1, 2].map(|i| self.min[i] + self.taille[i]) }

    #[inline]
    fn colonne(&self, x: i32, y: i32) -> Option<usize> {
        let (x, y) = (x - self.min[0], y - self.min[1]);
        (x >= 0 && y >= 0 && x < self.taille[0] && y < self.taille[1])
            .then(|| y as usize * self.taille[0] as usize + x as usize)
    }

    /// The index of a block of the box.
    #[inline]
    pub fn indice(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let z = z - self.min[2];
        let c = self.colonne(x, y)?;
        (z >= 0 && z < self.taille[2]).then(|| z as usize * self.sol.len() + c)
    }

    #[inline]
    pub fn position(&self, i: usize) -> [i32; 3] {
        let (nx, plan) = (self.taille[0] as usize, self.sol.len());
        [
            (i % nx) as i32 + self.min[0],
            (i % plan / nx) as i32 + self.min[1],
            (i / plan) as i32 + self.min[2],
        ]
    }

    /// The lowest block over the ground of a column (out of the box: none,
    /// the ground is everywhere).
    #[inline]
    pub fn sol(&self, x: i32, y: i32) -> i32 { self.colonne(x, y).map_or(i32::MAX, |c| self.sol[c]) }

    #[inline]
    pub fn nature(&self, x: i32, y: i32) -> u8 { self.colonne(x, y).map_or(HERBE, |c| self.nature[c]) }

    pub fn marquer(&mut self, x: i32, y: i32, nature: u8) {
        if let Some(c) = self.colonne(x, y) {
            self.nature[c] = nature;
        }
    }

    #[inline]
    pub fn bloc(&self, x: i32, y: i32, z: i32) -> u8 { self.indice(x, y, z).map_or(AIR, |i| self.blocs[i]) }

    /// Puts a block over the ground; a trunk stays a trunk.
    pub fn poser(&mut self, x: i32, y: i32, z: i32, bloc: u8) {
        if let Some(i) = self.indice(x, y, z)
            && z >= self.sol(x, y)
            && self.blocs[i] != BOIS
        {
            self.blocs[i] = bloc;
        }
    }

    /// Makes room: leaves, roots and bushes go, and what was built too if
    /// `construit`. A trunk stays.
    pub fn degager(&mut self, x: i32, y: i32, z: i32, construit: bool) {
        if let Some(i) = self.indice(x, y, z) {
            let b = self.blocs[i];
            if matches!(b, RACINE | FEUILLES | BUISSON) || (construit && matches!(b, PLANCHE | RAMBARDE)) {
                self.blocs[i] = AIR;
            }
        }
    }

    /// Whether a body can be in this block (air or a ladder, over the ground).
    #[inline]
    pub fn passable(&self, x: i32, y: i32, z: i32) -> bool {
        z >= self.sol(x, y) && matches!(self.bloc(x, y, z), AIR | ECHELLE | LIANE)
    }

    /// Every block that is not air, with its place.
    pub fn pleins(&self) -> impl Iterator<Item = ([i32; 3], u8)> + '_ {
        self.blocs.iter().enumerate().filter(|(_, b)| **b != AIR).map(|(i, b)| (self.position(i), *b))
    }

    pub fn nombre(&self) -> usize { self.blocs.len() }
}
