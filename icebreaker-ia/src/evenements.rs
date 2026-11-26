//! Ce qui arrive à un ennemi et lui fait tirer une réaction, et ce que les ennemis se disent.

use serde::Deserialize;
use vek::Vec3;

/// Un événement : le profil dit quelles réactions il fait tirer, avec quels poids
/// (`profils.ron` → `reactions`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Evenement {
    /// Il repère un intrus alors qu'il n'avait pas de cible.
    VoitIntrus,
    /// Un des siens meurt à portée d'oreille.
    AllieTombe,
    /// Un adversaire arme un coup contre lui.
    CoupArme,
    /// Il fuit sans arriver à s'éloigner, la menace toute proche.
    Coince,
    /// On le frappe alors que ses PV sont sous `appel_pv`.
    Touche,
    /// Son chef meurt à portée d'oreille.
    ChefTombe,
    /// Ses PV passent sous `pv_bas` (une fois).
    PvBas,
}

/// Ce qu'un ennemi crie aux siens, ou ce que le monde lui apprend. Jamais la position d'un joueur
/// que personne n'a perçu : seulement ce que l'auteur du message savait.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    /// « Des intrus ! » : où on les a vus, et quand.
    Signalement { lieu: Vec3<f32>, vu_a: f64 },
    /// Un des siens est mort là.
    AllieTombe { lieu: Vec3<f32> },
    /// « À l'aide ! » : qui l'a frappé, et où il est.
    Appel { agresseur: crate::cerveau::Id, lieu: Vec3<f32> },
    /// Le chef est tombé là.
    ChefTombe { lieu: Vec3<f32> },
}
