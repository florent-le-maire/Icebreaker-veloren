//! Icebreaker RO : la roue des objets (façon Monster Hunter). Tab tenu l'ouvre, un coup de souris dans
//! une direction déclenche l'action de ce secteur ; huit secteurs au plus. Le placement et le seuil du
//! coup de souris sont dans `assets/icebreaker/roue.ron`. Ce module décide (quelle direction, quel objet,
//! disponible ou non) ; `voxygen` ne fait que brancher et dessiner.

use crate::{
    assets::{AssetExt, Ron},
    comp::{
        Inventory,
        inventory::slot::{EquipSlot, InvSlotId},
        item::{ConsumableKind, Item, ItemKind},
    },
};
use serde::Deserialize;
use vek::Vec2;

/// Les huit directions de la roue, dans le repère de l'écran (N en haut).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum Direction8 {
    N,
    NE,
    E,
    SE,
    S,
    SO,
    O,
    NO,
}

impl Direction8 {
    /// Dans l'ordre des aiguilles d'une montre, depuis le nord.
    pub const TOUTES: [Direction8; 8] = [
        Direction8::N,
        Direction8::NE,
        Direction8::E,
        Direction8::SE,
        Direction8::S,
        Direction8::SO,
        Direction8::O,
        Direction8::NO,
    ];

    /// Vecteur unitaire de la direction, repère de l'écran (x vers la droite, y vers le bas).
    pub fn vecteur(self) -> Vec2<f32> {
        let i = Direction8::TOUTES.iter().position(|d| *d == self).unwrap_or(0) as f32;
        let angle = i * std::f32::consts::FRAC_PI_4;
        Vec2::new(angle.sin(), -angle.cos())
    }

    /// La direction la plus proche d'un mouvement (secteurs de 45° centrés sur les huit directions).
    pub fn de(delta: Vec2<f32>) -> Option<Direction8> {
        if delta.magnitude_squared() <= f32::EPSILON {
            return None;
        }
        // Angle depuis le nord, dans le sens des aiguilles d'une montre (y vers le bas).
        let angle = delta.x.atan2(-delta.y).rem_euclid(std::f32::consts::TAU);
        let i = (angle / std::f32::consts::FRAC_PI_4).round() as usize % 8;
        Some(Direction8::TOUTES[i])
    }
}

/// Ce que fait un secteur.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum Action {
    /// La carte : bientôt (ne fait rien pour l'instant).
    Carte,
    /// Échanger les armes des mains principales et secondaires (l'ancien Tab).
    ChangerArme,
    /// Boire la première potion du sac.
    Potion,
    /// Manger la première nourriture du sac.
    Nourriture,
}

impl Action {
    /// L'objet du sac qui convient à cette action (Potion : une boisson ; Nourriture : un plat).
    pub fn prend(self, kind: &ItemKind) -> bool {
        match (self, kind) {
            (Action::Potion, ItemKind::Consumable { kind, .. }) => *kind == ConsumableKind::Drink,
            (Action::Nourriture, ItemKind::Consumable { kind, .. }) => {
                matches!(kind, ConsumableKind::Food | ConsumableKind::ComplexFood)
            },
            _ => false,
        }
    }
}

/// Les réglages de la roue (`assets/icebreaker/roue.ron`).
#[derive(Clone, Debug, Deserialize)]
pub struct Reglages {
    /// Longueur du coup de souris (en pixels de souris, après la sensibilité) qui choisit un secteur.
    pub seuil: f32,
    /// Le placement : au plus un secteur par direction, donc huit au plus.
    pub secteurs: Vec<(Direction8, Action)>,
}

impl Default for Reglages {
    fn default() -> Self {
        Self {
            seuil: 60.0,
            secteurs: Vec::new(),
        }
    }
}

impl Reglages {
    /// Les réglages du jeu ; un fichier absent ou invalide donne une roue vide (avec un message).
    pub fn charger() -> Self {
        match Ron::<Self>::load("icebreaker.roue") {
            Ok(r) => {
                let reglages = r.read().0.clone();
                match reglages.verifier() {
                    Ok(()) => reglages,
                    Err(e) => {
                        tracing::warn!("Icebreaker roue.ron : {e}");
                        Self::default()
                    },
                }
            },
            Err(e) => {
                tracing::warn!("Icebreaker roue.ron illisible : {e}");
                Self::default()
            },
        }
    }

    /// Une direction ne porte qu'une action (donc huit secteurs au plus) ; le seuil est positif.
    pub fn verifier(&self) -> Result<(), String> {
        if self.seuil <= 0.0 {
            return Err(format!("seuil {} : il doit être positif", self.seuil));
        }
        for (i, (dir, _)) in self.secteurs.iter().enumerate() {
            if self.secteurs[..i].iter().any(|(d, _)| d == dir) {
                return Err(format!("deux secteurs sur la direction {dir:?}"));
            }
        }
        Ok(())
    }

    /// L'action placée sur une direction.
    pub fn action(&self, dir: Direction8) -> Option<Action> {
        self.secteurs
            .iter()
            .find(|(d, _)| *d == dir)
            .map(|(_, a)| *a)
    }
}

/// Le coup de souris en cours, pendant que la roue est ouverte.
#[derive(Clone, Debug, Default)]
pub struct Geste {
    /// Le mouvement cumulé depuis l'ouverture (repère de l'écran).
    pub cumul: Vec2<f32>,
    /// La direction déjà rendue : un geste ne choisit qu'une fois.
    pub choisi: Option<Direction8>,
}

impl Geste {
    /// Ajoute un mouvement de souris ; rend la direction la première fois que le cumul dépasse le seuil.
    pub fn ajouter(&mut self, delta: Vec2<f32>, seuil: f32) -> Option<Direction8> {
        if self.choisi.is_some() {
            return None;
        }
        self.cumul += delta;
        if self.cumul.magnitude() < seuil {
            return None;
        }
        self.choisi = Direction8::de(self.cumul);
        self.choisi
    }

    /// La direction visée pour l'affichage (même sous le seuil), et la part du seuil parcourue (0 à 1).
    pub fn vise(&self, seuil: f32) -> Option<(Direction8, f32)> {
        let part = (self.cumul.magnitude() / seuil.max(f32::EPSILON)).min(1.0);
        Direction8::de(self.cumul).map(|d| (d, part))
    }
}

/// Le premier objet du sac qui convient à l'action (Potion, Nourriture), dans l'ordre du sac.
pub fn premier_objet<'a, S>(
    action: Action,
    objets: impl IntoIterator<Item = (S, &'a Item)>,
) -> Option<(S, &'a Item)> {
    objets.into_iter().find(|(_, item)| action.prend(&item.kind()))
}

/// Ce qu'un secteur montre : l'action est-elle possible avec ce qu'on porte ?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etat {
    /// L'action est possible.
    Pret,
    /// Rien dans l'inventaire pour cette action : la croix rouge.
    Manque,
    /// Pas encore dans le jeu (la carte).
    Bientot,
}

/// L'état d'une action : `objet_trouve` pour Potion et Nourriture, `arme_de_rechange` pour ChangerArme.
pub fn etat(action: Action, objet_trouve: bool, arme_de_rechange: bool) -> Etat {
    let pret = match action {
        Action::Carte => return Etat::Bientot,
        Action::ChangerArme => arme_de_rechange,
        Action::Potion | Action::Nourriture => objet_trouve,
    };
    if pret { Etat::Pret } else { Etat::Manque }
}

/// Un secteur de la roue tel qu'on le montre et qu'on le joue.
#[derive(Clone, Debug, PartialEq)]
pub struct Secteur {
    pub direction: Direction8,
    pub action: Action,
    pub etat: Etat,
    /// L'objet du sac que l'action prendra (Potion, Nourriture).
    pub objet: Option<InvSlotId>,
}

/// Les secteurs de la roue pour ce que porte le personnage.
pub fn secteurs(reglages: &Reglages, inventaire: &Inventory) -> Vec<Secteur> {
    let arme_de_rechange = [EquipSlot::InactiveMainhand, EquipSlot::InactiveOffhand]
        .into_iter()
        .any(|slot| inventaire.equipped(slot).is_some());
    reglages
        .secteurs
        .iter()
        .map(|&(direction, action)| {
            let objet = premier_objet(
                action,
                inventaire
                    .slots_with_id()
                    .filter_map(|(id, slot)| slot.as_ref().map(|item| (id, item))),
            )
            .map(|(id, _)| id);
            Secteur {
                direction,
                action,
                etat: etat(action, objet.is_some(), arme_de_rechange),
                objet,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vec2<f32> { Vec2::new(x, y) }

    #[test]
    fn les_huit_directions_de_l_ecran() {
        // RO : « S, SW, W, WN, N, NE, E, SE » ; l'écran a y vers le bas.
        assert_eq!(Direction8::de(v(0.0, -1.0)), Some(Direction8::N));
        assert_eq!(Direction8::de(v(1.0, -1.0)), Some(Direction8::NE));
        assert_eq!(Direction8::de(v(1.0, 0.0)), Some(Direction8::E));
        assert_eq!(Direction8::de(v(1.0, 1.0)), Some(Direction8::SE));
        assert_eq!(Direction8::de(v(0.0, 1.0)), Some(Direction8::S));
        assert_eq!(Direction8::de(v(-1.0, 1.0)), Some(Direction8::SO));
        assert_eq!(Direction8::de(v(-1.0, 0.0)), Some(Direction8::O));
        assert_eq!(Direction8::de(v(-1.0, -1.0)), Some(Direction8::NO));
        assert_eq!(Direction8::de(v(0.0, 0.0)), None);
    }

    #[test]
    fn chaque_secteur_fait_45_degres() {
        // 20° du nord vers l'est : encore le nord ; 25° : le nord-est.
        let a = 20f32.to_radians();
        assert_eq!(Direction8::de(v(a.sin(), -a.cos())), Some(Direction8::N));
        let b = 25f32.to_radians();
        assert_eq!(Direction8::de(v(b.sin(), -b.cos())), Some(Direction8::NE));
        // Le vecteur d'une direction retombe sur elle.
        for d in Direction8::TOUTES {
            assert_eq!(Direction8::de(d.vecteur()), Some(d));
        }
    }

    #[test]
    fn sous_le_seuil_rien_n_est_choisi() {
        // RO : « pour sélectionner un objet il faut donner une direction » : un petit mouvement ne choisit pas.
        let mut g = Geste::default();
        assert_eq!(g.ajouter(v(20.0, 0.0), 60.0), None);
        assert_eq!(g.ajouter(v(20.0, 0.0), 60.0), None);
        assert_eq!(g.ajouter(v(25.0, 0.0), 60.0), Some(Direction8::E));
    }

    #[test]
    fn un_geste_ne_choisit_qu_une_fois() {
        let mut g = Geste::default();
        assert_eq!(g.ajouter(v(0.0, -100.0), 60.0), Some(Direction8::N));
        assert_eq!(g.ajouter(v(100.0, 0.0), 60.0), None);
        assert_eq!(g.ajouter(v(0.0, 300.0), 60.0), None);
    }

    #[test]
    fn la_vise_suit_le_cumul() {
        let mut g = Geste::default();
        g.ajouter(v(-30.0, 0.0), 60.0);
        assert_eq!(g.vise(60.0), Some((Direction8::O, 0.5)));
    }

    #[test]
    fn huit_secteurs_au_plus_un_par_direction() {
        // RO : « capacité maximale de 8 objets ».
        let r = Reglages {
            seuil: 60.0,
            secteurs: vec![(Direction8::E, Action::Potion), (Direction8::E, Action::Nourriture)],
        };
        assert!(r.verifier().is_err());
        let r = Reglages {
            seuil: 0.0,
            secteurs: vec![],
        };
        assert!(r.verifier().is_err());
    }

    #[test]
    fn le_placement_du_jeu() {
        // RO v1 : carte en haut, arme à gauche, potions à droite, nourriture juste en dessous.
        let r = Ron::<Reglages>::load_expect("icebreaker.roue").read().0.clone();
        assert!(r.verifier().is_ok());
        assert_eq!(r.action(Direction8::N), Some(Action::Carte));
        assert_eq!(r.action(Direction8::O), Some(Action::ChangerArme));
        assert_eq!(r.action(Direction8::E), Some(Action::Potion));
        assert_eq!(r.action(Direction8::SE), Some(Action::Nourriture));
        assert_eq!(r.action(Direction8::S), None);
    }

    #[test]
    fn le_premier_objet_du_sac_dans_l_ordre() {
        let pomme = Item::new_from_asset_expect("common.items.food.apple");
        let petite = Item::new_from_asset_expect("common.items.consumable.potion_minor");
        let grande = Item::new_from_asset_expect("common.items.consumable.potion_big");
        let sac = [(0, &pomme), (1, &petite), (2, &grande)];
        assert_eq!(premier_objet(Action::Potion, sac).map(|(s, _)| s), Some(1));
        assert_eq!(premier_objet(Action::Nourriture, sac).map(|(s, _)| s), Some(0));
        assert_eq!(premier_objet(Action::ChangerArme, sac).map(|(s, _)| s), None);
        let sans_potion = [(0, &pomme)];
        assert_eq!(premier_objet(Action::Potion, sans_potion).map(|(s, _)| s), None);
    }

    #[test]
    fn croix_rouge_si_rien_dans_l_inventaire() {
        // RO : « si une liste d'objets n'est pas présente dans l'inventaire on met une croix rouge ».
        assert_eq!(etat(Action::Potion, false, true), Etat::Manque);
        assert_eq!(etat(Action::Potion, true, false), Etat::Pret);
        assert_eq!(etat(Action::Nourriture, false, true), Etat::Manque);
        assert_eq!(etat(Action::ChangerArme, true, false), Etat::Manque);
        assert_eq!(etat(Action::ChangerArme, false, true), Etat::Pret);
        // La carte arrive bientôt : ni prête, ni croix.
        assert_eq!(etat(Action::Carte, true, true), Etat::Bientot);
    }
}
