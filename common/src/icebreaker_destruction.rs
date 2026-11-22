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

/// Ce qu'il en coûte de casser un bloc d'une matière.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Matiere {
    /// Coups de pioche pour que le bloc cède.
    pub coups: u8,
    /// Énergie payée à chaque coup.
    pub energie: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct Fichier {
    /// Par nom de `BlockKind` (`Rock`, `Wood`…).
    matieres: HashMap<String, Matiere>,
    /// Toute autre matière pleine.
    defaut: Matiere,
}

#[derive(Clone, Debug)]
pub struct Destruction {
    matieres: HashMap<BlockKind, Matiere>,
    defaut: Matiere,
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
        Ok(Self { matieres, defaut: fichier.defaut })
    }

    pub fn matiere(&self, kind: BlockKind) -> Matiere { self.matieres.get(&kind).copied().unwrap_or(self.defaut) }
}

/// Les réglages du fichier ; s'il est illisible, tout prend 5 coups à 15 d'énergie.
pub fn destruction() -> &'static Destruction {
    static DESTRUCTION: LazyLock<Destruction> = LazyLock::new(|| {
        Ron::<Fichier>::load("icebreaker.destruction")
            .map_err(|e| format!("{e:?}"))
            .and_then(|f| Destruction::depuis(f.read().0.clone()))
            .unwrap_or_else(|e| {
                tracing::error!("Icebreaker : assets/icebreaker/destruction.ron illisible ({e})");
                Destruction { matieres: HashMap::new(), defaut: Matiere { coups: 5, energie: 15.0 } }
            })
    });
    &DESTRUCTION
}

/// Un bloc que la pioche creuse matière comprise : plein, donc sans sprite (les minerais et les
/// meubles gardent les règles de Veloren).
pub fn se_creuse(block: &Block) -> bool { block.kind().is_filled() }

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
}
