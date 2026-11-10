//! Icebreaker: the training ground of the village (X.14). A row of Veloren's
//! training dummies, at an offset from the square, to try skills and damage.
//! They never die: after a series of hits, each attacker is told what they
//! did and the dummy gets all its health back. Settings in
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
    /// The first dummy, from the village square (in blocks).
    pub decalage: (f32, f32),
    /// How many dummies, in a row towards +x.
    pub nombre: u32,
    /// The gap between two dummies (in blocks).
    pub ecart: f32,
    /// Seconds without a hit that end a series.
    pub fin_de_serie: f64,
}

pub fn config() -> EntrainementConfig {
    Ron::<EntrainementConfig>::load_expect("icebreaker.entrainement")
        .read()
        .0
        .clone()
}

/// Where the dummies stand, around the square `place` (before finding the
/// ground).
pub fn postes(place: Vec2<f32>, config: &EntrainementConfig) -> Vec<Vec2<f32>> {
    let first = place + Vec2::from(config.decalage);
    (0..config.nombre)
        .map(|i| first + Vec2::new(i as f32 * config.ecart, 0.0))
        .collect()
}

/// The training dummy's body.
pub const BODY: Body = Body::Object(comp::object::Body::TrainingDummy);

/// The dummies of the village (none outside the village).
#[derive(Default)]
pub struct IcebreakerEntrainement {
    postes: Vec<Vec3<f32>>,
}

impl IcebreakerEntrainement {
    pub fn new(postes: Vec<Vec3<f32>>) -> Self {
        if !postes.is_empty() {
            tracing::info!(?postes, "Icebreaker: training dummies of the village");
        }
        Self { postes }
    }

    /// The dummies to spawn with the chunk `key`.
    pub fn mannequins(&self, key: Vec2<i32>) -> impl Iterator<Item = (NpcBuilder, comp::Pos)> + '_ {
        self.postes
            .iter()
            .filter(move |pos| chunk_of(**pos) == key)
            .map(|pos| (mannequin(), comp::Pos(*pos)))
    }
}

fn mannequin() -> NpcBuilder {
    NpcBuilder::new(
        comp::Stats::new(Content::with_attr("name-custom-village-dummy", "neut"), BODY),
        BODY,
        Alignment::Passive,
    )
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
    fn les_mannequins_sont_en_rang_depuis_la_place() {
        let config = EntrainementConfig {
            decalage: (10.0, -2.0),
            nombre: 3,
            ecart: 4.0,
            fin_de_serie: 4.0,
        };
        assert_eq!(postes(Vec2::new(100.0, 50.0), &config), vec![
            Vec2::new(110.0, 48.0),
            Vec2::new(114.0, 48.0),
            Vec2::new(118.0, 48.0),
        ]);
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
        assert!(config.nombre > 0 && config.fin_de_serie > 0.0);
    }
}
