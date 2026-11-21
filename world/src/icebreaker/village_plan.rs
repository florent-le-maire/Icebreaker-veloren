//! Icebreaker VF : le plan du village fixe (décision 0022), lu dans
//! `assets/icebreaker/village_plan.ron`. Tout y est en tuiles de site (6 blocs),
//! relatif au centre du village, x vers l'est, y vers le nord. Le site est
//! construit d'après ce plan par `Site::generate_icebreaker_village`.

use serde::Deserialize;
use vek::{Aabr, Vec2};

/// Un rectangle de tuiles : de `min` à `min + taille - 1`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub min: (i32, i32),
    pub taille: (i32, i32),
}

impl Rect {
    /// Ses tuiles, `max` exclu (comme les `Aabr` de tuiles de Veloren).
    pub fn aabr(&self) -> Aabr<i32> {
        let min = Vec2::from(self.min);
        Aabr {
            min,
            max: min + Vec2::from(self.taille),
        }
    }

    fn chevauche(&self, autre: &Rect) -> bool {
        let (a, b) = (self.aabr(), autre.aabr());
        a.min.x < b.max.x && b.min.x < a.max.x && a.min.y < b.max.y && b.min.y < a.max.y
    }
}

/// La forme de Veloren d'un lieu.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum Forme {
    Maison,
    Atelier,
    Taverne,
    /// Un terrain plat, rien dessus (champ, cimetière, emplacement…).
    Terrain,
}

impl Forme {
    /// Le plus grand côté (en tuiles) d'un bâtiment de cette forme.
    fn cote_max(self) -> i32 {
        match self {
            Forme::Maison | Forme::Atelier => 4,
            Forme::Taverne => 7,
            Forme::Terrain => i32::MAX,
        }
    }
}

/// Le côté d'un lieu où est sa porte.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum Cote {
    Nord,
    Sud,
    Est,
    Ouest,
}

impl Cote {
    /// La direction vers l'extérieur, de ce côté.
    pub fn dir(self) -> Vec2<i32> {
        match self {
            Cote::Nord => Vec2::unit_y(),
            Cote::Sud => -Vec2::unit_y(),
            Cote::Est => Vec2::unit_x(),
            Cote::Ouest => -Vec2::unit_x(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Lieu {
    /// L'id d'un bâtiment du jeu (`charrette`, `entrepot`…) ou un nom de décor.
    pub id: String,
    pub forme: Forme,
    pub min: (i32, i32),
    pub taille: (i32, i32),
    pub porte: Cote,
    /// 0 : bâti au départ ; sinon le rang du terrain à acheter (jalon C).
    pub palier: u32,
    /// La taille du bâtiment, posé contre la porte et centré ; tout le
    /// rectangle (dans la limite de la forme) si absent.
    #[serde(default)]
    pub bati: Option<(i32, i32)>,
}

impl Lieu {
    pub fn rect(&self) -> Rect {
        Rect {
            min: self.min,
            taille: self.taille,
        }
    }

    /// Les tuiles du bâtiment (`max` exclu) : contre le côté de la porte,
    /// centré le long de ce côté. Tout le rectangle pour un terrain.
    pub fn bati_aabr(&self) -> Aabr<i32> {
        let parcelle = self.rect().aabr();
        if self.forme == Forme::Terrain {
            return parcelle;
        }
        let taille = Vec2::from(self.bati.unwrap_or(self.taille))
            .map2(Vec2::from(self.taille), |b: i32, t: i32| b.min(t))
            .map(|b| b.clamp(1, self.forme.cote_max()));
        // Centré dans la parcelle, puis poussé contre la porte.
        let mut min = parcelle.min + (Vec2::from(self.taille) - taille) / 2;
        match self.porte {
            Cote::Nord => min.y = parcelle.max.y - taille.y,
            Cote::Sud => min.y = parcelle.min.y,
            Cote::Est => min.x = parcelle.max.x - taille.x,
            Cote::Ouest => min.x = parcelle.min.x,
        }
        Aabr {
            min,
            max: min + taille,
        }
    }

    /// La tuile de la porte : dans le bâtiment, au milieu de son côté porte.
    pub fn porte_tuile(&self) -> Vec2<i32> {
        let bati = self.bati_aabr();
        let centre = bati.min + (bati.max - bati.min) / 2;
        match self.porte {
            Cote::Nord => Vec2::new(centre.x, bati.max.y - 1),
            Cote::Sud => Vec2::new(centre.x, bati.min.y),
            Cote::Est => Vec2::new(bati.max.x - 1, centre.y),
            Cote::Ouest => Vec2::new(bati.min.x, centre.y),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct VillagePlan {
    /// Tuiles de raccord en pente douce autour du village.
    pub marge: i32,
    pub place: Rect,
    pub chemins: Vec<Rect>,
    pub lieux: Vec<Lieu>,
}

impl VillagePlan {
    pub fn load() -> Result<Self, String> {
        let path = common::assets::ASSETS_PATH.join("icebreaker/village_plan.ron");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let plan: Self = ron::from_str(text).map_err(|e| format!("village_plan.ron illisible : {e}"))?;
        let mut errors = Vec::new();
        let rects = std::iter::once(("place".to_string(), plan.place))
            .chain(plan.chemins.iter().enumerate().map(|(i, r)| (format!("chemin {i}"), *r)))
            .chain(plan.lieux.iter().map(|l| (l.id.clone(), l.rect())))
            .collect::<Vec<_>>();
        for (i, (nom, rect)) in rects.iter().enumerate() {
            if rect.taille.0 < 1 || rect.taille.1 < 1 {
                errors.push(format!("{nom} : taille vide"));
            }
            for (autre, r) in &rects[i + 1..] {
                if rect.chevauche(r) {
                    errors.push(format!("{nom} chevauche {autre}"));
                }
            }
        }
        for (i, lieu) in plan.lieux.iter().enumerate() {
            if plan.lieux[..i].iter().any(|l| l.id == lieu.id) {
                errors.push(format!("{} : id en double", lieu.id));
            }
        }
        if plan.marge < 0 {
            errors.push("marge négative".to_string());
        }
        if errors.is_empty() { Ok(plan) } else { Err(errors.join(" ; ")) }
    }

    /// Les tuiles qu'occupe le village (`max` exclu), sans la marge.
    pub fn emprise(&self) -> Aabr<i32> {
        std::iter::once(self.place)
            .chain(self.chemins.iter().copied())
            .chain(self.lieux.iter().map(|l| l.rect()))
            .map(|r| r.aabr())
            .reduce(|a, b| a.union(b))
            .unwrap_or(self.place.aabr())
    }

    pub fn lieu(&self, id: &str) -> Option<&Lieu> { self.lieux.iter().find(|l| l.id == id) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_du_depot() -> VillagePlan {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/icebreaker/village_plan.ron");
        VillagePlan::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn le_plan_du_depot_se_lit_sans_chevauchement() {
        let plan = plan_du_depot();
        for id in ["charrette", "intendance", "entrepot", "depart", "salle_armes", "forge"] {
            assert!(plan.lieu(id).is_some(), "{id} manque au plan");
        }
        assert!(plan.lieu("infirmerie").is_none());
        // Le centre du village est sur la place.
        let place = plan.place.aabr();
        assert!(place.min.x <= 0 && 0 < place.max.x && place.min.y <= 0 && 0 < place.max.y);
    }

    #[test]
    fn chaque_batiment_tient_dans_sa_parcelle_et_sa_porte_aussi() {
        let plan = plan_du_depot();
        for lieu in &plan.lieux {
            let parcelle = lieu.rect().aabr();
            let bati = lieu.bati_aabr();
            assert!(
                parcelle.contains_aabr(bati),
                "{} : bâtiment {bati:?} hors de {parcelle:?}",
                lieu.id
            );
            let porte = lieu.porte_tuile();
            assert!(bati.min.x <= porte.x && porte.x < bati.max.x && bati.min.y <= porte.y && porte.y < bati.max.y);
        }
    }

    #[test]
    fn un_chevauchement_est_refuse() {
        let texte = r#"(marge: 2, place: (min: (0, 0), taille: (4, 4)), chemins: [(min: (3, 3), taille: (2, 2))], lieux: [])"#;
        let err = VillagePlan::parse(texte).unwrap_err();
        assert!(err.contains("place chevauche chemin 0"), "{err}");
    }

    #[test]
    fn le_bati_est_pousse_contre_la_porte() {
        let lieu = Lieu {
            id: "x".into(),
            forme: Forme::Maison,
            min: (0, 0),
            taille: (6, 6),
            porte: Cote::Nord,
            palier: 0,
            bati: Some((2, 2)),
        };
        assert_eq!(lieu.bati_aabr(), Aabr { min: Vec2::new(2, 4), max: Vec2::new(4, 6) });
        assert_eq!(lieu.porte_tuile(), Vec2::new(3, 5));
    }
}
