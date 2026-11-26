//! La vue d'un ennemi (`design/creatures.md`, « Pour Veloren ») : un cône devant lui, sans mur entre
//! les deux ; tout près, il perçoit aussi dans son dos. Loin ou face à un joueur discret, il ne fait
//! que soupçonner. Le serveur mesure ([`Apercu`]), cette règle dit ce que l'ennemi en perçoit.

use crate::profil::Profil;

/// Ce que le serveur mesure entre un ennemi et quelqu'un d'autre.
#[derive(Clone, Copy, Debug)]
pub struct Apercu {
    /// Blocs.
    pub distance: f32,
    /// Degrés entre son regard et la direction de l'autre (0 : droit devant, 180 : dans son dos).
    pub angle: f32,
    /// Aucun bloc opaque entre leurs yeux.
    pub ligne_de_vue: bool,
    /// Discrétion de l'autre : facteur sur les distances de vue (1 : debout ; moins : accroupi…).
    pub discretion: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Niveau {
    Rien,
    /// Il a vu quelque chose : il regarde, sans savoir encore.
    Soupcon,
    /// Il sait qui est là.
    Net,
}

/// Ce qu'un ennemi de ce profil perçoit de cet aperçu.
pub fn niveau(profil: &Profil, apercu: &Apercu) -> Niveau {
    let dans = |portee: Option<f32>, facteur: f32| portee.is_some_and(|p| apercu.distance <= p * facteur);
    if !apercu.ligne_de_vue {
        Niveau::Rien
    } else if dans(profil.dos, 1.0) {
        Niveau::Net
    } else if profil.cone.is_none_or(|cone| apercu.angle > cone / 2.0) {
        Niveau::Rien
    } else if dans(profil.vue_nette, apercu.discretion) {
        Niveau::Net
    } else if dans(profil.vue, apercu.discretion) {
        Niveau::Soupcon
    } else {
        Niveau::Rien
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn humain() -> Profil {
        Profil { vue: Some(30.0), vue_nette: Some(16.0), cone: Some(120.0), dos: Some(3.0), ..Profil::default() }
    }

    fn a(distance: f32, angle: f32) -> Apercu { Apercu { distance, angle, ligne_de_vue: true, discretion: 1.0 } }

    /// « Vue : cône de 120° […], sans mur entre les deux. »
    #[test]
    fn il_voit_dans_son_cone_sans_mur() {
        assert_eq!(niveau(&humain(), &a(10.0, 0.0)), Niveau::Net);
        assert_eq!(niveau(&humain(), &a(10.0, 59.0)), Niveau::Net);
        assert_eq!(niveau(&humain(), &a(10.0, 61.0)), Niveau::Rien);
        let mur = Apercu { ligne_de_vue: false, ..a(10.0, 0.0) };
        assert_eq!(niveau(&humain(), &mur), Niveau::Rien);
    }

    /// « À moins de 2 m elle perçoit aussi dans son dos » (sans mur entre les deux).
    #[test]
    fn tout_pres_il_percoit_dans_son_dos() {
        assert_eq!(niveau(&humain(), &a(2.0, 180.0)), Niveau::Net);
        assert_eq!(niveau(&humain(), &a(4.0, 180.0)), Niveau::Rien);
        let mur = Apercu { ligne_de_vue: false, ..a(2.0, 180.0) };
        assert_eq!(niveau(&humain(), &mur), Niveau::Rien);
    }

    /// Loin, il ne fait que soupçonner ; plus loin que sa vue, rien.
    #[test]
    fn loin_il_soupconne() {
        assert_eq!(niveau(&humain(), &a(20.0, 0.0)), Niveau::Soupcon);
        assert_eq!(niveau(&humain(), &a(31.0, 0.0)), Niveau::Rien);
    }

    /// Un joueur discret est vu de moins loin.
    #[test]
    fn un_joueur_discret_est_vu_de_moins_loin() {
        let discret = |distance| Apercu { discretion: 0.5, ..a(distance, 0.0) };
        assert_eq!(niveau(&humain(), &discret(7.0)), Niveau::Net);
        assert_eq!(niveau(&humain(), &discret(10.0)), Niveau::Soupcon);
        assert_eq!(niveau(&humain(), &discret(16.0)), Niveau::Rien);
        // Mais pas dans son dos : tout près, la discrétion ne change rien.
        assert_eq!(niveau(&humain(), &Apercu { discretion: 0.5, ..a(2.0, 180.0) }), Niveau::Net);
    }
}
