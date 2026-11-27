//! Icebreaker CA : la carte de l'expédition, côté serveur. L'objet est remis à l'arrivée dans un lieu
//! (comme la balise) ; les objectifs du lieu (le chef à tuer) sont envoyés au joueur quand ils changent
//! (`sys::icebreaker_carte`). Le reste (ce qu'on a exploré, le dessin) est fait par le client.

use common::{
    comp::{Inventory, Item},
    icebreaker_carte::{CARTE, Objectif, est_carte},
};
use hashbrown::HashMap;
use specs::Entity;

/// Ce qui a déjà été envoyé à chaque joueur (`None` : pas en expédition), et quand on a regardé.
pub struct IcebreakerCartes {
    pub envoyes: HashMap<Entity, Option<Vec<Objectif>>>,
    pub vu: f64,
    /// Secondes entre deux regards (`envoi` de `assets/icebreaker/carte.ron`).
    pub envoi: f64,
}

impl Default for IcebreakerCartes {
    fn default() -> Self {
        Self {
            envoyes: HashMap::new(),
            vu: 0.0,
            envoi: common::icebreaker_carte::Reglages::charger().envoi,
        }
    }
}

/// Donne une carte à `inventory` s'il n'en a pas (à l'arrivée dans un donjon).
pub fn donner_si_absente(inventory: &mut Inventory) {
    if !inventory.slots().flatten().any(est_carte) {
        let _ = inventory.push(Item::new_from_asset_expect(CARTE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_carte_est_donnee_une_seule_fois() {
        // CA (Flo, 2026-11-27) : « un item qui sera donné dans l'inventaire ».
        let mut sac = Inventory::with_empty();
        let cartes = |sac: &Inventory| sac.slots().flatten().filter(|item| est_carte(item)).count();
        assert_eq!(cartes(&sac), 0);
        donner_si_absente(&mut sac);
        assert_eq!(cartes(&sac), 1);
        donner_si_absente(&mut sac);
        assert_eq!(cartes(&sac), 1);
    }
}
