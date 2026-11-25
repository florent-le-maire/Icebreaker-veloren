//! Icebreaker : lecture des chiffres du jeu, avec les structures du jeu (`common`, `world`).
//! Tout part des fichiers de `assets/` : rien n'est recopié ici.

use crate::capacite::{Capacite, net, nombre};
use anyhow::{Result, anyhow};
use common::{
    assets::{AssetExt, Ron},
    combat::{Damage, combat_rating},
    comp::{
        Energy, Health, Inventory, Poise, SkillSet,
        buff::BuffEffect,
        inventory::{loadout_builder::LoadoutBuilder, slot::EquipSlot},
        item::{
            Item, ItemKind, MaterialStatManifest,
            tool::{AbilityKind, AbilityMap, AbilitySpec, Stats},
        },
    },
    generation::EntityInfo,
    icebreaker_combat::IcebreakerCombat,
    icebreaker_competences::{Competence, SalleArmes, registre},
    skillset_builder::SkillSetBuilder,
};
use serde::Deserialize;
use serde_json::Value;
use vek::Vec3;

/// Une arme : ses multiplicateurs et ses attaques de base.
#[derive(Clone, Debug)]
pub struct Arme {
    pub id: String,
    pub puissance: f64,
    /// Ses stats, telles que le jeu les applique à une capacité.
    pub stats: Stats,
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
            stats,
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
    /// Expérience que donne sa mort (la note de combat de Veloren × 20), partagée entre ceux qui l'ont blessé.
    pub experience: f64,
}

impl Ennemi {
    pub fn lire(role: &str) -> Result<Self> { Self::fiche(&format!("icebreaker.ennemis.{role}"), role) }

    /// Une fiche d'entité quelconque (`assets/.../<fiche>.ron`), sous le nom `role`.
    pub fn fiche(id: &str, role: &str) -> Result<Self> {
        let mut alea = rand::rng();
        let info = EntityInfo::at(Vec3::zero()).with_asset_expect(id, &mut alea, None);
        let pv = net(info.icebreaker_pv.unwrap_or_else(|| f32::from(info.body.base_health())));
        let inventaire = Inventory::with_loadout(info.loadout.build(), info.body);
        let mut sante = Health::new(info.body);
        sante.icebreaker_ennemi(info.icebreaker_pv);
        let talents = info
            .skillset_asset
            .as_deref()
            .map_or_else(SkillSet::default, |id| SkillSetBuilder::from_asset_expect(id).build());
        let note = {
            let msm = MaterialStatManifest::load().read();
            let (energie, equilibre) = (Energy::new(info.body), Poise::new(info.body));
            combat_rating(&inventaire, &sante, &energie, &equilibre, &talents, info.body, &msm)
        };
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
            experience: f64::from(note * 20.0).round(),
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
    /// Multiplicateur de dégâts du niveau.
    pub puissance: f64,
    /// Dégâts de base de la capacité telle que le jeu la lance, puissances comprises : le témoin de la formule.
    pub degats_jeu: f64,
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
    pub donjon: crate::donjon::Donjon,
    pub tables: Vec<crate::butin::Table>,
    pub village: crate::butin::Village,
    pub prix_soin: Option<f64>,
    /// Ce qui n'a pas pu être lu : à montrer, pas à taire.
    pub alertes: Vec<String>,
}

fn lignes_de(
    classe: &str,
    jeu: &str,
    jeu_nom: &str,
    sorte: Sorte,
    competence: &Competence,
    arme: &Arme,
    alertes: &mut Vec<String>,
) -> Vec<Ligne> {
    (1..=competence.niveau_max())
        .filter_map(|niveau| {
            let effectif = competence.effectif(niveau);
            // Comme le jeu : les stats de l'arme en main, fois celles du niveau.
            let stats = arme.stats * effectif.stats;
            let lue = Capacite::lire(effectif.capacite, stats)
                .and_then(|c| Ok((c, Capacite::lancee(effectif.capacite, stats)?.degats())));
            let (capacite, degats_jeu) = match lue {
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
                degats_jeu,
                recharge: net(effectif.recharge),
                texte,
            })
        })
        .collect()
}

pub const ROLES: [&str; 4] = ["guerrier", "archer", "mage", "chef"];
/// Donjons générés par cas de l'onglet Expéditions.
const GRAINES: u64 = 20;

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
                    let attaque = Capacite::lire(capacite, arme.stats)
                        .and_then(|c| Ok((c, Capacite::lancee(capacite, arme.stats)?.degats())));
                    match attaque {
                        Ok((capacite, degats_jeu)) => lignes.push(Ligne {
                            classe: lue.id.clone(),
                            jeu: id.to_string(),
                            jeu_nom: jeu.nom.clone(),
                            nom: nom.to_string(),
                            niveau: 0,
                            sorte: Sorte::Base,
                            capacite,
                            puissance: 1.0,
                            degats_jeu,
                            recharge: 0.0,
                            texte: String::new(),
                        }),
                        Err(e) => alertes.push(format!("{} : {e}", jeu.nom)),
                    }
                }
                for (_, competence) in &jeu.competences {
                    lignes.extend(lignes_de(&lue.id, id, &jeu.nom, Sorte::Competence, competence, &arme, &mut alertes));
                }
                jeux.push(Jeu {
                    id: id.to_string(),
                    nom: jeu.nom.clone(),
                    arme,
                });
            }
            // Un ultime se lance avec l'arme en main : on prend celle du jeu d'armes par défaut de la classe.
            let defaut = registre().jeu_par_defaut(&lue.id).map_or("", |(id, _)| id);
            if let Some(arme) = jeux.iter().find(|j: &&Jeu| j.id == defaut).map(|j| j.arme.clone()) {
                for (_, ultime) in registre().ultimes_de(&lue.id) {
                    lignes.extend(lignes_de(&lue.id, defaut, "Ultime", Sorte::Ultime, ultime, &arme, &mut alertes));
                }
            }
        }
        // Le témoin : la formule du classeur (base × niveau × arme) doit redonner les dégâts que le jeu lance.
        for l in &lignes {
            let arme = jeux.iter().find(|j| j.id == l.jeu).map_or(1.0, |j| j.arme.puissance);
            let formule = l.capacite.degats() * l.puissance * arme;
            if (formule - l.degats_jeu).abs() > 0.01 {
                alertes.push(format!(
                    "{} niveau {} : la formule donne {formule} de base, le jeu {}",
                    l.nom, l.niveau, l.degats_jeu
                ));
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
            donjon: crate::donjon::lire(GRAINES)?,
            tables: crate::butin::tables()?,
            village: Ron::<crate::butin::Village>::load("icebreaker.village")
                .map_err(|e| anyhow!("village.ron illisible : {e}"))?
                .read()
                .0
                .clone(),
            prix_soin: crate::butin::prix_du_soin(),
            alertes,
        })
    }
}
