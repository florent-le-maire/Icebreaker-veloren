//! Icebreaker : d'où part le tir du joueur, pour viser (X.19).
//!
//! La direction du tir va de ce point au point visé (sous le réticule, ou le cercle du
//! verrouillage). Le serveur ne fait pas partir tout au même endroit : un projectile part
//! des yeux (`Body::projectile_offsets`), un rayon de la mi-hauteur du corps
//! (`basic_beam::beam_offsets`). Viser depuis les yeux faisait passer les rayons (sceptre,
//! Fournaise) sous le réticule, parallèlement à la bonne ligne.

use common::{
    comp::{Body, CharacterState},
    states::basic_beam,
    terrain::Block,
    util::Dir,
};
use vek::*;

/// Le départ du tir, relatif à la position du joueur : celui du rayon si un rayon est en
/// cours, sinon celui des projectiles. Mêmes calculs que le serveur.
pub fn depart_du_tir(
    etat: Option<&CharacterState>,
    body: &Body,
    regard_corps: Vec3<f32>,
    look_dir: Dir,
    scale: f32,
    vitesse_relative: Vec3<f32>,
    au_sol: Option<Block>,
) -> Vec3<f32> {
    match etat {
        Some(CharacterState::BasicBeam(_)) => basic_beam::beam_offsets(
            body,
            look_dir,
            regard_corps,
            vitesse_relative,
            au_sol,
        ),
        _ => body.projectile_offsets(regard_corps, scale),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::comp::humanoid;

    fn humain() -> Body { Body::Humanoid(humanoid::Body::random()) }

    #[test]
    fn un_rayon_part_plus_bas_qu_un_projectile() {
        let body = humain();
        let regard = Vec3::unit_y();
        let projectile = depart_du_tir(None, &body, regard, Dir::forward(), 1.0, Vec3::zero(), None);
        assert_eq!(projectile, body.projectile_offsets(regard, 1.0));
        let rayon = basic_beam::beam_offsets(&body, Dir::forward(), regard, Vec3::zero(), None);
        // Le rayon part à mi-hauteur, le projectile aux yeux : c'était l'écart sous le réticule.
        assert!((rayon.z - body.height() * 0.5).abs() < 1e-4);
        assert!(projectile.z - rayon.z > 0.3, "{projectile:?} {rayon:?}");
    }
}
