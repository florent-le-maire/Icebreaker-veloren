//! Icebreaker : les calculs du classeur d'équilibrage, les mêmes que ses formules Excel.

/// Dégâts d'une utilisation sur une créature ou un joueur, avant son armure.
pub fn degats(base: f64, puissance_niveau: f64, puissance_arme: f64, multiplicateur: f64) -> f64 {
    base * puissance_niveau * puissance_arme * multiplicateur
}

/// `ARRONDI.SUP(ARRONDI(x; 6); 0)` : le nombre de coups entiers, sans se faire piéger par 3,0000001.
pub fn coups(x: f64) -> f64 { ((x * 1e6).round() / 1e6).ceil() }

/// Touches qu'il faut pour venir à bout de `pv` avec `degats_touche` par touche contre `armure` (part retirée).
pub fn touches_pour_tuer(pv: f64, degats_touche: f64, armure: f64) -> f64 {
    coups(pv / (degats_touche * (1.0 - armure)))
}

/// Le temps d'une utilisation : sa durée, ce qui précède la première touche et suit la dernière. `tenu` : une
/// capacité qu'on tient (rayon), dont la durée est une seconde de tir et où `avant` s'ajoute une fois.
#[derive(Clone, Copy, Debug)]
pub struct Geste {
    pub duree: f64,
    pub avant: f64,
    pub apres: f64,
    pub tenu: bool,
}

impl Geste {
    /// Secondes pour placer `touches` touches quand une utilisation de `par_usage` touches y suffit.
    fn dans_un_usage(&self, touches: f64, par_usage: f64) -> f64 {
        self.avant + (self.duree - self.avant - self.apres) * touches / par_usage
    }

    fn en_tenant(&self, touches: f64, par_usage: f64) -> f64 { self.avant + self.duree * touches / par_usage }
}

/// Secondes pour tuer en répétant une attaque sans recharge : `touches` touches, à `par_usage` par geste.
pub fn secondes_en_repetant(touches: f64, par_usage: f64, geste: Geste) -> f64 {
    if geste.tenu {
        geste.en_tenant(touches, par_usage)
    } else if touches <= par_usage {
        geste.dans_un_usage(touches, par_usage)
    } else {
        geste.duree * touches / par_usage
    }
}

/// Secondes pour tuer avec une compétence à recharge : si une utilisation suffit, le temps d'y placer les
/// touches qu'il faut ; sinon la compétence une fois, puis l'attaque principale (`dps_principale`, avant armure)
/// jusqu'au bout.
pub fn secondes_avec_competence(
    touches: f64,
    par_usage: f64,
    geste: Geste,
    pv: f64,
    armure: f64,
    degats_usage: f64,
    dps_principale: f64,
) -> f64 {
    if geste.tenu {
        geste.en_tenant(touches, par_usage)
    } else if touches <= par_usage {
        geste.dans_un_usage(touches, par_usage)
    } else {
        geste.duree + (pv / (1.0 - armure) - degats_usage) / dps_principale
    }
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

    /// La Fournaise au bâton de départ : 1,875 s de préparation, puis 6 touches en 2,5 s, 0,5 s de récupération.
    /// Il en faut 5 pour le guerrier ennemi : la préparation, puis cinq sixièmes du rayon ; pas quatre
    /// utilisations séparées par 75 s de recharge.
    #[test]
    fn un_rayon_qui_suffit() {
        let fournaise = Geste { duree: 4.875, avant: 1.875, apres: 0.5, tenu: false };
        let t = secondes_avec_competence(5.0, 6.0, fournaise, 50.0, 0.5, 130.0, 20.0);
        assert!((t - (1.875 + 2.5 * 5.0 / 6.0)).abs() < 1e-9);
    }

    /// Une compétence qui ne suffit pas : on finit à l'attaque principale, sans attendre la recharge.
    #[test]
    fn on_finit_a_l_attaque_principale() {
        let coup = Geste { duree: 1.0, avant: 0.4, apres: 0.3, tenu: false };
        // 50 PV sans armure, une compétence de 20 en 1 s, puis 10 par seconde : 1 + 30 / 10.
        assert_eq!(secondes_avec_competence(3.0, 1.0, coup, 50.0, 0.0, 20.0, 10.0), 4.0);
        // Une attaque de base répétée : trois coups d'un geste d'une seconde ; un seul : sans la récupération.
        assert_eq!(secondes_en_repetant(3.0, 1.0, coup), 3.0);
        assert_eq!(secondes_en_repetant(1.0, 1.0, coup), 0.7);
    }

    /// Un rayon tenu : la préparation une fois, puis le temps qu'il faut à 3 touches par seconde.
    #[test]
    fn un_rayon_tenu() {
        let rayon = Geste { duree: 1.0, avant: 0.25, apres: 0.0, tenu: true };
        assert_eq!(secondes_en_repetant(6.0, 3.0, rayon), 2.25);
    }

    /// `salle_armes.ron` : 100, puis 150, 200… de maîtrise par point.
    #[test]
    fn maitrise() {
        assert_eq!(maitrise_pour(1.0, 100.0, 50.0), 100.0);
        assert_eq!(maitrise_pour(3.0, 100.0, 50.0), 450.0);
    }
}
