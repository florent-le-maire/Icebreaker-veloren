//! Icebreaker : essai du plan du village sans les mains (jalon N), pour voir
//! l'interface dans le vrai client sans y toucher.
//!
//! Avec `ICEBREAKER_AUTO_PLAN=caisse,charrette`, le client attend d'être entré
//! dans le monde, ouvre le plan du village et en prend une capture d'écran,
//! puis, pour chaque sorte de bâtiment de la liste (celles de
//! `vue_village.ron`), clique dessus et prend une capture de son panneau. Un
//! élément qui commence par `/` est une commande (`/expedition 2`), envoyée
//! comme le ferait un bouton, puis capturée. Les captures vont dans le dossier
//! de captures du client (`screenshots_path` de ses réglages). `plan` seul : le
//! plan sans panneau. `fermer` ferme le plan, pour voir ce qui se montre hors
//! du plan (`/balise`). `regarder` : mort, passe à l'allié suivant, comme le
//! clic gauche (`/kill` deux fois avant : à terre, puis mort). `onglet 3` : le
//! troisième onglet du panneau ouvert (salle d'armes). `controles` : les
//! réglages sur l'onglet Contrôles.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Le temps d'entrer dans le monde et de charger le village.
const ENTREE: Duration = Duration::from_secs(25);
/// Le temps qu'un panneau arrive et s'affiche avant sa capture.
const ENTRE_DEUX: Duration = Duration::from_secs(3);

/// Le temps entre deux pas : `ICEBREAKER_AUTO_PAS` (secondes), pour saisir un geste court (un ultime,
/// A.22), sinon `ENTRE_DEUX`.
fn entre_deux() -> Duration {
    std::env::var("ICEBREAKER_AUTO_PAS")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|s| *s > 0.0)
        .map_or(ENTRE_DEUX, Duration::from_secs_f32)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pas {
    Ouvrir,
    /// Fermer le plan du village.
    Fermer,
    /// Mort, regarder l'allié suivant (le clic gauche).
    Regarder,
    /// Allumer le portail ou prier à l'autel le plus proche, comme la touche d'interaction (`rituel`, T.10, T.12).
    Rituel,
    /// Allumer la lanterne (`lanterne`), pour voir une salle sombre.
    Lanterne,
    /// Appuyer (`true`) ou relâcher (`false`) la touche de compétence `n` (comptée depuis zéro).
    Touche(usize, bool),
    /// Cliquer le bâtiment de cette sorte.
    Cliquer(String),
    /// Cliquer l'onglet `n` (compté depuis zéro) du panneau ouvert.
    Onglet(usize),
    /// Ouvrir les réglages sur l'onglet Contrôles (`controles`, X.16).
    Controles,
    /// Envoyer cette commande (nom, arguments), comme un bouton du panneau.
    Commande(String, Vec<String>),
    Capturer,
}

pub struct Auto {
    a_faire: VecDeque<Pas>,
    /// Pas avant.
    quand: Instant,
}

impl Auto {
    /// L'essai demandé par `ICEBREAKER_AUTO_PLAN`, s'il l'est.
    pub fn charger() -> Option<Self> {
        let sortes = std::env::var("ICEBREAKER_AUTO_PLAN").ok()?;
        Some(Self::de(&sortes, Instant::now()))
    }

    fn de(sortes: &str, maintenant: Instant) -> Self {
        let mut a_faire = VecDeque::from([Pas::Ouvrir, Pas::Capturer]);
        for sorte in sortes
            .split(',')
            .map(str::trim)
            .filter(|sorte| !sorte.is_empty() && *sorte != "plan")
        {
            // `touche 1` : la touche de compétence 1 (A.6), appuyée, capturée, relâchée au pas
            // suivant.
            if let Some(n) = sorte
                .strip_prefix("touche ")
                .and_then(|n| n.trim().parse::<usize>().ok())
                .and_then(|n| n.checked_sub(1))
            {
                a_faire.extend([Pas::Touche(n, true), Pas::Capturer, Pas::Touche(n, false)]);
                continue;
            }
            // `onglet 3` : le troisième onglet du panneau ouvert, puis sa capture.
            if let Some(n) = sorte
                .strip_prefix("onglet ")
                .and_then(|n| n.trim().parse::<usize>().ok())
                .and_then(|n| n.checked_sub(1))
            {
                a_faire.extend([Pas::Onglet(n), Pas::Capturer]);
                continue;
            }
            a_faire.push_back(match sorte.strip_prefix('/') {
                Some(commande) => {
                    let mut mots = commande.split_whitespace().map(str::to_string);
                    Pas::Commande(mots.next().unwrap_or_default(), mots.collect())
                },
                None if sorte == "fermer" => Pas::Fermer,
                None if sorte == "regarder" => Pas::Regarder,
                None if sorte == "rituel" => Pas::Rituel,
                None if sorte == "lanterne" => Pas::Lanterne,
                None if sorte == "controles" => Pas::Controles,
                None => Pas::Cliquer(sorte.to_string()),
            });
            a_faire.push_back(Pas::Capturer);
        }
        Self {
            a_faire,
            quand: maintenant + ENTREE,
        }
    }

    /// Le pas à faire maintenant, s'il est temps ; le suivant attendra.
    pub fn pas(&mut self, maintenant: Instant) -> Option<Pas> {
        if maintenant < self.quand {
            return None;
        }
        let pas = self.a_faire.pop_front()?;
        self.quand = maintenant + entre_deux();
        Some(pas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_essai_ouvre_le_plan_puis_clique_et_capture_chaque_batiment() {
        let debut = Instant::now();
        let mut auto = Auto::de("caisse, charrette", debut);
        assert_eq!(auto.pas(debut), None, "pas avant d'être entré dans le monde");
        let mut t = debut + ENTREE;
        let mut pas = Vec::new();
        while let Some(p) = auto.pas(t) {
            assert_eq!(auto.pas(t), None, "un pas à la fois");
            pas.push(p);
            t += ENTRE_DEUX;
        }
        assert_eq!(pas, vec![
            Pas::Ouvrir,
            Pas::Capturer,
            Pas::Cliquer("caisse".into()),
            Pas::Capturer,
            Pas::Cliquer("charrette".into()),
            Pas::Capturer,
        ]);
    }

    #[test]
    fn un_element_qui_commence_par_une_barre_est_une_commande() {
        let debut = Instant::now();
        let mut auto = Auto::de("depart, /expedition 2, fermer, regarder", debut);
        let mut t = debut + ENTREE;
        let mut pas = Vec::new();
        while let Some(p) = auto.pas(t) {
            pas.push(p);
            t += ENTRE_DEUX;
        }
        assert_eq!(pas[2..], [
            Pas::Cliquer("depart".into()),
            Pas::Capturer,
            Pas::Commande("expedition".into(), vec!["2".into()]),
            Pas::Capturer,
            Pas::Fermer,
            Pas::Capturer,
            Pas::Regarder,
            Pas::Capturer,
        ]);
    }

    #[test]
    fn onglet_3_choisit_le_troisieme_onglet_puis_capture() {
        let debut = Instant::now();
        let mut auto = Auto::de("salle_armes, onglet 3, onglet zero", debut);
        let mut t = debut + ENTREE;
        let mut pas = Vec::new();
        while let Some(p) = auto.pas(t) {
            pas.push(p);
            t += ENTRE_DEUX;
        }
        assert_eq!(pas[2..], [
            Pas::Cliquer("salle_armes".into()),
            Pas::Capturer,
            Pas::Onglet(2),
            Pas::Capturer,
            // Pas un numéro : pris pour une sorte de bâtiment, comme avant.
            Pas::Cliquer("onglet zero".into()),
            Pas::Capturer,
        ]);
    }

    #[test]
    fn plan_seul_ne_clique_rien() {
        let debut = Instant::now();
        let mut auto = Auto::de("plan", debut);
        let t = debut + ENTREE;
        assert_eq!(auto.pas(t), Some(Pas::Ouvrir));
        assert_eq!(auto.pas(t + ENTRE_DEUX), Some(Pas::Capturer));
        assert_eq!(auto.pas(t + ENTRE_DEUX * 2), None);
    }
}
