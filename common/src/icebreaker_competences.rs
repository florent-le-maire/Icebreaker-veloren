//! Icebreaker : jeux d'armes, compétences, ultimes et passifs (jalon A, `decisions/0020`).
//!
//! Un **jeu d'armes** est ce qu'un personnage tient en mains (épée + bouclier, grande épée…) : il
//! donne un pool de compétences d'arme, dont deux sont placées sur les touches 1 et 2. La **classe**
//! donne un pool d'ultimes (un placé, touche 3) et de passifs (un actif). Une compétence renvoie à
//! un fichier de capacité de Veloren (`CharacterAbility`) : c'est lui qui porte le geste et les
//! dégâts ; nous y ajoutons une recharge et des niveaux.
//!
//! Données : `assets/icebreaker/jeux_armes/<jeu>.ron`, `ultimes/<classe>.ron`,
//! `passifs/<classe>.ron`, `salle_armes.ron`. Ce que le personnage a acquis ([`Acquis`]) vit dans
//! sa fiche du registre (`server/src/icebreaker_registre.rs`).

use crate::{
    assets::{self, AssetExt, Ron},
    comp::{
        Inventory, StatsModifier,
        ability::{Ability, AbilityInput, CharacterAbility, SpecifiedAbility},
        buff::{BuffEffect, ModifierKind},
        inventory::{
            item::{
                ItemKind,
                tool::{ContextualIndex, Hands, Stats, ToolKind},
            },
            slot::EquipSlot,
        },
    },
    resources::GameMode,
    states::behavior::JoinData,
};
use serde::{Deserialize, Serialize};
use specs::{Component, DerefFlaggedStorage, WorldExt};
use std::{collections::BTreeMap, sync::LazyLock};
use tracing::error;

/// Niveau d'une compétence qu'on vient de débloquer.
pub const NIVEAU_DEPART: u8 = 1;
/// Compétences d'arme placées (touches 1 et 2).
pub const PLACES_ARME: usize = 2;
/// La touche de l'ultime (la troisième), comptée depuis zéro comme les entrées de capacité.
pub const TOUCHE_ULTIME: usize = PLACES_ARME;
/// Nombre de touches de compétences : deux pour l'arme, une pour l'ultime.
pub const TOUCHES: usize = TOUCHE_ULTIME + 1;

/// Une arme attendue dans la main droite : son type et le nombre de mains qu'elle prend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct Prise {
    pub arme: ToolKind,
    pub mains: Hands,
}

/// Un jeu d'armes d'une classe (`assets/icebreaker/jeux_armes/<id>.ron`).
#[derive(Clone, Debug, Deserialize)]
pub struct JeuArmes {
    /// Ce que les joueurs lisent.
    pub nom: String,
    /// La classe qui peut l'apprendre (`classes.ron`).
    pub classe: String,
    /// Icône dans l'interface, parmi les icônes d'armes du HUD (comme `classes.ron`).
    #[serde(default)]
    pub icone: String,
    /// Le jeu d'armes que reçoit un personnage de cette classe créé avant le jalon A.
    #[serde(default)]
    pub defaut: bool,
    pub main_droite: Prise,
    /// Type de l'arme de main gauche ; `None` : main gauche vide.
    #[serde(default)]
    pub main_gauche: Option<ToolKind>,
    /// Les armes remises avec ce jeu (main droite, main gauche) : ids d'objets.
    pub armes: (String, Option<String>),
    /// Compétences d'arme, dans l'ordre d'affichage : (id, compétence). Les ids sont uniques dans
    /// tout le jeu (compétences d'arme et ultimes confondus).
    pub competences: Vec<(String, Competence)>,
}

/// Une compétence d'arme ou un ultime.
#[derive(Clone, Debug, Deserialize)]
pub struct Competence {
    pub nom: String,
    #[serde(default)]
    pub description: String,
    /// Id du fichier de la capacité (`CharacterAbility`), de Veloren ou à nous
    /// (`assets/icebreaker/abilities/`).
    pub capacite: String,
    /// Id d'une capacité de Veloren dont on reprend l'animation : le client choisit le geste d'après
    /// cet id. Absent : celui de `capacite` (qui doit alors être une capacité de Veloren).
    #[serde(default)]
    pub animation: Option<String>,
    /// Id d'une capacité de Veloren dont on reprend l'icône. Absent : celui de l'animation.
    #[serde(default)]
    pub icone: Option<String>,
    /// Secondes de recharge au niveau 1.
    pub recharge: f32,
    /// Niveaux 2 et suivants ; le niveau 1 est la compétence telle quelle. Vide pour un ultime.
    #[serde(default)]
    pub niveaux: Vec<Niveau>,
}

/// Ce que vaut une compétence à un niveau donné. Les multiplicateurs s'écrivent en entier à chaque
/// niveau (ils ne se cumulent pas) ; `capacite` et `animation` restent ceux du dernier niveau qui
/// en a donné.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Niveau {
    /// Multiplicateur des dégâts et des effets.
    pub puissance: f32,
    /// Multiplicateur de la portée.
    pub portee: f32,
    /// Multiplicateur de la recharge.
    pub recharge: f32,
    /// La compétence se transforme : id d'une autre capacité.
    pub capacite: Option<String>,
    /// Animation de la capacité transformée, si elle change.
    pub animation: Option<String>,
    /// Ce que le niveau apporte, en une phrase (salle d'armes).
    pub texte: String,
}

impl Default for Niveau {
    fn default() -> Self {
        Self {
            puissance: 1.0,
            portee: 1.0,
            recharge: 1.0,
            capacite: None,
            animation: None,
            texte: String::new(),
        }
    }
}

/// Une compétence à un niveau donné, prête à être lancée.
#[derive(Clone, Debug, PartialEq)]
pub struct Effectif<'a> {
    /// Id du fichier de la capacité à lancer.
    pub capacite: &'a str,
    /// Id de capacité de Veloren dont le client reprend l'animation.
    pub animation: &'a str,
    /// Multiplicateurs à passer à `CharacterAbility::adjusted_by_stats`.
    pub stats: Stats,
    /// Secondes de recharge.
    pub recharge: f32,
}

impl Competence {
    /// Le plus haut niveau que la compétence peut atteindre.
    pub fn niveau_max(&self) -> u8 { NIVEAU_DEPART + self.niveaux.len() as u8 }

    /// La compétence au niveau `niveau` (ramené entre 1 et le niveau maximum).
    pub fn effectif(&self, niveau: u8) -> Effectif<'_> {
        let niveau = niveau.clamp(NIVEAU_DEPART, self.niveau_max());
        let passes = &self.niveaux[..usize::from(niveau - NIVEAU_DEPART)];
        let capacite = passes
            .iter()
            .rev()
            .find_map(|n| n.capacite.as_deref())
            .unwrap_or(&self.capacite);
        let animation = passes
            .iter()
            .rev()
            .find_map(|n| n.animation.as_deref())
            .or(self.animation.as_deref())
            .unwrap_or(&self.capacite);
        let (puissance, portee, recharge) = passes
            .last()
            .map_or((1.0, 1.0, 1.0), |n| (n.puissance, n.portee, n.recharge));
        Effectif {
            capacite,
            animation,
            stats: Stats {
                power: puissance,
                effect_power: puissance,
                buff_strength: puissance,
                range: portee,
                ..Stats::one()
            },
            recharge: self.recharge * recharge,
        }
    }

    /// Id de capacité de Veloren dont l'interface reprend l'icône.
    pub fn id_icone(&self) -> &str {
        self.icone
            .as_deref()
            .or(self.animation.as_deref())
            .unwrap_or(&self.capacite)
    }
}

/// Un passif : des effets permanents sur les stats du personnage.
#[derive(Clone, Debug, Deserialize)]
pub struct Passif {
    pub nom: String,
    #[serde(default)]
    pub description: String,
    pub effets: Vec<BuffEffect>,
}

/// Les passifs d'une classe (`assets/icebreaker/passifs/<classe>.ron`).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Passifs {
    /// L'inné de la classe : toujours actif, non choisi.
    #[serde(default)]
    pub inne: Vec<BuffEffect>,
    /// Les passifs à apprendre, dans l'ordre d'affichage : (id, passif).
    #[serde(default)]
    pub passifs: Vec<(String, Passif)>,
}

/// Les ultimes d'une classe (`assets/icebreaker/ultimes/<classe>.ron`).
#[derive(Clone, Debug, Default, Deserialize)]
struct Ultimes {
    ultimes: Vec<(String, Competence)>,
}

/// D'où vient une compétence du registre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sorte {
    /// Compétence d'arme du jeu d'armes `jeu`.
    Arme { jeu: String },
    /// Ultime de la classe `classe`.
    Ultime { classe: String },
}

/// Une compétence du registre, avec son id et sa provenance.
#[derive(Clone, Debug)]
pub struct Entree {
    pub id: String,
    pub sorte: Sorte,
    pub competence: Competence,
}

/// Tout ce que les données décrivent, lu une fois.
#[derive(Clone, Debug, Default)]
pub struct Registre {
    /// Jeux d'armes par id (le nom du fichier).
    jeux: BTreeMap<String, JeuArmes>,
    /// Ultimes par classe.
    ultimes: BTreeMap<String, Vec<(String, Competence)>>,
    /// Passifs par classe.
    passifs: BTreeMap<String, Passifs>,
    /// Compétences d'arme et ultimes, triés par id : la place dans cette liste est le numéro de la
    /// compétence sur le réseau (`Ability::Icebreaker`), le même pour le client et le serveur
    /// tant qu'ils lisent les mêmes données.
    competences: Vec<Entree>,
}

static REGISTRE: LazyLock<Registre> = LazyLock::new(Registre::lire);

/// Les jeux d'armes, compétences, ultimes et passifs des données.
pub fn registre() -> &'static Registre { &REGISTRE }

/// Les fichiers d'un dossier de données : (nom du fichier, contenu).
fn lire_dossier<T>(dossier: &str) -> Vec<(String, T)>
where
    T: Clone + Send + Sync + for<'de> Deserialize<'de> + 'static,
{
    let Ok(ids) = assets::load_rec_dir::<Ron<T>>(dossier) else {
        return Vec::new();
    };
    ids.read()
        .ids()
        .filter_map(|id| match Ron::<T>::load(id) {
            Ok(fichier) => {
                let nom = id.rsplit('.').next().unwrap_or(id).to_string();
                Some((nom, fichier.read().0.clone()))
            },
            Err(e) => {
                error!(?e, "Icebreaker : fichier de compétences illisible : {id}");
                None
            },
        })
        .collect()
}

impl Registre {
    fn lire() -> Self {
        let jeux: BTreeMap<String, JeuArmes> =
            lire_dossier("icebreaker.jeux_armes").into_iter().collect();
        let ultimes: BTreeMap<String, Vec<(String, Competence)>> =
            lire_dossier::<Ultimes>("icebreaker.ultimes")
                .into_iter()
                .map(|(classe, fichier)| (classe, fichier.ultimes))
                .collect();
        let passifs = lire_dossier("icebreaker.passifs").into_iter().collect();
        Self::nouveau(jeux, ultimes, passifs)
    }

    fn nouveau(
        jeux: BTreeMap<String, JeuArmes>,
        ultimes: BTreeMap<String, Vec<(String, Competence)>>,
        passifs: BTreeMap<String, Passifs>,
    ) -> Self {
        let d_arme = jeux.iter().flat_map(|(jeu, def)| {
            def.competences.iter().map(move |(id, competence)| Entree {
                id: id.clone(),
                sorte: Sorte::Arme { jeu: jeu.clone() },
                competence: competence.clone(),
            })
        });
        let de_classe = ultimes.iter().flat_map(|(classe, liste)| {
            liste.iter().map(move |(id, competence)| Entree {
                id: id.clone(),
                sorte: Sorte::Ultime {
                    classe: classe.clone(),
                },
                competence: competence.clone(),
            })
        });
        let mut competences: Vec<Entree> = d_arme.chain(de_classe).collect();
        competences.sort_by(|a, b| a.id.cmp(&b.id));
        for paire in competences.windows(2) {
            if paire[0].id == paire[1].id {
                error!("Icebreaker : deux compétences portent l'id {}", paire[0].id);
            }
        }
        Self {
            jeux,
            ultimes,
            passifs,
            competences,
        }
    }

    /// Le jeu d'armes `id`.
    pub fn jeu(&self, id: &str) -> Option<&JeuArmes> { self.jeux.get(id) }

    /// Les jeux d'armes de `classe` : (id, jeu), par id.
    pub fn jeux_de<'a>(
        &'a self,
        classe: &str,
    ) -> impl Iterator<Item = (&'a str, &'a JeuArmes)> {
        self.jeux
            .iter()
            .filter(move |(_, jeu)| jeu.classe == classe)
            .map(|(id, jeu)| (id.as_str(), jeu))
    }

    /// `classe` est décrite par nos données : ses personnages jouent nos compétences. Les autres
    /// gardent celles de Veloren.
    pub fn classe_geree(&self, classe: &str) -> bool { self.jeux_de(classe).next().is_some() }

    /// Le jeu d'armes par défaut de `classe` (`defaut: true`, sinon le premier).
    pub fn jeu_par_defaut<'a>(&'a self, classe: &str) -> Option<(&'a str, &'a JeuArmes)> {
        self.jeux_de(classe)
            .find(|(_, jeu)| jeu.defaut)
            .or_else(|| self.jeux_de(classe).next())
    }

    /// Les ultimes de `classe`, dans l'ordre d'affichage.
    pub fn ultimes_de(&self, classe: &str) -> &[(String, Competence)] {
        self.ultimes.get(classe).map_or(&[], Vec::as_slice)
    }

    /// Les passifs de `classe`.
    pub fn passifs_de(&self, classe: &str) -> Option<&Passifs> { self.passifs.get(classe) }

    /// Toutes les compétences (d'arme et ultimes), triées par id.
    pub fn competences(&self) -> &[Entree] { &self.competences }

    /// Le numéro de la compétence `id`.
    pub fn numero(&self, id: &str) -> Option<u16> {
        self.competences
            .binary_search_by(|entree| entree.id.as_str().cmp(id))
            .ok()
            .and_then(|i| u16::try_from(i).ok())
    }

    /// La compétence de numéro `numero`.
    pub fn par_numero(&self, numero: u16) -> Option<&Entree> {
        self.competences.get(usize::from(numero))
    }

    /// La compétence `id`.
    pub fn competence(&self, id: &str) -> Option<&Entree> {
        self.numero(id).and_then(|numero| self.par_numero(numero))
    }
}

/// Le jeu d'armes que tient un personnage de `classe` : (id, jeu). `None` si ce qu'il a en mains
/// n'est pas un jeu d'armes de sa classe.
pub fn jeu_en_main(
    classe: &str,
    inventaire: Option<&Inventory>,
) -> Option<(&'static str, &'static JeuArmes)> {
    let outil = |emplacement| {
        inventaire
            .and_then(|inventaire| inventaire.equipped(emplacement))
            .and_then(|objet| match &*objet.kind() {
                ItemKind::Tool(outil) => Some((outil.kind, outil.hands)),
                _ => None,
            })
    };
    let (arme, mains) = outil(EquipSlot::ActiveMainhand)?;
    let gauche = outil(EquipSlot::ActiveOffhand).map(|(arme, _)| arme);
    registre()
        .jeux_de(classe)
        .find(|(_, jeu)| jeu.main_droite == Prise { arme, mains } && jeu.main_gauche == gauche)
}

/// Réglages de la progression (`assets/icebreaker/salle_armes.ron`).
#[derive(Clone, Debug, Deserialize)]
pub struct SalleArmes {
    /// Maîtrise qu'il faut pour le premier point d'arme d'un jeu d'armes.
    pub premier_point: u32,
    /// Chaque point suivant demande ce supplément de maîtrise par rapport au précédent.
    pub pas_point: u32,
    /// Maîtrise gagnée par point d'expérience reçu en expédition.
    pub maitrise_par_experience: f32,
    /// Points d'arme pour débloquer une compétence.
    pub cout_debloquer: u32,
    /// Points d'arme pour passer au niveau 2, 3, 4, 5…
    pub cout_niveaux: Vec<u32>,
    /// Or pour apprendre un jeu d'armes (ses armes sont remises).
    pub prix_jeu_armes: u32,
    /// Or pour apprendre un ultime.
    pub prix_ultime: u32,
    /// Or pour apprendre un passif.
    pub prix_passif: u32,
}

impl Default for SalleArmes {
    fn default() -> Self {
        Self {
            premier_point: 100,
            pas_point: 50,
            maitrise_par_experience: 1.0,
            cout_debloquer: 1,
            cout_niveaux: vec![1, 1, 2, 2],
            prix_jeu_armes: 150,
            prix_ultime: 200,
            prix_passif: 120,
        }
    }
}

impl SalleArmes {
    /// Réglages lus dans `assets/icebreaker/salle_armes.ron` (valeurs par défaut si le fichier
    /// manque).
    pub fn actuel() -> Self {
        Ron::<Self>::load("icebreaker.salle_armes")
            .map_or_else(|_| Self::default(), |s| s.read().0.clone())
    }

    /// (points d'arme gagnés, maîtrise déjà acquise vers le point suivant, maîtrise que demande le
    /// point suivant) pour une maîtrise cumulée de `maitrise`.
    pub fn progression(&self, maitrise: u32) -> (u32, u32, u32) {
        let mut points = 0;
        let mut reste = maitrise;
        let mut cout = self.premier_point.max(1);
        while reste >= cout {
            reste -= cout;
            points += 1;
            cout = cout.saturating_add(self.pas_point);
        }
        (points, reste, cout)
    }

    /// Points d'arme gagnés pour une maîtrise cumulée de `maitrise`.
    pub fn points_gagnes(&self, maitrise: u32) -> u32 { self.progression(maitrise).0 }

    /// Points d'arme pour atteindre `niveau` depuis le niveau d'avant ; `None` si les réglages ne
    /// le prévoient pas.
    pub fn cout_niveau(&self, niveau: u8) -> Option<u32> {
        usize::from(niveau)
            .checked_sub(usize::from(NIVEAU_DEPART) + 1)
            .and_then(|i| self.cout_niveaux.get(i).copied())
    }
}

/// Ce qu'un personnage a acquis dans un jeu d'armes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisJeu {
    /// Maîtrise cumulée, gagnée en expédition avec ce jeu d'armes en mains.
    #[serde(default)]
    pub maitrise: u32,
    /// Points d'arme déjà dépensés à la salle d'armes.
    #[serde(default)]
    pub points_depenses: u32,
    /// Compétences connues : id → niveau.
    #[serde(default)]
    pub competences: BTreeMap<String, u8>,
    /// Compétences placées sur les touches 1 et 2.
    #[serde(default)]
    pub placees: [Option<String>; PLACES_ARME],
}

impl AcquisJeu {
    /// Points d'arme à dépenser.
    pub fn points_disponibles(&self, salle: &SalleArmes) -> u32 {
        salle
            .points_gagnes(self.maitrise)
            .saturating_sub(self.points_depenses)
    }
}

/// Ce qu'un personnage a acquis : jeux d'armes appris, compétences et niveaux, ultimes et
/// passifs connus, et ce qu'il a placé. Sauvé dans sa fiche du registre.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acquis {
    /// Jeux d'armes appris, par id.
    #[serde(default)]
    pub jeux: BTreeMap<String, AcquisJeu>,
    /// Ultimes connus.
    #[serde(default)]
    pub ultimes: Vec<String>,
    /// Ultime placé (touche 3).
    #[serde(default)]
    pub ultime: Option<String>,
    /// Passifs connus.
    #[serde(default)]
    pub passifs: Vec<String>,
    /// Passif actif.
    #[serde(default)]
    pub passif: Option<String>,
}

impl Acquis {
    /// Rien d'acquis : la fiche date d'avant le jalon A, ou la classe n'est pas décrite.
    pub fn est_vide(&self) -> bool { self.jeux.is_empty() }

    /// Le lot d'un personnage de `classe` qui connaît le jeu d'armes `jeu` avec ses compétences
    /// `competences` (les deux premières sont placées), l'ultime et le passif donnés.
    pub fn lot(
        jeu: &str,
        competences: &[&str],
        ultime: Option<&str>,
        passif: Option<&str>,
    ) -> Self {
        let mut placees: [Option<String>; PLACES_ARME] = Default::default();
        for (place, id) in placees.iter_mut().zip(competences) {
            *place = Some((*id).to_string());
        }
        Self {
            jeux: std::iter::once((jeu.to_string(), AcquisJeu {
                competences: competences
                    .iter()
                    .map(|id| ((*id).to_string(), NIVEAU_DEPART))
                    .collect(),
                placees,
                ..Default::default()
            }))
            .collect(),
            ultimes: ultime.iter().map(|id| (*id).to_string()).collect(),
            ultime: ultime.map(str::to_string),
            passifs: passif.iter().map(|id| (*id).to_string()).collect(),
            passif: passif.map(str::to_string),
        }
    }

    /// Le lot par défaut de `classe` : son jeu d'armes par défaut avec ses deux premières
    /// compétences, son premier ultime et son premier passif. Vide si la classe n'est pas décrite.
    pub fn par_defaut(classe: &str) -> Self {
        let registre = registre();
        let Some((jeu, def)) = registre.jeu_par_defaut(classe) else {
            return Self::default();
        };
        let competences: Vec<&str> = def
            .competences
            .iter()
            .take(PLACES_ARME)
            .map(|(id, _)| id.as_str())
            .collect();
        let ultime = registre.ultimes_de(classe).first().map(|(id, _)| id.as_str());
        let passif = registre
            .passifs_de(classe)
            .and_then(|passifs| passifs.passifs.first())
            .map(|(id, _)| id.as_str());
        Self::lot(jeu, &competences, ultime, passif)
    }
}

/// Composant d'un personnage du registre en jeu : sa classe, ce qu'il a acquis (copie de sa fiche,
/// réécrite dans la fiche par le serveur) et ses recharges en cours. Envoyé au seul joueur qui le
/// joue. Un personnage sans ce composant garde les capacités de Veloren.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct IcebreakerCompetences {
    /// Id de sa classe (`classes.ron`).
    pub classe: String,
    pub acquis: Acquis,
    /// Fin de la recharge par numéro de compétence, en secondes du temps du serveur (`Time`).
    #[serde(default)]
    pub recharges: BTreeMap<u16, f64>,
}

impl Component for IcebreakerCompetences {
    type Storage = DerefFlaggedStorage<Self, specs::VecStorage<Self>>;
}

impl IcebreakerCompetences {
    /// Le composant d'un personnage de `classe` qui a acquis `acquis`, sans recharge en cours ;
    /// `None` si nos données ne décrivent pas cette classe.
    pub fn nouveau(classe: &str, acquis: Acquis) -> Option<Self> {
        registre().classe_geree(classe).then(|| Self {
            classe: classe.to_string(),
            acquis,
            recharges: BTreeMap::new(),
        })
    }

    /// La compétence placée sur la touche `touche` (0 et 1 : compétences du jeu d'armes en mains ;
    /// 2 : ultime) : (numéro, niveau). `None` : touche vide (rien de placé, jeu d'armes non appris,
    /// arme étrangère à la classe).
    pub fn sur_touche(&self, touche: usize, inventaire: Option<&Inventory>) -> Option<(u16, u8)> {
        let registre = registre();
        if touche < PLACES_ARME {
            let (jeu, _) = jeu_en_main(&self.classe, inventaire)?;
            let acquis = self.acquis.jeux.get(jeu)?;
            let id = acquis.placees[touche].as_deref()?;
            let niveau = *acquis.competences.get(id)?;
            let numero = registre.numero(id)?;
            // Une compétence d'une autre arme ne se lance pas avec celle-ci.
            let du_jeu = matches!(&registre.par_numero(numero)?.sorte, Sorte::Arme { jeu: j } if j == jeu);
            du_jeu.then_some((numero, niveau))
        } else if touche == TOUCHE_ULTIME {
            let id = self.acquis.ultime.as_deref()?;
            let numero = registre.numero(id)?;
            let de_la_classe = matches!(
                &registre.par_numero(numero)?.sorte,
                Sorte::Ultime { classe } if *classe == self.classe
            );
            de_la_classe.then_some((numero, NIVEAU_DEPART))
        } else {
            None
        }
    }

    /// Secondes de recharge qui restent à la compétence `numero` au temps `maintenant` (secondes
    /// du temps du serveur) ; `None` : prête.
    pub fn reste_recharge(&self, numero: u16, maintenant: f64) -> Option<f64> {
        self.recharges
            .get(&numero)
            .map(|fin| fin - maintenant)
            .filter(|reste| *reste > 0.0)
    }

    /// Le jeu d'armes en mains gagne de la maîtrise pour `experience` points d'expérience reçus
    /// (un ennemi tué en expédition) : (id du jeu, maîtrise gagnée, points d'arme gagnés). `None`
    /// si ce qu'on tient n'est pas un jeu d'armes appris : on ne progresse que dans ce qu'on sait
    /// manier.
    pub fn gagner_maitrise(
        &mut self,
        inventaire: Option<&Inventory>,
        experience: f32,
        salle: &SalleArmes,
    ) -> Option<(&'static str, u32, u32)> {
        let (jeu, _) = jeu_en_main(&self.classe, inventaire)?;
        let acquis = self.acquis.jeux.get_mut(jeu)?;
        let gain = (experience * salle.maitrise_par_experience).ceil().max(0.0) as u32;
        if gain == 0 {
            return None;
        }
        let avant = salle.points_gagnes(acquis.maitrise);
        acquis.maitrise = acquis.maitrise.saturating_add(gain);
        Some((jeu, gain, salle.points_gagnes(acquis.maitrise) - avant))
    }
}

/// La maîtrise gagnée dans un jeu d'armes depuis un état antérieur (bilan de retour).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progres {
    /// Id du jeu d'armes.
    pub jeu: String,
    /// Maîtrise gagnée.
    pub gain: u32,
    /// Points d'arme à dépenser dans ce jeu d'armes.
    pub points: u32,
}

impl Acquis {
    /// Ce que chaque jeu d'armes a gagné en maîtrise depuis `avant`, par id ; seuls ceux qui ont
    /// progressé.
    pub fn progres_depuis(&self, avant: &Acquis, salle: &SalleArmes) -> Vec<Progres> {
        self.jeux
            .iter()
            .filter_map(|(jeu, acquis)| {
                let gain = acquis
                    .maitrise
                    .saturating_sub(avant.jeux.get(jeu).map_or(0, |a| a.maitrise));
                (gain > 0).then(|| Progres {
                    jeu: jeu.clone(),
                    gain,
                    points: acquis.points_disponibles(salle),
                })
            })
            .collect()
    }
}

/// Composant d'un personnage du registre en jeu : ses effets permanents, l'inné de sa classe puis
/// son passif actif. Envoyé à tous les joueurs : le client recalcule lui aussi les PV max de ceux
/// qu'il voit. Appliqué à chaque tic par le système des effets (`common/systems/src/buff.rs`),
/// comme un effet qui ne finit pas et que rien ne dissipe.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct IcebreakerPassif {
    pub effets: Vec<BuffEffect>,
}

impl Component for IcebreakerPassif {
    type Storage = DerefFlaggedStorage<Self, specs::VecStorage<Self>>;
}

impl IcebreakerPassif {
    /// Les effets permanents du personnage : l'inné de sa classe, et son passif actif s'il le
    /// connaît.
    pub fn de(competences: &IcebreakerCompetences) -> Self {
        let Some(passifs) = registre().passifs_de(&competences.classe) else {
            return Self::default();
        };
        let acquis = &competences.acquis;
        let actif = acquis
            .passif
            .as_deref()
            .filter(|id| acquis.passifs.iter().any(|connu| connu == id))
            .and_then(|id| passifs.passifs.iter().find(|(passif, _)| passif == id));
        Self {
            effets: passifs
                .inne
                .iter()
                .chain(actif.iter().flat_map(|(_, passif)| &passif.effets))
                .cloned()
                .collect(),
        }
    }

    /// Ce que ces effets font aux PV max, pour entrer en jeu avec tous ses PV.
    pub fn pv_max(&self) -> StatsModifier {
        let mut pv = StatsModifier::default();
        for effet in &self.effets {
            if let BuffEffect::MaxHealthModifier { value, kind } = effet {
                match kind {
                    ModifierKind::Additive => pv.add_mod += value,
                    ModifierKind::Multiplicative => pv.mult_mod *= value,
                }
            }
        }
        pv
    }
}

/// Un effet qu'un passif ou un inné peut porter : un réglage permanent des stats. Les effets qui
/// agissent dans le temps (dégâts, soin, endurance, combo), changent de corps ou désignent une
/// cible n'ont pas de sens sans durée ni lanceur.
pub fn effet_permanent(effet: &BuffEffect) -> bool {
    !matches!(
        effet,
        BuffEffect::HealthChangeOverTime { .. }
            | BuffEffect::EnergyChangeOverTime { .. }
            | BuffEffect::ComboChangeOverTime { .. }
            | BuffEffect::MaxHealthChangeOverTime { .. }
            | BuffEffect::BodyChange(_)
            | BuffEffect::MarkEntity(_)
            | BuffEffect::DisableAuxiliaryAbilities
    )
}

/// Id de la capacité de Veloren dont l'interface reprend l'icône pour la compétence `numero`.
pub fn icone_de(numero: u16) -> Option<&'static str> {
    registre()
        .par_numero(numero)
        .map(|entree| entree.competence.id_icone())
}

/// Id de la capacité de Veloren dont le client reprend l'animation pour la compétence `numero`
/// lancée au niveau `niveau` (porté par l'index de contexte de la capacité lancée).
pub fn animation_de(numero: u16, niveau: Option<ContextualIndex>) -> Option<&'static str> {
    let niveau = niveau.map_or(NIVEAU_DEPART, |n| u8::try_from(n.0).unwrap_or(u8::MAX));
    registre()
        .par_numero(numero)
        .map(|entree| entree.competence.effectif(niveau).animation)
}

/// Ce que lance la touche `entree` pour un personnage qui a nos compétences. `None` : ce n'est pas
/// à nous de répondre (pas de composant, attaque de base, garde, roulade, planeur en main) :
/// Veloren s'en charge. `Some(None)` : rien ne part (touche vide, compétence en recharge).
/// Appelé par `states::utils::handle_ability`, chez le client (prédiction) comme sur le serveur.
pub fn resoudre(
    data: &JoinData<'_>,
    entree: AbilityInput,
) -> Option<Option<(CharacterAbility, bool, SpecifiedAbility)>> {
    let competences = data.icebreaker?;
    let AbilityInput::Auxiliary(touche) = entree else {
        return None;
    };
    if data.character.is_glide_wielded() {
        return None;
    }
    Some(lancer(competences, touche, data))
}

fn lancer(
    competences: &IcebreakerCompetences,
    touche: usize,
    data: &JoinData<'_>,
) -> Option<(CharacterAbility, bool, SpecifiedAbility)> {
    if data.stats.disable_auxiliary_abilities {
        return None;
    }
    let (numero, niveau) = competences.sur_touche(touche, data.inventory)?;
    if competences.reste_recharge(numero, data.time.0).is_some() {
        return None;
    }
    let effectif = registre().par_numero(numero)?.competence.effectif(niveau);
    let capacite = Ron::<CharacterAbility>::load(effectif.capacite)
        .ok()?
        .cloned()
        .into_inner();
    // Comme pour le jeu de capacités d'un objet, les stats de l'arme de main droite s'appliquent ;
    // le niveau de la compétence vient par-dessus.
    let arme = data
        .inventory
        .and_then(|inventaire| inventaire.equipped(EquipSlot::ActiveMainhand))
        .and_then(|objet| match &*objet.kind() {
            ItemKind::Tool(outil) => Some(outil.stats(objet.stats_durability_multiplier())),
            _ => None,
        })
        .unwrap_or_else(Stats::one);
    Some((
        capacite.adjusted_by_stats(arme * effectif.stats),
        false,
        SpecifiedAbility {
            ability: Ability::Icebreaker(numero),
            context_index: Some(ContextualIndex(usize::from(niveau))),
        },
    ))
}

/// La compétence que vient de lancer `data.entity` part en recharge, pour la durée de son niveau.
/// Seul le serveur l'arme ; le client lit la valeur qu'il reçoit. S'il l'armait aussi, un lancer que le
/// serveur refuse lui laisserait une recharge que rien ne viendrait corriger.
pub fn armer_recharge(data: &JoinData<'_>, lancee: &SpecifiedAbility) {
    let Ability::Icebreaker(numero) = lancee.ability else {
        return;
    };
    let niveau = lancee
        .context_index
        .map_or(NIVEAU_DEPART, |n| u8::try_from(n.0).unwrap_or(u8::MAX));
    let Some(entree) = registre().par_numero(numero) else {
        return;
    };
    let maintenant = data.time.0;
    let fin = maintenant + f64::from(entree.competence.effectif(niveau).recharge);
    let entity = data.entity;
    data.updater.exec_mut(move |world| {
        if *world.read_resource::<GameMode>() == GameMode::Client {
            return;
        }
        if let Some(mut competences) = world
            .write_storage::<IcebreakerCompetences>()
            .get_mut(entity)
        {
            // Les recharges finies n'ont plus à voyager vers le client.
            competences.recharges.retain(|_, fin| *fin > maintenant);
            competences.recharges.insert(numero, fin);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::{Item, inventory::loadout_builder::LoadoutBuilder};

    fn inventaire(armes: &(String, Option<String>)) -> Inventory {
        let loadout = LoadoutBuilder::empty()
            .active_mainhand(Some(Item::new_from_asset_expect(&armes.0)))
            .active_offhand(armes.1.as_deref().map(Item::new_from_asset_expect))
            .build();
        Inventory::with_loadout_humanoid(loadout)
    }

    fn competence(niveaux: Vec<Niveau>) -> Competence {
        Competence {
            nom: "Essai".into(),
            description: String::new(),
            capacite: "common.abilities.sword.heavy_sweep".into(),
            animation: None,
            icone: None,
            recharge: 10.0,
            niveaux,
        }
    }

    #[test]
    fn les_donnees_se_chargent() {
        let registre = registre();
        // Le guerrier a ses quatre jeux d'armes, de quatre compétences chacun.
        let jeux: Vec<_> = registre.jeux_de("guerrier").collect();
        assert_eq!(jeux.len(), 4, "{:?}", jeux.iter().map(|(id, _)| id).collect::<Vec<_>>());
        for (id, jeu) in &jeux {
            assert!(jeu.competences.len() >= 4, "{id}");
        }
        assert!(registre.ultimes_de("guerrier").len() >= 3);
        assert!(registre.passifs_de("guerrier").is_some_and(|p| p.passifs.len() >= 3));
        // Un inné ou un passif ne porte que des réglages permanents des stats.
        for classe in ["guerrier"] {
            let passifs = registre.passifs_de(classe).expect("passifs");
            let effets = passifs
                .inne
                .iter()
                .chain(passifs.passifs.iter().flat_map(|(_, passif)| &passif.effets));
            for effet in effets {
                assert!(effet_permanent(effet), "{classe} : {effet:?}");
            }
        }

        // Les ids sont uniques : le numéro d'une compétence en dépend.
        let ids: std::collections::BTreeSet<_> =
            registre.competences().iter().map(|e| &e.id).collect();
        assert_eq!(ids.len(), registre.competences().len());
        for (numero, entree) in registre.competences().iter().enumerate() {
            assert_eq!(registre.numero(&entree.id), Some(numero as u16));
        }

        // Chaque compétence, à chaque niveau, lance une capacité qui se charge, avec une animation
        // que Veloren connaît et une recharge.
        for entree in registre.competences() {
            let competence = &entree.competence;
            for niveau in NIVEAU_DEPART..=competence.niveau_max() {
                let effectif = competence.effectif(niveau);
                assert!(
                    Ron::<CharacterAbility>::load(effectif.capacite).is_ok(),
                    "{} niveau {niveau} : capacité {} illisible",
                    entree.id,
                    effectif.capacite
                );
                assert!(
                    effectif.animation.starts_with("common.abilities."),
                    "{} niveau {niveau} : animation {}",
                    entree.id,
                    effectif.animation
                );
                assert!(effectif.recharge > 0.0, "{}", entree.id);
            }
            assert!(competence.id_icone().starts_with("common.abilities."), "{}", entree.id);
        }

        // Par classe : un seul jeu par défaut, et le lot par défaut ne cite que ce qui existe.
        for classe in ["guerrier"] {
            assert_eq!(registre.jeux_de(classe).filter(|(_, jeu)| jeu.defaut).count(), 1, "{classe}");
            let lot = Acquis::par_defaut(classe);
            assert!(!lot.est_vide(), "{classe}");
            for (jeu, acquis) in &lot.jeux {
                assert!(registre.jeu(jeu).is_some(), "{jeu}");
                assert_eq!(acquis.competences.len(), PLACES_ARME);
                for id in acquis.placees.iter().flatten() {
                    assert_eq!(
                        registre.competence(id).map(|e| &e.sorte),
                        Some(&Sorte::Arme { jeu: jeu.clone() })
                    );
                }
            }
            assert!(lot.ultime.as_deref().is_some_and(|id| registre.competence(id).is_some()));
            assert!(lot.passif.is_some());
        }
        assert!(Acquis::par_defaut("classe inconnue").est_vide());
    }

    #[test]
    fn le_jeu_suit_les_armes_en_mains() {
        let registre = registre();
        for (id, jeu) in registre.jeux_de("guerrier") {
            // Les armes remises avec le jeu sont bien celles qu'il attend.
            let inventaire = inventaire(&jeu.armes);
            assert_eq!(
                jeu_en_main("guerrier", Some(&inventaire)).map(|(id, _)| id),
                Some(id),
                "{id}"
            );
            // Une autre classe ne tire rien de ces armes.
            assert!(jeu_en_main("mage", Some(&inventaire)).is_none(), "{id}");
        }
        // Mains vides, pas d'inventaire : pas de jeu d'armes.
        let vide = Inventory::with_loadout_humanoid(LoadoutBuilder::empty().build());
        assert!(jeu_en_main("guerrier", Some(&vide)).is_none());
        assert!(jeu_en_main("guerrier", None).is_none());
        // Une épée à une main sans bouclier n'est pas « épée + bouclier ».
        let epee_seule = inventaire(&("common.items.weapons.sword_1h.starter".to_string(), None));
        assert!(jeu_en_main("guerrier", Some(&epee_seule)).is_none());
    }

    #[test]
    fn un_niveau_change_les_chiffres_puis_la_capacite() {
        let competence = competence(vec![
            Niveau {
                puissance: 1.2,
                ..Default::default()
            },
            Niveau {
                puissance: 1.2,
                capacite: Some("icebreaker.abilities.essai_3".into()),
                ..Default::default()
            },
            Niveau {
                puissance: 1.4,
                recharge: 0.8,
                portee: 1.1,
                ..Default::default()
            },
        ]);
        assert_eq!(competence.niveau_max(), 4);

        let niveau_1 = competence.effectif(1);
        assert_eq!(niveau_1.capacite, "common.abilities.sword.heavy_sweep");
        assert_eq!(niveau_1.stats, Stats::one());
        assert_eq!(niveau_1.recharge, 10.0);
        // Le niveau 0 n'existe pas : c'est le niveau 1.
        assert_eq!(competence.effectif(0), niveau_1);

        let niveau_2 = competence.effectif(2);
        assert_eq!(niveau_2.capacite, niveau_1.capacite);
        assert_eq!(niveau_2.stats.power, 1.2);
        assert_eq!(niveau_2.stats.range, 1.0);

        // Le niveau 3 transforme la compétence ; l'animation reste celle de la capacité d'origine.
        let niveau_3 = competence.effectif(3);
        assert_eq!(niveau_3.capacite, "icebreaker.abilities.essai_3");
        assert_eq!(niveau_3.animation, "common.abilities.sword.heavy_sweep");

        // Le niveau 4 garde la transformation et écrit ses propres chiffres.
        let niveau_4 = competence.effectif(4);
        assert_eq!(niveau_4.capacite, "icebreaker.abilities.essai_3");
        assert_eq!(niveau_4.stats.power, 1.4);
        assert_eq!(niveau_4.stats.range, 1.1);
        assert_eq!(niveau_4.recharge, 8.0);
        // Au-delà du dernier niveau : le dernier niveau.
        assert_eq!(competence.effectif(9), niveau_4);
    }

    #[test]
    fn les_points_suivent_la_maitrise() {
        let salle = SalleArmes {
            premier_point: 100,
            pas_point: 50,
            cout_niveaux: vec![1, 1, 2, 2],
            ..Default::default()
        };
        // Premier point à 100, le deuxième 150 plus loin, le troisième 200 plus loin.
        assert_eq!(salle.progression(0), (0, 0, 100));
        assert_eq!(salle.progression(99), (0, 99, 100));
        assert_eq!(salle.progression(100), (1, 0, 150));
        assert_eq!(salle.progression(249), (1, 149, 150));
        assert_eq!(salle.progression(250), (2, 0, 200));
        assert_eq!(salle.points_gagnes(450), 3);

        let jeu = AcquisJeu {
            maitrise: 250,
            points_depenses: 1,
            ..Default::default()
        };
        assert_eq!(jeu.points_disponibles(&salle), 1);
        // On ne tombe jamais sous zéro si les réglages durcissent après coup.
        let depense = AcquisJeu {
            maitrise: 0,
            points_depenses: 3,
            ..Default::default()
        };
        assert_eq!(depense.points_disponibles(&salle), 0);

        assert_eq!(salle.cout_niveau(1), None);
        assert_eq!(salle.cout_niveau(2), Some(1));
        assert_eq!(salle.cout_niveau(5), Some(2));
        assert_eq!(salle.cout_niveau(6), None);
        // Les réglages du dépôt se lisent.
        assert!(SalleArmes::actuel().premier_point > 0);
    }

    #[test]
    fn un_lot_place_ses_deux_premieres_competences() {
        let lot = Acquis::lot("grande_epee", &["a", "b", "c"], Some("u"), None);
        let jeu = &lot.jeux["grande_epee"];
        assert_eq!(jeu.competences.len(), 3);
        assert_eq!(jeu.placees, [Some("a".to_string()), Some("b".to_string())]);
        assert_eq!(lot.ultime.as_deref(), Some("u"));
        assert_eq!(lot.ultimes, vec!["u".to_string()]);
        assert!(lot.passifs.is_empty() && lot.passif.is_none());
        // Une fiche d'avant le jalon A se relit comme « rien d'acquis ».
        let ancienne: Acquis = ron::from_str("()").unwrap();
        assert!(ancienne.est_vide());
    }

    #[test]
    fn les_touches_suivent_l_arme_en_mains() {
        let registre = registre();
        let numero = |id: &str| registre.numero(id).unwrap_or_else(|| panic!("{id}"));
        let armes = |jeu: &str| inventaire(&registre.jeu(jeu).unwrap().armes);
        let mut guerrier =
            IcebreakerCompetences::nouveau("guerrier", Acquis::par_defaut("guerrier")).unwrap();
        // Niveau 3 sur la touche 2 : le niveau suit la compétence.
        guerrier
            .acquis
            .jeux
            .get_mut("grande_epee")
            .unwrap()
            .competences
            .insert("tourbillon".into(), 3);

        // Grande épée en mains : ses deux compétences placées, puis l'ultime.
        let grande_epee = armes("grande_epee");
        assert_eq!(
            guerrier.sur_touche(0, Some(&grande_epee)),
            Some((numero("balayage_lourd"), 1))
        );
        assert_eq!(
            guerrier.sur_touche(1, Some(&grande_epee)),
            Some((numero("tourbillon"), 3))
        );
        assert_eq!(
            guerrier.sur_touche(TOUCHE_ULTIME, Some(&grande_epee)),
            Some((numero("fureur"), NIVEAU_DEPART))
        );
        assert_eq!(guerrier.sur_touche(TOUCHES, Some(&grande_epee)), None);

        // Un marteau jamais appris : touches 1 et 2 vides, l'ultime reste.
        let marteau = armes("marteau");
        assert_eq!(guerrier.sur_touche(0, Some(&marteau)), None);
        assert_eq!(guerrier.sur_touche(1, Some(&marteau)), None);
        assert_eq!(
            guerrier.sur_touche(TOUCHE_ULTIME, Some(&marteau)),
            Some((numero("fureur"), NIVEAU_DEPART))
        );
        // Mains vides : pareil.
        assert_eq!(guerrier.sur_touche(0, None), None);
        assert!(guerrier.sur_touche(TOUCHE_ULTIME, None).is_some());

        // Le marteau appris, avec une seule compétence placée, sur la touche 2.
        guerrier.acquis.jeux.insert("marteau".into(), AcquisJeu {
            competences: [("moulinet".to_string(), 2)].into_iter().collect(),
            placees: [None, Some("moulinet".into())],
            ..Default::default()
        });
        assert_eq!(guerrier.sur_touche(0, Some(&marteau)), None);
        assert_eq!(
            guerrier.sur_touche(1, Some(&marteau)),
            Some((numero("moulinet"), 2))
        );
        // Une compétence d'une autre arme placée par erreur ne part pas.
        guerrier.acquis.jeux.get_mut("marteau").unwrap().placees[0] = Some("tourbillon".into());
        guerrier
            .acquis
            .jeux
            .get_mut("marteau")
            .unwrap()
            .competences
            .insert("tourbillon".into(), 1);
        assert_eq!(guerrier.sur_touche(0, Some(&marteau)), None);

        // L'animation et l'icône d'une compétence sont celles d'une capacité de Veloren.
        assert_eq!(
            animation_de(numero("tourbillon"), Some(ContextualIndex(3))),
            Some("common.abilities.sword.cleaving_whirlwind_slice")
        );
        assert_eq!(icone_de(numero("fureur")), Some("common.abilities.axe.furor"));
        assert_eq!(animation_de(u16::MAX, None), None);
    }

    #[test]
    fn une_recharge_se_compte_en_temps_du_serveur() {
        let mut guerrier =
            IcebreakerCompetences::nouveau("guerrier", Acquis::par_defaut("guerrier")).unwrap();
        assert_eq!(guerrier.reste_recharge(4, 10.0), None);
        guerrier.recharges.insert(4, 16.0);
        assert_eq!(guerrier.reste_recharge(4, 10.0), Some(6.0));
        assert_eq!(guerrier.reste_recharge(4, 16.0), None);
        assert_eq!(guerrier.reste_recharge(4, 20.0), None);
        assert_eq!(guerrier.reste_recharge(5, 10.0), None);
    }

    #[test]
    fn le_passif_s_ajoute_a_l_inne_de_la_classe() {
        let passifs = registre().passifs_de("guerrier").expect("passifs du guerrier");
        let mut guerrier =
            IcebreakerCompetences::nouveau("guerrier", Acquis::par_defaut("guerrier")).unwrap();
        // Le lot par défaut : l'inné, puis le premier passif.
        let (actif, premier) = &passifs.passifs[0];
        assert_eq!(guerrier.acquis.passif.as_deref(), Some(actif.as_str()));
        let effets = IcebreakerPassif::de(&guerrier);
        assert_eq!(effets.effets.len(), passifs.inne.len() + premier.effets.len());
        assert_eq!(effets.effets[..passifs.inne.len()], passifs.inne[..]);
        // Les PV max : ce que disent les données (inné × 1,25, robuste × 1,15).
        let pv = effets.pv_max();
        assert!((pv.mult_mod - 1.25 * 1.15).abs() < 1e-4 && pv.add_mod == 0.0, "{pv:?}");

        // Un passif qu'il ne connaît pas, ou aucun : l'inné seul.
        guerrier.acquis.passif = Some(passifs.passifs[1].0.clone());
        assert_eq!(IcebreakerPassif::de(&guerrier).effets, passifs.inne);
        guerrier.acquis.passif = None;
        assert_eq!(IcebreakerPassif::de(&guerrier).effets, passifs.inne);
        // Une classe sans données : rien.
        guerrier.classe = "classe inconnue".into();
        assert!(IcebreakerPassif::de(&guerrier).effets.is_empty());
        // Les effets dans le temps ne sont pas des passifs.
        assert!(effet_permanent(&BuffEffect::AttackDamage(1.1)));
        assert!(!effet_permanent(&BuffEffect::ComboChangeOverTime {
            rate: 1.0,
            tick_dur: crate::resources::Secs(1.0),
        }));
    }

    #[test]
    fn la_maitrise_va_au_jeu_d_armes_en_mains() {
        let registre = registre();
        let armes = |jeu: &str| inventaire(&registre.jeu(jeu).unwrap().armes);
        let salle = SalleArmes {
            premier_point: 100,
            pas_point: 50,
            maitrise_par_experience: 2.0,
            ..Default::default()
        };
        let depart = Acquis::par_defaut("guerrier");
        let mut guerrier = IcebreakerCompetences::nouveau("guerrier", depart.clone()).unwrap();
        let grande_epee = armes("grande_epee");

        // 30 d'expérience, deux de maîtrise par point : 60, pas encore de point d'arme.
        assert_eq!(
            guerrier.gagner_maitrise(Some(&grande_epee), 30.0, &salle),
            Some(("grande_epee", 60, 0))
        );
        // 20,2 de plus : arrondi au-dessus (41), le premier point tombe à 100.
        assert_eq!(
            guerrier.gagner_maitrise(Some(&grande_epee), 20.2, &salle),
            Some(("grande_epee", 41, 1))
        );
        assert_eq!(guerrier.acquis.jeux["grande_epee"].maitrise, 101);
        // Rien pour une expérience nulle, une arme non apprise ou des mains vides.
        assert_eq!(guerrier.gagner_maitrise(Some(&grande_epee), 0.0, &salle), None);
        assert_eq!(guerrier.gagner_maitrise(Some(&armes("marteau")), 30.0, &salle), None);
        assert_eq!(guerrier.gagner_maitrise(None, 30.0, &salle), None);
        assert!(!guerrier.acquis.jeux.contains_key("marteau"));

        // Le bilan : ce qui a été gagné depuis le départ, et les points à dépenser.
        assert_eq!(guerrier.acquis.progres_depuis(&depart, &salle), vec![Progres {
            jeu: "grande_epee".into(),
            gain: 101,
            points: 1,
        }]);
        assert!(depart.progres_depuis(&depart, &salle).is_empty());
        // Un point déjà dépensé n'est plus à dépenser.
        guerrier.acquis.jeux.get_mut("grande_epee").unwrap().points_depenses = 1;
        assert_eq!(guerrier.acquis.progres_depuis(&depart, &salle)[0].points, 0);
    }
}
