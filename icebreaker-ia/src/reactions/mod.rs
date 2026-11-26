//! Les réactions qu'un ennemi peut tirer à un événement. Les deux premières sont celles du tronc
//! (`cerveau.rs`) ; chaque autre a son fichier ici.

pub mod coince;
pub mod defense;
pub mod prevenir;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Reaction {
    /// Se battre : ce que le tronc fait d'un intrus repéré.
    Combattre,
    /// Aller voir : ce que le tronc fait d'un bruit.
    AllerVoir,
    /// Courir prévenir un allié qui ne sait rien (`prevenir.rs`).
    Prevenir,
    /// Ne rien faire de plus (encaisser un coup, par exemple).
    Rien,
    /// Rouler hors d'un coup qui arrive (`defense.rs`).
    Esquiver,
    /// Parer un coup qui arrive (`defense.rs`).
    Parer,
    /// Coincé : se tapir sur place, face à la menace (`coince.rs`).
    SeTapir,
    /// Coincé : forcer le passage à travers la menace (`coince.rs`).
    Forcer,
    /// Frappé : crier « À l'aide ! » (le tronc ; ceux qui défendent viennent sur l'agresseur).
    AppelerALAide,
    /// Le chef est tombé : fuir un moment, puis se reprendre (le tronc).
    Deroute,
}
