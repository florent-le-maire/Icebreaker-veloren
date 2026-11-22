//! Icebreaker F.15 : le serveur de l'atelier des salles (`docs/icebreaker/PLAN-PLAYTEST.md`, chantier 1), lancé
//! par `.\icebreaker.ps1 salles` depuis la racine du dépôt. Il sert la page sur cette machine seulement, lui donne
//! tous les objets de Veloren (noms, modèles `.vox`), les salles et le catalogue des pièces, et enregistre une
//! salle : contrôlée par le lecteur du jeu, écrite dans `assets/icebreaker/donjons/salles/`, puis committée (ce
//! fichier seul). Même schéma que l'atelier des grottes (`icebreaker-caverne/src/bin/atelier.rs`).
//!
//! `GET /` la page ; `/objets` les sprites ; `/vox/<chemin>.vox` un modèle ; `/salles` les salles (les anciens
//! symboles de décor déjà changés en meubles) ; `/pieces` le catalogue ; `/reglages` les mesures d'une case.
//! `POST /salle` (première ligne : l'identifiant ; la suite : le fichier) et `POST /salle/supprimer`
//! (l'identifiant) répondent « ok » ou l'erreur.

use common::terrain::{SpriteKind, sprite};
use icebreaker_salles::Pieces;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::Command,
};
use world::icebreaker::dungeon::{
    Settings,
    plan::{self, Plan},
    rooms::{FURNITURE_CELL, FURNITURE_LEVEL, RoomCatalog, RoomRole, RoomTemplate},
    settings::DecorPlace,
};

const SALLES: &str = "assets/icebreaker/donjons/salles";
const MANIFESTE: &str = "assets/voxygen/voxel/sprite_manifest.ron";
const VOXELS: &str = "assets/voxygen/voxel";
const PAGE: &str = "icebreaker-salles/web";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.iter().position(|a| a == "--port").and_then(|i| args.get(i + 1)?.parse().ok()).unwrap_or(8097);
    if !Path::new(SALLES).exists() {
        eprintln!("Atelier des salles : {SALLES} introuvable, lancer depuis la racine du dépôt.");
        std::process::exit(1);
    }
    // The answers that never change while the workshop runs.
    let fixes = match Fixes::lire() {
        Ok(fixes) => std::sync::Arc::new(fixes),
        Err(e) => {
            eprintln!("Atelier des salles : {e}");
            std::process::exit(1);
        },
    };
    let ecoute = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(ecoute) => ecoute,
        Err(e) => {
            eprintln!("Atelier des salles : le port {port} est pris ({e}). Un atelier tourne peut-être déjà.");
            std::process::exit(1);
        },
    };
    let branche = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output();
    let branche = branche.map(|s| String::from_utf8_lossy(&s.stdout).trim().to_string()).unwrap_or_default();
    println!("Atelier des salles : http://localhost:{port} (Ctrl+C pour arrêter) ; les salles sont committées sur « {branche} »");
    for flux in ecoute.incoming().flatten() {
        let fixes = fixes.clone();
        std::thread::spawn(move || {
            let _ = servir(flux, port, &fixes);
        });
    }
}

/// What is read once: the sprites, the catalogue, the measures.
struct Fixes {
    objets: String,
    pieces: String,
    reglages: String,
    settings: Settings,
}

impl Fixes {
    fn lire() -> Result<Self, String> {
        let pieces = Pieces::load()?;
        let settings = Settings::load()?;
        let materiaux = world::icebreaker::dungeon::materiaux::Materiaux::load()?;
        let (c, h) = (settings.render.cell_size, settings.render.level_height);
        if (c, h) != (FURNITURE_CELL, FURNITURE_LEVEL) {
            return Err(format!("generation.ron : case de {c} × {h} blocs, les meubles sont écrits pour {FURNITURE_CELL} × {FURNITURE_LEVEL}"));
        }
        let roles: Vec<&str> = RoomRole::ALL.iter().map(|r| r.name()).collect();
        let reglages = json!({
            "case": c,
            "etage": h,
            "roles": roles,
            // Icebreaker F.29 : les matériaux du sol, des murs et du plafond, pour la fiche et la vue.
            "materiaux": materiaux,
        });
        Ok(Self {
            objets: objets(&pieces)?,
            pieces: serde_json::to_string(&pieces).map_err(|e| e.to_string())?,
            reglages: reglages.to_string(),
            settings,
        })
    }
}

/// An entry of the sprite manifest of Voxygen: only what the page draws.
#[derive(Deserialize)]
struct ConfigSprite {
    #[serde(default)]
    variations: Vec<Variation>,
}

#[derive(Deserialize)]
struct Variation {
    model: String,
    offset: (f32, f32, f32),
}

/// Every sprite of Veloren that has a model, as JSON: its name, its category, its models (file under
/// `assets/voxygen/voxel`, offset in voxels), its solid height in blocks, whether it turns and mirrors, and its
/// French name when the catalogue of the rooms gives one.
fn objets(pieces: &Pieces) -> Result<String, String> {
    let texte = std::fs::read_to_string(MANIFESTE).map_err(|e| format!("{MANIFESTE} : {e}"))?;
    let manifeste: HashMap<SpriteKind, Vec<ConfigSprite>> =
        ron::from_str(&texte).map_err(|e| format!("{MANIFESTE} : {e}"))?;
    let libelles: BTreeMap<&str, &str> = pieces
        .pieces
        .iter()
        .flat_map(|p| p.meubles.iter().rev())
        .map(|(libelle, sprite)| (sprite.as_str(), libelle.as_str()))
        .collect();
    let mut liste = Vec::new();
    for kind in SpriteKind::all() {
        let Some(variations) = manifeste.get(kind).and_then(|c| c.iter().find(|c| !c.variations.is_empty())) else {
            continue;
        };
        let nom = format!("{kind:?}");
        let modeles: Vec<_> = variations
            .variations
            .iter()
            .filter_map(|v| {
                // "voxygen.voxel.sprite.furniture.barrel" → "sprite/furniture/barrel.vox".
                let chemin = v.model.strip_prefix("voxygen.voxel.")?.replace('.', "/") + ".vox";
                Path::new(VOXELS).join(&chemin).exists().then(|| json!({ "vox": chemin, "offset": v.offset }))
            })
            .collect();
        if modeles.is_empty() {
            continue;
        }
        let categorie = kind.category();
        liste.push(json!({
            "nom": nom,
            "categorie": format!("{categorie:?}"),
            "modeles": modeles,
            "hauteur": kind.solid_height(),
            "ori": categorie.has_attr::<sprite::Ori>(),
            "miroir": categorie.has_attr::<sprite::MirrorX>(),
            "libelle": libelles.get(nom.as_str()),
        }));
    }
    Ok(serde_json::Value::Array(liste).to_string())
}

/// The rooms as JSON: `id`, `texte` (the file, its old decor symbols turned into `@meuble` lines), `convertie`
/// (whether that changed something: the file on disk still has symbols).
fn salles(settings: &Settings) -> Result<String, String> {
    let mut liste = Vec::new();
    let mut chemins: Vec<_> = std::fs::read_dir(SALLES)
        .map_err(|e| format!("{SALLES} : {e}"))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
        .collect();
    chemins.sort();
    for chemin in chemins {
        let id = chemin.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        let texte = std::fs::read_to_string(&chemin).map_err(|e| format!("{} : {e}", chemin.display()))?.replace('\r', "");
        let converti = convertir(&texte, settings)?;
        liste.push(json!({ "id": id, "convertie": converti != texte, "texte": converti }));
    }
    Ok(serde_json::Value::Array(liste).to_string())
}

/// The file of a room with its decor symbols (F.9, one piece per cell) turned into furniture placed by the
/// block (F.14), at the very place the game put them: against the wall the game chose, or in the middle of the
/// cell, chairs around a table. The cells become plain floor.
fn convertir(texte: &str, settings: &Settings) -> Result<String, String> {
    let plan = Plan::parse(texte)?;
    let Some((min_x, _, min_y, _)) = plan.bounds() else {
        return Ok(texte.to_string());
    };
    let c = FURNITURE_CELL;
    // Dir2::sprite_ori: toward +x 0, +y 2, -x 4, -y 6.
    let vers = |dx: i32, dy: i32| match (dx, dy) {
        (1, _) => 0,
        (-1, _) => 4,
        (_, 1) => 2,
        _ => 6,
    };
    let mut meubles = Vec::new();
    for (cell, symbole) in &plan.cells {
        let Some(item) = settings.decor.symbols.get(symbole) else { continue };
        let coin = ((cell.x - min_x) * c, (cell.y - min_y) * c, cell.level * FURNITURE_LEVEL);
        let mur = plan::DIRECTIONS.into_iter().find(|(dx, dy)| plan.get(cell.offset(*dx, *dy, 0)) == '#');
        match mur {
            Some((dx, dy)) if item.at == DecorPlace::Wall => {
                let le_long = |w: i32| if w > 0 { c - 1 } else if w < 0 { 0 } else { c / 2 };
                meubles.push(format!(
                    "@meuble: {} {} {} {} {}",
                    item.sprite,
                    coin.0 + le_long(dx),
                    coin.1 + le_long(dy),
                    coin.2,
                    vers(-dx, -dy)
                ));
            },
            _ => {
                let (x, y) = (coin.0 + c / 2, coin.1 + c / 2);
                meubles.push(format!("@meuble: {} {x} {y} {} 0", item.sprite, coin.2));
                if let Some(chaise) = &item.around {
                    for (dx, dy) in plan::DIRECTIONS {
                        meubles.push(format!("@meuble: {chaise} {} {} {} {}", x + dx, y + dy, coin.2, vers(-dx, -dy)));
                    }
                }
            },
        }
    }
    if meubles.is_empty() {
        return Ok(texte.to_string());
    }
    // The furniture goes after the last header; the symbols of the plan become floor.
    let lignes: Vec<&str> = texte.lines().collect();
    let dernier = lignes.iter().rposition(|l| l.starts_with('@')).map_or(0, |i| i + 1);
    let mut sortie = Vec::new();
    for (i, ligne) in lignes.iter().enumerate() {
        if i == dernier {
            sortie.extend(meubles.iter().cloned());
        }
        if ligne.starts_with([';', '@', '[']) {
            sortie.push(ligne.to_string());
        } else {
            sortie.push(ligne.chars().map(|s| if settings.decor.symbols.contains_key(&s) { '.' } else { s }).collect());
        }
    }
    Ok(sortie.join("\n") + "\n")
}

fn repondre(flux: &mut TcpStream, statut: &str, genre: &str, corps: &[u8]) -> std::io::Result<()> {
    write!(
        flux,
        "HTTP/1.1 {statut}\r\nContent-Type: {genre}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        corps.len()
    )?;
    flux.write_all(corps)
}

fn servir(mut flux: TcpStream, port: u16, fixes: &Fixes) -> std::io::Result<()> {
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
    let json = "application/json; charset=utf-8";
    match (methode, chemin) {
        ("GET", "/" | "/index.html") => match std::fs::read(format!("{PAGE}/index.html")) {
            Ok(corps) => repondre(&mut flux, "200 OK", "text/html; charset=utf-8", &corps),
            Err(e) => repondre(&mut flux, "404 Not Found", texte, format!("{PAGE}/index.html : {e}").as_bytes()),
        },
        ("GET", "/objets") => repondre(&mut flux, "200 OK", json, fixes.objets.as_bytes()),
        ("GET", "/pieces") => repondre(&mut flux, "200 OK", json, fixes.pieces.as_bytes()),
        ("GET", "/reglages") => repondre(&mut flux, "200 OK", json, fixes.reglages.as_bytes()),
        ("GET", "/salles") => match salles(&fixes.settings) {
            Ok(corps) => repondre(&mut flux, "200 OK", json, corps.as_bytes()),
            Err(e) => repondre(&mut flux, "500 Internal Server Error", texte, e.as_bytes()),
        },
        ("GET", chemin) if chemin.starts_with("/vox/") => {
            // Only a model of Veloren: lowercase names, no way out of the folder.
            let relatif = &chemin["/vox/".len()..];
            let sage = relatif.ends_with(".vox")
                && !relatif.contains("..")
                && relatif.chars().all(|c| c.is_ascii_alphanumeric() || "/_-.".contains(c));
            match sage.then(|| std::fs::read(Path::new(VOXELS).join(relatif))) {
                Some(Ok(corps)) => {
                    // The models never change while the workshop runs: the browser keeps them.
                    write!(
                        flux,
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nCache-Control: max-age=3600\r\nConnection: close\r\n\r\n",
                        corps.len()
                    )?;
                    flux.write_all(&corps)
                },
                _ => repondre(&mut flux, "404 Not Found", texte, "modèle introuvable".as_bytes()),
            }
        },
        ("POST", "/salle" | "/salle/supprimer") => {
            // Only the page of this server writes: not another site open in the browser.
            let ici = [format!("http://localhost:{port}"), format!("http://127.0.0.1:{port}")];
            if origine.is_some_and(|o| !ici.contains(&o)) || longueur > 200_000 {
                return repondre(&mut flux, "403 Forbidden", texte, "demande refusée".as_bytes());
            }
            let mut corps = vec![0; longueur];
            lecture.read_exact(&mut corps)?;
            let corps = String::from_utf8_lossy(&corps);
            let (id, fichier) = corps.split_once('\n').unwrap_or((&corps, ""));
            let fichier = (chemin == "/salle").then_some(fichier);
            match enregistrer(Path::new(SALLES), id.trim(), fichier, true) {
                Ok(()) => repondre(&mut flux, "200 OK", texte, "ok".as_bytes()),
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

/// What the game says of a room: read like the game reads it, in its eight orientations.
fn controler(id: &str, texte: &str) -> Result<(), String> {
    let salle = RoomTemplate::parse(id, texte)?;
    // Icebreaker F.29 : le sol, les murs et le plafond sont des noms de materiaux.ron.
    let erreurs = world::icebreaker::dungeon::materiaux::Materiaux::load()?.erreurs(&salle.habillage);
    if !erreurs.is_empty() {
        return Err(format!("salle « {id} » : {}", erreurs.join(" ; ")));
    }
    for orientation in 1..8 {
        salle.oriented(orientation)?;
    }
    Ok(())
}

/// The roles that have at least one room in `dossier`.
fn roles_servis(dossier: &Path) -> Result<Vec<RoomRole>, String> {
    let mut fichiers = Vec::new();
    for entree in std::fs::read_dir(dossier).map_err(|e| format!("{} : {e}", dossier.display()))?.flatten() {
        let chemin = entree.path();
        if chemin.extension().is_some_and(|e| e == "txt") {
            let id = chemin.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            fichiers.push((id, std::fs::read_to_string(&chemin).map_err(|e| e.to_string())?));
        }
    }
    let catalogue = RoomCatalog::parse(fichiers)?;
    Ok(RoomRole::ALL.into_iter().filter(|r| !catalogue.with_role(*r).is_empty()).collect())
}

/// Writes the room `id` in `dossier` (`fichier`: its text; none: removes it) and commits that file (`commit`);
/// refused, with nothing written, if the game can't read the room; undone if a role of the dungeon is left
/// without any room, or if the commit fails.
fn enregistrer(dossier: &Path, id: &str, fichier: Option<&str>, commit: bool) -> Result<(), String> {
    static UN_A_LA_FOIS: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _garde = UN_A_LA_FOIS.lock().unwrap_or_else(|e| e.into_inner());
    if id.is_empty() || id.len() > 40 || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err("nom de salle : des minuscules sans accent, des chiffres et « _ » (40 au plus)".into());
    }
    let chemin = dossier.join(format!("{id}.txt"));
    let avant = std::fs::read_to_string(&chemin).ok();
    let message = match fichier {
        Some(texte) => {
            controler(id, texte)?;
            if avant.as_deref().map(|a| a.replace('\r', "")) == Some(texte.to_string()) {
                return Ok(());
            }
            format!("Icebreaker : salle « {id} » (atelier)")
        },
        None if avant.is_none() => return Err(format!("salle « {id} » introuvable")),
        None => format!("Icebreaker : salle « {id} » retirée (atelier)"),
    };
    let servis = roles_servis(dossier)?;
    let remettre = || match &avant {
        Some(texte) => std::fs::write(&chemin, texte),
        None => std::fs::remove_file(&chemin),
    };
    let ecrit = match fichier {
        Some(texte) => std::fs::write(&chemin, texte),
        None => std::fs::remove_file(&chemin),
    };
    ecrit.map_err(|e| format!("{} : {e}", chemin.display()))?;
    let manquent: Vec<&str> = match roles_servis(dossier) {
        Ok(restent) => servis.iter().filter(|r| !restent.contains(r)).map(|r| r.name()).collect(),
        Err(e) => {
            let _ = remettre();
            return Err(e);
        },
    };
    if !manquent.is_empty() {
        let _ = remettre();
        return Err(format!("refusé : plus aucune salle ne servirait de {}", manquent.join(", ")));
    }
    if commit {
        let fichier = chemin.to_string_lossy().replace('\\', "/");
        if let Err(e) = git(&["add", "-A", "--", &fichier]).and_then(|_| git(&["commit", "-q", "-m", &message, "--", &fichier])) {
            let _ = remettre();
            let _ = git(&["reset", "-q", "--", &fichier]);
            return Err(format!("commit refusé, salle remise comme avant ({e})"));
        }
        println!("{message}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The old rooms open in the workshop: their symbols become furniture the game accepts, in every
    /// orientation, with as many pieces as the game placed (four chairs more around each table).
    #[test]
    fn les_anciennes_salles_se_convertissent() {
        // The tests run from the crate: the data is at the root of the repository.
        std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).unwrap();
        let settings = Settings::load().unwrap();
        let salle = "@roles: kitchen\n#######\n+.=..|+\n#.....#\n#######\n";
        let convertie = convertir(salle, &settings).unwrap();
        assert!(convertie.contains("@meuble: DiningtableWoodWoodlandRound 17 10 0 0\n"), "{convertie}");
        assert!(convertie.contains("@meuble: ChairWoodWoodland 18 10 0 4\n"), "{convertie}");
        assert!(convertie.contains("@meuble: ChairWoodWoodland 17 9 0 2\n"), "{convertie}");
        // The barrel is in the cell left of the east door: its wall is the north one (first found), it faces south.
        assert!(convertie.contains("@meuble: Barrel 38 7 0 2\n"), "{convertie}");
        assert!(convertie.ends_with("#######\n+.....+\n#.....#\n#######\n"), "{convertie}");
        controler("essai", &convertie).unwrap();
        assert_eq!(convertir(&convertie, &settings).unwrap(), convertie);

        let toutes: serde_json::Value = serde_json::from_str(&salles(&settings).unwrap()).unwrap();
        let toutes = toutes.as_array().unwrap();
        assert!(toutes.len() >= 30);
        for salle in toutes {
            let (id, texte) = (salle["id"].as_str().unwrap(), salle["texte"].as_str().unwrap());
            controler(id, texte).unwrap_or_else(|e| panic!("{id} : {e}"));
            let lue = RoomTemplate::parse(id, texte).unwrap();
            assert!(!lue.plan.cells.values().any(|s| plan::is_decor_symbol(*s)), "{id} garde des symboles");
        }
        assert!(toutes.iter().any(|s| s["convertie"] == true));
    }

    /// Every sprite the page lists has a model on the disk; the barrel is furniture that turns and mirrors.
    #[test]
    fn les_objets_ont_leur_modele() {
        std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).unwrap();
        let liste: serde_json::Value = serde_json::from_str(&objets(&Pieces::load().unwrap()).unwrap()).unwrap();
        let liste = liste.as_array().unwrap();
        assert!(liste.len() > 300, "{} objets", liste.len());
        let tonneau = liste.iter().find(|o| o["nom"] == "Barrel").unwrap();
        assert_eq!(tonneau["categorie"], "Furniture");
        assert_eq!(tonneau["modeles"][0]["vox"], "sprite/furniture/barrel.vox");
        assert_eq!((tonneau["ori"].as_bool(), tonneau["miroir"].as_bool()), (Some(true), Some(true)));
        assert_eq!(tonneau["libelle"], "tonneau");
        for objet in liste {
            for modele in objet["modeles"].as_array().unwrap() {
                assert!(Path::new(VOXELS).join(modele["vox"].as_str().unwrap()).exists());
            }
        }
    }

    /// A room the game can't read is never written; a good one is; the last room of a role can't be removed.
    #[test]
    fn une_salle_refusee_n_est_pas_ecrite() {
        let dossier = std::env::temp_dir().join(format!("icebreaker-atelier-salles-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        std::fs::create_dir_all(&dossier).unwrap();
        let bonne = "@roles: hall\n@meuble: Barrel 8 7 0 2\n#####\n+...+\n#...#\n#####\n";
        enregistrer(&dossier, "passage", Some(bonne), false).unwrap();
        assert_eq!(std::fs::read_to_string(dossier.join("passage.txt")).unwrap(), bonne);
        let dans_le_mur = bonne.replace("Barrel 8 7", "Barrel 1 1");
        let e = enregistrer(&dossier, "passage", Some(&dans_le_mur), false).unwrap_err();
        assert!(e.contains("hors du sol"), "{e}");
        assert_eq!(std::fs::read_to_string(dossier.join("passage.txt")).unwrap(), bonne);
        assert!(enregistrer(&dossier, "../ailleurs", Some(bonne), false).is_err());
        assert!(enregistrer(&dossier, "Passage", Some(bonne), false).is_err());
        let e = enregistrer(&dossier, "passage", None, false).unwrap_err();
        assert!(e.contains("plus aucune salle"), "{e}");
        assert!(dossier.join("passage.txt").exists());
        enregistrer(&dossier, "autre", Some(bonne), false).unwrap();
        enregistrer(&dossier, "passage", None, false).unwrap();
        assert!(!dossier.join("passage.txt").exists());
        let _ = std::fs::remove_dir_all(&dossier);
    }
}
