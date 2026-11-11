//! Icebreaker : le temps qui reste à un joueur à terre avant de mourir (jalon T, T.2).
//! Le serveur tue après `delai_a_terre` secondes (`assets/icebreaker/mort.ron`) ; le client note
//! quand il voit une entité tomber à terre et affiche le compte à rebours (barre du joueur,
//! au-dessus du corps pour les autres).

use common::{icebreaker_mort::IcebreakerMort, uid::Uid};
use hashbrown::HashMap;
use std::sync::{Mutex, OnceLock};

/// Une entité qu'on n'affiche plus depuis cette durée (secondes) est oubliée.
const OUBLI: f64 = 1.0;

/// Icebreaker : les entités vues à terre (N.9 : logique séparée de l'état global, pour les tests).
#[derive(Default)]
struct Vues {
    /// Pour chaque entité : depuis quand elle est à terre, et la dernière fois qu'on l'a vue.
    table: HashMap<Uid, (f64, f64)>,
    /// Dernier ménage de la table.
    dernier_menage: f64,
}

impl Vues {
    fn reste<'a>(
        &mut self,
        uid: Uid,
        a_terre: bool,
        maintenant: f64,
        regles: impl FnOnce() -> &'a IcebreakerMort,
    ) -> Option<(f64, f32)> {
        if !a_terre {
            // Cas de presque toutes les entités : une recherche, rien d'autre.
            if !self.table.is_empty() {
                self.table.remove(&uid);
            }
            return None;
        }
        // Le ménage (oubli des entités plus affichées) se fait au plus une fois par seconde.
        if maintenant - self.dernier_menage >= OUBLI {
            self.dernier_menage = maintenant;
            self.table.retain(|_, (_, vu)| maintenant - *vu < OUBLI);
        }
        let (depuis, vu) = self.table.entry(uid).or_insert((maintenant, maintenant));
        // Entrée périmée pas encore balayée : l'entité repart de zéro.
        if maintenant - *vu >= OUBLI {
            *depuis = maintenant;
        }
        *vu = maintenant;
        let regles = regles();
        let reste = regles.reste_a_terre(*depuis, maintenant);
        Some((reste, (reste / regles.delai_a_terre.max(0.001)) as f32))
    }
}

static A_TERRE: Mutex<Option<Vues>> = Mutex::new(None);

/// Réglages lus une seule fois (N.9).
static REGLES: OnceLock<IcebreakerMort> = OnceLock::new();

/// Secondes qui restent à `uid` avant de mourir s'il est à terre, et la part du délai qui reste
/// (1 à 0). À appeler à chaque image pour les entités affichées ; `maintenant` est le temps du
/// client.
pub fn reste(uid: Uid, a_terre: bool, maintenant: f64) -> Option<(f64, f32)> {
    let mut garde = A_TERRE.lock().unwrap_or_else(|e| e.into_inner());
    garde
        .get_or_insert_with(Vues::default)
        .reste(uid, a_terre, maintenant, || REGLES.get_or_init(IcebreakerMort::actuel))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u64) -> Uid { Uid(std::num::NonZeroU64::new(n).unwrap()) }

    #[test]
    fn tombe_a_terre_puis_se_releve() {
        let regles = IcebreakerMort::default();
        let mut v = Vues::default();
        assert_eq!(v.reste(uid(1), false, 0.0, || &regles), None);
        assert!(v.reste(uid(1), true, 1.0, || &regles).is_some());
        assert_eq!(v.reste(uid(1), false, 2.0, || &regles), None);
        assert!(v.table.is_empty());
        // Retombé : le compte à rebours repart.
        let (reste, part) = v.reste(uid(1), true, 5.0, || &regles).unwrap();
        assert_eq!(reste, regles.delai_a_terre);
        assert_eq!(part, 1.0);
    }

    #[test]
    fn compte_a_rebours_depuis_le_premier_instant_vu() {
        let regles = IcebreakerMort::default();
        let mut v = Vues::default();
        v.reste(uid(1), true, 10.0, || &regles);
        let mut t = 10.0;
        while t < 17.0 {
            t += 0.5;
            v.reste(uid(1), true, t, || &regles);
        }
        let (reste, _) = v.reste(uid(1), true, 17.0, || &regles).unwrap();
        assert_eq!(reste, regles.delai_a_terre - 7.0);
        while t < 45.0 {
            t += 0.5;
            v.reste(uid(1), true, t, || &regles);
        }
        let (reste, part) = v.reste(uid(1), true, 45.0, || &regles).unwrap();
        assert_eq!((reste, part), (0.0, 0.0));
    }

    #[test]
    fn entite_plus_affichee_oubliee() {
        let regles = IcebreakerMort::default();
        let mut v = Vues::default();
        v.reste(uid(1), true, 10.0, || &regles);
        v.reste(uid(2), true, 10.0, || &regles);
        // Seule l'entité 2 continue d'être affichée.
        v.reste(uid(2), true, 11.5, || &regles);
        v.reste(uid(2), true, 12.6, || &regles);
        assert!(!v.table.contains_key(&uid(1)));
        assert!(v.table.contains_key(&uid(2)));
        // Revenue plus tard, l'entité 1 repart de zéro.
        let (reste, _) = v.reste(uid(1), true, 20.0, || &regles).unwrap();
        assert_eq!(reste, regles.delai_a_terre);
    }

    #[test]
    fn entree_perimee_sans_menage_repart_de_zero() {
        let regles = IcebreakerMort::default();
        let mut v = Vues::default();
        v.reste(uid(1), true, 10.0, || &regles);
        let (reste, _) = v.reste(uid(1), true, 10.5 + 1.0, || &regles).unwrap();
        assert_eq!(reste, regles.delai_a_terre);
    }
}

/// Icebreaker : le serveur a un registre (campagne, village) : un mort y choisit entre le retour
/// au village et regarder un allié (T.4). Noté par l'écran de personnages.
static REGISTRE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_registre(actif: bool) { REGISTRE.store(actif, std::sync::atomic::Ordering::Relaxed); }

pub fn registre() -> bool { REGISTRE.load(std::sync::atomic::Ordering::Relaxed) }

/// L'allié à regarder après `actuel` : les autres joueurs vivants que le client voit, dans
/// l'ordre de leur `Uid`, en boucle.
pub fn allie_suivant(client: &client::Client, actuel: Option<specs::Entity>) -> Option<specs::Entity> {
    use specs::WorldExt;
    let ecs = client.state().ecs();
    let healths = ecs.read_storage::<common::comp::Health>();
    let ids = ecs.read_resource::<common::uid::IdMaps>();
    // Le composant `Player` des autres n'est pas envoyé au client (N.7 : la liste était toujours
    // vide) : les joueurs viennent de la liste des joueurs, leur entité de leur `Uid`.
    let mut allies = client
        .player_list()
        .keys()
        .filter_map(|uid| Some((*uid, ids.uid_entity(*uid)?)))
        .filter(|(_, e)| *e != client.entity())
        // Les fantômes (spectateurs) n'ont pas de santé.
        .filter(|(_, e)| healths.get(*e).is_some_and(|h| !h.is_dead))
        .collect::<Vec<_>>();
    allies.sort_by_key(|(uid, _)| uid.0);
    let actuel = actuel.and_then(|e| allies.iter().position(|(_, a)| *a == e));
    let suivant = actuel.map_or(0, |i| i + 1);
    allies.get(suivant % allies.len().max(1)).map(|(_, e)| *e)
}
