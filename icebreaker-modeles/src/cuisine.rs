//! Icebreaker F.25 : les modèles de la cuisine, lot 1 (`docs/icebreaker/PLAN-PLAYTEST.md`, « Cuisine vivante »).
//! La vie vient des traces d'activité (pain entamé, couperet planté, braises) et de ce qui pend en hauteur.
//!
//! 11 voxels font un bloc ; l'avant regarde -y. L'ancre est le point posé au centre du bas du bloc du sprite :
//! - posé au sol : le bloc devant, au milieu (âtre, billot) ;
//! - contre un mur : le bloc collé au mur, dos du modèle sur le mur (viande pendue) ;
//! - sous le plafond : le haut du modèle au haut du bloc (ustensiles pendus) ;
//! - sur un plan de travail : le bloc au-dessus du plan (vaisselle, préparation).

use crate::{Axe, Matiere::*, Modele, bruit};

pub fn modeles() -> Vec<Modele> {
    vec![atre(), billot(), viande_pendue(), ustensiles_pendus(), vaisselle(), preparation()]
}

/// L'âtre : un grand foyer de briques sous une hotte, cinq blocs de large, deux de profondeur, trois de haut. Une
/// voûte en arc, des bûches qui rougeoient, une marmite à la crémaillère, un manteau de bois chargé de pots.
pub fn atre() -> Modele {
    let mut m = Modele::new("atre", (55, 22, 32), (27.5, 5.5, 0.0));
    // Le corps, sa face avant en y = 3 (le manteau et la sole dépassent devant).
    m.boite((0, 3, 0), (54, 21, 19), Briques);
    // L'ouverture : droite jusqu'à z 11, puis en arc jusqu'à z 17 ; un rang de pierre autour de l'arc.
    let mut creux = Vec::new();
    for x in 13..=41 {
        let dx = (x as f32 - 27.0) / 14.5;
        let haut = 11 + ((1.0 - dx * dx).max(0.0).sqrt() * 6.0) as i32;
        for y in 3..=15 {
            for z in 1..=haut {
                creux.push((x, y, z));
            }
        }
        m.boite((x, 3, haut + 1), (x, 4, haut + 2), Pierre);
    }
    m.boite((11, 3, 1), (12, 4, 12), Pierre);
    m.boite((42, 3, 1), (43, 4, 12), Pierre);
    for p in &creux {
        m.enlever(*p);
    }
    // L'intérieur noirci par la fumée.
    for &(x, y, z) in &creux {
        for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, 0, 1)] {
            let q = (x + dx, y + dy, z + dz);
            if m.voxel(q).is_some() && q.1 > 4 {
                m.mettre(q, Suie);
            }
        }
    }
    // La sole de pierre, qui avance devant le foyer, et la cendre au fond.
    m.boite((9, 0, 0), (45, 15, 0), Pierre);
    for x in 14..=40 {
        for y in 6..=15 {
            if bruit(x, y, 1) % 3 == 0 {
                m.mettre((x, y, 1), Cendre);
            }
        }
    }
    // Les bûches, la plus haute calcinée.
    m.cylindre_axe(Axe::X, (11.0, 2.5), 1.9, 16, 38, BoisSombre);
    m.cylindre_axe(Axe::X, (7.5, 2.5), 1.7, 18, 36, BoisSombre);
    m.cylindre_axe(Axe::X, (9.3, 5.0), 1.6, 20, 34, Suie);
    // Les braises : sur le dessus des bûches et au pied du feu.
    for x in 15..=39 {
        for y in 4..=14 {
            for z in 1..=7 {
                let dessous = m.voxel((x, y, z - 1)).is_some();
                if m.voxel((x, y, z)).is_none() && dessous && bruit(x, y, z) % 3 == 0 {
                    m.mettre((x, y, z), Braise);
                }
            }
        }
    }
    // Les flammes : des langues de hauteur variable, le cœur clair, les bords rouges.
    for (i, x) in [18, 22, 25, 30, 33, 37].into_iter().enumerate() {
        let haut = 8 + (bruit(x, 3, i as i32) % 4) as i32 - i32::from((24..=31).contains(&x)) * 2;
        for z in 6..=haut {
            m.mettre((x, 9, z), Flamme);
            if z < haut - 1 {
                m.mettre((x + 1, 9, z), Braise);
                m.mettre((x, 10, z), Flamme);
            }
        }
    }
    // La marmite pendue à la crémaillère, au-dessus du feu.
    m.ligne((27, 10, 17), (27, 10, 14), Fer);
    m.ellipsoide((27.5, 10.5, 9.5), (5.0, 4.6, 4.0), FerSombre);
    m.creuser((21, 4, 13), (34, 16, 16));
    m.percer((27.5, 10.5), 3.8, 9, 13);
    m.cylindre((27.5, 10.5), 3.8, 10, 10, Jambon);
    m.mettre((26, 10, 11), Carotte);
    m.mettre((29, 11, 11), Chou);
    m.ligne((23, 10, 13), (27, 10, 15), Fer);
    m.ligne((32, 10, 13), (28, 10, 15), Fer);
    // Le manteau de bois sur deux corbeaux de pierre.
    m.boite((2, 0, 19), (52, 4, 21), BoisSombre);
    m.boite((4, 0, 14), (6, 2, 18), Pierre);
    m.boite((48, 0, 14), (50, 2, 18), Pierre);
    // La hotte, qui se resserre en montant.
    m.boite((5, 6, 22), (49, 21, 27), Briques);
    m.boite((11, 9, 28), (43, 21, 31), Briques);
    // Sur le manteau : des pots, une cruche, une assiette d'étain posée de chant.
    m.cylindre((8.5, 2.5), 1.6, 22, 25, TerreCuite);
    m.cylindre((13.0, 2.0), 1.2, 22, 24, TerreCuite);
    m.cylindre((40.5, 2.5), 1.8, 22, 26, Faience);
    m.cylindre_axe(Axe::Y, (46.5, 25.0), 3.0, 3, 3, Fer);
    // Des bouquets d'herbes qui sèchent, pendus au manteau.
    for x in [17, 21, 33, 37] {
        m.ligne((x, 1, 18), (x, 1, 15 - (x % 3)), Herbe);
        m.mettre((x, 1, 18), Corde);
    }
    m
}

/// Billot de boucher : un gros bloc de bois sur pieds, cerné de taches, un couperet planté et un quartier de viande.
pub fn billot() -> Modele {
    let mut m = Modele::new("billot", (11, 11, 14), (5.5, 5.5, 0.0));
    for (x, y) in [(1, 1), (8, 1), (1, 8), (8, 8)] {
        m.boite((x, y, 0), (x + 1, y + 1, 4), BoisSombre);
    }
    m.boite((0, 0, 5), (10, 10, 9), BoisClair);
    // Le dessus en bois de bout : des cernes, et des taches de sang.
    for x in 0..=10 {
        for y in 0..=10 {
            let d = (((x as f32 - 5.0).powi(2) + (y as f32 - 5.0).powi(2)).sqrt()) as i32;
            if d % 2 == 1 {
                m.mettre((x, y, 9), BoisSombre);
            }
            if bruit(x, y, 9) % 11 == 0 {
                m.mettre((x, y, 9), Viande);
            }
        }
    }
    // Le couperet planté, lame dans le bois, manche en l'air.
    m.boite((2, 6, 8), (6, 6, 11), Fer);
    m.boite((2, 6, 12), (6, 6, 12), FerSombre);
    m.ligne((7, 6, 12), (9, 6, 13), BoisSombre);
    // Le quartier de viande, son gras et son os.
    m.ellipsoide((6.5, 3.0, 11.0), (2.8, 1.9, 1.6), Viande);
    m.ellipsoide((8.8, 3.0, 11.0), (0.9, 1.6, 1.3), Gras);
    m.ligne((4, 3, 11), (2, 2, 11), Os);
    m.mettre((1, 2, 11), Os);
    m
}

/// Viande pendue contre un mur : une barre sur deux pattes de fer, un jambon, une volaille, un chapelet de saucisses.
pub fn viande_pendue() -> Modele {
    let mut m = Modele::new("viande_pendue", (22, 7, 22), (5.5, 1.5, 0.0));
    m.boite((0, 4, 19), (21, 6, 20), BoisSombre);
    m.boite((1, 5, 17), (2, 6, 21), FerSombre);
    m.boite((19, 5, 17), (20, 6, 21), FerSombre);
    for x in [4, 11, 17] {
        m.ligne((x, 4, 18), (x, 3, 16), Fer);
    }
    // Le jambon : en poire, la couenne grasse devant, l'os en haut.
    m.ellipsoide((4.5, 3.0, 8.5), (3.2, 2.5, 4.5), Jambon);
    m.ellipsoide((4.5, 3.0, 12.5), (2.0, 1.8, 2.5), Jambon);
    for z in 5..=12 {
        for x in 2..=7 {
            if bruit(x, 0, z) % 3 == 0 && m.voxel((x, 1, z)).is_some() {
                m.mettre((x, 1, z), Gras);
            }
        }
    }
    m.boite((4, 3, 15), (5, 3, 15), Os);
    // La volaille, pattes en l'air, liée par une ficelle.
    m.ellipsoide((11.5, 3.0, 10.5), (2.7, 2.3, 3.3), Volaille);
    m.ligne((10, 3, 13), (10, 3, 15), Volaille);
    m.ligne((12, 3, 13), (12, 3, 15), Volaille);
    m.boite((10, 3, 16), (12, 3, 16), Corde);
    // Le chapelet de saucisses.
    m.ligne((17, 3, 16), (17, 3, 3), Corde);
    for (i, z) in [14, 11, 8, 5].into_iter().enumerate() {
        m.ellipsoide((17.5, 3.0, z as f32), (1.4, 1.3, 1.6), if i % 2 == 0 { Viande } else { Jambon });
    }
    m
}

/// Ustensiles pendus sous le plafond : une perche suspendue par deux cordes, une poêle de cuivre, une marmite,
/// une louche, des herbes, une tresse d'ail et un chapelet d'oignons.
pub fn ustensiles_pendus() -> Modele {
    let mut m = Modele::new("ustensiles_pendus", (22, 11, 16), (5.5, 5.5, 5.0));
    m.boite((0, 5, 10), (21, 6, 11), BoisSombre);
    m.ligne((2, 5, 12), (2, 5, 15), Corde);
    m.ligne((19, 5, 12), (19, 5, 15), Corde);
    // La poêle de cuivre, de chant, son manche vers le haut.
    m.cylindre_axe(Axe::Y, (3.5, 4.5), 3.0, 4, 4, Cuivre);
    m.cylindre_axe(Axe::Y, (3.5, 4.5), 2.2, 3, 3, FerSombre);
    m.ligne((3, 4, 8), (3, 4, 9), BoisSombre);
    m.mettre((3, 5, 9), Fer);
    // La marmite de fer et son anse.
    m.ellipsoide((9.5, 5.5, 4.0), (2.6, 2.6, 2.2), FerSombre);
    m.percer((9.5, 5.5), 1.6, 5, 6);
    m.ligne((7, 5, 6), (9, 5, 9), Fer);
    m.ligne((12, 5, 6), (10, 5, 9), Fer);
    // La louche.
    m.ligne((13, 6, 9), (13, 6, 3), Fer);
    m.ellipsoide((13.5, 6.5, 2.0), (1.4, 1.4, 1.0), Fer);
    // Un bouquet d'herbes, tête en bas.
    for k in 0..4 {
        m.ligne((15 + k % 2, 4 + k, 9), (15 + k % 2, 4 + k, 4 + k % 3), Herbe);
    }
    m.boite((15, 4, 9), (16, 7, 9), Corde);
    // La tresse d'ail.
    m.ligne((18, 6, 9), (18, 6, 1), Corde);
    for (i, z) in [8, 6, 4, 2].into_iter().enumerate() {
        m.ellipsoide((17.5 + (i % 2) as f32 * 2.0, 6.5, z as f32 + 0.5), (1.1, 1.1, 1.0), Ail);
    }
    // Derrière, un chapelet d'oignons.
    m.ligne((6, 8, 10), (6, 8, 3), Corde);
    for z in [8, 5, 2] {
        m.ellipsoide((6.5, 8.5, z as f32 + 0.5), (1.3, 1.3, 1.2), Oignon);
    }
    m
}

/// Vaisselle d'un repas pas fini : une assiette avec un croûton et un os, une écuelle de ragoût, une chope, un pichet,
/// une cuillère et des miettes.
pub fn vaisselle() -> Modele {
    let mut m = Modele::new("vaisselle", (11, 11, 7), (5.5, 5.5, 0.0));
    // L'assiette.
    m.cylindre((3.5, 3.5), 2.7, 0, 1, Faience);
    m.percer((3.5, 3.5), 1.9, 1, 1);
    m.boite((3, 3, 1), (4, 3, 1), Pain);
    m.ligne((2, 4, 1), (4, 2, 1), Os);
    // L'écuelle de ragoût.
    m.cylindre((8.0, 3.0), 2.2, 0, 2, TerreCuite);
    m.percer((8.0, 3.0), 1.3, 1, 2);
    m.boite((7, 2, 1), (8, 3, 1), Jambon);
    m.mettre((8, 3, 1), Carotte);
    // La chope cerclée de fer.
    m.cylindre((8.5, 8.0), 1.6, 0, 4, BoisSombre);
    m.cylindre((8.5, 8.0), 1.7, 1, 1, Fer);
    m.cylindre((8.5, 8.0), 1.7, 3, 3, Fer);
    m.percer((8.5, 8.0), 0.9, 3, 4);
    m.boite((10, 8, 1), (10, 8, 3), BoisSombre);
    // Le pichet, plus haut, avec son bec et son anse.
    m.cylindre((3.0, 8.0), 1.9, 0, 4, TerreCuite);
    m.cylindre((3.0, 8.0), 1.2, 5, 6, TerreCuite);
    m.percer((3.0, 8.0), 0.6, 6, 6);
    m.mettre((1, 8, 6), TerreCuite);
    m.boite((5, 8, 2), (5, 8, 5), TerreCuite);
    // La cuillère et les miettes.
    m.ligne((5, 5, 0), (7, 6, 0), Fer);
    for (x, y) in [(6, 1), (1, 6), (6, 9), (10, 5)] {
        m.mettre((x, y, 0), Pain);
    }
    m
}

/// Préparation en cours sur un plan de travail : une planche, un pain entamé et ses tranches, un couteau, des
/// carottes coupées, un demi-chou, un oignon et de la farine renversée.
pub fn preparation() -> Modele {
    let mut m = Modele::new("preparation", (11, 11, 5), (5.5, 5.5, 0.0));
    // Le pain, coupé net : la mie se voit sur la tranche.
    m.ellipsoide((4.0, 4.5, 1.0), (3.2, 2.1, 2.4), Pain);
    m.creuser((6, 0, 0), (10, 10, 4));
    for y in 0..=10 {
        for z in 0..=4 {
            if m.voxel((5, y, z)).is_some() {
                m.mettre((5, y, z), Mie);
            }
        }
    }
    m.boite((1, 2, 0), (8, 7, 0), BoisClair);
    // Deux tranches couchées.
    m.boite((6, 3, 1), (7, 5, 1), Mie);
    m.boite((6, 3, 1), (6, 5, 1), Pain);
    // Le couteau au bord de la planche.
    m.ligne((1, 7, 1), (4, 7, 1), Fer);
    m.ligne((5, 7, 1), (7, 7, 1), BoisSombre);
    // Les carottes : une entière, des rondelles.
    m.ligne((9, 5, 0), (10, 9, 0), Carotte);
    m.mettre((10, 10, 0), Herbe);
    for (x, y) in [(9, 2), (10, 2), (9, 3), (8, 1)] {
        m.mettre((x, y, 0), Carotte);
    }
    // Le demi-chou et l'oignon.
    m.ellipsoide((2.5, 9.0, 0.5), (1.9, 1.6, 1.6), Chou);
    m.ellipsoide((6.5, 9.5, 0.5), (1.1, 1.1, 1.0), Oignon);
    // La farine renversée devant la planche.
    for x in 0..=10 {
        for y in 0..=1 {
            if m.voxel((x, y, 0)).is_none() && bruit(x, y, 0) % 3 == 0 {
                m.mettre((x, y, 0), Grain);
            }
        }
    }
    m
}
