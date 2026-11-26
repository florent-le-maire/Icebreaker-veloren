//! Coincé (`design/creatures.md`, « Coincé ») : une créature qui fuit sans arriver à s'éloigner, la
//! menace toute proche, tire une réaction : se tapir sur place, ou forcer le passage.

use vek::Vec3;

/// « Elle fonce vers un point 6 m derrière la menace, 1,5 m sur le côté. »
const DERRIERE: f32 = 6.0;
const DE_COTE: f32 = 1.5;

/// Un passage en force : où il fonce, et depuis quand.
#[derive(Clone, Copy, Debug)]
pub struct Forcer {
    pub lieu: Vec3<f32>,
    pub depuis: f64,
}

impl Forcer {
    /// À travers la menace : derrière elle, un peu sur le côté (`a_gauche`), pour la contourner en
    /// la frôlant.
    pub fn a_travers(moi: Vec3<f32>, menace: Vec3<f32>, a_gauche: bool, temps: f64) -> Self {
        let vers = (menace - moi).xy().try_normalized().unwrap_or(vek::Vec2::unit_x());
        let cote = vek::Vec2::new(-vers.y, vers.x) * if a_gauche { DE_COTE } else { -DE_COTE };
        Self { lieu: menace + Vec3::from(vers * DERRIERE + cote), depuis: temps }
    }
}
