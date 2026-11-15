//! Icebreaker: draws a cave without the workshop page, to check its shape:
//! `cargo run -p icebreaker-caverne --example coupes -- [profil] [graine] [dossier]`
//! writes `<profil>-<graine>-dessus.png` (seen from above: the highest
//! floors, lighter when higher, the trail in yellow, the zones in red) and
//! `-cote.png` (seen from the side).

use icebreaker_caverne::{Formes, generer};
use image::{Rgb, RgbImage};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string("assets/icebreaker/grottes/formes.ron").expect("lancer depuis la racine du dépôt");
    let formes = Formes::lire(&text).unwrap();
    if args.get(1).is_some_and(|a| a == "stats") {
        // How often the first seed tried gives a whole trail, and how long it takes.
        for (nom, profil) in &formes.profils {
            let start = std::time::Instant::now();
            let caves: Vec<_> = (0..40).map(|g| generer(&profil.forme, g, &|_, _| 0, i32::MIN)).collect();
            println!(
                "{nom} : {} sur 40 au premier essai, {} coupée(s), {:?} par grotte",
                caves.iter().filter(|c| c.essais == 1).count(),
                caves.iter().filter(|c| !c.praticable).count(),
                start.elapsed() / 40
            );
        }
        return;
    }
    let graine: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let (nom, forme) = formes.tirer(graine, args.get(1).map(String::as_str));
    let dossier = args.get(3).cloned().unwrap_or_else(|| ".".into());
    let start = std::time::Instant::now();
    let c = generer(forme, graine, &|_, _| 0, i32::MIN);
    println!("{nom}, graine {graine} ({:?}) : {}", start.elapsed(), c.resume);
    if !c.praticable {
        ruptures(&c);
    }
    let g = &c.grille;
    let (min, max) = (g.min, g.max());
    let shade = |z: i32| 60 + (190 * (z - min[2]) / (max[2] - min[2])) as u8;
    let mut dessus = RgbImage::new(g.taille[0] as u32, g.taille[1] as u32);
    let mut cote = RgbImage::new(g.taille[0] as u32, g.taille[2] as u32);
    for i in g.cellules_air() {
        let [x, y, z] = g.position(i);
        let (px, py, pz) = ((x - min[0]) as u32, (max[1] - 1 - y) as u32, (max[2] - 1 - z) as u32);
        let trail = g.sentier(x, y, z - 1);
        if !g.air(x, y, z - 1) {
            let seen = dessus.get_pixel(px, py).0;
            if trail {
                dessus.put_pixel(px, py, Rgb([255, 200, 40]));
            } else if seen[0] != 255 && shade(z) > seen[1] {
                dessus.put_pixel(px, py, Rgb([shade(z) / 2, shade(z), shade(z)]));
            }
        }
        let seen = cote.get_pixel(px, pz).0;
        if trail {
            cote.put_pixel(px, pz, Rgb([255, 200, 40]));
        } else if seen[0] != 255 {
            let v = seen[1].saturating_add(12).max(50);
            cote.put_pixel(px, pz, Rgb([v / 2, v, v]));
        }
    }
    for zone in &c.zones {
        let [x, y, _] = zone.ancre;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            dessus.put_pixel((x + dx - min[0]) as u32, (max[1] - 1 - y - dy) as u32, Rgb([255, 40, 40]));
        }
    }
    dessus.save(format!("{dossier}/{nom}-{graine}-dessus.png")).unwrap();
    cote.save(format!("{dossier}/{nom}-{graine}-cote.png")).unwrap();
}

/// Where the trail of each way stops being walked from the mouth, if it does.
#[allow(dead_code)]
fn ruptures(c: &icebreaker_caverne::Caverne) {
    let pieds = icebreaker_caverne::marcher(&c.grille, c.depart);
    let g = &c.grille;
    for (n, lien) in c.liens.iter().enumerate() {
        for (i, p) in lien.points.iter().enumerate() {
            let (x, y, z) = (p[0].round() as i32, p[1].round() as i32, p[2].round() as i32);
            let stands = (-2..=2).any(|dz| pieds.binary_search(&[x, y, z + dz]).is_ok());
            if !stands {
                println!("lien {n} ({:?} -> {}), puits {:?} : point {i}/{} en {x},{y},{z}", lien.de, lien.vers, lien.puits, lien.points.len());
                for dy in (-4..=4).rev() {
                    let row: Vec<String> = (-4..=4)
                        .map(|dx| {
                            (-6..=6).rev().find(|dz| icebreaker_caverne::debout(g, x + dx, y + dy, z + dz)).map_or(" . ".into(), |dz| format!("{dz:+2} "))
                        })
                        .collect();
                    println!("    {}", row.join(""));
                }
                break;
            }
        }
    }
}
