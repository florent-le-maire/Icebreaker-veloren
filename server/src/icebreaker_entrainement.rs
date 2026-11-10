//! Icebreaker: the training ground of the village (X.14, X.15). A row of
//! Veloren's training dummies on flat ground near the square, to try skills
//! and damage: the first ones stand still when hit, the others keep Veloren's
//! knockback. They never die: after a series of hits, each attacker is told
//! what they did and the dummy gets all its health back. Settings in
//! `assets/icebreaker/entrainement.ron`.

use crate::icebreaker_warehouse::chunk_of;
use common::{
    assets::{AssetExt, Ron},
    comp::{self, Alignment, Body, Content, Inventory},
    event::NpcBuilder,
};
use serde::Deserialize;
use vek::*;

#[derive(Clone, Debug, Deserialize)]
pub struct EntrainementConfig {
    /// Where the flat ground is looked for, from the village square (in blocks).
    pub decalage: (f32, f32),
    /// How far from there (in blocks) flat ground is looked for.
    pub recherche: i32,
    /// Flat ground kept around the row (in blocks).
    pub marge: i32,
    /// Dummies standing still when hit, then dummies with knockback, in a row
    /// towards +x.
    pub immobiles: u32,
    pub mobiles: u32,
    /// The gap between two dummies (in blocks).
    pub ecart: i32,
    /// Seconds without a hit that end a series.
    pub fin_de_serie: f64,
}

impl EntrainementConfig {
    /// Whether each dummy of the row stands still.
    pub fn immobilite(&self) -> Vec<bool> {
        std::iter::repeat_n(true, self.immobiles as usize)
            .chain(std::iter::repeat_n(false, self.mobiles as usize))
            .collect()
    }

    /// The area of the row (in blocks).
    pub fn emprise(&self) -> Vec2<i32> {
        let n = (self.immobiles + self.mobiles).max(1) as i32;
        Vec2::new((n - 1) * self.ecart + 1, 1)
    }
}

pub fn config() -> EntrainementConfig {
    Ron::<EntrainementConfig>::load_expect("icebreaker.entrainement")
        .read()
        .0
        .clone()
}

/// The dummies of the row starting at `premier`: where each stands, and
/// whether it stands still.
pub fn postes(premier: Vec3<f32>, config: &EntrainementConfig) -> Vec<(Vec3<f32>, bool)> {
    config
        .immobilite()
        .into_iter()
        .enumerate()
        .map(|(i, immobile)| (premier + Vec3::new((i as i32 * config.ecart) as f32, 0.0, 0.0), immobile))
        .collect()
}

/// The training dummy's body.
pub const BODY: Body = Body::Object(comp::object::Body::TrainingDummy);

/// How near (in blocks) a dummy must appear to a still post to be held there.
const PRES_DU_POSTE: f32 = 1.5;

/// The dummies of the village (none outside the village).
#[derive(Default)]
pub struct IcebreakerEntrainement {
    postes: Vec<(Vec3<f32>, bool)>,
}

impl IcebreakerEntrainement {
    pub fn new(postes: Vec<(Vec3<f32>, bool)>) -> Self {
        if !postes.is_empty() {
            tracing::info!(?postes, "Icebreaker: training dummies of the village (still?)");
        }
        Self { postes }
    }

    /// The dummies to spawn with the chunk `key`.
    pub fn mannequins(&self, key: Vec2<i32>) -> impl Iterator<Item = (NpcBuilder, comp::Pos)> + '_ {
        self.postes
            .iter()
            .filter(move |(pos, _)| chunk_of(*pos) == key)
            .map(|(pos, immobile)| (mannequin(*immobile), comp::Pos(*pos)))
    }

    /// The still post a dummy that just appeared at `pos` is held at, if any.
    pub fn poste_immobile(&self, pos: Vec3<f32>) -> Option<Vec3<f32>> {
        self.postes
            .iter()
            .find(|(poste, immobile)| *immobile && poste.xy().distance(pos.xy()) <= PRES_DU_POSTE)
            .map(|(poste, _)| *poste)
    }
}

fn mannequin(immobile: bool) -> NpcBuilder {
    let name = if immobile { "Mannequin immobile" } else { "Mannequin (recul)" };
    NpcBuilder::new(comp::Stats::new(Content::Plain(name.to_string()), BODY), BODY, Alignment::Passive)
        .with_health(comp::Health::new(BODY))
        .with_inventory(Inventory::with_empty())
}

/// What one attacker did to one dummy since its first hit (times in seconds).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Serie {
    pub debut: f64,
    pub dernier: f64,
    pub coups: u32,
    pub degats: f32,
}

impl Serie {
    pub fn new(time: f64, degats: f32) -> Self {
        Self {
            debut: time,
            dernier: time,
            coups: 1,
            degats,
        }
    }

    pub fn noter(&mut self, time: f64, degats: f32) {
        self.dernier = time;
        self.coups += 1;
        self.degats += degats;
    }

    /// The message for the attacker at the end of the series.
    pub fn bilan(&self) -> String {
        let duree = self.dernier - self.debut;
        let par_seconde = if duree > 0.0 {
            format!("{:.0} dégâts/s", f64::from(self.degats) / duree)
        } else {
            "un seul instant".to_string()
        };
        format!(
            "Mannequin : {:.0} dégâts en {} coup{} sur {} s ({par_seconde}).",
            self.degats,
            self.coups,
            if self.coups > 1 { "s" } else { "" },
            format!("{duree:.1}").replace('.', ",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_mannequins_sont_en_rang_les_immobiles_d_abord() {
        let config = EntrainementConfig {
            decalage: (10.0, -2.0),
            recherche: 20,
            marge: 2,
            immobiles: 2,
            mobiles: 1,
            ecart: 4,
            fin_de_serie: 4.0,
        };
        assert_eq!(config.emprise(), Vec2::new(9, 1));
        let postes = postes(Vec3::new(100.5, 50.5, 20.0), &config);
        assert_eq!(postes, vec![
            (Vec3::new(100.5, 50.5, 20.0), true),
            (Vec3::new(104.5, 50.5, 20.0), true),
            (Vec3::new(108.5, 50.5, 20.0), false),
        ]);
        let entrainement = IcebreakerEntrainement::new(postes);
        assert_eq!(entrainement.poste_immobile(Vec3::new(104.0, 51.0, 22.0)), Some(Vec3::new(104.5, 50.5, 20.0)));
        assert_eq!(entrainement.poste_immobile(Vec3::new(108.5, 50.5, 20.0)), None, "with knockback");
    }

    #[test]
    fn une_serie_donne_degats_coups_et_degats_par_seconde() {
        let mut serie = Serie::new(10.0, 30.0);
        serie.noter(11.0, 25.0);
        serie.noter(12.0, 45.0);
        assert_eq!(serie.coups, 3);
        assert_eq!(
            serie.bilan(),
            "Mannequin : 100 dégâts en 3 coups sur 2,0 s (50 dégâts/s)."
        );
        assert_eq!(
            Serie::new(3.0, 12.4).bilan(),
            "Mannequin : 12 dégâts en 1 coup sur 0,0 s (un seul instant)."
        );
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let config = config();
        assert!(config.immobiles + config.mobiles > 0 && config.fin_de_serie > 0.0);
    }
}
