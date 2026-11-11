//! Icebreaker : la recharge de la balise lumineuse vue par le client (jalon J). Le serveur dit
//! combien de secondes il reste à la pose et quand on en ramasse une
//! (`ServerGeneral::IcebreakerBaliseRecharge`) ; la barre de raccourcis grise la balise et
//! affiche les secondes qui restent.

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

/// Quand la charge revient ; absent : prête.
static PRETE_A: Mutex<Option<Instant>> = Mutex::new(None);

/// Le serveur annonce `reste` secondes de recharge (0 : prête).
pub fn recharge(reste: f32) {
    let mut garde = PRETE_A.lock().unwrap_or_else(|e| e.into_inner());
    *garde = (reste > 0.0).then(|| Instant::now() + Duration::from_secs_f32(reste));
}

/// Secondes entières qui restent avant que la charge revienne, ou rien si elle est prête.
pub fn reste() -> Option<u32> {
    let garde = PRETE_A.lock().unwrap_or_else(|e| e.into_inner());
    let reste = (*garde)?.checked_duration_since(Instant::now())?;
    Some(reste.as_secs_f32().ceil().max(1.0) as u32)
}

/// Ce que l'emplacement de la balise dit : la quantité comme pour tout objet (rien pour un
/// seul exemplaire), et pendant la recharge, l'emplacement grisé avec les secondes restantes
/// au centre (N.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emplacement {
    pub quantite: Option<u32>,
    pub grise: bool,
    pub secondes: Option<u32>,
}

pub fn emplacement(quantite: u32, reste: Option<u32>) -> Emplacement {
    Emplacement {
        quantite: Some(quantite).filter(|q| *q > 1),
        grise: reste.is_some(),
        secondes: reste,
    }
}

/// Le texte des secondes (« 7 s »), refait seulement quand le nombre change : `fabriquer` le
/// compose (langue du joueur).
pub fn texte_secondes(secondes: u32, fabriquer: impl FnOnce(u32) -> String) -> String {
    static TEXTE: Mutex<Option<(u32, String)>> = Mutex::new(None);
    let mut garde = TEXTE.lock().unwrap_or_else(|e| e.into_inner());
    match &*garde {
        Some((n, texte)) if *n == secondes => texte.clone(),
        _ => {
            let texte = fabriquer(secondes);
            *garde = Some((secondes, texte.clone()));
            texte
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_emplacement_garde_la_quantite_et_montre_la_recharge() {
        // Prête, un exemplaire : rien d'écrit, pas grisé.
        assert_eq!(emplacement(1, None), Emplacement { quantite: None, grise: false, secondes: None });
        // En recharge avec trois balises : la quantité reste, plus les secondes.
        assert_eq!(emplacement(3, Some(7)), Emplacement {
            quantite: Some(3),
            grise: true,
            secondes: Some(7),
        });
        // En recharge avec une seule : les secondes seules.
        assert_eq!(emplacement(1, Some(2)).quantite, None);
        assert_eq!(emplacement(5, None).quantite, Some(5));
    }

    #[test]
    fn le_texte_des_secondes_n_est_refait_que_s_il_change() {
        let appels = std::cell::Cell::new(0);
        let fabriquer = |n: u32| {
            appels.set(appels.get() + 1);
            format!("{n} s")
        };
        assert_eq!(texte_secondes(42, fabriquer), "42 s");
        assert_eq!(texte_secondes(42, fabriquer), "42 s");
        assert_eq!(appels.get(), 1);
        assert_eq!(texte_secondes(41, fabriquer), "41 s");
        assert_eq!(appels.get(), 2);
    }

    #[test]
    fn recharge_puis_prete() {
        recharge(5.0);
        assert_eq!(reste(), Some(5));
        recharge(0.0);
        assert_eq!(reste(), None);
    }
}
