//! Icebreaker U.19: the veins of ore (`docs/icebreaker/design/filons.md`).
//! A vein is a band of blocks taken in the rock at the surface of a wall, a
//! ceiling or a floor: it winds along the surface, has a width, goes a block
//! or two into the rock, and here and there a block sticks out into the air.
//! What the ore is, and where the vein starts, is up to the caller (the game,
//! or the workshop page).

use crate::{Caverne, Genre, Grille, Zone, bruit::Alea};
use std::collections::HashSet;

/// The shape of one vein, its values drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FormeFilon {
    /// Blocks walked along the surface.
    pub longueur: i32,
    /// Width of the band, in blocks.
    pub epaisseur: i32,
    /// Blocks taken in the rock under the surface.
    pub profondeur: i32,
    /// From 0 (straight) to 1 (turns all the time).
    pub sinuosite: f32,
    /// Share of the surface blocks that stick out a block into the air.
    pub relief: f32,
}

/// A vein: the rock blocks it turns to ore, and the air blocks its bumps fill.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filon {
    pub roche: Vec<[i32; 3]>,
    pub relief: Vec<[i32; 3]>,
}

impl Filon {
    pub fn blocs(&self) -> impl Iterator<Item = &[i32; 3]> { self.roche.iter().chain(&self.relief) }

    pub fn len(&self) -> usize { self.roche.len() + self.relief.len() }

    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

const FACES: [[i32; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

fn plus(a: [i32; 3], b: [i32; 3]) -> [i32; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }

/// Rock that touches the air by a face.
pub fn surface(g: &Grille, p: [i32; 3]) -> bool {
    !g.air(p[0], p[1], p[2]) && FACES.iter().any(|f| { let q = plus(p, *f); g.air(q[0], q[1], q[2]) })
}

/// Whether some air lies within `profondeur` blocks of `p`, straight along a face.
fn pres_de_l_air(g: &Grille, p: [i32; 3], profondeur: i32) -> bool {
    FACES.iter().any(|f| (1..=profondeur).any(|k| g.air(p[0] + f[0] * k, p[1] + f[1] * k, p[2] + f[2] * k)))
}

/// From the air around `p` into the rock: the axis away from its air faces.
fn vers_la_roche(g: &Grille, p: [i32; 3]) -> Option<[i32; 3]> {
    let mut n = [0; 3];
    for f in FACES {
        let q = plus(p, f);
        if g.air(q[0], q[1], q[2]) {
            for k in 0..3 {
                n[k] -= f[k];
            }
        }
    }
    let k = (0..3).max_by_key(|k| n[*k].abs())?;
    (n[k] != 0).then(|| {
        let mut d = [0; 3];
        d[k] = n[k].signum();
        d
    })
}

/// Traces a vein from `depart`, a block of rock at the surface. `libre` says
/// where the vein may go (a rock block it takes, or an air block a bump
/// fills): the caller keeps it off the trail, the safe room, the creatures.
pub fn tracer(g: &Grille, depart: [i32; 3], forme: &FormeFilon, rng: &mut Alea, libre: &dyn Fn([i32; 3]) -> bool) -> Filon {
    if !surface(g, depart) || !libre(depart) {
        return Filon::default();
    }
    // The walk along the surface: a direction kept from step to step, turned
    // by the winding.
    let mut chemin = vec![depart];
    let mut vus: HashSet<[i32; 3]> = HashSet::from([depart]);
    let angle = rng.entre(0.0, std::f32::consts::TAU);
    let mut cap = [angle.cos(), angle.sin(), rng.entre(-0.3, 0.3)];
    while (chemin.len() as i32) < forme.longueur.max(1) {
        let p = chemin[chemin.len() - 1];
        let mut meilleur: Option<([i32; 3], f32)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let q = plus(p, [dx, dy, dz]);
                    if (dx, dy, dz) == (0, 0, 0) || vus.contains(&q) || !surface(g, q) || !libre(q) {
                        continue;
                    }
                    let n = ((dx * dx + dy * dy + dz * dz) as f32).sqrt();
                    let suit = (cap[0] * dx as f32 + cap[1] * dy as f32 + cap[2] * dz as f32) / n;
                    let note = suit + forme.sinuosite * rng.entre(-1.0, 1.0);
                    if meilleur.is_none_or(|(_, m)| note > m) {
                        meilleur = Some((q, note));
                    }
                }
            }
        }
        let Some((q, _)) = meilleur else { break };
        // The direction follows the step taken, and turns a little.
        let pas = [(q[0] - p[0]) as f32, (q[1] - p[1]) as f32, (q[2] - p[2]) as f32];
        let tourne = forme.sinuosite * rng.entre(-1.0, 1.0);
        let (s, c) = tourne.sin_cos();
        let lisse = [0.5 * cap[0] + 0.5 * pas[0], 0.5 * cap[1] + 0.5 * pas[1], 0.5 * cap[2] + 0.5 * pas[2]];
        cap = [lisse[0] * c - lisse[1] * s, lisse[0] * s + lisse[1] * c, lisse[2] * 0.7];
        let n = (cap[0] * cap[0] + cap[1] * cap[1] + cap[2] * cap[2]).sqrt().max(1e-3);
        cap = cap.map(|e| e / n);
        vus.insert(q);
        chemin.push(q);
    }

    // The band: around each block of the walk, the rock near the air within
    // half the width, and as deep as the vein goes.
    let rayon = forme.epaisseur.max(1) as f32 / 2.0;
    let r = rayon.ceil() as i32;
    let mut roche: Vec<[i32; 3]> = Vec::new();
    let mut pris: HashSet<[i32; 3]> = HashSet::new();
    let mut prendre = |q: [i32; 3], roche: &mut Vec<[i32; 3]>| {
        if !g.air(q[0], q[1], q[2]) && libre(q) && pris.insert(q) {
            roche.push(q);
        }
    };
    for c in &chemin {
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if ((dx * dx + dy * dy + dz * dz) as f32).sqrt() > rayon + 0.01 {
                        continue;
                    }
                    let q = plus(*c, [dx, dy, dz]);
                    if g.air(q[0], q[1], q[2]) || !pres_de_l_air(g, q, forme.profondeur.max(1)) {
                        continue;
                    }
                    prendre(q, &mut roche);
                    if let Some(d) = vers_la_roche(g, q) {
                        for k in 1..forme.profondeur.max(1) {
                            prendre([q[0] + d[0] * k, q[1] + d[1] * k, q[2] + d[2] * k], &mut roche);
                        }
                    }
                }
            }
        }
    }

    // The bumps: a block of ore in the air, in front of some surface blocks.
    let mut relief = Vec::new();
    let mut bosses: HashSet<[i32; 3]> = HashSet::new();
    for q in &roche {
        if !surface(g, *q) || !rng.chance(forme.relief) {
            continue;
        }
        let Some(d) = vers_la_roche(g, *q) else { continue };
        let a = [q[0] - d[0], q[1] - d[1], q[2] - d[2]];
        if g.air(a[0], a[1], a[2]) && libre(a) && bosses.insert(a) {
            relief.push(a);
        }
    }
    Filon { roche, relief }
}

/// One ore as the placing sees it (`minerais.ron`): the shape of its veins
/// (ranges drawn per vein), its weight by level and in the ore room.
#[derive(Clone, Debug, PartialEq)]
pub struct Gisement {
    pub longueur: (i32, i32),
    pub epaisseur: (i32, i32),
    pub profondeur: i32,
    pub sinuosite: f32,
    pub relief: f32,
    pub poids: Vec<f32>,
    pub salle: f32,
}

impl Gisement {
    fn poids_au(&self, niveau: i32) -> f32 {
        let i = (niveau.max(0) as usize).min(self.poids.len().saturating_sub(1));
        self.poids.get(i).copied().unwrap_or(0.0)
    }
}

/// A vein placed: its ore (its place among the `Gisement`s), its blocks, and
/// whether it lies in the ore room.
#[derive(Clone, Debug, PartialEq)]
pub struct Pose {
    pub minerai: usize,
    pub filon: Filon,
    pub salle: bool,
}

/// An ore by the weights `poids`; `tirage` from 0 to 1.
fn tirer(gisements: &[Gisement], tirage: f32, poids: impl Fn(&Gisement) -> f32) -> Option<usize> {
    let total: f32 = gisements.iter().map(&poids).sum();
    if total <= 0.0 {
        return None;
    }
    let mut reste = tirage * total;
    let mut dernier = None;
    for (i, g) in gisements.iter().enumerate() {
        let p = poids(g);
        if p <= 0.0 {
            continue;
        }
        if reste < p {
            return Some(i);
        }
        reste -= p;
        dernier = Some(i);
    }
    dernier
}

/// The level of each zone: by how far it is along the trail, from 0 to
/// `niveaux - 1`; a niche and the ore room have the level of the zone they
/// open on.
pub fn niveaux_des_zones(c: &Caverne, niveaux: i32) -> Vec<i32> {
    let niveaux = niveaux.max(1);
    (0..c.zones.len())
        .map(|i| {
            let i = if matches!(c.zones[i].genre, Genre::Niche | Genre::Minerai) { c.zones[i].parent.unwrap_or(i) } else { i };
            ((c.avancee(i) * niveaux as f32) as i32).min(niveaux - 1)
        })
        .collect()
}

/// Places the veins of a cave: in each zone (not a niche, the safe room or
/// the ore room) `par_zone` veins by its level (the last value for the deeper
/// levels), starting at a wall within reach; `salle` veins over the walls and
/// ceiling of the ore room. Never on the trail, a wall to break, in the safe
/// room; no bump over the trail nor where something stands (`occupe`, the
/// blocks one stands in).
pub fn poser(c: &Caverne, gisements: &[Gisement], niveaux: &[i32], par_zone: &[i32], salle: i32, occupe: &dyn Fn([i32; 3]) -> bool, graine: u64) -> Vec<Pose> {
    let g = &c.grille;
    let zones = &c.zones;
    let mut rng = Alea::new(graine ^ 0xf11_0115);
    let mut pris: HashSet<[i32; 3]> = HashSet::new();
    let mut poses: Vec<Pose> = Vec::new();
    let abri = c.abri.map(|a| &zones[a]);
    let tracer_un = |depart: [i32; 3], minerai: usize, rng: &mut Alea, pris: &mut HashSet<[i32; 3]>, dans: Option<&Zone>| {
        let gi = &gisements[minerai];
        let entre = |rng: &mut Alea, (a, b): (i32, i32)| a + rng.indice((b - a).max(0) as usize + 1) as i32;
        let forme = FormeFilon {
            longueur: entre(rng, gi.longueur),
            epaisseur: entre(rng, gi.epaisseur),
            profondeur: gi.profondeur,
            sinuosite: gi.sinuosite,
            relief: gi.relief,
        };
        let libre = |p: [i32; 3]| {
            if pris.contains(&p)
                || abri.is_some_and(|a| a.contient(p, 1.3))
                || dans.is_some_and(|z| !z.contient(p, 1.6))
                || g.mur(p[0], p[1], p[2])
                || g.sentier(p[0], p[1], p[2])
            {
                return false;
            }
            if !g.air(p[0], p[1], p[2]) {
                return true;
            }
            let sentier = (-2..=2).any(|dx| (-2..=2).any(|dy| (1..=3).any(|dz| g.sentier(p[0] + dx, p[1] + dy, p[2] - dz))));
            !sentier && !(0..=2).any(|dz| occupe([p[0], p[1], p[2] - dz]))
        };
        let filon = tracer(g, depart, &forme, rng, &libre);
        // Too short to be seen as a vein: another place.
        if filon.roche.len() < (forme.longueur as usize).min(4) {
            return None;
        }
        pris.extend(filon.blocs().copied());
        Some(filon)
    };
    for (i, zone) in zones.iter().enumerate() {
        if matches!(zone.genre, Genre::Niche | Genre::Abri | Genre::Minerai) {
            continue;
        }
        let n = par_zone.get((niveaux[i].max(0) as usize).min(par_zone.len().saturating_sub(1))).copied().unwrap_or(0);
        // Rock at the surface of a wall beside a floor block, within reach.
        let mut departs: Vec<[i32; 3]> = zone
            .sols
            .iter()
            .flat_map(|p| {
                [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .into_iter()
                    .flat_map(move |(dx, dy)| (0..=2).map(move |dz| [p[0] + dx, p[1] + dy, p[2] + dz]))
            })
            .filter(|q| surface(g, *q))
            .collect();
        departs.sort();
        departs.dedup();
        if departs.is_empty() {
            continue;
        }
        for _ in 0..n.max(0) {
            let Some(minerai) = tirer(gisements, rng.unite(), |x| x.poids_au(niveaux[i])) else { break };
            for _ in 0..12 {
                let depart = departs[rng.indice(departs.len())];
                if let Some(filon) = tracer_un(depart, minerai, &mut rng, &mut pris, None) {
                    poses.push(Pose { minerai, filon, salle: false });
                    break;
                }
            }
        }
    }
    if let Some(m) = c.minerai {
        let room = &zones[m];
        let reach = room.rayon.map(|r| (r as f32 * 1.6) as i32 + 2);
        let mut departs = Vec::new();
        for x in -reach[0]..=reach[0] {
            for y in -reach[1]..=reach[1] {
                for z in -reach[2]..=reach[2] {
                    let p = [room.centre[0] + x, room.centre[1] + y, room.centre[2] + z];
                    if room.contient(p, 1.5) && surface(g, p) {
                        departs.push(p);
                    }
                }
            }
        }
        for _ in 0..salle.max(0) {
            let Some(minerai) = tirer(gisements, rng.unite(), |x| x.salle) else { break };
            for _ in 0..12 {
                if departs.is_empty() {
                    break;
                }
                let depart = departs[rng.indice(departs.len())];
                if let Some(filon) = tracer_un(depart, minerai, &mut rng, &mut pris, Some(room)) {
                    poses.push(Pose { minerai, filon, salle: true });
                    break;
                }
            }
        }
    }
    poses
}

/// Icebreaker U.21: what the workshop page needs to place veins with the
/// settings of the game, read from `minerais.ron` and `grotte.ron` (the rest of
/// those files is left to the game). Its cave has neither the ground of the
/// world nor the inhabitants of the game: same look and count, other places.
#[derive(Clone, Debug, PartialEq)]
pub struct Reglages {
    pub gisements: Vec<Gisement>,
    /// The colour of each ore, in the same order.
    pub couleurs: Vec<[u8; 3]>,
    pub par_zone: Vec<i32>,
    pub salle: i32,
    pub niveaux: i32,
}

#[derive(serde::Deserialize)]
struct MineraisRon {
    minerais: std::collections::BTreeMap<String, MineraiRon>,
}

#[derive(serde::Deserialize)]
struct MineraiRon {
    couleur: (u8, u8, u8),
    filon: FormeRon,
    poids: Vec<f32>,
    salle: f32,
}

#[derive(serde::Deserialize)]
struct FormeRon {
    longueur: (i32, i32),
    epaisseur: (i32, i32),
    profondeur: i32,
    sinuosite: f32,
    relief: f32,
}

#[derive(serde::Deserialize)]
struct GrotteRon {
    filons: FilonsRon,
}

#[derive(serde::Deserialize)]
struct FilonsRon {
    veines: VeinesRon,
    salle: SalleRon,
    niveaux: i32,
}

#[derive(serde::Deserialize)]
struct VeinesRon {
    par_zone: Vec<i32>,
}

#[derive(serde::Deserialize)]
struct SalleRon {
    filons: i32,
}

pub fn lire_reglages(minerais: &str, grotte: &str) -> Result<Reglages, String> {
    let m: MineraisRon = ron::from_str(minerais).map_err(|e| format!("minerais.ron : {e}"))?;
    let g: GrotteRon = ron::from_str(grotte).map_err(|e| format!("grotte.ron : {e}"))?;
    let (gisements, couleurs) = m
        .minerais
        .into_values()
        .map(|m| {
            let f = m.filon;
            let gisement = Gisement {
                longueur: f.longueur,
                epaisseur: f.epaisseur,
                profondeur: f.profondeur,
                sinuosite: f.sinuosite,
                relief: f.relief,
                poids: m.poids,
                salle: m.salle,
            };
            (gisement, [m.couleur.0, m.couleur.1, m.couleur.2])
        })
        .unzip();
    Ok(Reglages {
        gisements,
        couleurs,
        par_zone: g.filons.veines.par_zone,
        salle: g.filons.salle.filons,
        niveaux: g.filons.niveaux.max(1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A box of air 20 blocks wide, 4 high, in rock.
    fn salle() -> Grille {
        let mut g = Grille::new([-20, -20, -10], [20, 20, 10]);
        for x in -10..10 {
            for y in -10..10 {
                for z in 0..4 {
                    g.creuser(x, y, z);
                }
            }
        }
        g
    }

    const FORME: FormeFilon = FormeFilon { longueur: 12, epaisseur: 2, profondeur: 2, sinuosite: 0.4, relief: 0.3 };

    #[test]
    fn un_filon_court_le_long_de_la_paroi() {
        let g = salle();
        let depart = [10, 0, 1];
        assert!(surface(&g, depart));
        for graine in 0..20 {
            let f = tracer(&g, depart, &FORME, &mut Alea::new(graine), &|_| true);
            assert!(f.roche.len() >= FORME.longueur as usize, "graine {graine} : {} bloc(s)", f.roche.len());
            assert!(f.roche.iter().all(|p| !g.air(p[0], p[1], p[2])), "du minerai dans l'air");
            assert!(f.relief.iter().all(|p| g.air(p[0], p[1], p[2])), "une bosse dans la roche");
            // Jamais plus profond que la profondeur demandée.
            assert!(f.roche.iter().all(|p| pres_de_l_air(&g, *p, FORME.profondeur)), "graine {graine}");
            // Pris dans la roche : des blocs derrière la surface.
            assert!(f.roche.iter().any(|p| !surface(&g, *p)), "graine {graine} : filon tout en surface");
            let total: HashSet<_> = f.blocs().collect();
            assert_eq!(total.len(), f.len(), "deux fois le même bloc");
        }
        // La même graine, le même filon.
        assert_eq!(tracer(&g, depart, &FORME, &mut Alea::new(3), &|_| true), tracer(&g, depart, &FORME, &mut Alea::new(3), &|_| true));
    }

    #[test]
    fn un_filon_reste_ou_on_le_laisse_aller() {
        let g = salle();
        let f = tracer(&g, [10, 0, 1], &FORME, &mut Alea::new(1), &|p| p[1] >= 0);
        assert!(!f.is_empty() && f.blocs().all(|p| p[1] >= 0));
        // Pas de départ dans l'air ou au cœur de la roche, ni là où c'est interdit.
        assert!(tracer(&g, [0, 0, 1], &FORME, &mut Alea::new(1), &|_| true).is_empty());
        assert!(tracer(&g, [15, 0, 1], &FORME, &mut Alea::new(1), &|_| true).is_empty());
        assert!(tracer(&g, [10, 0, 1], &FORME, &mut Alea::new(1), &|_| false).is_empty());
    }

    #[test]
    fn plus_epais_plus_de_blocs_sans_relief_pas_de_bosse() {
        let g = salle();
        let fin = FormeFilon { epaisseur: 1, profondeur: 1, relief: 0.0, ..FORME };
        let large = FormeFilon { epaisseur: 3, profondeur: 1, relief: 0.0, ..FORME };
        let (a, b) = (tracer(&g, [10, 0, 1], &fin, &mut Alea::new(2), &|_| true), tracer(&g, [10, 0, 1], &large, &mut Alea::new(2), &|_| true));
        assert!(a.relief.is_empty() && b.relief.is_empty());
        assert!(b.roche.len() > a.roche.len() * 2, "{} / {}", b.roche.len(), a.roche.len());
        // Épaisseur 1, profondeur 1 : rien que la surface.
        assert!(a.roche.iter().all(|p| surface(&g, *p)));
    }

    /// Les réglages du jeu se lisent, et une grotte de l'atelier porte des filons (posés par la même fonction que
    /// le jeu, pas aux mêmes places : l'atelier n'a ni le relief du monde ni les habitants).
    #[test]
    fn l_atelier_pose_des_filons_avec_les_reglages_du_jeu() {
        let r = lire_reglages(
            include_str!("../../assets/icebreaker/grottes/minerais.ron"),
            include_str!("../../assets/icebreaker/grottes/grotte.ron"),
        )
        .unwrap();
        assert!(r.gisements.len() >= 5 && r.couleurs.len() == r.gisements.len() && r.niveaux >= 2);
        let c = crate::generer(&crate::Forme::default(), 3, &|_, _| 0, i32::MIN);
        let niveaux = niveaux_des_zones(&c, r.niveaux);
        let poses = poser(&c, &r.gisements, &niveaux, &r.par_zone, r.salle, &|_| false, 3);
        assert!(poses.iter().filter(|p| !p.salle).count() >= 3, "{} filon(s)", poses.len());
        assert!(c.minerai.is_none() || poses.iter().any(|p| p.salle));
        assert_eq!(poses, poser(&c, &r.gisements, &niveaux, &r.par_zone, r.salle, &|_| false, 3));
    }
}
