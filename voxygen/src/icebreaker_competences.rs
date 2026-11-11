//! Icebreaker : la barre de compétences vue par le client (jalon A, A.6). Un personnage du
//! registre qui porte `IcebreakerCompetences` lance, avec les touches 1 à 3, la compétence du jeu
//! d'armes qu'il tient puis son ultime. À chaque image, le HUD calcule ce que montrent les trois
//! cases ([`cases`]) et le garde dans un instantané ([`rafraichir`]) que la barre de raccourcis
//! lit. Sans le composant, rien n'est gardé et la barre reste celle de Veloren.

use common::{
    assets::{AssetExt, Ron},
    comp::{Inventory, ability::CharacterAbility},
    icebreaker_competences::{IcebreakerCompetences, TOUCHES, TOUCHE_ULTIME, registre},
};
use std::sync::Mutex;

/// Ce que montre la case d'une touche de compétence.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    /// Id de la capacité de Veloren dont on reprend l'icône.
    pub icone: &'static str,
    pub nom: &'static str,
    pub description: &'static str,
    pub niveau: u8,
    /// Secondes de recharge à ce niveau.
    pub recharge: f32,
    /// Secondes entières qui restent, arrondies au-dessus ; `None` : prête.
    pub reste: Option<u32>,
    /// Coût en endurance.
    pub endurance: f32,
    pub ultime: bool,
}

/// Les trois cases d'un personnage qui a nos compétences, à l'instant `maintenant` (secondes du
/// temps du serveur). `None` : touche vide.
pub fn cases(
    competences: &IcebreakerCompetences,
    inventaire: Option<&Inventory>,
    maintenant: f64,
) -> [Option<Case>; TOUCHES] {
    std::array::from_fn(|touche| {
        let (numero, niveau) = competences.sur_touche(touche, inventaire)?;
        let entree = registre().par_numero(numero)?;
        let competence = &entree.competence;
        let effectif = competence.effectif(niveau);
        let endurance = Ron::<CharacterAbility>::load(effectif.capacite)
            .ok()
            .map_or(0.0, |capacite| capacite.read().0.energy_cost());
        Some(Case {
            icone: competence.id_icone(),
            nom: competence.nom.as_str(),
            description: competence.description.as_str(),
            niveau,
            recharge: effectif.recharge,
            reste: competences
                .reste_recharge(numero, maintenant)
                .map(|reste| reste.ceil().max(1.0) as u32),
            endurance,
            ultime: touche == TOUCHE_ULTIME,
        })
    })
}

/// Ce que montre la case ronde du passif actif (toujours actif, sans touche).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CasePassif {
    pub nom: &'static str,
    pub description: &'static str,
}

/// Le passif actif du personnage ; `None` : il n'en a pas, ou son id est inconnu de la classe.
pub fn passif(competences: &IcebreakerCompetences) -> Option<CasePassif> {
    let id = competences.acquis.passif.as_deref()?;
    let (_, passif) = registre()
        .passifs_de(&competences.classe)?
        .passifs
        .iter()
        .find(|(autre, _)| autre == id)?;
    Some(CasePassif {
        nom: passif.nom.as_str(),
        description: passif.description.as_str(),
    })
}

/// Les cases et le passif du personnage vu ; `None` : il n'a pas nos compétences.
type Instantane = ([Option<Case>; TOUCHES], Option<CasePassif>);
static INSTANTANE: Mutex<Option<Instantane>> = Mutex::new(None);

/// Le HUD range ce que montrent les cases et la case ronde à cette image.
pub fn rafraichir(vu: Option<([Option<Case>; TOUCHES], Option<CasePassif>)>) {
    *INSTANTANE.lock().unwrap_or_else(|e| e.into_inner()) = vu;
}

/// Le personnage vu a nos compétences : la barre n'est plus celle de Veloren.
pub fn actif() -> bool { INSTANTANE.lock().unwrap_or_else(|e| e.into_inner()).is_some() }

/// Le passif actif du personnage vu.
pub fn passif_actif() -> Option<CasePassif> {
    INSTANTANE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|(_, passif)| *passif)
}

/// La case de la touche `touche` (comptée depuis zéro).
pub fn case(touche: usize) -> Option<Case> {
    INSTANTANE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|(cases, _)| cases.get(touche).cloned().flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{
        comp::{Item, inventory::loadout_builder::LoadoutBuilder},
        icebreaker_competences::Acquis,
    };

    fn en_mains(jeu: &str) -> Inventory {
        let armes = &registre().jeu(jeu).unwrap().armes;
        let loadout = LoadoutBuilder::empty()
            .active_mainhand(Some(Item::new_from_asset_expect(&armes.0)))
            .active_offhand(armes.1.as_deref().map(Item::new_from_asset_expect))
            .build();
        Inventory::with_loadout_humanoid(loadout)
    }

    fn guerrier() -> IcebreakerCompetences {
        IcebreakerCompetences::nouveau("guerrier", Acquis::par_defaut("guerrier")).unwrap()
    }

    #[test]
    fn trois_cases_dont_l_ultime_avec_la_grande_epee() {
        let inventaire = en_mains("grande_epee");
        let cases = cases(&guerrier(), Some(&inventaire), 100.0);
        let [Some(a), Some(b), Some(ultime)] = &cases else {
            panic!("trois cases attendues : {cases:?}");
        };
        assert!(!a.ultime && !b.ultime);
        assert!(ultime.ultime);
        assert_ne!(a.nom, b.nom);
        assert_eq!(a.niveau, 1);
        assert!(a.recharge > 0.0);
        assert_eq!(a.reste, None);
        assert!(a.icone.starts_with("common.abilities."));
    }

    #[test]
    fn la_recharge_en_cours_s_arrondit_au_dessus() {
        let inventaire = en_mains("grande_epee");
        let mut competences = guerrier();
        let (numero, _) = competences.sur_touche(0, Some(&inventaire)).unwrap();
        competences.recharges.insert(numero, 106.2);
        let cases = cases(&competences, Some(&inventaire), 100.0);
        assert_eq!(cases[0].as_ref().unwrap().reste, Some(7));
        assert_eq!(cases[1].as_ref().unwrap().reste, None);
        // Presque finie : une seconde, jamais zéro.
        competences.recharges.insert(numero, 100.01);
        let cases = super::cases(&competences, Some(&inventaire), 100.0);
        assert_eq!(cases[0].as_ref().unwrap().reste, Some(1));
        // Finie : prête.
        let cases = super::cases(&competences, Some(&inventaire), 101.0);
        assert_eq!(cases[0].as_ref().unwrap().reste, None);
    }

    #[test]
    fn un_jeu_non_appris_laisse_les_deux_premieres_cases_vides() {
        let inventaire = en_mains("marteau");
        let cases = cases(&guerrier(), Some(&inventaire), 0.0);
        assert!(cases[0].is_none() && cases[1].is_none());
        assert!(cases[2].as_ref().is_some_and(|c| c.ultime));
    }

    #[test]
    fn l_instantane_se_lit_et_se_vide() {
        let inventaire = en_mains("grande_epee");
        let competences = guerrier();
        rafraichir(Some((
            cases(&competences, Some(&inventaire), 0.0),
            passif(&competences),
        )));
        assert!(actif());
        assert!(passif_actif().is_some());
        assert!(case(0).is_some() && case(TOUCHE_ULTIME).is_some());
        assert!(case(TOUCHES).is_none());
        rafraichir(None);
        assert!(!actif());
        assert!(case(0).is_none());
        assert!(passif_actif().is_none());
    }

    #[test]
    fn le_passif_actif_a_un_nom_et_une_description() {
        let passif = passif(&guerrier()).expect("un guerrier par défaut a un passif");
        assert!(!passif.nom.is_empty() && !passif.description.is_empty());
    }

    #[test]
    fn sans_passif_actif_ou_avec_un_inconnu_pas_de_case_ronde() {
        let mut competences = guerrier();
        competences.acquis.passif = Some("n_existe_pas".to_string());
        assert_eq!(passif(&competences), None);
        competences.acquis.passif = None;
        assert_eq!(passif(&competences), None);
    }
}
