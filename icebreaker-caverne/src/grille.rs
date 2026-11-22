//! The blocks of a cave: one bit per block (air or rock), plus what the
//! rock is (the trail, a concretion).

/// A box of blocks around the mouth (at 0, 0, 0); everything out of it is rock.
#[derive(Clone)]
pub struct Grille {
    pub min: [i32; 3],
    pub taille: [i32; 3],
    air: Vec<u64>,
    sentier: Vec<u64>,
    concretion: Vec<u64>,
    mur: Vec<u64>,
}

impl Grille {
    pub fn new(min: [i32; 3], max: [i32; 3]) -> Self {
        let taille = [max[0] - min[0], max[1] - min[1], max[2] - min[2]].map(|e| e.max(3));
        let words = (taille[0] as usize * taille[1] as usize * taille[2] as usize).div_ceil(64);
        Self {
            min,
            taille,
            air: vec![0; words],
            sentier: vec![0; words],
            concretion: vec![0; words],
            mur: vec![0; words],
        }
    }

    pub fn max(&self) -> [i32; 3] { [0, 1, 2].map(|i| self.min[i] + self.taille[i]) }

    #[inline]
    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let (x, y, z) = (x - self.min[0], y - self.min[1], z - self.min[2]);
        let [nx, ny, nz] = self.taille;
        // The outer shell stays rock: the cave never opens on the edge of the box.
        (x >= 1 && y >= 1 && z >= 1 && x < nx - 1 && y < ny - 1 && z < nz - 1)
            .then(|| (z as usize * ny as usize + y as usize) * nx as usize + x as usize)
    }

    /// The block of an index (see `cellules_air`).
    #[inline]
    pub fn position(&self, i: usize) -> [i32; 3] {
        let (nx, ny) = (self.taille[0] as usize, self.taille[1] as usize);
        [
            (i % nx) as i32 + self.min[0],
            (i / nx % ny) as i32 + self.min[1],
            (i / (nx * ny)) as i32 + self.min[2],
        ]
    }

    #[inline]
    fn bit(layer: &[u64], i: usize) -> bool { layer[i >> 6] >> (i & 63) & 1 == 1 }

    #[inline]
    pub fn air(&self, x: i32, y: i32, z: i32) -> bool { self.index(x, y, z).is_some_and(|i| Self::bit(&self.air, i)) }

    /// Rock of the trail (the block one walks on).
    #[inline]
    pub fn sentier(&self, x: i32, y: i32, z: i32) -> bool {
        self.index(x, y, z).is_some_and(|i| Self::bit(&self.sentier, i))
    }

    /// Rock of a stalactite, a stalagmite or a pillar.
    #[inline]
    pub fn concretion(&self, x: i32, y: i32, z: i32) -> bool {
        self.index(x, y, z).is_some_and(|i| Self::bit(&self.concretion, i))
    }

    /// Icebreaker U.13: rock of a wall to break, put back across a way between two zones.
    #[inline]
    pub fn mur(&self, x: i32, y: i32, z: i32) -> bool { self.index(x, y, z).is_some_and(|i| Self::bit(&self.mur, i)) }

    pub fn creuser(&mut self, x: i32, y: i32, z: i32) {
        if let Some(i) = self.index(x, y, z) {
            self.air[i >> 6] |= 1 << (i & 63);
            self.sentier[i >> 6] &= !(1 << (i & 63));
            self.concretion[i >> 6] &= !(1 << (i & 63));
            self.mur[i >> 6] &= !(1 << (i & 63));
        }
    }

    /// Puts rock back: plain (0), of the trail (1), of a concretion (2) or of a
    /// wall to break (3).
    pub fn remplir(&mut self, x: i32, y: i32, z: i32, nature: u8) {
        if let Some(i) = self.index(x, y, z) {
            let bit = 1 << (i & 63);
            self.air[i >> 6] &= !bit;
            self.sentier[i >> 6] = if nature == 1 { self.sentier[i >> 6] | bit } else { self.sentier[i >> 6] & !bit };
            self.concretion[i >> 6] =
                if nature == 2 { self.concretion[i >> 6] | bit } else { self.concretion[i >> 6] & !bit };
            self.mur[i >> 6] = if nature == 3 { self.mur[i >> 6] | bit } else { self.mur[i >> 6] & !bit };
        }
    }

    /// Every air block, by index (`position` gives the block).
    pub fn cellules_air(&self) -> impl Iterator<Item = usize> + '_ {
        self.air.iter().enumerate().filter(|(_, w)| **w != 0).flat_map(|(wi, w)| {
            let mut w = *w;
            std::iter::from_fn(move || {
                (w != 0).then(|| {
                    let b = w.trailing_zeros() as usize;
                    w &= w - 1;
                    wi * 64 + b
                })
            })
        })
    }

    pub fn volume(&self) -> usize { self.air.iter().map(|w| w.count_ones() as usize).sum() }

    /// Keeps only the air one reaches from `depart` through air (the bubbles
    /// closed in the rock are filled back).
    pub fn garder_relie(&mut self, depart: [i32; 3]) {
        let Some(start) = self.index(depart[0], depart[1], depart[2]) else { return };
        if !Self::bit(&self.air, start) {
            return;
        }
        let mut seen = vec![0u64; self.air.len()];
        seen[start >> 6] |= 1 << (start & 63);
        let mut stack = vec![start];
        let (nx, ny) = (self.taille[0] as usize, self.taille[1] as usize);
        let steps = [1, nx, nx * ny];
        while let Some(i) = stack.pop() {
            for step in steps {
                for j in [i.wrapping_sub(step), i + step] {
                    // The shell is rock, so a neighbour of an air block is in the box.
                    if j < self.air.len() * 64 && Self::bit(&self.air, j) && !Self::bit(&seen, j) {
                        seen[j >> 6] |= 1 << (j & 63);
                        stack.push(j);
                    }
                }
            }
        }
        self.air = seen;
    }
}
