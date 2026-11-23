//! Icebreaker : tout se casse à la pioche (U.16, `assets/icebreaker/destruction.ron`,
//! `docs/icebreaker/design/destruction.md`). Dans une Grotte ou un donjon généré, chaque bloc plein
//! cède après un nombre de coups de pioche qui dépend de sa matière ; chaque coup coûte de
//! l'énergie, et sans énergie le coup ne fait rien. Le serveur décide où cela vaut
//! (`world::icebreaker::se_creuse_ici`) et compte les coups.

use crate::{
    assets::{AssetExt, Ron},
    terrain::{Block, BlockKind},
};
use hashbrown::HashMap;
use serde::Deserialize;
use std::{str::FromStr, sync::LazyLock};
use vek::{Vec2, Vec3};

/// Ce qu'il en coûte de casser un bloc d'une matière.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Matiere {
    /// Coups de pioche pour que le bloc cède.
    pub coups: u8,
    /// Énergie payée à chaque coup.
    pub energie: f32,
}

/// U.17 : l'éclat qui part quand un bloc cède, un cône qui part du corps de qui creuse (un peu plus
/// large que lui, haut comme lui) et se referme devant lui, vers le bloc visé.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Cone {
    /// Demi-largeur au corps de qui creuse, en blocs (0 : seul le bloc visé cède).
    pub largeur: f32,
    /// Demi-hauteur au corps, en blocs ; sous 1,5 le sol sous les pieds et la voûte restent.
    pub hauteur: f32,
    /// Distance de la pointe devant qui creuse, en blocs.
    pub profondeur: f32,
}

const CONE_DEFAUT: Cone = Cone { largeur: 1.9, hauteur: 1.4, profondeur: 5.0 };

/// Le cône commence un demi-bloc derrière le milieu du corps : les blocs à côté de qui creuse
/// partent aussi, et le boyau garde sa largeur quand on avance.
const CONE_DEBUT: f32 = -0.5;

#[derive(Clone, Debug, Deserialize)]
struct Fichier {
    /// Par nom de `BlockKind` (`Rock`, `Wood`…).
    matieres: HashMap<String, Matiere>,
    /// Toute autre matière pleine.
    defaut: Matiere,
    cone: Cone,
}

#[derive(Clone, Debug)]
pub struct Destruction {
    matieres: HashMap<BlockKind, Matiere>,
    defaut: Matiere,
    cone: Cone,
}

impl Destruction {
    fn depuis(fichier: Fichier) -> Result<Self, String> {
        let mut matieres = HashMap::new();
        for (nom, matiere) in fichier.matieres {
            let kind = BlockKind::from_str(&nom).map_err(|_| format!("matière inconnue : {nom}"))?;
            if !kind.is_filled() {
                return Err(format!("{nom} n'est pas une matière pleine"));
            }
            matieres.insert(kind, matiere);
        }
        for matiere in matieres.values().chain([&fichier.defaut]) {
            if matiere.coups == 0 || matiere.energie < 0.0 {
                return Err("il faut au moins un coup, et une énergie positive ou nulle".into());
            }
        }
        if !(fichier.cone.largeur >= 0.0 && fichier.cone.hauteur >= 0.0 && fichier.cone.profondeur > 0.0) {
            return Err("cône : largeur et hauteur positives ou nulles, profondeur positive".into());
        }
        Ok(Self { matieres, defaut: fichier.defaut, cone: fichier.cone })
    }

    pub fn matiere(&self, kind: BlockKind) -> Matiere { self.matieres.get(&kind).copied().unwrap_or(self.defaut) }

    /// U.20 : la matière d'un bloc ; un bloc de minerai (`icebreaker_minerais`) a la dureté de son minerai.
    pub fn matiere_de(&self, block: &Block) -> Matiere {
        crate::icebreaker_minerais::minerais()
            .de_bloc(block)
            .map_or_else(|| self.matiere(block.kind()), |(_, m)| m.matiere())
    }

    pub fn cone(&self) -> Cone { self.cone }

    /// U.17 : un bloc pris dans le cône part avec le bloc visé s'il n'est pas plus dur que lui.
    pub fn part_avec(&self, vise: BlockKind, autre: &Block) -> bool {
        let (a, v) = (self.matiere(autre.kind()), self.matiere(vise));
        // Ni plus de coups, ni plus d'énergie par coup : le bois n'emporte pas la pierre.
        // U.20 : jamais un bloc de minerai : on le dégage morceau par morceau.
        se_creuse(autre)
            && a.coups <= v.coups
            && a.energie <= v.energie
            && crate::icebreaker_minerais::minerais().de_bloc(autre).is_none()
    }
}

/// Les réglages du fichier ; s'il est illisible, tout prend 3 coups à 15 d'énergie.
pub fn destruction() -> &'static Destruction {
    static DESTRUCTION: LazyLock<Destruction> = LazyLock::new(|| {
        Ron::<Fichier>::load("icebreaker.destruction")
            .map_err(|e| format!("{e:?}"))
            .and_then(|f| Destruction::depuis(f.read().0.clone()))
            .unwrap_or_else(|e| {
                tracing::error!("Icebreaker : assets/icebreaker/destruction.ron illisible ({e})");
                Destruction {
                    matieres: HashMap::new(),
                    defaut: Matiere { coups: 3, energie: 15.0 },
                    cone: CONE_DEFAUT,
                }
            })
    });
    &DESTRUCTION
}

/// Un bloc que la pioche creuse matière comprise : plein, donc sans sprite (les minerais et les
/// meubles gardent les règles de Veloren).
pub fn se_creuse(block: &Block) -> bool { block.kind().is_filled() }

/// U.20 : le morceau qui part quand un bloc de minerai cède : le bloc visé (en premier) et les blocs
/// à `rayon` de lui ; l'appelant garde ceux du même minerai.
pub fn blocs_du_morceau(vise: Vec3<i32>, rayon: f32) -> Vec<Vec3<i32>> {
    let mut blocs = vec![vise];
    let r = rayon.max(0.0).floor() as i32;
    for x in -r..=r {
        for y in -r..=r {
            for z in -r..=r {
                if (x, y, z) != (0, 0, 0) && ((x * x + y * y + z * z) as f32).sqrt() <= rayon + 0.01 {
                    blocs.push(vise + Vec3::new(x, y, z));
                }
            }
        }
    }
    blocs
}

/// Ce que fait un coup de pioche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coup {
    /// Pas assez d'énergie : rien ne se passe, rien n'est payé.
    ASec,
    /// Le bloc est entamé ; il a reçu ce nombre de coups.
    Entame(u8),
    /// Le bloc cède.
    Casse,
}

/// Un coup sur un bloc de `matiere` qui a déjà reçu `deja` coups, par quelqu'un qui a `energie`.
/// Hors `ASec`, l'appelant retire `matiere.energie`.
pub fn coup(matiere: Matiere, energie: f32, deja: u8) -> Coup {
    if energie < matiere.energie {
        Coup::ASec
    } else if deja.saturating_add(1) >= matiere.coups {
        Coup::Casse
    } else {
        Coup::Entame(deja + 1)
    }
}

/// U.17 : les blocs du cône qui part quand `vise` cède, pour qui creuse et a le milieu du corps en
/// `origine`. Le cône part de là (`cone.largeur` de chaque côté, `cone.hauteur` au-dessus et en
/// dessous) et se referme `cone.profondeur` blocs plus loin, dans l'axe du bloc visé ; le bloc visé
/// en fait toujours partie.
pub fn blocs_du_cone(vise: Vec3<i32>, origine: Option<Vec3<f32>>, cone: Cone) -> Vec<Vec3<i32>> {
    let mut blocs = vec![vise];
    let milieu = |b: Vec3<i32>| b.map(|e| e as f32 + 0.5);
    // Un bloc visé à hauteur du corps (pieds ou tête) : on creuse droit devant soi, à plat.
    let vers = |o: Vec3<f32>| {
        let v = milieu(vise) - o;
        if v.z.abs() < 1.0 { v.with_z(0.0) } else { v }
    };
    let Some((origine, axe)) = origine.and_then(|o| Some((o, vers(o).try_normalized()?))) else {
        return blocs;
    };
    if cone.largeur <= 0.0 || cone.hauteur <= 0.0 {
        return blocs;
    }
    // Le côté (à l'horizontale) et le haut du cône, en travers de l'axe ; à la verticale, le
    // côté est pris au hasard.
    let cote = axe.cross(Vec3::unit_z()).try_normalized().unwrap_or(Vec3::unit_x());
    let haut = cote.cross(axe);
    let portee = cone.largeur.max(cone.hauteur).max(cone.profondeur).ceil() as i32 + 1;
    let centre = origine.map(|e| e.floor() as i32);
    for x in -portee..=portee {
        for y in -portee..=portee {
            for z in -portee..=portee {
                let bloc = centre + Vec3::new(x, y, z);
                if bloc == vise {
                    continue;
                }
                let d = milieu(bloc) - origine;
                // Le long de l'axe (0 au corps) ; le cône se referme de 1 au corps à 0 à la pointe.
                let t = d.dot(axe);
                let ouverture = 1.0 - t.max(0.0) / cone.profondeur;
                let ecart = Vec2::new(d.dot(cote) / cone.largeur, d.dot(haut) / cone.hauteur).magnitude();
                if t > CONE_DEBUT && t < cone.profondeur && ecart <= ouverture + 0.01 {
                    blocs.push(bloc);
                }
            }
        }
    }
    blocs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fichier_de_donnees_lisible() {
        let fichier = Ron::<Fichier>::load_expect("icebreaker.destruction").read().0.clone();
        let d = Destruction::depuis(fichier).expect("destruction.ron");
        // Plus ou moins facile selon la matière : la roche dure demande plus que la tendre.
        let (dure, tendre) = (d.matiere(BlockKind::Rock), d.matiere(BlockKind::WeakRock));
        assert!(dure.coups > tendre.coups && dure.energie >= tendre.energie);
        // Une matière que le fichier ne nomme pas prend le défaut.
        assert_eq!(d.matiere(BlockKind::Misc), d.defaut);
    }

    #[test]
    fn un_bloc_cede_au_dernier_coup() {
        let roche = Matiere { coups: 3, energie: 10.0 };
        assert_eq!(coup(roche, 50.0, 0), Coup::Entame(1));
        assert_eq!(coup(roche, 50.0, 1), Coup::Entame(2));
        assert_eq!(coup(roche, 50.0, 2), Coup::Casse);
        // Une matière à un coup cède tout de suite.
        assert_eq!(coup(Matiere { coups: 1, energie: 5.0 }, 5.0, 0), Coup::Casse);
    }

    #[test]
    fn a_sec_on_ne_creuse_plus() {
        let roche = Matiere { coups: 3, energie: 10.0 };
        assert_eq!(coup(roche, 9.9, 2), Coup::ASec);
        assert_eq!(coup(roche, 10.0, 2), Coup::Casse);
    }

    /// Les pieds en z = 30 au milieu d'un bloc (milieu du corps un bloc plus haut), face à +x.
    const CORPS: Vec3<f32> = Vec3::new(10.5, 20.5, 31.0);

    #[test]
    fn le_cone_part_du_corps_et_se_referme_devant() {
        // On vise le mur à hauteur de tête.
        let blocs = blocs_du_cone(Vec3::new(12, 20, 31), Some(CORPS), CONE_DEFAUT);
        // Largeur du boyau à hauteur des pieds et de la tête, tranche par tranche.
        let largeur = |x: i32, z: i32| blocs.iter().filter(|b| b.x == x && b.z == z).count();
        // Trois blocs de large autour du corps et devant lui, puis ça se referme.
        for (x, large) in [(10, 3), (11, 3), (12, 1), (13, 1), (14, 0)] {
            assert_eq!((largeur(x, 30), largeur(x, 31)), (large, large), "x = {x}");
        }
        // Rien derrière soi.
        assert!(blocs.iter().all(|b| b.x >= 10));
    }

    #[test]
    fn on_avance_debout_sans_perdre_le_sol() {
        // Qu'on vise à hauteur de tête ou des pieds, le sol sous les pieds et devant reste.
        for vise in [Vec3::new(12, 20, 31), Vec3::new(12, 20, 30)] {
            let blocs = blocs_du_cone(vise, Some(CORPS), CONE_DEFAUT);
            for bloc in [(11, 20, 30), (11, 20, 31), (12, 20, 30), (12, 20, 31)] {
                assert!(blocs.contains(&Vec3::from(bloc)), "{bloc:?} pour {vise:?}");
            }
            assert!(blocs.iter().all(|b| b.z == 30 || b.z == 31), "le sol ou la voûte part pour {vise:?}");
        }
        // En visant le sol devant soi, on descend.
        assert!(blocs_du_cone(Vec3::new(11, 20, 29), Some(CORPS), CONE_DEFAUT).iter().any(|b| b.z < 29));
    }

    #[test]
    fn sans_largeur_ou_sans_creuseur_seul_le_bloc_vise_cede() {
        let vise = Vec3::new(1, 2, 3);
        let ici = Some(Vec3::new(1.5, 2.5, 1.0));
        assert_eq!(blocs_du_cone(vise, ici, Cone { largeur: 0.0, ..CONE_DEFAUT }), vec![vise]);
        assert_eq!(blocs_du_cone(vise, None, CONE_DEFAUT), vec![vise]);
        // Visé de loin, le bloc cède quand même.
        assert!(blocs_du_cone(Vec3::new(1, 2, 20), ici, CONE_DEFAUT).contains(&Vec3::new(1, 2, 20)));
        // Droit vers le bas, un puits.
        assert!(blocs_du_cone(Vec3::new(1, 2, -1), ici, CONE_DEFAUT).contains(&Vec3::new(1, 2, -2)));
    }

    #[test]
    fn le_cone_n_emporte_pas_plus_dur_que_le_bloc_vise() {
        use vek::Rgb;
        let fichier = Ron::<Fichier>::load_expect("icebreaker.destruction").read().0.clone();
        let d = Destruction::depuis(fichier).expect("destruction.ron");
        let (roche, tendre) =
            (Block::new(BlockKind::Rock, Rgb::broadcast(90)), Block::new(BlockKind::WeakRock, Rgb::broadcast(90)));
        assert!(d.part_avec(BlockKind::Rock, &tendre));
        assert!(d.part_avec(BlockKind::Rock, &roche));
        assert!(!d.part_avec(BlockKind::WeakRock, &roche));
        assert!(!d.part_avec(BlockKind::Wood, &roche));
        assert!(!d.part_avec(BlockKind::Rock, &Block::air(crate::terrain::SpriteKind::Iron)));
        assert!(d.cone().profondeur > 0.0);
    }

    #[test]
    fn seuls_les_blocs_pleins_se_creusent() {
        use crate::terrain::SpriteKind;
        use vek::Rgb;
        assert!(se_creuse(&Block::new(BlockKind::Rock, Rgb::broadcast(90))));
        assert!(se_creuse(&Block::new(BlockKind::Wood, Rgb::broadcast(90))));
        assert!(!se_creuse(&Block::air(SpriteKind::Empty)));
        assert!(!se_creuse(&Block::air(SpriteKind::Iron)));
        assert!(!se_creuse(&Block::new(BlockKind::Water, Rgb::zero())));
    }

    #[test]
    fn le_minerai_a_sa_durete_et_reste_dans_le_cone() {
        use vek::Rgb;
        let fichier = Ron::<Fichier>::load_expect("icebreaker.destruction").read().0.clone();
        let d = Destruction::depuis(fichier).expect("destruction.ron");
        let minerais = crate::icebreaker_minerais::minerais();
        let (_, or) = minerais.liste().iter().find(|(nom, _)| nom == "or").expect("de l'or");
        let roche = Block::new(BlockKind::Rock, Rgb::broadcast(90));
        assert_eq!(d.matiere_de(&or.bloc()), or.matiere());
        assert_eq!(d.matiere_de(&roche), d.matiere(BlockKind::Rock));
        // Le cône de la roche n'emporte pas le minerai, même tendre.
        for (_, m) in minerais.liste() {
            assert!(!d.part_avec(BlockKind::Rock, &m.bloc()));
        }
    }

    #[test]
    fn le_morceau_part_autour_du_bloc_vise() {
        let vise = Vec3::new(5, 5, 5);
        assert_eq!(blocs_du_morceau(vise, 0.0), vec![vise]);
        let un = blocs_du_morceau(vise, 1.0);
        assert_eq!(un.len(), 7);
        assert_eq!(un[0], vise);
        assert_eq!(blocs_du_morceau(vise, 1.5).len(), 19);
    }
}
