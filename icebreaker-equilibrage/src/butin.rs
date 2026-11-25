//! Icebreaker : ce que les tables de butin donnent en moyenne (`assets/icebreaker/butin.ron` et les tables de
//! Veloren qu'il nomme), et les prix qui ne sont pas dans les fichiers du combat.

use anyhow::{Result, anyhow};
use common::{
    assets::AssetExt,
    lottery::{LootSpec, Lottery},
};
use serde::Deserialize;
use std::collections::BTreeMap;
use world::icebreaker::butin::Butin;

/// Ce qu'un tirage d'une table de butin donne en moyenne.
#[derive(Clone, Debug)]
pub struct Table {
    /// Ce qui la lâche : « Coffre, palier 1 »…
    pub role: String,
    /// Clé courte : `coffre_1`, `creature_2`, `chef`…
    pub cle: String,
    pub id: String,
    pub or_min: f64,
    pub or_moyen: f64,
    pub or_max: f64,
    /// Nombre moyen d'objets par sorte (hors or), dans l'ordre de `SORTES`.
    pub par_sorte: Vec<f64>,
}

pub const OR: &str = "common.items.utility.coins";
pub const SORTES: [&str; 6] =
    ["Potions et bombes", "Nourriture", "Armes", "Armures", "Minerais et matériaux", "Autres"];

fn sorte(id: &str) -> usize {
    let dans = |morceau: &str| id.contains(morceau);
    if dans(".consumable.") || dans("utility.bomb") || dans("potion") {
        0
    } else if dans(".food.") {
        1
    } else if dans(".weapons.") || dans("modular") || dans(".tool.") {
        2
    } else if dans(".armor.") {
        3
    } else if dans(".mineral.") || dans(".crafting_ing.") || dans(".log.") {
        4
    } else {
        5
    }
}

type Lots = Vec<(f32, LootSpec<String>)>;

fn table_de(id: &str) -> Result<Lots> {
    let table = Lottery::<LootSpec<String>>::load(id).map_err(|e| anyhow!("table de butin {id} illisible : {e}"))?;
    let table = table.read();
    // La table garde des poids cumulés (où chaque lot commence) : on retrouve le poids de chacun.
    let debuts: Vec<f32> = table.iter().map(|(debut, _)| *debut).chain([table.total()]).collect();
    Ok(table.iter().enumerate().map(|(i, (_, lot))| (debuts[i + 1] - debuts[i], lot.clone())).collect())
}

fn loterie(lots: &Lots, facteur: f64, total: &mut BTreeMap<String, f64>) -> Result<()> {
    let poids: f64 = lots.iter().map(|(p, _)| f64::from(*p)).sum();
    lots.iter().try_for_each(|(p, lot)| attendu(lot, facteur * f64::from(*p) / poids, total))
}

/// Nombre moyen de chaque objet pour `facteur` tirages de `butin`.
pub fn attendu(butin: &LootSpec<String>, facteur: f64, total: &mut BTreeMap<String, f64>) -> Result<()> {
    match butin {
        LootSpec::Item(id) => *total.entry(id.clone()).or_default() += facteur,
        LootSpec::LootTable(id) => loterie(&table_de(id)?, facteur, total)?,
        LootSpec::Lottery(lots) => loterie(lots, facteur, total)?,
        LootSpec::MultiDrop(lot, mini, maxi) => attendu(lot, facteur * f64::from(mini + maxi) / 2.0, total)?,
        LootSpec::All(lots) => lots.iter().try_for_each(|lot| attendu(lot, facteur, total))?,
        LootSpec::Nothing => {},
        _ => *total.entry("modular".to_string()).or_default() += facteur,
    }
    Ok(())
}

fn bornes_loterie(lots: &Lots) -> Result<(f64, f64)> {
    lots.iter().try_fold((f64::MAX, 0.0_f64), |(mini, maxi), (_, lot)| {
        bornes_or(lot).map(|(a, b)| (mini.min(a), maxi.max(b)))
    })
}

/// Le moins et le plus d'or qu'un tirage de `butin` peut donner.
pub fn bornes_or(butin: &LootSpec<String>) -> Result<(f64, f64)> {
    Ok(match butin {
        LootSpec::Item(id) if id == OR => (1.0, 1.0),
        LootSpec::LootTable(id) => bornes_loterie(&table_de(id)?)?,
        LootSpec::Lottery(lots) => bornes_loterie(lots)?,
        LootSpec::MultiDrop(lot, mini, maxi) => {
            let (a, b) = bornes_or(lot)?;
            (a * f64::from(*mini), b * f64::from(*maxi))
        },
        LootSpec::All(lots) => lots
            .iter()
            .try_fold((0.0, 0.0), |(mini, maxi), lot| bornes_or(lot).map(|(a, b)| (mini + a, maxi + b)))?,
        _ => (0.0, 0.0),
    })
}

impl Table {
    pub fn lire(role: &str, cle: &str, id: &str) -> Result<Self> {
        let butin = LootSpec::LootTable(id.to_string());
        let mut total = BTreeMap::new();
        attendu(&butin, 1.0, &mut total)?;
        let (or_min, or_max) = bornes_or(&butin)?;
        let arrondi = |x: f64| (x * 100.0).round() / 100.0;
        let mut par_sorte = vec![0.0; SORTES.len()];
        for (objet, nombre) in total.iter().filter(|(objet, _)| objet.as_str() != OR) {
            par_sorte[sorte(objet)] += nombre;
        }
        Ok(Self {
            role: role.to_string(),
            cle: cle.to_string(),
            id: id.to_string(),
            or_min,
            or_moyen: arrondi(total.get(OR).copied().unwrap_or(0.0)),
            or_max,
            par_sorte: par_sorte.into_iter().map(arrondi).collect(),
        })
    }
}

/// Les tables de `butin.ron`, dans l'ordre : coffres et créatures par palier, chef, boss, caisse.
pub fn tables() -> Result<Vec<Table>> {
    let butin = Butin::load().map_err(anyhow::Error::msg)?;
    let mut tables = Vec::new();
    for (i, id) in butin.coffres.iter().enumerate() {
        tables.push(Table::lire(&format!("Coffre, palier {}", i + 1), &format!("coffre_{}", i + 1), id)?);
    }
    for (i, id) in butin.creatures.iter().enumerate() {
        tables.push(Table::lire(&format!("Créature, palier {}", i + 1), &format!("creature_{}", i + 1), id)?);
    }
    tables.push(Table::lire("Chef", "chef", &butin.chef)?);
    tables.push(Table::lire("Boss", "boss", &butin.boss)?);
    tables.push(Table::lire("Caisse du décor", "caisse", &butin.caisse)?);
    Ok(tables)
}

/// Ce qu'on lit de `village.ron`.
#[derive(Clone, Debug, Deserialize)]
pub struct Village {
    pub prix_maison: u32,
}

/// Le prix du soin : il n'est pas dans `assets/`, mais dans les réglages par défaut du serveur.
pub fn prix_du_soin() -> Option<f64> {
    let source = std::fs::read_to_string("server/src/settings/mod.rs").ok()?;
    // La première mention est le champ, la suivante sa valeur par défaut.
    source
        .split("heal_price:")
        .skip(1)
        .find_map(|apres| apres.trim_start().split(|c: char| !c.is_ascii_digit()).next()?.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece() -> LootSpec<String> { LootSpec::Item(OR.to_string()) }

    /// Une table écrite à la main : 10 à 30 pièces toujours, puis une chance sur quatre de 4 pièces de plus.
    #[test]
    fn esperance_et_bornes() {
        let butin = LootSpec::All(vec![
            LootSpec::MultiDrop(Box::new(piece()), 10, 30),
            LootSpec::Lottery(vec![(3.0, LootSpec::Nothing), (1.0, LootSpec::MultiDrop(Box::new(piece()), 4, 4))]),
        ]);
        let mut total = BTreeMap::new();
        attendu(&butin, 1.0, &mut total).unwrap();
        assert_eq!(total[OR], 21.0);
        assert_eq!(bornes_or(&butin).unwrap(), (10.0, 34.0));
    }

    /// Une table lue par le jeu (poids cumulés) donne la même moyenne que le fichier relu tel quel.
    #[test]
    fn poids_des_tables_du_jeu() {
        let id = "common.loot_tables.icebreaker.butin.caisse";
        let chemin = common::assets::ASSETS_PATH.join("common/loot_tables/icebreaker/butin/caisse.ron");
        let lots: Lots = ron::from_str(&std::fs::read_to_string(chemin).unwrap()).unwrap();
        let mut du_fichier = BTreeMap::new();
        loterie(&lots, 1.0, &mut du_fichier).unwrap();
        let table = Table::lire("Caisse", "caisse", id).unwrap();
        assert!(du_fichier[OR] > 0.0);
        assert!((table.or_moyen - du_fichier[OR]).abs() < 0.01, "{} contre {}", table.or_moyen, du_fichier[OR]);
    }
}
