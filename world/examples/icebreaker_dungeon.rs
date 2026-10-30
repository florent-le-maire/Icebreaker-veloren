//! Icebreaker: generates a small world holding a dungeon of our generator (as
//! `icebreaker.ps1 donjon -Site Genere` does), then checks its blocks against
//! its plan: every reachable cell has a floor and room to stand.
//!
//! `cargo run -p veloren-world --example icebreaker_dungeon -- [seed]`
//! (the mission comes from the same `ICEBREAKER_DONJON_*` variables as the
//! server's).

use veloren_world::{
    World,
    sim::{FileOpts, GenOpts, WorldOpts},
};

fn main() {
    let seed = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1234);
    // SAFETY: set before any other thread is started.
    unsafe { std::env::set_var("ICEBREAKER_INSTANCE_SITE", "Genere") };
    let threadpool = rayon::ThreadPoolBuilder::new().build().unwrap();
    let (world, index) = World::generate(
        seed,
        WorldOpts {
            seed_elements: true,
            world_file: FileOpts::Generate(GenOpts {
                x_lg: 6,
                y_lg: 6,
                ..GenOpts::default()
            }),
            calendar: None,
        },
        &threadpool,
        &|_| {},
    );
    let Some((plan, problems)) =
        veloren_world::icebreaker::check_generated_dungeon(&world, index.as_index_ref())
    else {
        eprintln!("pas de donjon généré dans ce monde (graine {seed})");
        std::process::exit(2);
    };
    println!("{plan}");
    for problem in &problems {
        println!("PROBLÈME {problem}");
    }
    println!("graine {seed} : {} problème(s)", problems.len());
    std::process::exit(if problems.is_empty() { 0 } else { 1 });
}
