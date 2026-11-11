//! Icebreaker : le temps qui reste à un joueur à terre avant de mourir (jalon T, T.2).
//! Le serveur tue après `delai_a_terre` secondes (`assets/icebreaker/mort.ron`) ; le client note
//! quand il voit une entité tomber à terre et affiche le compte à rebours (barre du joueur,
//! au-dessus du corps pour les autres).

use common::{icebreaker_mort::IcebreakerMort, uid::Uid};
use hashbrown::HashMap;
use std::sync::Mutex;

/// Pour chaque entité vue à terre : depuis quand, et la dernière fois qu'on l'a vue.
static A_TERRE: Mutex<Option<HashMap<Uid, (f64, f64)>>> = Mutex::new(None);

/// Secondes qui restent à `uid` avant de mourir s'il est à terre, et la part du délai qui reste
/// (1 à 0). À appeler à chaque image pour les entités affichées ; `maintenant` est le temps du
/// client.
pub fn reste(uid: Uid, a_terre: bool, maintenant: f64) -> Option<(f64, f32)> {
    let mut garde = A_TERRE.lock().unwrap_or_else(|e| e.into_inner());
    let vues = garde.get_or_insert_with(HashMap::new);
    // Une entité qu'on n'affiche plus depuis une seconde est oubliée.
    vues.retain(|_, (_, vu)| maintenant - *vu < 1.0);
    if !a_terre {
        vues.remove(&uid);
        return None;
    }
    let (depuis, vu) = vues.entry(uid).or_insert((maintenant, maintenant));
    *vu = maintenant;
    let regles = IcebreakerMort::actuel();
    let reste = regles.reste_a_terre(*depuis, maintenant);
    Some((reste, (reste / regles.delai_a_terre.max(0.001)) as f32))
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
