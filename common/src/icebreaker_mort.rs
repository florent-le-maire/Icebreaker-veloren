//! Icebreaker : à terre, relève et mort (jalon T, `assets/icebreaker/mort.ron`).
//! À 0 PV un humanoïde tombe à terre (état de Veloren) ; un allié le relève avec une part de ses
//! PV ; passé un délai, il meurt. Mort en expédition, il laisse un sac qu'un allié fouille.

use crate::assets::{AssetExt, Ron};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct IcebreakerMort {
    /// Secondes à terre avant de mourir.
    pub delai_a_terre: f64,
    /// Part des PV maximum gardée en tombant à terre (T.9 : plusieurs coups pour achever, au lieu
    /// du 1 PV de Veloren).
    #[serde(default = "pv_a_terre_defaut")]
    pub pv_a_terre: f32,
    /// Part des PV maximum rendue à la relève (0 à 1).
    pub pv_releve: f32,
    /// Secondes pour fouiller le sac d'un mort.
    pub duree_fouille: f32,
    /// Icebreaker T.12 : l'autel de résurrection (`docs/icebreaker/design/donjon.md`, « Portail et autel »).
    #[serde(default)]
    pub autel: Autel,
}

/// Icebreaker T.12 : l'autel relève les morts dont le prieur porte le cristal.
#[derive(Clone, Debug, Deserialize)]
pub struct Autel {
    /// Secondes d'interaction.
    pub duree: f32,
    /// Part des PV maximum rendue à chaque ressuscité (0 à 1).
    pub pv_resurrection: f32,
    /// Part de sa vie du moment que perd le prieur, une seule fois (0 à 1).
    pub part_vie_perdue: f32,
    /// PV qu'il faut au moins pour prier.
    pub pv_minimum: f32,
}

impl Default for Autel {
    fn default() -> Self {
        Self {
            duree: 5.0,
            pv_resurrection: 0.25,
            part_vie_perdue: 0.5,
            pv_minimum: 2.0,
        }
    }
}

impl Default for IcebreakerMort {
    fn default() -> Self {
        Self {
            delai_a_terre: 30.0,
            pv_a_terre: pv_a_terre_defaut(),
            pv_releve: 0.25,
            duree_fouille: 3.0,
            autel: Autel::default(),
        }
    }
}

fn pv_a_terre_defaut() -> f32 { 0.9 }

/// T.9 : un allié est en train de relever `cible` (état « interagir » de Veloren, relève) : son
/// décompte à terre s'arrête.
pub fn en_releve<'a>(
    cible: crate::uid::Uid,
    etats: impl IntoIterator<Item = &'a crate::comp::CharacterState>,
) -> bool {
    use crate::{comp::CharacterState, interaction::InteractionKind, states::interact::InteractKind};
    etats.into_iter().any(|etat| {
        matches!(etat, CharacterState::Interact(c) if matches!(
            c.static_data.interact,
            InteractKind::Entity { target, kind: InteractionKind::HelpDowned } if target == cible
        ))
    })
}

/// A.22 : la relève de zone d'un ultime (« Envol ») atteint une entité à terre à `rayon` au plus de `centre`.
pub fn dans_la_releve_de_zone(
    centre: vek::Vec3<f32>,
    rayon: f32,
    pos: vek::Vec3<f32>,
    a_terre: bool,
) -> bool {
    a_terre && centre.distance_squared(pos) <= rayon * rayon
}

/// L'objet « sac du mort » posé au sol à la mort d'un joueur en expédition (T.3).
pub const SAC_DU_MORT: &str = "common.items.icebreaker.sac_du_mort";

/// `item` est un sac de mort : il se fouille, il ne se ramasse pas.
pub fn est_sac(item: &crate::comp::Item) -> bool {
    matches!(
        item.item_definition_id(),
        crate::comp::item::ItemDefinitionId::Simple(id) if id == SAC_DU_MORT
    )
}

/// Le cristal d'un joueur mort en expédition (T.11) : dans son sac, à son nom ; l'autel le consomme
/// pour relever ce joueur (T.12).
pub const CRISTAL: &str = "common.items.icebreaker.cristal";

/// `item` est le cristal d'un mort.
pub fn est_cristal(item: &crate::comp::Item) -> bool {
    matches!(
        item.item_definition_id(),
        crate::comp::item::ItemDefinitionId::Simple(id) if id == CRISTAL
    )
}

/// Le cristal du joueur mort `uid`, nommé `nom`.
pub fn cristal_de(uid: crate::uid::Uid, nom: &str) -> crate::comp::Item {
    let mut cristal = crate::comp::Item::new_from_asset_expect(CRISTAL);
    cristal.icebreaker_lier(uid, nom.to_string());
    cristal
}

impl IcebreakerMort {
    /// Réglages lus dans `assets/icebreaker/mort.ron` (valeurs par défaut si le fichier manque).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.mort").map_or_else(|_| Self::default(), |m| m.read().0.clone())
    }

    /// Secondes qui restent à un joueur tombé à terre à `depuis`, à l'instant `maintenant`.
    pub fn reste_a_terre(&self, depuis: f64, maintenant: f64) -> f64 {
        (self.delai_a_terre - (maintenant - depuis)).max(0.0)
    }

    /// Le délai à terre est écoulé : le joueur meurt.
    pub fn doit_mourir(&self, depuis: f64, maintenant: f64) -> bool {
        maintenant - depuis >= self.delai_a_terre
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn releve_de_zone() {
        let centre = vek::Vec3::new(10.0, 10.0, 5.0);
        let pres = vek::Vec3::new(14.0, 10.0, 5.0);
        let loin = vek::Vec3::new(30.0, 10.0, 5.0);
        assert!(dans_la_releve_de_zone(centre, 8.0, pres, true));
        // Debout, ou hors du rayon : rien.
        assert!(!dans_la_releve_de_zone(centre, 8.0, pres, false));
        assert!(!dans_la_releve_de_zone(centre, 8.0, loin, true));
    }

    #[test]
    fn un_cristal_par_mort() {
        use crate::comp::item::{ItemDesc, ItemI18n};
        let uid = |n| crate::uid::Uid(std::num::NonZeroU64::new(n).unwrap());
        let (a, b) = (cristal_de(uid(1), "Flo"), cristal_de(uid(2), "Sayanel"));
        assert!(est_cristal(&a) && !est_sac(&a));
        assert_eq!(a.icebreaker_proprietaire(), Some((uid(1), "Flo")));
        // Deux cristaux de morts différents ne s'empilent pas ; deux du même, si.
        assert!(!a.can_merge(&b));
        assert!(a.can_merge(&cristal_de(uid(1), "Flo")));
        // La copie (envoi au client, partage) garde son mort ; le nom affiché le dit.
        let msm = crate::comp::item::MaterialStatManifest::load().read();
        let abilities = crate::comp::item::tool::AbilityMap::load().read();
        assert_eq!(b.duplicate(&abilities, &msm).icebreaker_proprietaire(), Some((uid(2), "Sayanel")));
        let (nom, _) = b.i18n(&ItemI18n::new_expect());
        assert!(format!("{nom:?}").contains("icebreaker-cristal-de") && format!("{nom:?}").contains("Sayanel"), "{nom:?}");
        // Un objet ordinaire n'a pas de propriétaire.
        let (nom, _) = crate::comp::Item::new_from_asset_expect(CRISTAL).i18n(&ItemI18n::new_expect());
        assert!(!format!("{nom:?}").contains("cristal-de"), "{nom:?}");
    }

    #[test]
    fn delai_a_terre() {
        let m = IcebreakerMort::default();
        assert!(!m.doit_mourir(10.0, 39.9));
        assert!(m.doit_mourir(10.0, 40.0));
        assert_eq!(m.reste_a_terre(10.0, 17.0), 23.0);
        assert_eq!(m.reste_a_terre(10.0, 99.0), 0.0);
    }

    #[test]
    fn fichier_de_donnees_lisible() {
        let m = Ron::<IcebreakerMort>::load_expect("icebreaker.mort").read().0.clone();
        assert!(m.delai_a_terre > 0.0);
        assert!((0.0..=1.0).contains(&m.pv_releve));
        assert!(m.duree_fouille > 0.0);
        assert!((0.0..=1.0).contains(&m.pv_a_terre));
    }

    #[test]
    fn trois_coups_pour_achever_un_joueur_a_terre() {
        // T.9 : un coup moyen de gnarling (2,5 à 4 dégâts, × degats_contre_joueurs) contre un
        // joueur de 100 PV tombé à terre avec `pv_a_terre` de ses PV.
        let m = Ron::<IcebreakerMort>::load_expect("icebreaker.mort").read().0.clone();
        let c = Ron::<crate::icebreaker_combat::IcebreakerCombat>::load_expect("icebreaker.combat")
            .read()
            .0
            .clone();
        let coups = 100.0 * m.pv_a_terre / (3.25 * c.degats_contre_joueurs);
        assert!((2.5..=3.5).contains(&coups), "{coups}");
    }

    #[test]
    fn decompte_arrete_pendant_la_releve() {
        use crate::{
            comp::CharacterState,
            interaction::InteractionKind,
            states::{
                interact::{Data, InteractKind, StaticData},
                utils::StageSection,
            },
        };
        use std::time::Duration;
        let uid = |n| crate::uid::Uid(std::num::NonZeroU64::new(n).unwrap());
        let interagir = |target, kind| {
            CharacterState::Interact(Data {
                static_data: StaticData {
                    buildup_duration: Duration::ZERO,
                    use_duration: None,
                    recover_duration: Duration::ZERO,
                    interact: InteractKind::Entity { target, kind },
                    was_wielded: false,
                    was_sneak: false,
                    required_item: None,
                },
                timer: Duration::ZERO,
                stage_section: StageSection::Buildup,
            })
        };
        let etats = [
            CharacterState::Idle(Default::default()),
            interagir(uid(2), InteractionKind::HelpDowned),
            interagir(uid(3), InteractionKind::Pet),
        ];
        assert!(en_releve(uid(2), &etats));
        assert!(!en_releve(uid(1), &etats));
        assert!(!en_releve(uid(3), &etats));
    }
}
