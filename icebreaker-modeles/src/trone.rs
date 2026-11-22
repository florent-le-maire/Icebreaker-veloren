//! Icebreaker F.36 : les modèles de la salle du trône (`docs/icebreaker/PLAN-PLAYTEST.md`, « Salle du trône
//! royale »). Épurée : peu d'objets, chacun travaillé ; marbre clair, pourpre et or ; la vie vient de la lumière
//! (vitrail, bougies), de la hauteur (piliers jusqu'au plafond) et du tapis qui mène au trône.
//!
//! Repère et ancres comme `cuisine.rs`. L'estrade se fait de blocs `estrade` (1 bloc) précédés d'une rangée de
//! `marche` (un demi-bloc) ; au milieu de la rangée, `tapis_marche` porte le tapis du sol jusque sur l'estrade, où il
//! continue en `tapis` posés à z 1.

use crate::{Axe, Matiere, Matiere::*, Modele, bruit};

pub fn modeles() -> Vec<Modele> {
    vec![trone(), estrade(), marche(), tapis(), tapis_bout(), tapis_marche(), pilier(), vitrail(), chandelier()]
}

/// Le tapis en travers (x de 0 à 32) : bordure d'or, frise pointillée, et un champ pourpre où des losanges d'or se
/// suivent, un par bloc (`y` le long du tapis, pour qu'ils se raccordent d'un bloc à l'autre).
fn tapis_motif(x: i32, y: i32) -> Matiere {
    match x.min(32 - x) {
        0 | 3 => PourpreSombre,
        1 | 2 => Or,
        4 if y.rem_euclid(4) < 2 => Or,
        4 | 5 => Pourpre,
        _ => {
            let d = (x - 16).abs() + (y.rem_euclid(11) - 5).abs();
            if (4..=5).contains(&d) {
                Or
            } else if d <= 2 {
                PourpreSombre
            } else {
                Pourpre
            }
        },
    }
}

/// Le trône : trois blocs de large, deux de profondeur, quatre de haut ; posé sur l'estrade (z 1), dos au mur. Un
/// siège de marbre au panneau encadré comme le sol, un coussin pourpre, des accoudoirs à boules d'or, un haut dossier
/// cintré, un disque d'or au-dessus du dossier rembourré, deux pinacles.
pub fn trone() -> Modele {
    let mut m = Modele::new("trone", (33, 22, 44), (16.5, 16.5, 0.0));
    // Le socle, bordé de sombre devant.
    m.boite((2, 1, 0), (30, 21, 2), Marbre);
    m.boite((2, 1, 2), (30, 1, 2), MarbreSombre);
    // Le siège et son panneau de devant : un cadre sombre, un losange d'or.
    m.boite((6, 3, 3), (26, 17, 11), Marbre);
    for x in 9..=23 {
        for z in 4..=10 {
            let bord = x == 9 || x == 23 || z == 4 || z == 10;
            m.mettre((x, 2, z), if bord { MarbreSombre } else { Marbre });
        }
    }
    for (x, z) in [(16, 6), (15, 7), (16, 7), (17, 7), (16, 8)] {
        m.mettre((x, 2, z), Or);
    }
    // Le coussin, un passepoil d'or devant, deux glands aux coins.
    m.boite((7, 3, 12), (25, 16, 13), Pourpre);
    m.boite((7, 3, 12), (25, 3, 12), Or);
    for x in [7, 25] {
        m.boite((x, 2, 10), (x, 2, 12), Or);
    }
    // Le dossier, cintré, son sommet doré.
    for x in 5..=27 {
        let dx = (x as f32 - 16.0) / 11.5;
        let haut = 36 + ((1.0 - dx * dx).max(0.0).sqrt() * 5.0).round() as i32;
        m.boite((x, 17, 3), (x, 21, haut), Marbre);
        m.boite((x, 17, haut), (x, 21, haut), Or);
    }
    // Le dossier rembourré, bordé d'or, et le disque d'or au-dessus, une pierre sombre au centre.
    for x in 9..=23 {
        for z in 14..=30 {
            let bord = x == 9 || x == 23 || z == 30;
            m.mettre((x, 16, z), if bord { Or } else { Pourpre });
        }
    }
    m.cylindre_axe(Axe::Y, (16.5, 35.0), 3.4, 16, 16, Or);
    m.boite((16, 15, 34), (17, 15, 35), PourpreSombre);
    // Les accoudoirs : une tablette sombre qui déborde devant, une boule d'or au bout, une rainure sur la face.
    for x0 in [3, 26] {
        m.boite((x0, 3, 3), (x0 + 3, 17, 15), Marbre);
        m.boite((x0, 2, 16), (x0 + 3, 17, 16), MarbreSombre);
        m.boite((x0 + 1, 2, 4), (x0 + 2, 2, 14), MarbreSombre);
        m.ellipsoide((x0 as f32 + 2.0, 3.5, 18.5), (2.2, 2.2, 2.0), Or);
    }
    // Les pinacles, de chaque côté du dossier, coiffés d'or.
    for x0 in [1, 28] {
        m.boite((x0, 17, 0), (x0 + 3, 21, 37), Marbre);
        m.boite((x0, 17, 0), (x0 + 3, 17, 0), MarbreSombre);
        m.boite((x0, 16, 36), (x0 + 3, 21, 37), MarbreSombre);
        m.ellipsoide((x0 as f32 + 2.0, 19.0, 40.5), (2.0, 2.0, 2.5), Or);
    }
    m
}

/// Un bloc de l'estrade : marbre plein, plinthe et moulure sombres (elles se suivent d'un bloc à l'autre), un joint
/// fin sur le dessus.
pub fn estrade() -> Modele {
    let mut m = Modele::new("estrade", (11, 11, 11), (5.5, 5.5, 0.0));
    m.boite((0, 0, 0), (10, 10, 10), Marbre);
    m.boite((0, 0, 0), (10, 10, 0), MarbreSombre);
    m.boite((0, 0, 8), (10, 10, 8), MarbreSombre);
    for i in 0..=10 {
        m.mettre((i, 0, 10), MarbreSombre);
        m.mettre((0, i, 10), MarbreSombre);
    }
    m
}

/// Une marche devant l'estrade : un demi-bloc de marbre, plinthe sombre.
pub fn marche() -> Modele {
    let mut m = Modele::new("marche", (11, 11, 6), (5.5, 5.5, 0.0));
    m.boite((0, 0, 0), (10, 10, 5), Marbre);
    m.boite((0, 0, 0), (10, 10, 0), MarbreSombre);
    m
}

/// Le tapis : trois blocs de large, un de long, à poser bout à bout vers le trône.
pub fn tapis() -> Modele {
    let mut m = Modele::new("tapis", (33, 11, 1), (16.5, 5.5, 0.0));
    for x in 0..33 {
        for y in 0..11 {
            m.mettre((x, y, 0), tapis_motif(x, y));
        }
    }
    m
}

/// Le bout du tapis : la bordure d'or qui le ferme et des franges devant (l'avant, -y, vers la porte).
pub fn tapis_bout() -> Modele {
    let mut m = Modele::new("tapis_bout", (33, 11, 1), (16.5, 5.5, 0.0));
    for x in 0..33 {
        for y in 4..11 {
            m.mettre((x, y, 0), tapis_motif(x, y));
        }
        if (1..=31).contains(&x) {
            m.mettre((x, 3, 0), Or);
            m.mettre((x, 2, 0), PourpreSombre);
        }
        if (2..=30).contains(&x) && x % 2 == 0 {
            m.boite((x, 0, 0), (x, 1, 0), Or);
        }
    }
    m
}

/// La marche du milieu, sous le tapis : le tapis descend devant la marche, couvre son dessus (et celui des marches
/// voisines, un voxel au-dessus) et monte la contremarche de l'estrade jusqu'à son dessus.
pub fn tapis_marche() -> Modele {
    // y 0 : la descente devant, dans le bloc d'avant ; y 1 à 11 : le bloc de la marche.
    let mut m = Modele::new("tapis_marche", (33, 12, 12), (16.5, 6.5, 0.0));
    m.boite((11, 1, 0), (21, 11, 5), Marbre);
    m.boite((11, 1, 0), (21, 11, 0), MarbreSombre);
    for x in 0..33 {
        for z in 1..=6 {
            m.mettre((x, 0, z), tapis_motif(x, z));
        }
        for y in 1..=10 {
            m.mettre((x, y, 6), tapis_motif(x, y + 6));
        }
        for z in 6..=11 {
            m.mettre((x, 11, z), tapis_motif(x, z + 11));
        }
    }
    m
}

/// Un pilier jusqu'au plafond (six blocs) : base carrée et tore de marbre, fût de pierre rose où monte une vigne
/// peinte, bague d'or, chapiteau et abaque. Le fût tient presque dans le bloc d'ancre (la collision).
pub fn pilier() -> Modele {
    let mut m = Modele::new("pilier", (33, 33, 66), (16.5, 16.5, 0.0));
    let c = (16.5, 16.5);
    m.boite((3, 3, 0), (29, 29, 4), Marbre);
    m.boite((3, 3, 0), (29, 29, 0), MarbreSombre);
    m.boite((5, 5, 5), (27, 27, 6), Marbre);
    m.cylindre(c, 10.5, 7, 8, Marbre);
    m.cylindre(c, 9.5, 9, 9, MarbreSombre);
    m.cylindre(c, 7.5, 10, 55, PierreRose);
    // La vigne : deux tiges en hélice sur la surface du fût, des feuilles qui partent de chaque côté.
    for z in 12..=50 {
        for x in 7..=26 {
            for y in 7..=26 {
                let (dx, dy) = (x as f32 + 0.5 - c.0, y as f32 + 0.5 - c.1);
                if m.voxel((x, y, z)).is_none() || dx * dx + dy * dy < 6.2 * 6.2 {
                    continue;
                }
                let u = dy.atan2(dx) / std::f32::consts::TAU * 48.0;
                let s = (u - z as f32 * 0.9).rem_euclid(24.0);
                let feuille = (s < 4.0 && z.rem_euclid(8) < 2) || (s > 21.0 && (4..6).contains(&z.rem_euclid(8)));
                if s < 1.8 || s > 22.6 || feuille {
                    m.mettre((x, y, z), Vigne);
                }
            }
        }
    }
    m.cylindre(c, 8.2, 52, 53, Or);
    m.cylindre(c, 9.5, 56, 56, MarbreSombre);
    m.cylindre(c, 10.5, 57, 59, Marbre);
    m.boite((5, 5, 60), (27, 27, 61), Marbre);
    m.boite((3, 3, 62), (29, 29, 65), Marbre);
    m.boite((3, 3, 62), (29, 29, 62), MarbreSombre);
    m
}

/// Un vitrail contre le mur : cadre de marbre en plein cintre, verres de couleur sertis de plomb en losanges, un
/// médaillon de feu au milieu. Les verres chauds luisent (indices 13 à 15) ; la lueur du bloc éclaire autour.
pub fn vitrail() -> Modele {
    let mut m = Modele::new("vitrail", (33, 3, 44), (16.5, -2.5, 0.0));
    let arc = |x: i32, z: i32, r: f32, bas: i32| {
        let (dx, dz) = (x as f32 + 0.5 - 16.5, z as f32 + 0.5 - 28.0);
        z >= bas && if z < 28 { dx.abs() <= r } else { dx * dx + dz * dz <= r * r }
    };
    for x in 0..33 {
        for z in 0..44 {
            if arc(x, z, 11.5, 4) {
                let (dx, dz) = (x as f32 + 0.5 - 16.5, z as f32 + 0.5 - 22.0);
                let r = (dx * dx + dz * dz).sqrt();
                let verre = if r < 2.6 {
                    Flamme
                } else if r < 5.4 {
                    Braise
                } else if r < 6.6 || (x + z).rem_euclid(6) == 0 || (x - z).rem_euclid(6) == 0 {
                    FerSombre
                } else {
                    match bruit((x + z).div_euclid(6), (x - z).div_euclid(6), 5) % 6 {
                        0..=3 => VerreBleu,
                        4 => VerreVert,
                        _ => Braise,
                    }
                };
                m.mettre((x, 1, z), verre);
            } else if arc(x, z, 15.5, 0) {
                let rebord = arc(x, z, 12.6, 3);
                m.boite((x, 0, z), (x, 2, z), if rebord { MarbreSombre } else { Marbre });
            }
        }
    }
    // L'appui, qui avance un peu.
    m.boite((1, 0, 0), (31, 2, 1), MarbreSombre);
    m
}

/// Un chandelier de fer sur pied, un peu plus de deux blocs : trépied, fût à deux nœuds, croix de bras, cinq bougies
/// allumées (la cire coule, les flammes luisent).
pub fn chandelier() -> Modele {
    let mut m = Modele::new("chandelier", (11, 11, 34), (5.5, 5.5, 0.0));
    for pied in [(0, 1), (10, 1), (5, 10)] {
        for dz in 0..=1 {
            m.ligne((5, 5, 6 + dz), (pied.0, pied.1, dz), FerSombre);
            m.ligne((6, 6, 6 + dz), (pied.0, pied.1, dz), FerSombre);
        }
    }
    m.boite((5, 5, 4), (6, 6, 23), FerSombre);
    m.ellipsoide((6.0, 6.0, 10.5), (2.0, 2.0, 1.5), Fer);
    m.ellipsoide((6.0, 6.0, 17.5), (1.8, 1.8, 1.3), Fer);
    m.boite((0, 5, 22), (10, 6, 23), FerSombre);
    m.boite((5, 0, 22), (6, 10, 23), FerSombre);
    for (i, (cx, cy)) in [(0, 5), (9, 5), (5, 0), (5, 9), (5, 5)].into_iter().enumerate() {
        m.boite((cx - 1, cy - 1, 24), (cx + 2, cy + 2, 24), Fer);
        let haut = 25 + if i == 4 { 5 } else { 3 + (bruit(cx, cy, 7) % 2) as i32 };
        m.boite((cx, cy, 25), (cx + 1, cy + 1, haut), Cire);
        // Une coulure de cire sur le côté.
        m.boite((cx + 2, cy, haut - 2), (cx + 2, cy, haut - 1), Cire);
        m.boite((cx, cy, haut + 1), (cx + 1, cy + 1, haut + 1), Flamme);
        m.boite((cx, cy, haut + 2), (cx + 1, cy, haut + 2), Flamme);
        m.mettre((cx + 1, cy + 1, haut + 1), Braise);
    }
    m
}
