//! A forest as triangles, for the workshop page: one square per face between
//! a block and the air, turned toward the air, and the ground under it. The
//! leaves come last, so that the page can leave them out.

use crate::{
    Foret,
    grille::{AIR, BOIS, BUISSON, CLAIRIERE, ECHELLE, FEUILLES, PLANCHE, RACINE, RAMBARDE, SENTE, SENTIER},
};

pub struct Maillage {
    /// Three coordinates per vertex, in blocks.
    pub sommets: Vec<i16>,
    /// Red, green, blue per vertex.
    pub couleurs: Vec<u8>,
    /// Three vertices per triangle.
    pub indices: Vec<u32>,
    /// How many of `indices` come before the leaves.
    pub sans_feuilles: usize,
}

impl Maillage {
    /// The face of the block `p` on the side `cote` of `axe`, seen from there.
    fn face(&mut self, p: [i32; 3], axe: usize, cote: i32, base: [f32; 3]) {
        let ombre = match (axe, cote) {
            (2, 1) => 1.12,
            (2, _) => 0.5,
            (0, _) => 0.9,
            _ => 0.72,
        };
        let grain = 0.92 + 0.16 * ((p[0] * 7 + p[1] * 13 + p[2] * 29).rem_euclid(17) as f32 / 16.0);
        let (u, v) = ((axe + 1) % 3, (axe + 2) % 3);
        let mut o = p;
        if cote == 1 {
            o[axe] += 1;
        }
        let coins = if cote == 1 { [(0, 0), (1, 0), (1, 1), (0, 1)] } else { [(0, 0), (0, 1), (1, 1), (1, 0)] };
        let premier = (self.sommets.len() / 3) as u32;
        for (du, dv) in coins {
            let mut q = o;
            q[u] += du;
            q[v] += dv;
            self.sommets.extend(q.map(|e| e as i16));
            self.couleurs.extend(base.map(|e| (e * ombre * grain).clamp(0.0, 255.0) as u8));
        }
        self.indices.extend([premier, premier + 1, premier + 2, premier, premier + 2, premier + 3]);
    }
}

pub fn mailler(f: &Foret) -> Maillage {
    let g = &f.grille;
    let mut m = Maillage { sommets: Vec::new(), couleurs: Vec::new(), indices: Vec::new(), sans_feuilles: 0 };
    let (min, max) = (g.min, g.max());
    // The ground: its top, and its sides where it steps down.
    for y in min[1]..max[1] {
        for x in min[0]..max[0] {
            let z = g.sol(x, y);
            let base = match g.nature(x, y) {
                SENTIER => [205.0, 175.0, 115.0],
                CLAIRIERE => [130.0, 160.0, 85.0],
                SENTE => [170.0, 160.0, 105.0],
                _ => [72.0, 108.0, 60.0],
            };
            if g.bloc(x, y, z) == AIR {
                m.face([x, y, z - 1], 2, 1, base);
            }
            for (axe, cote) in [(0, 1), (0, -1), (1, 1), (1, -1)] {
                let mut n = [x, y];
                n[axe] += cote;
                let voisin = g.sol(n[0], n[1]);
                if voisin < z {
                    for zz in voisin..z {
                        m.face([x, y, zz], axe, cote, [120.0, 95.0, 70.0]);
                    }
                }
            }
        }
    }
    let haut = (max[2] - min[2]).max(1) as f32;
    for feuilles in [false, true] {
        if feuilles {
            m.sans_feuilles = m.indices.len();
        }
        for (p, bloc) in g.pleins().filter(|(_, b)| (*b == FEUILLES) == feuilles) {
            let base = match bloc {
                BOIS => [112.0, 82.0, 56.0],
                RACINE => [96.0, 72.0, 50.0],
                FEUILLES => {
                    // Lighter at the top of the forest.
                    let k = 0.75 + 0.45 * (p[2] - min[2]) as f32 / haut;
                    [58.0 * k, 122.0 * k, 54.0 * k]
                },
                PLANCHE => [200.0, 158.0, 98.0],
                RAMBARDE => [150.0, 112.0, 70.0],
                ECHELLE => [240.0, 205.0, 110.0],
                BUISSON => [84.0, 142.0, 62.0],
                _ => [255.0, 0.0, 255.0],
            };
            for axe in 0..3 {
                for cote in [1, -1] {
                    let mut n = p;
                    n[axe] += cote;
                    // Seen from the air; the ground hides the rest.
                    let sol = g.sol(n[0], n[1]);
                    // (Or from leaves, for a block that is not one: the page may hide them.)
                    let voisin = g.bloc(n[0], n[1], n[2]);
                    if (voisin == AIR || (voisin == FEUILLES && !feuilles)) && (sol == i32::MAX || n[2] >= sol) {
                        m.face(p, axe, cote, base);
                    }
                }
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Forme, generer};

    /// Every face is turned toward the air it bounds, and the leaves are last.
    #[test]
    fn faces_look_into_the_air() {
        let forme = Forme { emprise: 50.0, clairieres: 2.0, clairiere_rayon: 8.0, ..Forme::default() };
        let f = generer(&forme, 1, &|_, _| 0);
        let m = mailler(&f);
        assert!(m.sans_feuilles > 0 && m.sans_feuilles < m.indices.len());
        let sommet = |i: u32| [0, 1, 2].map(|k| m.sommets[i as usize * 3 + k] as f32);
        for t in m.indices.chunks(6) {
            let (a, b, d) = (sommet(t[0]), sommet(t[1]), sommet(t[2]));
            let (e1, e2) = ([0, 1, 2].map(|k| b[k] - a[k]), [0, 1, 2].map(|k| d[k] - a[k]));
            let normale = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let vers = [0, 1, 2].map(|k| ((a[k] + d[k]) / 2.0 + normale[k] * 0.5).floor() as i32);
            let g = &f.grille;
            let dehors = g.indice(vers[0], vers[1], vers[2]).is_none();
            // (Or the leaves, hidden or not, over a trunk or a platform.)
            let feuilles = g.bloc(vers[0], vers[1], vers[2]) == FEUILLES;
            assert!(dehors || feuilles || g.passable(vers[0], vers[1], vers[2]), "face vers {vers:?}");
        }
    }
}
