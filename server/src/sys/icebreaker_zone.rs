use crate::client::Client;
use common::{
    comp::{ChatType, Content, ForceUpdate, Player, Pos, Vel},
    resources::Time,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use hashbrown::HashMap;
use specs::{Entities, Entity, Join, Read, ReadStorage, Write, WriteStorage};
use vek::*;

/// Cooldown (in seconds) between two "can't leave the area" messages.
const NOTIFY_TIME: f64 = 5.0;

/// The playable area of an encounter instance.
#[derive(Copy, Clone, Debug)]
pub struct Zone {
    pub center: Vec2<f32>,
    pub radius: f32,
}

impl Zone {
    /// Where a player at `pos` must be put back, if outside the zone.
    pub fn push_back(&self, pos: Vec2<f32>) -> Option<Vec2<f32>> {
        let offset = pos - self.center;
        (offset.magnitude_squared() > self.radius.powi(2)).then(|| {
            let dir = offset.try_normalized().unwrap_or(Vec2::unit_y());
            self.center + dir * (self.radius - 1.0)
        })
    }
}

/// Where a player at `pos` must be put back, if outside every zone: on the
/// edge of the nearest one (the campaign has a zone per place, decision 0016).
pub fn push_back_into(zones: &[Zone], pos: Vec2<f32>) -> Option<Vec2<f32>> {
    let pushes = zones
        .iter()
        .map(|zone| zone.push_back(pos))
        .collect::<Option<Vec<_>>>()?;
    pushes
        .into_iter()
        .min_by(|a, b| a.distance_squared(pos).total_cmp(&b.distance_squared(pos)))
}

/// Icebreaker: the zones of this server, if it runs an encounter instance or
/// the campaign.
#[derive(Default)]
pub struct IcebreakerZone {
    zones: Vec<Zone>,
    last_notified: HashMap<Entity, f64>,
}

impl IcebreakerZone {
    pub fn new(zones: Vec<Zone>) -> Self {
        Self {
            zones,
            last_notified: HashMap::new(),
        }
    }
}

/// Icebreaker: pushes players back inside the zone of an encounter instance.
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Write<'a, IcebreakerZone>,
        Read<'a, Time>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Client>,
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
        WriteStorage<'a, ForceUpdate>,
    );

    const NAME: &'static str = "icebreaker_zone";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            mut zone,
            time,
            players,
            clients,
            mut positions,
            mut velocities,
            mut force_updates,
        ): Self::SystemData,
    ) {
        if zone.zones.is_empty() {
            return;
        }

        for (entity, _, client, pos, vel) in (
            &entities,
            &players,
            &clients,
            &mut positions,
            &mut velocities,
        )
            .join()
        {
            let Some(inside) = push_back_into(&zone.zones, pos.0.xy()) else {
                continue;
            };
            pos.0 = inside.with_z(pos.0.z);
            vel.0 = Vec3::new(0.0, 0.0, vel.0.z);
            if let Some(force_update) = force_updates.get_mut(entity) {
                force_update.update();
            }

            let last = zone.last_notified.get(&entity).copied();
            if last.is_none_or(|t| time.0 - t > NOTIFY_TIME) {
                zone.last_notified.insert(entity, time.0);
                client.send_fallible(ServerGeneral::server_msg(
                    ChatType::CommandInfo,
                    Content::Plain("Vous ne pouvez pas quitter la zone.".to_string()),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn players_outside_the_zone_are_put_back_on_its_edge() {
        let zone = Zone {
            center: Vec2::new(100.0, 100.0),
            radius: 50.0,
        };
        assert_eq!(zone.push_back(Vec2::new(120.0, 100.0)), None);
        assert_eq!(
            zone.push_back(Vec2::new(200.0, 100.0)),
            Some(Vec2::new(149.0, 100.0))
        );
    }

    #[test]
    fn with_several_zones_players_stay_in_any_and_go_back_to_the_nearest() {
        let zones = [
            Zone {
                center: Vec2::new(100.0, 100.0),
                radius: 50.0,
            },
            Zone {
                center: Vec2::new(1000.0, 100.0),
                radius: 50.0,
            },
        ];
        assert_eq!(push_back_into(&zones, Vec2::new(1020.0, 100.0)), None);
        assert_eq!(
            push_back_into(&zones, Vec2::new(900.0, 100.0)),
            Some(Vec2::new(951.0, 100.0))
        );
        assert_eq!(push_back_into(&[], Vec2::zero()), None);
    }
}
