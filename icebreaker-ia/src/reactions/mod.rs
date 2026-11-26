//! Les réactions qu'un ennemi peut tirer à un événement. Les deux premières sont celles du tronc
//! (`cerveau.rs`) ; chaque autre a son fichier ici.

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
}
