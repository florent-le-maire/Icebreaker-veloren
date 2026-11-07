//! Icebreaker : la rubrique « Créatures » du wiki (W.15) : boss et chefs, créatures de base, jouables, PNJ du
//! village, une carte par créature avec son icône et sa capture (ou l'encadré « capture à prendre »).
//!
//! Sources : `donjons/generation.ron` (factions et rôles), `grottes/grotte.ron` (ambiances), `classes.ron`, les
//! fichiers d'entité de Veloren (`assets/common/entity/`), les noms de `assets/voxygen/i18n/fr/name.ftl`, et le
//! complément écrit à la main `docs/icebreaker/wiki/creatures.toml` (PNJ, phrases).

use std::{collections::BTreeMap, path::Path, sync::OnceLock};

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::reglages::{Entree, Fichier};

/// Dossier des captures, depuis la racine du dépôt.
pub const DOSSIER_CAPTURES: &str = "docs/icebreaker/wiki/images/creatures/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Categorie {
    Boss,
    Base,
    Jouable,
    Pnj,
}

impl Categorie {
    fn toutes() -> [Categorie; 4] { [Categorie::Boss, Categorie::Base, Categorie::Jouable, Categorie::Pnj] }

    fn titre(self) -> &'static str {
        match self {
            Categorie::Boss => "Boss et chefs",
            Categorie::Base => "Créatures de base",
            Categorie::Jouable => "Jouables",
            Categorie::Pnj => "PNJ du village",
        }
    }

    fn fichier(self) -> &'static str {
        match self {
            Categorie::Boss => "boss",
            Categorie::Base => "base",
            Categorie::Jouable => "jouables",
            Categorie::Pnj => "pnj",
        }
    }

    fn resume(self) -> &'static str {
        match self {
            Categorie::Boss => {
                "Les boss au bout des antres de boss, les chefs qui gardent l'objectif « tuer le chef », et les chefs des \
                 grottes."
            },
            Categorie::Base => "Ce qu'on croise dans les salles et les couloirs des donjons, et dans les poches des grottes.",
            Categorie::Jouable => {
                "Les classes des recrues de la charrette, et les espèces qu'une recrue peut avoir (tirée au hasard)."
            },
            Categorie::Pnj => "Les habitants du village et ceux qui tiennent ses bâtiments.",
        }
    }

    fn icone(self) -> &'static str {
        match self {
            Categorie::Boss => "voxygen/element/ui/generic/icons/skull_2.png",
            Categorie::Base => "voxygen/element/ui/map/buttons/dungeon.png",
            Categorie::Jouable => "voxygen/element/ui/map/buttons/character.png",
            Categorie::Pnj => "voxygen/element/ui/map/buttons/town.png",
        }
    }
}

/// Une carte de créature.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Creature {
    /// Identifiant de la capture et de l'ancre (`dungeon-gnarling-logger`).
    pub id: String,
    pub nom: String,
    pub categorie: Categorie,
    /// Groupe de la page où la carte est rangée (`Donjon : gnarlings`).
    pub groupe: String,
    /// Tous les endroits où on la croise.
    pub lieux: Vec<String>,
    pub role: String,
    pub note: String,
    /// Chemin d'entité de Veloren (`common.entity.…`), vide si aucun.
    pub entite: String,
    /// Icône principale et icône du rôle, chemins sous `assets/`.
    pub icone: String,
    pub icone_role: String,
}

#[derive(Debug, Deserialize, Default)]
struct Complement {
    #[serde(default)]
    pnj: Vec<Pnj>,
    #[serde(default)]
    notes: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Pnj {
    id: String,
    nom: String,
    role: String,
    icone: String,
    #[serde(default)]
    entite: String,
    #[serde(default)]
    note: String,
}

/// Les noms français de `name.ftl` : clé → forme masculine (ou la valeur d'une ligne simple).
pub fn lire_noms(ftl: &str) -> BTreeMap<String, String> {
    let mut noms = BTreeMap::new();
    let mut cle: Option<String> = None;
    for ligne in ftl.lines() {
        let ligne = ligne.trim_end_matches('\r');
        if ligne.starts_with('#') || ligne.trim().is_empty() {
            continue;
        }
        if !ligne.starts_with(' ') {
            if let Some((k, v)) = ligne.split_once('=') {
                let (k, v) = (k.trim().to_string(), v.trim());
                if !v.is_empty() {
                    noms.insert(k.clone(), v.to_string());
                }
                cle = Some(k);
            }
        } else if let Some(k) = &cle {
            let l = ligne.trim();
            if let Some(v) = l.strip_prefix(".masc").and_then(|r| r.trim_start().strip_prefix('=')) {
                noms.insert(k.clone(), v.trim().to_string());
            } else if let Some(v) = l.strip_prefix(".fem").and_then(|r| r.trim_start().strip_prefix('=')) {
                noms.entry(k.clone()).or_insert_with(|| v.trim().to_string());
            }
        }
    }
    noms
}

/// Le nom français d'une entité d'après le texte de son fichier : `Translate("clé")`, sinon le nom de l'espèce
/// (`RandomWith("espèce")` → `name-body-<corps>-<espèce>`), sinon `repli`.
pub fn nom_d_entite(texte: &str, noms: &BTreeMap<String, String>, repli: &str) -> String {
    static TRADUIT: OnceLock<Regex> = OnceLock::new();
    static ESPECE: OnceLock<Regex> = OnceLock::new();
    let traduit = TRADUIT.get_or_init(|| Regex::new(r#"name:\s*Translate\("([^"]+)"\)"#).expect("regex"));
    let espece = ESPECE.get_or_init(|| Regex::new(r#"body:\s*RandomWith\("([^"]+)"\)"#).expect("regex"));
    if let Some(n) = traduit.captures(texte).and_then(|c| noms.get(&c[1])) {
        return n.clone();
    }
    if let Some(c) = espece.captures(texte) {
        let fin = format!("-{}", &c[1]);
        if let Some((_, n)) = noms.iter().find(|(k, _)| k.starts_with("name-body-") && k.ends_with(&fin)) {
            return n.clone();
        }
    }
    repli.to_string()
}

/// L'identifiant d'une entité : son chemin sans `common.entity.`, les points en tirets.
pub fn id_d_entite(entite: &str) -> String { entite.trim_start_matches("common.entity.").replace('.', "-") }

fn chercher<'a>(entrees: &'a [Entree], cle: &str) -> Option<&'a Entree> { entrees.iter().find(|e| e.cle == cle) }

/// Les chaînes `"…"` d'une valeur (`["a", "b"]` ou `"a"`).
fn chaines(valeur: &str) -> Vec<String> {
    static CHAINE: OnceLock<Regex> = OnceLock::new();
    let chaine = CHAINE.get_or_init(|| Regex::new(r#""([^"]+)""#).expect("regex"));
    chaine.captures_iter(valeur).map(|c| c[1].to_string()).collect()
}

fn nom_de_faction(faction: &str) -> String {
    match faction {
        "gnarling" => "gnarlings".into(),
        "adlet" => "adlets".into(),
        "cultist" => "cultistes".into(),
        f => f.into(),
    }
}

fn icone_de_faction(faction: &str) -> &'static str {
    match faction {
        "gnarling" => "voxygen/element/ui/map/buttons/gnarling.png",
        "adlet" => "voxygen/element/ui/map/buttons/adlet.png",
        "cultist" => "voxygen/element/ui/map/buttons/mindflayer.png",
        _ => "voxygen/element/ui/map/buttons/dungeon.png",
    }
}

/// Libellé, phrase et icône d'un rôle de donjon (`generation.ron`, d'après `design/creatures.md`).
fn role_de_donjon(role: &str) -> (&'static str, &'static str, &'static str) {
    match role {
        "brute" => (
            "Brute",
            "Corps à corps, frappe fort ; flâne autour de son poste et dort à la caserne.",
            "voxygen/element/weapons/axe.png",
        ),
        "archer" => (
            "Archer",
            "Tire de loin, préfère les hauteurs et crie à l'aide quand il faiblit.",
            "voxygen/element/weapons/bow.png",
        ),
        "lookout" => (
            "Guetteur",
            "Garde son poste en balayant la salle du regard, et va chercher des renforts.",
            "voxygen/element/weapons/daggers.png",
        ),
        "beast" => (
            "Bête",
            "Dort en meute au nid ; le bruit la réveille.",
            "voxygen/element/de_buffs/debuff_bleed_0.png",
        ),
        "chief" => (
            "Chef",
            "Garde la salle de l'objectif : le tuer réussit la mission « tuer le chef ».",
            "voxygen/element/ui/generic/icons/skull_2.png",
        ),
        "boss" => (
            "Boss",
            "Attend dans l'antre de boss, au bout du chemin principal.",
            "voxygen/element/ui/generic/icons/skull_2.png",
        ),
        _ => ("Créature", "", "voxygen/element/ui/map/buttons/unknown.png"),
    }
}

/// Ajoute une créature, ou un lieu à celle qui a la même entité.
fn ajouter(liste: &mut Vec<Creature>, creature: Creature) {
    match liste.iter_mut().find(|c| !c.entite.is_empty() && c.entite == creature.entite) {
        Some(c) => {
            for l in creature.lieux {
                if !c.lieux.contains(&l) {
                    c.lieux.push(l);
                }
            }
        },
        None => liste.push(creature),
    }
}

/// Les entrées de données utiles, pour pouvoir tester sans le dépôt.
pub struct Donnees<'a> {
    pub reglages: &'a [Fichier],
    /// Texte de `classes.ron`.
    pub classes: &'a str,
    pub noms: &'a BTreeMap<String, String>,
    /// Le texte d'un fichier d'entité d'après son chemin (`common.entity.…`).
    pub entite: &'a dyn Fn(&str) -> Option<String>,
    /// Texte de `creatures.toml`.
    pub complement: &'a str,
}

/// Toutes les créatures, dans l'ordre des pages.
pub fn collecter(d: &Donnees) -> Result<Vec<Creature>> {
    let complement: Complement = toml::from_str(d.complement).context("lecture de wiki/creatures.toml")?;
    let note = |entite: &str, defaut: &str| complement.notes.get(entite).cloned().unwrap_or_else(|| defaut.to_string());
    let nom = |entite: &str| {
        let repli = entite.rsplit('.').next().unwrap_or(entite);
        match (d.entite)(entite) {
            Some(texte) => nom_d_entite(&texte, d.noms, repli),
            None => {
                eprintln!("créatures : entité introuvable, {entite}");
                repli.to_string()
            },
        }
    };
    let fichier = |chemin: &str| d.reglages.iter().find(|f| f.chemin == chemin);
    let mut liste: Vec<Creature> = Vec::new();

    // Donjon généré : les boss d'abord, puis les chefs, puis les rôles de base.
    let factions = fichier("donjons/generation.ron")
        .and_then(|f| chercher(&f.entrees, "factions"))
        .map(|e| e.enfants.as_slice())
        .unwrap_or_default();
    for roles in [&["boss"][..], &["chief"], &["brute", "archer", "lookout", "beast"]] {
        for faction in factions {
            let Some(creatures) = chercher(&faction.enfants, "creatures") else { continue };
            for role in roles {
                let Some(entree) = chercher(&creatures.enfants, role) else { continue };
                let (libelle, phrase, icone_role) = role_de_donjon(role);
                for entite in chaines(&entree.valeur) {
                    let lieu = format!("Donjon : {}", nom_de_faction(&faction.cle));
                    ajouter(&mut liste, Creature {
                        id: id_d_entite(&entite),
                        nom: nom(&entite),
                        categorie: if matches!(*role, "boss" | "chief") { Categorie::Boss } else { Categorie::Base },
                        groupe: lieu.clone(),
                        lieux: vec![lieu],
                        role: libelle.into(),
                        note: note(&entite, phrase),
                        icone: icone_de_faction(&faction.cle).into(),
                        icone_role: icone_role.into(),
                        entite,
                    });
                }
            }
        }
    }

    // Grotte : le chef de chaque ambiance, puis ses créatures.
    let ambiances = fichier("grottes/grotte.ron")
        .and_then(|f| chercher(&f.entrees, "ambiances"))
        .map(|e| e.enfants.as_slice())
        .unwrap_or_default();
    for chefs in [true, false] {
        for ambiance in ambiances {
            let lieu = format!("Grotte : {}", ambiance.cle);
            let cle = if chefs { "chef" } else { "creatures" };
            let Some(entree) = chercher(&ambiance.enfants, cle) else { continue };
            for entite in chaines(&entree.valeur) {
                let paisible = entite.contains(".peaceful.");
                let (role, phrase, icone_role) = if chefs {
                    ("Chef de grotte", "Le chef de la grotte, à affronter au fond.", "voxygen/element/ui/generic/icons/skull_2.png")
                } else if paisible {
                    ("Paisible", "Habite les poches de la grotte et n'attaque pas.", "voxygen/element/ui/map/buttons/tree.png")
                } else {
                    ("Sauvage", "Habite les poches de la grotte et attaque à vue.", "voxygen/element/ui/map/buttons/excl.png")
                };
                ajouter(&mut liste, Creature {
                    id: id_d_entite(&entite),
                    nom: nom(&entite),
                    categorie: if chefs { Categorie::Boss } else { Categorie::Base },
                    groupe: lieu.clone(),
                    lieux: vec![lieu.clone()],
                    role: role.into(),
                    note: note(&entite, phrase),
                    icone: "voxygen/element/ui/map/buttons/cave.png".into(),
                    icone_role: icone_role.into(),
                    entite,
                });
            }
        }
    }

    // Jouables : les classes, puis les espèces humanoïdes de Veloren (une recrue a un corps au hasard).
    static CLASSE: OnceLock<Regex> = OnceLock::new();
    let classe = CLASSE.get_or_init(|| {
        Regex::new(r#"\(id:\s*"([^"]+)",\s*nom:\s*"([^"]+)",\s*equipement:\s*"([^"]+)""#).expect("regex")
    });
    for c in classe.captures_iter(d.classes) {
        let nom: String = c[2].to_string();
        let icone = crate::icones::pour_texte(&nom).unwrap_or("voxygen/element/ui/map/buttons/character.png");
        let mut majuscule = nom.chars();
        let nom = majuscule.next().map(|p| p.to_uppercase().chain(majuscule).collect()).unwrap_or_default();
        liste.push(Creature {
            id: format!("classe-{}", &c[1]),
            nom,
            categorie: Categorie::Jouable,
            groupe: "Classes".into(),
            lieux: vec!["Charrette du village".into()],
            role: "Classe".into(),
            note: note(&format!("classe.{}", &c[1]), &format!("Équipement de départ : `{}` (classes.ron).", &c[3])),
            entite: String::new(),
            icone: icone.into(),
            icone_role: icone.into(),
        });
    }
    for (cle, nom) in d.noms.iter().filter(|(k, _)| k.starts_with("name-body-humanoid-")) {
        let espece = cle.trim_start_matches("name-body-humanoid-");
        liste.push(Creature {
            id: format!("espece-{espece}"),
            nom: nom.clone(),
            categorie: Categorie::Jouable,
            groupe: "Espèces".into(),
            lieux: vec!["Recrues".into()],
            role: "Espèce".into(),
            note: note(
                &format!("espece.{espece}"),
                "Espèce humanoïde de Veloren : même squelette, n'importe quelle arme, mêmes stats de base.",
            ),
            entite: String::new(),
            icone: "voxygen/element/ui/map/buttons/character.png".into(),
            icone_role: "voxygen/element/ui/map/buttons/character.png".into(),
        });
    }

    // PNJ du village.
    for p in &complement.pnj {
        liste.push(Creature {
            id: format!("pnj-{}", p.id),
            nom: p.nom.clone(),
            categorie: Categorie::Pnj,
            groupe: "Village".into(),
            lieux: vec!["Village".into()],
            role: p.role.clone(),
            note: p.note.clone(),
            entite: p.entite.clone(),
            icone: p.icone.clone(),
            icone_role: "voxygen/element/ui/map/buttons/town.png".into(),
        });
    }
    Ok(liste)
}

/// Une carte prête pour le gabarit : chemins relatifs à la racine du site.
#[derive(Debug, Serialize)]
struct CarteVue<'a> {
    #[serde(flatten)]
    creature: &'a Creature,
    note_html: String,
    /// Capture dans le site (`creatures/images/x.png`), si le fichier existe.
    capture: Option<String>,
    /// Où déposer la capture, sinon.
    capture_a_deposer: Option<String>,
}

#[derive(Debug, Serialize)]
struct GroupeVue<'a> {
    titre: String,
    cartes: Vec<CarteVue<'a>>,
}

/// L'URL de la page d'une catégorie.
pub fn url(categorie: Categorie) -> String { format!("creatures/{}.html", categorie.fichier()) }

/// Une page par catégorie, dans la rubrique « Créatures » ; `captures` : le dossier des captures (absolu).
pub fn pages(creatures: &[Creature], captures: &Path) -> Vec<crate::Page> {
    Categorie::toutes()
        .into_iter()
        .enumerate()
        .map(|(ordre, categorie)| {
            let mut images: Vec<(String, String)> = Vec::new();
            let mut copier = |source: String, cible: String| {
                if !images.iter().any(|(_, c)| *c == cible) {
                    images.push((source, cible));
                }
            };
            copier(format!("assets/{}", categorie.icone()), format!("assets/{}", categorie.icone()));
            let mut groupes: Vec<GroupeVue> = Vec::new();
            for c in creatures.iter().filter(|c| c.categorie == categorie) {
                copier(format!("assets/{}", c.icone), format!("assets/{}", c.icone));
                copier(format!("assets/{}", c.icone_role), format!("assets/{}", c.icone_role));
                let fichier = format!("{}.png", c.id);
                let present = captures.join(&fichier).is_file();
                let capture = present.then(|| format!("creatures/images/{fichier}"));
                if let Some(cible) = &capture {
                    copier(format!("{DOSSIER_CAPTURES}{fichier}"), cible.clone());
                }
                let carte = CarteVue {
                    creature: c,
                    note_html: crate::markdown::rendre_inline(&c.note),
                    capture,
                    capture_a_deposer: (!present).then(|| format!("{DOSSIER_CAPTURES}{fichier}")),
                };
                match groupes.iter_mut().find(|g| g.titre == c.groupe) {
                    Some(g) => g.cartes.push(carte),
                    None => groupes.push(GroupeVue { titre: c.groupe.clone(), cartes: vec![carte] }),
                }
            }
            let nombre: usize = groupes.iter().map(|g| g.cartes.len()).sum();
            crate::Page {
                source: format!("wiki/creatures/{}", categorie.fichier()),
                url: url(categorie),
                rubrique: crate::Rubrique::Creatures,
                titre: categorie.titre().to_string(),
                html: String::new(),
                gabarit: "creatures.html",
                extra: serde_json::json!({
                    "titre": categorie.titre(),
                    "resume": categorie.resume(),
                    "icone": categorie.icone(),
                    "nombre": nombre,
                    "groupes": groupes,
                    "dossier_captures": DOSSIER_CAPTURES,
                }),
                ordre,
                icone: Some(categorie.icone().to_string()),
                images,
            }
        })
        .collect()
}

/// Lit les sources du dépôt et rend les quatre pages.
pub fn pages_du_depot(racine: &Path, reglages: &[Fichier]) -> Result<Vec<crate::Page>> {
    let lire = |chemin: &str| std::fs::read_to_string(racine.join(chemin)).with_context(|| format!("lecture de {chemin}"));
    let noms = lire_noms(&lire("assets/voxygen/i18n/fr/name.ftl")?);
    let classes = lire("assets/icebreaker/classes.ron")?;
    let complement = lire("docs/icebreaker/wiki/creatures.toml")?;
    let entite = |chemin: &str| {
        let relatif = chemin.trim_start_matches("common.entity.").replace('.', "/");
        std::fs::read_to_string(racine.join("assets/common/entity").join(format!("{relatif}.ron"))).ok()
    };
    let creatures = collecter(&Donnees {
        reglages,
        classes: &classes,
        noms: &noms,
        entite: &entite,
        complement: &complement,
    })?;
    Ok(pages(&creatures, &racine.join(DOSSIER_CAPTURES)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FTL: &str = "## Noms\nname-custom-dungeon-gnarling-logger =\n    .fem = Bûcheronne Teigneuse\n    .masc = \
                       Bûcheron Teigneux\nname-body-biped_large-troll_cave =\n    .fem = Troll des cavernes\n    .masc \
                       = Troll des cavernes\nname-body-humanoid-dwarf =\n    .fem = Naine\n    .masc = Nain\nname-simple \
                       = Simple\n";

    #[test]
    fn noms_francais() {
        let noms = lire_noms(FTL);
        assert_eq!(noms["name-simple"], "Simple");
        assert_eq!(noms["name-body-humanoid-dwarf"], "Nain");
        let logger = "(\n    name: Translate(\"name-custom-dungeon-gnarling-logger\"),\n    body: RandomWith(\"gnarling\"),\n)";
        assert_eq!(nom_d_entite(logger, &noms, "logger"), "Bûcheron Teigneux");
        let troll = "(\n    name: Automatic,\n    body: RandomWith(\"troll_cave\"),\n)";
        assert_eq!(nom_d_entite(troll, &noms, "cave_troll"), "Troll des cavernes");
        assert_eq!(nom_d_entite("(name: Automatic)", &noms, "repli"), "repli");
        assert_eq!(id_d_entite("common.entity.dungeon.gnarling.logger"), "dungeon-gnarling-logger");
    }

    const GENERATION: &str = "(\n    factions: {\n        \"gnarling\": (\n            creatures: {\n                \
                              \"brute\": [\"common.entity.dungeon.gnarling.logger\"],\n                \"chief\": \
                              [\"common.entity.dungeon.gnarling.chieftain\"],\n                \"boss\": \
                              [\"common.entity.dungeon.gnarling.harvester\"],\n            },\n            chest: \
                              \"DungeonChest0\",\n        ),\n    },\n)\n";
    const GROTTE: &str = "(\n    ambiances: {\n        \"cristaux\": (\n            creatures: \
                          [\"common.entity.wild.aggressive.cave_spider\", \"common.entity.wild.peaceful.bat\"],\n            \
                          chef: \"common.entity.wild.aggressive.basilisk\",\n        ),\n        \"ossuaire\": (\n            \
                          creatures: [\"common.entity.wild.aggressive.cave_spider\"],\n            chef: \
                          \"common.entity.wild.aggressive.ogre\",\n        ),\n    },\n)\n";
    const CLASSES: &str = "(\n    classes: [\n        (id: \"guerrier\", nom: \"guerrier\", equipement: \
                           \"icebreaker.classes.guerrier\",\n            competences: \"x\"),\n    ],\n)\n";
    const COMPLEMENT: &str = "[[pnj]]\nid = \"intendant\"\nnom = \"Intendant\"\nrole = \"Marchand\"\nicone = \
                              \"x.png\"\n\n[notes]\n\"common.entity.dungeon.gnarling.harvester\" = \"Le boss.\"\n";

    fn exemple() -> Vec<Creature> {
        let reglages = vec![
            crate::reglages::analyser(GENERATION, "donjons/generation.ron"),
            crate::reglages::analyser(GROTTE, "grottes/grotte.ron"),
        ];
        let noms = lire_noms(FTL);
        let entite = |c: &str| {
            (c == "common.entity.dungeon.gnarling.logger")
                .then(|| "name: Translate(\"name-custom-dungeon-gnarling-logger\"),".to_string())
        };
        collecter(&Donnees {
            reglages: &reglages,
            classes: CLASSES,
            noms: &noms,
            entite: &entite,
            complement: COMPLEMENT,
        })
        .unwrap()
    }

    #[test]
    fn categories_et_lieux() {
        let liste = exemple();
        let c = |id: &str| liste.iter().find(|c| c.id == id).unwrap_or_else(|| panic!("{id} absent"));
        assert_eq!(liste[0].id, "dungeon-gnarling-harvester"); // les boss d'abord
        assert_eq!(c("dungeon-gnarling-harvester").categorie, Categorie::Boss);
        assert_eq!(c("dungeon-gnarling-harvester").note, "Le boss.");
        assert_eq!(c("dungeon-gnarling-chieftain").role, "Chef");
        assert_eq!(c("dungeon-gnarling-logger").categorie, Categorie::Base);
        assert_eq!(c("dungeon-gnarling-logger").nom, "Bûcheron Teigneux");
        assert_eq!(c("dungeon-gnarling-logger").lieux, vec!["Donjon : gnarlings"]);
        assert_eq!(c("wild-aggressive-basilisk").categorie, Categorie::Boss);
        assert_eq!(c("wild-peaceful-bat").role, "Paisible");
        // L'araignée est dans deux ambiances : une carte, deux lieux.
        assert_eq!(liste.iter().filter(|c| c.id == "wild-aggressive-cave_spider").count(), 1);
        assert_eq!(c("wild-aggressive-cave_spider").lieux, vec!["Grotte : cristaux", "Grotte : ossuaire"]);
        assert_eq!(c("classe-guerrier").nom, "Guerrier");
        assert_eq!(c("classe-guerrier").icone, "voxygen/element/weapons/sword.png");
        assert_eq!(c("espece-dwarf").nom, "Nain");
        assert_eq!(c("pnj-intendant").categorie, Categorie::Pnj);
    }

    #[test]
    fn capture_presente_ou_a_prendre() {
        let liste = exemple();
        let dossier = std::env::temp_dir().join(format!("icebreaker-wiki-creatures-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        std::fs::write(dossier.join("dungeon-gnarling-harvester.png"), b"png").unwrap();
        let pages = pages(&liste, &dossier);
        assert_eq!(pages.len(), 4);
        let boss = &pages[0];
        assert_eq!(boss.url, "creatures/boss.html");
        let cartes = &boss.extra["groupes"][0]["cartes"];
        assert_eq!(cartes[0]["capture"], "creatures/images/dungeon-gnarling-harvester.png");
        assert!(cartes[1]["capture"].is_null());
        assert_eq!(
            cartes[1]["capture_a_deposer"],
            "docs/icebreaker/wiki/images/creatures/dungeon-gnarling-chieftain.png"
        );
        assert!(boss.images.iter().any(|(s, c)| s.ends_with("creatures/dungeon-gnarling-harvester.png")
            && c == "creatures/images/dungeon-gnarling-harvester.png"));
        std::fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn pages_du_vrai_depot() {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let reglages = crate::reglages::lire_dossier(&racine.join("assets/icebreaker")).unwrap();
        let pages = pages_du_depot(&racine, &reglages).unwrap();
        let nombre = |i: usize| pages[i].extra["nombre"].as_u64().unwrap();
        assert!(nombre(0) >= 6, "boss et chefs : {}", nombre(0)); // 3 boss, 3 chefs de donjon, chefs de grotte
        assert!(nombre(1) >= 12);
        assert!(nombre(2) >= 3 + 6); // trois classes, six espèces
        assert!(nombre(3) >= 1);
        for p in &pages {
            for (source, _) in &p.images {
                assert!(racine.join(source).is_file(), "{source} introuvable");
            }
        }
    }
}
