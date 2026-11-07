//! Icebreaker : le wiki en local, avec saisie (W.12) : `cargo run -p icebreaker-wiki -- serve`.
//!
//! Sert `site/` sur 127.0.0.1 ; les formulaires des pages envoient un POST qui modifie le fichier de
//! `docs/icebreaker/`, en fait un commit (ce fichier seul) et régénère le site. Serveur HTTP minimal en `std` :
//! une requête par connexion, un fil par connexion, les écritures l'une après l'autre.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::edition;

pub struct Options {
    pub port: u16,
    /// Nom écrit avec chaque saisie (« Réponse de Flo »).
    pub nom: String,
    /// Écrire même si le dossier n'est pas sur `master` (essais).
    pub branche_libre: bool,
}

struct Etat {
    racine: PathBuf,
    nom: String,
    /// Raison de refuser les écritures (pas sur `master`), sinon `None`.
    refus: Option<String>,
    /// Une écriture à la fois : fichier, commit, régénération.
    verrou: Mutex<()>,
}

/// Génère le site en mode saisie dans `racine/site`.
fn generer(racine: &Path) -> Result<usize> {
    let docs = racine.join("docs").join("icebreaker");
    let pages = crate::collecter(racine, &docs)?;
    crate::site::ecrire(&racine.join("site"), &pages, racine, true)
}

fn git(racine: &Path, args: &[&str]) -> Result<String> {
    let sortie = Command::new("git")
        .arg("-C")
        .arg(racine)
        .args(args)
        .output()
        .context("git introuvable")?;
    if !sortie.status.success() {
        bail!("git {} : {}", args.join(" "), String::from_utf8_lossy(&sortie.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&sortie.stdout).trim().to_string())
}

pub fn lancer(racine: &Path, options: Options) -> Result<()> {
    let branche = git(racine, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let refus = (!options.branche_libre && branche != "master").then(|| {
        format!(
            "ce dossier est sur la branche « {branche} » : la saisie n'écrit que sur master (lancer depuis \
             Veloren-test, ou --branche-libre pour un essai)"
        )
    });
    let nb = generer(racine)?;
    let ecouteur = TcpListener::bind(("127.0.0.1", options.port))
        .with_context(|| format!("port {} déjà pris ? (--port N)", options.port))?;
    println!("{nb} pages ; wiki sur http://localhost:{}/ (branche {branche})", options.port);
    match &refus {
        Some(r) => println!("saisie désactivée : {r}"),
        None => println!("saisie au nom de {} : chaque envoi modifie docs/icebreaker/ et fait un commit", options.nom),
    }
    let etat = Arc::new(Etat {
        racine: racine.to_path_buf(),
        nom: options.nom,
        refus,
        verrou: Mutex::new(()),
    });
    for flux in ecouteur.incoming() {
        let Ok(flux) = flux else { continue };
        let etat = Arc::clone(&etat);
        std::thread::spawn(move || {
            if let Err(e) = servir(flux, &etat) {
                eprintln!("{e:#}");
            }
        });
    }
    Ok(())
}

struct Requete {
    methode: String,
    chemin: String,
    corps: String,
}

fn lire_requete(flux: &TcpStream) -> Result<Requete> {
    let mut lecteur = BufReader::new(flux);
    let mut ligne = String::new();
    lecteur.read_line(&mut ligne)?;
    let mut morceaux = ligne.split_whitespace();
    let methode = morceaux.next().unwrap_or("").to_string();
    let chemin = morceaux.next().unwrap_or("/").to_string();
    let mut longueur = 0usize;
    loop {
        let mut entete = String::new();
        if lecteur.read_line(&mut entete)? == 0 || entete.trim().is_empty() {
            break;
        }
        if let Some((cle, valeur)) = entete.split_once(':')
            && cle.trim().eq_ignore_ascii_case("content-length")
        {
            longueur = valeur.trim().parse().unwrap_or(0);
        }
    }
    let mut corps = vec![0; longueur.min(1 << 20)];
    lecteur.read_exact(&mut corps)?;
    Ok(Requete { methode, chemin, corps: String::from_utf8_lossy(&corps).into_owned() })
}

/// Décode `%XX` (et `+` en espace si `plus`), en UTF-8.
fn decoder(texte: &str, plus: bool) -> String {
    let octets = texte.as_bytes();
    let mut sortie = Vec::with_capacity(octets.len());
    let mut i = 0;
    while i < octets.len() {
        match octets[i] {
            b'+' if plus => sortie.push(b' '),
            b'%' if i + 2 < octets.len() => {
                let chiffre = |o: u8| (o as char).to_digit(16);
                match (chiffre(octets[i + 1]), chiffre(octets[i + 2])) {
                    (Some(a), Some(b)) => {
                        sortie.push((a * 16 + b) as u8);
                        i += 2;
                    },
                    _ => sortie.push(b'%'),
                }
            },
            o => sortie.push(o),
        }
        i += 1;
    }
    String::from_utf8_lossy(&sortie).into_owned()
}

/// Les champs d'un formulaire `application/x-www-form-urlencoded`.
fn champs(corps: &str) -> HashMap<String, String> {
    corps
        .split('&')
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (decoder(k, true), decoder(v, true)))
        .collect()
}

fn type_de(chemin: &str) -> &'static str {
    match chemin.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn repondre_http(flux: &mut TcpStream, statut: &str, entetes: &[(&str, &str)], corps: &[u8]) -> Result<()> {
    let mut tete = format!("HTTP/1.1 {statut}\r\nContent-Length: {}\r\nConnection: close\r\n", corps.len());
    for (k, v) in entetes {
        tete.push_str(&format!("{k}: {v}\r\n"));
    }
    tete.push_str("\r\n");
    flux.write_all(tete.as_bytes())?;
    flux.write_all(corps)?;
    Ok(())
}

fn echapper(texte: &str) -> String {
    texte.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn page_erreur(flux: &mut TcpStream, statut: &str, message: &str, retour: &str) -> Result<()> {
    let corps = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Wiki : erreur</title><link rel=\"stylesheet\" href=\"/style.css\">\
         <body style=\"padding:2rem\"><h1>Rien n'a été écrit</h1><p>{}</p><p><a href=\"{retour}\">Retour</a></p></body>",
        echapper(message)
    );
    repondre_http(flux, statut, &[("Content-Type", "text/html; charset=utf-8")], corps.as_bytes())
}

fn servir(mut flux: TcpStream, etat: &Etat) -> Result<()> {
    flux.set_read_timeout(Some(Duration::from_secs(10)))?;
    let requete = lire_requete(&flux)?;
    let chemin = requete.chemin.split(['?', '#']).next().unwrap_or("/");
    match requete.methode.as_str() {
        "GET" | "HEAD" => servir_fichier(&mut flux, etat, chemin),
        "POST" => {
            let retour = if chemin.starts_with("/todo/") {
                "/A-FAIRE.html"
            } else if chemin.starts_with("/en-jeu/") {
                "/a-voir.html"
            } else {
                "/QUESTIONS.html"
            };
            if let Some(refus) = &etat.refus {
                return page_erreur(&mut flux, "403 Forbidden", refus, retour);
            }
            let champs = champs(&requete.corps);
            let _garde = etat.verrou.lock().unwrap_or_else(|e| e.into_inner());
            match saisir(etat, chemin, &champs) {
                Ok(vers) => repondre_http(&mut flux, "303 See Other", &[("Location", &vers)], b""),
                Err(e) => page_erreur(&mut flux, "400 Bad Request", &format!("{e:#}"), retour),
            }
        },
        _ => repondre_http(&mut flux, "405 Method Not Allowed", &[], b""),
    }
}

fn servir_fichier(flux: &mut TcpStream, etat: &Etat, chemin: &str) -> Result<()> {
    let mut relatif = decoder(chemin.trim_start_matches('/'), false);
    if relatif.is_empty() || relatif.ends_with('/') {
        relatif.push_str("index.html");
    }
    if relatif.split(['/', '\\']).any(|p| p == "..") {
        return repondre_http(flux, "404 Not Found", &[], b"");
    }
    match std::fs::read(etat.racine.join("site").join(&relatif)) {
        Ok(contenu) => repondre_http(
            flux,
            "200 OK",
            &[("Content-Type", type_de(&relatif)), ("Cache-Control", "no-store")],
            &contenu,
        ),
        Err(_) => repondre_http(flux, "404 Not Found", &[("Content-Type", "text/plain; charset=utf-8")], b"introuvable"),
    }
}

/// Applique une saisie : modifie le fichier, commit, régénère ; renvoie l'adresse où revenir.
fn saisir(etat: &Etat, chemin: &str, champs: &HashMap<String, String>) -> Result<String> {
    let champ = |nom: &str| champs.get(nom).map(String::as_str).unwrap_or("");
    let date = edition::aujourd_hui();
    let nom = etat.nom.as_str();
    let (fichier, vers) = if chemin.starts_with("/questions/") {
        ("QUESTIONS.md", format!("/QUESTIONS.html#{}", champ("numero")))
    } else if chemin.starts_with("/todo/") || chemin == "/en-jeu/casse" {
        ("A-FAIRE.md", if chemin == "/en-jeu/casse" { "/a-voir.html" } else { "/A-FAIRE.html" }.to_string())
    } else if chemin == "/en-jeu/vu" {
        // Seulement les plans : `PLAN.md`, `HISTORIQUE.md`, `PLAN-*.md`.
        let f = champ("fichier");
        let plan = f == "PLAN.md" || f == "HISTORIQUE.md" || (f.starts_with("PLAN-") && f.ends_with(".md"));
        if !plan || f.contains(['/', '\\']) {
            bail!("fichier de plan refusé : {f}");
        }
        (f, "/a-voir.html".to_string())
    } else {
        bail!("adresse inconnue : {chemin}");
    };
    let index = || champ("index").parse::<usize>().map_err(|_| anyhow::anyhow!("index de todo invalide"));
    let resume = |texte: &str| -> String {
        let court: String = texte.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(60).collect();
        court.trim_start_matches("[ ] ").to_string()
    };
    let relatif = format!("docs/icebreaker/{fichier}");
    let chemin_fichier = etat.racine.join(&relatif);
    let avant = std::fs::read_to_string(&chemin_fichier).unwrap_or_default();
    let numero = champ("numero");
    let (apres, message) = match chemin {
        "/questions/repondre" => (
            edition::repondre(&avant, numero, champ("reponse"), nom, &date)?,
            format!("Icebreaker wiki : réponse de {nom} à {numero}\n\n{}", champ("reponse").trim()),
        ),
        "/questions/obsolete" => (
            edition::rendre_obsolete(&avant, numero, nom, &date)?,
            format!("Icebreaker wiki : {numero} obsolète ({nom})"),
        ),
        "/questions/supprimer" => (
            edition::supprimer_question(&avant, numero)?,
            format!("Icebreaker wiki : {numero} supprimée ({nom})"),
        ),
        "/todo/ajouter" => (
            edition::ajouter_todo(&avant, champ("texte"), nom, &date)?,
            format!("Icebreaker wiki : todo de {nom}\n\n{}", champ("texte").trim()),
        ),
        "/todo/cocher" => (
            edition::cocher_todo(&avant, index()?, champ("texte"), nom, &date)?,
            format!("Icebreaker wiki : todo fait ({nom})\n\n{}", resume(champ("texte"))),
        ),
        "/todo/supprimer" => (
            edition::supprimer_todo(&avant, index()?, champ("texte"))?,
            format!("Icebreaker wiki : todo supprimé ({nom})\n\n{}", resume(champ("texte"))),
        ),
        "/en-jeu/vu" => (
            edition::marquer_vu(&avant, champ("id"), champ("texte"), nom, &date)?,
            format!("Icebreaker wiki : {} vu en jeu ({nom})\n\n{}", champ("id"), champ("texte").trim()),
        ),
        "/en-jeu/casse" => {
            let texte = champ("texte").trim();
            if texte.is_empty() {
                bail!("dire ce qui casse dans le commentaire");
            }
            let entree = format!("[en jeu] {} {} : {texte}", champ("id"), champ("titre"));
            (
                edition::ajouter_todo(&avant, &entree, nom, &date)?,
                format!("Icebreaker wiki : {} casse en jeu ({nom})\n\n{texte}", champ("id")),
            )
        },
        _ => bail!("adresse inconnue : {chemin}"),
    };
    std::fs::write(&chemin_fichier, &apres).with_context(|| format!("écriture de {relatif}"))?;
    let commit = git(&etat.racine, &["add", "--", &relatif])
        .and_then(|_| git(&etat.racine, &["commit", "-q", "-m", &message, "--", &relatif]));
    if let Err(e) = commit {
        std::fs::write(&chemin_fichier, &avant)?;
        return Err(e.context("commit refusé, fichier remis comme avant"));
    }
    println!("{}", message.lines().next().unwrap_or(""));
    generer(&etat.racine)?;
    Ok(vers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodage_des_formulaires() {
        let c = champs("numero=Q12&reponse=Oui%2C+on+garde+%C3%A9t%C3%A9+100%25");
        assert_eq!(c["numero"], "Q12");
        assert_eq!(c["reponse"], "Oui, on garde été 100%");
        assert_eq!(decoder("fiches/m%C3%A9t+a.html", false), "fiches/mét+a.html");
        assert_eq!(decoder("50%", true), "50%");
        assert_eq!(decoder("%zz", true), "%zz");
    }
}
