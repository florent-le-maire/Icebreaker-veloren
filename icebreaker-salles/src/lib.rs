//! Icebreaker : ce que le tableur des salles (`main.rs`) et l'atelier des salles (`bin/atelier-salles.rs`)
//! partagent : le catalogue des pièces, `assets/icebreaker/donjons/pieces.ron` (F.10).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PIECES: &str = "assets/icebreaker/donjons/pieces.ron";

/// Le catalogue des pièces.
#[derive(Deserialize, Serialize)]
pub struct Pieces {
    pub pieces: Vec<Piece>,
    pub modeles: BTreeMap<String, Model>,
}

/// Une fonction de salle : ce qu'on s'attend à y voir, ce que Veloren a déjà, ce qui manque.
#[derive(Deserialize, Serialize)]
pub struct Piece {
    pub id: String,
    pub nom: String,
    pub roles: Vec<String>,
    pub themes: Vec<String>,
    pub idee: String,
    /// (meuble, sprite de Veloren).
    pub meubles: Vec<(String, String)>,
    /// Identifiants de `modeles`.
    pub manques: Vec<String>,
}

/// Un modèle voxel à créer (F.13).
#[derive(Deserialize, Serialize)]
pub struct Model {
    pub nom: String,
    pub lot: u32,
    pub idee: String,
}

impl Pieces {
    /// Lit `pieces.ron` (depuis la racine du dépôt).
    pub fn load() -> Result<Self, String> {
        let text = std::fs::read_to_string(PIECES).map_err(|e| format!("{PIECES} : {e}"))?;
        ron::from_str(&text).map_err(|e| format!("{PIECES} : {e}"))
    }
}
