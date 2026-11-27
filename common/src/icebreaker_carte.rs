//! Icebreaker CA : la carte 3D de l'expédition (un objet du sac). Elle montre en volume ce que le
//! personnage a exploré, qu'on fait tourner à la souris, et les objectifs en rouge. Les réglages sont
//! dans `assets/icebreaker/carte.ron`. Ce module décide (ce qui est découvert, l'image) ; `voxygen` ne
//! fait que brancher (terrain chargé, souris, affichage).

use crate::assets::{AssetExt, Ron};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use vek::{Rgb, Vec2, Vec3};

/// L'objet « carte de l'expédition » : dans le sac, il ouvre la carte.
pub const CARTE: &str = "common.items.icebreaker.carte";

/// Les réglages de la carte (`assets/icebreaker/carte.ron`).
#[derive(Clone, Debug, Deserialize)]
pub struct Reglages {
    /// Côté d'une case de la carte, en blocs.
    pub maille: i32,
    /// Jusqu'où on découvre autour de soi, en blocs parcourus dans l'air (un mur arrête).
    pub rayon: u32,
    /// Secondes entre deux découvertes.
    pub periode: f32,
    /// Cases gardées au plus ; au-delà, la carte ne grandit plus.
    pub max_cases: usize,
    /// Côté de l'image de la carte, en pixels.
    pub taille: u32,
    /// Zoom à l'ouverture, en pixels par bloc, et ses bornes (molette).
    pub zoom: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    /// Inclinaison à l'ouverture, en degrés (90 : vue de dessus), et ses bornes.
    pub tangage: f32,
    pub tangage_min: f32,
    pub tangage_max: f32,
    /// Degrés tournés par pixel de souris.
    pub sensibilite: f32,
    /// Éclaircit les couleurs du terrain (1 : telles quelles).
    pub eclat: f32,
    /// Part de lumière d'une face vue de biais (0 : noire, 1 : pas d'ombre).
    pub ombre: f32,
    /// Assombrit ce qui est loin derrière (0 : pas de brume).
    pub brume: f32,
    /// Couleurs des repères : objectif à remplir, objectif rempli, moi.
    pub couleur_objectif: (u8, u8, u8),
    pub couleur_fait: (u8, u8, u8),
    pub couleur_moi: (u8, u8, u8),
}

impl Default for Reglages {
    fn default() -> Self {
        Self {
            maille: 2,
            rayon: 24,
            periode: 0.5,
            max_cases: 200_000,
            taille: 512,
            zoom: 3.0,
            zoom_min: 1.0,
            zoom_max: 8.0,
            tangage: 50.0,
            tangage_min: 15.0,
            tangage_max: 90.0,
            sensibilite: 0.4,
            eclat: 1.4,
            ombre: 0.45,
            brume: 0.25,
            couleur_objectif: (230, 40, 40),
            couleur_fait: (120, 120, 120),
            couleur_moi: (255, 220, 120),
        }
    }
}

impl Reglages {
    /// Les réglages du jeu ; un fichier absent ou invalide donne ceux par défaut (avec un message).
    pub fn charger() -> Self {
        match Ron::<Self>::load("icebreaker.carte") {
            Ok(r) => {
                let reglages = r.read().0.clone();
                match reglages.verifier() {
                    Ok(()) => reglages,
                    Err(e) => {
                        tracing::warn!("Icebreaker carte.ron : {e}");
                        Self::default()
                    },
                }
            },
            Err(e) => {
                tracing::warn!("Icebreaker carte.ron illisible : {e}");
                Self::default()
            },
        }
    }

    pub fn verifier(&self) -> Result<(), String> {
        if self.maille < 1 {
            return Err(format!("maille {} : au moins 1 bloc", self.maille));
        }
        if self.rayon < 1 {
            return Err("rayon : au moins 1 bloc".to_string());
        }
        if self.periode <= 0.0 {
            return Err(format!("periode {} : elle doit être positive", self.periode));
        }
        if self.taille < 16 {
            return Err(format!("taille {} : au moins 16 pixels", self.taille));
        }
        if !(self.zoom_min > 0.0 && self.zoom_min <= self.zoom && self.zoom <= self.zoom_max) {
            return Err("zoom : il faut 0 < zoom_min <= zoom <= zoom_max".to_string());
        }
        if !(self.tangage_min <= self.tangage && self.tangage <= self.tangage_max && self.tangage_max <= 90.0)
            || self.tangage_min < 0.0
        {
            return Err("tangage : il faut 0 <= tangage_min <= tangage <= tangage_max <= 90".to_string());
        }
        Ok(())
    }
}

/// Ce qu'il y a à un endroit du monde, vu par la carte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bloc {
    /// On peut voir à travers (air, eau).
    Air,
    /// Un bloc plein, de cette couleur.
    Plein(Rgb<u8>),
    /// Terrain pas chargé : on ne sait pas.
    Inconnu,
}

const VOISINS: [Vec3<i32>; 6] = [
    Vec3::new(1, 0, 0),
    Vec3::new(-1, 0, 0),
    Vec3::new(0, 1, 0),
    Vec3::new(0, -1, 0),
    Vec3::new(0, 0, 1),
    Vec3::new(0, 0, -1),
];

/// Ce qu'un personnage a découvert du lieu : les cases de surface (sol, murs, plafonds) qu'il a vues.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Exploration {
    /// Case (position du monde divisée par la maille) → couleur du bloc, et les faces qui donnent sur
    /// l'air (un bit par direction de `VOISINS`).
    cases: HashMap<Vec3<i32>, (Rgb<u8>, u8)>,
}

impl Exploration {
    pub fn cases(&self) -> impl Iterator<Item = (Vec3<i32>, Rgb<u8>)> + '_ {
        self.cases.iter().map(|(c, (col, _))| (*c, *col))
    }

    pub fn len(&self) -> usize { self.cases.len() }

    pub fn is_empty(&self) -> bool { self.cases.is_empty() }

    /// La case d'une position du monde.
    pub fn case_de(wpos: Vec3<i32>, maille: i32) -> Vec3<i32> { wpos.map(|e| e.div_euclid(maille.max(1))) }

    pub fn connait(&self, wpos: Vec3<i32>, maille: i32) -> bool {
        self.cases.contains_key(&Self::case_de(wpos, maille))
    }

    /// Découvre autour de `depuis` : on avance dans l'air, de `rayon` blocs au plus, et on note les blocs
    /// pleins qu'on touche. Un mur arrête la découverte ; le terrain pas chargé aussi. Rend `true` si la
    /// carte a changé.
    pub fn explorer(&mut self, depuis: Vec3<i32>, bloc: impl Fn(Vec3<i32>) -> Bloc, reglages: &Reglages) -> bool {
        if bloc(depuis) != Bloc::Air {
            return false;
        }
        let mut change = false;
        let mut vus = hashbrown::HashSet::new();
        let mut file = VecDeque::new();
        vus.insert(depuis);
        file.push_back((depuis, 0u32));
        while let Some((pos, pas)) = file.pop_front() {
            for (i, d) in VOISINS.iter().enumerate() {
                let voisin = pos + *d;
                match bloc(voisin) {
                    Bloc::Air => {
                        if pas < reglages.rayon && vus.insert(voisin) {
                            file.push_back((voisin, pas + 1));
                        }
                    },
                    Bloc::Plein(couleur) => {
                        let case = Self::case_de(voisin, reglages.maille);
                        if self.cases.len() < reglages.max_cases || self.cases.contains_key(&case) {
                            // La face touchée regarde vers l'air d'où l'on vient : la direction opposée.
                            let face = 1u8 << (i ^ 1);
                            let (_, faces) = self.cases.entry(case).or_insert((couleur, 0));
                            if *faces & face == 0 {
                                *faces |= face;
                                change = true;
                            }
                        }
                    },
                    Bloc::Inconnu => {},
                }
            }
        }
        change
    }
}

/// Un objectif de l'expédition, montré en rouge sur la carte (le serveur l'envoie à l'arrivée).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objectif {
    /// Où il est dans le monde.
    pub pos: Vec3<i32>,
    /// Ce que c'est : « chief », « boss ».
    pub sorte: String,
    /// Rempli : il n'est plus rouge.
    pub fait: bool,
}

/// D'où l'on regarde la carte : on tourne autour de soi.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vue {
    /// Rotation autour de la verticale, en radians.
    pub lacet: f32,
    /// Inclinaison, en radians (π/2 : vue de dessus).
    pub tangage: f32,
    /// Pixels par bloc.
    pub zoom: f32,
}

/// Ce qu'un cran de molette multiplie.
const CRAN: f32 = 1.15;

impl Vue {
    /// La vue à l'ouverture de la carte.
    pub fn nouvelle(reglages: &Reglages) -> Self {
        Self {
            lacet: 0.0,
            tangage: reglages.tangage.to_radians(),
            zoom: reglages.zoom,
        }
    }

    /// Un mouvement de souris (pixels, y vers le bas) fait tourner la carte ; l'inclinaison est bornée.
    pub fn tourner(&mut self, delta: Vec2<f32>, reglages: &Reglages) {
        let pas = reglages.sensibilite.to_radians();
        self.lacet = (self.lacet + delta.x * pas).rem_euclid(std::f32::consts::TAU);
        self.tangage = (self.tangage + delta.y * pas)
            .clamp(reglages.tangage_min.to_radians(), reglages.tangage_max.to_radians());
    }

    /// Un cran de molette (positif : plus près) ; le zoom est borné.
    pub fn zoomer(&mut self, crans: f32, reglages: &Reglages) {
        self.zoom = (self.zoom * CRAN.powf(crans)).clamp(reglages.zoom_min, reglages.zoom_max);
    }

    /// Où tombe dans l'image un point donné par rapport au centre (en blocs) : (x, y) en pixels, y vers
    /// le bas, et sa profondeur (plus grand : plus loin de l'œil).
    pub fn projeter(&self, p: Vec3<f32>, cote: u32) -> (f32, f32, f32) {
        let (s, c) = self.lacet.sin_cos();
        let (st, ct) = self.tangage.sin_cos();
        let x = p.x * c - p.y * s;
        let y = p.x * s + p.y * c;
        let centre = cote as f32 / 2.0;
        (
            centre + x * self.zoom,
            centre - (y * st + p.z * ct) * self.zoom,
            y * ct - p.z * st,
        )
    }

    /// À quel point une face de normale `n` (du monde) est tournée vers l'œil : 1 de face, 0 de profil,
    /// négatif de dos.
    fn de_face(&self, n: Vec3<f32>) -> f32 {
        let (s, c) = self.lacet.sin_cos();
        let (st, ct) = self.tangage.sin_cos();
        -(n.x * s + n.y * c) * ct + n.z * st
    }
}

/// L'image de la carte : `cote` × `cote` pixels RGBA, fond transparent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub cote: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    /// Le pixel (x, y), y vers le bas ; transparent hors de l'image.
    pub fn pixel(&self, x: i32, y: i32) -> [u8; 4] {
        if x < 0 || y < 0 || x >= self.cote as i32 || y >= self.cote as i32 {
            return [0; 4];
        }
        let i = (y as usize * self.cote as usize + x as usize) * 4;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2], self.rgba[i + 3]]
    }

    fn poser(&mut self, x: i32, y: i32, (r, g, b): (u8, u8, u8)) {
        if x < 0 || y < 0 || x >= self.cote as i32 || y >= self.cote as i32 {
            return;
        }
        let i = (y as usize * self.cote as usize + x as usize) * 4;
        self.rgba[i..i + 4].copy_from_slice(&[r, g, b, 255]);
    }

    /// Un losange plein, bordé de noir.
    fn losange(&mut self, cx: i32, cy: i32, rayon: i32, couleur: (u8, u8, u8)) {
        for dy in -rayon - 1..=rayon + 1 {
            for dx in -rayon - 1..=rayon + 1 {
                match dx.abs() + dy.abs() {
                    d if d <= rayon => self.poser(cx + dx, cy + dy, couleur),
                    d if d == rayon + 1 => self.poser(cx + dx, cy + dy, (0, 0, 0)),
                    _ => {},
                }
            }
        }
    }

    fn trait_(&mut self, de: Vec2<f32>, a: Vec2<f32>, couleur: (u8, u8, u8)) {
        let pas = (a - de).map(f32::abs).reduce_partial_max().ceil().max(1.0);
        for i in 0..=pas as i32 {
            let p = de + (a - de) * (i as f32 / pas);
            self.poser(p.x.round() as i32, p.y.round() as i32, couleur);
        }
    }

    fn triangle(&mut self, [a, b, c]: [Vec2<f32>; 3], couleur: (u8, u8, u8)) {
        let cote_de = |p: Vec2<f32>, q: Vec2<f32>, r: Vec2<f32>| (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x);
        let min = a.map2(b, f32::min).map2(c, f32::min).map(|e| e.floor() as i32);
        let max = a.map2(b, f32::max).map2(c, f32::max).map(|e| e.ceil() as i32);
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let p = Vec2::new(x as f32, y as f32);
                let (u, v, w) = (cote_de(a, b, p), cote_de(b, c, p), cote_de(c, a, p));
                if (u >= 0.0 && v >= 0.0 && w >= 0.0) || (u <= 0.0 && v <= 0.0 && w <= 0.0) {
                    self.poser(x, y, couleur);
                }
            }
        }
    }
}

/// Tailles des repères, en pixels de l'image.
const OBJECTIF_RAYON: i32 = 6;
const MOI_LONGUEUR: f32 = 9.0;
const MOI_LARGEUR: f32 = 5.0;
/// Un objectif hors de l'image reste à cette distance du bord.
const MARGE: f32 = 10.0;

/// Dessine la carte, centrée sur `moi` : le terrain découvert (seules les faces tournées vers l'œil : vu
/// de dessus, un plafond ne cache pas le sol), puis par-dessus les objectifs (rouges, gris une fois
/// remplis ; un objectif hors de l'image reste collé au bord, dans sa direction) et la flèche de « moi »,
/// tournée vers `regard` (direction horizontale du monde).
pub fn dessiner(
    exploration: &Exploration,
    objectifs: &[Objectif],
    moi: Vec3<f32>,
    regard: Vec2<f32>,
    vue: &Vue,
    reglages: &Reglages,
) -> Image {
    let cote = reglages.taille;
    let centre = cote as f32 / 2.0;
    let mut image = Image {
        cote,
        rgba: vec![0; (cote * cote * 4) as usize],
    };

    // Le terrain : un pavé par case, le plus proche de l'œil gagne.
    let mut profondeurs = vec![f32::INFINITY; (cote * cote) as usize];
    let maille = reglages.maille.max(1) as f32;
    let demi = (maille * vue.zoom * 0.75).ceil().max(1.0) as i32;
    let portee = centre / vue.zoom;
    let de_face = VOISINS.map(|n| vue.de_face(n.map(|e| e as f32)));
    for (case, (couleur, faces)) in &exploration.cases {
        let lumiere = (0..6)
            .filter(|i| faces & (1 << i) != 0)
            .map(|i| de_face[i])
            .fold(0.0, f32::max);
        if lumiere <= 0.01 {
            continue;
        }
        let milieu = (case.map(|e| e as f32) + 0.5) * maille - moi;
        let (x, y, profondeur) = vue.projeter(milieu, cote);
        let (x, y) = (x.floor() as i32, y.floor() as i32);
        if x + demi < 0 || y + demi < 0 || x - demi >= cote as i32 || y - demi >= cote as i32 {
            continue;
        }
        let eclairage = (reglages.ombre + (1.0 - reglages.ombre) * lumiere)
            * (1.0 - reglages.brume * (profondeur / portee).clamp(-1.0, 1.0))
            * reglages.eclat;
        let teinte = couleur.map(|c| (c as f32 * eclairage).clamp(0.0, 255.0) as u8);
        for py in (y - demi).max(0)..(y + demi).min(cote as i32) {
            for px in (x - demi).max(0)..(x + demi).min(cote as i32) {
                let i = (py as u32 * cote + px as u32) as usize;
                if profondeur < profondeurs[i] {
                    profondeurs[i] = profondeur;
                    image.poser(px, py, (teinte.r, teinte.g, teinte.b));
                }
            }
        }
    }

    // Les objectifs, toujours visibles ; un trait les relie à ma hauteur pour lire la profondeur.
    for objectif in objectifs {
        let couleur = if objectif.fait { reglages.couleur_fait } else { reglages.couleur_objectif };
        let rel = objectif.pos.map(|e| e as f32 + 0.5) - moi;
        let (x, y, _) = vue.projeter(rel, cote);
        let ecart = Vec2::new(x, y) - centre;
        let loin = ecart.map(f32::abs).reduce_partial_max();
        let limite = centre - MARGE;
        let ici = if loin > limite {
            ecart * (limite / loin) + centre
        } else {
            let (bx, by, _) = vue.projeter(Vec3::new(rel.x, rel.y, 0.0), cote);
            image.trait_(Vec2::new(bx, by), Vec2::new(x, y), couleur);
            Vec2::new(x, y)
        };
        image.losange(ici.x.round() as i32, ici.y.round() as i32, OBJECTIF_RAYON, couleur);
    }

    // Moi : une flèche au centre, tournée où je regarde.
    let (rx, ry, _) = vue.projeter(Vec3::new(regard.x, regard.y, 0.0), cote);
    let sens = (Vec2::new(rx, ry) - centre).try_normalized().unwrap_or(-Vec2::unit_y());
    let travers = Vec2::new(-sens.y, sens.x);
    let ici = Vec2::broadcast(centre);
    let base = ici - sens * (MOI_LONGUEUR / 3.0);
    image.triangle(
        [ici + sens * MOI_LONGUEUR, base + travers * MOI_LARGEUR, base - travers * MOI_LARGEUR],
        reglages.couleur_moi,
    );
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIERRE: Rgb<u8> = Rgb::new(90, 90, 90);

    fn reglages() -> Reglages {
        Reglages {
            maille: 1,
            rayon: 40,
            ..Reglages::default()
        }
    }

    /// Deux salles de 5 de côté creusées dans la pierre, séparées par un mur en x = 5 ; `porte` y perce
    /// un trou.
    fn deux_salles(porte: bool) -> impl Fn(Vec3<i32>) -> Bloc {
        move |p| {
            let dans = |x0: i32| (x0..x0 + 5).contains(&p.x) && (0..5).contains(&p.y) && (0..5).contains(&p.z);
            if dans(0) || dans(6) || (porte && p == Vec3::new(5, 2, 0)) {
                Bloc::Air
            } else {
                Bloc::Plein(PIERRE)
            }
        }
    }

    fn au_dela_du_mur(e: &Exploration) -> usize { e.cases().filter(|(c, _)| c.x > 5).count() }

    #[test]
    fn un_mur_cache_la_salle_voisine() {
        // CA (Flo, 2026-11-27) : la carte montre « ce que mon personnage a exploré ».
        let mut e = Exploration::default();
        assert!(e.explorer(Vec3::new(2, 2, 2), deux_salles(false), &reglages()));
        assert!(e.connait(Vec3::new(2, 2, -1), 1), "le sol de ma salle");
        assert!(e.connait(Vec3::new(5, 2, 2), 1), "le mur, vu de mon côté");
        assert_eq!(au_dela_du_mur(&e), 0, "rien de la salle d'à côté");
    }

    #[test]
    fn une_porte_ouverte_montre_la_salle_voisine() {
        let mut e = Exploration::default();
        e.explorer(Vec3::new(2, 2, 2), deux_salles(true), &reglages());
        assert!(e.connait(Vec3::new(8, 2, -1), 1), "le sol de l'autre salle");
    }

    #[test]
    fn on_ne_decouvre_pas_plus_loin_que_le_rayon() {
        // Un couloir droit de 200 blocs ; rayon 10.
        let couloir = |p: Vec3<i32>| {
            if (0..200).contains(&p.x) && p.y == 0 && p.z == 0 {
                Bloc::Air
            } else {
                Bloc::Plein(PIERRE)
            }
        };
        let r = Reglages { rayon: 10, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(50, 0, 0), couloir, &r);
        assert!(e.connait(Vec3::new(58, 0, -1), 1));
        assert!(!e.connait(Vec3::new(62, 0, -1), 1));
        assert!(!e.connait(Vec3::new(38, 0, -1), 1));
    }

    #[test]
    fn la_carte_ne_retrecit_jamais() {
        // On garde ce qu'on a vu en s'éloignant.
        let monde = deux_salles(true);
        let r = Reglages { rayon: 4, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(1, 2, 2), &monde, &r);
        let avant: Vec<_> = e.cases().collect();
        assert!(!avant.is_empty());
        assert!(e.explorer(Vec3::new(9, 2, 2), &monde, &r));
        for (case, _) in avant {
            assert!(e.connait(case, 1));
        }
        // Rien de nouveau au même endroit.
        assert!(!e.explorer(Vec3::new(9, 2, 2), &monde, &r));
    }

    #[test]
    fn meme_endroit_meme_carte() {
        let mut a = Exploration::default();
        let mut b = Exploration::default();
        a.explorer(Vec3::new(2, 2, 2), deux_salles(true), &reglages());
        b.explorer(Vec3::new(2, 2, 2), deux_salles(true), &reglages());
        assert!(!a.is_empty());
        assert_eq!(a, b);
    }

    #[test]
    fn le_plafond_de_cases_est_tenu() {
        let r = Reglages { max_cases: 20, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(2, 2, 2), deux_salles(true), &r);
        assert_eq!(e.len(), 20);
    }

    #[test]
    fn le_terrain_pas_charge_arrete_la_decouverte() {
        // Tout ce qui est en x >= 3 n'est pas chargé : on ne le traverse pas et on ne le dessine pas.
        let monde = deux_salles(true);
        let coupe = |p: Vec3<i32>| if p.x >= 3 { Bloc::Inconnu } else { monde(p) };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(1, 2, 2), coupe, &reglages());
        assert!(!e.is_empty());
        assert_eq!(e.cases().filter(|(c, _)| c.x >= 3).count(), 0);
    }

    #[test]
    fn une_case_regroupe_les_blocs_de_la_maille() {
        let r = Reglages { maille: 2, ..reglages() };
        let mut e = Exploration::default();
        e.explorer(Vec3::new(2, 2, 2), deux_salles(false), &r);
        // Le sol (z = -1) tombe dans les cases z = -1 ; 5 blocs de large → 3 cases.
        assert_eq!(Exploration::case_de(Vec3::new(3, 3, -1), 2), Vec3::new(1, 1, -1));
        let sol = e.cases().filter(|(c, _)| c.z == -1).count();
        assert_eq!(sol, 9);
        assert_eq!(e.cases().next().map(|(_, col)| col), Some(PIERRE));
    }

    #[test]
    fn depuis_un_bloc_plein_on_ne_decouvre_rien() {
        let mut e = Exploration::default();
        assert!(!e.explorer(Vec3::new(5, 0, 0), deux_salles(false), &reglages()));
        assert!(e.is_empty());
    }

    #[test]
    fn fichier_de_reglages_lisible() {
        let r = Ron::<Reglages>::load_expect("icebreaker.carte").read().0.clone();
        assert!(r.verifier().is_ok());
        assert!(Reglages { maille: 0, ..Reglages::default() }.verifier().is_err());
    }

    // --- CA.2 : le dessin ---

    const ROUGE: Rgb<u8> = Rgb::new(200, 0, 0);
    const BLEU: Rgb<u8> = Rgb::new(0, 0, 200);
    const MOI: Vec3<f32> = Vec3::new(0.5, 0.5, 1.0);

    fn reglages_dessin() -> Reglages {
        Reglages {
            maille: 1,
            rayon: 8,
            taille: 128,
            ..Reglages::default()
        }
    }

    fn de_dessus() -> Vue {
        Vue {
            lacet: 0.0,
            tangage: std::f32::consts::FRAC_PI_2,
            zoom: 4.0,
        }
    }

    fn explore(monde: impl Fn(Vec3<i32>) -> Bloc) -> Exploration {
        let mut e = Exploration::default();
        e.explorer(Vec3::new(0, 0, 1), monde, &reglages_dessin());
        assert!(!e.is_empty());
        e
    }

    fn image(e: &Exploration, objectifs: &[Objectif], vue: &Vue) -> Image {
        dessiner(e, objectifs, MOI, Vec2::unit_y(), vue, &reglages_dessin())
    }

    /// Le pixel où tombe un point du monde.
    fn pixel_de(img: &Image, vue: &Vue, wpos: Vec3<f32>) -> [u8; 4] {
        let (x, y, _) = vue.projeter(wpos - MOI, img.cote);
        img.pixel(x as i32, y as i32)
    }

    fn pixels(img: &Image) -> impl Iterator<Item = (i32, i32, [u8; 4])> + '_ {
        let cote = img.cote as i32;
        (0..cote * cote).map(move |i| (i % cote, i / cote, img.pixel(i % cote, i / cote)))
    }

    fn compte(img: &Image, (r, g, b): (u8, u8, u8)) -> usize {
        pixels(img).filter(|(_, _, p)| *p == [r, g, b, 255]).count()
    }

    fn objectif(pos: Vec3<i32>, fait: bool) -> Objectif {
        Objectif {
            pos,
            sorte: "chief".to_string(),
            fait,
        }
    }

    #[test]
    fn vue_de_dessus_le_nord_est_en_haut() {
        let vue = de_dessus();
        let (cx, cy, _) = vue.projeter(Vec3::zero(), 128);
        assert_eq!((cx, cy), (64.0, 64.0));
        let (x, y, _) = vue.projeter(Vec3::new(5.0, 0.0, 0.0), 128);
        assert!(x > cx && (y - cy).abs() < 0.01, "l'est à droite");
        let (x, y, _) = vue.projeter(Vec3::new(0.0, 5.0, 0.0), 128);
        assert!(y < cy && (x - cx).abs() < 0.01, "le nord en haut");
        let (_, _, haut) = vue.projeter(Vec3::new(0.0, 0.0, 5.0), 128);
        let (_, _, bas) = vue.projeter(Vec3::new(0.0, 0.0, -5.0), 128);
        assert!(haut < bas, "vu de dessus, ce qui est haut est plus près de l'œil");
    }

    #[test]
    fn ce_qui_est_devant_cache_ce_qui_est_derriere() {
        // Deux blocs l'un au-dessus de l'autre, vus de dessus : on voit celui du haut.
        let e = explore(|p| match (p.x, p.y, p.z) {
            (4, 0, 0) => Bloc::Plein(ROUGE),
            (4, 0, 3) => Bloc::Plein(BLEU),
            _ => Bloc::Air,
        });
        let vue = de_dessus();
        let p = pixel_de(&image(&e, &[], &vue), &vue, Vec3::new(4.5, 0.5, 0.0));
        assert_eq!(p[3], 255);
        assert!(p[2] > p[0], "le bloc bleu, en haut, est devant : {p:?}");
    }

    #[test]
    fn vu_de_dessus_le_plafond_ne_cache_pas_le_sol() {
        // Une salle : sol rouge, plafond bleu. On regarde dedans, pas le dos du plafond.
        let e = explore(|p| match p.z {
            0 => Bloc::Plein(ROUGE),
            3 => Bloc::Plein(BLEU),
            1 | 2 => Bloc::Air,
            _ => Bloc::Inconnu,
        });
        let vue = de_dessus();
        let p = pixel_de(&image(&e, &[], &vue), &vue, Vec3::new(4.5, 0.5, 0.0));
        assert_eq!(p[3], 255);
        assert!(p[0] > p[2], "le sol rouge : {p:?}");
    }

    #[test]
    fn un_objectif_est_rouge_meme_sans_terrain() {
        // CA (Flo, 2026-11-27) : les objectifs rouges sont visibles dès l'arrivée, même dans le vide.
        let r = reglages_dessin();
        let img = image(&Exploration::default(), &[objectif(Vec3::new(10, 0, 1), false)], &de_dessus());
        assert!(compte(&img, r.couleur_objectif) > 0);
        assert_eq!(compte(&img, r.couleur_fait), 0);
    }

    #[test]
    fn un_objectif_rempli_n_est_plus_rouge() {
        let r = reglages_dessin();
        let img = image(&Exploration::default(), &[objectif(Vec3::new(10, 0, 1), true)], &de_dessus());
        assert_eq!(compte(&img, r.couleur_objectif), 0);
        assert!(compte(&img, r.couleur_fait) > 0);
    }

    #[test]
    fn un_objectif_lointain_reste_au_bord_de_l_image() {
        // Il donne la direction : à l'est, donc sur le bord droit.
        let r = reglages_dessin();
        let img = image(&Exploration::default(), &[objectif(Vec3::new(5000, 0, 1), false)], &de_dessus());
        let rouges: Vec<_> = pixels(&img)
            .filter(|(_, _, p)| *p == [r.couleur_objectif.0, r.couleur_objectif.1, r.couleur_objectif.2, 255])
            .collect();
        assert!(!rouges.is_empty());
        assert!(rouges.iter().all(|(x, _, _)| *x > 96), "sur le bord droit");
    }

    #[test]
    fn carte_vide_seul_moi_est_dessine() {
        // CA : seul autre repère, « moi ».
        let (r, g, b) = reglages_dessin().couleur_moi;
        let img = image(&Exploration::default(), &[], &de_dessus());
        let dessines: Vec<_> = pixels(&img).filter(|(_, _, p)| p[3] != 0).collect();
        assert!(!dessines.is_empty());
        assert!(dessines.iter().all(|(_, _, p)| *p == [r, g, b, 255]));
        assert_eq!(img.pixel(64, 64), [r, g, b, 255], "au centre de l'image");
    }

    #[test]
    fn la_fleche_pointe_ou_je_regarde() {
        let r = reglages_dessin();
        let (cr, cg, cb) = r.couleur_moi;
        let etendue = |regard: Vec2<f32>| {
            let img = dessiner(&Exploration::default(), &[], MOI, regard, &de_dessus(), &r);
            let xs: Vec<i32> = pixels(&img)
                .filter(|(_, _, p)| *p == [cr, cg, cb, 255])
                .map(|(x, _, _)| x)
                .collect();
            (64 - xs.iter().min().unwrap(), xs.iter().max().unwrap() - 64)
        };
        let (gauche, droite) = etendue(Vec2::unit_x());
        assert!(droite > gauche, "je regarde à l'est : la pointe est à droite");
        let (gauche, droite) = etendue(-Vec2::unit_x());
        assert!(gauche > droite, "je regarde à l'ouest : la pointe est à gauche");
    }

    #[test]
    fn un_demi_tour_inverse_gauche_et_droite() {
        let e = explore(|p| if p == Vec3::new(6, 0, 0) { Bloc::Plein(BLEU) } else { Bloc::Air });
        let milieu_des_bleus = |lacet: f32| {
            let vue = Vue { lacet, ..de_dessus() };
            let img = image(&e, &[], &vue);
            let xs: Vec<i32> = pixels(&img)
                .filter(|(_, _, p)| p[3] == 255 && p[2] > p[0])
                .map(|(x, _, _)| x)
                .collect();
            assert!(!xs.is_empty());
            xs.iter().sum::<i32>() as f32 / xs.len() as f32
        };
        assert!(milieu_des_bleus(0.0) > 70.0);
        assert!(milieu_des_bleus(std::f32::consts::PI) < 58.0);
    }

    #[test]
    fn la_vue_est_bornee() {
        let r = reglages_dessin();
        let mut vue = Vue::nouvelle(&r);
        assert_eq!(vue.zoom, r.zoom);
        vue.tourner(Vec2::new(0.0, 100_000.0), &r);
        assert!((vue.tangage - r.tangage_max.to_radians()).abs() < 1e-4);
        vue.tourner(Vec2::new(0.0, -100_000.0), &r);
        assert!((vue.tangage - r.tangage_min.to_radians()).abs() < 1e-4);
        let lacet = vue.lacet;
        vue.tourner(Vec2::new(50.0, 0.0), &r);
        assert!(vue.lacet != lacet);
        vue.zoomer(100.0, &r);
        assert_eq!(vue.zoom, r.zoom_max);
        vue.zoomer(-100.0, &r);
        assert_eq!(vue.zoom, r.zoom_min);
    }
}
