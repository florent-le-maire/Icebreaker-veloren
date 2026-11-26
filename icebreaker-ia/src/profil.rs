//! Les réglages d'un ennemi : ceux de sa race, de son rôle et de son cran d'habileté, posés sur ceux
//! qui sont communs à tous (`assets/icebreaker/ia/profils.ron`). Tout est facultatif : une couche ne
//! dit que ce qui la distingue.

use crate::{evenements::Evenement, reactions::Reaction};
use serde::Deserialize;
use std::collections::HashMap;

/// Déclare les réglages facultatifs du profil, et la fusion de deux couches : un réglage ajouté ici
/// ne peut pas être oublié par `sur`.
macro_rules! profil {
    ($( $(#[$doc:meta])* $champ:ident : $ty:ty, )*) => {
        /// Les réglages d'un ennemi, une fois ses couches posées sur le commun.
        #[derive(Clone, Debug, Default, Deserialize, PartialEq)]
        #[serde(default)]
        pub struct Profil {
            $( $(#[$doc])* pub $champ: Option<$ty>, )*
            /// Par événement : les réactions qu'il tire, avec leur poids. Un événement absent : ce
            /// que fait le tronc.
            pub reactions: HashMap<Evenement, Vec<(Reaction, f32)>>,
        }

        impl Profil {
            /// Ces réglages posés sur `base` : ce qui est dit ici l'emporte, le reste vient de `base`.
            pub fn sur(&self, base: &Profil) -> Profil {
                Profil {
                    $( $champ: self.$champ.clone().or_else(|| base.$champ.clone()), )*
                    reactions: {
                        // Événement par événement : la liste d'ici remplace celle de la base.
                        let mut reactions = base.reactions.clone();
                        reactions.extend(self.reactions.clone());
                        reactions
                    },
                }
            }
        }
    };
}

profil! {
    // --- Vue (IA.2) ---
    /// Jusqu'où il voit devant lui (blocs) ; au-delà de `vue_nette`, il ne fait que soupçonner.
    vue: f32,
    /// Jusqu'où il reconnaît tout de suite ce qu'il voit.
    vue_nette: f32,
    /// Largeur de son cône de vue, en degrés.
    cone: f32,
    /// Rayon dans lequel il perçoit aussi dans son dos.
    dos: f32,
    /// Secondes pendant lesquelles il doit garder un soupçon en vue pour le repérer.
    soupcon: f64,
    // --- Ouïe (IA.3) ---
    /// Finesse d'ouïe : facteur sur la portée des bruits (`bruits.ron`) ; rien = sourd.
    ouie: f32,
    // --- Chercher (IA.2) ---
    /// Secondes pendant lesquelles il court vers la dernière position connue d'une cible perdue de vue.
    poursuite: f64,
    /// Secondes qu'il passe à fouiller du regard l'endroit où il est venu voir.
    fouille: f64,
    /// Secondes au plus pour rejoindre l'endroit à fouiller ; au-delà, il renonce.
    trajet: f64,
    /// Distance (blocs) à laquelle un endroit est atteint.
    arrivee: f32,
    /// Blocs au plus entre lui et son poste pendant un combat ; au-delà, il lâche sa cible et rentre.
    laisse: f32,
    // --- Choix de cible (Y.2, Y.9) ---
    /// Achever : un joueur à terre à moins de ce rayon (blocs) passe en tête des cibles.
    acheve: f32,
    /// Il n'achève qu'à plusieurs : ce nombre de membres de la race (lui compris) à moins de
    /// `meute_rayon` blocs ; seul, il vise d'abord un joueur debout. Rien = il achève toujours.
    meute: usize,
    meute_rayon: f32,
    /// Contre de l'achèvement : celui qui vient de le frapper (depuis moins de ces secondes) passe
    /// avant un joueur à terre.
    provoque_par_coup: f64,
    /// Secondes entre deux choix de cible pendant un combat, ± 25 % selon la créature.
    reciblage: f64,
    /// Blocs « ajoutés » à la distance d'une cible par allié qui la vise déjà (0 = le plus proche).
    repartition: f32,
    // --- Réactions (IA.4) ---
    /// Secondes au moins entre deux tirages du même événement.
    tirage_delai: f64,
    /// Distance (blocs) à l'allié à laquelle celui qui vient prévenir crie.
    prevenir_a: f32,
    /// Rayon (blocs) dans lequel il connaît les postes des siens.
    voisinage: f32,
    // --- Habileté (IA.5) : ce que règle le cran ---
    /// Délai de réaction : secondes entre le moment où il repère un intrus et son premier coup.
    reaction: f64,
    /// Temps mort : secondes sans attaquer après chacune de ses attaques.
    pause: f64,
    /// Secondes au plus pendant lesquelles il tient une attaque sans s'arrêter (un enchaînement, un
    /// rayon) ; rien = tant qu'il veut.
    enchainement: f64,
    /// Il fait les roulades que sa façon de se battre lui propose.
    roule: bool,
    /// Il pare quand sa façon de se battre le lui propose.
    pare: bool,
    // --- Défense réactive (IA.7) ---
    /// Distance (blocs) sous laquelle un coup de mêlée armé vers lui le menace.
    garde: f32,
    // --- Le mage (IA.6) ---
    /// Sa table de sorts (`combat.rs`) : il choisit lui-même le sort à lancer, selon la distance et
    /// la situation. Rien = la façon de se battre de Veloren choisit.
    sorts: Vec<crate::combat::Sort>,
    // --- Fuite ---
    /// Part de ses PV (0 à 1) sous laquelle il fuit ; rien = ne fuit pas.
    fuite: f32,
}

/// Le fichier `profils.ron`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Profils {
    #[serde(default)]
    pub commun: Profil,
    /// Par race : "human", "orc", "dwarf", "elf", "draugr", "danari".
    #[serde(default)]
    pub races: HashMap<String, Profil>,
    /// Par rôle : "guerrier", "archer", "mage", "chef" (le nom de la fiche de `ennemis/`).
    #[serde(default)]
    pub roles: HashMap<String, Profil>,
    /// Par cran d'habileté : les clés de `crans` de `difficulte.ron`.
    #[serde(default)]
    pub crans: HashMap<String, Profil>,
}

impl Profils {
    /// Le profil d'un ennemi : son cran posé sur son rôle, posé sur sa race, posée sur le commun
    /// (ce que le fichier ne connaît pas ne change rien).
    pub fn profil(&self, race: &str, role: &str, cran: &str) -> Profil {
        [self.races.get(race), self.roles.get(role), self.crans.get(cran)]
            .into_iter()
            .flatten()
            .fold(self.commun.clone(), |base, couche| couche.sur(&base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_race_l_emporte_sur_le_commun_champ_par_champ() {
        let profils = Profils {
            commun: Profil { reciblage: Some(3.0), repartition: Some(8.0), ..Profil::default() },
            races: HashMap::from([("danari".to_string(), Profil {
                reciblage: Some(2.0),
                fuite: Some(0.3),
                ..Profil::default()
            })]),
            ..Profils::default()
        };
        let danari = profils.profil("danari", "", "");
        assert_eq!(danari.reciblage, Some(2.0));
        assert_eq!(danari.repartition, Some(8.0));
        assert_eq!(danari.fuite, Some(0.3));
        // Une race que le fichier ne décrit pas : le commun.
        assert_eq!(profils.profil("orc", "", ""), profils.commun);
    }

    /// « Profil = commun + race + rôle + cran » : le cran l'emporte sur le rôle, qui l'emporte sur
    /// la race, qui l'emporte sur le commun.
    #[test]
    fn le_cran_l_emporte_sur_le_role_puis_la_race() {
        let un = |profil: Profil| HashMap::from([("x".to_string(), profil)]);
        let profils = Profils {
            commun: Profil { vue: Some(1.0), cone: Some(1.0), dos: Some(1.0), ouie: Some(1.0), ..Profil::default() },
            races: un(Profil { cone: Some(2.0), dos: Some(2.0), ouie: Some(2.0), ..Profil::default() }),
            roles: un(Profil { dos: Some(3.0), ouie: Some(3.0), ..Profil::default() }),
            crans: un(Profil { ouie: Some(4.0), ..Profil::default() }),
        };
        let p = profils.profil("x", "x", "x");
        assert_eq!((p.vue, p.cone, p.dos, p.ouie), (Some(1.0), Some(2.0), Some(3.0), Some(4.0)));
        // Un rôle ou un cran inconnu ne change rien.
        assert_eq!(profils.profil("x", "?", "?").ouie, Some(2.0));
    }

    /// Une couche se pose sur une autre réglage par réglage, et événement par événement pour les
    /// réactions (la fusion est écrite par la macro `profil!` : aucun réglage ne peut y manquer).
    #[test]
    fn une_couche_se_pose_reglage_par_reglage() {
        let base = Profil {
            vue: Some(1.0),
            roule: Some(false),
            reactions: HashMap::from([
                (Evenement::VoitIntrus, vec![(Reaction::Prevenir, 1.0)]),
                (Evenement::AllieTombe, vec![(Reaction::Prevenir, 1.0)]),
            ]),
            ..Profil::default()
        };
        let couche = Profil {
            roule: Some(true),
            pause: Some(2.0),
            reactions: HashMap::from([(Evenement::VoitIntrus, vec![(Reaction::Combattre, 1.0)])]),
            ..Profil::default()
        };
        let p = couche.sur(&base);
        assert_eq!((p.vue, p.roule, p.pause), (Some(1.0), Some(true), Some(2.0)));
        assert_eq!(p.reactions[&Evenement::VoitIntrus], vec![(Reaction::Combattre, 1.0)]);
        assert_eq!(p.reactions[&Evenement::AllieTombe], vec![(Reaction::Prevenir, 1.0)]);
        assert_eq!(base.sur(&Profil::default()), base);
        assert_eq!(Profil::default().sur(&base), base);
    }
}
