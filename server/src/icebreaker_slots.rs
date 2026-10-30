//! Icebreaker: the dungeon slots of the campaign (decision 0016). The campaign
//! server generates one world: the village and, far from it, a few dungeons
//! (`assets/icebreaker/campagne.ron`). An expedition goes to a free slot of its
//! destination, each with its own closed zone.

use common::{store::Id, terrain::TerrainChunkSize};
use tracing::info;
use vek::*;
use world::{IndexRef, World, site::Site};

/// A dungeon slot.
#[derive(Clone, Debug)]
pub struct Slot {
    /// Its site name (`icebreaker.ps1 -Site`), e.g. `Gnarling`.
    pub name: String,
    pub site: Id<Site>,
    /// Center and radius of the site (world coordinates).
    pub center: Vec2<f32>,
    pub radius: f32,
    /// Radius of its closed zone: an expedition is there while one of its
    /// players is within it.
    pub zone_radius: f32,
    /// Where the expedition arrives.
    pub arrival: Vec3<f32>,
}

/// The dungeon slots (none outside the campaign).
#[derive(Default)]
pub struct IcebreakerSlots(pub Vec<Slot>);

impl Slot {
    /// Whether `pos` is within its zone.
    pub fn holds(&self, pos: Vec2<f32>) -> bool {
        pos.distance_squared(self.center) <= self.zone_radius.powi(2)
    }
}

/// Where players arrive at a dungeon site: out of the entrance tunnel of a
/// dungeon of our generator, or just outside the site, on the side facing the
/// map center.
pub fn arrival(site: &Site, map_center: Vec2<i32>) -> Vec2<i32> {
    world::icebreaker::dungeon_entrance(site).unwrap_or_else(|| {
        const MARGIN: f32 = 24.0;
        let toward_center = (map_center - site.origin)
            .as_::<f32>()
            .try_normalized()
            .unwrap_or(Vec2::unit_y());
        site.origin + (toward_center * (site.radius() + MARGIN)).as_::<i32>()
    })
}

impl IcebreakerSlots {
    /// The dungeon slots of the campaign's world.
    pub fn new(world: &World, index: IndexRef, zone_margin: f32) -> Self {
        let center_chunk = world.sim().map_size_lg().chunks().map(i32::from) / 2;
        let map_center = TerrainChunkSize::center_wpos(center_chunk);
        let mut names = world::icebreaker::campaign_slot_names();
        names.dedup();
        let slots = names
            .into_iter()
            .filter_map(|name| Some((world::icebreaker::site_kind(&name)?, name)))
            .flat_map(|(kind, name)| {
                world
                    .civs()
                    .sites()
                    .filter(move |site| site.kind == kind)
                    .filter_map(|site| site.site_tmp)
                    .map(move |site| (name.clone(), site))
            })
            .map(|(name, id)| {
                let site = &index.sites[id];
                let arrival = world.find_accessible_pos(index, arrival(site, map_center), false);
                let slot = Slot {
                    name,
                    site: id,
                    center: site.origin.as_(),
                    radius: site.radius(),
                    zone_radius: site.radius() + zone_margin,
                    arrival,
                };
                info!(?slot.name, ?slot.center, ?slot.arrival, "Icebreaker dungeon slot");
                slot
            })
            .collect();
        Self(slots)
    }
}

impl IcebreakerSlots {
    /// A slot of site `site` holding none of the `players` (positions).
    pub fn free(&self, site: &str, players: &[Vec2<f32>]) -> Option<&Slot> {
        self.0
            .iter()
            .find(|slot| slot.name == site && !players.iter().any(|pos| slot.holds(*pos)))
    }

    /// The slot holding `pos`, if any.
    pub fn at(&self, pos: Vec2<f32>) -> Option<&Slot> { self.0.iter().find(|slot| slot.holds(pos)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_with_a_player_in_its_zone_is_taken() {
        let slot = |name: &str, x: f32| Slot {
            name: name.to_string(),
            site: Id::new(0),
            center: Vec2::new(x, 0.0),
            radius: 50.0,
            zone_radius: 100.0,
            arrival: Vec3::new(x, 60.0, 0.0),
        };
        let slots = IcebreakerSlots(vec![
            slot("Gnarling", 0.0),
            slot("Gnarling", 1000.0),
            slot("Genere", 2000.0),
        ]);
        let players = [Vec2::new(90.0, 0.0)];
        assert_eq!(slots.free("Gnarling", &players).map(|s| s.center.x), Some(1000.0));
        assert_eq!(slots.free("Genere", &players).map(|s| s.center.x), Some(2000.0));
        let players = [Vec2::new(0.0, 0.0), Vec2::new(1000.0, 50.0)];
        assert!(slots.free("Gnarling", &players).is_none());
        assert!(slots.free("Haniwa", &[]).is_none());
        assert_eq!(slots.at(Vec2::new(2050.0, 0.0)).map(|s| s.name.as_str()), Some("Genere"));
        assert!(slots.at(Vec2::new(500.0, 0.0)).is_none());
    }
}
