use crate::client::Client;
use common::{
    calendar::Calendar,
    comp::Player,
    resources::{Time, TimeOfDay, TimeScale},
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use serde::{Deserialize, Serialize};
use specs::{Join, Read, ReadStorage, Write};
use std::path::PathBuf;
use tracing::{info, warn};

/// File of the campaign directory holding the map clock.
const CLOCK_FILE: &str = "horloge.ron";

/// What is saved of the map clock.
#[derive(Serialize, Deserialize)]
struct SavedClock {
    time_of_day: f64,
}

/// Icebreaker: the map clock is shared by every instance of a campaign.
///
/// An instance with players saves its time of day in the campaign directory;
/// an empty instance takes it back when its first player arrives. The clock
/// thus follows the players from one instance to the next and stands still
/// while nobody plays.
#[derive(Default)]
pub struct IcebreakerClock {
    path: Option<PathBuf>,
    save_interval: f64,
    last_save: f64,
    had_players: bool,
}

impl IcebreakerClock {
    pub fn new(campaign_dir: Option<PathBuf>, save_interval: f64) -> Self {
        Self {
            path: campaign_dir.map(|dir| dir.join(CLOCK_FILE)),
            save_interval,
            last_save: 0.0,
            had_players: false,
        }
    }

    /// The saved time of day, if any.
    pub fn load(&self) -> Option<f64> {
        let text = std::fs::read_to_string(self.path.as_ref()?).ok()?;
        match ron::from_str::<SavedClock>(&text) {
            Ok(clock) => Some(clock.time_of_day),
            Err(err) => {
                warn!(?err, "Icebreaker: unreadable map clock, ignored");
                None
            },
        }
    }

    fn save(&self, time_of_day: f64) {
        let Some(path) = &self.path else {
            return;
        };
        let result = ron::to_string(&SavedClock { time_of_day })
            .map_err(std::io::Error::other)
            .and_then(|text| {
                // Written aside then renamed, so that another instance never reads half a file.
                let tmp = path.with_extension("ron.tmp");
                std::fs::write(&tmp, text)?;
                std::fs::rename(&tmp, path)
            });
        if let Err(err) = result {
            warn!(?err, ?path, "Icebreaker: could not save the map clock");
        }
    }
}

/// Icebreaker: keeps the map clock of the campaign (see [`IcebreakerClock`]).
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Write<'a, IcebreakerClock>,
        Write<'a, TimeOfDay>,
        Read<'a, Time>,
        Read<'a, Calendar>,
        Read<'a, TimeScale>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Client>,
    );

    const NAME: &'static str = "icebreaker_clock";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (mut clock, mut time_of_day, time, calendar, time_scale, players, clients): Self::SystemData,
    ) {
        if clock.path.is_none() {
            return;
        }

        let has_players = (&players, &clients).join().next().is_some();
        if has_players && !clock.had_players {
            // First player in: the clock goes on from where the last instance left it.
            if let Some(saved) = clock.load() {
                info!(time_of_day = saved, "Icebreaker: map clock taken from the campaign");
                time_of_day.0 = saved;
                let mut msg = None;
                for client in (&clients).join() {
                    let msg = msg.get_or_insert_with(|| {
                        client.prepare(ServerGeneral::TimeOfDay(
                            *time_of_day,
                            (*calendar).clone(),
                            *time,
                            *time_scale,
                        ))
                    });
                    let _ = client.send_prepared(msg);
                }
            }
            clock.last_save = time.0;
        } else if clock.had_players
            && (!has_players || time.0 - clock.last_save >= clock.save_interval)
        {
            // Saved regularly while people play, and when the last one leaves.
            clock.save(time_of_day.0);
            clock.last_save = time.0;
        }
        clock.had_players = has_players;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_saved_clock_is_read_back() {
        let dir = std::env::temp_dir().join(format!("icebreaker-clock-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let clock = IcebreakerClock::new(Some(dir.clone()), 5.0);
        assert_eq!(clock.load(), None);
        clock.save(45_000.5);
        assert_eq!(clock.load(), Some(45_000.5));
        assert_eq!(IcebreakerClock::new(None, 5.0).load(), None);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
