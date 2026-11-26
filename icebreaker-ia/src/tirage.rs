//! Le tirage pondéré d'une réaction à un événement (`design/creatures.md`, « Pas de rôle fixe dans un
//! groupe » : « chaque ennemi tire sa réaction, avec des poids par race, rôle et cran »).

/// Un des choix, en proportion de son poids ; `hasard` est tiré entre 0 et 1. Un poids nul ou
/// négatif ne sort jamais ; sans poids positif, rien.
pub fn tirer<T: Copy>(poids: &[(T, f32)], hasard: f32) -> Option<T> {
    let total: f32 = poids.iter().map(|(_, p)| p.max(0.0)).sum();
    let mut reste = hasard.clamp(0.0, 1.0) * total;
    let mut dernier = None;
    for (choix, p) in poids.iter().filter(|(_, p)| *p > 0.0) {
        dernier = Some(*choix);
        if reste < *p {
            break;
        }
        reste -= p;
    }
    dernier
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chaque choix sort en proportion de son poids ; le même hasard donne le même choix.
    #[test]
    fn le_tirage_suit_les_poids() {
        let poids = [("a", 3.0), ("b", 1.0)];
        assert_eq!(tirer(&poids, 0.0), Some("a"));
        assert_eq!(tirer(&poids, 0.74), Some("a"));
        assert_eq!(tirer(&poids, 0.76), Some("b"));
        assert_eq!(tirer(&poids, 1.0), Some("b"));
        let sortis = (0..1000).filter(|i| tirer(&poids, *i as f32 / 1000.0) == Some("b")).count();
        assert_eq!(sortis, 250);
    }

    /// Un poids nul (ou négatif) ne sort jamais ; sans poids, rien.
    #[test]
    fn un_poids_nul_ne_sort_jamais() {
        let poids = [("a", 0.0), ("b", 2.0), ("c", -1.0)];
        for i in 0..=10 {
            assert_eq!(tirer(&poids, i as f32 / 10.0), Some("b"));
        }
        assert_eq!(tirer::<&str>(&[], 0.5), None);
        assert_eq!(tirer(&[("a", 0.0)], 0.5), None);
    }
}
