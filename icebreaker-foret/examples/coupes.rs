//! Icebreaker: draws a forest without the workshop page, to check its shape:
//! `cargo run --release -p icebreaker-foret --example coupes -- [profil] [graine] [dossier]`
//! writes `<profil>-<graine>-dessus.png` (seen from above without the leaves:
//! trails, clearings, trunks, platforms and bridges lighter when higher,
//! ladders in yellow), `-feuilles.png` (the same under its leaves) and
//! `-cote.png` (seen from the side). `-- stats` counts, for each profile, the
//! forests walked whole over 40 seeds.

use icebreaker_foret::{
    Formes, generer,
    grille::{BOIS, BUISSON, CLAIRIERE, ECHELLE, FEUILLES, PLANCHE, RACINE, RAMBARDE, SENTE, SENTIER},
};
use image::{Rgb, RgbImage};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string("assets/icebreaker/forets/formes.ron").expect("lancer depuis la racine du dépôt");
    let formes = Formes::lire(&text).unwrap();
    if args.get(1).is_some_and(|a| a == "stats") {
        for (nom, profil) in &formes.profils {
            let debut = std::time::Instant::now();
            let forets: Vec<_> = (0..40).map(|g| generer(&profil.forme, g, &|_, _| 0)).collect();
            let moyenne = |f: &dyn Fn(&icebreaker_foret::Foret) -> usize| forets.iter().map(f).sum::<usize>() / 40;
            println!(
                "{nom} : {} sur 40 au premier essai, {} coupée(s) ; en moyenne {} arbres, {} plateformes, {} ponts, {} montées ; {:?} par forêt",
                forets.iter().filter(|f| f.essais == 1).count(),
                forets.iter().filter(|f| !f.praticable).count(),
                moyenne(&|f| f.arbres.len()),
                moyenne(&|f| f.plateformes()),
                moyenne(&|f| f.ponts.len()),
                moyenne(&|f| f.arbres.iter().filter(|t| t.montee).count()),
                debut.elapsed() / 40
            );
        }
        return;
    }
    let graine: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let (nom, forme) = formes.tirer(graine, args.get(1).map(String::as_str));
    let dossier = args.get(3).cloned().unwrap_or_else(|| ".".into());
    let debut = std::time::Instant::now();
    let f = generer(forme, graine, &|_, _| 0);
    println!("{nom}, graine {graine} ({:?}, {} essai(s)) : {}", debut.elapsed(), f.essais, f.resume);
    let g = &f.grille;
    let (min, max) = (g.min, g.max());
    let (large, haut) = (g.taille[0] as u32, g.taille[2] as u32);
    let mut dessus = RgbImage::new(large, g.taille[1] as u32);
    let mut feuilles = RgbImage::new(large, g.taille[1] as u32);
    let mut cote = RgbImage::from_pixel(large, haut, Rgb([18, 22, 28]));
    let teinte = |c: [u8; 3], z: i32| {
        let k = 0.55 + 0.45 * (z - min[2]) as f32 / (max[2] - min[2]) as f32;
        Rgb(c.map(|e| (e as f32 * k) as u8))
    };
    let couleur = |bloc: u8| match bloc {
        BOIS | RACINE => [130, 95, 65],
        FEUILLES => [60, 130, 55],
        PLANCHE => [235, 190, 120],
        RAMBARDE => [170, 125, 80],
        ECHELLE => [255, 225, 60],
        BUISSON => [95, 160, 70],
        icebreaker_foret::grille::LANTERNE => [255, 230, 120],
        icebreaker_foret::grille::LIANE => [70, 150, 60],
        _ => [255, 0, 255],
    };
    for y in min[1]..max[1] {
        for x in min[0]..max[0] {
            let sol = match g.nature(x, y) {
                SENTIER => [190, 160, 105],
                CLAIRIERE => [120, 150, 80],
                SENTE => [160, 150, 100],
                _ => [45, 75, 42],
            };
            let (px, py) = ((x - min[0]) as u32, (max[1] - 1 - y) as u32);
            dessus.put_pixel(px, py, Rgb(sol));
            feuilles.put_pixel(px, py, Rgb(sol));
            for z in g.sol(x, y)..max[2] {
                let bloc = g.bloc(x, y, z);
                if bloc == 0 {
                    continue;
                }
                feuilles.put_pixel(px, py, teinte(couleur(bloc), z));
                if bloc != FEUILLES {
                    dessus.put_pixel(px, py, if bloc == ECHELLE { Rgb(couleur(bloc)) } else { teinte(couleur(bloc), z) });
                    cote.put_pixel(px, (max[2] - 1 - z) as u32, Rgb(couleur(bloc)));
                } else if cote.get_pixel(px, (max[2] - 1 - z) as u32).0 == [18, 22, 28] {
                    cote.put_pixel(px, (max[2] - 1 - z) as u32, Rgb([34, 70, 36]));
                }
            }
        }
    }
    for c in &f.clairieres {
        let rouge = Rgb(if c.genre == icebreaker_foret::Genre::Arrivee { [90, 220, 110] } else { [255, 60, 60] });
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            dessus.put_pixel((c.centre[0] + dx - min[0]) as u32, (max[1] - 1 - c.centre[1] - dy) as u32, rouge);
        }
    }
    dessus.save(format!("{dossier}/{nom}-{graine}-dessus.png")).unwrap();
    feuilles.save(format!("{dossier}/{nom}-{graine}-feuilles.png")).unwrap();
    cote.save(format!("{dossier}/{nom}-{graine}-cote.png")).unwrap();
}
