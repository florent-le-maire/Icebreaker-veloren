//! Icebreaker : les classes, leurs jeux d'armes et leurs compétences dans le wiki (A.18).
//!
//! Lit les données du jalon A (`docs/icebreaker/PLAN-CLASSES.md`) : `assets/icebreaker/jeux_armes/*.ron` (un fichier
//! par jeu d'armes, avec sa classe), `ultimes/<classe>.ron` et `passifs/<classe>.ron`. Toutes les classes trouvées
//! sont prises, aucune n'est nommée ici. Le format est celui de `common/src/icebreaker_competences.rs` ; on le lit
//! avec le petit lecteur de RON de `competences.rs`, sans la crate `ron`.
//!
//! Sert deux choses : la page « Classes et compétences » de la rubrique Compétences, et le lien capacité de
//! Veloren → « Guerrier · grande épée : Tourbillon » de ses tableaux.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::Result;
use serde::Serialize;

use crate::competences::{champs, chaine, dedans, sans_commentaires, sans_some, separer};

/// Un niveau de 2 à 5 d'une compétence.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NiveauVue {
    pub niveau: u8,
    pub texte: String,
    /// La compétence se transforme à ce niveau (capacité de remplacement).
    pub transforme: bool,
    /// Recharge à ce niveau, en secondes (celle du niveau 1 fois le multiplicateur du niveau).
    pub recharge: String,
}

/// Une compétence d'arme ou un ultime.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CompetenceVue {
    pub id: String,
    pub nom: String,
    pub description: String,
    /// Id du fichier de capacité de Veloren que la compétence lance au niveau 1.
    pub capacite: String,
    pub recharge: String,
    pub niveaux: Vec<NiveauVue>,
}

/// Un jeu d'armes : ce qu'on tient en mains et le pool de compétences qui va avec.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct JeuVue {
    pub id: String,
    pub nom: String,
    /// Chemin d'une icône sous `assets/`.
    pub icone: String,
    pub armes: Vec<String>,
    pub competences: Vec<CompetenceVue>,
}

/// Un passif au choix.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PassifVue {
    pub id: String,
    pub nom: String,
    pub description: String,
    pub effets: Vec<String>,
}

/// Une classe d'Icebreaker, telle que les données la décrivent.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Classe {
    pub id: String,
    pub nom: String,
    pub icone: String,
    pub inne: Vec<String>,
    pub passifs: Vec<PassifVue>,
    pub ultimes: Vec<CompetenceVue>,
    pub jeux: Vec<JeuVue>,
}

/// Les textes bruts des fichiers de données : (id = nom du fichier sans `.ron`, texte).
#[derive(Debug, Default, Clone)]
pub struct Sources {
    pub jeux: Vec<(String, String)>,
    pub ultimes: Vec<(String, String)>,
    pub passifs: Vec<(String, String)>,
    /// id d'objet de Veloren → nom français, pour les armes des jeux d'armes.
    pub noms_objets: BTreeMap<String, String>,
}

fn majuscule(texte: &str) -> String {
    let mut c = texte.chars();
    c.next().map(|p| p.to_uppercase().chain(c).collect()).unwrap_or_default()
}

fn minuscule(texte: &str) -> String {
    let mut c = texte.chars();
    c.next().map(|p| p.to_lowercase().chain(c).collect()).unwrap_or_default()
}

/// `6.0` → « 6 s », `5.4` → « 5,4 s ».
fn secondes(s: f32) -> String {
    let arrondi = (s * 10.0).round() / 10.0;
    if (arrondi - arrondi.round()).abs() < 1e-6 {
        format!("{} s", arrondi.round() as i64)
    } else {
        format!("{arrondi:.1} s").replace('.', ",")
    }
}

/// Le contenu des parenthèses (ou des crochets) d'un texte entier.
fn contenu(texte: &str) -> &str { dedans(texte, "").unwrap_or("") }

/// Les champs au niveau 0 d'un fichier de données `( … )`.
fn champs_du_fichier(texte: &str) -> Vec<(String, String)> {
    let propre = sans_commentaires(texte);
    champs(contenu(&propre)).into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn champ<'a>(liste: &'a [(String, String)], cle: &str) -> Option<&'a str> {
    liste.iter().find(|(k, _)| k == cle).map(|(_, v)| v.as_str())
}

/// Une chaîne RON : sans guillemets, avec `\"` rendu.
fn texte(v: &str) -> String { chaine(sans_some(v).unwrap_or("")).replace("\\\"", "\"") }

/// Une compétence `( nom: …, description: …, capacite: …, recharge: …, niveaux: […] )`.
fn competence(id: &str, corps: &str) -> CompetenceVue {
    let c: Vec<(String, String)> =
        champs(contenu(corps)).into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let recharge = champ(&c, "recharge").and_then(|r| r.parse::<f32>().ok()).unwrap_or(0.0);
    let niveaux = champ(&c, "niveaux")
        .map(|n| {
            separer(contenu(n))
                .into_iter()
                .enumerate()
                .map(|(i, niveau)| {
                    let n: Vec<(String, String)> =
                        champs(contenu(niveau)).into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
                    let multiplicateur = champ(&n, "recharge").and_then(|r| r.parse::<f32>().ok()).unwrap_or(1.0);
                    NiveauVue {
                        niveau: i as u8 + 2,
                        texte: champ(&n, "texte").map(texte).unwrap_or_default(),
                        transforme: champ(&n, "capacite").is_some(),
                        recharge: secondes(recharge * multiplicateur),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    CompetenceVue {
        id: id.to_string(),
        nom: champ(&c, "nom").map(texte).unwrap_or_else(|| id.to_string()),
        description: champ(&c, "description").map(texte).unwrap_or_default(),
        capacite: champ(&c, "capacite").map(texte).unwrap_or_default(),
        recharge: secondes(recharge),
        niveaux,
    }
}

/// Une liste `[ ("id", ( … )), … ]` de compétences.
fn liste_de_competences(v: &str) -> Vec<CompetenceVue> {
    separer(contenu(v))
        .into_iter()
        .filter_map(|entree| {
            let morceaux = separer(contenu(entree));
            let [id, corps] = morceaux.as_slice() else { return None };
            Some(competence(&chaine(id), corps))
        })
        .collect()
}

/// L'icône d'un jeu d'armes : son champ `icone` (noms du jeu : epee, epees, hache…), sinon d'après son nom.
fn icone_du_jeu(icone: &str, nom: &str) -> String {
    let chemin = match icone {
        "epee" => Some("voxygen/element/weapons/sword.png"),
        "epees" => Some("voxygen/element/weapons/swordshield.png"),
        "baton" => Some("voxygen/element/weapons/staff.png"),
        "arc" => Some("voxygen/element/weapons/bow.png"),
        "hache" => Some("voxygen/element/weapons/axe.png"),
        "marteau" => Some("voxygen/element/weapons/hammer.png"),
        "sceptre" => Some("voxygen/element/weapons/sceptre.png"),
        _ => None,
    };
    chemin.or_else(|| crate::icones::pour_texte(nom)).unwrap_or("voxygen/element/weapons/swords_crossed.png").to_string()
}

/// Un jeu d'armes : (classe, vue).
fn jeu(id: &str, texte_ron: &str, noms_objets: &BTreeMap<String, String>) -> Option<(String, JeuVue)> {
    let c = champs_du_fichier(texte_ron);
    let classe = texte(champ(&c, "classe")?);
    let nom = champ(&c, "nom").map(texte).unwrap_or_else(|| id.to_string());
    let armes = champ(&c, "armes")
        .map(|a| {
            separer(contenu(a))
                .into_iter()
                .filter_map(sans_some)
                .map(|id| {
                    let id = chaine(id);
                    noms_objets.get(&id).cloned().unwrap_or(id)
                })
                .collect()
        })
        .unwrap_or_default();
    let icone = icone_du_jeu(&champ(&c, "icone").map(texte).unwrap_or_default(), &nom);
    let competences = champ(&c, "competences").map(liste_de_competences).unwrap_or_default();
    Some((classe, JeuVue { id: id.to_string(), nom, icone, armes, competences }))
}

/// `Name(a)` ou `Name(value: x, kind: K)` en phrase ; le texte brut si l'effet n'est pas connu.
fn effet(brut: &str) -> String {
    let brut = brut.trim();
    let Some((nom, reste)) = brut.split_once('(') else { return brut.to_string() };
    let args = reste.trim_end().trim_end_matches(')');
    let c = champs(args);
    let nombre = |cle: &str| c.iter().find(|(k, _)| *k == cle).and_then(|(_, v)| v.parse::<f32>().ok());
    let simple = args.trim().parse::<f32>().ok();
    let pct = |x: f32| {
        let p = (x * 1000.0).round() / 10.0;
        if (p - p.round()).abs() < 1e-6 { format!("{}", p.round() as i64) } else { format!("{p}").replace('.', ",") }
    };
    let plus = |libelle: &str, x: f32| format!("{libelle} {}{} %", if x >= 1.0 { "+" } else { "-" }, pct((x - 1.0).abs()));
    match (nom.trim(), simple, nombre("value")) {
        ("MaxHealthModifier" | "MaxEnergyModifier", _, Some(v)) => {
            let libelle = if nom.trim() == "MaxHealthModifier" { "PV max" } else { "endurance max" };
            if c.iter().any(|(k, w)| *k == "kind" && *w == "Additive") {
                format!("{libelle} {}{}", if v >= 0.0 { "+" } else { "-" }, v.abs())
            } else {
                plus(libelle, v)
            }
        },
        ("DamageReduction", Some(x), _) => format!("dégâts reçus -{} %", pct(x)),
        ("PoiseReduction", Some(x), _) => format!("déséquilibre subi -{} %", pct(x)),
        ("CrowdControlResistance", Some(x), _) => format!("résistance aux contrôles +{} %", pct(x)),
        ("AttackDamage", Some(x), _) => plus("dégâts", x),
        ("MovementSpeed", Some(x), _) => plus("vitesse de déplacement", x),
        ("AttackSpeed", Some(x), _) => plus("vitesse d'attaque", x),
        ("EnergyReward", Some(x), _) => plus("endurance rendue", x),
        ("EnergyEfficiency", Some(x), _) => plus("efficacité de l'endurance", x),
        ("PrecisionPowerMult", Some(x), _) => plus("dégâts de précision", x),
        _ => brut.to_string(),
    }
}

fn effets(v: &str) -> Vec<String> { separer(contenu(v)).into_iter().map(effet).collect() }

/// L'inné et les passifs de `passifs/<classe>.ron`.
fn passifs(texte_ron: &str) -> (Vec<String>, Vec<PassifVue>) {
    let c = champs_du_fichier(texte_ron);
    let inne = champ(&c, "inne").map(effets).unwrap_or_default();
    let liste = champ(&c, "passifs")
        .map(|p| {
            separer(contenu(p))
                .into_iter()
                .filter_map(|entree| {
                    let morceaux = separer(contenu(entree));
                    let [id, corps] = morceaux.as_slice() else { return None };
                    let p: Vec<(String, String)> =
                        champs(contenu(corps)).into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
                    Some(PassifVue {
                        id: chaine(id),
                        nom: champ(&p, "nom").map(texte).unwrap_or_else(|| chaine(id)),
                        description: champ(&p, "description").map(texte).unwrap_or_default(),
                        effets: champ(&p, "effets").map(effets).unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    (inne, liste)
}

/// Toutes les classes que les données décrivent, par ordre d'identifiant.
pub fn lire(s: &Sources) -> Vec<Classe> {
    let mut jeux: BTreeMap<String, Vec<JeuVue>> = BTreeMap::new();
    for (id, t) in &s.jeux {
        if let Some((classe, j)) = jeu(id, t, &s.noms_objets) {
            jeux.entry(classe).or_default().push(j);
        }
    }
    let ultimes: BTreeMap<&str, Vec<CompetenceVue>> = s
        .ultimes
        .iter()
        .map(|(classe, t)| {
            let c = champs_du_fichier(t);
            (classe.as_str(), champ(&c, "ultimes").map(liste_de_competences).unwrap_or_default())
        })
        .collect();
    let passifs: BTreeMap<&str, (Vec<String>, Vec<PassifVue>)> =
        s.passifs.iter().map(|(classe, t)| (classe.as_str(), passifs(t))).collect();
    let ids: BTreeSet<&str> = jeux.keys().map(String::as_str).chain(ultimes.keys().copied()).chain(passifs.keys().copied()).collect();
    ids.into_iter()
        .map(|id| {
            let jeux = jeux.get(id).cloned().unwrap_or_default();
            let (inne, passifs) = passifs.get(id).cloned().unwrap_or_default();
            let icone = jeux
                .first()
                .map(|j| j.icone.clone())
                .or_else(|| crate::icones::pour_texte(id).map(str::to_string))
                .unwrap_or_else(|| "voxygen/element/ui/map/buttons/character.png".to_string());
            Classe {
                id: id.to_string(),
                nom: majuscule(id),
                icone,
                inne,
                passifs,
                ultimes: ultimes.get(id).cloned().unwrap_or_default(),
                jeux,
            }
        })
        .collect()
}

/// Capacité de Veloren → les compétences d'Icebreaker qui la reprennent (« Guerrier · grande épée : Tourbillon »).
pub fn par_capacite(classes: &[Classe]) -> BTreeMap<String, Vec<String>> {
    let mut sortie: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in classes {
        for j in &c.jeux {
            for k in &j.competences {
                sortie
                    .entry(k.capacite.clone())
                    .or_default()
                    .push(format!("{} · {} : {}", c.nom, minuscule(&j.nom), k.nom));
            }
        }
        for u in &c.ultimes {
            sortie.entry(u.capacite.clone()).or_default().push(format!("{} · ultime : {}", c.nom, u.nom));
        }
    }
    sortie
}

/// Les textes `.ron` d'un dossier : (nom du fichier sans extension, texte), par ordre de nom.
fn lire_dossier(dossier: &Path) -> Vec<(String, String)> {
    let mut fichiers: Vec<(String, String)> = std::fs::read_dir(dossier)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ron"))
        .filter_map(|e| {
            let nom = e.path().file_stem()?.to_string_lossy().into_owned();
            Some((nom, std::fs::read_to_string(e.path()).ok()?))
        })
        .collect();
    fichiers.sort();
    fichiers
}

/// Lit `assets/icebreaker/{jeux_armes,ultimes,passifs}/` du dépôt.
pub fn lire_depot(racine: &Path, noms_objets: BTreeMap<String, String>) -> Vec<Classe> {
    let base = racine.join("assets/icebreaker");
    lire(&Sources {
        jeux: lire_dossier(&base.join("jeux_armes")),
        ultimes: lire_dossier(&base.join("ultimes")),
        passifs: lire_dossier(&base.join("passifs")),
        noms_objets,
    })
}

const ICONE: &str = "voxygen/element/weapons/swords_crossed.png";

/// La page « Classes et compétences » de la rubrique Compétences.
pub fn page(classes: &[Classe]) -> crate::Page {
    let mut images: BTreeSet<String> = BTreeSet::new();
    images.insert(ICONE.to_string());
    for c in classes {
        images.insert(c.icone.clone());
        images.extend(c.jeux.iter().map(|j| j.icone.clone()));
    }
    crate::Page {
        source: "wiki/competences/jeux-armes".into(),
        url: "competences/classes.html".into(),
        rubrique: crate::Rubrique::Competences,
        titre: "Classes et jeux d'armes".into(),
        html: String::new(),
        gabarit: "classes.html",
        extra: serde_json::json!({
            "titre": "Classes et jeux d'armes d'Icebreaker",
            "resume": "Pour chaque classe : son inné, ses passifs, ses ultimes, puis ses jeux d'armes avec leurs \
                       compétences, leur recharge et ce que chaque niveau apporte. Lu dans les données, à jour à \
                       chaque génération du wiki.",
            "icone": ICONE,
            "classes": classes,
            "nombre": classes.iter().map(|c| c.jeux.iter().map(|j| j.competences.len()).sum::<usize>()).sum::<usize>(),
        }),
        ordre: 0,
        icone: Some(ICONE.into()),
        images: images.into_iter().map(|i| (format!("assets/{i}"), format!("assets/{i}"))).collect(),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub const JEU: &str = r#"// commentaire
#![enable(implicit_some)]
(
    nom: "Grande épée",
    classe: "guerrier",
    icone: "epee",
    defaut: true,
    main_droite: (arme: Sword, mains: Two),
    armes: ("common.items.weapons.sword.starter", None),
    competences: [
        ("tourbillon", (
            nom: "Tourbillon",
            description: "Deux tours d'épée, et \"tout\" autour.",
            capacite: "common.abilities.sword.cleaving_whirlwind_slice",
            recharge: 10.0,
            niveaux: [
                (puissance: 1.1, texte: "10 % de dégâts en plus."),
                (puissance: 1.0, capacite: "icebreaker.abilities.x_3", texte: "Un troisième tour, et la zone s'élargit."),
                (puissance: 1.1, recharge: 0.9, texte: "Recharge plus courte, mais pas trop."),
                (capacite: "icebreaker.abilities.x_5", recharge: 0.85, texte: "Les ennemis sont ralentis."),
            ],
        )),
        ("frappe", (
            nom: "Frappe",
            capacite: "common.abilities.sword.basic_fell_strike",
            recharge: 5.0,
        )),
    ],
)
"#;

    pub const ULTIMES: &str = r#"#![enable(implicit_some)]
(
    ultimes: [
        ("fureur", (nom: "Fureur", description: "Quinze secondes de fureur.", capacite: "common.abilities.axe.furor", recharge: 90.0)),
    ],
)
"#;

    pub const PASSIFS: &str = r#"// inné
(
    inne: [
        MaxHealthModifier(value: 1.25, kind: Multiplicative),
        PoiseReduction(0.2),
    ],
    passifs: [
        ("peau_dure", (nom: "Peau dure", description: "10 % de dégâts reçus en moins.", effets: [DamageReduction(0.1)])),
        ("inconnu", (nom: "Inconnu", effets: [Bizarre(3)])),
    ],
)
"#;

    pub fn exemple() -> Vec<Classe> {
        lire(&Sources {
            jeux: vec![("grande_epee".into(), JEU.into())],
            ultimes: vec![("guerrier".into(), ULTIMES.into())],
            passifs: vec![("guerrier".into(), PASSIFS.into())],
            noms_objets: [("common.items.weapons.sword.starter".to_string(), "Épée de départ".to_string())].into(),
        })
    }

    #[test]
    fn jeu_d_armes_avec_niveaux() {
        let classes = exemple();
        assert_eq!(classes.len(), 1);
        let g = &classes[0];
        assert_eq!((g.id.as_str(), g.nom.as_str()), ("guerrier", "Guerrier"));
        let j = &g.jeux[0];
        assert_eq!(j.nom, "Grande épée");
        assert_eq!(j.armes, vec!["Épée de départ"]);
        assert_eq!(j.icone, "voxygen/element/weapons/sword.png");
        assert_eq!(j.competences.len(), 2);
        let t = &j.competences[0];
        assert_eq!(t.nom, "Tourbillon");
        assert_eq!(t.description, "Deux tours d'épée, et \"tout\" autour.");
        assert_eq!(t.recharge, "10 s");
        assert_eq!(t.niveaux.len(), 4);
        assert_eq!(t.niveaux[0], NiveauVue { niveau: 2, texte: "10 % de dégâts en plus.".into(), transforme: false, recharge: "10 s".into() });
        assert!(t.niveaux[1].transforme);
        assert_eq!(t.niveaux[1].texte, "Un troisième tour, et la zone s'élargit.");
        assert_eq!((t.niveaux[2].niveau, t.niveaux[2].recharge.as_str()), (4, "9 s"));
        assert_eq!((t.niveaux[3].niveau, t.niveaux[3].recharge.as_str()), (5, "8,5 s"));
        assert!(j.competences[1].niveaux.is_empty());
        assert_eq!(j.competences[1].description, "");
    }

    #[test]
    fn ultimes_passifs_et_inne() {
        let g = &exemple()[0];
        assert_eq!(g.ultimes.len(), 1);
        assert_eq!((g.ultimes[0].nom.as_str(), g.ultimes[0].recharge.as_str()), ("Fureur", "90 s"));
        assert_eq!(g.inne, vec!["PV max +25 %", "déséquilibre subi -20 %"]);
        assert_eq!(g.passifs.len(), 2);
        assert_eq!(g.passifs[0].nom, "Peau dure");
        assert_eq!(g.passifs[0].effets, vec!["dégâts reçus -10 %"]);
        assert_eq!(g.passifs[1].effets, vec!["Bizarre(3)"]);
    }

    #[test]
    fn capacites_reprises() {
        let r = par_capacite(&exemple());
        assert_eq!(r["common.abilities.sword.cleaving_whirlwind_slice"], vec!["Guerrier · grande épée : Tourbillon"]);
        assert_eq!(r["common.abilities.axe.furor"], vec!["Guerrier · ultime : Fureur"]);
        assert!(!r.contains_key("icebreaker.abilities.x_3"));
    }

    #[test]
    fn classe_sans_jeu_ni_nom_en_dur() {
        // Une classe qui n'a que des passifs (mage et archer, avant leurs jeux d'armes) apparaît quand même.
        let classes = lire(&Sources { passifs: vec![("mage".into(), PASSIFS.into())], ..Sources::default() });
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].nom, "Mage");
        assert!(classes[0].jeux.is_empty() && classes[0].ultimes.is_empty());
    }

    #[test]
    fn page_rendue() {
        let p = page(&exemple());
        assert_eq!(p.url, "competences/classes.html");
        assert_eq!(p.extra["nombre"], 2);
        assert!(p.images.iter().any(|(s, _)| s.ends_with("weapons/sword.png")));
    }

    #[test]
    fn classes_du_vrai_depot() {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let classes = lire_depot(&racine, BTreeMap::new());
        let guerrier = classes.iter().find(|c| c.id == "guerrier").expect("guerrier");
        assert!(!guerrier.jeux.is_empty() && !guerrier.ultimes.is_empty() && !guerrier.passifs.is_empty());
        assert!(!guerrier.inne.is_empty());
        for c in &classes {
            for j in &c.jeux {
                assert!(!j.armes.is_empty(), "{} sans armes", j.id);
                for k in &j.competences {
                    assert!(!k.nom.is_empty() && !k.capacite.is_empty() && k.recharge != "0 s", "{}", k.id);
                    assert!(!k.description.is_empty(), "{} sans description", k.id);
                    assert_eq!(k.niveaux.len(), 4, "{} : niveaux 2 à 5", k.id);
                    assert!(k.niveaux.iter().all(|n| !n.texte.is_empty()), "{} : texte de niveau", k.id);
                }
            }
            for u in &c.ultimes {
                assert!(!u.nom.is_empty() && !u.description.is_empty() && !u.capacite.is_empty(), "{}", u.id);
            }
            for p in &c.passifs {
                assert!(!p.nom.is_empty() && !p.effets.is_empty(), "{}", p.id);
            }
        }
        let tout = par_capacite(&classes);
        assert!(tout.values().flatten().any(|l| l.starts_with("Guerrier · ")));
    }
}
