//! Icebreaker: generates a small world holding a cave (as
//! `icebreaker.ps1 donjon -Site Grotte` does), then checks its blocks: the
//! pockets and the bores are hollow, from the mouth down.
//!
//! `cargo run -p veloren-world --example icebreaker_grotte -- [seed]`

use veloren_world::{
    World,
    sim::{FileOpts, GenOpts, WorldOpts},
};

fn main() {
    let seed = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1234);
    // SAFETY: set before any other thread is started.
    unsafe { std::env::set_var("ICEBREAKER_INSTANCE_SITE", "Grotte") };
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
    let Some((summary, problems)) =
        veloren_world::icebreaker::check_generated_grotte(&world, index.as_index_ref())
    else {
        eprintln!("pas de grotte dans ce monde (graine {seed})");
        std::process::exit(2);
    };
    println!("{summary}");
    for problem in &problems {
        println!("  - {problem}");
    }
    println!("graine {seed} : {} problème(s)", problems.len());
    if !problems.is_empty() {
        std::process::exit(1);
    }
}
