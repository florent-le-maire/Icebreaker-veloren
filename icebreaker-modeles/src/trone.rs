//! Icebreaker F.37 : les modèles de la salle du trône (`docs/icebreaker/PLAN-PLAYTEST.md`, « Salle du trône
//! royale »). Épurée : peu d'objets, chacun travaillé ; marbre clair, pourpre et or ; la vie vient de la lumière
//! (vitrail, bougies), de la hauteur (piliers jusqu'au plafond) et du tapis qui mène au trône.
//!
//! Repère et ancres comme `cuisine.rs`. L'estrade se fait de blocs `estrade` (1 bloc) précédés d'une rangée de
//! `marche` (un demi-bloc) ; au milieu de la rangée, `tapis_marche` porte le tapis du sol jusque sur l'estrade, où il
//! continue en `tapis` posés à z 1.

use crate::{Axe, Matiere, Matiere::*, Modele, bruit};

pub fn modeles() -> Vec<Modele> {
    vec![trone(), estrade(), marche(), tapis(), tapis_bout(), tapis_marche(), pilier(), vitrail(), chandelier(),
        banniere(), statue(), lustre(), rosace(), vitrail_haut(), pilier_haut(),
    ]
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

/// Un pilier jusqu'au plafond de la petite salle (six blocs).
pub fn pilier() -> Modele { pilier_de("pilier", 66, 7.5) }

/// Icebreaker F.41 : un pilier de la grande salle, jusqu'à la voûte du second étage (quatorze blocs), plus gros.
pub fn pilier_haut() -> Modele { pilier_de("pilier_haut", 154, 9.5) }

/// Un pilier de `h` voxels : base carrée et tore de marbre, fût de pierre rose de rayon `r` où monte une vigne
/// peinte, bague d'or, chapiteau et abaque. Le fût déborde peu du bloc d'ancre (la collision).
fn pilier_de(id: &'static str, h: i32, r: f32) -> Modele {
    let mut m = Modele::new(id, (33, 33, h), (16.5, 16.5, 0.0));
    let c = (16.5, 16.5);
    // Les carrés de la base et de l'abaque, de demi-côté `d` autour du centre.
    let carre = |m: &mut Modele, d: i32, z0: i32, z1: i32, mat| m.boite((16 - d, 16 - d, z0), (16 + d, 16 + d, z1), mat);
    let base = (r as i32 + 6).min(16);
    carre(&mut m, base, 0, 4, Marbre);
    carre(&mut m, base, 0, 0, MarbreSombre);
    carre(&mut m, base - 2, 5, 6, Marbre);
    m.cylindre(c, r + 3.0, 7, 8, Marbre);
    m.cylindre(c, r + 2.0, 9, 9, MarbreSombre);
    m.cylindre(c, r, 10, h - 11, PierreRose);
    // La vigne : deux tiges en hélice sur la surface du fût, des feuilles qui partent de chaque côté.
    let tour = std::f32::consts::TAU * r;
    let demi = (tour / 2.0).round();
    for z in 12..=h - 16 {
        for x in 0..33 {
            for y in 0..33 {
                let (dx, dy) = (x as f32 + 0.5 - c.0, y as f32 + 0.5 - c.1);
                if m.voxel((x, y, z)).is_none() || dx * dx + dy * dy < (r - 1.3) * (r - 1.3) {
                    continue;
                }
                let u = dy.atan2(dx) / std::f32::consts::TAU * demi * 2.0;
                let s = (u - z as f32 * 0.9).rem_euclid(demi);
                let feuille = (s < 4.0 && z.rem_euclid(8) < 2) || (s > demi - 3.0 && (4..6).contains(&z.rem_euclid(8)));
                if s < 1.8 || s > demi - 1.4 || feuille {
                    m.mettre((x, y, z), Vigne);
                }
            }
        }
    }
    m.cylindre(c, r + 0.7, h - 14, h - 13, Or);
    m.cylindre(c, r + 2.0, h - 10, h - 10, MarbreSombre);
    m.cylindre(c, r + 3.0, h - 9, h - 7, Marbre);
    carre(&mut m, base - 2, h - 6, h - 5, Marbre);
    carre(&mut m, base, h - 4, h - 1, Marbre);
    carre(&mut m, base, h - 4, h - 4, MarbreSombre);
    m
}

/// Un vitrail contre le mur, quatre blocs de haut.
pub fn vitrail() -> Modele { vitrail_de("vitrail", 44, &[22.0]) }

/// Icebreaker F.41 : un grand vitrail de neuf blocs pour les murs de la grande salle, deux médaillons.
pub fn vitrail_haut() -> Modele { vitrail_de("vitrail_haut", 99, &[34.0, 66.0]) }

/// Un vitrail de `h` voxels : cadre de marbre en plein cintre, verres de couleur sertis de plomb en losanges, des
/// médaillons de feu (hauteurs données). Les verres chauds luisent (indices 13 à 15) ; la lueur du bloc éclaire.
fn vitrail_de(id: &'static str, h: i32, medaillons: &[f32]) -> Modele {
    let mut m = Modele::new(id, (33, 3, h), (16.5, -2.5, 0.0));
    let centre_arc = h - 16;
    let arc = |x: i32, z: i32, r: f32, bas: i32| {
        let (dx, dz) = (x as f32 + 0.5 - 16.5, (z - centre_arc) as f32 + 0.5);
        z >= bas && if z < centre_arc { dx.abs() <= r } else { dx * dx + dz * dz <= r * r }
    };
    for x in 0..33 {
        for z in 0..h {
            if arc(x, z, 11.5, 4) {
                let r = medaillons
                    .iter()
                    .map(|mz| {
                        let (dx, dz) = (x as f32 + 0.5 - 16.5, z as f32 + 0.5 - mz);
                        (dx * dx + dz * dz).sqrt()
                    })
                    .fold(f32::MAX, f32::min);
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

/// Icebreaker F.41 : une rosace au-dessus du trône, cinq blocs de large, contre le mur (posée à z 5). Douze pétales
/// sertis de plomb, un cœur de feu, une couronne de petits verres, un cadre de marbre ; une corniche dessous.
pub fn rosace() -> Modele {
    let mut m = Modele::new("rosace", (55, 3, 66), (27.5, -2.5, 0.0));
    let (cx, cz) = (27.5, 38.0);
    for x in 0..55 {
        for z in 0..66 {
            let (dx, dz) = (x as f32 + 0.5 - cx, z as f32 + 0.5 - cz);
            let r = (dx * dx + dz * dz).sqrt();
            let a = dz.atan2(dx).rem_euclid(std::f32::consts::TAU);
            // La distance (en voxels) au rayon de plomb le plus proche, pour `n` secteurs.
            let rayon_plomb = |n: f32| {
                let s = a / std::f32::consts::TAU * n;
                (s - s.round()).abs() * std::f32::consts::TAU / n * r
            };
            let secteur = |n: f32| (a / std::f32::consts::TAU * n) as i32;
            if r < 23.0 {
                let verre = if r < 3.5 {
                    Flamme
                } else if r < 5.0 || (12.6..13.8).contains(&r) || (19.6..20.8).contains(&r) {
                    FerSombre
                } else if r < 12.6 {
                    if rayon_plomb(12.0) < 0.7 { FerSombre } else if secteur(12.0) % 2 == 0 { Flamme } else { VerreVert }
                } else if r < 19.6 {
                    if rayon_plomb(12.0) < 0.7 { FerSombre } else if secteur(12.0) % 2 == 0 { Braise } else { VerreBleu }
                } else if rayon_plomb(24.0) < 0.7 {
                    FerSombre
                } else if secteur(24.0) % 3 == 0 {
                    Braise
                } else {
                    VerreBleu
                };
                m.mettre((x, 1, z), verre);
            } else if r < 27.4 {
                m.boite((x, 0, z), (x, 2, z), if r < 24.2 { MarbreSombre } else { Marbre });
            }
        }
    }
    m.boite((6, 0, 0), (48, 2, 2), MarbreSombre);
    m.boite((8, 0, 3), (46, 2, 4), Marbre);
    m
}

/// Icebreaker F.41 : une bannière contre le mur, haut placée (posée à z 5) : une hampe d'or sur deux pattes de fer,
/// un long drap pourpre en pointe, bordé d'or, un soleil d'or au milieu, un gland à la pointe.
pub fn banniere() -> Modele {
    let mut m = Modele::new("banniere", (33, 4, 77), (16.5, -1.5, 0.0));
    // Les pattes scellées dans le mur et la hampe, boules d'or aux bouts.
    for x in [5, 27] {
        m.boite((x, 1, 73), (x + 1, 3, 74), FerSombre);
    }
    m.boite((2, 1, 73), (30, 1, 74), Or);
    m.ellipsoide((1.5, 1.5, 74.0), (1.6, 1.6, 1.6), Or);
    m.ellipsoide((31.5, 1.5, 74.0), (1.6, 1.6, 1.6), Or);
    // Le drap, en pointe vers le bas, bordé d'or.
    for x in 4..=28i32 {
        let d = (x - 16).abs();
        let bas = 4 + (d as f32 * 0.9) as i32;
        for z in bas..=72 {
            let bord = x <= 5 || x >= 27 || z <= bas + 1;
            m.mettre((x, 2, z), if bord { Or } else { Pourpre });
        }
    }
    // Deux bandes sombres en haut et le soleil.
    m.boite((6, 2, 66), (26, 2, 67), PourpreSombre);
    m.cylindre_axe(Axe::Y, (16.5, 44.5), 7.0, 2, 2, Or);
    m.cylindre_axe(Axe::Y, (16.5, 44.5), 5.2, 2, 2, PourpreSombre);
    m.cylindre_axe(Axe::Y, (16.5, 44.5), 3.0, 2, 2, Or);
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::TAU / 8.0;
        let p = |r: f32| ((16.5 + r * a.cos()) as i32, (44.5 + r * a.sin()) as i32);
        let ((x0, z0), (x1, z1)) = (p(8.0), p(10.5));
        m.ligne((x0, 2, z0), (x1, 2, z1), Or);
    }
    // Le gland, à la pointe.
    m.boite((16, 2, 1), (17, 2, 3), Or);
    m.boite((15, 2, 0), (18, 2, 0), Or);
    m
}

/// Icebreaker F.41 : la statue d'un roi de pierre sur son socle, six blocs : robe, manteau, couronne d'or, les mains
/// sur le pommeau d'une épée de fer plantée devant lui. Une plaque d'or au socle.
pub fn statue() -> Modele {
    let mut m = Modele::new("statue", (33, 33, 72), (16.5, 16.5, 0.0));
    // Le socle.
    m.boite((6, 6, 0), (26, 26, 1), MarbreSombre);
    m.boite((7, 7, 2), (25, 25, 10), Marbre);
    m.boite((6, 6, 11), (26, 26, 12), MarbreSombre);
    m.boite((11, 6, 4), (21, 6, 8), MarbreSombre);
    m.boite((12, 6, 5), (20, 6, 7), Or);
    let (cx, cy) = (16.5, 17.5);
    // Le manteau, derrière, qui s'évase vers le bas.
    for z in 13..=53 {
        let demi = 6.5 + (53 - z) as f32 * 0.06;
        m.boite(((cx - demi) as i32, 20, z), ((cx + demi) as i32, 23, z), Pierre);
    }
    // La robe : des ellipses qui se resserrent jusqu'à la taille.
    for z in 13..=42 {
        let t = (z - 13) as f32 / 29.0;
        let (rx, ry) = (7.5 - 2.5 * t, 6.0 - 2.0 * t);
        for x in 0..33 {
            for y in 0..33 {
                let (dx, dy) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
                if dx * dx + dy * dy <= 1.0 {
                    m.mettre((x, y, z), Pierre);
                }
            }
        }
    }
    m.ellipsoide((cx, cy, 48.0), (5.5, 3.8, 7.0), Pierre);
    m.ellipsoide((cx, cy + 1.0, 53.0), (7.5, 4.5, 3.0), Pierre);
    m.boite((15, 16, 54), (18, 19, 56), Pierre);
    m.ellipsoide((cx, cy - 0.5, 60.0), (3.3, 3.3, 3.9), Pierre);
    // La couronne : un cercle d'or et quatre pointes.
    m.cylindre((cx, cy - 0.5), 3.7, 63, 64, Or);
    m.percer((cx, cy - 0.5), 2.4, 63, 64);
    for (x, y) in [(13, 16), (19, 16), (13, 18), (19, 18)] {
        m.boite((x, y, 65), (x, y, 66), Or);
    }
    // L'épée plantée devant, la garde et le pommeau.
    m.boite((16, 8, 14), (17, 9, 36), Fer);
    m.boite((16, 8, 13), (17, 8, 13), Fer);
    m.boite((12, 8, 37), (21, 9, 37), Or);
    m.boite((16, 8, 38), (17, 9, 41), BoisSombre);
    m.ellipsoide((16.5, 8.5, 42.5), (1.4, 1.4, 1.2), Or);
    // Les bras, des épaules aux mains posées sur le pommeau.
    for (epaule, main) in [((10.5, 18.0, 52.0), (14.0, 10.0, 41.0)), ((22.5, 18.0, 52.0), (19.0, 10.0, 41.0))] {
        for i in 0..=12 {
            let t = i as f32 / 12.0;
            let p = (
                epaule.0 + (main.0 - epaule.0) * t,
                epaule.1 + (main.1 - epaule.1) * t,
                epaule.2 + (main.2 - epaule.2) * t,
            );
            m.ellipsoide(p, (1.9, 1.9, 1.9), Pierre);
        }
    }
    m.ellipsoide((16.5, 9.5, 41.0), (3.2, 1.8, 1.8), Pierre);
    m
}

/// Icebreaker F.41 : un lustre pendu à la voûte (posé à z 5, sa chaîne monte de neuf blocs) : une roue de fer à huit
/// bougies, quatre chaînes qui se rejoignent, une longue chaîne jusqu'au plafond.
pub fn lustre() -> Modele {
    let mut m = Modele::new("lustre", (33, 33, 99), (16.5, 16.5, 0.0));
    let c = (16.5, 16.5);
    m.cylindre(c, 14.5, 3, 4, FerSombre);
    m.percer(c, 12.6, 3, 4);
    m.ellipsoide((16.5, 16.5, 4.0), (2.5, 2.5, 2.0), Fer);
    m.ellipsoide((16.5, 16.5, 1.0), (1.3, 1.3, 1.3), Or);
    for k in 0..4 {
        let a = k as f32 * std::f32::consts::TAU / 4.0 + std::f32::consts::FRAC_PI_4;
        let (x, y) = ((16.5 + 13.5 * a.cos()) as i32, (16.5 + 13.5 * a.sin()) as i32);
        m.ligne((16, 16, 4), (x, y, 4), FerSombre);
        m.ligne((16, 17, 4), (x, y, 3), FerSombre);
        // Les chaînes vers le haut, doublées pour qu'elles se voient.
        m.ligne((x, y, 5), (16, 16, 24), FerSombre);
        m.ligne((x, y, 6), (17, 17, 24), FerSombre);
    }
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::TAU / 8.0;
        let (x, y) = ((16.0 + 13.5 * a.cos()).round() as i32, (16.0 + 13.5 * a.sin()).round() as i32);
        m.boite((x - 1, y - 1, 5), (x + 1, y + 1, 5), Fer);
        let haut = 9 + (bruit(x, y, 3) % 2) as i32;
        m.boite((x, y, 6), (x + 1, y + 1, haut), Cire);
        m.boite((x, y, haut + 1), (x + 1, y + 1, haut + 1), Flamme);
        m.boite((x, y, haut + 2), (x + 1, y, haut + 2), Flamme);
        m.mettre((x + 1, y + 1, haut + 1), Braise);
    }
    // La chaîne : des maillons de deux voxels, croisés.
    for z in 24..99 {
        if (z / 2) % 2 == 0 {
            m.boite((16, 16, z), (17, 16, z), FerSombre);
        } else {
            m.boite((16, 16, z), (16, 17, z), FerSombre);
        }
    }
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
