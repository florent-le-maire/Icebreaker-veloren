//! Icebreaker U.23 : les modèles des filons de la grotte (`docs/icebreaker/design/filons.md`). Chaque bloc d'un
//! filon est un sprite : un bloc de gangue bosselé qui porte son minerai (pépites brillantes pour les métaux, blocs
//! à facettes pour le charbon, cristaux qui luisent pour les gemmes). Trois variantes par minerai ; le jeu en tire
//! une par position et la tourne par quarts de tour, donc le modèle n'a pas d'avant.
//!
//! Le modèle fait 17 voxels de côté : les 11 du bloc et 3 de marge de chaque côté pour ce qui dépasse. L'ancre est
//! le centre du bas du bloc. La teinte vient de la couleur du minerai (la même que `couleur` dans
//! `assets/icebreaker/grottes/minerais.ron`).

use crate::{Axe, Matiere, Modele, bruit};

const MARGE: i32 = 3;
const BLOC: i32 = 11;
const TAILLE: i32 = BLOC + 2 * MARGE;

#[derive(Clone, Copy, PartialEq)]
enum Genre {
    /// De la gangue et des pépites brillantes qui ressortent.
    Metal,
    /// Des blocs noirs à facettes.
    Charbon,
    /// De la gangue d'où sortent des cristaux qui luisent.
    Gemme,
}

/// Les minerais : les noms de fichier de leurs trois variantes, leur couleur, leur genre.
const FILONS: [([&str; 3], [u8; 3], Genre); 11] = [
    (["filon_cuivre_1", "filon_cuivre_2", "filon_cuivre_3"], [196, 110, 52], Genre::Metal),
    (["filon_etain_1", "filon_etain_2", "filon_etain_3"], [168, 186, 160], Genre::Metal),
    (["filon_fer_1", "filon_fer_2", "filon_fer_3"], [150, 70, 50], Genre::Metal),
    (["filon_charbon_1", "filon_charbon_2", "filon_charbon_3"], [44, 44, 50], Genre::Charbon),
    (["filon_cobalt_1", "filon_cobalt_2", "filon_cobalt_3"], [40, 80, 200], Genre::Metal),
    (["filon_argent_1", "filon_argent_2", "filon_argent_3"], [225, 228, 238], Genre::Metal),
    (["filon_or_1", "filon_or_2", "filon_or_3"], [240, 190, 40], Genre::Metal),
    (["filon_amethyste_1", "filon_amethyste_2", "filon_amethyste_3"], [165, 80, 225], Genre::Gemme),
    (["filon_rubis_1", "filon_rubis_2", "filon_rubis_3"], [220, 30, 50], Genre::Gemme),
    (["filon_saphir_1", "filon_saphir_2", "filon_saphir_3"], [40, 120, 255], Genre::Gemme),
    (["filon_emeraude_1", "filon_emeraude_2", "filon_emeraude_3"], [30, 200, 90], Genre::Gemme),
];

pub fn modeles() -> Vec<Modele> {
    FILONS
        .iter()
        .enumerate()
        .flat_map(|(i, (ids, couleur, genre))| {
            ids.iter().enumerate().map(move |(v, id)| filon(id, *couleur, *genre, (i * 3 + v) as i32 + 1))
        })
        .collect()
}

/// `a` tiré vers `b` de la part `t`.
fn vers(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [0, 1, 2].map(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8)
}

/// Un bruit lisse entre 0 et 1 (valeurs tirées aux coins d'une grille, fondues entre eux).
fn lisse(x: f32, y: f32, z: f32, graine: i32) -> f32 {
    let (x0, y0, z0) = (x.floor(), y.floor(), z.floor());
    let fondu = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy, fz) = (fondu(x - x0), fondu(y - y0), fondu(z - z0));
    let coin = |dx: i32, dy: i32, dz: i32| {
        bruit(x0 as i32 + dx + graine * 31, y0 as i32 + dy - graine * 17, z0 as i32 + dz + graine * 7) as f32
            / u32::MAX as f32
    };
    let mele = |a: f32, b: f32, t: f32| a + (b - a) * t;
    mele(
        mele(mele(coin(0, 0, 0), coin(1, 0, 0), fx), mele(coin(0, 1, 0), coin(1, 1, 0), fx), fy),
        mele(mele(coin(0, 0, 1), coin(1, 0, 1), fx), mele(coin(0, 1, 1), coin(1, 1, 1), fx), fy),
        fz,
    )
}

fn filon(id: &'static str, couleur: [u8; 3], genre: Genre, graine: i32) -> Modele {
    let c = TAILLE as f32 / 2.0;
    let mut m = Modele::new(id, (TAILLE, TAILLE, TAILLE), (c, c, MARGE as f32));
    let blanc = [255, 255, 255];
    let noir = [0, 0, 0];
    // Le minerai : trois tons brillants et un éclat ; les gemmes luisent de la même teinte.
    m.teinter(8, vers(couleur, blanc, 0.22));
    m.teinter(9, couleur);
    m.teinter(10, vers(couleur, noir, 0.3));
    m.teinter(11, vers(couleur, blanc, 0.55));
    m.teinter(13, couleur);
    m.teinter(14, vers(couleur, blanc, 0.3));
    m.teinter(15, vers(couleur, blanc, 0.6));
    if genre == Genre::Charbon {
        // Le charbon : des tons serrés, de rares reflets.
        m.teinter(8, [60, 60, 68]);
        m.teinter(10, [31, 31, 36]);
        m.teinter(11, [92, 94, 106]);
    }
    let rare = if genre == Genre::Charbon { 24 } else { 12 };
    // La gangue prend un peu de la teinte de son minerai.
    for i in 87..=89 {
        m.teinter(i, vers(crate::couleur(i), couleur, 0.14));
    }

    // La masse : un cube aux coins arrondis dont la surface se creuse et se bombe ; le minerai y fait des amas.
    let (ampleur, pas, part) = match genre {
        Genre::Metal => (2.6, 4.0, 0.5),
        Genre::Charbon => (3.4, 1.0, 0.0),
        Genre::Gemme => (2.0, 4.0, 0.72),
    };
    for x in 0..TAILLE {
        for y in 0..TAILLE {
            for z in 0..TAILLE {
                let (dx, dy, dz) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c, z as f32 + 0.5 - c);
                let distance = (dx.abs().powi(6) + dy.abs().powi(6) + dz.abs().powi(6)).powf(1.0 / 6.0)
                    - BLOC as f32 / 2.0;
                let (relief, amas) = if genre == Genre::Charbon {
                    // Des blocs de 3 voxels de côté, chacun à sa hauteur : des facettes.
                    let bloc = |v: i32| (v + graine).div_euclid(3);
                    let (bx, by, bz) = (bloc(x), bloc(y), bloc(z));
                    ((bruit(bx, by, bz + graine) % 100) as f32 / 100.0, bruit(bx + 50, by, bz) % 9 != 0)
                } else {
                    (
                        lisse(x as f32 / pas, y as f32 / pas, z as f32 / pas, graine),
                        lisse(x as f32 / 2.6, y as f32 / 2.6, z as f32 / 2.6, graine + 100) > part,
                    )
                };
                // Un amas de métal ressort de la gangue.
                let bosse = if amas && genre == Genre::Metal { 0.9 } else { 0.0 };
                if distance < (relief - 0.42) * ampleur + bosse {
                    let matiere = match (amas, bruit(x + graine, y, z) % rare == 0) {
                        (true, true) => Matiere::Eclat,
                        (true, false) => Matiere::Minerai,
                        (false, _) => Matiere::Gangue,
                    };
                    m.mettre((x, y, z), matiere);
                }
            }
        }
    }
    if genre == Genre::Gemme {
        cristaux(&mut m, graine);
    }
    m
}

/// Des cristaux droits qui sortent de chaque face du bloc : un fût brillant, une arête et une pointe qui luisent.
fn cristaux(m: &mut Modele, graine: i32) {
    for face in 0..6 {
        let axe = [Axe::X, Axe::Y, Axe::Z][face as usize / 2];
        let dehors = face % 2 == 1;
        let tire = |n: i32, k: i32| bruit(graine * 13 + face, n, k) as i32 & 0x7fff;
        for n in 0..3 + tire(9, 0) % 2 {
            let large = 2 + tire(n, 1) % 2;
            let long = large;
            let (u0, v0) = (MARGE + 1 + tire(n, 3) % (BLOC - large - 1), MARGE + 1 + tire(n, 4) % (BLOC - large - 1));
            for i in -2..=long {
                let t = if dehors { MARGE + BLOC - 1 + i } else { MARGE - i };
                for du in 0..large {
                    for dv in 0..large {
                        let milieu = (du, dv) == (large / 2, large / 2);
                        // La pointe : le dernier rang ne garde que le milieu.
                        if i == long && !milieu {
                            continue;
                        }
                        let matiere = if i == long {
                            Matiere::GemmeCoeur
                        } else if milieu {
                            Matiere::Gemme
                        } else {
                            Matiere::Minerai
                        };
                        m.mettre(axe.point(t, u0 + du, v0 + dv), matiere);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trois_variantes_par_minerai_qui_tiennent_dans_un_sprite() {
        let modeles = modeles();
        assert_eq!(modeles.len(), 33);
        for m in &modeles {
            // Au-delà de 31 voxels de large, le client rétrécit le sprite.
            assert!(m.taille.0 <= 31 && m.taille.1 <= 31 && m.taille.2 <= 63, "{}", m.id);
            assert_eq!(m.offset(), (-8.5, -8.5, -3.0), "{}", m.id);
        }
    }

    /// Le bloc est presque plein : on ne voit pas à travers un filon.
    #[test]
    fn le_bloc_est_presque_plein_et_porte_du_minerai() {
        for m in modeles() {
            let dans_le_bloc = |(x, y, z): crate::P| [x, y, z].iter().all(|v| (MARGE..MARGE + BLOC).contains(v));
            let pleins = m.voxels().filter(|(p, _)| dans_le_bloc(*p)).count();
            assert!(pleins * 100 >= (BLOC * BLOC * BLOC) as usize * 80, "{} : {pleins} voxels dans le bloc", m.id);
            let minerai = m.voxels().filter(|(_, i)| (8..16).contains(i)).count();
            assert!(minerai * 6 >= m.nombre_de_voxels(), "{} : {minerai} voxels de minerai", m.id);
        }
    }

    #[test]
    fn deux_variantes_ne_sont_pas_le_meme_modele() {
        let modeles = modeles();
        for paire in modeles.chunks(3) {
            assert_ne!(paire[0].vox(), paire[1].vox(), "{}", paire[0].id);
            assert_ne!(paire[1].vox(), paire[2].vox(), "{}", paire[1].id);
        }
    }
}
