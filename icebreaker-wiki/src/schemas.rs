//! Les schémas des fiches : du SVG dessiné depuis les valeurs des `.ron`, pour que le dessin suive les réglages.
//! Une fiche déclare `schema = "frise-mort"` dans son en-tête ; chaque schéma sait quel fichier il lit.

use regex::Regex;

use crate::reglages::{Entree, Fichier};

/// Un schéma prêt pour le gabarit : le SVG, sa légende, et les icônes de Veloren qu'il affiche (sous `assets/`).
pub struct Schema {
    pub svg: String,
    pub legende: String,
    pub images: Vec<String>,
}

const TOWN: &str = "voxygen/element/ui/map/buttons/town.png";
const GNARLING: &str = "voxygen/element/ui/map/buttons/gnarling.png";
const DUNGEON: &str = "voxygen/element/ui/map/buttons/dungeon.png";
const CAVE: &str = "voxygen/element/ui/map/buttons/cave.png";
const SKULL: &str = "voxygen/element/ui/generic/icons/skull_2.png";
const CHARACTER: &str = "voxygen/element/ui/map/buttons/character.png";
const BAG: &str = "voxygen/element/items/item_bag_leather_large.png";

/// Les noms de schéma connus.
pub const NOMS: [&str; 7] = [
    "frise-mort",
    "carte-campagne",
    "circuit-charrette",
    "etages-donjon",
    "coupe-grotte",
    "portee-verrouillage",
    "plan-village",
];

/// Dessine le schéma `nom` ; `racine` est le préfixe vers la racine du site (pour les icônes).
pub fn dessiner(nom: &str, reglages: &[Fichier], racine: &str) -> Option<Schema> {
    let fichier = |chemin: &str| reglages.iter().find(|f| f.chemin == chemin);
    match nom {
        "frise-mort" => Some(frise_mort(fichier("mort.ron")?, racine)),
        "carte-campagne" => Some(carte_campagne(fichier("campagne.ron")?, racine)),
        "circuit-charrette" => Some(circuit_charrette(fichier("registre.ron")?, racine)),
        "etages-donjon" => Some(etages_donjon(fichier("donjons/generation.ron")?)),
        "coupe-grotte" => Some(coupe_grotte(fichier("grottes/grotte.ron")?, racine)),
        "portee-verrouillage" => Some(portee_verrouillage(fichier("verrouillage.ron")?, fichier("combat.ron")?, racine)),
        "plan-village" => Some(plan_village(fichier("village.ron")?)),
        _ => None,
    }
}

/// Cherche `cle` dans l'arbre des entrées, y compris dans la valeur texte d'un groupe écrit sur une ligne
/// (`entree: (profondeur: 18, …)`) ; renvoie le ou les nombres qui suivent (`30.0` → (30, None),
/// `(6, 10)` → (6, Some(10))).
pub fn nombres(entrees: &[Entree], cle: &str) -> Option<(f64, Option<f64>)> {
    let motif = Regex::new(&format!(r"\b{}:\s*\(?\s*(-?[\d.]+)(?:\s*,\s*(-?[\d.]+))?", regex::escape(cle))).ok()?;
    for e in entrees {
        if e.cle == cle {
            if let Some(c) = motif.captures(&format!("{cle}: {}", e.valeur)) {
                return Some((c[1].parse().ok()?, c.get(2).and_then(|m| m.as_str().parse().ok())));
            }
        }
        if let Some(c) = motif.captures(&e.valeur) {
            return Some((c[1].parse().ok()?, c.get(2).and_then(|m| m.as_str().parse().ok())));
        }
        if let Some(trouve) = nombres(&e.enfants, cle) {
            return Some(trouve);
        }
    }
    None
}

/// Le sous-ensemble d'un fichier : l'entrée `cle` seule (ses enfants, ou sa valeur si le groupe tient sur une ligne),
/// pour chercher dedans sans tomber sur une clé homonyme d'un autre groupe.
fn groupe(f: &Fichier, cle: &str) -> Fichier {
    fn chercher(entrees: &[Entree], cle: &str) -> Option<Entree> {
        entrees
            .iter()
            .find(|e| e.cle == cle)
            .cloned()
            .or_else(|| entrees.iter().find_map(|e| chercher(&e.enfants, cle)))
    }
    let entrees = match chercher(&f.entrees, cle) {
        Some(e) if !e.enfants.is_empty() => e.enfants,
        Some(e) => vec![Entree { cle: String::new(), ..e }],
        None => Vec::new(),
    };
    Fichier { chemin: f.chemin.clone(), entete: String::new(), entrees }
}

fn nombre(f: &Fichier, cle: &str, defaut: f64) -> f64 {
    nombres(&f.entrees, cle).map(|(a, _)| a).unwrap_or(defaut)
}

fn plage(f: &Fichier, cle: &str, defaut: (f64, f64)) -> (f64, f64) {
    match nombres(&f.entrees, cle) {
        Some((a, Some(b))) => (a, b),
        Some((a, None)) => (a, a),
        None => defaut,
    }
}

fn entier(n: f64) -> String {
    if (n - n.round()).abs() < 1e-9 {
        format!("{}", n.round() as i64)
    } else {
        format!("{n}")
    }
}

fn plage_texte(p: (f64, f64)) -> String {
    if p.0 == p.1 {
        entier(p.0)
    } else {
        format!("{} à {}", entier(p.0), entier(p.1))
    }
}

fn image(racine: &str, chemin: &str, x: f64, y: f64, taille: f64) -> String {
    format!(
        "<image href=\"{racine}assets/{chemin}\" x=\"{x}\" y=\"{y}\" width=\"{taille}\" height=\"{taille}\" class=\"pixel\"/>"
    )
}

fn texte(x: f64, y: f64, classe: &str, contenu: &str) -> String {
    format!("<text x=\"{x}\" y=\"{y}\" class=\"{classe}\">{}</text>", echapper(contenu))
}

fn echapper(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn boite(x: f64, y: f64, l: f64, h: f64, classe: &str) -> String {
    format!("<rect x=\"{x}\" y=\"{y}\" width=\"{l}\" height=\"{h}\" rx=\"8\" class=\"{classe}\"/>")
}

fn fleche(x1: f64, y1: f64, x2: f64, y2: f64, classe: &str) -> String {
    format!("<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" class=\"{classe}\" marker-end=\"url(#sch-pointe)\"/>")
}

fn svg(largeur: f64, hauteur: f64, corps: &str) -> String {
    format!(
        "<svg viewBox=\"0 0 {largeur} {hauteur}\" class=\"sch\" role=\"img\"><defs>\
<marker id=\"sch-pointe\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\">\
<path d=\"M0 0L10 5 0 10z\" class=\"sch-pointe\"/></marker></defs>{corps}</svg>"
    )
}

/// À terre, relève, mort, sac : une frise de gauche à droite.
fn frise_mort(mort: &Fichier, racine: &str) -> Schema {
    let delai = nombre(mort, "delai_a_terre", 30.0);
    let pv = nombre(mort, "pv_releve", 0.25) * 100.0;
    let fouille = nombre(mort, "duree_fouille", 3.0);
    let mut c = String::new();
    // Ligne du temps.
    c += &fleche(60.0, 120.0, 690.0, 120.0, "sch-ligne");
    c += &texte(60.0, 150.0, "sch-petit", "0 PV : on tombe");
    c += "<circle cx=\"60\" cy=\"120\" r=\"6\" class=\"sch-point\"/>";
    // Barre « à terre ».
    c += &boite(60.0, 100.0, 330.0, 40.0, "sch-barre-chaud");
    c += &texte(225.0, 125.0, "sch-centre sch-fort", &format!("à terre : {} s à ramper", entier(delai)));
    // Relève, par-dessus.
    c += &boite(130.0, 20.0, 230.0, 44.0, "sch-boite-ok");
    c += &texte(245.0, 39.0, "sch-centre sch-fort", "un allié relève (E)");
    c += &texte(245.0, 56.0, "sch-centre sch-petit", &format!("debout avec {} % des PV", entier(pv)));
    c += &fleche(245.0, 100.0, 245.0, 68.0, "sch-ligne-ok");
    // Mort.
    c += "<circle cx=\"390\" cy=\"120\" r=\"7\" class=\"sch-point-danger\"/>";
    c += &texte(390.0, 150.0, "sch-centre sch-danger", "mort");
    c += &texte(390.0, 166.0, "sch-centre sch-petit", "le personnage est perdu");
    // Sac du mort.
    c += &boite(470.0, 88.0, 210.0, 64.0, "sch-boite");
    c += &image(racine, BAG, 482.0, 100.0, 40.0);
    c += &texte(530.0, 112.0, "sch-fort", "sac du mort au sol");
    c += &texte(530.0, 130.0, "sch-petit", &format!("tout l'inventaire ; fouillé en {} s", entier(fouille)));
    c += &texte(530.0, 145.0, "sch-petit", "par un allié");
    Schema {
        svg: svg(720.0, 176.0, &c),
        legende: format!(
            "Tomber à terre laisse {} s à un allié pour relever ({} % des PV) ; sinon on meurt et le sac reste au sol, \
fouillé en {} s.",
            entier(delai),
            entier(pv),
            entier(fouille)
        ),
        images: vec![BAG.to_string()],
    }
}

/// Le village au centre, les emplacements de donjon autour, chacun dans son cercle fermé.
fn carte_campagne(campagne: &Fichier, racine: &str) -> Schema {
    let motif = Regex::new(r#"\(\s*"(\w+)"\s*,\s*(\d+)\s*\)"#).expect("regex");
    let valeur = nombres_bruts(&campagne.entrees, "emplacements");
    let mut lieux: Vec<(String, usize)> = motif
        .captures_iter(&valeur)
        .map(|c| (c[1].to_string(), c[2].parse().unwrap_or(1)))
        .collect();
    if lieux.is_empty() {
        lieux.push(("Donjon".into(), 1));
    }
    let noeuds: Vec<&str> = lieux.iter().flat_map(|(n, k)| std::iter::repeat_n(n.as_str(), *k)).collect();
    let mut images = vec![TOWN.to_string()];
    let mut c = String::new();
    let (cx, cy) = (360.0, 190.0);
    // Zone du village.
    c += &format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"58\" class=\"sch-zone\"/>");
    c += &image(racine, TOWN, cx - 22.0, cy - 30.0, 44.0);
    c += &texte(cx, cy + 36.0, "sch-centre sch-fort", "village");
    c += &texte(cx, cy + 52.0, "sch-centre sch-petit", "la base, au centre");
    let n = noeuds.len() as f64;
    for (i, nom) in noeuds.iter().enumerate() {
        let angle = -std::f64::consts::FRAC_PI_2 + i as f64 * std::f64::consts::TAU / n;
        let (x, y) = (cx + 280.0 * angle.cos(), cy + 130.0 * angle.sin());
        let (icone, libelle) = match *nom {
            "Gnarling" => (GNARLING, "donjon Gnarling (Veloren)"),
            "Genere" => (DUNGEON, "donjon généré (le nôtre)"),
            "Grotte" => (CAVE, "grotte"),
            autre => (DUNGEON, autre),
        };
        if !images.iter().any(|i| i == icone) {
            images.push(icone.to_string());
        }
        c += &format!("<line x1=\"{cx}\" y1=\"{cy}\" x2=\"{x}\" y2=\"{y}\" class=\"sch-pointille\"/>");
        c += &format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"40\" class=\"sch-zone\"/>");
        c += &image(racine, icone, x - 16.0, y - 24.0, 32.0);
        c += &texte(x, y + 24.0, "sch-centre sch-petit", libelle);
    }
    c += &texte(cx, cy - 72.0, "sch-centre sch-petit", "téléportation à l'aller, /rentrer au retour");
    Schema {
        svg: svg(720.0, 380.0, &c),
        legende: format!(
            "Un seul monde : le village au centre et {} emplacements loin autour ({}). Chaque cercle est une zone \
fermée ; un emplacement occupé est remis à neuf quand tout le monde en est parti.",
            noeuds.len(),
            lieux
                .iter()
                .map(|(n, k)| format!("{k} {n}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        images,
    }
}

/// La valeur texte d'une clé (liste repliée sur une ligne), vide si absente.
fn nombres_bruts(entrees: &[Entree], cle: &str) -> String {
    for e in entrees {
        if e.cle == cle {
            return e.valeur.clone();
        }
        let v = nombres_bruts(&e.enfants, cle);
        if !v.is_empty() {
            return v;
        }
    }
    String::new()
}

/// Fantôme, charrette, registre, expédition : le circuit d'un personnage.
fn circuit_charrette(registre: &Fichier, racine: &str) -> Schema {
    let par_jour = nombre(registre, "recrues_par_jour", 3.0);
    let minimum = nombre(registre, "minimum_charrette", 2.0);
    let places = nombre(registre, "places_registre", 10.0);
    let reservations = nombre(registre, "reservations_par_compte", 5.0);
    let mut c = String::new();
    let y = 60.0;
    let etapes = [
        (20.0, "fantôme", "on arrive sans corps", CHARACTER),
        (200.0, "charrette", "les recrues du jour", BAG),
        (380.0, "registre", "les personnages de l'équipe", TOWN),
        (560.0, "expédition", "un joueur par personnage", GNARLING),
    ];
    for (i, (x, titre, sous, icone)) in etapes.iter().enumerate() {
        c += &boite(*x, y, 140.0, 70.0, "sch-boite");
        c += &image(racine, icone, x + 10.0, y + 19.0, 32.0);
        c += &texte(x + 50.0, y + 30.0, "sch-fort", titre);
        c += &texte(x + 50.0, y + 48.0, "sch-petit", sous);
        if i + 1 < etapes.len() {
            c += &fleche(x + 142.0, y + 35.0, x + 178.0, y + 35.0, "sch-ligne");
        }
    }
    c += &texte(170.0, y + 26.0, "sch-centre sch-petit", "engage");
    c += &texte(350.0, y + 26.0, "sch-centre sch-petit", "inscrit");
    c += &texte(530.0, y + 26.0, "sch-centre sch-petit", "prend");
    c += &texte(270.0, 178.0, "sch-centre sch-petit", &format!("{} recrues par jour de carte, au moins {}", entier(par_jour), entier(minimum)));
    c += &texte(450.0, 160.0, "sch-centre sch-petit", &format!("{} places, {} réservées par joueur", entier(places), entier(reservations)));
    // Retours.
    c += &format!("<path d=\"M630 132 C630 225 450 225 450 132\" class=\"sch-ligne-ok\" fill=\"none\" marker-end=\"url(#sch-pointe)\"/>");
    c += &texte(540.0, 218.0, "sch-centre sch-ok", "rentre vivant : retour au registre");
    c += &fleche(630.0, 58.0, 630.0, 30.0, "sch-ligne-danger");
    c += &texte(630.0, 20.0, "sch-centre sch-danger", "meurt : le personnage est perdu");
    Schema {
        svg: svg(740.0, 232.0, &c),
        legende: format!(
            "Un personnage appartient à l'équipe : engagé à la charrette ({} par jour), inscrit au registre ({} places), \
pris par un joueur à la fois ; mort, il quitte le registre.",
            entier(par_jour),
            entier(places)
        ),
        images: vec![CHARACTER.into(), BAG.into(), TOWN.into(), GNARLING.into()],
    }
}

/// Petit, moyen, grand : les étages et les salles de chaque taille de donjon.
fn etages_donjon(generation: &Fichier) -> Schema {
    let tailles = ["small", "medium", "large"];
    let noms = ["petit", "moyen", "grand"];
    let hauteur_etage = nombre(generation, "level_height", 7.0);
    let case = nombre(generation, "cell_size", 7.0);
    let mut c = String::new();
    let mut legende_parts = Vec::new();
    for (i, (taille, nom)) in tailles.iter().zip(noms).enumerate() {
        let sous = groupe(&groupe(generation, "sizes"), taille);
        if sous.entrees.is_empty() {
            continue;
        }
        let etages = plage(&sous, "levels", (1.0, 1.0));
        let salles = plage(&sous, "main_path", (4.0, 5.0));
        let branches = plage(&sous, "branches", (0.0, 0.0));
        let verrous = plage(&sous, "locks", (0.0, 0.0));
        let x = 30.0 + i as f64 * 240.0;
        c += &texte(x + 90.0, 24.0, "sch-centre sch-fort", nom);
        let n = etages.1 as usize;
        for k in 0..n {
            let y = 40.0 + k as f64 * 34.0;
            let classe = if (k as f64) < etages.0 { "sch-etage" } else { "sch-etage-option" };
            c += &boite(x, y, 180.0, 28.0, classe);
            c += &texte(x + 10.0, y + 19.0, "sch-petit", &format!("étage {}", k + 1));
        }
        let bas = 40.0 + n as f64 * 34.0;
        c += &texte(x, bas + 16.0, "sch-petit", &format!("{} étages", plage_texte(etages)));
        c += &texte(x, bas + 34.0, "sch-petit", &format!("{} salles sur le chemin principal", plage_texte(salles)));
        c += &texte(x, bas + 52.0, "sch-petit", &format!("{} branches, {} verrous", plage_texte(branches), plage_texte(verrous)));
        legende_parts.push(format!("{nom} : {} salles", plage_texte(salles)));
    }
    Schema {
        svg: svg(740.0, 260.0, &c),
        legende: format!(
            "Les étages pleins sont toujours là, les clairs seulement parfois ({}). Un étage fait {} blocs de haut, une case {} blocs.",
            legende_parts.join(" ; "),
            entier(hauteur_etage),
            entier(case)
        ),
        images: Vec::new(),
    }
}

/// Coupe de la grotte : la bouche, le boyau d'entrée, les niveaux de poches, le chef au fond.
fn coupe_grotte(grotte: &Fichier, racine: &str) -> Schema {
    let (entree, poche, reseau) = (groupe(grotte, "entree"), groupe(grotte, "poche"), groupe(grotte, "reseau"));
    let profondeur = nombre(&entree, "profondeur", 18.0);
    let niveaux = nombre(&reseau, "niveaux", 3.0) as usize;
    let ecart = nombre(&reseau, "ecart_niveau", 16.0);
    let rayon = plage(&poche, "rayon", (10.0, 14.0));
    let hauteur = plage(&poche, "hauteur", (5.0, 7.0));
    let poches = plage(&reseau, "poches", (6.0, 10.0));
    let echelle = 3.6;
    let surface = 30.0;
    let mut c = String::new();
    c += &format!("<line x1=\"0\" y1=\"{surface}\" x2=\"720\" y2=\"{surface}\" class=\"sch-ligne\"/>");
    c += &texte(8.0, surface - 8.0, "sch-petit", "surface");
    let x0 = 120.0;
    let y1 = surface + profondeur * echelle;
    c += &format!("<path d=\"M{x0} {surface} L{} {y1}\" class=\"sch-boyau\"/>", x0 + 40.0);
    c += &texte(x0 + 8.0, surface + 40.0, "sch-petit", &format!("boyau d'entrée, {} blocs", entier(profondeur)));
    let (rx, ry) = (rayon.1 * echelle, hauteur.1 * echelle);
    let mut precedent = (x0 + 40.0, y1);
    for k in 0..niveaux {
        let y = y1 + k as f64 * ecart * echelle;
        let xs: &[f64] = match k % 2 {
            0 => &[160.0, 420.0],
            _ => &[290.0, 560.0],
        };
        for (j, x) in xs.iter().enumerate() {
            if k > 0 || j > 0 {
                c += &format!("<path d=\"M{} {} L{x} {y}\" class=\"sch-boyau\"/>", precedent.0, precedent.1);
            }
            c += &format!("<ellipse cx=\"{x}\" cy=\"{y}\" rx=\"{rx}\" ry=\"{ry}\" class=\"sch-poche\"/>");
            precedent = (*x, y);
        }
        c += &texte(690.0, y + 4.0, "sch-fin sch-petit", &format!("niveau {}", k + 1));
        if k + 1 < niveaux {
            c += &format!("<line x1=\"700\" y1=\"{}\" x2=\"700\" y2=\"{}\" class=\"sch-cote\"/>", y + ry, y + ecart * echelle - ry);
            c += &texte(706.0, y + ecart * echelle / 2.0 + 4.0, "sch-petit", &format!("{}", entier(ecart)));
        }
    }
    c += &image(racine, SKULL, precedent.0 - 12.0, precedent.1 - 12.0, 24.0);
    c += &texte(precedent.0 + 18.0, precedent.1 + 4.0, "sch-danger", "le chef, dans la poche la plus profonde");
    let hauteur_svg = y1 + (niveaux as f64 - 1.0) * ecart * echelle + ry + 20.0;
    Schema {
        svg: svg(760.0, hauteur_svg, &c),
        legende: format!(
            "{} poches sur {} niveaux, espacés de {} blocs ; chaque poche fait {} blocs de rayon et {} de demi-hauteur. \
Le chef attend au fond.",
            plage_texte(poches),
            niveaux,
            entier(ecart),
            plage_texte(rayon),
            plage_texte(hauteur)
        ),
        images: vec![SKULL.into()],
    }
}

/// Les deux rayons du verrouillage autour du joueur, et les multiplicateurs de dégâts.
fn portee_verrouillage(verrouillage: &Fichier, combat: &Fichier, racine: &str) -> Schema {
    let rayon = nombre(verrouillage, "rayon", 20.0);
    let perte = nombre(verrouillage, "rayon_perte", 30.0);
    let creatures = nombre(combat, "degats_contre_creatures", 5.0);
    let joueurs = nombre(combat, "degats_contre_joueurs", 9.0);
    let echelle = 4.6;
    let (cx, cy) = (170.0, 160.0);
    let mut c = String::new();
    c += &format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{}\" class=\"sch-zone-loin\"/>", perte * echelle);
    c += &format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{}\" class=\"sch-zone\"/>", rayon * echelle);
    c += &image(racine, CHARACTER, cx - 14.0, cy - 14.0, 28.0);
    c += &fleche(cx, cy, cx + rayon * echelle - 2.0, cy, "sch-ligne");
    c += &texte(cx + rayon * echelle / 2.0, cy - 8.0, "sch-centre sch-petit", &format!("verrouille à {} blocs", entier(rayon)));
    c += &format!("<line x1=\"{cx}\" y1=\"{cy}\" x2=\"{}\" y2=\"{}\" class=\"sch-pointille\"/>", cx + perte * echelle * 0.7071, cy + perte * echelle * 0.7071);
    c += &texte(cx + perte * echelle * 0.5, cy + perte * echelle * 0.5 + 16.0, "sch-petit", &format!("saute à {} blocs", entier(perte)));
    // Multiplicateurs de dégâts.
    let x = 400.0;
    c += &texte(x, 60.0, "sch-fort", "dégâts multipliés");
    let maxi = creatures.max(joueurs).max(1.0);
    for (i, (nom, val, classe)) in [("contre les créatures", creatures, "sch-barre-accent"), ("contre les joueurs", joueurs, "sch-barre-chaud")].iter().enumerate() {
        let y = 90.0 + i as f64 * 70.0;
        c += &texte(x, y, "sch-petit", nom);
        c += &boite(x, y + 8.0, 300.0 * val / maxi, 26.0, classe);
        c += &texte(x + 8.0, y + 26.0, "sch-fort", &format!("× {}", entier(*val)));
    }
    c += &texte(x, 250.0, "sch-petit", "une épée de départ tue un gnarling en ~3 coups ;");
    c += &texte(x, 268.0, "sch-petit", "un gnarling tue un joueur en 3 à 4 coups");
    Schema {
        svg: svg(740.0, 320.0, &c),
        legende: format!(
            "On verrouille un ennemi jusqu'à {} blocs, le verrouillage saute passé {} blocs. Les dégâts sont multipliés par {} \
contre les créatures et par {} contre les joueurs.",
            entier(rayon),
            entier(perte),
            entier(creatures),
            entier(joueurs)
        ),
        images: vec![CHARACTER.into()],
    }
}

/// Le plan du village : la place, les bâtiments, les maisons bâties, les terrains proposés.
fn plan_village(village: &Fichier) -> Schema {
    let maisons = nombre(village, "maisons_au_depart", 8.0) as usize;
    let terrains = nombre(village, "terrains_ouverts", 6.0) as usize;
    let prix = nombre(village, "prix_maison", 300.0);
    let heures = nombre(village, "heures_chantier", 6.0);
    let habitants = nombre(village, "habitants_par_maison", 2.0);
    let batiments = ["entrepôt", "caisse", "intendance", "infirmerie", "départ", "charrette"];
    let (cx, cy) = (360.0, 170.0);
    let mut c = String::new();
    c += &format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"46\" class=\"sch-place\"/>");
    c += &texte(cx, cy + 5.0, "sch-centre sch-fort", "la place");
    let total = maisons + terrains;
    for i in 0..total {
        let (ray_x, ray_y, angle) = if i < batiments.len() {
            (150.0, 95.0, -std::f64::consts::FRAC_PI_2 + i as f64 * std::f64::consts::TAU / batiments.len() as f64)
        } else {
            let k = i - batiments.len();
            let n = (total - batiments.len()).max(1);
            (290.0, 140.0, -std::f64::consts::FRAC_PI_2 + (k as f64 + 0.5) * std::f64::consts::TAU / n as f64)
        };
        let (x, y) = (cx + ray_x * angle.cos(), cy + ray_y * angle.sin());
        let (l, h) = (76.0, 30.0);
        let classe = if i < batiments.len() {
            "sch-batiment"
        } else if i < maisons {
            "sch-maison"
        } else {
            "sch-terrain"
        };
        c += &boite(x - l / 2.0, y - h / 2.0, l, h, classe);
        let libelle = if i < batiments.len() {
            batiments[i].to_string()
        } else if i < maisons {
            "maison".to_string()
        } else {
            "à bâtir".to_string()
        };
        c += &texte(x, y + 4.0, "sch-centre sch-petit", &libelle);
    }
    c += &texte(20.0, 352.0, "sch-petit", &format!("{} maisons bâties au départ (bâtiments compris), {} terrains proposés, les plus proches de la place d'abord", maisons, terrains));
    c += &texte(20.0, 370.0, "sch-petit", &format!("une maison : {} or de la caisse, {} h de carte de chantier, {} habitants", entier(prix), entier(heures), entier(habitants)));
    Schema {
        svg: svg(740.0, 380.0, &c),
        legende: format!(
            "Les six bâtiments de l'équipe entourent la place ; {} maisons sont bâties au départ et {} terrains attendent \
l'or de la caisse ({} or, {} h de carte).",
            maisons,
            terrains,
            entier(prix),
            entier(heures)
        ),
        images: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reglages;

    fn fichiers() -> Vec<Fichier> {
        let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        reglages::lire_dossier(&racine.join("assets").join("icebreaker")).unwrap()
    }

    #[test]
    fn nombres_dans_l_arbre_et_dans_les_groupes_en_ligne() {
        let f = reglages::analyser(
            "(\n delai: 30.0,\n entree: (profondeur: 18, longueur: 36),\n reseau: (\n  poches: (6, 10),\n ),\n)\n",
            "x.ron",
        );
        assert_eq!(nombres(&f.entrees, "delai"), Some((30.0, None)));
        assert_eq!(nombres(&f.entrees, "profondeur"), Some((18.0, None)));
        assert_eq!(nombres(&f.entrees, "longueur"), Some((36.0, None)));
        assert_eq!(nombres(&f.entrees, "poches"), Some((6.0, Some(10.0))));
        assert_eq!(nombres(&f.entrees, "absent"), None);
    }

    #[test]
    fn tous_les_schemas_se_dessinent_sur_les_vrais_reglages() {
        let fichiers = fichiers();
        for nom in NOMS {
            let s = dessiner(nom, &fichiers, "../").unwrap_or_else(|| panic!("schéma {nom}"));
            assert!(s.svg.starts_with("<svg"), "{nom}");
            assert!(!s.legende.is_empty(), "{nom}");
            for i in &s.images {
                assert!(s.svg.contains(&format!("../assets/{i}")), "{nom} : {i}");
            }
        }
        let mort = dessiner("frise-mort", &fichiers, "../").unwrap();
        assert!(mort.svg.contains("30 s à ramper"));
        assert!(mort.svg.contains("25 % des PV"));
        let campagne = dessiner("carte-campagne", &fichiers, "../").unwrap();
        assert_eq!(campagne.svg.matches("sch-zone\"").count(), 6); // le village et cinq emplacements
        assert!(dessiner("inconnu", &fichiers, "../").is_none());
    }
}
