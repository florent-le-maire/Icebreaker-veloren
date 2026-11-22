//! Icebreaker: the village that grows (jalon C). The whole town is generated,
//! but only some of its houses stand: the others are plots to build on. Its
//! houses are ranked from the nearest to the main square; the first ones hold
//! the village buildings and always stand. Which ones stand is kept in the
//! campaign directory; the start and the inhabitants per house are in
//! `assets/icebreaker/village.ron`.

use crate::icebreaker_buildings::Building;
use common::assets::{AssetExt, Ron};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use tracing::{info, warn};
use world::{icebreaker::VillageHouse, site::Site};

/// File of the campaign directory holding what of the village stands.
const VILLAGE_FILE: &str = "village.ron";

#[derive(Clone, Debug, Deserialize)]
pub struct VillageConfig {
    /// Houses standing at the start of the campaign (buildings included).
    pub maisons_au_depart: usize,
    /// Inhabitants per standing house.
    pub habitants_par_maison: u32,
    /// Plots to build on offered at a time, nearest to the square first.
    pub terrains_ouverts: usize,
    /// What a house costs, in coins of the treasury.
    pub prix_maison: u32,
    /// How long a house takes to build, in hours of map time.
    pub heures_chantier: f64,
    /// The signs around the door of each building (by `Building::id`): names of
    /// `SpriteKind`, either side of the door (X.4).
    #[serde(default)]
    pub enseignes: std::collections::HashMap<String, Vec<String>>,
}

pub fn config() -> VillageConfig {
    Ron::<VillageConfig>::load_expect("icebreaker.village")
        .read()
        .0
        .clone()
}

/// What is saved of the village: the ranks of its standing houses, and the
/// houses being built.
#[derive(Debug, Default, Deserialize, Serialize)]
struct SavedVillage {
    construites: Vec<usize>,
    #[serde(default)]
    chantiers: Vec<Chantier>,
    /// Icebreaker VF.3 : au village fixe, les terrains bâtis par leur id dans le
    /// plan (un rang change si l'on ajoute un lieu au plan, pas un id).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    batis: Vec<String>,
    /// Icebreaker VF.3 : au village fixe, les chantiers par id du terrain (id, fin).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    chantiers_par_id: Vec<(String, f64)>,
}

fn village_file() -> Option<PathBuf> {
    std::env::var_os("ICEBREAKER_CAMPAIGN_DIR").map(|dir| PathBuf::from(dir).join(VILLAGE_FILE))
}

fn read_saved(saved: Option<&str>) -> Option<SavedVillage> {
    saved.and_then(|text| {
        ron::from_str::<SavedVillage>(text)
            .inspect_err(|err| warn!(?err, "Icebreaker: unreadable village, as at the start"))
            .ok()
    })
}

/// The ranks of the standing houses: those saved in the campaign, else the
/// first ones; the buildings' always. Icebreaker VF.3 : au village fixe (`ids`,
/// l'id de chaque maison), les bâtiments et les terrains bâtis enregistrés par
/// id ; au départ, rien d'autre que les bâtiments (les lieux au palier 0 ne sont
/// pas des terrains à bâtir).
fn standing_ranks(config: &VillageConfig, saved: Option<&str>, ids: &[String]) -> Vec<usize> {
    let saved = read_saved(saved);
    let mut ranks = if ids.is_empty() {
        saved.map_or_else(
            || (0..config.maisons_au_depart).collect(),
            |saved| saved.construites,
        )
    } else {
        let batis = saved.map(|saved| saved.batis).unwrap_or_default();
        ids.iter()
            .enumerate()
            .filter(|(_, id)| batis.contains(id))
            .map(|(rank, _)| rank)
            .collect()
    };
    ranks.extend(0..Building::ALL.len());
    ranks.sort_unstable();
    ranks.dedup();
    ranks
}

/// Icebreaker VF : les maisons du village fixe (décision 0022), rangées comme
/// les attend le reste : les bâtiments dans l'ordre de `Building::ALL`, trouvés
/// par leur id dans le plan, puis les terrains à bâtir par palier. Aucune si
/// le site n'a pas de plan ou s'il y manque un bâtiment (la ville tirée au
/// hasard prend alors le relais). La caisse, absente du croquis (Q63), est
/// dans la Guilde (le lieu de la charrette), une tuile plus loin que la porte ;
/// au plan, elle en occupe la moitié ouest. Rend aussi l'id de chaque maison
/// (VF.3).
pub fn houses_du_plan(lieux: &[(String, u32, VillageHouse)]) -> Option<(Vec<VillageHouse>, Vec<String>)> {
    if lieux.is_empty() {
        return None;
    }
    let lieu = |id: &str| lieux.iter().find(|(i, ..)| i == id).map(|(_, _, house)| *house);
    let mut houses = Vec::new();
    let mut ids = Building::ALL.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
    for building in Building::ALL {
        let house = lieu(building.id()).or_else(|| {
            (building == Building::Treasury).then_some(())?;
            let mut guilde = lieu(Building::Charrette.id())?;
            guilde.inside += guilde.inside - guilde.front;
            // La moitié ouest de la Guilde : un clic au plan y trouve la caisse, au milieu la charrette.
            guilde.bounds.max.x = guilde.bounds.center().x;
            Some(guilde)
        });
        let Some(house) = house else {
            warn!(id = building.id(), "Icebreaker : bâtiment absent du plan du village");
            return None;
        };
        houses.push(house);
    }
    // Icebreaker VF.5 : la caisse dans la Guilde, la charrette en garde la moitié est (le plan
    // à l'échelle les montre côte à côte).
    let rang = |b: Building| Building::ALL.iter().position(|x| *x == b).unwrap_or(0);
    let (caisse, charrette) = (rang(Building::Treasury), rang(Building::Charrette));
    if lieu(Building::Treasury.id()).is_none() {
        houses[charrette].bounds.min.x = houses[caisse].bounds.max.x;
    }
    let mut terrains = lieux
        .iter()
        .filter(|(id, palier, _)| *palier > 0 && Building::ALL.iter().all(|b| b.id() != id))
        .collect::<Vec<_>>();
    terrains.sort_by_key(|(_, palier, _)| *palier);
    houses.extend(terrains.iter().map(|(_, _, house)| *house));
    ids.extend(terrains.iter().map(|(id, ..)| id.clone()));
    Some((houses, ids))
}

/// The first `count` plots to build on (rank, house): the houses not standing,
/// nearest to the square first.
fn open_plots(
    houses: &[VillageHouse],
    standing: &[usize],
    count: usize,
) -> Vec<(usize, VillageHouse)> {
    houses
        .iter()
        .copied()
        .enumerate()
        .filter(|(rank, _)| !standing.contains(rank))
        .take(count)
        .collect()
}

/// A house being built on a plot (C.3): its rank, and when it is done (map
/// time, `TimeOfDay`).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Chantier {
    pub rang: usize,
    pub fin: f64,
}

/// The village as it stands (none outside the village).
#[derive(Default)]
pub struct IcebreakerVillage {
    /// The town's houses, nearest to the square first.
    houses: Vec<VillageHouse>,
    /// The ranks of the standing houses.
    standing: Vec<usize>,
    chantiers: Vec<Chantier>,
    path: Option<PathBuf>,
    /// Icebreaker VF.3 : l'id de chaque maison au village fixe (vide pour la
    /// ville tirée au hasard).
    ids: Vec<String>,
}

impl IcebreakerVillage {
    /// The village of the town of `site`, whose `houses` are ranked: tells the
    /// world which ones stand and how many people live there. Icebreaker VF.3 :
    /// `ids`, l'id de chaque maison au village fixe (vide sinon).
    pub fn new(site: &Site, houses: &[VillageHouse], ids: &[String]) -> Self {
        let config = config();
        let path = village_file();
        let saved = path.as_ref().and_then(|path| std::fs::read_to_string(path).ok());
        let standing = standing_ranks(&config, saved.as_deref(), ids);
        let chantiers = saved_chantiers(read_saved(saved.as_deref()), ids);
        let built = standing
            .iter()
            .filter_map(|rank| houses.get(*rank))
            .map(|house| house.plot)
            .collect::<HashSet<_>>();
        let inhabitants = built.len() as u32 * config.habitants_par_maison;
        let signs = Building::ALL
            .iter()
            .enumerate()
            .filter_map(|(i, building)| {
                let sprites = config.enseignes.get(building.id())?.clone();
                Some((houses.get(i)?.plot, sprites))
            })
            .collect::<Vec<_>>();
        world::icebreaker::set_village(site, &built, inhabitants, &signs);
        let village = Self {
            houses: houses.to_vec(),
            standing,
            chantiers,
            path,
            ids: ids.to_vec(),
        };
        info!(plots = ?village.open_plots(), chantiers = ?village.chantiers, "Icebreaker: plots to build on");
        village
    }

    /// The plots to build on now (chantiers included): (rank, bounds, altitude).
    pub fn open_plots(&self) -> Vec<(usize, vek::Aabr<i32>, i32)> {
        open_plots(&self.houses, &self.standing, config().terrains_ouverts)
            .into_iter()
            .map(|(rank, house)| (rank, house.bounds, house.alt))
            .collect()
    }

    /// Icebreaker VF.3 : le rang du terrain d'id `id` au village fixe.
    pub fn rang_de(&self, id: &str) -> Option<usize> { self.ids.iter().position(|i| i == id) }

    /// The house being built on the plot of rank `rang`, if any.
    pub fn chantier(&self, rang: usize) -> Option<Chantier> {
        self.chantiers.iter().find(|c| c.rang == rang).copied()
    }

    /// Why a house cannot be built on the plot of rank `rang`, if it cannot.
    pub fn cannot_build(&self, rang: usize) -> Option<String> {
        if self.chantier(rang).is_some() {
            Some("Un chantier est déjà en cours sur ce terrain.".to_string())
        } else if !self.open_plots().iter().any(|(rank, ..)| *rank == rang) {
            Some("Ce terrain n'est pas à bâtir.".to_string())
        } else {
            None
        }
    }

    /// Starts building a house on the plot of rank `rang` at map time `now`
    /// (check `cannot_build` first); returns when it is done.
    pub fn build(&mut self, rang: usize, now: f64) -> f64 {
        let fin = now + config().heures_chantier * 3600.0;
        self.chantiers.push(Chantier { rang, fin });
        self.save();
        fin
    }

    /// The chantiers done at map time `now`: their houses now stand (seen at
    /// the next start of the server until C.4). Returns their ranks.
    pub fn finish_due(&mut self, now: f64) -> Vec<usize> {
        let (done, going): (Vec<_>, Vec<_>) = self.chantiers.iter().partition(|c| c.fin <= now);
        if done.is_empty() {
            return Vec::new();
        }
        self.chantiers = going;
        let ranks = done.iter().map(|c| c.rang).collect::<Vec<_>>();
        self.standing.extend(&ranks);
        self.standing.sort_unstable();
        self.save();
        ranks
    }

    fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let saved = self.to_saved();
        match crate::icebreaker_warehouse::write_campaign_file(path, &saved) {
            Ok(()) => info!(?saved, "Icebreaker: village saved"),
            Err(err) => warn!(?err, ?path, "Icebreaker: could not save the village"),
        }
    }

    /// Ce qui s'enregistre : par rang pour la ville tirée au hasard, par id au
    /// village fixe (VF.3 ; les bâtiments, toujours debout, n'y sont pas).
    fn to_saved(&self) -> SavedVillage {
        if self.ids.is_empty() {
            return SavedVillage {
                construites: self.standing.clone(),
                chantiers: self.chantiers.clone(),
                ..SavedVillage::default()
            };
        }
        let id = |rang: usize| self.ids.get(rang).cloned();
        SavedVillage {
            batis: self
                .standing
                .iter()
                .filter(|rang| **rang >= Building::ALL.len())
                .filter_map(|rang| id(*rang))
                .collect(),
            chantiers_par_id: self
                .chantiers
                .iter()
                .filter_map(|c| Some((id(c.rang)?, c.fin)))
                .collect(),
            ..SavedVillage::default()
        }
    }
}

/// Les chantiers enregistrés : par rang, ou par id au village fixe (VF.3 ; un
/// id qui n'est plus au plan est oublié).
fn saved_chantiers(saved: Option<SavedVillage>, ids: &[String]) -> Vec<Chantier> {
    let Some(saved) = saved else {
        return Vec::new();
    };
    if ids.is_empty() {
        return saved.chantiers;
    }
    saved
        .chantiers_par_id
        .iter()
        .filter_map(|(id, fin)| {
            let rang = ids.iter().position(|i| i == id)?;
            Some(Chantier { rang, fin: *fin })
        })
        .collect()
}

/// `/batir <terrain>` (the « Bâtir » button of a plot's panel, C.3): starts
/// building a house on the plot of this rank, paid by the treasury.
pub(crate) fn handle_batir(
    server: &mut crate::Server,
    client: specs::Entity,
    _target: specs::Entity,
    args: Vec<String>,
    _action: &common::cmd::ServerChatCommand,
) -> Result<(), common::comp::Content> {
    use crate::icebreaker_treasury::IcebreakerTreasury;
    use common::comp::{self, Content};
    use specs::WorldExt;

    let ecs = server.state.ecs();
    // Icebreaker VF.3 : au village fixe, un terrain se désigne aussi par son id dans le plan.
    let Some(rang) = args.first().and_then(|arg| {
        arg.parse::<usize>()
            .ok()
            .or_else(|| ecs.read_resource::<IcebreakerVillage>().rang_de(arg))
    }) else {
        return Err(Content::Plain("Quel terrain ? (/batir <numéro ou id>)".to_string()));
    };
    if let Some(why) = ecs.read_resource::<IcebreakerVillage>().cannot_build(rang) {
        return Err(Content::Plain(why));
    }
    let config = config();
    let price = u64::from(config.prix_maison);
    {
        let mut treasury = ecs.write_resource::<IcebreakerTreasury>();
        if !treasury.spend(price) {
            return Err(Content::Plain(format!(
                "La caisse n'a que {} pièces d'or ; une maison en coûte {price}.",
                treasury.coins()
            )));
        }
    }
    // The treasurer's name shows the treasury.
    crate::icebreaker_treasury::refresh_treasurer_name(ecs);
    let now = ecs.read_resource::<common::resources::TimeOfDay>().0;
    ecs.write_resource::<IcebreakerVillage>().build(rang, now);
    let alias = ecs
        .read_storage::<comp::Player>()
        .get(client)
        .map_or_else(|| "Quelqu'un".to_string(), |player| player.alias.clone());
    server.notify_players(common_net::msg::ServerGeneral::server_msg(
        comp::ChatType::Meta,
        Content::Plain(format!(
            "{alias} lance un chantier de maison au village ({price} or de la caisse) : fini dans \
             {} h de carte.",
            config.heures_chantier
        )),
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Icebreaker VF : chaque bâtiment est le lieu de même id ; la caisse est dans la Guilde.
    #[test]
    fn le_village_fixe_range_ses_batiments_par_id() {
        let house = |x: i32| VillageHouse {
            plot: common::store::Id::new(x as u64),
            front: vek::Vec2::new(x * 100, -6),
            inside: vek::Vec2::new(x * 100, 0),
            bounds: vek::Aabr { min: vek::Vec2::new(x * 100, 0), max: vek::Vec2::new(x * 100 + 10, 10) },
            alt: 0,
        };
        let ids = ["salle_armes", "terrain_b", "charrette", "entrepot", "intendance", "terrain_a", "forge", "depart"];
        let paliers = [0, 2, 0, 0, 0, 1, 0, 0];
        let lieux = ids
            .iter()
            .zip(paliers)
            .enumerate()
            .map(|(i, (id, palier))| (id.to_string(), palier, house(i as i32)))
            .collect::<Vec<_>>();
        let (houses, ids_plan) = houses_du_plan(&lieux).unwrap();
        assert_eq!(ids_plan, ["entrepot", "caisse", "intendance", "forge", "depart", "charrette", "salle_armes", "terrain_a", "terrain_b"]);
        let plots = houses.iter().map(|h| h.plot).collect::<Vec<_>>();
        let id = |i: u64| common::store::Id::new(i);
        // entrepot, caisse (= charrette), intendance, forge, depart, charrette, salle_armes, puis les terrains.
        assert_eq!(plots, [id(3), id(2), id(4), id(6), id(7), id(2), id(0), id(5), id(1)]);
        assert_eq!(houses[1].inside, vek::Vec2::new(200, 6), "la caisse, une tuile plus loin que la porte");
        assert_eq!(houses[1].bounds.max.x, 205, "la caisse, la moitié ouest de la Guilde");
        assert_eq!(houses[5].bounds.min.x, 205, "la charrette, la moitié est (VF.5)");
        // Un bâtiment manque : pas de village fixe.
        assert!(houses_du_plan(&lieux[1..]).is_none());
        assert!(houses_du_plan(&[]).is_none());
    }

    #[test]
    fn the_village_starts_small_and_keeps_its_buildings() {
        let config = config();
        let start = standing_ranks(&config, None, &[]);
        assert_eq!(start, (0..config.maisons_au_depart).collect::<Vec<_>>());
        assert!(start.len() >= Building::ALL.len());

        // Saved: houses 0, 1 and 9; the buildings (0 to 6) always stand.
        let saved = standing_ranks(&config, Some("(construites: [9, 1, 0])"), &[]);
        assert_eq!(saved, vec![0, 1, 2, 3, 4, 5, 6, 9]);
    }

    // Icebreaker VF.3 : au village fixe, les terrains se gardent par id.
    #[test]
    fn le_village_fixe_garde_ses_terrains_par_id() {
        let config = config();
        let mut ids = Building::ALL.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
        ids.extend(["terrain_a".to_string(), "terrain_b".to_string()]);
        let n = Building::ALL.len();
        // Au départ, les bâtiments seuls, quel que soit `maisons_au_depart`.
        assert_eq!(standing_ranks(&config, None, &ids), (0..n).collect::<Vec<_>>());
        // Un id enregistré tient debout ; un id qui n'est plus au plan est oublié.
        let texte = "(construites: [], batis: [\"terrain_b\", \"ancien\"], chantiers_par_id: [(\"terrain_a\", 5.0), (\"ancien\", 1.0)])";
        let mut attendu = (0..n).collect::<Vec<_>>();
        attendu.push(n + 1);
        assert_eq!(standing_ranks(&config, Some(texte), &ids), attendu);
        let chantiers = saved_chantiers(read_saved(Some(texte)), &ids);
        assert_eq!(chantiers, vec![Chantier { rang: n, fin: 5.0 }]);

        // Et s'enregistrent par id, sans les bâtiments.
        let village = IcebreakerVillage {
            houses: Vec::new(),
            standing: attendu,
            chantiers,
            path: None,
            ids: ids.clone(),
        };
        let saved = village.to_saved();
        assert_eq!(saved.batis, ["terrain_b"]);
        assert_eq!(saved.chantiers_par_id, [("terrain_a".to_string(), 5.0)]);
        assert!(saved.construites.is_empty());
        assert_eq!(village.rang_de("terrain_a"), Some(n));
    }

    #[test]
    fn the_plots_offered_are_the_nearest_free_houses() {
        let house = |x| VillageHouse {
            plot: common::store::Id::new(x as u64),
            front: vek::Vec2::new(x, 0),
            inside: vek::Vec2::new(x, 1),
            alt: 0,
            bounds: vek::Aabr {
                min: vek::Vec2::new(x, 0),
                max: vek::Vec2::new(x + 5, 5),
            },
        };
        let houses = (0..10).map(house).collect::<Vec<_>>();
        let ranks = open_plots(&houses, &[0, 1, 2, 3, 5], 3)
            .into_iter()
            .map(|(rank, _)| rank)
            .collect::<Vec<_>>();
        assert_eq!(ranks, vec![4, 6, 7]);
    }

    #[test]
    fn a_house_is_built_on_a_free_plot_and_stands_once_done() {
        let house = |x| VillageHouse {
            plot: common::store::Id::new(x as u64),
            front: vek::Vec2::new(x, 0),
            inside: vek::Vec2::new(x, 1),
            alt: 0,
            bounds: vek::Aabr {
                min: vek::Vec2::new(x, 0),
                max: vek::Vec2::new(x + 5, 5),
            },
        };
        let mut village = IcebreakerVillage {
            houses: (0..20).map(house).collect(),
            standing: (0..7).collect(),
            chantiers: Vec::new(),
            path: None,
            ids: Vec::new(),
        };
        assert!(village.cannot_build(2).is_some(), "a standing house");
        assert!(village.cannot_build(19).is_some(), "too far, not offered yet");
        assert!(village.cannot_build(7).is_none());

        let fin = village.build(7, 1000.0);
        assert_eq!(fin, 1000.0 + config().heures_chantier * 3600.0);
        assert!(village.cannot_build(7).is_some(), "already being built");
        assert_eq!(village.finish_due(fin - 1.0), Vec::<usize>::new());
        assert_eq!(village.finish_due(fin), vec![7]);
        assert!(village.standing.contains(&7) && village.chantier(7).is_none());

        let saved = ron::from_str::<SavedVillage>(
            "(construites: [0, 1], chantiers: [(rang: 8, fin: 5.0)])",
        )
        .unwrap();
        assert_eq!(saved.chantiers, vec![Chantier { rang: 8, fin: 5.0 }]);
    }
}
