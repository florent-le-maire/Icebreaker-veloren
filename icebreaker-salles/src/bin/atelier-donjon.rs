//! Icebreaker F.34 : le serveur de l'atelier du donjon (`docs/icebreaker/PLAN-PLAYTEST.md`, « Donjon souterrain »),
//! lancé par `.\icebreaker.ps1 donjon-atelier` depuis la racine du dépôt. Comme l'atelier des grottes
//! (`icebreaker-caverne/src/bin/atelier.rs`) : il sert la page sur cette machine seulement, calcule le plan d'un
//! donjon par le générateur du jeu, et enregistre un preset dans `assets/icebreaker/donjons/formes.ron` (ce fichier
//! seul, committé).
//!
//! `GET /` la page ; `GET /formes` les presets et les valeurs de `generation.ron` (JSON) ;
//! `POST /plan` `{graine, taille, forme}` → le plan (JSON) ; `POST /preset` `{nom, poids, forme}` et
//! `POST /preset/supprimer` `{nom}` répondent « ok » ou l'erreur.

use serde::Deserialize;
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::Command,
};
use world::icebreaker::dungeon::{
    self, Settings,
    formes::{Forme, Formes},
};

const FORMES: &str = "assets/icebreaker/donjons/formes.ron";
const PAGE: &str = "icebreaker-salles/web/donjon.html";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.iter().position(|a| a == "--port").and_then(|i| args.get(i + 1)?.parse().ok()).unwrap_or(8098);
    if !Path::new(FORMES).exists() {
        eprintln!("Atelier du donjon : {FORMES} introuvable, lancer depuis la racine du dépôt.");
        std::process::exit(1);
    }
    let ecoute = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(ecoute) => ecoute,
        Err(e) => {
            eprintln!("Atelier du donjon : le port {port} est pris ({e}). Un atelier tourne peut-être déjà.");
            std::process::exit(1);
        },
    };
    let branche = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output();
    let branche = branche.map(|s| String::from_utf8_lossy(&s.stdout).trim().to_string()).unwrap_or_default();
    println!("Atelier du donjon : http://localhost:{port} (Ctrl+C pour arrêter) ; les presets sont committés sur « {branche} »");
    for flux in ecoute.incoming().flatten() {
        std::thread::spawn(move || {
            let _ = servir(flux, port);
        });
    }
}

fn repondre(flux: &mut TcpStream, statut: &str, genre: &str, corps: &[u8]) -> std::io::Result<()> {
    write!(
        flux,
        "HTTP/1.1 {statut}\r\nContent-Type: {genre}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        corps.len()
    )?;
    flux.write_all(corps)
}

fn servir(mut flux: TcpStream, port: u16) -> std::io::Result<()> {
    flux.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
    let mut lecture = BufReader::new(flux.try_clone()?);
    let mut premiere = String::new();
    lecture.read_line(&mut premiere)?;
    let mut mots = premiere.split_whitespace();
    let (methode, chemin) = (mots.next().unwrap_or(""), mots.next().unwrap_or("/"));
    let chemin = chemin.split(['?', '#']).next().unwrap_or("/");
    let (mut longueur, mut origine) = (0usize, None);
    loop {
        let mut ligne = String::new();
        if lecture.read_line(&mut ligne)? == 0 || ligne.trim().is_empty() {
            break;
        }
        if let Some((cle, valeur)) = ligne.split_once(':') {
            match cle.to_ascii_lowercase().as_str() {
                "content-length" => longueur = valeur.trim().parse().unwrap_or(0),
                "origin" => origine = Some(valeur.trim().to_string()),
                _ => {},
            }
        }
    }
    let texte = "text/plain; charset=utf-8";
    let json_type = "application/json; charset=utf-8";
    if methode == "GET" {
        return match chemin {
            "/" | "/index.html" => match std::fs::read(PAGE) {
                Ok(corps) => repondre(&mut flux, "200 OK", "text/html; charset=utf-8", &corps),
                Err(e) => repondre(&mut flux, "404 Not Found", texte, format!("{PAGE} : {e}").as_bytes()),
            },
            "/formes" => match formes_json() {
                Ok(corps) => repondre(&mut flux, "200 OK", json_type, corps.as_bytes()),
                Err(e) => repondre(&mut flux, "500 Internal Server Error", texte, e.as_bytes()),
            },
            _ => repondre(&mut flux, "404 Not Found", texte, "introuvable".as_bytes()),
        };
    }
    if methode != "POST" || !matches!(chemin, "/plan" | "/preset" | "/preset/supprimer") {
        return repondre(&mut flux, "404 Not Found", texte, "introuvable".as_bytes());
    }
    // Only the page of this server asks: not another site open in the browser.
    let ici = [format!("http://localhost:{port}"), format!("http://127.0.0.1:{port}")];
    if origine.is_some_and(|o| !ici.contains(&o)) || longueur > 100_000 {
        return repondre(&mut flux, "403 Forbidden", texte, "demande refusée".as_bytes());
    }
    let mut corps = vec![0; longueur];
    lecture.read_exact(&mut corps)?;
    let corps = String::from_utf8_lossy(&corps);
    match chemin {
        "/plan" => {
            let reponse = plan_json(&corps).unwrap_or_else(|e| json!({ "erreur": e }).to_string());
            repondre(&mut flux, "200 OK", json_type, reponse.as_bytes())
        },
        _ => match enregistrer(chemin == "/preset", &corps) {
            Ok(()) => repondre(&mut flux, "200 OK", texte, b"ok"),
            Err(e) => repondre(&mut flux, "400 Bad Request", texte, e.as_bytes()),
        },
    }
}

/// The presets, and what `generation.ron` gives without one (the graph of each size, the spacing, the depth).
fn formes_json() -> Result<String, String> {
    let settings = Settings::load()?;
    let tailles: serde_json::Map<String, serde_json::Value> = settings
        .sizes
        .iter()
        .map(|(nom, s)| {
            (nom.clone(), json!({
                "main_path": s.main_path, "levels": s.levels, "branches": s.branches,
                "branch_length": s.branch_length, "locks": s.locks, "loops": s.loops, "drops": s.drops,
            }))
        })
        .collect();
    Ok(json!({
        "formes": Formes::load()?,
        "base": {
            "tailles": tailles,
            "spacing": settings.assembly.spacing,
            "profondeur": settings.placement.profondeur,
        },
    })
    .to_string())
}

#[derive(Deserialize)]
struct Demande {
    graine: u64,
    taille: String,
    forme: Forme,
}

/// The plan of the dungeon of `graine`, of size `taille`, with the settings changed by `forme`: its cells
/// `[x, y, level, symbol, room]` (room: index in `salles`, -1 out of the rooms), its rooms, where one arrives.
fn plan_json(corps: &str) -> Result<String, String> {
    let demande: Demande = serde_json::from_str(corps).map_err(|e| format!("demande illisible : {e}"))?;
    let erreurs = demande.forme.erreurs();
    if !erreurs.is_empty() {
        return Err(erreurs.join(" ; "));
    }
    let settings = demande.forme.appliquer(Settings::load()?);
    let donjon = dungeon::plan_atelier(&settings, demande.graine, &demande.taille)?;
    let mut salle_de = std::collections::BTreeMap::new();
    for (i, salle) in donjon.salles.iter().enumerate() {
        for cell in &salle.cells {
            salle_de.entry(*cell).or_insert(i as i32);
        }
    }
    let cases: Vec<serde_json::Value> = donjon
        .plan
        .cells
        .iter()
        .filter(|(_, s)| **s != ' ')
        .map(|(c, s)| json!([c.x, c.y, c.level, s.to_string(), salle_de.get(c).copied().unwrap_or(-1)]))
        .collect();
    let mut niveaux: Vec<i32> = donjon.plan.cells.keys().map(|c| c.level).collect();
    niveaux.sort_unstable_by(|a, b| b.cmp(a));
    niveaux.dedup();
    let creatures = donjon.plan.cells.values().filter(|s| dungeon::populate::kind_of(**s).is_some()).count();
    let coffres = donjon.plan.cells.values().filter(|s| **s == '$').count();
    let salles: Vec<serde_json::Value> = donjon
        .salles
        .iter()
        .map(|s| json!({ "role": s.role, "id": s.id, "cases": s.cells.iter().map(|c| [c.x, c.y, c.level]).collect::<Vec<_>>() }))
        .collect();
    let a = donjon.arrival;
    Ok(json!({
        "cases": cases,
        "salles": salles,
        "niveaux": niveaux,
        "arrivee": [a.x, a.y, a.level],
        "creatures": creatures,
        "coffres": coffres,
        "resume": donjon.summary,
    })
    .to_string())
}

fn git(args: &[&str]) -> Result<(), String> {
    let sortie = Command::new("git").args(args).output().map_err(|e| format!("git : {e}"))?;
    if sortie.status.success() {
        Ok(())
    } else {
        Err(format!("git {} : {}", args[0], String::from_utf8_lossy(&sortie.stderr).trim()))
    }
}

#[derive(Deserialize)]
struct Preset {
    nom: String,
    #[serde(default)]
    poids: f64,
    #[serde(default)]
    forme: Forme,
}

/// Writes the preset in `formes.ron` (or removes it) and commits the file; the file is put back as it was if the
/// commit fails.
fn enregistrer(poser: bool, corps: &str) -> Result<(), String> {
    static UN_A_LA_FOIS: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _garde = UN_A_LA_FOIS.lock().unwrap_or_else(|e| e.into_inner());
    let preset: Preset = serde_json::from_str(corps).map_err(|e| format!("demande illisible : {e}"))?;
    let nom = preset.nom.trim();
    let avant = std::fs::read_to_string(FORMES).map_err(|e| format!("{FORMES} : {e}"))?;
    let (apres, message) = if poser {
        (
            dungeon::formes::poser_profil(&avant, nom, preset.poids, &preset.forme)?,
            format!("Icebreaker : preset de donjon « {nom} » (atelier)"),
        )
    } else {
        (dungeon::formes::retirer_profil(&avant, nom)?, format!("Icebreaker : preset de donjon « {nom} » retiré (atelier)"))
    };
    if apres == avant {
        return Ok(());
    }
    std::fs::write(FORMES, &apres).map_err(|e| format!("écriture de {FORMES} : {e}"))?;
    if let Err(e) = git(&["add", "--", FORMES]).and_then(|_| git(&["commit", "-q", "-m", &message, "--", FORMES])) {
        let _ = std::fs::write(FORMES, &avant);
        let _ = git(&["reset", "-q", "--", FORMES]);
        return Err(format!("commit refusé, fichier remis comme avant ({e})"));
    }
    println!("{message}");
    Ok(())
}
