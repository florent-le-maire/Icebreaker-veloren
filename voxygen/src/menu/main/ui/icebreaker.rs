//! Icebreaker : lieux de la campagne lancés depuis le menu principal.
//!
//! Chaque lieu (`assets/icebreaker/lieux.ron`) est un serveur à part, lancé par
//! `icebreaker.ps1` depuis la racine du dépôt. Le bouton du lieu démarre ce
//! serveur s'il ne tourne pas encore, puis le client s'y connecte comme en
//! multijoueur ; il réessaie toutes les 5 s pendant que le monde se génère. Les
//! serveurs lancés ainsi restent ouverts quand on quitte le jeu (fenêtre à
//! fermer).
//!
//! X.10 : le bouton ouvre d'abord le choix de la partie (continuer une partie
//! de `userdata-icebreaker/parties`, ou en créer une), passée en `-Partie`.

use common::assets::{AssetExt, AssetHandle, Ron};
use serde::Deserialize;
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime},
};
use tracing::{info, warn};

#[derive(Clone, Debug, Deserialize)]
pub struct Lieu {
    pub nom: String,
    /// Arguments d'`icebreaker.ps1` (le port est ajouté avec `-Port`).
    pub arguments: Vec<String>,
    pub port: u16,
    pub actif: bool,
    /// Whether the main menu offers it (decision 0016: the campaign only;
    /// the other places are the destinations of expeditions).
    #[serde(default)]
    pub au_menu: bool,
    /// Icebreaker S.3 : adresse d'un serveur distant (« hôte:port ») ; le
    /// bouton s'y connecte directement, sans choix de partie ni serveur lancé
    /// sur ce PC.
    #[serde(default)]
    pub adresse: Option<String>,
}

pub fn lieux() -> AssetHandle<Ron<Vec<Lieu>>> {
    Ron::load_or_insert_with("icebreaker.lieux", |error| {
        warn!("Icebreaker : lieux.ron illisible, pas de boutons de campagne : {error:#?}");
        Ron(Vec::new())
    })
}

/// Icebreaker X.10 : dossier des parties (une partie = un sous-dossier, écrit
/// par `icebreaker.ps1 -Partie <nom>`), depuis la racine du dépôt.
const PARTIES: &str = "userdata-icebreaker/parties";
/// Où `icebreaker.ps1` note la partie du serveur de la campagne.
const PARTIE_DU_SERVEUR: &str = "userdata-icebreaker/monde/partie.txt";
/// Fichier de la partie récrit toutes les 5 s pendant qu'on joue : sa date dit
/// quand on y a joué pour la dernière fois.
const HORLOGE: &str = "horloge.ron";
const NOM_MAX: usize = 30;

/// Une partie de campagne déjà commencée.
#[derive(Clone, Debug)]
pub struct Partie {
    pub nom: String,
    pub modifiee: SystemTime,
}

/// Le panneau de la campagne ouvert au menu : le lieu cliqué (index dans
/// `lieux.ron`), les parties lues à l'ouverture et le nom tapé.
pub struct MenuParties {
    pub lieu: usize,
    pub parties: Vec<Partie>,
    pub nom: String,
}

/// Les parties du dossier `dossier`, de la plus récente à la plus ancienne.
pub fn parties_dans(dossier: &Path) -> Vec<Partie> {
    let Ok(entrees) = std::fs::read_dir(dossier) else {
        return Vec::new();
    };
    let mut parties: Vec<Partie> = entrees
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| {
            let nom = e.file_name().into_string().ok()?;
            let date = |p: PathBuf| std::fs::metadata(p).and_then(|m| m.modified()).ok();
            let modifiee = date(e.path().join(HORLOGE))
                .or_else(|| date(e.path()))
                .unwrap_or(SystemTime::UNIX_EPOCH);
            Some(Partie { nom, modifiee })
        })
        .collect();
    parties.sort_by(|a, b| b.modifiee.cmp(&a.modifiee).then_with(|| a.nom.cmp(&b.nom)));
    parties
}

/// Les parties de `userdata-icebreaker/parties`.
pub fn parties() -> Vec<Partie> { parties_dans(Path::new(PARTIES)) }

/// La date d'une partie, pour son bouton (« 04/11 14:30 »).
pub fn date_partie(partie: &Partie) -> String {
    chrono::DateTime::<chrono::Local>::from(partie.modifiee)
        .format("%d/%m %H:%M")
        .to_string()
}

/// Pourquoi un nom de nouvelle partie est refusé (clé `.ftl`).
#[derive(Debug, PartialEq, Eq)]
pub enum NomRefuse {
    Caracteres,
    TropLong,
    Existe,
}

impl NomRefuse {
    pub fn cle(&self) -> &'static str {
        match self {
            NomRefuse::Caracteres => "icebreaker-partie-nom-caracteres",
            NomRefuse::TropLong => "icebreaker-partie-nom-long",
            NomRefuse::Existe => "icebreaker-partie-nom-existe",
        }
    }
}

/// Le nom d'une nouvelle partie à partir de la saisie : rogné ; vide, le
/// premier « Partie N » libre ; lettres, chiffres, espaces, `-` et `_`
/// seulement (c'est un nom de dossier) ; pas un nom déjà pris.
pub fn nom_partie(saisie: &str, existantes: &[Partie]) -> Result<String, NomRefuse> {
    let pris = |nom: &str| existantes.iter().any(|p| p.nom.eq_ignore_ascii_case(nom));
    let nom = saisie.trim();
    if nom.is_empty() {
        return Ok((1..)
            .map(|n| format!("Partie {n}"))
            .find(|nom| !pris(nom))
            .expect("un numéro libre"));
    }
    if !nom
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
    {
        return Err(NomRefuse::Caracteres);
    }
    if nom.chars().count() > NOM_MAX {
        return Err(NomRefuse::TropLong);
    }
    if pris(nom) {
        return Err(NomRefuse::Existe);
    }
    Ok(nom.to_string())
}

/// Lance le serveur du lieu pour la partie `partie` s'il n'écoute pas déjà, et
/// renvoie l'adresse où se connecter. Un serveur qui tourne déjà pour une autre
/// partie est refusé (on ne peut en faire tourner qu'une à la fois).
pub fn lancer(lieu: &Lieu, partie: &str) -> Result<String, String> {
    // Icebreaker S.3 : un serveur distant tourne déjà, avec sa partie.
    if let Some(adresse) = &lieu.adresse {
        return Ok(adresse.clone());
    }
    let adresse = format!("localhost:{}", lieu.port);
    let local = SocketAddr::from((Ipv4Addr::LOCALHOST, lieu.port));
    if TcpStream::connect_timeout(&local, Duration::from_millis(300)).is_ok() {
        let en_cours = std::fs::read_to_string(PARTIE_DU_SERVEUR).unwrap_or_default();
        let en_cours = en_cours.trim();
        if !en_cours.is_empty() && en_cours != partie {
            return Err(format!(
                "La partie « {en_cours} » tourne déjà : fermer la fenêtre de son serveur pour \
                 jouer à « {partie} »."
            ));
        }
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
        .args(["-Port", &lieu.port.to_string()])
        .args(["-Partie", partie]);
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
        "Icebreaker : serveur {} lancé (partie « {partie} »), connexion à {adresse}",
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
        assert!(lieux.iter().any(|l| l.au_menu && l.actif));
        let mut ports: Vec<u16> = lieux
            .iter()
            .filter(|l| l.adresse.is_none())
            .flat_map(|l| [l.port, l.port + 1])
            .collect();
        ports.sort_unstable();
        ports.dedup();
        let locaux = lieux.iter().filter(|l| l.adresse.is_none()).count();
        assert_eq!(ports.len(), 2 * locaux, "deux lieux partagent un port");
        assert!(
            lieux
                .iter()
                .filter_map(|l| l.adresse.as_ref())
                .all(|a| a.contains(':'))
        );
    }

    fn partie(nom: &str) -> Partie {
        Partie {
            nom: nom.into(),
            modifiee: SystemTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn nom_de_partie() {
        let existantes = [partie("Partie 1"), partie("Essai")];
        assert_eq!(nom_partie("  ", &existantes), Ok("Partie 2".into()));
        assert_eq!(nom_partie("", &[]), Ok("Partie 1".into()));
        assert_eq!(
            nom_partie(" Les Givrés_2 ", &existantes),
            Ok("Les Givrés_2".into())
        );
        assert_eq!(nom_partie("essai", &existantes), Err(NomRefuse::Existe));
        assert_eq!(nom_partie("a/b", &existantes), Err(NomRefuse::Caracteres));
        assert_eq!(nom_partie("..", &existantes), Err(NomRefuse::Caracteres));
        assert_eq!(
            nom_partie(&"x".repeat(31), &existantes),
            Err(NomRefuse::TropLong)
        );
    }

    #[test]
    fn parties_de_la_plus_recente() {
        let dossier =
            std::env::temp_dir().join(format!("icebreaker-parties-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        for nom in ["Ancienne", "Récente"] {
            std::fs::create_dir_all(dossier.join(nom)).unwrap();
        }
        std::fs::write(dossier.join("derniere.txt"), "Récente").unwrap();
        let horloge = std::fs::File::create(dossier.join("Récente").join(HORLOGE)).unwrap();
        horloge
            .set_modified(SystemTime::now() + Duration::from_secs(60))
            .unwrap();
        let noms: Vec<String> = parties_dans(&dossier).into_iter().map(|p| p.nom).collect();
        let _ = std::fs::remove_dir_all(&dossier);
        assert_eq!(noms, ["Récente", "Ancienne"]);
        assert!(parties_dans(&dossier).is_empty());
    }
}
