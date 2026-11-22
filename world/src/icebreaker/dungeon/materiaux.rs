//! Icebreaker F.27 : ce dont sont faits le sol, les murs et le plafond des salles du donjon
//! (`assets/icebreaker/donjons/materiaux.ron`). Un matériau est un motif (dalles, rangs de pierre, planches,
//! poutres…) et quelques couleurs ; il donne le bloc de chaque position, au bloc près. Une salle choisit les siens
//! (`@sol:`, `@murs:`, `@plafond:`), sinon le thème du donjon décide (`par_theme`), sinon `defaut`.

use common::terrain::{Block, BlockKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vek::{Rgb, Vec2, Vec3};

/// Le dessin d'un matériau. Les longueurs sont en blocs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Motif {
    /// Une teinte par bloc, prise au hasard dans les couleurs (terre battue, roche brute, enduit).
    Uni,
    /// Des dalles carrées de `cote` blocs, une couleur par dalle, le bord un peu plus sombre (carrelage, dallage).
    Dalles { cote: i32 },
    /// Des pierres de `longueur` blocs sur `hauteur` rangs, décalées d'un rang à l'autre, une couleur par pierre et
    /// un joint sombre entre deux (pierre taillée sur un mur, pavés au sol).
    Rangs { longueur: i32, hauteur: i32 },
    /// Des planches de `largeur` blocs le long de x, aboutées tous les `longueur` blocs (plancher, lambris).
    Planches { largeur: i32, longueur: i32 },
    /// Une poutre de bois de la couleur `bois` tous les `ecart` blocs le long de x, le reste dans les couleurs
    /// (plafond à poutres).
    Poutres { ecart: i32, bois: (u8, u8, u8) },
    /// Icebreaker F.36 : un carré de `cote` blocs (7 = une case) en marbre (couleurs 0 et 1, une teinte par bloc),
    /// un cadre sombre (couleur 2) de `bord` blocs en retrait d'un bloc ; une case sur deux a le cœur du cadre
    /// plein (couleur 3). Compté depuis le coin de la case (`bloc_case`), pour tomber juste sur les cases.
    Cadres { cote: i32, bord: i32 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Materiau {
    /// Le nom affiché dans l'atelier des salles.
    pub nom: String,
    pub motif: Motif,
    pub couleurs: Vec<(u8, u8, u8)>,
    /// Bloc de bois (planches) plutôt que de roche.
    #[serde(default)]
    pub bois: bool,
    /// Un mur : la couleur du rang du bas, posé sur le sol (plinthe), et celle du rang du haut (corniche).
    #[serde(default)]
    pub plinthe: Option<(u8, u8, u8)>,
    #[serde(default)]
    pub corniche: Option<(u8, u8, u8)>,
}

/// Les trois matériaux d'une salle (des noms de `materiaux.ron`).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Habillage {
    pub sol: String,
    pub murs: String,
    pub plafond: String,
}

/// Ce qu'une salle demande : chaque partie peut manquer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HabillageSalle {
    pub sol: Option<String>,
    pub murs: Option<String>,
    pub plafond: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Materiaux {
    pub sols: BTreeMap<String, Materiau>,
    pub murs: BTreeMap<String, Materiau>,
    pub plafonds: BTreeMap<String, Materiau>,
    /// L'habillage par thème de donjon (`@theme:` des salles, thème dominant du donjon).
    pub par_theme: BTreeMap<String, Habillage>,
    pub defaut: Habillage,
}

/// Un nombre fixe par position.
fn bruit(x: i32, y: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ (z as u32).wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 13)
}

fn assombrir((r, g, b): (u8, u8, u8), f: f32) -> (u8, u8, u8) {
    let m = |c: u8| (c as f32 * f).clamp(0.0, 255.0) as u8;
    (m(r), m(g), m(b))
}

impl Materiau {
    fn couleur(&self, n: u32) -> (u8, u8, u8) {
        match self.couleurs.len() {
            0 => (128, 128, 128),
            l => self.couleurs[n as usize % l],
        }
    }

    /// Le bloc en `pos` (monde). `hauteur` : la hauteur du bloc au-dessus du sol de son étage (0 = la couche du
    /// sol), pour les rangs des murs, la plinthe et la corniche ; `etage` : les blocs d'un étage.
    pub fn bloc(&self, pos: Vec3<i32>, hauteur: i32, etage: i32) -> Block { self.bloc_case(pos, Vec2::zero(), hauteur, etage) }

    /// Icebreaker F.36 : le même, avec le coin de la case où tombe `pos` (le motif `Cadres` se compte depuis là).
    pub fn bloc_case(&self, pos: Vec3<i32>, coin: Vec2<i32>, hauteur: i32, etage: i32) -> Block {
        let (x, y, z) = (pos.x, pos.y, pos.z);
        let mut kind = if self.bois { BlockKind::Wood } else { BlockKind::Rock };
        let couleur = match &self.motif {
            Motif::Uni => self.couleur(bruit(x, y, z)),
            Motif::Dalles { cote } => {
                let c = (*cote).max(1);
                let (dx, dy) = (x.div_euclid(c), y.div_euclid(c));
                let base = self.couleur(bruit(dx, dy, 11));
                let bord = x.rem_euclid(c) == 0 || y.rem_euclid(c) == 0;
                if bord && c > 1 { assombrir(base, 0.86) } else { base }
            },
            Motif::Rangs { longueur, hauteur: haut } => {
                let (l, h) = ((*longueur).max(1), (*haut).max(1));
                // Sur un mur, le long du mur = x + y (le motif suit les deux faces) et les rangs montent en z ; au
                // sol et au plafond, les rangs vont le long de y.
                let (le_long, rang) = if hauteur > 0 && hauteur < etage { (x + y, z.div_euclid(h)) } else { (x, y.div_euclid(h)) };
                let decale = le_long + if rang % 2 == 0 { 0 } else { l / 2 };
                let base = self.couleur(bruit(decale.div_euclid(l), rang, 23));
                if decale.rem_euclid(l) == 0 && l > 1 { assombrir(base, 0.8) } else { base }
            },
            Motif::Planches { largeur, longueur } => {
                let (w, l) = ((*largeur).max(1), (*longueur).max(2));
                let planche = y.div_euclid(w);
                let decale = x + planche * 3;
                let base = self.couleur(bruit(planche, decale.div_euclid(l), 31));
                if decale.rem_euclid(l) == 0 { assombrir(base, 0.75) } else { base }
            },
            Motif::Poutres { ecart, bois } => {
                if x.rem_euclid((*ecart).max(2)) == 0 {
                    kind = BlockKind::Wood;
                    *bois
                } else {
                    self.couleur(bruit(x, y, z))
                }
            },
            Motif::Cadres { cote, bord } => {
                let (c, b) = ((*cote).max(3), (*bord).max(1));
                let (lx, ly) = (x - coin.x, y - coin.y);
                let (ix, iy) = (lx.rem_euclid(c), ly.rem_euclid(c));
                // Distance au bord du carré : 0 = le marbre autour, 1..=b = le cadre, au-delà = le cœur.
                let d = ix.min(iy).min(c - 1 - ix).min(c - 1 - iy);
                let plein = (lx.div_euclid(c) + ly.div_euclid(c)).rem_euclid(2) == 0;
                if d >= 1 && d <= b {
                    self.couleur(2)
                } else if d > b && plein {
                    self.couleur(3)
                } else {
                    self.couleur(bruit(x, y, z) % 2)
                }
            },
        };
        // La plinthe et la corniche d'un mur.
        let couleur = match (self.plinthe, self.corniche) {
            (Some(p), _) if hauteur == 1 => p,
            (_, Some(c)) if hauteur == etage - 1 => c,
            _ => couleur,
        };
        // Un peu de grain, bloc par bloc.
        let grain = 0.94 + (bruit(x, y, z ^ 0x55) % 13) as f32 / 100.0;
        let (r, g, b) = assombrir(couleur, grain);
        Block::new(kind, Rgb::new(r, g, b))
    }
}

impl Materiaux {
    pub fn load() -> Result<Self, String> { Self::parse(&super::read_data("materiaux.ron")?) }

    pub fn parse(text: &str) -> Result<Self, String> {
        let m: Self = ron::from_str(text).map_err(|e| format!("materiaux.ron : {e}"))?;
        let mut erreurs = Vec::new();
        for (theme, h) in m.par_theme.iter().map(|(t, h)| (t.as_str(), h)).chain([("defaut", &m.defaut)]) {
            erreurs.extend(m.erreurs(&HabillageSalle {
                sol: Some(h.sol.clone()),
                murs: Some(h.murs.clone()),
                plafond: Some(h.plafond.clone()),
            }).into_iter().map(|e| format!("{theme} : {e}")));
        }
        if erreurs.is_empty() { Ok(m) } else { Err(format!("materiaux.ron : {}", erreurs.join(" ; "))) }
    }

    /// Les noms qu'une salle demande et que `materiaux.ron` n'a pas.
    pub fn erreurs(&self, h: &HabillageSalle) -> Vec<String> {
        let mut e = Vec::new();
        for (nom, liste, partie) in [(&h.sol, &self.sols, "sol"), (&h.murs, &self.murs, "murs"), (&h.plafond, &self.plafonds, "plafond")] {
            if let Some(nom) = nom
                && !liste.contains_key(nom)
            {
                e.push(format!(
                    "{partie} inconnu « {nom} » (connus : {})",
                    liste.keys().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }
        e
    }

    /// L'habillage d'une salle : ce qu'elle demande, sinon celui du thème, sinon le défaut.
    pub fn habillage(&self, salle: &HabillageSalle, theme: Option<&str>) -> Habillage {
        let base = theme.and_then(|t| self.par_theme.get(t)).unwrap_or(&self.defaut);
        let choisir = |demande: &Option<String>, liste: &BTreeMap<String, Materiau>, sinon: &String| {
            demande.clone().filter(|n| liste.contains_key(n)).unwrap_or_else(|| sinon.clone())
        };
        Habillage {
            sol: choisir(&salle.sol, &self.sols, &base.sol),
            murs: choisir(&salle.murs, &self.murs, &base.murs),
            plafond: choisir(&salle.plafond, &self.plafonds, &base.plafond),
        }
    }

    // Les matériaux d'un habillage (un nom inconnu prend le premier de la liste).
    pub fn sol(&self, h: &Habillage) -> &Materiau { Self::trouver(&self.sols, &h.sol) }

    pub fn mur(&self, h: &Habillage) -> &Materiau { Self::trouver(&self.murs, &h.murs) }

    pub fn plafond(&self, h: &Habillage) -> &Materiau { Self::trouver(&self.plafonds, &h.plafond) }

    fn trouver<'a>(liste: &'a BTreeMap<String, Materiau>, nom: &str) -> &'a Materiau {
        liste.get(nom).or_else(|| liste.values().next()).expect("materiaux.ron a au moins un matériau par partie")
    }
}

/// L'habillage de chaque case du donjon : celui de la salle à qui elle appartient.
#[derive(Clone, Debug)]
pub struct Habillages {
    pub salles: Vec<Habillage>,
    /// Case → salle (index dans `salles`) ; les cases de sol (et portes, escaliers) seulement.
    pub cases: BTreeMap<super::Cell, usize>,
    /// Le tunnel, la roche hors des salles.
    pub defaut: Habillage,
    pub materiaux: std::sync::Arc<Materiaux>,
}

impl Habillages {
    pub fn de(&self, cell: super::Cell) -> Option<&Habillage> { self.cases.get(&cell).map(|i| &self.salles[*i]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materiaux_load() { Materiaux::load().unwrap(); }

    #[test]
    fn la_salle_choisit_sinon_le_theme_sinon_le_defaut() {
        let m = Materiaux::load().unwrap();
        let theme = m.par_theme.keys().next().cloned().unwrap();
        let sol = m.sols.keys().last().cloned().unwrap();
        let h = m.habillage(&HabillageSalle { sol: Some(sol.clone()), ..Default::default() }, Some(&theme));
        assert_eq!(h.sol, sol);
        assert_eq!(h.murs, m.par_theme[&theme].murs);
        assert_eq!(m.habillage(&HabillageSalle::default(), None), m.defaut);
        assert_eq!(m.habillage(&HabillageSalle::default(), Some("inconnu")), m.defaut);
    }

    #[test]
    fn un_nom_inconnu_est_signale() {
        let m = Materiaux::load().unwrap();
        assert_eq!(m.erreurs(&HabillageSalle { murs: Some("guimauve".into()), ..Default::default() }).len(), 1);
    }

    #[test]
    fn les_motifs_varient() {
        let m = Materiaux::load().unwrap();
        for (nom, mat) in m.sols.iter().chain(&m.murs).chain(&m.plafonds) {
            let couleurs: std::collections::BTreeSet<_> = (0..12)
                .flat_map(|x| (0..12).map(move |y| (x, y)))
                .map(|(x, y)| mat.bloc(Vec3::new(x, y, 3), 3, 7).get_color().map(|c| (c.r, c.g, c.b)))
                .collect();
            assert!(couleurs.len() > 3, "{nom} : trop uni");
        }
    }

    /// Icebreaker F.36 : le cadre du marbre tombe sur la case, où qu'elle soit dans le monde.
    #[test]
    fn les_cadres_suivent_la_case() {
        let m = Materiaux::load().unwrap();
        let marbre = &m.sols["marbre"];
        let sombre = |c: Option<Rgb<u8>>| c.map(|c| c.r < 100).unwrap();
        for coin in [Vec2::new(0, 0), Vec2::new(1003, -45)] {
            let teinte = |x, y| marbre.bloc_case(Vec3::new(coin.x + x, coin.y + y, 0), coin, 0, 7).get_color();
            for i in 1..=5 {
                assert!(sombre(teinte(1, i)) && sombre(teinte(i, 5)), "le cadre en ({i}) depuis {coin:?}");
            }
            assert!(!sombre(teinte(0, 3)) && !sombre(teinte(6, 3)) && !sombre(teinte(3, 0)), "le marbre autour");
            assert!(!sombre(teinte(3, 3)), "le cœur");
        }
    }
}
