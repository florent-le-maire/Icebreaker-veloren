//! Icebreaker : lecture des chiffres du jeu, avec les structures du jeu (`common`, `world`).
//! Tout part des fichiers de `assets/` : rien n'est recopié ici.

use anyhow::{Context, Result, anyhow};
use common::{
    assets::{AssetExt, Ron},
    combat::Damage,
    comp::{
        Inventory,
        ability::CharacterAbility,
        buff::BuffEffect,
        inventory::{loadout_builder::LoadoutBuilder, slot::EquipSlot},
        item::{
            Item, ItemKind, MaterialStatManifest,
            tool::{AbilityKind, AbilityMap, AbilitySpec},
        },
    },
    generation::EntityInfo,
    icebreaker_combat::IcebreakerCombat,
    icebreaker_competences::{Competence, SalleArmes, registre},
};
use serde::Deserialize;
use serde_json::Value;
use vek::Vec3;

/// Le type d'une attaque, pour le multiplicateur de `combat.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Source {
    Melee,
    Projectile,
    Rayon,
    Explosion,
    Onde,
    Autre,
    /// Ne blesse pas (aura, soin, protection).
    Aucun,
}

impl Source {
    pub const BLESSANTES: [Source; 6] =
        [Source::Melee, Source::Projectile, Source::Rayon, Source::Explosion, Source::Onde, Source::Autre];

    pub fn nom(self) -> &'static str {
        match self {
            Source::Melee => "Corps à corps",
            Source::Projectile => "Projectile",
            Source::Rayon => "Rayon",
            Source::Explosion => "Explosion",
            Source::Onde => "Onde de choc",
            Source::Autre => "Autre (arc, zone)",
            Source::Aucun => "Sans dégâts",
        }
    }

    /// La clé de l'hypothèse qui porte son multiplicateur contre une créature.
    pub fn cle(self) -> &'static str {
        match self {
            Source::Melee => "combat.melee",
            Source::Projectile => "combat.projectile",
            Source::Rayon => "combat.rayon",
            Source::Explosion => "combat.explosion",
            Source::Onde => "combat.onde",
            Source::Autre | Source::Aucun => "combat.autre",
        }
    }
}

/// Ce qu'une capacité (`CharacterAbility`) vaut, avant l'arme et les multiplicateurs.
#[derive(Clone, Debug)]
pub struct Capacite {
    pub id: String,
    /// La sorte de capacité de Veloren (`ComboMelee2`, `BasicRanged`…).
    pub variante: String,
    pub source: Source,
    /// Nombre de touches d'une utilisation complète sur une cible.
    pub touches: f64,
    /// Dégâts de base d'une utilisation complète (somme des touches).
    pub degats: f64,
    /// Énergie d'une utilisation complète.
    pub energie: f64,
    /// Secondes du geste que la vitesse de l'arme raccourcit (préparation, frappe, récupération).
    pub duree: f64,
    /// Secondes que l'arme ne change pas (charge, élan, durée d'un rayon).
    pub duree_fixe: f64,
    pub portee: Option<f64>,
    /// Ce que le calcul ne compte pas.
    pub remarque: String,
}

fn nombre(valeur: Option<&Value>) -> f64 { valeur.and_then(Value::as_f64).unwrap_or(0.0) }

/// Toutes les valeurs numériques rangées sous `cle`, à n'importe quelle profondeur.
fn partout(valeur: &Value, cle: &str, trouve: &mut Vec<f64>) {
    match valeur {
        Value::Object(champs) => {
            for (nom, v) in champs {
                if nom == cle {
                    if let Some(n) = v.as_f64() {
                        trouve.push(n);
                    }
                }
                partout(v, cle, trouve);
            }
        },
        Value::Array(liste) => liste.iter().for_each(|v| partout(v, cle, trouve)),
        _ => {},
    }
}

fn somme(valeur: &Value, cle: &str) -> f64 {
    let mut trouve = Vec::new();
    partout(valeur, cle, &mut trouve);
    trouve.iter().sum()
}

fn contient(valeur: &Value, cle: &str) -> bool {
    match valeur {
        Value::Object(champs) => champs.iter().any(|(nom, v)| nom == cle || contient(v, cle)),
        Value::Array(liste) => liste.iter().any(|v| contient(v, cle)),
        _ => false,
    }
}

const GESTE: [&str; 4] = ["buildup_duration", "swing_duration", "recover_duration", "shoot_duration"];

/// Arrondi d'un nombre du jeu (f32) : 0,6 et pas 0,6000000238.
pub fn net(x: f32) -> f64 { (f64::from(x) * 1e4).round() / 1e4 }

impl Capacite {
    /// Lit la capacité `id` avec la structure du jeu, puis la parcourt sous forme générique : pas un cas par
    /// sorte de capacité, seulement pour celles qui frappent plusieurs fois.
    pub fn lire(id: &str) -> Result<Self> {
        let capacite = Ron::<CharacterAbility>::load(id)
            .map_err(|e| anyhow!("capacité {id} illisible : {e}"))?
            .read()
            .0
            .clone();
        let json = serde_json::to_value(&capacite).with_context(|| format!("capacité {id}"))?;
        let (variante, corps) = json
            .as_object()
            .and_then(|o| o.iter().next())
            .map(|(k, v)| (k.clone(), v.clone()))
            .ok_or_else(|| anyhow!("capacité {id} : forme inattendue"))?;
        let mut degats_lus = Vec::new();
        partout(&corps, "damage", &mut degats_lus);
        let total: f64 = degats_lus.iter().sum();
        let n_lus = degats_lus.len() as f64;
        let lu = |cle: &str| nombre(corps.get(cle));
        let geste: f64 = GESTE.iter().map(|cle| somme(&corps, cle)).sum();
        let cout = lu("energy_cost");
        let mut remarques: Vec<String> = Vec::new();
        // (touches, dégâts d'une utilisation, énergie, geste lié à l'arme, durée fixe)
        let (touches, degats, energie, duree, duree_fixe) = match variante.as_str() {
            "ComboMelee2" => (n_lus, total, lu("energy_cost_per_strike") * n_lus, geste, 0.0),
            "RapidMelee" => {
                let n = lu("max_strikes").max(1.0);
                let d = degats_lus.first().copied().unwrap_or(0.0);
                let duree = lu("buildup_duration") + lu("swing_duration") * n + lu("recover_duration");
                (n, d * n, cout * n, duree, 0.0)
            },
            "RapidRanged" => {
                let n = somme(&corps, "max_projectiles").max(1.0);
                let d = degats_lus.first().copied().unwrap_or(0.0);
                let duree = lu("buildup_duration") + lu("shoot_duration") * n + lu("recover_duration");
                (n, d * n, cout * n, duree, 0.0)
            },
            "BasicBeam" => {
                let secondes = lu("beam_duration");
                let n = (lu("tick_rate") * secondes).max(1.0);
                (
                    n,
                    lu("damage") * n,
                    lu("energy_drain") * secondes,
                    lu("buildup_duration") + lu("recover_duration"),
                    secondes,
                )
            },
            "ChargedMelee" | "ChargedRanged" => {
                remarques.push("À pleine charge.".into());
                (1.0, total, cout + lu("energy_drain") * lu("charge_duration"), geste, lu("charge_duration"))
            },
            "DiveMelee" => {
                let base = degats_lus.first().copied().unwrap_or(0.0);
                remarques.push(format!(
                    "Dégâts au plus bas ; la chute en ajoute (jusqu'à {} de plus par palier de vitesse).",
                    total - base
                ));
                (1.0, base, cout, geste, 0.0)
            },
            "DashMelee" => {
                remarques.push("Sans l'élan ; la course ajoute les dégâts de charge.".into());
                (1.0, degats_lus.first().copied().unwrap_or(0.0), cout, geste, lu("charge_duration"))
            },
            "LeapExplosionShockwave" => {
                if lu("shockwave_damage") > 0.0 {
                    remarques.push(format!("Onde de choc de {} en plus, non comptée.", lu("shockwave_damage")));
                }
                (1.0, lu("explosion_damage"), cout, geste, lu("movement_duration"))
            },
            _ if contient(&corps, "tick_dur") => {
                // Une zone qui brûle : une touche par battement, tant qu'on y reste.
                let n = (somme(&corps, "duration") / somme(&corps, "tick_dur").max(0.01)).max(1.0);
                remarques.push(format!("Zone persistante : compté {n} battements, si la cible y reste."));
                (n, total * n, cout, geste, 0.0)
            },
            _ => (n_lus, total, cout, geste, lu("movement_duration")),
        };
        let projectiles = corps
            .get("num_projectiles")
            .and_then(|n| n.get("Value"))
            .and_then(Value::as_f64)
            .unwrap_or(1.0);
        if projectiles > 1.0 {
            remarques.push(format!("{projectiles} projectiles à la fois ; un seul compté."));
        }
        let source = if degats <= 0.0 {
            Source::Aucun
        } else {
            match variante.as_str() {
                "BasicBeam" => Source::Rayon,
                "Shockwave" | "LeapShockwave" => Source::Onde,
                "Explosion" | "LeapExplosionShockwave" => Source::Explosion,
                _ if contient(&corps, "tick_dur") => Source::Autre,
                "BasicRanged" | "RapidRanged" | "ChargedRanged" | "LeapRanged" | "Throw" => {
                    if contient(&corps, "Explosive") { Source::Explosion } else { Source::Projectile }
                },
                "StaticAura" | "BasicAura" | "SpriteSummon" | "BasicSummon" => Source::Autre,
                _ => Source::Melee,
            }
        };
        if let Some(pluie) = corps.get("icebreaker_pluie").filter(|p| !p.is_null()) {
            remarques.push(format!(
                "Pluie : {} salves de {} traits sur un rayon de {} ; une seule touche comptée.",
                nombre(pluie.get("salves")),
                nombre(pluie.get("par_salve")),
                nombre(pluie.get("rayon")),
            ));
        }
        let mut portees = Vec::new();
        partout(&corps, "range", &mut portees);
        let portee = portees.iter().copied().reduce(f64::max).filter(|p| *p > 0.0);
        // Les nombres du jeu sont des f32 : on les arrondit pour l'affichage.
        let net = |x: f64| (x * 1e4).round() / 1e4 + 0.0;
        Ok(Self {
            id: id.to_string(),
            variante,
            source,
            touches: net(touches.max(0.0)),
            degats: net(degats.max(0.0)),
            energie: net(energie),
            duree: net(duree),
            duree_fixe: net(duree_fixe),
            portee,
            remarque: remarques.join(" "),
        })
    }

    /// Le nom court de l'id : la fin, sans `common.abilities.`.
    pub fn court(&self) -> &str {
        self.id
            .strip_prefix("common.abilities.")
            .or_else(|| self.id.strip_prefix("icebreaker.abilities."))
            .unwrap_or(&self.id)
    }
}

/// Une arme : ses multiplicateurs et ses attaques de base.
#[derive(Clone, Debug)]
pub struct Arme {
    pub id: String,
    pub puissance: f64,
    pub vitesse: f64,
    /// Les attaques de base de l'arme (clic gauche, clic droit).
    pub primaire: Option<String>,
    pub secondaire: Option<String>,
}

fn premier_id(sorte: &AbilityKind<String>) -> String {
    match sorte {
        AbilityKind::Simple(_, id) => id.clone(),
        // La capacité sans posture : la dernière de la liste.
        AbilityKind::Contextualized { abilities, .. } => {
            abilities.last().map(|(_, (_, id))| id.clone()).unwrap_or_default()
        },
    }
}

impl Arme {
    pub fn lire(id: &str) -> Result<Self> {
        let objet = Item::new_from_asset(id).map_err(|e| anyhow!("objet {id} illisible : {e:?}"))?;
        Self::de(&objet)
    }

    fn de(objet: &Item) -> Result<Self> {
        let id = objet.item_definition_id().itemdef_id().unwrap_or("?").to_string();
        let ItemKind::Tool(outil) = &*objet.kind() else {
            return Err(anyhow!("{id} n'est pas une arme"));
        };
        let stats = outil.stats(objet.stats_durability_multiplier());
        let spec = objet
            .ability_spec()
            .map(|s| s.into_owned())
            .unwrap_or(AbilitySpec::Tool(outil.kind));
        let table = Ron::<AbilityMap<String>>::load("common.abilities.ability_set_manifest")
            .map_err(|e| anyhow!("table des capacités illisible : {e}"))?;
        let table = table.read();
        let (primaire, secondaire) = match table.0.get_ability_set(&spec) {
            Some(jeu) => (Some(premier_id(&jeu.primary)), Some(premier_id(&jeu.secondary))),
            None => (None, None),
        };
        Ok(Self {
            id,
            puissance: net(stats.power),
            vitesse: net(stats.speed),
            primaire,
            secondaire,
        })
    }
}

/// La réduction de dégâts que donne l'armure d'un équipement.
fn reduction(inventaire: &Inventory) -> f64 {
    let msm = MaterialStatManifest::load().read();
    f64::from(Damage::compute_damage_reduction(None, Some(inventaire), None, &msm))
}

/// Un ennemi d'un rôle (`assets/icebreaker/ennemis/<role>.ron`).
#[derive(Clone, Debug)]
pub struct Ennemi {
    pub role: String,
    pub fichier: String,
    pub pv: f64,
    pub arme: Arme,
    pub armure: f64,
}

impl Ennemi {
    pub fn lire(role: &str) -> Result<Self> {
        let id = format!("icebreaker.ennemis.{role}");
        let mut alea = rand::rng();
        let info = EntityInfo::at(Vec3::zero()).with_asset_expect(&id, &mut alea, None);
        let pv = net(info.icebreaker_pv.unwrap_or_else(|| f32::from(info.body.base_health())));
        let inventaire = Inventory::with_loadout(info.loadout.build(), info.body);
        let arme = inventaire
            .equipped(EquipSlot::ActiveMainhand)
            .ok_or_else(|| anyhow!("{id} : pas d'arme en main"))
            .and_then(Arme::de)?;
        Ok(Self {
            role: role.to_string(),
            fichier: format!("ennemis/{role}.ron"),
            pv,
            arme,
            armure: (reduction(&inventaire) * 1e3).round() / 1e3,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ClasseLue {
    id: String,
    nom: String,
    equipement: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ClassesLues {
    classes: Vec<ClasseLue>,
}

/// Une classe de joueur.
#[derive(Clone, Debug)]
pub struct Classe {
    pub id: String,
    pub nom: String,
    /// PV d'un humanoïde, avant l'inné.
    pub pv_base: f64,
    /// Multiplicateur de PV de l'inné.
    pub inne_pv: f64,
    /// Réduction de dégâts de l'armure de départ.
    pub armure: f64,
    pub inne: Vec<BuffEffect>,
}

impl Classe {
    fn lire(lue: &ClasseLue) -> Result<Self> {
        let mut alea = rand::rng();
        let equipement = LoadoutBuilder::from_asset_expect(&lue.equipement, &mut alea, None).build();
        let inventaire = Inventory::with_loadout_humanoid(equipement);
        let inne = registre().passifs_de(&lue.id).map(|p| p.inne.clone()).unwrap_or_default();
        let inne_pv = inne
            .iter()
            .filter_map(|effet| multiplicateur_pv(effet))
            .product::<f64>();
        Ok(Self {
            id: lue.id.clone(),
            nom: lue.nom.clone(),
            pv_base: 100.0,
            inne_pv,
            armure: (reduction(&inventaire) * 1e3).round() / 1e3,
            inne,
        })
    }
}

/// Le multiplicateur de PV max d'un effet, s'il en est un.
pub fn multiplicateur_pv(effet: &BuffEffect) -> Option<f64> {
    let json = serde_json::to_value(effet).ok()?;
    let corps = json.get("MaxHealthModifier")?;
    (corps.get("kind")?.as_str()? == "Multiplicative").then(|| (nombre(corps.get("value")) * 1e4).round() / 1e4)
}

/// Un effet de passif, en français.
pub fn effet_en_clair(effet: &BuffEffect) -> String {
    let json = serde_json::to_value(effet).unwrap_or(Value::Null);
    let Some((nom, corps)) = json.as_object().and_then(|o| o.iter().next()) else {
        return format!("{effet:?}");
    };
    let valeur = corps.as_f64().unwrap_or_else(|| nombre(corps.get("value")));
    let pourcent = |x: f64| format!("{:+.0} %", (x - 1.0) * 100.0);
    match nom.as_str() {
        "MaxHealthModifier" => format!("PV max {}", pourcent(valeur)),
        "MaxEnergyModifier" => format!("Endurance max {}", pourcent(valeur)),
        "DamageReduction" => format!("Dégâts reçus {:+.0} %", -valeur * 100.0),
        "PoiseReduction" => format!("Déséquilibre reçu {:+.0} %", -valeur * 100.0),
        "AttackDamage" => format!("Dégâts {}", pourcent(valeur)),
        "EnergyEfficiency" => format!("Coût des compétences {:+.0} %", (1.0 / valeur - 1.0) * 100.0),
        "MovementSpeed" => format!("Vitesse {}", pourcent(valeur)),
        "AttackSpeed" => format!("Vitesse d'attaque {}", pourcent(valeur)),
        "PrecisionPowerMult" => format!("Dégâts de précision {}", pourcent(valeur)),
        autre => format!("{autre} {valeur}"),
    }
}

/// Une compétence d'un jeu d'armes ou un ultime, à un niveau.
#[derive(Clone, Debug)]
pub struct Ligne {
    pub classe: String,
    /// Id du jeu d'armes ; pour un ultime, celui par défaut de la classe.
    pub jeu: String,
    pub jeu_nom: String,
    pub nom: String,
    /// 0 : attaque de base ; 1 à 5 : niveau ; ultime : 1.
    pub niveau: u8,
    pub sorte: Sorte,
    pub capacite: Capacite,
    /// Multiplicateurs du niveau.
    pub puissance: f64,
    pub portee: f64,
    /// Secondes de recharge à ce niveau.
    pub recharge: f64,
    pub texte: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sorte {
    Base,
    Competence,
    Ultime,
}

impl Sorte {
    pub fn nom(self) -> &'static str {
        match self {
            Sorte::Base => "Attaque de base",
            Sorte::Competence => "Compétence",
            Sorte::Ultime => "Ultime",
        }
    }
}

/// Un jeu d'armes : son arme de main droite et ses lignes.
#[derive(Clone, Debug)]
pub struct Jeu {
    pub id: String,
    pub nom: String,
    pub classe: String,
    pub arme: Arme,
}

/// Tout ce que le classeur montre du combat.
pub struct Donnees {
    pub combat: IcebreakerCombat,
    pub salle: SalleArmes,
    pub classes: Vec<Classe>,
    pub jeux: Vec<Jeu>,
    pub lignes: Vec<Ligne>,
    pub ennemis: Vec<Ennemi>,
    /// Ce qui n'a pas pu être lu : à montrer, pas à taire.
    pub alertes: Vec<String>,
}

fn lignes_de(
    classe: &str,
    jeu: &str,
    jeu_nom: &str,
    sorte: Sorte,
    competence: &Competence,
    alertes: &mut Vec<String>,
) -> Vec<Ligne> {
    (1..=competence.niveau_max())
        .filter_map(|niveau| {
            let effectif = competence.effectif(niveau);
            let capacite = match Capacite::lire(effectif.capacite) {
                Ok(c) => c,
                Err(e) => {
                    alertes.push(format!("{} niveau {niveau} : {e}", competence.nom));
                    return None;
                },
            };
            let texte = match niveau {
                1 => competence.description.clone(),
                n => competence.niveaux[usize::from(n) - 2].texte.clone(),
            };
            Some(Ligne {
                classe: classe.to_string(),
                jeu: jeu.to_string(),
                jeu_nom: jeu_nom.to_string(),
                nom: competence.nom.clone(),
                niveau,
                sorte,
                capacite,
                puissance: net(effectif.stats.power),
                portee: net(effectif.stats.range),
                recharge: net(effectif.recharge),
                texte,
            })
        })
        .collect()
}

pub const ROLES: [&str; 4] = ["guerrier", "archer", "mage", "chef"];

impl Donnees {
    pub fn lire() -> Result<Self> {
        let combat = Ron::<IcebreakerCombat>::load("icebreaker.combat")
            .map_err(|e| anyhow!("combat.ron illisible : {e} (lancer depuis la racine du dépôt)"))?
            .read()
            .0
            .clone();
        let lues = Ron::<ClassesLues>::load("icebreaker.classes")
            .map_err(|e| anyhow!("classes.ron illisible : {e}"))?
            .read()
            .0
            .clone();
        let mut alertes = Vec::new();
        let mut classes = Vec::new();
        let mut jeux = Vec::new();
        let mut lignes = Vec::new();
        for lue in &lues.classes {
            classes.push(Classe::lire(lue)?);
            for (id, jeu) in registre().jeux_de(&lue.id) {
                let arme = Arme::lire(&jeu.armes.0)?;
                for (nom, capacite) in
                    [("Attaque principale", &arme.primaire), ("Attaque secondaire", &arme.secondaire)]
                {
                    let Some(capacite) = capacite else { continue };
                    match Capacite::lire(capacite) {
                        Ok(capacite) => lignes.push(Ligne {
                            classe: lue.id.clone(),
                            jeu: id.to_string(),
                            jeu_nom: jeu.nom.clone(),
                            nom: nom.to_string(),
                            niveau: 0,
                            sorte: Sorte::Base,
                            capacite,
                            puissance: 1.0,
                            portee: 1.0,
                            recharge: 0.0,
                            texte: String::new(),
                        }),
                        Err(e) => alertes.push(format!("{} : {e}", jeu.nom)),
                    }
                }
                for (_, competence) in &jeu.competences {
                    lignes.extend(lignes_de(&lue.id, id, &jeu.nom, Sorte::Competence, competence, &mut alertes));
                }
                jeux.push(Jeu {
                    id: id.to_string(),
                    nom: jeu.nom.clone(),
                    classe: lue.id.clone(),
                    arme,
                });
            }
            // Un ultime se lance avec l'arme en main : on prend celle du jeu d'armes par défaut de la classe.
            let defaut = registre().jeu_par_defaut(&lue.id).map_or("", |(id, _)| id);
            for (_, ultime) in registre().ultimes_de(&lue.id) {
                lignes.extend(lignes_de(&lue.id, defaut, "Ultime", Sorte::Ultime, ultime, &mut alertes));
            }
        }
        let ennemis = ROLES.iter().map(|role| Ennemi::lire(role)).collect::<Result<Vec<_>>>()?;
        Ok(Self {
            combat,
            salle: SalleArmes::actuel(),
            classes,
            jeux,
            lignes,
            ennemis,
            alertes,
        })
    }
}
