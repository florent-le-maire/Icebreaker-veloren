//! Icebreaker : la rubrique « Compétences » du wiki (W.17) : toutes les compétences de Veloren, pas seulement
//! celles d'Icebreaker, avec pour chacune l'arme ou la créature qui l'a, la touche, la condition (posture, combo,
//! flèche enchantée, deux armes), ce qu'il faut débloquer dans l'arbre et ses prérequis, son type, ses chiffres et
//! la classe d'Icebreaker qui l'a au départ.
//!
//! Sources : `assets/common/abilities/ability_set_manifest.ron` (qui a quoi, sur quelle touche) et un fichier par
//! compétence à côté, les noms de `assets/voxygen/i18n/{fr,en}/hud/ability.ftl`, les prérequis de
//! `assets/common/skill_trees/skill_prerequisites.ron`, les porteurs (objets `ability_spec: Some(Custom(…))` →
//! équipements → entités, et `default_main_tool` de `common/src/comp/inventory/loadout_builder.rs` pour les
//! créatures sauvages), et les classes d'Icebreaker (`assets/icebreaker/classes.ron`, `classes/competences/`).

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::OnceLock,
};

use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

use crate::creatures::{lire_noms, nom_d_entite};

/// Combien de porteurs on montre avant « et N autres ».
const PORTEURS_MAX: usize = 14;

// --- Lecture du RON de Veloren, sans crate `ron` : parenthèses équilibrées et virgules au niveau 0. ---

/// Enlève les commentaires `// …` (hors chaînes) et les lignes `#![…]`.
pub fn sans_commentaires(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for ligne in texte.lines() {
        if ligne.trim_start().starts_with("#!") {
            sortie.push('\n');
            continue;
        }
        let mut dans_chaine = false;
        let mut precedent = ' ';
        let mut coupe = ligne.len();
        for (i, c) in ligne.char_indices() {
            if c == '"' && precedent != '\\' {
                dans_chaine = !dans_chaine;
            } else if !dans_chaine && c == '/' && precedent == '/' {
                coupe = i - 1;
                break;
            }
            precedent = c;
        }
        sortie.push_str(&ligne[..coupe]);
        sortie.push('\n');
    }
    sortie
}

/// L'indice de la parenthèse (ou du crochet, de l'accolade) qui ferme celle en `ouvrant`.
fn fermeture(texte: &str, ouvrant: usize) -> Option<usize> {
    let mut profondeur = 0i32;
    let mut dans_chaine = false;
    let mut precedent = ' ';
    for (i, c) in texte[ouvrant..].char_indices() {
        match c {
            '"' if precedent != '\\' => dans_chaine = !dans_chaine,
            '(' | '[' | '{' if !dans_chaine => profondeur += 1,
            ')' | ']' | '}' if !dans_chaine => {
                profondeur -= 1;
                if profondeur == 0 {
                    return Some(ouvrant + i);
                }
            },
            _ => {},
        }
        precedent = c;
    }
    None
}

/// Les morceaux séparés par des virgules au niveau 0, sans espaces autour, sans les vides.
fn separer(texte: &str) -> Vec<&str> {
    let mut morceaux = Vec::new();
    let (mut profondeur, mut dans_chaine, mut precedent, mut debut) = (0i32, false, ' ', 0);
    for (i, c) in texte.char_indices() {
        match c {
            '"' if precedent != '\\' => dans_chaine = !dans_chaine,
            '(' | '[' | '{' if !dans_chaine => profondeur += 1,
            ')' | ']' | '}' if !dans_chaine => profondeur -= 1,
            ',' if !dans_chaine && profondeur == 0 => {
                morceaux.push(texte[debut..i].trim());
                debut = i + 1;
            },
            _ => {},
        }
        precedent = c;
    }
    morceaux.push(texte[debut..].trim());
    morceaux.into_iter().filter(|m| !m.is_empty()).collect()
}

/// Les champs `clé: valeur` au niveau 0.
fn champs(texte: &str) -> Vec<(&str, &str)> {
    separer(texte)
        .into_iter()
        .filter_map(|m| m.split_once(':').map(|(k, v)| (k.trim(), v.trim())))
        .collect()
}

/// Le contenu de `Nom( … )` (ou `( … )`, `[ … ]` si `nom` est vide), si le texte a cette forme.
fn dedans<'a>(texte: &'a str, nom: &str) -> Option<&'a str> {
    let texte = texte.trim();
    let reste = texte.strip_prefix(nom)?;
    let ouvrant = texte.len() - reste.len();
    if !matches!(reste.chars().next(), Some('(' | '[' | '{')) {
        return None;
    }
    let fin = fermeture(texte, ouvrant)?;
    (fin == texte.len() - 1).then(|| &texte[ouvrant + 1..fin])
}

/// `Some(x)` → `x`, `None` → rien (le manifeste écrit les deux, `implicit_some`).
fn sans_some(texte: &str) -> Option<&str> {
    let t = texte.trim();
    if t == "None" {
        None
    } else {
        Some(dedans(t, "Some").unwrap_or(t).trim())
    }
}

fn chaine(texte: &str) -> String { texte.trim().trim_matches('"').to_string() }

/// `CrescentSlash` → `crescent_slash`.
pub fn serpent(nom: &str) -> String {
    let mut s = String::new();
    for (i, c) in nom.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push('_');
        }
        s.extend(c.to_lowercase());
    }
    s
}

/// `CrescentSlash` → `Crescent Slash`.
fn mots(nom: &str) -> String {
    let mut s = String::new();
    for (i, c) in nom.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push(' ');
        }
        s.push(c);
    }
    s
}

// --- Le manifeste ---

/// Où se déclenche une compétence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Emplacement {
    Principale,
    Secondaire,
    Garde,
    Auxiliaire(usize),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Contexte {
    /// `Sword(Heavy)`, `Bow(Heartseeker)`.
    pub posture: Option<String>,
    pub combo: Option<String>,
    /// `IgniteArrow`, `ArdentHunt`.
    pub buff: Option<String>,
    pub deux_armes: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variante {
    pub contexte: Contexte,
    /// Compétence de l'arbre à débloquer : `Sword(CrescentSlash)`.
    pub competence: Option<String>,
    /// `common.abilities.sword.heavy_crescent_slash`.
    pub id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub emplacement: Emplacement,
    /// Le nom commun des variantes (`Contextualized`), sinon rien.
    pub pseudo: Option<String>,
    pub variantes: Vec<Variante>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Jeu {
    /// `Tool(Sword)` ou `Custom("Husk")`.
    pub spec: String,
    pub outil: Option<String>,
    pub custom: Option<String>,
    pub cases: Vec<Case>,
}

fn lire_contexte(texte: &str) -> Contexte {
    let mut c = Contexte::default();
    for (k, v) in champs(texte) {
        match k {
            "stance" => c.posture = sans_some(v).map(str::to_string),
            "combo" => c.combo = sans_some(v).map(str::to_string),
            "buff" => c.buff = sans_some(v).map(str::to_string),
            "dual_wielding_same_kind" => c.deux_armes = v == "true",
            _ => {},
        }
    }
    c
}

/// Une case : `Simple(compétence, "id")` ou `Contextualized(pseudo_id: …, abilities: […])`.
fn lire_case(texte: &str, emplacement: Emplacement) -> Option<Case> {
    let texte = sans_some(texte)?;
    if let Some(inner) = dedans(texte, "Simple") {
        let parts = separer(inner);
        let (competence, id) = (parts.first()?, parts.get(1)?);
        return Some(Case {
            emplacement,
            pseudo: None,
            variantes: vec![Variante {
                contexte: Contexte::default(),
                competence: sans_some(competence).map(str::to_string),
                id: chaine(id),
            }],
        });
    }
    let inner = dedans(texte, "Contextualized")?;
    let mut pseudo = None;
    let mut variantes = Vec::new();
    for (k, v) in champs(inner) {
        match k {
            "pseudo_id" => pseudo = Some(chaine(v)),
            "abilities" => {
                for m in separer(dedans(v, "").unwrap_or("")) {
                    let paire = separer(dedans(m, "").unwrap_or(""));
                    let (Some(ctx), Some(cible)) = (paire.first(), paire.get(1)) else { continue };
                    let cible = separer(dedans(cible, "").unwrap_or(""));
                    let (Some(competence), Some(id)) = (cible.first(), cible.get(1)) else { continue };
                    variantes.push(Variante {
                        contexte: lire_contexte(dedans(ctx, "").unwrap_or("")),
                        competence: sans_some(competence).map(str::to_string),
                        id: chaine(id),
                    });
                }
            },
            _ => {},
        }
    }
    Some(Case { emplacement, pseudo, variantes })
}

/// Tous les jeux de compétences du manifeste, dans son ordre.
pub fn lire_manifeste(texte: &str) -> Vec<Jeu> {
    static ENTREE: OnceLock<Regex> = OnceLock::new();
    let entree = ENTREE.get_or_init(|| {
        Regex::new(r#"(Tool\((\w+)\)|Custom\("([^"]+)"\))\s*:\s*AbilitySet\("#).expect("regex")
    });
    let texte = sans_commentaires(texte);
    let mut jeux = Vec::new();
    for m in entree.captures_iter(&texte) {
        let tout = m.get(0).expect("capture");
        let Some(fin) = fermeture(&texte, tout.end() - 1) else { continue };
        let corps = &texte[tout.end()..fin];
        let corps = dedans(corps, "").unwrap_or(corps);
        let mut cases = Vec::new();
        for (k, v) in champs(corps) {
            let simple = match k {
                "primary" => Some(Emplacement::Principale),
                "secondary" => Some(Emplacement::Secondaire),
                "guard" => Some(Emplacement::Garde),
                _ => None,
            };
            if let Some(e) = simple {
                cases.extend(lire_case(v, e));
            } else if k == "abilities" {
                for (i, a) in separer(dedans(v, "").unwrap_or("")).into_iter().enumerate() {
                    cases.extend(lire_case(a, Emplacement::Auxiliaire(i)));
                }
            }
        }
        cases.sort_by_key(|c| c.emplacement);
        jeux.push(Jeu {
            spec: m[1].to_string(),
            outil: m.get(2).map(|x| x.as_str().to_string()),
            custom: m.get(3).map(|x| x.as_str().to_string()),
            cases,
        });
    }
    jeux
}

// --- Les noms (fluent) ---

/// Les textes de `ability.ftl` : clé → (nom, description), espaces repliés, variables `{ $x }` en « N ».
pub fn lire_ftl(ftl: &str) -> BTreeMap<String, (String, String)> {
    static VARIABLE: OnceLock<Regex> = OnceLock::new();
    let variable = VARIABLE.get_or_init(|| Regex::new(r"\{\s*\$\w+\s*\}").expect("regex"));
    let mut textes: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut cle: Option<String> = None;
    let mut dans_desc = false;
    for ligne in ftl.lines() {
        let ligne = ligne.trim_end_matches('\r');
        if ligne.starts_with('#') || ligne.trim().is_empty() {
            continue;
        }
        let ajouter = |texte: &mut String, v: &str| {
            let v = variable.replace_all(v.trim(), "N");
            if !v.is_empty() {
                if !texte.is_empty() {
                    texte.push(' ');
                }
                texte.push_str(&v);
            }
        };
        if !ligne.starts_with(' ') {
            if let Some((k, v)) = ligne.split_once('=') {
                let k = k.trim().to_string();
                let mut nom = String::new();
                ajouter(&mut nom, v);
                textes.insert(k.clone(), (nom, String::new()));
                cle = Some(k);
                dans_desc = false;
            }
        } else if let Some(k) = &cle {
            let l = ligne.trim();
            let Some(entree) = textes.get_mut(k) else { continue };
            if let Some(reste) = l.strip_prefix('.') {
                dans_desc = reste.starts_with("desc");
                if dans_desc {
                    if let Some((_, v)) = reste.split_once('=') {
                        ajouter(&mut entree.1, v);
                    }
                }
            } else if dans_desc {
                ajouter(&mut entree.1, l);
            } else {
                ajouter(&mut entree.0, l);
            }
        }
    }
    textes
}

// --- Les données du dépôt, et ce qu'on en tire ---

/// Les entrées utiles, lues d'avance (les tests en fabriquent de petites).
#[derive(Default)]
pub struct Donnees {
    pub manifeste: String,
    /// id de compétence (`common.abilities.…`) → texte du fichier.
    pub fichiers: BTreeMap<String, String>,
    pub fr: BTreeMap<String, (String, String)>,
    pub en: BTreeMap<String, (String, String)>,
    pub prerequis: String,
    /// Noms français des créatures (`name.ftl`).
    pub noms: BTreeMap<String, String>,
    /// id d'objet (`common.items.…`) → texte, pour les objets qui ont une `ability_spec`.
    pub objets: BTreeMap<String, String>,
    /// id d'objet → nom français (`item_i18n_manifest.ron` et `i18n/fr/item/`).
    pub noms_objets: BTreeMap<String, String>,
    /// id d'équipement (`common.loadout.…`) → texte.
    pub equipements: BTreeMap<String, String>,
    /// id d'entité (`common.entity.…`) → texte.
    pub entites: BTreeMap<String, String>,
    /// Texte de `loadout_builder.rs` (pour `default_main_tool`).
    pub outils_par_defaut: String,
    /// Texte de `classes.ron` et des fichiers de compétences des classes (id `icebreaker.classes.competences.x`).
    pub classes: String,
    pub competences_classes: BTreeMap<String, String>,
}

/// D'où vient le nom affiché.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Traduction {
    Fr,
    En,
    Aucune,
}

/// Une ligne de tableau : une compétence à un emplacement, dans un contexte.
#[derive(Debug, Clone, Serialize)]
pub struct Ligne {
    pub id: String,
    pub nom: String,
    pub description: String,
    pub traduction: Traduction,
    pub touche: String,
    pub condition: String,
    pub deblocage: String,
    pub prerequis: String,
    pub genre: String,
    pub chiffres: String,
    /// « Fait entrer en posture lourde ».
    pub effet: String,
    /// Classes d'Icebreaker qui l'ont au départ (« Guerrier · touche 1 »).
    pub icebreaker: Vec<String>,
}

/// Un groupe de lignes : une case du manifeste (plusieurs variantes si elle dépend du contexte).
#[derive(Debug, Clone, Serialize)]
pub struct GroupeVue {
    /// Le nom commun des variantes, s'il y en a plusieurs.
    pub titre: Option<String>,
    pub lignes: Vec<Ligne>,
}

/// Un jeu de compétences rendu : une arme, une créature, un instrument…
#[derive(Debug, Clone, Serialize)]
pub struct JeuVue {
    pub ancre: String,
    pub spec: String,
    pub titre: String,
    pub objets: Vec<String>,
    pub porteurs: Vec<String>,
    pub autres_porteurs: usize,
    pub classes: Vec<String>,
    pub groupes: Vec<GroupeVue>,
    pub nombre: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Categorie {
    Epee,
    Hache,
    Marteau,
    Arc,
    Baton,
    Sceptre,
    AutresArmes,
    Creatures,
    Divers,
}

impl Categorie {
    pub fn toutes() -> [Categorie; 9] {
        use Categorie::*;
        [Epee, Hache, Marteau, Arc, Baton, Sceptre, AutresArmes, Creatures, Divers]
    }

    fn titre(self) -> &'static str {
        match self {
            Categorie::Epee => "Épée",
            Categorie::Hache => "Hache",
            Categorie::Marteau => "Marteau",
            Categorie::Arc => "Arc",
            Categorie::Baton => "Bâton de feu",
            Categorie::Sceptre => "Sceptre",
            Categorie::AutresArmes => "Lance, dague, bouclier et armes de PNJ",
            Categorie::Creatures => "Créatures et PNJ",
            Categorie::Divers => "Instruments, objets et outils",
        }
    }

    fn fichier(self) -> &'static str {
        match self {
            Categorie::Epee => "epee",
            Categorie::Hache => "hache",
            Categorie::Marteau => "marteau",
            Categorie::Arc => "arc",
            Categorie::Baton => "baton",
            Categorie::Sceptre => "sceptre",
            Categorie::AutresArmes => "autres-armes",
            Categorie::Creatures => "creatures",
            Categorie::Divers => "divers",
        }
    }

    fn icone(self) -> &'static str {
        match self {
            Categorie::Epee => "voxygen/element/weapons/sword.png",
            Categorie::Hache => "voxygen/element/weapons/axe.png",
            Categorie::Marteau => "voxygen/element/weapons/hammer.png",
            Categorie::Arc => "voxygen/element/weapons/bow.png",
            Categorie::Baton => "voxygen/element/weapons/staff.png",
            Categorie::Sceptre => "voxygen/element/weapons/sceptre.png",
            Categorie::AutresArmes => "voxygen/element/weapons/swordshield.png",
            Categorie::Creatures => "voxygen/element/ui/generic/icons/skull_2.png",
            Categorie::Divers => "voxygen/element/weapons/pickaxe.png",
        }
    }

    fn resume(self) -> &'static str {
        match self {
            Categorie::Epee | Categorie::Hache | Categorie::Marteau | Categorie::Arc | Categorie::Baton
            | Categorie::Sceptre => {
                "Les compétences d'une arme de joueur : celles de base sur les clics et la garde, les autres à \
                 débloquer dans l'arbre puis à placer sur les touches 1 à 5."
            },
            Categorie::AutresArmes => {
                "Les armes de joueur sans arbre (lance, dague, bouclier) et les versions simplifiées que portent les \
                 PNJ humanoïdes."
            },
            Categorie::Creatures => {
                "Les attaques des créatures et des PNJ : l'IA déclenche la principale, la secondaire et les \
                 compétences numérotées selon sa tactique."
            },
            Categorie::Divers => "Instruments de musique, objets à lancer (bombes, feux d'artifice), outils et débogage.",
        }
    }

    fn de_jeu(jeu: &Jeu) -> Categorie {
        match jeu.outil.as_deref() {
            Some("Sword") => return Categorie::Epee,
            Some("Axe") => return Categorie::Hache,
            Some("Hammer") => return Categorie::Marteau,
            Some("Bow") => return Categorie::Arc,
            Some("Staff") => return Categorie::Baton,
            Some("Sceptre") => return Categorie::Sceptre,
            Some("Spear" | "Dagger" | "Shield") => return Categorie::AutresArmes,
            Some(_) => return Categorie::Divers,
            None => {},
        }
        let dossier = jeu
            .cases
            .iter()
            .flat_map(|c| &c.variantes)
            .next()
            .and_then(|v| v.id.strip_prefix("common.abilities."))
            .and_then(|r| r.split('.').next())
            .unwrap_or("");
        match dossier {
            d if d.ends_with("simple") => Categorie::AutresArmes,
            "music" | "throw" | "debug" | "farming" | "pick" | "shovel" | "empty" | "tool" => Categorie::Divers,
            _ => Categorie::Creatures,
        }
    }
}

fn nom_d_outil(outil: &str) -> String {
    match outil {
        "Sword" => "Épée",
        "Axe" => "Hache",
        "Hammer" => "Marteau",
        "Bow" => "Arc",
        "Staff" => "Bâton de feu",
        "Sceptre" => "Sceptre",
        "Spear" => "Lance",
        "Dagger" => "Dague",
        "Shield" => "Bouclier",
        "Debug" => "Débogage (admin)",
        "Farming" => "Outil agricole",
        "Pick" => "Pioche",
        "Shovel" => "Pelle",
        "Empty" => "Mains nues",
        o => o,
    }
    .to_string()
}

/// Le type d'une compétence en mots simples, d'après sa variante de `CharacterAbility`.
pub fn genre(variante: &str) -> &'static str {
    match variante {
        "BasicMelee" => "Coup de mêlée",
        "ComboMelee2" => "Combo de mêlée",
        "FinisherMelee" => "Coup final (consomme le combo)",
        "RapidMelee" => "Rafale de mêlée",
        "DashMelee" => "Charge",
        "LeapMelee" => "Bond et frappe",
        "DiveMelee" => "Plongeon",
        "ChargedMelee" => "Coup chargé",
        "RiposteMelee" => "Riposte",
        "BasicRanged" => "Tir",
        "RapidRanged" => "Rafale de tirs",
        "ChargedRanged" => "Tir chargé",
        "LeapRanged" => "Bond et tir",
        "Throw" => "Lancer",
        "BasicBeam" => "Rayon",
        "Shockwave" => "Onde de choc",
        "LeapShockwave" => "Bond et onde de choc",
        "LeapExplosionShockwave" => "Bond explosif",
        "Explosion" => "Explosion",
        "BasicAura" => "Aura",
        "StaticAura" => "Aura au sol",
        "SelfBuff" => "Renforcement",
        "BasicSummon" => "Invocation",
        "SpriteSummon" => "Invocation de décor",
        "Blink" => "Téléportation",
        "BasicBlock" => "Garde, parade",
        "Boost" => "Élan",
        "GlideBoost" => "Élan en vol",
        "Music" => "Musique",
        "Transform" => "Transformation",
        "RegrowHead" => "Repousse de tête",
        "Roll" => "Roulade",
        "Simple" => "Action simple",
        _ => "Autre",
    }
}

/// La variante (`ComboMelee2`…) d'un fichier de compétence.
pub fn variante_de(texte: &str) -> Option<String> {
    static VARIANTE: OnceLock<Regex> = OnceLock::new();
    let v = VARIANTE.get_or_init(|| Regex::new(r"^\s*([A-Z]\w*)\s*\(").expect("regex"));
    v.captures(sans_commentaires(texte).trim_start()).map(|c| c[1].to_string())
}

/// Coût d'énergie, dégâts du premier coup et nombre de coups d'un fichier de compétence.
fn chiffres_de(texte: &str) -> String {
    static ENERGIE: OnceLock<Regex> = OnceLock::new();
    static DEGATS: OnceLock<Regex> = OnceLock::new();
    static DRAIN: OnceLock<Regex> = OnceLock::new();
    let energie =
        ENERGIE.get_or_init(|| Regex::new(r"\benergy_cost(_per_strike)?:\s*([0-9.]+)").expect("regex"));
    let drain = DRAIN.get_or_init(|| Regex::new(r"\benergy_drain:\s*([0-9.]+)").expect("regex"));
    let degats = DEGATS.get_or_init(|| Regex::new(r"\bdamage:\s*([0-9.]+)").expect("regex"));
    let mut parts = Vec::new();
    let nombre = |s: &str| s.trim_end_matches(".0").to_string();
    if let Some(c) = degats.captures(texte).filter(|c| c[1].parse::<f32>().is_ok_and(|x| x > 0.0)) {
        parts.push(format!("dégâts {}", nombre(&c[1])));
    }
    let coups = texte.matches("melee_constructor").count();
    if texte.trim_start().starts_with("ComboMelee2") && coups > 1 {
        parts.push(format!("{coups} coups"));
    }
    if let Some(c) = energie.captures(texte) {
        if c[2].parse::<f32>().is_ok_and(|e| e > 0.0) {
            let par = if c.get(1).is_some() { " par coup" } else { "" };
            parts.push(format!("énergie {}{par}", nombre(&c[2])));
        }
    } else if let Some(c) = drain.captures(texte) {
        parts.push(format!("énergie {}/s", nombre(&c[1])));
    }
    parts.join(", ")
}

/// La posture dans laquelle une compétence fait entrer (`EnterStance(Sword(Heavy))`).
fn posture_entree(texte: &str) -> Option<String> {
    static ENTREE: OnceLock<Regex> = OnceLock::new();
    let e = ENTREE.get_or_init(|| Regex::new(r"EnterStance\(\s*(\w+\(\w+\))\s*\)").expect("regex"));
    e.captures(texte).map(|c| c[1].to_string())
}

/// La clé fluent d'une posture (`Stance::pseudo_ability_id` de `common/src/comp/ability.rs`).
fn cle_de_posture(posture: &str) -> String {
    let (outil, nom) = posture.trim_end_matches(')').split_once('(').unwrap_or(("", posture));
    match outil {
        "Sword" => format!("veloren-core-pseudo_abilities-sword-{}_stance", serpent(nom)),
        o => format!("common-abilities-{}-{}", o.to_lowercase(), serpent(nom)),
    }
}

/// Ce qu'on tire des données : les jeux rendus, rangés par page, et le bilan de la vue d'ensemble.
pub struct Resultat {
    pub pages: BTreeMap<Categorie, Vec<JeuVue>>,
    pub postures: BTreeMap<String, Vec<(String, String)>>,
    pub genres: BTreeMap<String, usize>,
    pub inutilises: Vec<String>,
    pub sans_francais: Vec<String>,
    pub references: usize,
}

struct Textes<'a> {
    d: &'a Donnees,
}

impl Textes<'_> {
    fn de(&self, cle: &str) -> Option<(&str, &str, Traduction)> {
        if let Some((n, desc)) = self.d.fr.get(cle).filter(|(n, _)| !n.is_empty()) {
            return Some((n, desc, Traduction::Fr));
        }
        self.d.en.get(cle).filter(|(n, _)| !n.is_empty()).map(|(n, desc)| (n.as_str(), desc.as_str(), Traduction::En))
    }

    /// Nom, description et origine d'une compétence d'après son id.
    fn competence(&self, id: &str) -> (String, String, Traduction) {
        match self.de(&id.replace('.', "-")) {
            Some((n, d, t)) => (n.to_string(), d.to_string(), t),
            None => {
                let court = id.rsplit('.').next().unwrap_or(id);
                (court.replace('_', " "), String::new(), Traduction::Aucune)
            },
        }
    }
}

/// Les porteurs de chaque `Custom` : (objets, porteurs).
fn porteurs(d: &Donnees) -> BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> {
    static SPEC: OnceLock<Regex> = OnceLock::new();
    static LEGACY: OnceLock<Regex> = OnceLock::new();
    static OBJET: OnceLock<Regex> = OnceLock::new();
    static EQUIPEMENT: OnceLock<Regex> = OnceLock::new();
    static PAR_DEFAUT: OnceLock<Regex> = OnceLock::new();
    let spec = SPEC.get_or_init(|| Regex::new(r#"ability_spec:\s*Some\(\s*Custom\("([^"]+)"\)"#).expect("regex"));
    let legacy = LEGACY.get_or_init(|| Regex::new(r#"legacy_name:\s*"([^"]+)""#).expect("regex"));
    let objet = OBJET.get_or_init(|| Regex::new(r#"common\.items\.[\w.]+"#).expect("regex"));
    let equipement = EQUIPEMENT.get_or_init(|| Regex::new(r#"common\.loadout\.[\w.]+"#).expect("regex"));
    let par_defaut = PAR_DEFAUT
        .get_or_init(|| Regex::new(r#"(\w+)::Species::(\w+)|"(common\.items\.[\w.]+)""#).expect("regex"));

    let mut sortie: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    // Objet → Custom.
    let mut custom_de: BTreeMap<&str, String> = BTreeMap::new();
    for (id, texte) in &d.objets {
        let Some(c) = spec.captures(texte) else { continue };
        custom_de.insert(id, c[1].to_string());
        let nom = d
            .noms_objets
            .get(id)
            .cloned()
            .or_else(|| legacy.captures(texte).map(|l| l[1].to_string()))
            .unwrap_or_else(|| id.rsplit('.').next().unwrap_or(id).to_string());
        sortie.entry(c[1].to_string()).or_default().0.insert(nom);
    }
    // Équipement → objets, en suivant les équipements qu'il cite.
    fn objets_de(
        id: &str,
        d: &Donnees,
        objet: &Regex,
        equipement: &Regex,
        profondeur: usize,
        vus: &mut BTreeSet<String>,
    ) {
        let Some(texte) = d.equipements.get(id) else { return };
        for m in objet.find_iter(texte) {
            vus.insert(m.as_str().to_string());
        }
        if profondeur < 4 {
            for m in equipement.find_iter(texte) {
                if m.as_str() != id {
                    objets_de(m.as_str(), d, objet, equipement, profondeur + 1, vus);
                }
            }
        }
    }
    for (id, texte) in &d.entites {
        let mut vus: BTreeSet<String> = objet.find_iter(texte).map(|m| m.as_str().to_string()).collect();
        for m in equipement.find_iter(texte) {
            objets_de(m.as_str(), d, objet, equipement, 0, &mut vus);
        }
        let repli = id.rsplit('.').next().unwrap_or(id);
        for o in vus {
            if let Some(c) = custom_de.get(o.as_str()) {
                sortie.entry(c.clone()).or_default().1.insert(nom_d_entite(texte, &d.noms, repli));
            }
        }
    }
    // Créatures sauvages : l'arme naturelle selon l'espèce (`default_main_tool`).
    if let Some(debut) = d.outils_par_defaut.find("fn default_main_tool") {
        let reste = &d.outils_par_defaut[debut..];
        let corps = &reste[..reste.find("\n}\n").unwrap_or(reste.len())];
        let mut especes: Vec<(String, String)> = Vec::new();
        for c in par_defaut.captures_iter(corps) {
            if let Some(o) = c.get(3) {
                if let Some(custom) = custom_de.get(o.as_str()) {
                    for (corps, espece) in &especes {
                        sortie.entry(custom.clone()).or_default().1.insert(nom_d_espece(corps, espece, &d.noms));
                    }
                }
                especes.clear();
            } else {
                especes.push((c[1].to_string(), c[2].to_string()));
            }
        }
    }
    sortie
}

/// Le nom français d'une espèce (`quadruped_medium`, `Saber`) d'après `name.ftl`.
fn nom_d_espece(corps: &str, espece: &str, noms: &BTreeMap<String, String>) -> String {
    for cle in [format!("name-body-{corps}-{}", serpent(espece)), format!("name-body-{corps}-{}", espece.to_lowercase())] {
        if let Some(n) = noms.get(&cle) {
            return n.clone();
        }
    }
    // `Cultistwarlock` → `name-body-biped_large-cultist_warlock` : même clé sans les tirets bas.
    let prefixe = format!("name-body-{corps}-");
    let cherche = espece.to_lowercase();
    noms.iter()
        .find(|(k, _)| k.strip_prefix(&prefixe).is_some_and(|s| s.replace('_', "") == cherche))
        .map_or_else(|| mots(espece), |(_, n)| n.clone())
}

/// Les classes d'Icebreaker : (nom, arme, compétences débloquées, touches).
fn classes(d: &Donnees) -> Vec<(String, String, Vec<String>, Vec<String>)> {
    static CLASSE: OnceLock<Regex> = OnceLock::new();
    static ARME: OnceLock<Regex> = OnceLock::new();
    static COMPETENCE: OnceLock<Regex> = OnceLock::new();
    let classe = CLASSE.get_or_init(|| {
        Regex::new(r#"(?s)\(id:\s*"[^"]+",\s*nom:\s*"([^"]+)".*?competences:\s*"([^"]+)"(?:,\s*touches:\s*\[([^\]]*)\])?"#)
            .expect("regex")
    });
    let arme = ARME.get_or_init(|| Regex::new(r"Group\(Weapon\((\w+)\)\)").expect("regex"));
    let competence = COMPETENCE.get_or_init(|| Regex::new(r"Skill\(\(\s*(\w+\(\w+\))").expect("regex"));
    let texte = sans_commentaires(&d.classes);
    classe
        .captures_iter(&texte)
        .filter_map(|c| {
            let fichier = d.competences_classes.get(&c[2])?;
            let arme = arme.captures(fichier)?[1].to_string();
            let competences = competence.captures_iter(fichier).map(|s| s[1].to_string()).collect();
            let touches = c.get(3).map_or(Vec::new(), |t| separer(t.as_str()).into_iter().map(str::to_string).collect());
            let mut nom = c[1].chars();
            let nom = nom.next().map(|p| p.to_uppercase().chain(nom).collect()).unwrap_or_default();
            Some((nom, arme, competences, touches))
        })
        .collect()
}

/// Les prérequis de l'arbre : compétence → (« Any » ou « All », compétences).
fn prerequis(texte: &str) -> BTreeMap<String, (bool, Vec<String>)> {
    static LIGNE: OnceLock<Regex> = OnceLock::new();
    static CLE: OnceLock<Regex> = OnceLock::new();
    let ligne = LIGNE.get_or_init(|| Regex::new(r"(\w+\(\w+\)):\s*(Any|All)\(\{([^}]*)\}\)").expect("regex"));
    let cle = CLE.get_or_init(|| Regex::new(r"(\w+\(\w+\)):\s*\d+").expect("regex"));
    ligne
        .captures_iter(&sans_commentaires(texte))
        .map(|c| (c[1].to_string(), (&c[2] == "Any", cle.captures_iter(&c[3]).map(|k| k[1].to_string()).collect())))
        .collect()
}

/// Tout ce qu'il faut pour les pages.
pub fn collecter(d: &Donnees) -> Resultat {
    let t = Textes { d };
    let jeux = lire_manifeste(&d.manifeste);
    let prerequis = prerequis(&d.prerequis);
    let porteurs = porteurs(d);
    let classes = classes(d);

    // Le nom d'une compétence de l'arbre : celui de la case `Simple` qui la demande, sinon le nom commun d'une
    // case à variantes, sinon ses mots.
    let mut noms_arbre: BTreeMap<String, String> = BTreeMap::new();
    for simple in [true, false] {
        for case in jeux.iter().flat_map(|j| &j.cases) {
            if (case.pseudo.is_none()) != simple {
                continue;
            }
            for v in &case.variantes {
                let Some(s) = &v.competence else { continue };
                if noms_arbre.contains_key(s) {
                    continue;
                }
                let nom = match &case.pseudo {
                    Some(p) => t.de(&p.replace('.', "-")).map(|(n, _, _)| n.to_string()),
                    None => None,
                }
                .or_else(|| {
                    let (n, _, traduction) = t.competence(&v.id);
                    (traduction != Traduction::Aucune).then_some(n)
                });
                if let Some(nom) = nom {
                    noms_arbre.insert(s.clone(), nom);
                }
            }
        }
    }
    let nom_arbre = |s: &str| {
        noms_arbre.get(s).cloned().unwrap_or_else(|| {
            let nom = s.trim_end_matches(')').split_once('(').map_or(s, |(_, n)| n);
            mots(nom)
        })
    };
    let nom_posture = |p: &str| {
        let nom = t
            .de(&cle_de_posture(p))
            .map(|(n, _, _)| n.to_string())
            .unwrap_or_else(|| mots(p.trim_end_matches(')').split_once('(').map_or(p, |(_, n)| n)));
        if nom.to_lowercase().starts_with("posture") { nom } else { format!("Posture {nom}") }
    };

    let mut resultat = Resultat {
        pages: BTreeMap::new(),
        postures: BTreeMap::new(),
        genres: BTreeMap::new(),
        inutilises: Vec::new(),
        sans_francais: Vec::new(),
        references: 0,
    };
    let mut references: BTreeSet<&str> = BTreeSet::new();
    let mut ancres: BTreeSet<String> = BTreeSet::new();

    for jeu in &jeux {
        let categorie = Categorie::de_jeu(jeu);
        let outil = jeu.outil.as_deref();
        let titre = match (&jeu.outil, &jeu.custom) {
            (Some(o), _) => nom_d_outil(o),
            (_, Some(c)) => c.clone(),
            _ => jeu.spec.clone(),
        };
        let mut groupes = Vec::new();
        for case in &jeu.cases {
            let touche = match (case.emplacement, outil.is_some() || categorie == Categorie::AutresArmes) {
                (Emplacement::Principale, true) => "Clic gauche".to_string(),
                (Emplacement::Secondaire, true) => "Clic droit".to_string(),
                (Emplacement::Garde, _) => "Garde".to_string(),
                (Emplacement::Auxiliaire(i), true) => format!("Compétence {} (touches 1 à 5)", i + 1),
                (Emplacement::Principale, false) => "Attaque principale".to_string(),
                (Emplacement::Secondaire, false) => "Attaque secondaire".to_string(),
                (Emplacement::Auxiliaire(i), false) => format!("Compétence {}", i + 1),
            };
            let plusieurs = case.variantes.len() > 1;
            let mut lignes = Vec::new();
            for v in &case.variantes {
                references.insert(&v.id);
                let (nom, description, traduction) = t.competence(&v.id);
                let fichier = d.fichiers.get(&v.id).map(String::as_str).unwrap_or("");
                let variante = variante_de(fichier).unwrap_or_default();
                *resultat.genres.entry(variante.clone()).or_default() += 1;
                if outil.is_some() && traduction != Traduction::Fr {
                    resultat.sans_francais.push(v.id.clone());
                }
                let mut condition = Vec::new();
                let c = &v.contexte;
                if let Some(p) = &c.posture {
                    condition.push(format!("en {}", nom_posture(p).to_lowercase()));
                }
                if let Some(b) = &c.buff {
                    let outil_b = c
                        .posture
                        .as_deref()
                        .and_then(|p| p.split_once('('))
                        .map(|(o, _)| o.to_string())
                        .or(outil.map(str::to_string))
                        .unwrap_or_default();
                    condition.push(format!("avec l'effet « {} » actif", nom_arbre(&format!("{outil_b}({b})"))));
                }
                if let Some(n) = &c.combo {
                    condition.push(format!("combo ≥ {n}"));
                }
                if c.deux_armes {
                    condition.push("deux armes pareilles en main".into());
                }
                let condition = if !condition.is_empty() {
                    condition.join(", ")
                } else if plusieurs {
                    "sinon (par défaut)".into()
                } else {
                    String::new()
                };
                let (deblocage, prereq) = match &v.competence {
                    None => (String::new(), String::new()),
                    Some(s) => {
                        let p = prerequis.get(s).map_or(String::new(), |(un, liste)| {
                            let noms: Vec<String> = liste.iter().map(|x| nom_arbre(x)).collect();
                            match (un, noms.len()) {
                                (_, 1) => noms[0].clone(),
                                (true, _) => format!("une de : {}", noms.join(", ")),
                                (false, _) => format!("toutes : {}", noms.join(", ")),
                            }
                        });
                        (nom_arbre(s), p)
                    },
                };
                let mut icebreaker = Vec::new();
                for (classe, arme, competences, touches) in &classes {
                    if outil != Some(arme.as_str()) {
                        continue;
                    }
                    match &v.competence {
                        None => icebreaker.push(classe.clone()),
                        Some(s) if competences.contains(s) => match touches.iter().position(|x| x == s) {
                            Some(i) => icebreaker.push(format!("{classe} · touche {}", i + 1)),
                            None => icebreaker.push(classe.clone()),
                        },
                        _ => {},
                    }
                }
                lignes.push(Ligne {
                    id: v.id.clone(),
                    nom,
                    description,
                    traduction,
                    touche: touche.clone(),
                    condition,
                    deblocage,
                    prerequis: prereq,
                    genre: if variante.is_empty() { "?".into() } else { genre(&variante).into() },
                    chiffres: chiffres_de(fichier),
                    effet: posture_entree(fichier)
                        .map(|p| format!("Fait entrer en {}.", nom_posture(&p).to_lowercase()))
                        .unwrap_or_default(),
                    icebreaker,
                });
            }
            let titre = plusieurs.then(|| {
                case.pseudo
                    .as_ref()
                    .and_then(|p| t.de(&p.replace('.', "-")).map(|(n, _, _)| n.to_string()))
                    .unwrap_or_else(|| lignes.last().map(|l: &Ligne| l.nom.clone()).unwrap_or_default())
            });
            groupes.push(GroupeVue { titre: titre.map(|n| format!("{touche} : {n}")), lignes });
        }
        // Les postures de l'arme (celles qui servent de condition).
        let mut postures: Vec<String> = jeu
            .cases
            .iter()
            .flat_map(|c| &c.variantes)
            .filter_map(|v| v.contexte.posture.clone())
            .collect();
        postures.dedup();
        let mut vues = BTreeSet::new();
        for p in postures {
            if vues.insert(p.clone()) {
                let desc = t.de(&cle_de_posture(&p)).map_or(String::new(), |(_, d, _)| d.to_string());
                let nom = nom_posture(&p);
                resultat.postures.entry(titre.clone()).or_default().push((nom, desc));
            }
        }
        let (objets, mut liste): (Vec<String>, Vec<String>) = match jeu.custom.as_ref().and_then(|c| porteurs.get(c)) {
            Some((o, p)) => (o.iter().cloned().collect(), p.iter().cloned().collect()),
            None => (Vec::new(), Vec::new()),
        };
        let autres = liste.len().saturating_sub(PORTEURS_MAX);
        liste.truncate(PORTEURS_MAX);
        let mut ancre = serpent(&titre.replace([' ', '\'', ','], "")).replace('_', "-");
        ancre = ancre.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
        while !ancres.insert(ancre.clone()) {
            ancre.push('-');
        }
        let nombre = groupes.iter().map(|g| g.lignes.len()).sum();
        let classes_de = classes
            .iter()
            .filter(|(_, arme, _, _)| outil == Some(arme.as_str()))
            .map(|(n, _, _, _)| n.clone())
            .collect();
        resultat.pages.entry(categorie).or_default().push(JeuVue {
            ancre,
            spec: jeu.spec.clone(),
            titre,
            objets,
            porteurs: liste,
            autres_porteurs: autres,
            classes: classes_de,
            groupes,
            nombre,
        });
    }
    resultat.references = references.len();
    resultat.inutilises = d.fichiers.keys().filter(|id| !references.contains(id.as_str())).cloned().collect();
    resultat.sans_francais.sort();
    resultat.sans_francais.dedup();
    resultat
}

// --- Les pages ---

/// L'URL d'une page de la rubrique.
pub fn url(categorie: Option<Categorie>) -> String {
    match categorie {
        Some(c) => format!("competences/{}.html", c.fichier()),
        None => "competences/index.html".into(),
    }
}

const ICONE_VUE: &str = "voxygen/element/weapons/swords_crossed.png";

/// La vue d'ensemble puis une page par catégorie, dans la rubrique « Compétences ».
pub fn pages(r: &Resultat) -> Vec<crate::Page> {
    let mut pages = Vec::new();
    let mut liste = Vec::new();
    let mut total = 0;
    for (ordre, categorie) in Categorie::toutes().into_iter().enumerate() {
        let Some(jeux) = r.pages.get(&categorie) else { continue };
        let nombre: usize = jeux.iter().map(|j| j.nombre).sum();
        total += nombre;
        liste.push(serde_json::json!({
            "titre": categorie.titre(),
            "url": url(Some(categorie)),
            "icone": categorie.icone(),
            "nombre": nombre,
            "jeux": jeux.len(),
        }));
        let postures: Vec<serde_json::Value> = jeux
            .iter()
            .filter_map(|j| r.postures.get(&j.titre).map(|p| (j, p)))
            .map(|(j, p)| {
                serde_json::json!({
                    "arme": j.titre,
                    "postures": p.iter().map(|(n, d)| serde_json::json!({"nom": n, "desc": d})).collect::<Vec<_>>(),
                })
            })
            .collect();
        pages.push(crate::Page {
            source: format!("wiki/competences/{}", categorie.fichier()),
            url: url(Some(categorie)),
            rubrique: crate::Rubrique::Competences,
            titre: categorie.titre().to_string(),
            html: String::new(),
            gabarit: "competences.html",
            extra: serde_json::json!({
                "titre": categorie.titre(),
                "resume": categorie.resume(),
                "icone": categorie.icone(),
                "nombre": nombre,
                "jeux": jeux,
                "sommaire": jeux.len() > 3,
                "postures": postures,
                "joueur": !matches!(categorie, Categorie::Creatures | Categorie::Divers),
            }),
            ordre: ordre + 1,
            icone: Some(categorie.icone().to_string()),
            images: vec![(format!("assets/{}", categorie.icone()), format!("assets/{}", categorie.icone()))],
        });
    }
    let genres: Vec<serde_json::Value> = {
        let mut g: Vec<(&String, &usize)> = r.genres.iter().collect();
        g.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        g.into_iter()
            .map(|(v, n)| serde_json::json!({"variante": v, "genre": genre(v), "nombre": n}))
            .collect()
    };
    let mut images: Vec<(String, String)> = vec![(format!("assets/{ICONE_VUE}"), format!("assets/{ICONE_VUE}"))];
    for c in Categorie::toutes() {
        images.push((format!("assets/{}", c.icone()), format!("assets/{}", c.icone())));
    }
    pages.insert(0, crate::Page {
        source: "wiki/competences/index".into(),
        url: url(None),
        rubrique: crate::Rubrique::Competences,
        titre: "Vue d'ensemble".into(),
        html: String::new(),
        gabarit: "competences.html",
        extra: serde_json::json!({
            "titre": "Toutes les compétences de Veloren",
            "resume": "Chaque arme, créature, instrument ou objet a un jeu de compétences : une attaque principale, \
                       une secondaire, parfois une garde, et des compétences numérotées. Lu dans les données du jeu, \
                       à jour à chaque génération du wiki.",
            "icone": ICONE_VUE,
            "nombre": total,
            "vue": true,
            "pages": liste,
            "references": r.references,
            "genres": genres,
            "inutilises": r.inutilises,
            "sans_francais": r.sans_francais,
        }),
        ordre: 0,
        icone: Some(ICONE_VUE.into()),
        images,
    });
    pages
}

/// Les textes des fichiers `.ron` d'un dossier, par id (`prefixe` + chemin avec des points).
fn lire_ids(dossier: &Path, prefixe: &str, filtre: impl Fn(&str) -> bool) -> BTreeMap<String, String> {
    let mut sortie = BTreeMap::new();
    for e in WalkDir::new(dossier).sort_by_file_name().into_iter().filter_map(Result::ok) {
        let chemin = e.path();
        if chemin.extension().and_then(|x| x.to_str()) != Some("ron") {
            continue;
        }
        let Ok(relatif) = chemin.strip_prefix(dossier) else { continue };
        let id = format!("{prefixe}{}", relatif.with_extension("").to_string_lossy().replace(['\\', '/'], "."));
        let Ok(texte) = std::fs::read_to_string(chemin) else { continue };
        if filtre(&texte) {
            sortie.insert(id, texte);
        }
    }
    sortie
}

/// Lit les sources du dépôt et rend les pages.
pub fn pages_du_depot(racine: &Path) -> Result<Vec<crate::Page>> {
    let lire = |chemin: &str| std::fs::read_to_string(racine.join(chemin)).with_context(|| format!("lecture de {chemin}"));
    let assets = racine.join("assets");
    let mut fichiers = lire_ids(&assets.join("common/abilities"), "common.abilities.", |_| true);
    fichiers.remove("common.abilities.ability_set_manifest");
    // Noms des objets : `item_i18n_manifest.ron` (id → clé) et les `.ftl` français des objets.
    let mut noms_cles = BTreeMap::new();
    for e in WalkDir::new(assets.join("voxygen/i18n/fr/item")).into_iter().filter_map(Result::ok) {
        if e.path().extension().and_then(|x| x.to_str()) == Some("ftl") {
            if let Ok(t) = std::fs::read_to_string(e.path()) {
                noms_cles.extend(lire_noms(&t));
            }
        }
    }
    let manifeste_objets = lire("assets/common/item_i18n_manifest.ron").unwrap_or_default();
    let paire = Regex::new(r#"Simple\(\s*"([^"]+)",?\s*\):\s*"([^"]+)""#).expect("regex");
    let noms_objets = paire
        .captures_iter(&manifeste_objets)
        .filter_map(|c| noms_cles.get(&c[2]).map(|n| (c[1].to_string(), n.clone())))
        .collect();
    let d = Donnees {
        manifeste: lire("assets/common/abilities/ability_set_manifest.ron")?,
        fichiers,
        fr: lire_ftl(&lire("assets/voxygen/i18n/fr/hud/ability.ftl")?),
        en: lire_ftl(&lire("assets/voxygen/i18n/en/hud/ability.ftl")?),
        prerequis: lire("assets/common/skill_trees/skill_prerequisites.ron")?,
        noms: lire_noms(&lire("assets/voxygen/i18n/fr/name.ftl")?),
        objets: lire_ids(&assets.join("common/items"), "common.items.", |t| t.contains("ability_spec")),
        noms_objets,
        equipements: lire_ids(&assets.join("common/loadout"), "common.loadout.", |_| true),
        entites: lire_ids(&assets.join("common/entity"), "common.entity.", |_| true),
        outils_par_defaut: lire("common/src/comp/inventory/loadout_builder.rs").unwrap_or_default(),
        classes: lire("assets/icebreaker/classes.ron")?,
        competences_classes: lire_ids(&assets.join("icebreaker/classes/competences"), "icebreaker.classes.competences.", |_| true),
    };
    Ok(pages(&collecter(&d)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFESTE: &str = r#"#![enable(implicit_some)]
// commentaire
({
    Tool(Sword): AbilitySet((
        guard: Some(Simple(None, "common.abilities.sword.basic_guard")),
        primary: Contextualized(
            pseudo_id: "veloren.core.pseudo_abilities.sword.double_slash",
            abilities: [
                ((stance: Sword(Heavy)), (None, "common.abilities.sword.heavy_double_slash")),
                ((), (None, "common.abilities.sword.basic_double_slash")),
            ],
        ),
        secondary: Simple(None, "common.abilities.sword.basic_thrust"),
        abilities: [
            Simple(Sword(CrescentSlash), "common.abilities.sword.basic_crescent_slash"),
            Contextualized(
                pseudo_id: "common.abilities.sword.sweep",
                abilities: [
                    ((stance: Sword(Heavy), dual_wielding_same_kind: true), (Sword(HeavySweep), "common.abilities.sword.heavy_sweep")),
                    ((combo: Some(30)), (Some(Sword(HeavySweep)), "common.abilities.sword.sweep")),
                ],
            ),
        ],
    )),
    Custom("Husk"): AbilitySet((
        primary: Simple(None, "common.abilities.custom.husk.singlestrike"),
        secondary: Simple(None, "common.abilities.custom.husk.triplestrike"),
        abilities: [],
    )),
    Custom("Admin's Eagle"): AbilitySet((
        primary: Simple(None, "common.abilities.debug.glide_speeder"),
        abilities: [],
    )),
})
"#;

    const FR: &str = "common-abilities-sword-basic_crescent_slash = Coup ascendant basique\n    .desc = Un coup.\n\
                      common-abilities-sword-heavy_sweep = Balayage lourd\n    .desc =\n        Un balayage\n        \
                      { $min_combo } fois.\nveloren-core-pseudo_abilities-sword-heavy_stance = Posture lourde\n    .desc \
                      = Lent.\nveloren-core-pseudo_abilities-sword-double_slash = Double entaille\n";

    fn donnees() -> Donnees {
        let fichier = |t: &str| t.to_string();
        Donnees {
            manifeste: MANIFESTE.into(),
            fichiers: [
                ("common.abilities.sword.basic_crescent_slash", "ComboMelee2(strikes: [(melee_constructor: (kind: Slash(damage: 12.0)))], energy_cost_per_strike: 5,)"),
                ("common.abilities.sword.heavy_sweep", "#![enable(implicit_some)]\nFinisherMelee(energy_cost: 0, damage: 30, meta: (init_event: EnterStance(Sword(Heavy))))"),
                ("common.abilities.custom.husk.singlestrike", "BasicMelee(energy_cost: 0)"),
                ("common.abilities.custom.husk.unused", "BasicMelee()"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), fichier(v)))
            .collect(),
            fr: lire_ftl(FR),
            en: lire_ftl("common-abilities-sword-basic_thrust = Basic Thrust\n"),
            prerequis: "{\n    Sword(HeavySweep): Any({Sword(CrescentSlash): 1, Sword(Cascade): 1}),\n}".into(),
            noms: lire_noms("name-custom-husk = Enveloppe\nname-body-golem-stone_golem = Golem de pierre\n"),
            objets: [
                ("common.items.npc_weapons.unique.husk", "ItemDef(legacy_name: \"Husk\", ability_spec: Some(Custom(\"Husk\")))"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
            noms_objets: BTreeMap::new(),
            equipements: [("common.loadout.husk", "(active_hands: InHands((Item(\"common.items.npc_weapons.unique.husk\"), None)))")]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            entites: [("common.entity.dungeon.cultist.husk", "(name: Translate(\"name-custom-husk\"), inventory: (loadout: Asset(\"common.loadout.husk\")))")]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            outils_par_defaut: "pub fn default_main_tool(body: &Body) -> Option<&'static str> {\n    golem::Species::StoneGolem => \"common.items.npc_weapons.unique.husk\",\n}\n".into(),
            classes: "(classes: [(id: \"guerrier\", nom: \"guerrier\", equipement: \"x\",\n competences: \"icebreaker.classes.competences.guerrier\",\n touches: [Sword(HeavySweep)]),])".into(),
            competences_classes: [(
                "icebreaker.classes.competences.guerrier".to_string(),
                "[Group(Weapon(Sword)), Skill((Sword(CrescentSlash), 1)), Skill((Sword(HeavySweep), 1))]".to_string(),
            )]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn manifeste_lu() {
        let jeux = lire_manifeste(MANIFESTE);
        assert_eq!(jeux.len(), 3);
        let epee = &jeux[0];
        assert_eq!(epee.outil.as_deref(), Some("Sword"));
        assert_eq!(epee.cases.len(), 5);
        assert_eq!(epee.cases[0].emplacement, Emplacement::Principale);
        assert_eq!(epee.cases[0].variantes[0].contexte.posture.as_deref(), Some("Sword(Heavy)"));
        assert_eq!(epee.cases[2].emplacement, Emplacement::Garde);
        let balayage = &epee.cases[4];
        assert_eq!(balayage.emplacement, Emplacement::Auxiliaire(1));
        assert!(balayage.variantes[0].contexte.deux_armes);
        assert_eq!(balayage.variantes[1].contexte.combo.as_deref(), Some("30"));
        assert_eq!(balayage.variantes[1].competence.as_deref(), Some("Sword(HeavySweep)"));
        assert_eq!(jeux[2].custom.as_deref(), Some("Admin's Eagle"));
    }

    #[test]
    fn fluent_et_ron() {
        let fr = lire_ftl(FR);
        assert_eq!(fr["common-abilities-sword-heavy_sweep"], ("Balayage lourd".into(), "Un balayage N fois.".into()));
        assert_eq!(sans_commentaires("a // b \"//\"\n\"x//y\" // z"), "a \n\"x//y\" \n");
        assert_eq!(separer("a, (b, c), \"d,e\","), vec!["a", "(b, c)", "\"d,e\""]);
        assert_eq!(variante_de("#![enable(implicit_some)]\n// c\nBasicRanged(\n)").as_deref(), Some("BasicRanged"));
        assert_eq!(serpent("CrescentSlash"), "crescent_slash");
    }

    #[test]
    fn lignes_completes() {
        let r = collecter(&donnees());
        let epee = &r.pages[&Categorie::Epee][0];
        assert_eq!(epee.classes, vec!["Guerrier"]);
        let ligne = |id: &str| {
            epee.groupes.iter().flat_map(|g| &g.lignes).find(|l| l.id == id).unwrap_or_else(|| panic!("{id}"))
        };
        assert_eq!(epee.groupes[0].titre.as_deref(), Some("Clic gauche : Double entaille"));
        assert_eq!(ligne("common.abilities.sword.heavy_double_slash").condition, "en posture lourde");
        assert_eq!(ligne("common.abilities.sword.basic_double_slash").condition, "sinon (par défaut)");
        assert_eq!(ligne("common.abilities.sword.basic_thrust").traduction, Traduction::En);
        let croissant = ligne("common.abilities.sword.basic_crescent_slash");
        assert_eq!(croissant.nom, "Coup ascendant basique");
        assert_eq!(croissant.touche, "Compétence 1 (touches 1 à 5)");
        assert_eq!(croissant.genre, "Combo de mêlée");
        assert_eq!(croissant.chiffres, "dégâts 12, énergie 5 par coup");
        assert_eq!(croissant.icebreaker, vec!["Guerrier"]);
        let lourd = ligne("common.abilities.sword.heavy_sweep");
        assert_eq!(lourd.condition, "en posture lourde, deux armes pareilles en main");
        assert_eq!(lourd.deblocage, "Balayage lourd");
        assert_eq!(lourd.prerequis, "une de : Coup ascendant basique, Cascade");
        assert_eq!(lourd.effet, "Fait entrer en posture lourde.");
        assert_eq!(lourd.icebreaker, vec!["Guerrier · touche 1"]);
        assert_eq!(ligne("common.abilities.sword.sweep").condition, "combo ≥ 30");
        assert_eq!(ligne("common.abilities.sword.basic_guard").icebreaker, vec!["Guerrier"]);
        assert_eq!(r.postures["Épée"], vec![("Posture lourde".to_string(), "Lent.".to_string())]);

        let husk = &r.pages[&Categorie::Creatures][0];
        assert_eq!(husk.titre, "Husk");
        assert_eq!(husk.objets, vec!["Husk"]);
        assert_eq!(husk.porteurs, vec!["Enveloppe", "Golem de pierre"]);
        assert_eq!(husk.groupes[0].lignes[0].touche, "Attaque principale");
        assert_eq!(husk.groupes[0].lignes[0].nom, "singlestrike");
        assert_eq!(r.pages[&Categorie::Divers][0].ancre, "admins-eagle");
        assert_eq!(r.inutilises, vec!["common.abilities.custom.husk.unused"]);
        assert!(r.sans_francais.contains(&"common.abilities.sword.basic_thrust".to_string()));
    }

    #[test]
    fn pages_du_vrai_depot() {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let pages = pages_du_depot(&racine).unwrap();
        assert_eq!(pages[0].url, "competences/index.html");
        assert!(pages[0].extra["references"].as_u64().unwrap() >= 600, "{}", pages[0].extra["references"]);
        for c in Categorie::toutes() {
            assert!(pages.iter().any(|p| p.url == url(Some(c))), "{c:?} sans page");
        }
        let epee = pages.iter().find(|p| p.url == "competences/epee.html").unwrap();
        let texte = epee.extra.to_string();
        assert!(texte.contains("common.abilities.sword.heavy_crescent_slash"));
        assert!(texte.contains("Guerrier · touche 1"));
        let creatures = pages.iter().find(|p| p.url == "competences/creatures.html").unwrap().extra.to_string();
        assert!(creatures.contains("Quad Med Jump"));
        for p in &pages {
            for (source, _) in &p.images {
                assert!(racine.join(source).is_file(), "{source} introuvable");
            }
        }
    }
}
