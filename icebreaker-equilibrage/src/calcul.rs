//! Icebreaker : les calculs du classeur d'équilibrage, les mêmes que ses formules Excel.

/// Dégâts d'une utilisation sur une créature ou un joueur, avant son armure.
pub fn degats(base: f64, puissance_niveau: f64, puissance_arme: f64, multiplicateur: f64) -> f64 {
    base * puissance_niveau * puissance_arme * multiplicateur
}

/// Secondes d'un geste : la part que l'arme raccourcit, et celle qu'elle ne change pas.
pub fn geste(duree: f64, vitesse: f64, fixe: f64) -> f64 { duree / vitesse + fixe }

/// `ARRONDI.SUP(ARRONDI(x; 6); 0)` : le nombre de coups entiers, sans se faire piéger par 3,0000001.
pub fn coups(x: f64) -> f64 { ((x * 1e6).round() / 1e6).ceil() }

/// Touches qu'il faut pour venir à bout de `pv` avec `degats_touche` par touche contre `armure` (part retirée).
pub fn touches_pour_tuer(pv: f64, degats_touche: f64, armure: f64) -> f64 {
    coups(pv / (degats_touche * (1.0 - armure)))
}

/// Secondes pour tuer en répétant une compétence : chaque utilisation attend le geste ou la recharge.
pub fn secondes_pour_tuer(pv: f64, degats_usage: f64, armure: f64, geste: f64, recharge: f64) -> f64 {
    (coups(pv / (degats_usage * (1.0 - armure))) - 1.0) * geste.max(recharge) + geste
}

/// Maîtrise cumulée qu'il faut pour `points` points d'arme : le premier coûte `premier`, chaque suivant `pas`
/// de plus.
pub fn maitrise_pour(points: f64, premier: f64, pas: f64) -> f64 {
    points * premier + pas * points * (points - 1.0) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les repères écrits en tête de `assets/icebreaker/combat.ron` : une épée de départ (5 à 7 de dégâts,
    /// puissance 0,6, × 5 au corps à corps) vient à bout d'un gnarling (50 PV, ~10 % d'armure) en 3 coups environ.
    #[test]
    fn repere_epee_contre_gnarling() {
        let faible = touches_pour_tuer(50.0, degats(5.0, 1.0, 0.6, 5.0), 0.1);
        let fort = touches_pour_tuer(50.0, degats(7.0, 1.0, 0.6, 5.0), 0.1);
        assert_eq!((fort, faible), (3.0, 4.0));
    }

    /// Un gnarling (2,5 à 4 de dégâts, × 9 contre un joueur) tue un joueur de 100 PV en 3 à 4 coups, sans armure.
    #[test]
    fn repere_gnarling_contre_joueur() {
        assert_eq!(touches_pour_tuer(100.0, degats(4.0, 1.0, 1.0, 9.0), 0.0), 3.0);
        assert_eq!(touches_pour_tuer(100.0, degats(3.0, 1.0, 1.0, 9.0), 0.0), 4.0);
    }

    #[test]
    fn coups_entiers() {
        assert_eq!(coups(3.000_000_01), 3.0);
        assert_eq!(coups(3.01), 4.0);
    }

    #[test]
    fn secondes() {
        // 3 utilisations d'un geste de 1 s rechargé en 6 s : 6 + 6 + 1.
        assert_eq!(secondes_pour_tuer(50.0, 20.0, 0.0, 1.0, 6.0), 13.0);
        // Sans recharge, les gestes s'enchaînent.
        assert_eq!(secondes_pour_tuer(50.0, 20.0, 0.0, 1.0, 0.0), 3.0);
    }

    /// `salle_armes.ron` : 100, puis 150, 200… de maîtrise par point.
    #[test]
    fn maitrise() {
        assert_eq!(maitrise_pour(1.0, 100.0, 50.0), 100.0);
        assert_eq!(maitrise_pour(3.0, 100.0, 50.0), 450.0);
    }
}
