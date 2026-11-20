//! Icebreaker : la mort en expédition (jalon T). Le mort laisse un sac avec tout son inventaire
//! hors équipement porté (T.3) ; un allié le fouille pour en prendre le contenu.

use common::{
    comp::{Inventory, Item},
    icebreaker_mort::SAC_DU_MORT,
};

/// Vide l'inventaire d'un mort (hors équipement porté : armes, armure, sacs équipés restent ;
/// le contenu des sacs part) dans un sac du mort, avec son `cristal` en première place (T.11). Rien à
/// prendre : pas de sac.
pub fn remplir_sac(inventory: &mut Inventory, cristal: Option<Item>) -> Option<Item> {
    let mut sac = Item::new_from_asset_expect(SAC_DU_MORT);
    let places = sac.slots().len();
    let objets = cristal.into_iter().chain(inventory.drain().collect::<Vec<_>>()).collect::<Vec<_>>();
    if objets.is_empty() {
        return None;
    }
    let mut objets = objets.into_iter();
    for (place, objet) in sac.slots_mut().iter_mut().zip(objets.by_ref()) {
        *place = Some(objet);
    }
    // Plus d'objets que de places (ne devrait pas arriver) : ils restent au mort.
    for objet in objets {
        let _ = inventory.push(objet);
    }
    tracing::debug!(places, "Icebreaker : sac du mort rempli");
    Some(sac)
}

/// Passe le contenu du sac dans l'inventaire du fouilleur, autant qu'il en rentre ; vrai si le
/// sac est vide après.
pub fn fouiller(sac: &mut Item, inventory: &mut Inventory) -> bool {
    for place in sac.slots_mut().iter_mut() {
        if let Some(objet) = place.take()
            && let Err((reste, _)) = inventory.push(objet)
        {
            *place = Some(reste);
        }
    }
    sac.slots().iter().all(Option::is_none)
}

/// Retire de `inventory` les cristaux de morts qu'il porte (T.11) : à l'autel, qui relève leurs
/// morts, ou au retour au village, où ils ne servent plus.
pub fn prendre_cristaux(inventory: &mut Inventory) -> Vec<Item> {
    let places = inventory
        .slots_with_id()
        .filter(|(_, objet)| objet.as_ref().is_some_and(common::icebreaker_mort::est_cristal))
        .map(|(place, _)| place)
        .collect::<Vec<_>>();
    places.into_iter().filter_map(|place| inventory.remove(place)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pieces(n: u32) -> Item {
        let mut item = Item::new_from_asset_expect("common.items.utility.coins");
        item.set_amount(n).unwrap();
        item
    }

    #[test]
    fn le_sac_prend_l_inventaire_et_le_rend() {
        let mut mort = Inventory::with_empty();
        mort.push(pieces(120)).unwrap();
        mort.push(Item::new_from_asset_expect("common.items.consumable.potion_minor"))
            .unwrap();
        let mut sac = remplir_sac(&mut mort, None).expect("un sac");
        assert_eq!(mort.populated_slots(), 0);

        let mut allie = Inventory::with_empty();
        assert!(fouiller(&mut sac, &mut allie));
        assert_eq!(allie.populated_slots(), 2);
    }

    #[test]
    fn le_cristal_du_mort_passe_au_fouilleur() {
        use common::icebreaker_mort::{cristal_de, est_cristal};
        let uid = |n| common::uid::Uid(std::num::NonZeroU64::new(n).unwrap());
        // Même sans rien porter, le mort laisse un sac : son cristal y est, en première place.
        let mut mort = Inventory::with_empty();
        let mut sac = remplir_sac(&mut mort, Some(cristal_de(uid(7), "Flo"))).expect("un sac");
        assert!(sac.slots()[0].as_ref().is_some_and(est_cristal));

        let mut allie = Inventory::with_empty();
        allie.push(pieces(5)).unwrap();
        allie.push(cristal_de(uid(8), "Sayanel")).unwrap();
        assert!(fouiller(&mut sac, &mut allie));
        // L'allié porte deux cristaux, de deux morts : l'autel (ou le retour) les lui retire.
        let cristaux = prendre_cristaux(&mut allie);
        let mut morts = cristaux.iter().filter_map(|c| c.icebreaker_proprietaire()).map(|(u, _)| u).collect::<Vec<_>>();
        morts.sort_by_key(|u| u.0);
        assert_eq!(morts, [uid(7), uid(8)]);
        assert_eq!(allie.populated_slots(), 1);
    }

    #[test]
    fn rien_a_prendre_pas_de_sac() {
        assert!(remplir_sac(&mut Inventory::with_empty(), None).is_none());
    }

    #[test]
    fn inventaire_plein_le_reste_reste_dans_le_sac() {
        let mut mort = Inventory::with_empty();
        mort.push(Item::new_from_asset_expect("common.items.consumable.potion_minor"))
            .unwrap();
        let mut sac = remplir_sac(&mut mort, None).unwrap();

        let mut allie = Inventory::with_empty();
        // Remplit chaque place de l'allié d'un objet qui ne s'empile pas avec la potion.
        while allie
            .push(Item::new_from_asset_expect("common.items.weapons.sword.starter"))
            .is_ok()
        {}
        assert!(!fouiller(&mut sac, &mut allie));
        assert!(sac.slots().iter().any(Option::is_some));
    }
}
