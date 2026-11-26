//! L'ouïe d'un ennemi (`design/creatures.md`, « Pour Veloren ») : un combat, une mort, un cri font un
//! bruit qui porte à un rayon donné (`assets/icebreaker/ia/bruits.ron`), multiplié par la finesse
//! d'ouïe de celui qui écoute, et atténué selon l'écart de hauteur. Les murs n'arrêtent aucun son
//! tant que le serveur ne connaît pas les salles (IA.13).

use serde::Deserialize;
use std::collections::HashMap;
use vek::Vec3;

/// Le fichier `bruits.ron`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Bruits {
    /// Par sorte de bruit ("combat", "mort", "cri"…) : sa portée, en blocs.
    pub portees: HashMap<String, f32>,
    /// Ce que compte un bloc d'écart de hauteur (1 : comme un bloc à plat ; 2 : le double).
    pub hauteur: f32,
    /// Facteur sur la portée pour celui qui dort.
    pub endormi: f32,
}

impl Bruits {
    /// La portée d'une sorte de bruit (0 : inconnue, personne ne l'entend).
    pub fn portee(&self, sorte: &str) -> f32 { self.portees.get(sorte).copied().unwrap_or(0.0) }

    /// Un ennemi de cette finesse d'ouïe entend-il ce bruit, fait à `ecart` de lui ?
    pub fn entend(&self, sorte: &str, finesse: f32, ecart: Vec3<f32>) -> bool {
        let loin = Vec3::new(ecart.x, ecart.y, ecart.z * self.hauteur.max(1.0)).magnitude();
        loin <= self.portee(sorte) * finesse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bruits() -> Bruits {
        Bruits { portees: HashMap::from([("combat".to_string(), 20.0), ("mort".to_string(), 30.0)]), hauteur: 2.0, endormi: 0.5 }
    }

    /// « Un combat, une mort, un cri font un bruit qui porte à un rayon donné. »
    #[test]
    fn un_bruit_porte_a_son_rayon() {
        assert!(bruits().entend("combat", 1.0, Vec3::new(19.0, 0.0, 0.0)));
        assert!(!bruits().entend("combat", 1.0, Vec3::new(21.0, 0.0, 0.0)));
        assert!(bruits().entend("mort", 1.0, Vec3::new(21.0, 0.0, 0.0)));
        // Une sorte que le fichier ne connaît pas ne s'entend pas.
        assert!(!bruits().entend("soupir", 1.0, Vec3::zero() + 0.1));
    }

    /// « … multiplié par la finesse d'ouïe. »
    #[test]
    fn la_finesse_d_ouie_multiplie_la_portee() {
        assert!(bruits().entend("combat", 1.5, Vec3::new(28.0, 0.0, 0.0)));
        assert!(!bruits().entend("combat", 0.5, Vec3::new(12.0, 0.0, 0.0)));
        // Sourd : rien.
        assert!(!bruits().entend("combat", 0.0, Vec3::new(1.0, 0.0, 0.0)));
    }

    /// « Le bruit […] atténué selon l'écart de hauteur » : un étage au-dessus s'entend de moins loin.
    #[test]
    fn un_ecart_de_hauteur_attenue() {
        assert!(bruits().entend("combat", 1.0, Vec3::new(15.0, 0.0, 0.0)));
        assert!(!bruits().entend("combat", 1.0, Vec3::new(15.0, 0.0, 8.0)));
        assert!(!bruits().entend("combat", 1.0, Vec3::new(0.0, 0.0, -11.0)));
        assert!(bruits().entend("combat", 1.0, Vec3::new(0.0, 0.0, -9.0)));
    }
}
