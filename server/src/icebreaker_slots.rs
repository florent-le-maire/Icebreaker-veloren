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
    /// Where the expedition arrives.
    pub arrival: Vec3<f32>,
}

/// The dungeon slots (none outside the campaign).
#[derive(Default)]
pub struct IcebreakerSlots(pub Vec<Slot>);

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
    pub fn new(world: &World, index: IndexRef) -> Self {
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
                    arrival,
                };
                info!(?slot.name, ?slot.center, ?slot.arrival, "Icebreaker dungeon slot");
                slot
            })
            .collect();
        Self(slots)
    }
}
