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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recharge_puis_prete() {
        recharge(5.0);
        assert_eq!(reste(), Some(5));
        recharge(0.0);
        assert_eq!(reste(), None);
    }
}
