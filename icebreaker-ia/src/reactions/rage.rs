//! La rage (Y.4, IA.11) : « Orcs en rage — sous la moitié des PV, une fois : dégâts et vitesse en
//! plus. » Trois réglages de profil font une race qui s'enrage : `pv_bas` (le seuil), `rage` (cet
//! effet), et `reactions: { PvBas: [(Rage, 1.0)] }`.

use serde::Deserialize;

/// Ce que la rage lui donne (`profils.ron` → `rage`).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Rage {
    /// Sa force : +`force` de dégâts infligés (0,4 = +40 %), plus vite, et autant de dégâts reçus en plus.
    pub force: f32,
    /// Secondes qu'elle dure.
    pub duree: f64,
}

/// Ce qu'une réaction fait à celui qui la tire, que le serveur applique avec les moyens de Veloren.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effet {
    /// La rage, de cette force, pour tant de secondes.
    Rage { force: f32, duree: f64 },
}

impl Rage {
    pub fn effet(&self) -> Effet { Effet::Rage { force: self.force, duree: self.duree } }
}
