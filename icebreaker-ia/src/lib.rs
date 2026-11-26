//! Icebreaker : la décision de nos ennemis humanoïdes (jalon IA, `docs/icebreaker/PLAN-IA.md`).
//!
//! Le serveur construit à chaque tick l'[`Observation`] d'un ennemi (ce qu'il perçoit, rien d'autre),
//! [`Cerveau::tick`] rend son [`Intention`], que le serveur exécute avec les moyens de Veloren. Les
//! réglages viennent de `assets/icebreaker/ia/profils.ron` ([`Profils`]).

pub mod cerveau;
pub mod cible;
pub mod ouie;
pub mod perception;
pub mod profil;

pub use cerveau::{Cerveau, Etat, Id, Intention, Observation, Percu};
pub use ouie::Bruits;
pub use perception::{Apercu, Niveau, niveau};
pub use profil::{Profil, Profils};
