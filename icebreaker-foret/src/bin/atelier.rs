//! Icebreaker FO.3: the server of the forest workshop
//! (`docs/icebreaker/PLAN-FORET.md`), started by `.\icebreaker.ps1 foret` from
//! the root of the repository. As the one of the caves
//! (`icebreaker-caverne/src/bin/atelier.rs`): it serves the page on this
//! machine only, and saves the presets: a preset is a profile of
//! `assets/icebreaker/forets/formes.ron`, written then committed (that file
//! alone).
//!
//! `GET /` the page, `/foret-wasm.js` the generator, `/formes` the file;
//! `POST /preset` (three lines: name, weight, the settings in the order of
//! `CHAMPS`) and `POST /preset/supprimer` (the name) answer the new file.

use icebreaker_foret::{
    CHAMPS, Forme,
    presets::{poser_profil, retirer_profil},
};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::Command,
};

const FORMES: &str = "assets/icebreaker/forets/formes.ron";
const PAGE: &str = "icebreaker-foret/web";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.iter().position(|a| a == "--port").and_then(|i| args.get(i + 1)?.parse().ok()).unwrap_or(8099);
    if !Path::new(FORMES).exists() {
        eprintln!("Atelier : {FORMES} introuvable, lancer depuis la racine du dépôt.");
        std::process::exit(1);
    }
    let ecoute = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(ecoute) => ecoute,
        Err(e) => {
            eprintln!("Atelier : le port {port} est pris ({e}). Un atelier tourne peut-être déjà.");
            std::process::exit(1);
        },
    };
    println!("Atelier des forêts : http://localhost:{port} (Ctrl+C pour arrêter)");
    // A thread for each connection: a browser keeps idle ones open.
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
    let fichier = |flux: &mut TcpStream, chemin: &str, genre: &str| match std::fs::read(chemin) {
        Ok(corps) => repondre(flux, "200 OK", genre, &corps),
        Err(e) => repondre(flux, "404 Not Found", texte, format!("{chemin} : {e}").as_bytes()),
    };
    match (methode, chemin) {
        ("GET", "/" | "/index.html") => fichier(&mut flux, &format!("{PAGE}/index.html"), "text/html; charset=utf-8"),
        ("GET", "/foret-wasm.js") => fichier(&mut flux, &format!("{PAGE}/foret-wasm.js"), "text/javascript; charset=utf-8"),
        ("GET", "/formes") => fichier(&mut flux, FORMES, texte),
        ("POST", "/preset" | "/preset/supprimer") => {
            // Only the page of this server writes: not another site open in the browser.
            let ici = [format!("http://localhost:{port}"), format!("http://127.0.0.1:{port}")];
            if origine.is_some_and(|o| !ici.contains(&o)) || longueur > 100_000 {
                return repondre(&mut flux, "403 Forbidden", texte, "demande refusée".as_bytes());
            }
            let mut corps = vec![0; longueur];
            lecture.read_exact(&mut corps)?;
            match enregistrer(chemin == "/preset", &String::from_utf8_lossy(&corps)) {
                Ok(formes) => repondre(&mut flux, "200 OK", texte, formes.as_bytes()),
                Err(e) => repondre(&mut flux, "400 Bad Request", texte, e.as_bytes()),
            }
        },
        _ => repondre(&mut flux, "404 Not Found", texte, "introuvable".as_bytes()),
    }
}

fn git(args: &[&str]) -> Result<(), String> {
    let sortie = Command::new("git").args(args).output().map_err(|e| format!("git : {e}"))?;
    if sortie.status.success() {
        Ok(())
    } else {
        Err(format!("git {} : {}", args[0], String::from_utf8_lossy(&sortie.stderr).trim()))
    }
}

/// Writes the preset in `formes.ron` (or removes it) and commits the file;
/// the file is put back as it was if the commit fails.
fn enregistrer(poser: bool, corps: &str) -> Result<String, String> {
    static UN_A_LA_FOIS: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _garde = UN_A_LA_FOIS.lock().unwrap_or_else(|e| e.into_inner());
    let avant = std::fs::read_to_string(FORMES).map_err(|e| format!("{FORMES} : {e}"))?;
    let mut lignes = corps.lines();
    let nom = lignes.next().unwrap_or("").trim();
    let (apres, message) = if poser {
        let poids: f32 = lignes.next().unwrap_or("").trim().parse().map_err(|_| "poids illisible")?;
        let valeurs: Vec<f32> = lignes.next().unwrap_or("").split_whitespace().filter_map(|v| v.parse().ok()).collect();
        if valeurs.len() != CHAMPS.len() {
            return Err(format!(
                "{} réglages reçus, {} attendus : recharger la page (générateur recompilé ?)",
                valeurs.len(),
                CHAMPS.len()
            ));
        }
        let forme = Forme::depuis_valeurs(&valeurs);
        (poser_profil(&avant, nom, poids, &forme)?, format!("Icebreaker : preset de forêt « {nom} » (atelier)"))
    } else {
        (retirer_profil(&avant, nom)?, format!("Icebreaker : preset de forêt « {nom} » retiré (atelier)"))
    };
    if apres == avant {
        return Ok(apres);
    }
    std::fs::write(FORMES, &apres).map_err(|e| format!("écriture de {FORMES} : {e}"))?;
    if let Err(e) = git(&["add", "--", FORMES]).and_then(|_| git(&["commit", "-q", "-m", &message, "--", FORMES])) {
        let _ = std::fs::write(FORMES, &avant);
        let _ = git(&["reset", "-q", "--", FORMES]);
        return Err(format!("commit refusé, fichier remis comme avant ({e})"));
    }
    println!("{message}");
    Ok(apres)
}
