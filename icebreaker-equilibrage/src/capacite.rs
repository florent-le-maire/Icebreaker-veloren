//! Icebreaker : ce que vaut une capacité (`CharacterAbility`) telle que le jeu la lance. On applique d'abord
//! les stats de l'arme et du niveau comme le jeu (`adjusted_by_stats`), puis on lit le résultat sorte par sorte,
//! d'après ce que fait son état de combat (`common/src/states/`) : combien de touches, en combien de temps.

use anyhow::{Context, Result, anyhow};
use common::{
    assets::{AssetExt, Ron},
    comp::{ability::CharacterAbility, item::tool::Stats},
};
use serde_json::Value;

/// Le type d'une attaque, pour le multiplicateur de `combat.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Source {
    Melee,
    Projectile,
    Rayon,
    Explosion,
    Onde,
    Autre,
}

impl Source {
    pub fn nom(self) -> &'static str {
        match self {
            Source::Melee => "Corps à corps",
            Source::Projectile => "Projectile",
            Source::Rayon => "Rayon",
            Source::Explosion => "Explosion",
            Source::Onde => "Onde de choc",
            Source::Autre => "Zone persistante",
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
            Source::Autre => "combat.autre",
        }
    }
}

/// Une part des dégâts d'une capacité : un type d'attaque, ses touches, leurs dégâts de base en tout.
#[derive(Clone, Debug, PartialEq)]
pub struct Composante {
    pub source: Source,
    pub touches: f64,
    /// Dégâts de base de toutes ces touches, avant la puissance de l'arme et du niveau.
    pub degats: f64,
}

/// Ce qu'une utilisation d'une capacité vaut.
#[derive(Clone, Debug)]
pub struct Capacite {
    pub id: String,
    /// La sorte de capacité de Veloren (`ComboMelee2`, `BasicRanged`…).
    pub variante: String,
    pub composantes: Vec<Composante>,
    /// Énergie d'une utilisation.
    pub energie: f64,
    /// Secondes d'une utilisation, avec la vitesse de l'arme.
    pub geste: f64,
    /// Secondes avant la première touche, et après la dernière.
    pub avant: f64,
    pub apres: f64,
    /// Une capacité qu'on tient (rayon, tir continu) : l'utilisation est une seconde ou un tir, `avant` s'y ajoute.
    pub tenu: bool,
    /// Ce que le calcul suppose ou ne compte pas.
    pub remarque: String,
}

/// Le serveur avance par ticks (`TPS` de `server-cli/src/main.rs`). Un état de combat ne change d'étape
/// (préparation, frappe, récupération…) qu'à un tick, une fois sa durée dépassée, et ce tick-là ne fait pas
/// avancer l'étape suivante.
pub const TICKS_PAR_SECONDE: f64 = 30.0;

/// Secondes perdues par un geste de `etapes` étapes qu'on répète : par étape, un tick de passage et un demi-tick
/// de dépassement en moyenne ; un tick pour relancer le geste. Vérifié sur l'enchaînement et l'estoc de la grande
/// épée, déroulés tick par tick (`common/src/states/combo_melee2.rs`, `charged_melee.rs`) : 1,00 s et 1,17 s,
/// contre 0,75 s et 1,00 s d'après les seules durées des fichiers.
pub fn ticks_perdus(etapes: f64) -> f64 { (etapes * 1.5 + 1.0) / TICKS_PAR_SECONDE }

/// Rayon d'une cible vue du dessus, pour estimer combien de traits d'une pluie la touchent.
const RAYON_CIBLE: f64 = 0.5;

pub fn nombre(valeur: Option<&Value>) -> f64 { valeur.and_then(Value::as_f64).unwrap_or(0.0) }

/// Arrondi d'un nombre du jeu (f32) : 0,6 et pas 0,6000000238.
pub fn net(x: f32) -> f64 { net64(f64::from(x)) }

fn net64(x: f64) -> f64 { (x * 1e4).round() / 1e4 + 0.0 }

/// La première valeur rangée sous `cle`, à n'importe quelle profondeur.
fn trouve<'a>(valeur: &'a Value, cle: &str) -> Option<&'a Value> {
    match valeur {
        Value::Object(champs) => champs
            .get(cle)
            .filter(|v| !v.is_null())
            .or_else(|| champs.values().find_map(|v| trouve(v, cle))),
        Value::Array(liste) => liste.iter().find_map(|v| trouve(v, cle)),
        _ => None,
    }
}

/// Dégâts d'un coup de mêlée : (de départ, ajoutés à pleine charge ou pleine vitesse).
fn melee(constructeur: &Value) -> (f64, f64) {
    let lu = |sorte: Option<&Value>| {
        sorte.and_then(Value::as_object).and_then(|o| o.values().next()).map_or(0.0, |v| nombre(v.get("damage")))
    };
    (lu(constructeur.get("kind")), lu(constructeur.get("scaled").and_then(|s| s.get("kind"))))
}

/// Un projectile : (dégâts de départ, ajoutés à pleine charge, type d'attaque, battements d'une zone, remarque).
fn projectile(p: &Value) -> (f64, f64, Source, f64, Option<String>) {
    let depart = nombre(p.get("attack").and_then(|a| a.get("damage")));
    let charge = nombre(p.get("scaled").and_then(|s| s.get("damage")));
    let sorte = p.get("kind");
    let explosif = sorte.and_then(|k| k.get("Explosive"));
    let zone = sorte.and_then(|k| k.get("Pool"));
    if let Some(e) = explosif {
        // Selon sa cible, le projectile explose sur l'ennemi (une explosion) ou le frappe et n'explose qu'au sol.
        return match e.get("target").and_then(Value::as_str) {
            Some("SolidOnlyEntityAttack") => (depart, charge, Source::Projectile, 1.0, None),
            Some("SolidOnly") => (
                depart,
                charge,
                Source::Explosion,
                1.0,
                Some("N'explose qu'au sol ou contre un mur : compté au centre de l'explosion.".into()),
            ),
            _ => (depart, charge, Source::Explosion, 1.0, None),
        };
    }
    if let Some(z) = zone {
        let battements = (nombre(z.get("duration")) / nombre(z.get("tick_dur")).max(0.01)).max(1.0);
        return (
            depart,
            charge,
            Source::Autre,
            battements,
            Some(format!("Zone au sol : {battements} battements comptés, si la cible y reste jusqu'au bout.")),
        );
    }
    (depart, charge, Source::Projectile, 1.0, None)
}

/// Un effet de brûlure ou de saignement porté par l'attaque : il n'est pas compté, mais il pèse.
fn alteration(corps: &Value) -> Option<String> {
    fn cherche(v: &Value) -> Option<&Value> {
        match v {
            Value::Object(o) if o.get("strength").is_some_and(|s| s.get("DamageFraction").is_some()) => Some(v),
            Value::Object(o) => o.values().find_map(cherche),
            Value::Array(l) => l.iter().find_map(cherche),
            _ => None,
        }
    }
    let effet = cherche(corps)?;
    let part = nombre(effet.get("strength").and_then(|s| s.get("DamageFraction")));
    Some(format!(
        "Non compté : {} ({:.0} % des dégâts en plus sur {} s, {:.0} % de chances).",
        effet.get("kind").and_then(Value::as_str).unwrap_or("altération"),
        part * 100.0,
        nombre(effet.get("dur_secs")),
        nombre(effet.get("chance")) * 100.0,
    ))
}

impl Capacite {
    /// La capacité `id` lancée avec `stats` (celles de l'arme, fois celles du niveau), dégâts de base : la
    /// puissance est mise à 1, c'est le classeur qui la remet (elle multiplie tous les dégâts).
    pub fn lire(id: &str, stats: Stats) -> Result<Self> { Self::lancee(id, Stats { power: 1.0, ..stats }) }

    /// La capacité `id` telle que le jeu la lance avec `stats`, puissance comprise.
    pub fn lancee(id: &str, stats: Stats) -> Result<Self> {
        let capacite = Ron::<CharacterAbility>::load(id)
            .map_err(|e| anyhow!("capacité {id} illisible : {e}"))?
            .read()
            .0
            .clone()
            .adjusted_by_stats(stats);
        let json = serde_json::to_value(&capacite).with_context(|| format!("capacité {id}"))?;
        let (variante, corps) = json
            .as_object()
            .and_then(|o| o.iter().next())
            .map(|(k, v)| (k.clone(), v.clone()))
            .ok_or_else(|| anyhow!("capacité {id} : forme inattendue"))?;
        let lu = |cle: &str| nombre(corps.get(cle));
        let (preparation, frappe, recuperation) =
            (lu("buildup_duration"), lu("swing_duration"), lu("recover_duration"));
        let cout = lu("energy_cost");
        let mut remarques: Vec<String> = Vec::new();
        let mut composantes: Vec<Composante> = Vec::new();
        // Par défaut la première touche vient après la préparation, la dernière avant la récupération.
        let (mut avant, mut apres, mut tenu) = (preparation, recuperation, false);
        let mut ajoute = |source: Source, touches: f64, degats: f64| {
            if degats > 0.0 && touches > 0.0 {
                composantes.push(Composante { source, touches: net64(touches), degats: net64(degats) });
            }
        };
        // Les étapes du geste (préparation, frappe, récupération le plus souvent), pour les ticks perdus.
        let mut etapes = 3.0;
        // (énergie, secondes d'une utilisation)
        let (energie, geste) = match variante.as_str() {
            "BasicMelee" => {
                ajoute(Source::Melee, 1.0, melee(&corps["melee_constructor"]).0);
                (cout, preparation + frappe + recuperation)
            },
            "ComboMelee2" => {
                // Les coups s'enchaînent sans récupération ; seule celle du dernier compte.
                let coups = corps["strikes"].as_array().cloned().unwrap_or_default();
                let n = coups.len() as f64;
                etapes = 2.0 * n + 1.0;
                let degats: f64 = coups.iter().map(|c| melee(&c["melee_constructor"]).0).sum();
                let duree: f64 = coups
                    .iter()
                    .map(|c| nombre(c.get("buildup_duration")) + nombre(c.get("swing_duration")))
                    .sum::<f64>()
                    + coups.last().map_or(0.0, |c| nombre(c.get("recover_duration")));
                ajoute(Source::Melee, n, degats);
                avant = coups.first().map_or(0.0, |c| nombre(c.get("buildup_duration")));
                apres = coups.last().map_or(0.0, |c| nombre(c.get("recover_duration")));
                if n > 1.0 {
                    remarques.push("Enchaînement entier, touche tenue.".into());
                }
                (lu("energy_cost_per_strike") * n, duree)
            },
            "RapidMelee" => {
                let d = melee(&corps["melee_constructor"]).0;
                match corps.get("max_strikes").and_then(Value::as_f64) {
                    Some(n) => {
                        etapes = n + 2.0;
                        ajoute(Source::Melee, n, d * n);
                        (cout * n, preparation + frappe * n + recuperation)
                    },
                    None => {
                        ajoute(Source::Melee, 1.0, d);
                        (tenu, apres) = (true, 0.0);
                        remarques.push("Tenu : compté par frappe.".into());
                        (cout, frappe)
                    },
                }
            },
            "RiposteMelee" => {
                ajoute(Source::Melee, 1.0, melee(&corps["melee_constructor"]).0);
                remarques.push("Ne frappe que si la parade prend un coup.".into());
                (cout, preparation + frappe + recuperation)
            },
            "DashMelee" => {
                let (depart, charge) = melee(&corps["melee_constructor"]);
                ajoute(Source::Melee, 1.0, depart + charge);
                remarques.push(format!("À plein élan ; sans élan : {depart} de base."));
                etapes = 4.0;
                avant = preparation + lu("charge_duration");
                (
                    cout + lu("energy_drain") * lu("charge_duration"),
                    preparation + lu("charge_duration") + frappe + recuperation,
                )
            },
            "DiveMelee" => {
                let (depart, vitesse) = melee(&corps["melee_constructor"]);
                ajoute(Source::Melee, 1.0, depart);
                remarques.push(format!(
                    "Frappé depuis le sol ; en tombant vite, jusqu'à {} de base.",
                    net64(depart + vitesse * lu("max_scaling"))
                ));
                (cout, preparation + frappe + recuperation)
            },
            "LeapMelee" => {
                ajoute(Source::Melee, 1.0, melee(&corps["melee_constructor"]).0);
                avant = preparation + lu("movement_duration");
                etapes = 4.0;
                (cout, preparation + lu("movement_duration") + frappe + recuperation)
            },
            "ChargedMelee" => {
                let (depart, charge) = melee(&corps["melee_constructor"]);
                ajoute(Source::Melee, 1.0, depart + charge);
                avant = preparation + lu("charge_duration");
                remarques.push(format!("À pleine charge ; sans charger : {depart} de base."));
                (
                    cout + lu("energy_drain") * lu("charge_duration"),
                    preparation + lu("charge_duration") + frappe + recuperation,
                )
            },
            "ChargedRanged" => {
                let (depart, charge, source, _, note) = projectile(&corps["projectile"]);
                ajoute(source, 1.0, depart + charge);
                avant = preparation + lu("charge_duration");
                remarques.push(format!("À pleine charge ; sans charger : {depart} de base."));
                remarques.extend(note);
                (
                    cout + lu("energy_drain") * lu("charge_duration"),
                    preparation + lu("charge_duration") + recuperation,
                )
            },
            "BasicRanged" | "Throw" => {
                let (depart, _, source, battements, note) = projectile(&corps["projectile"]);
                remarques.extend(note);
                let mut touches = battements;
                let ensemble = nombre(corps.get("num_projectiles").and_then(|n| n.get("Value"))).max(1.0);
                if let Some(pluie) = corps.get("icebreaker_pluie").filter(|p| !p.is_null()) {
                    // Les traits tombent au hasard dans un cercle : la part qui touche une cible immobile.
                    let traits = nombre(pluie.get("salves")) * nombre(pluie.get("par_salve"));
                    let rayon = nombre(pluie.get("rayon")).max(RAYON_CIBLE);
                    touches = (traits * (RAYON_CIBLE / rayon).powi(2)).max(1.0);
                    remarques.push(format!(
                        "Pluie de {traits} traits sur un rayon de {rayon} : estimation pour une cible immobile de \
                         {RAYON_CIBLE} de rayon."
                    ));
                } else if ensemble > 1.0 {
                    remarques.push(format!("{ensemble} projectiles en éventail ; un seul compté sur la cible."));
                }
                ajoute(source, touches, depart * touches);
                etapes = 2.0;
                (cout, preparation + recuperation)
            },
            "RapidRanged" => {
                let (depart, _, source, _, note) = projectile(&corps["projectile"]);
                remarques.extend(note);
                let tir = lu("shoot_duration");
                match trouve(&corps, "max_projectiles").and_then(Value::as_f64) {
                    Some(n) => {
                        etapes = n + 2.0;
                        ajoute(source, n, depart * n);
                        (cout * n, preparation + tir * n + recuperation)
                    },
                    None => {
                        ajoute(source, 1.0, depart);
                        (tenu, apres) = (true, 0.0);
                        remarques.push("Tenu : compté par tir.".into());
                        (cout, tir)
                    },
                }
            },
            "LeapRanged" => {
                if let Some(coup) = corps.get("melee").filter(|m| !m.is_null()) {
                    ajoute(Source::Melee, 1.0, melee(coup).0);
                }
                let (depart, _, source, _, note) = projectile(&corps["projectile"]);
                remarques.extend(note);
                ajoute(source, 1.0, depart);
                (cout, preparation + lu("movement_duration") + recuperation)
            },
            "BasicBeam" => {
                // Le rayon dure tant que la touche est tenue (et que l'énergie suit), ou `icebreaker_duree`.
                let battements = lu("tick_rate");
                match corps.get("icebreaker_duree").and_then(Value::as_f64) {
                    Some(duree) => {
                        ajoute(Source::Rayon, battements * duree, lu("damage") * battements * duree);
                        (lu("energy_drain") * duree, preparation + duree + recuperation)
                    },
                    None => {
                        ajoute(Source::Rayon, battements, lu("damage") * battements);
                        (tenu, apres) = (true, 0.0);
                        remarques.push("Tenu : compté par seconde de rayon.".into());
                        (lu("energy_drain"), 1.0)
                    },
                }
            },
            "Shockwave" | "LeapShockwave" => {
                // Icebreaker : plusieurs frappes d'affilée, une onde à chacune (Séisme).
                let n = corps.get("icebreaker_frappes").and_then(Value::as_f64).unwrap_or(1.0).max(1.0);
                ajoute(Source::Onde, n, lu("damage") * n);
                etapes = 2.0 * n + 1.0;
                avant = preparation + lu("movement_duration") + frappe;
                (cout, lu("movement_duration") + (preparation + frappe) * n + recuperation)
            },
            "LeapExplosionShockwave" => {
                ajoute(Source::Explosion, 1.0, lu("explosion_damage"));
                ajoute(Source::Onde, 1.0, lu("shockwave_damage"));
                avant = preparation + lu("movement_duration");
                etapes = 4.0;
                (cout, preparation + lu("movement_duration") + frappe + recuperation)
            },
            "Explosion" => {
                ajoute(Source::Explosion, 1.0, lu("damage"));
                (cout, preparation + lu("action_duration") + recuperation)
            },
            // Aura, protection, soin, invocation : rien qui blesse directement.
            _ => (cout, preparation + lu("cast_duration") + frappe + recuperation),
        };
        if !composantes.is_empty() {
            remarques.extend(alteration(&corps));
        }
        // EQ.9 : un geste qu'on répète perd des ticks à chaque étape ; un rayon ou un tir qu'on tient n'en change pas.
        let geste = if tenu { geste } else { geste + ticks_perdus(etapes) };
        Ok(Self {
            id: id.to_string(),
            variante,
            composantes,
            energie: net64(energie),
            geste: net64(geste),
            avant: net64(avant),
            apres: net64(apres),
            tenu,
            remarque: remarques.join(" "),
        })
    }

    pub fn blesse(&self) -> bool { !self.composantes.is_empty() }

    /// Touches d'une utilisation, tous types confondus.
    pub fn touches(&self) -> f64 { self.composantes.iter().map(|c| c.touches).sum() }

    /// Dégâts de base d'une utilisation.
    pub fn degats(&self) -> f64 { self.composantes.iter().map(|c| c.degats).sum() }

    /// Dégâts d'une utilisation, chaque type d'attaque pesé par `multiplicateur`.
    pub fn pondere(&self, multiplicateur: impl Fn(Source) -> f64) -> f64 {
        self.composantes.iter().map(|c| c.degats * multiplicateur(c.source)).sum()
    }

    /// Ses types d'attaque, en clair.
    pub fn types(&self) -> String {
        if self.composantes.is_empty() {
            return "Sans dégâts".into();
        }
        self.composantes.iter().map(|c| c.source.nom()).collect::<Vec<_>>().join(" + ")
    }

    /// Le nom court de l'id : la fin, sans `common.abilities.`.
    pub fn court(&self) -> &str {
        self.id
            .strip_prefix("common.abilities.")
            .or_else(|| self.id.strip_prefix("icebreaker.abilities."))
            .unwrap_or(&self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(vitesse: f32) -> Stats { Stats { speed: vitesse, ..Stats::one() } }

    /// La Fournaise (`assets/icebreaker/abilities/mage/ultimes/fournaise.ron`) : un rayon de 2,5 s, quelle que
    /// soit la touche, à 3 touches par seconde. Ce n'est pas `beam_duration` qui donne sa durée.
    #[test]
    fn fournaise_dure_son_temps() {
        let c = Capacite::lire("icebreaker.abilities.mage.ultimes.fournaise", stats(1.0)).unwrap();
        assert_eq!(c.composantes, vec![Composante { source: Source::Rayon, touches: 7.5, degats: 150.0 }]);
        assert_eq!(c.geste, net64(1.5 + 2.5 + 0.4 + ticks_perdus(3.0)));
    }

    /// Le Séisme (`icebreaker_frappes: Some(3)`) : trois frappes d'affilée, une onde à chacune.
    #[test]
    fn seisme_frappe_trois_fois() {
        let c = Capacite::lire("icebreaker.abilities.guerrier.ultimes.seisme", stats(1.0)).unwrap();
        assert_eq!(c.composantes, vec![Composante { source: Source::Onde, touches: 3.0, degats: 90.0 }]);
        assert_eq!(c.geste, net64((0.3 + 0.25) * 3.0 + 0.3 + ticks_perdus(7.0)));
    }

    /// Le lance-flammes du bâton se tient : on le compte par seconde, 3 touches de 5.
    #[test]
    fn lance_flammes_par_seconde() {
        let c = Capacite::lire("common.abilities.staff.flamethrower", stats(1.0)).unwrap();
        assert_eq!((c.touches(), c.degats(), c.geste, c.energie), (3.0, 15.0, 1.0, 20.0));
    }

    /// Une arme lente allonge le geste et espace les touches d'un rayon, comme dans le jeu.
    #[test]
    fn la_vitesse_de_l_arme_compte() {
        let c = Capacite::lire("common.abilities.staff.flamethrower", stats(0.8)).unwrap();
        assert_eq!(c.touches(), 2.4);
        let coup = Capacite::lire("common.abilities.gnarling.chieftain.flamestrike", stats(0.5)).unwrap();
        assert_eq!(coup.geste, net64(4.3 + ticks_perdus(3.0)));
    }

    /// Un enchaînement : la récupération ne compte qu'après le dernier coup.
    #[test]
    fn enchainement() {
        let c = Capacite::lire("icebreaker.abilities.guerrier.grande_epee.balayage_lourd_3", stats(1.0)).unwrap();
        assert_eq!((c.touches(), c.degats(), c.energie), (2.0, 25.0, 24.0));
        assert_eq!(c.geste, net64(0.3 + 0.2 + 0.15 + 0.2 + 0.2 + ticks_perdus(5.0)));
    }

    /// EQ.9 (mesure de Flo au mannequin, 2026-11-25) : à la grande épée (vitesse 0,8), l'enchaînement de base
    /// déroulé tick par tick dure 30 ticks et l'estoc chargé 35 ; les durées des fichiers seules donnaient 0,75 s
    /// et 1,00 s, ce qui mettait l'attaque normale devant l'attaque chargée.
    #[test]
    fn les_ticks_du_serveur_comptent() {
        let combo = Capacite::lire("common.abilities.sword.basic_double_slash", stats(0.8)).unwrap();
        let estoc = Capacite::lire("common.abilities.sword.basic_thrust", stats(0.8)).unwrap();
        assert!((combo.geste - 30.0 / TICKS_PAR_SECONDE).abs() < 0.05, "{}", combo.geste);
        assert!((estoc.geste - 35.0 / TICKS_PAR_SECONDE).abs() < 0.05, "{}", estoc.geste);
        // Un rayon tenu ne change pas d'étape : une seconde reste une seconde.
        assert_eq!(Capacite::lire("common.abilities.staff.flamethrower", stats(1.0)).unwrap().geste, 1.0);
    }

    /// Le Trait lourd frappe la cible comme un projectile : il n'explose qu'au sol.
    #[test]
    fn projectile_qui_n_explose_qu_au_sol() {
        let c = Capacite::lire("common.abilities.bow.heavy_nock", stats(1.0)).unwrap();
        assert_eq!(c.composantes[0].source, Source::Projectile);
        let bombe = Capacite::lire("common.abilities.staff.firebomb", stats(1.0)).unwrap();
        assert_eq!(bombe.composantes[0].source, Source::Explosion);
    }

    /// La puissance de l'arme multiplie tous les dégâts : c'est ce que suppose la formule du classeur.
    #[test]
    fn la_puissance_multiplie_les_degats() {
        for id in [
            "icebreaker.abilities.mage.ultimes.fournaise",
            "icebreaker.abilities.guerrier.ultimes.envol",
            "common.abilities.bow.broadhead",
            "common.abilities.sword.cleaving_whirlwind_slice",
            "icebreaker.abilities.archer.arc.bond_arriere_3",
        ] {
            let stats = Stats { power: 0.6, speed: 0.8, ..Stats::one() };
            let (base, lancee) = (Capacite::lire(id, stats).unwrap(), Capacite::lancee(id, stats).unwrap());
            assert!((base.degats() * 0.6 - lancee.degats()).abs() < 1e-3, "{id}");
            assert_eq!(base.geste, lancee.geste, "{id}");
        }
    }
}
