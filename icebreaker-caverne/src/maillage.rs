//! The walls of a cave as triangles, for the workshop page: one square per
//! face between an air block and a rock block, turned toward the air (seen
//! from outside, the near walls are culled: one looks into the cave).

use crate::Caverne;

pub struct Maillage {
    /// Three coordinates per vertex, in blocks.
    pub sommets: Vec<i16>,
    /// Red, green, blue per vertex.
    pub couleurs: Vec<u8>,
    /// Three vertices per triangle.
    pub indices: Vec<u32>,
}

pub fn mailler(c: &Caverne) -> Maillage { mailler_avec(c, &|_| None) }

/// Icebreaker U.21: the same, with the blocks of ore `minerai` gives a colour
/// to (a block of the rock, or a bump in the air, then solid).
pub fn mailler_avec(c: &Caverne, minerai: &dyn Fn([i32; 3]) -> Option<[u8; 3]>) -> Maillage {
    let g = &c.grille;
    let mut m = Maillage {
        sommets: Vec::new(),
        couleurs: Vec::new(),
        indices: Vec::new(),
    };
    let (bas, haut) = (g.min[2] as f32, g.max()[2] as f32);
    for i in g.cellules_air() {
        let p = g.position(i);
        if minerai(p).is_some() {
            continue;
        }
        // How closed in the block is: the rock among its 26 neighbours darkens it.
        let mut closed = None;
        for axis in 0..3 {
            for side in [1, -1] {
                let mut n = p;
                n[axis] += side;
                let ore = minerai(n);
                if g.air(n[0], n[1], n[2]) && ore.is_none() {
                    continue;
                }
                // What the rock is, then the light: by the way the face
                // looks, the depth, and a grain by block.
                let base: [f32; 3] = if let Some(o) = ore {
                    o.map(f32::from)
                } else if g.sentier(n[0], n[1], n[2]) {
                    [240.0, 190.0, 70.0]
                } else if g.concretion(n[0], n[1], n[2]) {
                    [205.0, 210.0, 225.0]
                } else {
                    match (axis, side) {
                        (2, -1) => [165.0, 135.0, 100.0],
                        (2, _) => [100.0, 95.0, 120.0],
                        _ => [135.0, 120.0, 120.0],
                    }
                };
                let shade = match (axis, side) {
                    (2, -1) => 1.15,
                    (2, _) => 0.55,
                    (0, _) => 0.95,
                    _ => 0.72,
                };
                let closed = *closed.get_or_insert_with(|| {
                    let mut rock = 0;
                    for dz in -1..=1 {
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                rock += !g.air(p[0] + dx, p[1] + dy, p[2] + dz) as i32;
                            }
                        }
                    }
                    rock as f32 / 26.0
                });
                let depth = ((p[2] as f32 - bas) / (haut - bas)).clamp(0.0, 1.0);
                let grain = 0.94 + 0.12 * ((p[0] * 7 + p[1] * 13 + p[2] * 29).rem_euclid(17) as f32 / 16.0);
                let light = shade * grain * (0.6 + 0.5 * depth) * (1.25 - 0.9 * closed);
                let colour = [
                    base[0] * light * (0.7 + 0.3 * depth),
                    base[1] * light * (0.85 + 0.15 * depth),
                    base[2] * light,
                ];
                // The four corners of the face, counter-clockwise seen from the air.
                let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                let mut o = p;
                if side == 1 {
                    o[axis] += 1;
                }
                let corners = if side == 1 { [(0, 0), (0, 1), (1, 1), (1, 0)] } else { [(0, 0), (1, 0), (1, 1), (0, 1)] };
                let first = (m.sommets.len() / 3) as u32;
                for (du, dv) in corners {
                    let mut q = o;
                    q[u] += du;
                    q[v] += dv;
                    m.sommets.extend(q.map(|e| e as i16));
                    m.couleurs.extend(colour.map(|e| e.clamp(0.0, 255.0) as u8));
                }
                m.indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Forme, generer};

    /// Every face is turned toward the air block it bounds.
    #[test]
    fn faces_look_into_the_air() {
        let forme = Forme {
            zones: 2.0,
            plaine_rayon: 15.0,
            chaos_rayon: 10.0,
            ..Forme::default()
        };
        let c = generer(&forme, 1, &|_, _| 0, i32::MIN);
        let m = mailler(&c);
        assert!(!m.indices.is_empty());
        let vertex = |i: u32| [0, 1, 2].map(|k| m.sommets[i as usize * 3 + k] as f32);
        for t in m.indices.chunks(6) {
            let (a, b, d) = (vertex(t[0]), vertex(t[1]), vertex(t[2]));
            let (e1, e2) = ([0, 1, 2].map(|k| b[k] - a[k]), [0, 1, 2].map(|k| d[k] - a[k]));
            let normal = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            // Half a block along the normal from the middle of the face: air.
            let mid = [0, 1, 2].map(|k| (a[k] + d[k]) / 2.0 + normal[k] * 0.5);
            assert!(c.grille.air(mid[0].floor() as i32, mid[1].floor() as i32, mid[2].floor() as i32));
        }
    }
}
