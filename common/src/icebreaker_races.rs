//! Icebreaker : le comportement de chaque race ennemie (jalon Y, `assets/icebreaker/ennemis/races.ron`).
//! La race d'un ennemi est l'espèce de son corps humanoïde ; seuls les ennemis (alignement `Enemy`)
//! humanoïdes en tiennent compte. Une race absente du fichier se comporte comme dans Veloren.

use crate::{
    assets::{AssetExt, Ron},
    comp::{Body, humanoid},
    uid::Uid,
};
use hashbrown::HashMap;
use serde::Deserialize;

/// Ce qu'une race fait de différent (tout est facultatif : rien = comme Veloren).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Comportement {
    /// Achever : un joueur à terre à moins de ce rayon (blocs) passe en tête des cibles.
    pub acheve: Option<f32>,
    /// Y.9 : il n'achève qu'à plusieurs : ce nombre de membres de la race (lui compris) à moins de
    /// `meute_rayon` blocs ; seul, il vise d'abord un joueur debout. Rien = il achève toujours.
    pub meute: Option<usize>,
    pub meute_rayon: f32,
    /// Contre de l'achèvement : celui qui vient de frapper l'ennemi (depuis moins de ces secondes)
    /// passe avant un joueur à terre.
    pub provoque_par_coup: f64,
    /// Secondes entre deux choix de cible pendant un combat (10 dans Veloren).
    pub reciblage: Option<f64>,
    /// Part de ses PV (0 à 1) sous laquelle il fuit ; rien = ne fuit pas.
    pub fuite: Option<f32>,
    /// Des groupes font la ronde entre les salles du donjon généré (Y.3).
    pub ronde: Option<Ronde>,
}

/// Les rondes d'une race (Y.3) : des guerriers d'une salle partent ensemble vers les salles voisines,
/// s'arrêtent un moment à chacune, puis reviennent sur leurs pas.
#[derive(Clone, Debug, Deserialize)]
pub struct Ronde {
    /// Nombre de groupes au plus par donjon.
    pub groupes: usize,
    /// Guerriers d'une même salle par groupe, au plus.
    pub membres: usize,
    /// Salles visitées, la sienne comprise.
    pub etapes: usize,
    /// Allure (1 = la course de Veloren).
    pub vitesse: f32,
    /// Secondes d'arrêt dans chaque salle.
    pub pause: f64,
    /// Distance (blocs) à laquelle une salle est atteinte.
    pub arrivee: f32,
}

/// Y.9 : ce que tous les ennemis humanoïdes font pour choisir leur cible parmi les joueurs qu'ils
/// ont repérés : un joueur debout avant un joueur à terre (tous à terre : ils sont achevés), et
/// ils se répartissent entre les joueurs.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Commun {
    /// Secondes entre deux choix de cible pendant un combat (rien = 10, comme Veloren).
    pub reciblage: Option<f64>,
    /// Blocs « ajoutés » à la distance d'une cible par allié qui la vise déjà (0 = le plus proche).
    pub repartition: f32,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IcebreakerRaces {
    #[serde(default)]
    pub commun: Commun,
    /// Par race : "human", "orc", "dwarf", "elf", "draugr", "danari" (les noms de `generation.ron`).
    pub races: HashMap<String, Comportement>,
}

/// Le nom d'une espèce humanoïde dans les fichiers de données.
pub fn nom(species: humanoid::Species) -> &'static str {
    match species {
        humanoid::Species::Danari => "danari",
        humanoid::Species::Dwarf => "dwarf",
        humanoid::Species::Elf => "elf",
        humanoid::Species::Human => "human",
        humanoid::Species::Orc => "orc",
        humanoid::Species::Draugr => "draugr",
    }
}

impl IcebreakerRaces {
    /// Réglages lus dans `assets/icebreaker/ennemis/races.ron` (rien si le fichier manque).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.ennemis.races").map_or_else(|_| Self::default(), |r| r.read().0.clone())
    }

    /// Le comportement d'un corps : celui de sa race s'il est humanoïde et que la race est décrite.
    pub fn de(&self, body: Option<&Body>) -> Option<&Comportement> {
        match body {
            Some(Body::Humanoid(h)) => self.races.get(nom(h.species)),
            _ => None,
        }
    }
}

/// Le comportement de la race d'un corps, lu dans le fichier de données (rien hors humanoïde).
pub fn comportement(body: Option<&Body>) -> Option<Comportement> {
    let body @ Some(Body::Humanoid(_)) = body else {
        return None;
    };
    Ron::<IcebreakerRaces>::load("icebreaker.ennemis.races")
        .ok()
        .and_then(|r| r.read().0.de(body).cloned())
}

/// Y.9 : les règles d'un ennemi humanoïde pour choisir sa cible : celles de sa race (aucune si la
/// race n'est pas décrite) et les règles communes à toutes.
#[derive(Clone, Debug, Default)]
pub struct Regles {
    pub race: Comportement,
    pub commun: Commun,
}

/// Les règles du corps d'un ennemi, lues dans le fichier de données (rien hors humanoïde, ou si
/// le fichier manque).
pub fn regles(body: Option<&Body>) -> Option<Regles> {
    let body @ Some(Body::Humanoid(_)) = body else {
        return None;
    };
    let races = Ron::<IcebreakerRaces>::load("icebreaker.ennemis.races").ok()?;
    let races = races.read();
    Some(Regles {
        race: races.0.de(body).cloned().unwrap_or_default(),
        commun: races.0.commun.clone(),
    })
}

/// Y.9 : combien d'ennemis visent déjà chaque cible (par `Uid`), compté à chaque tick par le
/// serveur (`server/src/sys/icebreaker_focus.rs`) : les agents ne voient pas les cibles des autres.
#[derive(Clone, Debug, Default)]
pub struct IcebreakerFocus(pub HashMap<Uid, u8>);

impl Regles {
    /// La race achève-t-elle les joueurs à terre, avec `proches` membres de la race (lui compris)
    /// autour de lui ? Seulement si elle achève (`acheve`), et à plusieurs si `meute` le demande.
    pub fn acheve(&self, proches: usize) -> bool {
        self.race.acheve.is_some() && self.race.meute.is_none_or(|m| proches >= m)
    }

    /// Rang d'une cible (plus petit = choisie d'abord) : 0 celui qui vient de frapper une race
    /// qui achève (le contre) ; puis, si la race achève ici (`acheve`), 1 un joueur à terre à
    /// portée et 2 le reste ; sinon 1 un joueur debout et 2 un joueur à terre (tous à terre :
    /// ils sont achevés quand même).
    pub fn rang(&self, distance: f32, a_terre: bool, l_a_frappe_depuis: Option<f64>, acheve: bool) -> u8 {
        if self.race.acheve.is_some() && l_a_frappe_depuis.is_some_and(|s| s < self.race.provoque_par_coup) {
            0
        } else if acheve {
            if a_terre && self.race.acheve.is_some_and(|r| distance < r) { 1 } else { 2 }
        } else if a_terre {
            2
        } else {
            1
        }
    }

    /// À rang égal, la cible au plus petit score : sa distance, plus `repartition` blocs par allié
    /// qui la vise déjà (`dessus`, sans se compter) : les ennemis se répartissent.
    pub fn score(&self, distance: f32, dessus: u8) -> f32 { distance + self.commun.repartition * dessus as f32 }

    /// Secondes entre deux choix de cible en combat pour la créature `id` (rien = comme
    /// Veloren) : celui de la race, sinon le commun, décalé de ± 25 % selon la créature pour
    /// que tout un groupe ne change pas de cible au même instant.
    pub fn reciblage(&self, id: u32) -> Option<f64> {
        let base = self.race.reciblage.or(self.commun.reciblage)?;
        Some(base * (0.75 + 0.5 * (id % 8) as f64 / 7.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orc() -> Regles {
        Regles {
            race: Comportement::default(),
            commun: Commun {
                reciblage: Some(3.0),
                repartition: 8.0,
            },
        }
    }

    fn danari() -> Regles {
        Regles {
            race: Comportement {
                acheve: Some(20.0),
                provoque_par_coup: 3.0,
                meute: Some(2),
                ..Comportement::default()
            },
            ..orc()
        }
    }

    #[test]
    fn un_joueur_debout_passe_avant_un_joueur_a_terre() {
        let o = orc();
        assert!(!o.acheve(5));
        // Même tout près, le joueur à terre attend.
        assert!(o.rang(20.0, false, None, false) < o.rang(1.0, true, None, false));
        // Tous à terre : même rang, le plus proche est achevé.
        assert_eq!(o.rang(3.0, true, None, false), o.rang(9.0, true, None, false));
        assert!(o.score(3.0, 0) < o.score(9.0, 0));
    }

    #[test]
    fn le_danari_acheve_a_plusieurs_seulement() {
        let d = danari();
        // Seul : comme les autres.
        assert!(!d.acheve(1));
        assert!(d.rang(15.0, false, None, d.acheve(1)) < d.rang(2.0, true, None, d.acheve(1)));
        // À deux : le joueur à terre d'abord, s'il est à portée.
        assert!(d.acheve(2));
        assert!(d.rang(15.0, true, None, true) < d.rang(2.0, false, None, true));
        assert_eq!(d.rang(25.0, true, None, true), 2);
    }

    #[test]
    fn frapper_le_danari_le_detourne() {
        let d = danari();
        assert!(d.rang(10.0, false, Some(1.0), true) < d.rang(2.0, true, None, true));
        // Le coup est trop ancien : il retourne au joueur à terre.
        assert!(d.rang(10.0, false, Some(5.0), true) > d.rang(2.0, true, None, true));
        // Un orc ne se laisse pas détourner d'un joueur debout par un coup.
        let o = orc();
        assert_eq!(o.rang(10.0, false, Some(1.0), false), o.rang(2.0, false, None, false));
    }

    #[test]
    fn les_ennemis_se_repartissent() {
        let o = orc();
        // Deux joueurs debout à 5 et 9 blocs : le deuxième ennemi prend le plus loin, libre.
        assert!(o.score(9.0, 0) < o.score(5.0, 1));
        // Mais pas un joueur bien plus loin.
        assert!(o.score(30.0, 0) > o.score(5.0, 1));
        // Autant d'ennemis sur chacun : le plus proche.
        assert!(o.score(5.0, 1) < o.score(9.0, 1));
    }

    #[test]
    fn le_reciblage_est_decale_par_creature() {
        let o = orc();
        let delais: Vec<f64> = (0..8).map(|id| o.reciblage(id).unwrap()).collect();
        assert!(delais.iter().all(|d| (2.25..=3.75).contains(d)));
        assert!(delais.windows(2).all(|w| w[0] < w[1]));
        // Sans réglage : comme Veloren.
        assert_eq!(Regles::default().reciblage(3), None);
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let r = Ron::<IcebreakerRaces>::load_expect("icebreaker.ennemis.races").read().0.clone();
        for race in ["human", "orc", "dwarf", "elf", "draugr", "danari"] {
            assert!(r.races.contains_key(race), "{race} manque dans races.ron");
        }
        assert!(r.commun.repartition > 0.0 && r.commun.reciblage.is_some());
        let body = Body::Humanoid(humanoid::Body {
            species: humanoid::Species::Danari,
            ..humanoid::Body::random()
        });
        let d = regles(Some(&body)).unwrap();
        assert!(d.race.acheve.is_some() && d.race.meute.is_some());
        assert!(d.acheve(2) && !d.acheve(1));
    }
}
