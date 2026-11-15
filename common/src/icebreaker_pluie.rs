//! Icebreaker : pluie de projectiles sur une zone visée (ultime « Volée
//! mortelle », A.23). Une attaque à distance qui porte `icebreaker_pluie` ne
//! tire pas : elle marque un cercle au sol, là où le joueur vise, puis les
//! projectiles y tombent du ciel en plusieurs salves. Les réglages sont dans
//! le fichier de la capacité (`assets/icebreaker/abilities/…`).

use serde::{Deserialize, Serialize};
use vek::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pluie {
    /// Secondes entre l'apparition du cercle et la première salve.
    pub delai: f32,
    /// Nombre de salves.
    pub salves: u32,
    /// Secondes entre deux salves.
    pub intervalle: f32,
    /// Projectiles par salve.
    pub par_salve: u32,
    /// Rayon du cercle, en blocs.
    pub rayon: f32,
    /// Hauteur d'où tombent les projectiles (moins sous un plafond).
    pub hauteur: f32,
    /// Distance au plus entre le lanceur et le centre du cercle.
    pub portee: f32,
}

impl Pluie {
    /// Centre du cercle : le point visé, ramené à `portee` du lanceur ; sans
    /// point visé, à `portee` devant lui.
    pub fn centre(&self, lanceur: Vec3<f32>, vise: Option<Vec3<f32>>, regard: Vec3<f32>) -> Vec3<f32> {
        match vise {
            Some(vise) => {
                let ecart = vise - lanceur;
                let distance = ecart.magnitude();
                if distance > self.portee && distance > 0.0 {
                    lanceur + ecart * (self.portee / distance)
                } else {
                    vise
                }
            },
            None => {
                let devant = Vec2::from(regard).try_normalized().unwrap_or(Vec2::unit_y());
                lanceur + Vec3::from(devant * self.portee)
            },
        }
    }

    /// Nombre de salves déjà dues `age` secondes après l'apparition du cercle.
    pub fn salves_dues(&self, age: f64) -> u32 {
        let depuis = age - f64::from(self.delai);
        if depuis < 0.0 {
            0
        } else if self.intervalle <= 0.0 {
            self.salves
        } else {
            ((depuis / f64::from(self.intervalle)) as u32 + 1).min(self.salves)
        }
    }

    /// Secondes entre l'apparition du cercle et la dernière salve.
    pub fn duree(&self) -> f64 {
        f64::from(self.delai) + f64::from(self.intervalle) * f64::from(self.salves.saturating_sub(1))
    }

    /// Point de départ d'un projectile au-dessus du cercle : `alea` (deux
    /// nombres de 0 à 1) le place dans le disque, sans tasser au centre.
    pub fn depart(&self, centre: Vec3<f32>, hauteur: f32, alea: (f32, f32)) -> Vec3<f32> {
        let distance = self.rayon * alea.0.clamp(0.0, 1.0).sqrt();
        let angle = alea.1 * std::f32::consts::TAU;
        centre + Vec3::new(distance * angle.cos(), distance * angle.sin(), hauteur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pluie() -> Pluie {
        Pluie {
            delai: 0.8,
            salves: 5,
            intervalle: 1.0,
            par_salve: 12,
            rayon: 6.0,
            hauteur: 14.0,
            portee: 30.0,
        }
    }

    #[test]
    fn les_salves_tombent_une_a_une() {
        let p = pluie();
        assert_eq!(p.salves_dues(0.0), 0);
        assert_eq!(p.salves_dues(0.79), 0);
        assert_eq!(p.salves_dues(0.81), 1);
        assert_eq!(p.salves_dues(1.79), 1);
        assert_eq!(p.salves_dues(1.81), 2);
        assert_eq!(p.salves_dues(4.81), 5);
        // Pas plus que prévu, même longtemps après.
        assert_eq!(p.salves_dues(60.0), 5);
        assert!((p.duree() - 4.8).abs() < 1e-6);
    }

    #[test]
    fn le_cercle_reste_a_portee() {
        let p = pluie();
        let lanceur = Vec3::new(0.0, 0.0, 10.0);
        // À portée : le point visé.
        let pres = Vec3::new(10.0, 0.0, 10.0);
        assert_eq!(p.centre(lanceur, Some(pres), Vec3::unit_y()), pres);
        // Trop loin : ramené à la portée, dans la même direction.
        let loin = Vec3::new(90.0, 0.0, 10.0);
        let centre = p.centre(lanceur, Some(loin), Vec3::unit_y());
        assert!((centre - Vec3::new(30.0, 0.0, 10.0)).magnitude() < 1e-3);
        // Rien de visé : à portée devant le lanceur, à sa hauteur.
        let devant = p.centre(lanceur, None, Vec3::new(0.0, 1.0, -0.5));
        assert!((devant - Vec3::new(0.0, 30.0, 10.0)).magnitude() < 1e-3);
    }

    #[test]
    fn les_projectiles_partent_dans_le_disque() {
        let p = pluie();
        let centre = Vec3::new(5.0, 5.0, 0.0);
        for alea in [(0.0, 0.0), (1.0, 0.25), (0.5, 0.9), (1.0, 1.0)] {
            let depart = p.depart(centre, 14.0, alea);
            assert!((depart - centre).xy().magnitude() <= p.rayon + 1e-3);
            assert_eq!(depart.z, 14.0);
        }
    }
}
