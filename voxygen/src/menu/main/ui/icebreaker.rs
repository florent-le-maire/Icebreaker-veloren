//! Icebreaker : lieux de la campagne lancés depuis le menu principal.
//!
//! Chaque lieu (`assets/icebreaker/lieux.ron`) est un serveur à part, lancé par
//! `icebreaker.ps1` depuis la racine du dépôt. Le bouton du lieu démarre ce
//! serveur s'il ne tourne pas encore, puis le client s'y connecte comme en
//! multijoueur ; il réessaie toutes les 5 s pendant que le monde se génère. Les
//! serveurs lancés ainsi restent ouverts quand on quitte le jeu (fenêtre à
//! fermer).

use common::assets::{AssetExt, AssetHandle, Ron};
use serde::Deserialize;
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    path::PathBuf,
    process::Command,
    time::Duration,
};
use tracing::{info, warn};

#[derive(Clone, Debug, Deserialize)]
pub struct Lieu {
    pub nom: String,
    /// Arguments d'`icebreaker.ps1` (le port est ajouté avec `-Port`).
    pub arguments: Vec<String>,
    pub port: u16,
    pub actif: bool,
}

pub fn lieux() -> AssetHandle<Ron<Vec<Lieu>>> {
    Ron::load_or_insert_with("icebreaker.lieux", |error| {
        warn!("Icebreaker : lieux.ron illisible, pas de boutons de campagne : {error:#?}");
        Ron(Vec::new())
    })
}

/// Lance le serveur du lieu s'il n'écoute pas déjà, et renvoie l'adresse où se
/// connecter.
pub fn lancer(lieu: &Lieu) -> Result<String, String> {
    let adresse = format!("localhost:{}", lieu.port);
    let local = SocketAddr::from((Ipv4Addr::LOCALHOST, lieu.port));
    if TcpStream::connect_timeout(&local, Duration::from_millis(300)).is_ok() {
        info!("Icebreaker : {} tourne déjà sur {adresse}", lieu.nom);
        return Ok(adresse);
    }

    let script = PathBuf::from("icebreaker.ps1");
    if !script.is_file() {
        return Err(format!(
            "Icebreaker : icebreaker.ps1 introuvable, lancer le jeu depuis la racine du dépôt \
             (dossier courant : {}).",
            std::env::current_dir()
                .map(|d| d.display().to_string())
                .unwrap_or_default()
        ));
    }
    let mut commande = Command::new("powershell");
    commande
        // -NoExit : la fenêtre reste ouverte si le serveur s'arrête (serveur non compilé, erreur).
        .args(["-NoExit", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&script)
        .args(&lieu.arguments)
        .args(["-Port", &lieu.port.to_string()]);
    // Une console à lui : on voit le journal du serveur et on le ferme en fermant
    // la fenêtre.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        commande.creation_flags(CREATE_NEW_CONSOLE);
    }
    commande
        .spawn()
        .map_err(|e| format!("Icebreaker : impossible de lancer {} : {e}", lieu.nom))?;
    info!(
        "Icebreaker : serveur {} lancé, connexion à {adresse}",
        lieu.nom
    );
    Ok(adresse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lieux_ron_se_lit() {
        let lieux = Ron::<Vec<Lieu>>::load_expect("icebreaker.lieux")
            .read()
            .0
            .clone();
        assert!(lieux.iter().any(|l| l.nom == "Village" && l.actif));
        let mut ports: Vec<u16> = lieux.iter().flat_map(|l| [l.port, l.port + 1]).collect();
        ports.sort_unstable();
        ports.dedup();
        assert_eq!(ports.len(), 2 * lieux.len(), "deux lieux partagent un port");
    }
}
