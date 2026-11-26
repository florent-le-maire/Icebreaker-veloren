//! Le cerveau d'un ennemi : son état, sa cible, et ce qu'il veut faire à ce tick.
//!
//! Il ne connaît que ce que son [`Observation`] contient : ceux qu'il perçoit à ce tick. D'une cible
//! perdue de vue, il ne garde que la dernière position où il l'a perçue.

use crate::{
    combat::{self, Situation},
    evenements::{Evenement, Message},
    profil::Profil,
    reactions::{
        Reaction,
        coince::Forcer,
        defense::{Defense, Geste},
        prevenir::{self, Etape, Prevenir},
    },
    tirage::tirer,
};
use vek::Vec3;

/// Une créature ou un joueur, vu du cerveau (le serveur y met l'`Uid` de Veloren).
pub type Id = u64;

/// Un adversaire que l'ennemi perçoit à ce tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Percu {
    pub id: Id,
    pub pos: Vec3<f32>,
    /// Il sait qui c'est (sinon : un soupçon, `perception::Niveau`).
    pub net: bool,
    pub a_terre: bool,
    /// Il a frappé l'ennemi il y a tant de secondes (rien : pas lui, ou jamais).
    pub l_a_frappe_depuis: Option<f64>,
    /// Combien d'ennemis le visent déjà (l'observateur compris, s'il le vise).
    pub dessus: u8,
    /// La vitesse à laquelle il se rapproche de l'ennemi (blocs par seconde ; négative : il s'éloigne).
    pub approche: f32,
    /// Il arme un coup contre l'ennemi, qui tombera dans tant de secondes (0 : le coup est en train de
    /// tomber ; rien : il n'arme rien, ou pas contre lui).
    pub frappe_dans: Option<f32>,
}

/// Un des siens, dont il connaît le poste (IA.4).
#[derive(Clone, Debug, PartialEq)]
pub struct Allie {
    pub id: Id,
    pub pos: Vec3<f32>,
    /// Il sait déjà qu'il se passe quelque chose (il enquête, se bat, fuit, ou vient d'être prévenu).
    pub alerte: bool,
    /// Il est parti prévenir.
    pub messager: bool,
}

/// Ce qu'un ennemi sait à ce tick.
#[derive(Clone, Debug, Default)]
pub struct Observation {
    /// L'heure du serveur, en secondes.
    pub temps: f64,
    /// Lui-même (sert à décaler ses horloges de celles de ses voisins).
    pub moi: Id,
    pub pos: Vec3<f32>,
    /// Part de ses PV, de 0 à 1.
    pub pv: f32,
    /// Membres de sa race autour de lui, lui compris.
    pub meute: usize,
    /// Les adversaires qu'il perçoit.
    pub percus: Vec<Percu>,
    /// Sa cible du tick précédent est-elle encore en vie (et attaquable) ?
    pub cible_en_vie: bool,
    /// Les siens autour de lui (`voisinage`), sans lui.
    pub allies: Vec<Allie>,
    /// Un nombre tiré entre 0 et 1 à ce tick, pour ses tirages.
    pub hasard: f32,
    /// Il est en train d'attaquer (un coup, un tir, un sort en cours).
    pub attaque: bool,
    /// Les sorts qu'il a et peut payer à ce tick (le nom de `combat::Sort`) ; vide s'il n'a pas de table.
    pub sorts: Vec<String>,
    /// Il a de quoi parer (une garde à son arme).
    pub peut_parer: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etat {
    Repos,
    /// Il a un soupçon, ou il est venu voir un endroit (« ? »).
    Enquete,
    Combat,
    Fuite,
    /// Il court prévenir un allié.
    Prevenir,
    /// Coincé, il se tapit sur place.
    Tapi,
    /// Coincé, il force le passage.
    Force,
    /// Son chef est tombé : il fuit sans réfléchir.
    Deroute,
}

/// Ce que l'ennemi veut faire ; le serveur l'exécute.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intention {
    /// Vaquer : l'errance et les ordres de Veloren.
    Repos,
    /// Se battre contre cette cible, qu'il perçoit.
    Combattre(Id),
    /// S'éloigner de cet endroit.
    Fuir(Vec3<f32>),
    /// Courir à cet endroit.
    AllerVoir(Vec3<f32>),
    /// Rester sur place et regarder vers cet endroit.
    Regarder(Vec3<f32>),
    /// Rentrer à son poste.
    Rentrer(Vec3<f32>),
    /// Courir prévenir l'allié qui est là.
    Prevenir(Vec3<f32>),
    /// Se tapir sur place, face à cet endroit.
    SeTapir(Vec3<f32>),
    /// Foncer à cet endroit, à travers ce qui le menace.
    Forcer(Vec3<f32>),
}

/// Distance à laquelle un endroit est atteint quand le profil ne dit rien (`arrivee`).
const ARRIVEE: f32 = 2.0;
/// La fouille du regard : il regarde un point à `REGARD` blocs, qui tourne de `TOUR` radians par seconde.
const REGARD: f32 = 4.0;
const TOUR: f32 = 1.0;
/// Le bruit de la mort de sa victime : moins de `VICTIME` secondes après, à moins de `VICTIME_PRES` blocs
/// de là où il l'a vue.
const VICTIME: f64 = 1.0;
const VICTIME_PRES: f32 = 6.0;
/// Secondes pendant lesquelles celui qu'un allié vient de prévenir compte comme alerté.
const PREVENU: f64 = 30.0;
/// Secondes pendant lesquelles un mage demande un sort qui ne part pas, avant d'en choisir un autre.
const SORT_ATTENTE: f64 = 1.5;
/// Un fuyard qui a fait moins de tant de blocs pendant `coince_temps` n'avance pas.
const COINCE_PAS: f32 = 1.5;

/// Un endroit où il est venu voir.
#[derive(Clone, Copy, Debug)]
struct Enquete {
    lieu: Vec3<f32>,
    depuis: f64,
    /// Arrivé sur place à cette heure.
    sur_place: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Cerveau {
    etat: Etat,
    cible: Option<Id>,
    /// Quand il a choisi sa cible pour la dernière fois.
    choisie_a: f64,
    /// Quand et où il a perçu sa cible pour la dernière fois.
    percue_a: f64,
    percue_en: Vec3<f32>,
    /// Ceux qu'il soupçonne, et depuis quand sans interruption.
    soupcons: Vec<(Id, f64)>,
    enquete: Option<Enquete>,
    /// Sa place : où il rentre après un combat ou une enquête.
    poste: Option<Vec3<f32>>,
    /// Il doit rentrer à son poste.
    rentre: bool,
    /// Le dernier bruit entendu : où, quand.
    bruit: Option<(Vec3<f32>, f64)>,
    /// Sa victime vient de mourir là, à cette heure : le bruit de cette mort n'est pas à aller voir.
    victime: Option<(Vec3<f32>, f64)>,
    /// Il court prévenir un allié.
    prevenir: Option<Prevenir>,
    /// Un des siens vient de tomber là, à cette heure : à lui de tirer sa réaction.
    allie_tombe: Option<(Vec3<f32>, f64)>,
    /// Quand il a tiré chaque événement pour la dernière fois.
    tirages: Vec<(Evenement, f64)>,
    /// Ce qu'il a crié depuis la dernière fois qu'on le lui a demandé (`cris`).
    cris: Vec<Message>,
    /// Prévenu par un allié à cette heure.
    prevenu_a: Option<f64>,
    /// Son rôle et son cran d'habileté (`Profils::profil`), posés à son apparition.
    fiche: (String, String),
    /// Quand il a repéré l'intrus qu'il combat (son délai de réaction part de là).
    engage_a: f64,
    /// Il attaque sans s'arrêter depuis cette heure.
    attaque_depuis: Option<f64>,
    /// Son temps mort après une attaque dure jusqu'à cette heure.
    pause_jusqu_a: f64,
    /// En combat, il se retient de frapper à ce tick.
    retenu: bool,
    /// Le sort qu'il a choisi de lancer (mage), et depuis quand il le demande.
    sort: Option<String>,
    sort_depuis: f64,
    /// Il lâche à ce tick le sort qu'il tenait.
    lache: bool,
    /// Son dernier lancer de chaque sort.
    lances: Vec<(String, f64)>,
    /// Sa réponse au coup qu'on arme contre lui en ce moment.
    defense: Option<Defense>,
    /// Le geste de défense à faire à ce tick.
    geste: Option<Geste>,
    /// En fuite : où il était, et quand, la dernière fois qu'il a regardé s'il s'éloignait.
    repere: Option<(Vec3<f32>, f64)>,
    /// Sa dernière réaction de coincé (rien : jamais).
    coince_a: Option<f64>,
    /// Il force le passage.
    forcer: Option<Forcer>,
    /// Il s'est tapi à cette heure.
    tapi: Option<f64>,
    /// Sa part de PV au tick précédent (un coup la fait baisser).
    pv_avant: Option<f32>,
    /// Son dernier appel à l'aide.
    appel_a: Option<f64>,
    /// Un appel à l'aide entendu, pas encore traité : l'agresseur, où il est, quand.
    appel_recu: Option<(Id, Vec3<f32>, f64)>,
    /// Il vient défendre un allié : l'agresseur, où on le lui a signalé, jusqu'à quelle heure.
    renfort: Option<(Id, Vec3<f32>, f64)>,
    /// Son chef vient de tomber là : à lui de tirer sa réaction.
    chef_tombe: Option<Vec3<f32>>,
    /// En déroute : ce qu'il fuit, jusqu'à quelle heure.
    deroute: Option<(Vec3<f32>, f64)>,
}

impl Default for Cerveau {
    fn default() -> Self {
        Self {
            etat: Etat::Repos,
            cible: None,
            choisie_a: 0.0,
            percue_a: 0.0,
            percue_en: Vec3::zero(),
            soupcons: Vec::new(),
            enquete: None,
            poste: None,
            rentre: false,
            bruit: None,
            victime: None,
            prevenir: None,
            allie_tombe: None,
            tirages: Vec::new(),
            cris: Vec::new(),
            prevenu_a: None,
            fiche: Default::default(),
            engage_a: 0.0,
            attaque_depuis: None,
            pause_jusqu_a: 0.0,
            retenu: false,
            sort: None,
            sort_depuis: 0.0,
            lache: false,
            lances: Vec::new(),
            defense: None,
            geste: None,
            repere: None,
            coince_a: None,
            forcer: None,
            tapi: None,
            pv_avant: None,
            appel_a: None,
            appel_recu: None,
            renfort: None,
            chef_tombe: None,
            deroute: None,
        }
    }
}

impl Cerveau {
    /// Le cerveau d'un ennemi de ce rôle ("guerrier", "chef"…) et de ce cran d'habileté.
    pub fn pour(role: &str, cran: &str) -> Self { Self { fiche: (role.to_string(), cran.to_string()), ..Self::default() } }

    /// Son rôle et son cran d'habileté (vides : inconnus).
    pub fn fiche(&self) -> (&str, &str) { (&self.fiche.0, &self.fiche.1) }

    pub fn etat(&self) -> Etat { self.etat }

    /// En combat, il se retient de frapper à ce tick (délai de réaction, temps mort après une
    /// attaque) : il garde sa cible en face, sans lancer de coup.
    pub fn retenu(&self) -> bool { self.retenu }

    /// Le sort qu'il veut lancer (ou tenir) à ce tick, s'il a une table de sorts (`combat.rs`).
    pub fn sort(&self) -> Option<&str> {
        self.sort.as_deref().filter(|_| self.etat == Etat::Combat && !self.lache)
    }

    /// Il force le passage à travers ce qui est là.
    fn forcer_le_passage(&mut self, menace: Vec3<f32>, obs: &Observation) -> Intention {
        // À gauche ou à droite : selon l'instant.
        let passage = Forcer::a_travers(obs.pos, menace, (obs.temps * 10.0) as u64 % 2 == 0, obs.temps);
        self.forcer = Some(passage);
        self.etat = Etat::Force;
        Intention::Forcer(passage.lieu)
    }

    /// Il fuit la menace (là où il l'a perçue pour la dernière fois). Coincé, il tire une réaction :
    /// se tapir, forcer le passage ; sinon il continue de fuir comme il peut.
    fn fuir(&mut self, obs: &Observation, profil: &Profil, arrivee: f32) -> Intention {
        let menace = self.percue_en;
        self.etat = Etat::Fuite;
        // Il force le passage : jusqu'à la sortie, ou tant de secondes au plus.
        if let Some(passage) = self.forcer {
            if obs.temps - passage.depuis <= profil.forcer_duree.unwrap_or(0.0) && obs.pos.distance(passage.lieu) > arrivee {
                self.etat = Etat::Force;
                return Intention::Forcer(passage.lieu);
            }
            self.forcer = None;
            self.repere = Some((obs.pos, obs.temps));
        }
        // Tapi, il guette ; frappé, il détale à travers son agresseur.
        if let Some(depuis) = self.tapi {
            let agresseur = obs.percus.iter().find(|p| p.l_a_frappe_depuis.is_some_and(|il_y_a| obs.temps - il_y_a > depuis));
            if let Some(agresseur) = agresseur {
                self.tapi = None;
                return self.forcer_le_passage(agresseur.pos, obs);
            }
            self.etat = Etat::Tapi;
            return Intention::SeTapir(menace);
        }
        // Coincé : il n'a pas avancé depuis `coince_temps`, la menace est proche.
        let (ou, quand) = *self.repere.get_or_insert((obs.pos, obs.temps));
        if profil.coince_temps.is_some_and(|duree| obs.temps - quand >= duree) {
            self.repere = Some((obs.pos, obs.temps));
            let coince = obs.pos.distance(ou) < COINCE_PAS && obs.pos.distance(menace) <= profil.coince_portee.unwrap_or(0.0);
            let delai = profil.coince_delai.unwrap_or(0.0);
            if coince && self.coince_a.is_none_or(|avant| obs.temps - avant >= delai) {
                match self.tirer(Evenement::Coince, obs, profil) {
                    Some(Reaction::SeTapir) => {
                        self.coince_a = Some(obs.temps);
                        self.tapi = Some(obs.temps);
                        self.etat = Etat::Tapi;
                        return Intention::SeTapir(menace);
                    },
                    Some(Reaction::Forcer) => {
                        self.coince_a = Some(obs.temps);
                        return self.forcer_le_passage(menace, obs);
                    },
                    _ => {},
                }
            }
        }
        Intention::Fuir(menace)
    }

    /// Il ne fuit plus : il oublie où il en était de sa fuite.
    fn fin_de_fuite(&mut self) {
        self.repere = None;
        self.forcer = None;
        self.tapi = None;
    }

    /// Le geste de défense qu'il fait à ce tick, face à un coup armé (IA.7).
    pub fn geste(&self) -> Option<Geste> { self.geste }

    /// Un coup s'arme contre lui : il tire une fois sa réponse, et la fait après son délai de réaction.
    fn regler_defense(&mut self, obs: &Observation, profil: &Profil) {
        if !obs.percus.iter().any(|p| p.frappe_dans.is_some()) {
            self.defense = None;
            return;
        }
        if self.defense.is_none() {
            let geste = match self.tirer(Evenement::CoupArme, obs, profil) {
                Some(Reaction::Esquiver) => Some(Geste::Esquiver),
                Some(Reaction::Parer) => Some(Geste::Parer),
                _ => None,
            };
            self.defense = Some(Defense { vu_a: obs.temps, geste, fait: false });
        }
        self.geste = self.defense.as_mut().and_then(|d| d.geste(obs.temps, profil.reaction.unwrap_or(0.0)));
    }

    /// Ce sort est parti (ou ne part pas) : sa recharge court.
    fn noter_lancer(&mut self, sort: String, temps: f64) {
        self.lances.retain(|(nom, _)| *nom != sort);
        self.lances.push((sort, temps));
    }

    /// Le mage en combat : le sort à demander à ce tick, face à sa cible.
    fn regler_sort(&mut self, obs: &Observation, profil: &Profil, situation: Situation) {
        let Some(table) = &profil.sorts else {
            self.sort = None;
            return;
        };
        if obs.attaque {
            // Un sort en cours : il le tient tant que la cible est à sa portée et que son cran le laisse faire.
            let porte = self.sort.as_ref().is_some_and(|s| table.iter().any(|l| l.sort == *s && l.porte(situation.distance)));
            self.lache = self.retenu || !porte;
        } else if self.retenu {
            self.sort = None;
        } else {
            // Il choisit : rien de demandé, ou un sort qu'il n'a plus, ou qui ne part pas.
            let attendu = obs.temps - self.sort_depuis > SORT_ATTENTE;
            if self.sort.as_ref().is_none_or(|s| attendu || !obs.sorts.contains(s)) {
                if let Some(rate) = self.sort.take().filter(|_| attendu) {
                    self.noter_lancer(rate, obs.temps);
                }
                self.sort = combat::choisir(table, &obs.sorts, situation, &self.lances, obs.temps, obs.hasard)
                    .map(|s| s.sort.clone());
                self.sort_depuis = obs.temps;
            }
        }
    }

    pub fn cible(&self) -> Option<Id> { self.cible }

    /// Il entend un bruit fait à cet endroit, à cette heure (`ouie.rs` dit s'il l'entend).
    pub fn entendre(&mut self, lieu: Vec3<f32>, temps: f64) {
        if self.bruit.is_none_or(|(_, avant)| temps >= avant) {
            self.bruit = Some((lieu, temps));
        }
    }

    /// Il sait qu'il se passe quelque chose : un allié n'a pas à venir le prévenir.
    pub fn alerte(&self) -> bool { self.etat != Etat::Repos || self.bruit.is_some() || self.prevenu_a.is_some() }

    /// Il est parti prévenir un allié.
    pub fn messager(&self) -> bool { self.prevenir.is_some() }

    /// Ce qu'il a crié depuis le dernier appel : le serveur le fait entendre aux siens.
    pub fn cris(&mut self) -> Vec<Message> { std::mem::take(&mut self.cris) }

    /// Il reçoit un message, à cette heure (un cri d'allié entendu, ou ce que le monde lui apprend).
    pub fn recevoir(&mut self, message: Message, temps: f64) {
        match message {
            Message::Signalement { lieu, .. } => {
                self.prevenu_a = Some(temps);
                self.entendre(lieu, temps);
            },
            Message::AllieTombe { lieu } => {
                self.allie_tombe = Some((lieu, temps));
                self.entendre(lieu, temps);
            },
            Message::Appel { agresseur, lieu } => self.appel_recu = Some((agresseur, lieu, temps)),
            Message::ChefTombe { lieu } => self.chef_tombe = Some(lieu),
        }
    }

    /// La réaction qu'il tire à cet événement (rien : celle du tronc). Une réaction ne sort que s'il
    /// peut la faire : « prévenir » s'il y a quelqu'un à prévenir, « esquiver » et « parer » s'il sait
    /// (et a de quoi). Voir un intrus ou entendre tomber un allié n'est pas retiré avant `tirage_delai`.
    fn tirer(&mut self, evenement: Evenement, obs: &Observation, profil: &Profil) -> Option<Reaction> {
        let poids = profil.reactions.get(&evenement)?;
        let delai = match evenement {
            Evenement::VoitIntrus | Evenement::AllieTombe => profil.tirage_delai.unwrap_or(0.0),
            Evenement::CoupArme | Evenement::Coince | Evenement::Touche | Evenement::ChefTombe => 0.0,
        };
        if self.tirages.iter().any(|(e, quand)| *e == evenement && obs.temps - quand < delai) {
            return None;
        }
        self.tirages.retain(|(e, _)| *e != evenement);
        self.tirages.push((evenement, obs.temps));
        let possibles: Vec<(Reaction, f32)> = poids
            .iter()
            .filter(|(reaction, _)| match reaction {
                Reaction::Prevenir => prevenir::qui(obs).is_some(),
                Reaction::Esquiver => profil.roule.unwrap_or(true),
                Reaction::Parer => profil.pare.unwrap_or(true) && obs.peut_parer,
                Reaction::Combattre
                | Reaction::AllerVoir
                | Reaction::Rien
                | Reaction::SeTapir
                | Reaction::Forcer
                | Reaction::AppelerALAide
                | Reaction::Deroute => true,
            })
            .copied()
            .collect();
        tirer(&possibles, obs.hasard)
    }

    /// Il part prévenir, s'il y a quelqu'un à prévenir : où courir.
    fn partir_prevenir(&mut self, lieu: Vec3<f32>, obs: &Observation) -> Option<Intention> {
        let allie = prevenir::qui(obs)?;
        self.prevenir = Some(Prevenir { qui: allie.id, lieu, vu_a: obs.temps, depuis: obs.temps });
        self.cible = None;
        self.enquete = None;
        self.bruit = None;
        self.etat = Etat::Prevenir;
        Some(Intention::Prevenir(allie.pos))
    }

    pub fn tick(&mut self, obs: &Observation, profil: &Profil) -> Intention {
        let poste = *self.poste.get_or_insert(obs.pos);
        let arrivee = profil.arrivee.unwrap_or(ARRIVEE);
        if self.prevenu_a.is_some_and(|quand| obs.temps - quand > PREVENU) {
            self.prevenu_a = None;
        }
        // Une attaque qui finit ouvre son temps mort.
        self.lache = false;
        self.geste = None;
        if self.attaque_depuis.is_some() && !obs.attaque {
            self.pause_jusqu_a = obs.temps + profil.pause.unwrap_or(0.0);
            self.attaque_depuis = None;
            if let Some(lance) = self.sort.take() {
                self.noter_lancer(lance, obs.temps);
            }
        } else if obs.attaque {
            self.attaque_depuis.get_or_insert(obs.temps);
        }
        self.retenu = false;
        let fuirait = profil.fuite.is_some_and(|seuil| obs.pv < seuil);

        // On le frappe (ses PV baissent) alors qu'il est sous `appel_pv` : il tire sa réaction ; l'appel à
        // l'aide dit qui l'a frappé et où il est.
        let touche = self.pv_avant.is_some_and(|avant| obs.pv < avant);
        self.pv_avant = Some(obs.pv);
        if touche
            && profil.appel_pv.is_some_and(|seuil| obs.pv < seuil)
            && self.appel_a.is_none_or(|avant| obs.temps - avant >= profil.appel_delai.unwrap_or(0.0))
            && let Some(agresseur) = obs
                .percus
                .iter()
                .filter(|p| p.l_a_frappe_depuis.is_some())
                .min_by(|a, b| a.l_a_frappe_depuis.unwrap_or(f64::MAX).total_cmp(&b.l_a_frappe_depuis.unwrap_or(f64::MAX)))
            && self.tirer(Evenement::Touche, obs, profil) == Some(Reaction::AppelerALAide)
        {
            self.appel_a = Some(obs.temps);
            self.cris.push(Message::Appel { agresseur: agresseur.id, lieu: agresseur.pos });
        }

        // Un appel à l'aide entendu : celui qui défend vient sur l'agresseur, les autres viennent voir.
        if let Some((agresseur, lieu, quand)) = self.appel_recu.take() {
            if profil.defend == Some(true) && !fuirait {
                self.renfort = Some((agresseur, lieu, quand + profil.defend_duree.unwrap_or(0.0)));
            } else {
                self.entendre(lieu, quand);
            }
        }

        // Son chef est tombé : il tire sa réaction. En déroute, il fuit l'ennemi le plus proche (ou l'endroit
        // où le chef est tombé), même si on le frappe ; puis il se reprend et va voir où était la menace.
        if let Some(lieu) = self.chef_tombe.take()
            && let Some(duree) = profil.deroute.filter(|duree| *duree > 0.0)
            && self.tirer(Evenement::ChefTombe, obs, profil) == Some(Reaction::Deroute)
        {
            self.deroute = Some((lieu, obs.temps + duree));
        }
        if let Some((menace, fin)) = self.deroute {
            if obs.temps <= fin {
                let proche = obs.percus.iter().min_by(|a, b| {
                    a.pos.distance_squared(obs.pos).total_cmp(&b.pos.distance_squared(obs.pos))
                });
                let menace = proche.map_or(menace, |p| p.pos);
                self.deroute = Some((menace, fin));
                self.cible = None;
                self.prevenir = None;
                self.renfort = None;
                self.enquete = None;
                self.fin_de_fuite();
                self.etat = Etat::Deroute;
                return Intention::Fuir(menace);
            }
            self.deroute = None;
            self.enquete = Some(Enquete { lieu: menace, depuis: obs.temps, sur_place: None });
        }

        // Parti prévenir : il court vers l'allié sans se retourner, crie à portée de voix, puis
        // retourne voir là où étaient les intrus.
        if let Some(course) = self.prevenir {
            match course.etape(obs, profil) {
                Etape::Courir(vers) => {
                    self.etat = Etat::Prevenir;
                    return Intention::Prevenir(vers);
                },
                etape => {
                    if etape == Etape::Crier {
                        self.cris.push(Message::Signalement { lieu: course.lieu, vu_a: course.vu_a });
                    }
                    self.prevenir = None;
                    self.enquete = Some(Enquete { lieu: course.lieu, depuis: obs.temps, sur_place: None });
                },
            }
        }
        // Trop loin de son poste (`laisse`) : il ne se bat plus, il rentre.
        let dehors = profil.laisse.is_some_and(|laisse| obs.pos.distance(poste) > laisse);

        // Ses soupçons : ceux qu'il entrevoit, depuis quand sans interruption.
        self.soupcons.retain(|(id, _)| obs.percus.iter().any(|p| p.id == *id && !p.net));
        for p in obs.percus.iter().filter(|p| !p.net) {
            if !self.soupcons.iter().any(|(id, _)| *id == p.id) {
                self.soupcons.push((p.id, obs.temps));
            }
        }

        // Sa cible : il note où il la perçoit ; morte, ou lui trop loin de son poste, il la lâche.
        let mut vue = false;
        if let Some(cible) = self.cible {
            if let Some(p) = obs.percus.iter().find(|p| p.id == cible) {
                vue = true;
                self.percue_a = obs.temps;
                self.percue_en = p.pos;
            }
            if !obs.cible_en_vie || dehors {
                self.cible = None;
                self.rentre = true;
                // Le bruit entendu pendant ce combat était le sien.
                self.bruit = None;
                if !obs.cible_en_vie {
                    self.victime = Some((self.percue_en, obs.temps));
                }
            }
        }

        // Il vient défendre un allié : l'agresseur est sa cible s'il le perçoit ; sinon il court là où on le
        // lui a signalé, même s'il se battait contre un autre.
        let mut defend = false;
        if let Some((agresseur, lieu, jusqu_a)) = self.renfort {
            if obs.temps > jusqu_a || dehors {
                self.renfort = None;
            } else if let Some(p) = obs.percus.iter().find(|p| p.id == agresseur) {
                if self.cible != Some(agresseur) {
                    self.cible = Some(agresseur);
                    self.choisie_a = obs.temps;
                }
                vue = true;
                defend = true;
                self.percue_a = obs.temps;
                self.percue_en = p.pos;
                self.enquete = None;
            } else if obs.pos.distance(lieu) <= arrivee {
                self.renfort = None;
                self.enquete = Some(Enquete { lieu, depuis: obs.temps, sur_place: None });
            } else {
                self.cible = None;
                self.etat = Etat::Enquete;
                return Intention::AllerVoir(lieu);
            }
        }

        // Il (re)choisit : sans cible, à intervalles, ou dès que sa cible tombe à terre.
        let depuis = obs.temps - self.choisie_a;
        let a_terre = self.cible.is_some_and(|c| obs.percus.iter().any(|p| p.id == c && p.a_terre));
        let a_choisir = !dehors
            && !defend
            && (self.cible.is_none()
                || profil.reciblage_de(obs.moi).is_some_and(|delai| depuis > delai)
                || (a_terre && depuis > 0.5));
        if a_choisir && let Some(choisie) = self.choisir(obs, profil) {
            // Un intrus, alors qu'il n'avait pas de cible : il tire sa réaction.
            if self.cible.is_none()
                && self.tirer(Evenement::VoitIntrus, obs, profil) == Some(Reaction::Prevenir)
                && let Some(intention) = self.partir_prevenir(choisie.pos, obs)
            {
                return intention;
            }
            vue = true;
            if self.cible.is_none() {
                self.engage_a = obs.temps;
            }
            self.cible = Some(choisie.id);
            self.choisie_a = obs.temps;
            self.percue_a = obs.temps;
            self.percue_en = choisie.pos;
            self.enquete = None;
        }

        if let Some(cible) = self.cible {
            let fuit = profil.fuite.is_some_and(|seuil| obs.pv < seuil);
            // Perdue de vue : il sait encore un moment où elle était (le fuyard, plus longtemps).
            let memoire = if fuit { profil.fuite_memoire.or(profil.poursuite) } else { profil.poursuite };
            if vue || obs.temps - self.percue_a <= memoire.unwrap_or(0.0) {
                if fuit {
                    return self.fuir(obs, profil, arrivee);
                }
                self.fin_de_fuite();
                return if vue {
                    self.etat = Etat::Combat;
                    self.retenu = match self.attaque_depuis {
                        // Il ne tient pas un enchaînement plus longtemps que son cran ne le permet.
                        Some(debut) => profil.enchainement.is_some_and(|max| obs.temps - debut > max),
                        None => obs.temps - self.engage_a < profil.reaction.unwrap_or(0.0) || obs.temps < self.pause_jusqu_a,
                    };
                    self.regler_defense(obs, profil);
                    if let Some(p) = obs.percus.iter().find(|p| p.id == cible) {
                        let situation = Situation { distance: p.pos.distance(obs.pos), approche: p.approche };
                        self.regler_sort(obs, profil, situation);
                    }
                    Intention::Combattre(cible)
                } else {
                    self.etat = Etat::Combat;
                    Intention::AllerVoir(self.percue_en)
                };
            }
            self.cible = None;
            self.fin_de_fuite();
            if fuit {
                // Le fuyard se calme là où il est : c'est sa nouvelle place.
                self.poste = Some(obs.pos);
                self.rentre = false;
            } else {
                // Il va voir ce qu'il sait de plus récent : où il l'a vue, ou un bruit entendu depuis.
                let lieu = self.bruit.take().filter(|(_, quand)| *quand > self.percue_a).map_or(self.percue_en, |(lieu, _)| lieu);
                self.enquete = Some(Enquete { lieu, depuis: obs.temps, sur_place: None });
            }
        }

        // Un des siens est tombé : il tire sa réaction (aller voir, comme pour un bruit, ou prévenir).
        if let Some((lieu, _)) = self.allie_tombe.take()
            && !dehors
            && !fuirait
            && self.tirer(Evenement::AllieTombe, obs, profil) == Some(Reaction::Prevenir)
            && let Some(intention) = self.partir_prevenir(lieu, obs)
        {
            return intention;
        }

        // Un bruit : il va voir (pas celui qui fuirait, ni celui que sa laisse retient).
        if let Some((lieu, quand)) = self.bruit.take()
            && !self.victime.is_some_and(|(ou, mort)| quand - mort < VICTIME && lieu.distance(ou) < VICTIME_PRES)
            && !dehors
            && !fuirait
        {
            self.enquete = Some(Enquete { lieu, depuis: obs.temps, sur_place: None });
        }

        // Un soupçon : il s'arrête et regarde.
        if !dehors && let Some(p) = obs.percus.iter().find(|p| !p.net) {
            self.etat = Etat::Enquete;
            return Intention::Regarder(p.pos);
        }

        // Il va voir un endroit, y fouille du regard, puis rentre.
        if let Some(enquete) = &mut self.enquete {
            if enquete.sur_place.is_none() && obs.pos.distance(enquete.lieu) <= arrivee {
                enquete.sur_place = Some(obs.temps);
            }
            let intention = match enquete.sur_place {
                None if obs.temps - enquete.depuis <= profil.trajet.unwrap_or(0.0) => {
                    Some(Intention::AllerVoir(enquete.lieu))
                },
                Some(debut) if obs.temps - debut <= profil.fouille.unwrap_or(0.0) => {
                    let angle = (obs.temps - debut) as f32 * TOUR;
                    Some(Intention::Regarder(obs.pos + Vec3::new(angle.cos(), angle.sin(), 0.0) * REGARD))
                },
                _ => None,
            };
            if let Some(intention) = intention {
                self.etat = Etat::Enquete;
                return intention;
            }
            self.enquete = None;
            self.rentre = true;
        }

        self.etat = Etat::Repos;
        if self.rentre {
            let poste = self.poste.unwrap_or(poste);
            if obs.pos.distance(poste) > arrivee {
                return Intention::Rentrer(poste);
            }
            self.rentre = false;
        }
        Intention::Repos
    }

    /// La meilleure cible parmi ceux qu'il a repérés (reconnus, déjà sa cible, ou soupçonnés
    /// depuis `soupcon` secondes) : le plus petit rang, puis le plus petit score (`cible.rs`).
    fn choisir<'a>(&self, obs: &'a Observation, profil: &Profil) -> Option<&'a Percu> {
        let acheve = profil.acheve_a(obs.meute);
        let repere = |p: &Percu| {
            p.net
                || self.cible == Some(p.id)
                || self.soupcons.iter().any(|(id, depuis)| {
                    *id == p.id && obs.temps - depuis >= profil.soupcon.unwrap_or(0.0)
                })
        };
        obs.percus
            .iter()
            .filter(|p| repere(p))
            .map(|p| {
                let distance = p.pos.distance(obs.pos);
                // Sans se compter parmi ceux qui la visent déjà.
                let dessus = p.dessus.saturating_sub(u8::from(self.cible == Some(p.id)));
                let rang = profil.rang(distance, p.a_terre, p.l_a_frappe_depuis, acheve);
                (rang, profil.score(distance, dessus), p)
            })
            .min_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)))
            .map(|(_, _, p)| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{combat::Sort, reactions::defense::Geste};
    use std::collections::HashMap;

    fn orc() -> Profil {
        Profil {
            reciblage: Some(3.0),
            repartition: Some(8.0),
            soupcon: Some(1.0),
            poursuite: Some(3.0),
            fouille: Some(6.0),
            trajet: Some(20.0),
            arrivee: Some(2.0),
            ..Profil::default()
        }
    }

    fn danari() -> Profil {
        Profil {
            acheve: Some(20.0),
            meute: Some(2),
            meute_rayon: Some(20.0),
            provoque_par_coup: Some(3.0),
            fuite: Some(0.3),
            ..orc()
        }
    }

    fn en(x: f32, y: f32) -> Vec3<f32> { Vec3::new(x, y, 0.0) }

    /// Un joueur debout, reconnu, à `distance` blocs à l'est de l'origine.
    fn joueur(id: Id, distance: f32) -> Percu {
        Percu {
            id,
            pos: en(distance, 0.0),
            net: true,
            a_terre: false,
            l_a_frappe_depuis: None,
            dessus: 0,
            approche: 0.0,
            frappe_dans: None,
        }
    }

    /// L'observation d'un ennemi en pleine forme à l'origine, seul de sa race (`moi` 0 : reciblage
    /// à 75 %).
    fn obs(temps: f64, percus: Vec<Percu>) -> Observation {
        Observation { temps, moi: 0, pos: Vec3::zero(), pv: 1.0, meute: 1, percus, cible_en_vie: true, ..Observation::default() }
    }

    fn obs_en(temps: f64, pos: Vec3<f32>, percus: Vec<Percu>) -> Observation {
        Observation { pos, ..obs(temps, percus) }
    }

    #[test]
    fn sans_personne_il_reste_au_repos() {
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![]), &orc()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.cible(), None);
    }

    #[test]
    fn il_combat_celui_qu_il_percoit() {
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![joueur(7, 10.0)]), &orc()), Intention::Combattre(7));
        assert_eq!(c.etat(), Etat::Combat);
    }

    /// `creatures.md`, choix de cible : un joueur debout passe avant un joueur à terre.
    #[test]
    fn il_prend_le_joueur_debout_avant_le_joueur_a_terre() {
        let mut c = Cerveau::default();
        let a_terre = Percu { a_terre: true, ..joueur(1, 2.0) };
        assert_eq!(c.tick(&obs(0.0, vec![a_terre, joueur(2, 15.0)]), &orc()), Intention::Combattre(2));
    }

    /// `creatures.md` : « elle garde sa cible tant qu'elle la perçoit » ; le reciblage (Y.9) ne
    /// revient que toutes les `reciblage` secondes.
    #[test]
    fn il_garde_sa_cible_jusqu_au_reciblage() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        // Un joueur plus proche arrive : pas avant le délai (3 s × 0,75 pour `moi` 0).
        let deux = vec![joueur(1, 10.0), joueur(2, 3.0)];
        assert_eq!(c.tick(&obs(1.0, deux.clone()), &orc()), Intention::Combattre(1));
        assert_eq!(c.tick(&obs(2.5, deux), &orc()), Intention::Combattre(2));
    }

    /// Y.9 : il ne se compte pas parmi ceux qui visent déjà sa cible (sinon il la quitterait pour
    /// un joueur plus loin dès le premier reciblage).
    #[test]
    fn il_ne_se_compte_pas_sur_sa_propre_cible() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 5.0), joueur(2, 9.0)]), &orc());
        let sur_lui = vec![Percu { dessus: 1, ..joueur(1, 5.0) }, joueur(2, 9.0)];
        assert_eq!(c.tick(&obs(10.0, sur_lui), &orc()), Intention::Combattre(1));
    }

    /// Y.9 : une cible qui tombe à terre fait choisir tout de suite.
    #[test]
    fn sa_cible_tombe_a_terre_il_en_change_aussitot() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0), joueur(2, 12.0)]), &orc());
        let tombe = vec![Percu { a_terre: true, ..joueur(1, 3.0) }, joueur(2, 12.0)];
        assert_eq!(c.tick(&obs(1.0, tombe), &orc()), Intention::Combattre(2));
    }

    /// Sa cible morte, il en prend une autre ; sinon il rentre à son poste, sans fouiller.
    #[test]
    fn sa_cible_meurt_il_en_prend_une_autre_ou_rentre() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        let morte = Observation { cible_en_vie: false, ..obs(0.5, vec![joueur(2, 12.0)]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Combattre(2));
        // Il a couru 30 blocs ; sa cible meurt : il rentre d'où il vient.
        let morte = Observation { cible_en_vie: false, ..obs_en(1.0, en(30.0, 0.0), vec![]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Rentrer(Vec3::zero()));
        assert_eq!(c.etat(), Etat::Repos);
        // Rentré, il vaque.
        assert_eq!(c.tick(&obs(9.0, vec![]), &orc()), Intention::Repos);
    }

    /// « Pour Veloren » : un joueur loin ou discret est d'abord soupçonné (l'ennemi se tourne et
    /// regarde), puis repéré s'il reste visible.
    #[test]
    fn un_soupcon_qui_dure_devient_un_combat() {
        let loin = |id| Percu { net: false, ..joueur(id, 25.0) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![loin(1)]), &orc()), Intention::Regarder(en(25.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
        assert_eq!(c.cible(), None);
        assert_eq!(c.tick(&obs(0.9, vec![loin(1)]), &orc()), Intention::Regarder(en(25.0, 0.0)));
        assert_eq!(c.tick(&obs(1.1, vec![loin(1)]), &orc()), Intention::Combattre(1));
    }

    /// Un soupçon qui disparaît repart de zéro : on passe pendant qu'il regarde ailleurs.
    #[test]
    fn un_soupcon_interrompu_repart_de_zero() {
        let loin = Percu { net: false, ..joueur(1, 25.0) };
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![loin.clone()]), &orc());
        assert_eq!(c.tick(&obs(0.8, vec![]), &orc()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.tick(&obs(1.2, vec![loin.clone()]), &orc()), Intention::Regarder(en(25.0, 0.0)));
        assert_eq!(c.tick(&obs(2.3, vec![loin]), &orc()), Intention::Combattre(1));
    }

    /// `creatures.md` : « Une cible perdue de vue est poursuivie 3 s vers sa dernière position
    /// connue, puis la créature passe en alerte à cet endroit » ; elle y fouille 6 s, puis rentre.
    #[test]
    fn cible_perdue_il_court_ou_il_l_a_vue_fouille_puis_rentre() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        // Le joueur passe derrière un mur : l'ennemi ne sait que là où il l'a vu.
        assert_eq!(c.tick(&obs(1.0, vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        assert_eq!(c.etat(), Etat::Combat);
        // Après 3 s, il n'a plus de cible : il enquête là-bas.
        assert_eq!(c.tick(&obs_en(3.5, en(6.0, 0.0), vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
        assert_eq!(c.cible(), None);
        // Arrivé, il fouille du regard, pas toujours du même côté.
        let sur_place = en(9.0, 0.0);
        let Intention::Regarder(a) = c.tick(&obs_en(5.0, sur_place, vec![]), &orc()) else {
            panic!("il doit regarder autour de lui")
        };
        let Intention::Regarder(b) = c.tick(&obs_en(7.0, sur_place, vec![]), &orc()) else {
            panic!("il doit regarder autour de lui")
        };
        assert!(a.distance(b) > 1.0, "il regarde toujours au même endroit");
        assert_eq!(c.etat(), Etat::Enquete);
        // 6 s plus tard, il rentre à son poste (là où il était au premier tick), puis vaque.
        assert_eq!(c.tick(&obs_en(11.5, sur_place, vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.tick(&obs_en(15.0, en(1.0, 0.0), vec![]), &orc()), Intention::Repos);
    }

    #[test]
    fn revue_pendant_la_fouille_le_combat_reprend() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        c.tick(&obs_en(3.5, en(6.0, 0.0), vec![]), &orc());
        c.tick(&obs_en(5.0, en(9.0, 0.0), vec![]), &orc());
        assert_eq!(c.tick(&obs_en(6.0, en(9.0, 0.0), vec![joueur(1, 14.0)]), &orc()), Intention::Combattre(1));
        assert_eq!(c.etat(), Etat::Combat);
    }

    /// Un endroit qu'il n'arrive pas à rejoindre : il renonce après `trajet` secondes.
    #[test]
    fn il_renonce_a_un_endroit_qu_il_n_atteint_pas() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        c.tick(&obs_en(3.5, en(5.0, 0.0), vec![]), &orc());
        assert_eq!(c.tick(&obs_en(20.0, en(5.0, 0.0), vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        assert_eq!(c.tick(&obs_en(24.0, en(5.0, 0.0), vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
    }

    /// Y.5 : avec une laisse, il ne s'éloigne pas de son poste : il lâche sa cible et rentre, et
    /// n'en reprend pas tant qu'il est dehors.
    #[test]
    fn la_laisse_le_ramene_a_son_poste() {
        let nain = Profil { laisse: Some(15.0), ..orc() };
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &nain);
        let dehors = en(16.0, 0.0);
        let devant = Percu { pos: en(20.0, 0.0), ..joueur(1, 0.0) };
        assert_eq!(c.tick(&obs_en(4.0, dehors, vec![devant.clone()]), &nain), Intention::Rentrer(Vec3::zero()));
        assert_eq!(c.cible(), None);
        assert_eq!(c.tick(&obs_en(4.5, dehors, vec![devant.clone()]), &nain), Intention::Rentrer(Vec3::zero()));
        // De retour dans son rayon, il se bat de nouveau.
        assert_eq!(c.tick(&obs_en(6.0, en(8.0, 0.0), vec![devant]), &nain), Intention::Combattre(1));
    }

    /// Y.1 : un danari fuit sous 30 % de ses PV ; un orc ne fuit pas.
    #[test]
    fn le_danari_blesse_fuit_pas_l_orc() {
        let blesse = Observation { pv: 0.2, ..obs(0.0, vec![joueur(1, 3.0)]) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&blesse, &danari()), Intention::Fuir(en(3.0, 0.0)));
        assert_eq!(c.etat(), Etat::Fuite);
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&blesse, &orc()), Intention::Combattre(1));
        // Sans personne à fuir, il se repose.
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&Observation { pv: 0.2, ..obs(0.0, vec![]) }, &danari()), Intention::Repos);
    }

    /// `creatures.md`, « le fuyard n'oublie pas » : il fuit encore une menace qu'il ne voit plus,
    /// puis se calme là où il est : c'est sa nouvelle place, il ne revient pas voir.
    #[test]
    fn le_fuyard_se_calme_la_ou_il_est() {
        let mut c = Cerveau::default();
        c.tick(&Observation { pv: 0.2, ..obs(0.0, vec![joueur(1, 3.0)]) }, &danari());
        let loin = en(-20.0, 0.0);
        let seul = |temps| Observation { pv: 0.2, ..obs_en(temps, loin, vec![]) };
        assert_eq!(c.tick(&seul(2.0), &danari()), Intention::Fuir(en(3.0, 0.0)));
        assert_eq!(c.tick(&seul(4.0), &danari()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
        assert_eq!(c.tick(&seul(30.0), &danari()), Intention::Repos);
    }

    /// Y.2 : les danari à plusieurs achèvent le joueur à terre, sauf si quelqu'un vient de les frapper.
    #[test]
    fn les_danari_a_plusieurs_achevent_sauf_si_on_les_frappe() {
        let a_terre = Percu { a_terre: true, ..joueur(1, 15.0) };
        let a_deux = |percus| Observation { meute: 2, ..obs(0.0, percus) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&a_deux(vec![a_terre.clone(), joueur(2, 2.0)]), &danari()), Intention::Combattre(1));
        // Seul, il fait comme les autres.
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![a_terre.clone(), joueur(2, 2.0)]), &danari()), Intention::Combattre(2));
        // Frappé à l'instant par le joueur debout : c'est lui d'abord.
        let frappe = Percu { l_a_frappe_depuis: Some(1.0), ..joueur(2, 2.0) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&a_deux(vec![a_terre, frappe]), &danari()), Intention::Combattre(2));
    }

    /// « Entendre un bruit → enquête : il va voir l'endroit, fouille un moment, puis rentre à son
    /// poste. »
    #[test]
    fn un_bruit_le_fait_venir_voir_fouiller_puis_rentrer() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![]), &orc());
        c.entendre(en(15.0, 0.0), 1.0);
        assert_eq!(c.tick(&obs(1.0, vec![]), &orc()), Intention::AllerVoir(en(15.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
        let sur_place = en(14.0, 0.0);
        assert!(matches!(c.tick(&obs_en(5.0, sur_place, vec![]), &orc()), Intention::Regarder(_)));
        assert_eq!(c.tick(&obs_en(11.5, sur_place, vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
    }

    /// Un bruit plus récent déplace l'enquête.
    #[test]
    fn un_nouveau_bruit_deplace_l_enquete() {
        let mut c = Cerveau::default();
        c.entendre(en(15.0, 0.0), 0.0);
        c.tick(&obs(0.0, vec![]), &orc());
        c.entendre(en(0.0, 12.0), 2.0);
        assert_eq!(c.tick(&obs(2.0, vec![]), &orc()), Intention::AllerVoir(en(0.0, 12.0)));
    }

    /// « Il entend aussi pendant un combat » : il ne lâche pas la cible qu'il voit ; s'il la perd,
    /// il va voir le bruit entendu depuis plutôt que l'endroit où il l'a vue.
    #[test]
    fn en_combat_il_retient_le_bruit_pour_quand_il_perd_sa_cible() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 10.0)]), &orc());
        c.entendre(en(0.0, -14.0), 0.5);
        assert_eq!(c.tick(&obs(1.0, vec![joueur(1, 10.0)]), &orc()), Intention::Combattre(1));
        // Perdue de vue : ce bruit est plus vieux que la dernière fois qu'il l'a vue.
        assert_eq!(c.tick(&obs(2.0, vec![]), &orc()), Intention::AllerVoir(en(10.0, 0.0)));
        c.entendre(en(0.0, -14.0), 3.0);
        assert_eq!(c.tick(&obs(4.5, vec![]), &orc()), Intention::AllerVoir(en(0.0, -14.0)));
        assert_eq!(c.etat(), Etat::Enquete);
    }

    /// Le bruit de son propre combat ne le fait pas fouiller quand sa cible meurt : il rentre.
    #[test]
    fn sa_cible_morte_il_oublie_le_bruit_de_son_combat() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        c.entendre(en(3.0, 0.0), 0.5);
        let morte = Observation { cible_en_vie: false, ..obs_en(1.0, en(30.0, 0.0), vec![]) };
        assert_eq!(c.tick(&morte, &orc()), Intention::Rentrer(Vec3::zero()));
        // Le bruit de la mort de sa victime lui arrive juste après : il ne va pas la voir.
        c.entendre(en(3.5, 0.0), 1.03);
        assert_eq!(c.tick(&obs_en(1.1, en(30.0, 0.0), vec![]), &orc()), Intention::Rentrer(Vec3::zero()));
        // Une autre mort, ailleurs ou plus tard : il y va.
        c.entendre(en(3.5, 20.0), 1.2);
        assert_eq!(c.tick(&obs_en(1.2, en(30.0, 0.0), vec![]), &orc()), Intention::AllerVoir(en(3.5, 20.0)));
    }

    /// Celui qui fuirait (PV sous son seuil) ne va pas voir un bruit.
    #[test]
    fn un_blesse_qui_fuit_ne_va_pas_voir_un_bruit() {
        let mut c = Cerveau::default();
        c.entendre(en(15.0, 0.0), 0.0);
        assert_eq!(c.tick(&Observation { pv: 0.2, ..obs(0.0, vec![]) }, &danari()), Intention::Repos);
    }

    // --- IA.4 : prévenir ---

    /// Un orc qui, à la vue d'un intrus, se bat (poids 1) ou part prévenir (poids 1).
    fn orc_qui_previent() -> Profil {
        Profil {
            reactions: HashMap::from([
                (Evenement::VoitIntrus, vec![(Reaction::Combattre, 1.0), (Reaction::Prevenir, 1.0)]),
                (Evenement::AllieTombe, vec![(Reaction::AllerVoir, 1.0), (Reaction::Prevenir, 1.0)]),
            ]),
            prevenir_a: Some(6.0),
            tirage_delai: Some(45.0),
            ..orc()
        }
    }

    /// Un allié au repos, à `distance` blocs à l'ouest.
    fn allie(id: Id, distance: f32) -> Allie {
        Allie { id, pos: en(-distance, 0.0), alerte: false, messager: false }
    }

    fn avec(obs: Observation, hasard: f32, allies: Vec<Allie>) -> Observation { Observation { hasard, allies, ..obs } }

    /// « À chaque événement […] chaque ennemi tire sa réaction » : à la vue d'un intrus, selon le
    /// tirage, il se bat ou part prévenir. « Chaque décision tirée s'annonce » : son état le dit.
    #[test]
    fn a_la_vue_d_un_intrus_il_tire_se_battre_ou_prevenir() {
        let vu = |hasard| avec(obs(0.0, vec![joueur(7, 10.0)]), hasard, vec![allie(2, 40.0)]);
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&vu(0.2), &orc_qui_previent()), Intention::Combattre(7));
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&vu(0.8), &orc_qui_previent()), Intention::Prevenir(en(-40.0, 0.0)));
        assert_eq!(c.etat(), Etat::Prevenir);
        assert!(c.messager());
        // Un profil sans tirage se bat toujours.
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&vu(0.8), &orc()), Intention::Combattre(7));
    }

    /// « Celui qui tire "aller prévenir" court vers l'allié non alerté le plus proche. » Sans allié
    /// à prévenir, il se bat.
    #[test]
    fn il_previent_l_allie_non_alerte_le_plus_proche() {
        let vu = |allies| avec(obs(0.0, vec![joueur(7, 10.0)]), 0.8, allies);
        let alerte = Allie { alerte: true, ..allie(2, 20.0) };
        let mut c = Cerveau::default();
        let allies = vec![alerte.clone(), allie(3, 50.0), allie(4, 35.0)];
        assert_eq!(c.tick(&vu(allies), &orc_qui_previent()), Intention::Prevenir(en(-35.0, 0.0)));
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&vu(vec![alerte]), &orc_qui_previent()), Intention::Combattre(7));
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&vu(vec![]), &orc_qui_previent()), Intention::Combattre(7));
    }

    /// « Les poids tiennent compte de ce qu'il voit faire ses voisins (pas deux messagers). »
    #[test]
    fn pas_deux_messagers() {
        let parti = Allie { alerte: true, messager: true, ..allie(2, 8.0) };
        let vu = avec(obs(0.0, vec![joueur(7, 10.0)]), 0.8, vec![parti, allie(3, 40.0)]);
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&vu, &orc_qui_previent()), Intention::Combattre(7));
    }

    /// « … et crie ; le cri porte un signalement (où il a vu les intrus, quand). » Puis il retourne
    /// voir là-bas.
    #[test]
    fn arrive_pres_de_l_allie_il_crie_le_signalement_et_y_retourne() {
        let mut c = Cerveau::default();
        c.tick(&avec(obs(0.0, vec![joueur(7, 10.0)]), 0.8, vec![allie(2, 40.0)]), &orc_qui_previent());
        // En route, il ne se retourne pas vers l'intrus, et ne crie pas encore.
        let en_route = avec(obs_en(2.0, en(-15.0, 0.0), vec![joueur(7, 30.0)]), 0.0, vec![allie(2, 40.0)]);
        assert_eq!(c.tick(&en_route, &orc_qui_previent()), Intention::Prevenir(en(-40.0, 0.0)));
        assert!(c.cris().is_empty());
        // À portée de voix de l'allié.
        let arrive = avec(obs_en(6.0, en(-35.0, 0.0), vec![]), 0.0, vec![allie(2, 40.0)]);
        assert_eq!(c.tick(&arrive, &orc_qui_previent()), Intention::AllerVoir(en(10.0, 0.0)));
        assert_eq!(c.cris(), vec![Message::Signalement { lieu: en(10.0, 0.0), vu_a: 0.0 }]);
        assert!(!c.messager());
        assert_eq!(c.etat(), Etat::Enquete);
        // Le cri n'est poussé qu'une fois.
        assert!(c.cris().is_empty());
    }

    /// « Ceux qui l'entendent vont voir à cet endroit ; ils attaquent s'ils voient un joueur, sinon
    /// ils fouillent. »
    #[test]
    fn celui_qui_recoit_le_signalement_va_voir_attaque_ou_fouille() {
        let lieu = en(30.0, 0.0);
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![]), &orc());
        c.recevoir(Message::Signalement { lieu, vu_a: 0.0 }, 5.0);
        assert!(c.alerte());
        assert_eq!(c.tick(&obs(5.0, vec![]), &orc()), Intention::AllerVoir(lieu));
        // Personne sur place : il fouille.
        assert!(matches!(c.tick(&obs_en(9.0, en(29.0, 0.0), vec![]), &orc()), Intention::Regarder(_)));
        // Un joueur en vue : il attaque.
        let la = Percu { pos: en(35.0, 0.0), ..joueur(7, 0.0) };
        assert_eq!(c.tick(&obs_en(10.0, en(29.0, 0.0), vec![la]), &orc()), Intention::Combattre(7));
    }

    /// Il ne retire pas « prévenir » à chaque fois qu'il revoit l'intrus : pas avant `tirage_delai`.
    #[test]
    fn il_ne_repart_pas_prevenir_tout_de_suite() {
        let mut c = Cerveau::default();
        c.tick(&avec(obs(0.0, vec![joueur(7, 10.0)]), 0.8, vec![allie(2, 40.0)]), &orc_qui_previent());
        c.tick(&avec(obs_en(6.0, en(-35.0, 0.0), vec![]), 0.0, vec![allie(2, 40.0)]), &orc_qui_previent());
        // Il revoit l'intrus : un autre allié est encore au repos, mais il se bat.
        let revu = avec(obs_en(10.0, en(-20.0, 0.0), vec![joueur(7, 0.0)]), 0.8, vec![allie(3, 60.0)]);
        assert_eq!(c.tick(&revu, &orc_qui_previent()), Intention::Combattre(7));
    }

    /// L'allié reste introuvable : il renonce après `trajet` secondes et retourne voir.
    #[test]
    fn il_renonce_a_prevenir_un_allie_qu_il_n_atteint_pas() {
        let mut c = Cerveau::default();
        c.tick(&avec(obs(0.0, vec![joueur(7, 10.0)]), 0.8, vec![allie(2, 40.0)]), &orc_qui_previent());
        let bloque = |temps| avec(obs_en(temps, en(-5.0, 0.0), vec![]), 0.0, vec![allie(2, 40.0)]);
        assert_eq!(c.tick(&bloque(19.0), &orc_qui_previent()), Intention::Prevenir(en(-40.0, 0.0)));
        assert_eq!(c.tick(&bloque(21.0), &orc_qui_previent()), Intention::AllerVoir(en(10.0, 0.0)));
        assert!(c.cris().is_empty());
    }

    /// Un allié tombe à portée d'oreille : il tire aller voir, ou prévenir (le signalement dit où
    /// l'allié est tombé).
    #[test]
    fn un_allie_tombe_il_va_voir_ou_previent() {
        let la = en(15.0, 0.0);
        let entendu = |hasard| {
            let mut c = Cerveau::default();
            c.tick(&obs(0.0, vec![]), &orc_qui_previent());
            c.recevoir(Message::AllieTombe { lieu: la }, 1.0);
            let intention = c.tick(&avec(obs(1.0, vec![]), hasard, vec![allie(2, 40.0)]), &orc_qui_previent());
            (c, intention)
        };
        assert_eq!(entendu(0.2).1, Intention::AllerVoir(la));
        let (mut c, intention) = entendu(0.8);
        assert_eq!(intention, Intention::Prevenir(en(-40.0, 0.0)));
        c.tick(&avec(obs_en(6.0, en(-36.0, 0.0), vec![]), 0.0, vec![allie(2, 40.0)]), &orc_qui_previent());
        assert_eq!(c.cris(), vec![Message::Signalement { lieu: la, vu_a: 1.0 }]);
    }

    // --- IA.5 : habileté ---

    fn debutant() -> Profil { Profil { reaction: Some(0.8), pause: Some(1.5), ..orc() } }

    /// « Trois crans d'habileté […] : délai de réaction » : il repère un intrus, se met en garde,
    /// mais ne frappe qu'après son délai de réaction.
    #[test]
    fn il_ne_frappe_qu_apres_son_delai_de_reaction() {
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &debutant()), Intention::Combattre(1));
        assert!(c.retenu());
        c.tick(&obs(0.7, vec![joueur(1, 3.0)]), &debutant());
        assert!(c.retenu());
        c.tick(&obs(0.9, vec![joueur(1, 3.0)]), &debutant());
        assert!(!c.retenu());
        // Sans délai dans son profil, il frappe aussitôt.
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &orc());
        assert!(!c.retenu());
    }

    /// « … temps mort entre deux attaques » : son attaque finie, il attend `pause` avant la suivante.
    #[test]
    fn il_laisse_un_temps_mort_apres_une_attaque() {
        let frappe = |temps, attaque| Observation { attaque, ..obs(temps, vec![joueur(1, 3.0)]) };
        let mut c = Cerveau::default();
        c.tick(&frappe(0.0, false), &debutant());
        c.tick(&frappe(1.0, true), &debutant());
        assert!(!c.retenu(), "il ne se retient pas au milieu de son attaque");
        c.tick(&frappe(1.6, false), &debutant());
        assert!(c.retenu());
        c.tick(&frappe(3.0, false), &debutant());
        assert!(c.retenu());
        c.tick(&frappe(3.2, false), &debutant());
        assert!(!c.retenu());
        // Un vétéran enchaîne.
        let veteran = Profil { pause: Some(0.0), ..orc() };
        let mut c = Cerveau::default();
        c.tick(&frappe(0.0, true), &veteran);
        c.tick(&frappe(0.6, false), &veteran);
        assert!(!c.retenu());
    }

    /// « Compétences utilisées » : au cran bas, il ne tient pas un enchaînement plus de
    /// `enchainement` secondes : il lâche, son attaque finit, son temps mort commence.
    #[test]
    fn il_ne_tient_pas_un_enchainement_trop_longtemps() {
        let simple = Profil { enchainement: Some(1.0), ..debutant() };
        let frappe = |temps, attaque| Observation { attaque, ..obs(temps, vec![joueur(1, 3.0)]) };
        let mut c = Cerveau::default();
        c.tick(&frappe(0.0, false), &simple);
        c.tick(&frappe(2.0, true), &simple);
        c.tick(&frappe(2.9, true), &simple);
        assert!(!c.retenu());
        c.tick(&frappe(3.1, true), &simple);
        assert!(c.retenu(), "il enchaîne depuis plus d'une seconde");
        // L'attaque finie, le temps mort ; puis une nouvelle attaque repart de zéro.
        c.tick(&frappe(3.4, false), &simple);
        assert!(c.retenu());
        c.tick(&frappe(5.0, false), &simple);
        assert!(!c.retenu());
        c.tick(&frappe(5.2, true), &simple);
        assert!(!c.retenu());
        // Sans limite dans son profil, il enchaîne tant qu'il veut.
        let mut c = Cerveau::default();
        c.tick(&frappe(0.0, true), &debutant());
        c.tick(&frappe(9.0, true), &debutant());
        assert!(!c.retenu());
    }

    /// Hors combat, il n'est pas « retenu ».
    #[test]
    fn hors_combat_il_n_est_pas_retenu() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &debutant());
        c.tick(&Observation { cible_en_vie: false, ..obs(0.2, vec![]) }, &debutant());
        assert!(!c.retenu());
    }

    // --- IA.6 : le mage ---

    fn mage() -> Profil {
        let sort = |nom: &str, min, max, recharge| Sort { sort: nom.to_string(), min, max, poids: 1.0, recharge, approche: None };
        Profil { sorts: Some(vec![sort("boule", 8.0, 40.0, 0.0), sort("rayon", 3.0, 12.0, 0.0), sort("onde", 0.0, 5.0, 6.0)]), ..orc() }
    }

    /// Le mage en pleine forme, à l'origine, qui voit un joueur à `distance` ; `attaque` : un sort est en cours.
    fn face(temps: f64, distance: f32, attaque: bool) -> Observation {
        let sorts = vec!["boule".to_string(), "rayon".to_string(), "onde".to_string()];
        Observation { attaque, sorts, hasard: 0.5, ..obs(temps, vec![joueur(1, distance)]) }
    }

    /// « Le mage choisit son sort selon la distance et la situation » ; il garde son choix jusqu'à
    /// ce que le sort parte, puis en choisit un autre.
    #[test]
    fn le_mage_demande_le_sort_qui_convient_a_la_distance() {
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&face(0.0, 25.0, false), &mage()), Intention::Combattre(1));
        assert_eq!(c.sort(), Some("boule"));
        // Le sort part, puis finit ; la cible est maintenant au contact : l'onde.
        c.tick(&face(0.3, 25.0, true), &mage());
        assert_eq!(c.sort(), Some("boule"));
        c.tick(&face(1.0, 2.0, false), &mage());
        assert_eq!(c.sort(), Some("onde"));
        // L'onde lancée recharge : au contact il n'a rien d'autre.
        c.tick(&face(1.2, 2.0, true), &mage());
        c.tick(&face(2.0, 2.0, false), &mage());
        assert_eq!(c.sort(), None);
        // Un guerrier n'a pas de table : il laisse faire Veloren.
        let mut c = Cerveau::default();
        c.tick(&face(0.0, 25.0, false), &orc());
        assert_eq!(c.sort(), None);
    }

    /// Un sort tenu (le rayon) est lâché quand la cible sort de sa portée.
    #[test]
    fn le_mage_lache_un_sort_tenu_quand_la_cible_sort_de_portee() {
        let mut c = Cerveau::default();
        c.tick(&Observation { hasard: 0.9, ..face(0.0, 10.0, false) }, &mage());
        assert_eq!(c.sort(), Some("rayon"));
        c.tick(&face(0.5, 10.0, true), &mage());
        assert_eq!(c.sort(), Some("rayon"));
        c.tick(&face(1.0, 16.0, true), &mage());
        assert_eq!(c.sort(), None);
    }

    /// Pendant son délai de réaction ou son temps mort, il ne demande aucun sort.
    #[test]
    fn le_mage_qui_se_retient_ne_demande_rien() {
        let lent = Profil { reaction: Some(1.0), ..mage() };
        let mut c = Cerveau::default();
        c.tick(&face(0.0, 25.0, false), &lent);
        assert!(c.retenu());
        assert_eq!(c.sort(), None);
        c.tick(&face(1.2, 25.0, false), &lent);
        assert_eq!(c.sort(), Some("boule"));
    }

    /// Un sort demandé qui ne part pas (plus d'énergie, gêné) : il passe à un autre.
    #[test]
    fn le_mage_renonce_a_un_sort_qui_ne_part_pas() {
        let mut c = Cerveau::default();
        c.tick(&Observation { hasard: 0.9, ..face(0.0, 10.0, false) }, &mage());
        assert_eq!(c.sort(), Some("rayon"));
        let sans_rayon = |temps| Observation { sorts: vec!["boule".to_string()], ..face(temps, 10.0, false) };
        c.tick(&sans_rayon(0.5), &mage());
        assert_eq!(c.sort(), Some("boule"));
    }

    // --- IA.7 : défense réactive ---

    /// Un vétéran : il réagit en 0,25 s, sait rouler et parer, et tire l'un ou l'autre.
    fn veteran() -> Profil {
        Profil {
            reaction: Some(0.25),
            roule: Some(true),
            pare: Some(true),
            reactions: HashMap::from([(Evenement::CoupArme, vec![(Reaction::Esquiver, 1.0), (Reaction::Parer, 1.0)])]),
            ..orc()
        }
    }

    /// Le joueur 1, au contact, dont le coup tombera dans `dans` secondes (rien : il n'arme rien).
    fn arme(temps: f64, dans: Option<f32>, hasard: f32) -> Observation {
        let joueur = Percu { frappe_dans: dans, ..joueur(1, 2.0) };
        Observation { hasard, peut_parer: true, ..obs(temps, vec![joueur]) }
    }

    /// « Défense réactive : le joueur arme un coup : esquive ou parade tirée après le délai de
    /// réaction. » Un coup lent est esquivé ou paré.
    #[test]
    fn un_coup_lent_est_esquive_ou_pare() {
        for (hasard, geste) in [(0.2, Geste::Esquiver), (0.8, Geste::Parer)] {
            let mut c = Cerveau::default();
            c.tick(&arme(0.0, None, hasard), &veteran());
            c.tick(&arme(1.0, Some(0.8), hasard), &veteran());
            assert_eq!(c.geste(), None, "il n'a pas encore réagi");
            c.tick(&arme(1.2, Some(0.6), hasard), &veteran());
            assert_eq!(c.geste(), None);
            c.tick(&arme(1.3, Some(0.5), hasard), &veteran());
            assert_eq!(c.geste(), Some(geste));
        }
    }

    /// « Une compétence rapide passe » : le coup tombe avant la fin de son délai de réaction.
    #[test]
    fn un_coup_rapide_passe() {
        let mut c = Cerveau::default();
        c.tick(&arme(0.0, None, 0.2), &veteran());
        c.tick(&arme(1.0, Some(0.2), 0.2), &veteran());
        c.tick(&arme(1.1, Some(0.1), 0.2), &veteran());
        assert_eq!(c.geste(), None);
        // Le coup est tombé.
        c.tick(&arme(1.3, None, 0.2), &veteran());
        assert_eq!(c.geste(), None);
    }

    /// La roulade est un geste d'un instant ; la parade se tient jusqu'à ce que le coup soit passé.
    #[test]
    fn la_roulade_part_une_fois_la_parade_se_tient() {
        let mut c = Cerveau::default();
        c.tick(&arme(0.0, None, 0.2), &veteran());
        c.tick(&arme(1.0, Some(0.8), 0.2), &veteran());
        c.tick(&arme(1.3, Some(0.5), 0.2), &veteran());
        assert_eq!(c.geste(), Some(Geste::Esquiver));
        c.tick(&arme(1.4, Some(0.4), 0.2), &veteran());
        assert_eq!(c.geste(), None);
        let mut c = Cerveau::default();
        c.tick(&arme(0.0, None, 0.8), &veteran());
        c.tick(&arme(1.0, Some(0.8), 0.8), &veteran());
        c.tick(&arme(1.3, Some(0.5), 0.8), &veteran());
        c.tick(&arme(1.7, Some(0.1), 0.8), &veteran());
        assert_eq!(c.geste(), Some(Geste::Parer));
        // Le coup passé, il baisse sa garde.
        c.tick(&arme(1.9, None, 0.8), &veteran());
        assert_eq!(c.geste(), None);
    }

    /// Un seul tirage par coup armé ; le coup suivant en fait tirer un autre.
    #[test]
    fn un_tirage_par_coup_arme() {
        let hesitant = Profil {
            reactions: HashMap::from([(Evenement::CoupArme, vec![(Reaction::Rien, 1.0), (Reaction::Parer, 1.0)])]),
            ..veteran()
        };
        let mut c = Cerveau::default();
        c.tick(&arme(0.0, None, 0.2), &hesitant);
        // Il tire « rien » : même si le hasard change ensuite, il ne pare pas ce coup-là.
        c.tick(&arme(1.0, Some(0.9), 0.2), &hesitant);
        c.tick(&arme(1.4, Some(0.5), 0.8), &hesitant);
        assert_eq!(c.geste(), None);
        c.tick(&arme(2.0, None, 0.8), &hesitant);
        // Le coup suivant : il tire « parer ».
        c.tick(&arme(3.0, Some(0.9), 0.8), &hesitant);
        c.tick(&arme(3.4, Some(0.5), 0.2), &hesitant);
        assert_eq!(c.geste(), Some(Geste::Parer));
    }

    /// Le cran bas ne sait ni rouler ni parer ; celui qui n'a pas de garde (un arc) ne pare pas.
    #[test]
    fn sans_savoir_faire_pas_de_defense() {
        let maladroit = Profil { roule: Some(false), pare: Some(false), ..veteran() };
        let mut c = Cerveau::default();
        c.tick(&arme(0.0, None, 0.8), &maladroit);
        c.tick(&arme(1.0, Some(0.9), 0.8), &maladroit);
        c.tick(&arme(1.5, Some(0.4), 0.8), &maladroit);
        assert_eq!(c.geste(), None);
        // Sans garde, « parer » ne sort pas : il lui reste la roulade.
        let sans_garde = |temps, dans| Observation { peut_parer: false, ..arme(temps, dans, 0.8) };
        let mut c = Cerveau::default();
        c.tick(&sans_garde(0.0, None), &veteran());
        c.tick(&sans_garde(1.0, Some(0.9)), &veteran());
        c.tick(&sans_garde(1.3, Some(0.6)), &veteran());
        assert_eq!(c.geste(), Some(Geste::Esquiver));
    }

    // --- IA.8 : fuite et coincé ---

    /// Un danari qui, coincé, se tapit (poids 1) ou force le passage (poids 1).
    fn fuyard() -> Profil {
        Profil {
            fuite_memoire: Some(10.0),
            coince_temps: Some(1.0),
            coince_portee: Some(8.0),
            coince_delai: Some(3.0),
            forcer_duree: Some(4.0),
            reactions: HashMap::from([(Evenement::Coince, vec![(Reaction::SeTapir, 1.0), (Reaction::Forcer, 1.0)])]),
            ..danari()
        }
    }

    /// Le danari blessé, à `pos`, qui voit le joueur 1 à `menace`.
    fn blesse(temps: f64, pos: Vec3<f32>, menace: Vec3<f32>, hasard: f32) -> Observation {
        Observation { pv: 0.2, hasard, ..obs_en(temps, pos, vec![Percu { pos: menace, ..joueur(1, 0.0) }]) }
    }

    /// « Le fuyard n'oublie pas » : il fuit encore `fuite_memoire` secondes une menace qu'il ne voit
    /// plus (plus longtemps qu'un combattant ne poursuit), puis se calme là où il est.
    #[test]
    fn le_fuyard_fuit_longtemps_une_menace_qu_il_ne_voit_plus() {
        let mut c = Cerveau::default();
        c.tick(&blesse(0.0, Vec3::zero(), en(3.0, 0.0), 0.0), &fuyard());
        let seul = |temps| Observation { pv: 0.2, ..obs_en(temps, en(-30.0, 0.0), vec![]) };
        assert_eq!(c.tick(&seul(8.0), &fuyard()), Intention::Fuir(en(3.0, 0.0)));
        assert_eq!(c.tick(&seul(11.0), &fuyard()), Intention::Repos);
        assert_eq!(c.tick(&seul(40.0), &fuyard()), Intention::Repos);
    }

    /// « Une créature qui fuit est coincée quand elle essaie d'avancer depuis 1 s sans s'éloigner,
    /// menace à moins de 8 m. Elle tire alors au sort une réaction » : se tapir, ou forcer le passage.
    #[test]
    fn coince_il_se_tapit_ou_force_le_passage() {
        let (coin, menace) = (Vec3::zero(), en(4.0, 0.0));
        let coince = |hasard| {
            let mut c = Cerveau::default();
            assert_eq!(c.tick(&blesse(0.0, coin, menace, hasard), &fuyard()), Intention::Fuir(menace));
            assert_eq!(c.tick(&blesse(0.8, coin, menace, hasard), &fuyard()), Intention::Fuir(menace));
            let intention = c.tick(&blesse(1.2, coin, menace, hasard), &fuyard());
            (c, intention)
        };
        let (c, intention) = coince(0.2);
        assert_eq!(intention, Intention::SeTapir(menace));
        assert_eq!(c.etat(), Etat::Tapi);
        // « Forcer le passage : elle fonce vers un point 6 m derrière la menace, 1,5 m sur le côté. »
        let (c, intention) = coince(0.8);
        let Intention::Forcer(lieu) = intention else { panic!("il doit forcer le passage : {intention:?}") };
        assert_eq!(c.etat(), Etat::Force);
        assert!((lieu.x - 10.0).abs() < 0.01 && (lieu.y.abs() - 1.5).abs() < 0.01, "{lieu:?}");
    }

    /// Il n'est pas coincé s'il s'éloigne, ni si la menace est loin ; sans réaction dans son profil,
    /// « elle continue de fuir comme elle peut ».
    #[test]
    fn pas_coince_s_il_avance_ou_si_la_menace_est_loin() {
        let menace = en(4.0, 0.0);
        let mut c = Cerveau::default();
        c.tick(&blesse(0.0, Vec3::zero(), menace, 0.2), &fuyard());
        assert_eq!(c.tick(&blesse(1.2, en(-5.0, 0.0), menace, 0.2), &fuyard()), Intention::Fuir(menace));
        assert_eq!(c.tick(&blesse(2.4, en(-10.0, 0.0), menace, 0.2), &fuyard()), Intention::Fuir(menace));
        // Bloqué, mais la menace est à 20 blocs.
        let loin = en(20.0, 0.0);
        let mut c = Cerveau::default();
        c.tick(&blesse(0.0, Vec3::zero(), loin, 0.2), &fuyard());
        assert_eq!(c.tick(&blesse(1.2, Vec3::zero(), loin, 0.2), &fuyard()), Intention::Fuir(loin));
        // Un profil sans réaction à « coincé ».
        let mut c = Cerveau::default();
        c.tick(&blesse(0.0, Vec3::zero(), menace, 0.2), &danari());
        assert_eq!(c.tick(&blesse(1.2, Vec3::zero(), menace, 0.2), &danari()), Intention::Fuir(menace));
    }

    /// « Cachée, elle guette ; frappée, elle détale à travers son agresseur. »
    #[test]
    fn tapi_il_guette_et_frappe_il_detale_a_travers_l_agresseur() {
        let (coin, menace) = (Vec3::zero(), en(4.0, 0.0));
        let mut c = Cerveau::default();
        c.tick(&blesse(0.0, coin, menace, 0.2), &fuyard());
        c.tick(&blesse(1.2, coin, menace, 0.2), &fuyard());
        // Il reste tapi, face à la menace, même longtemps.
        assert_eq!(c.tick(&blesse(9.0, coin, en(2.0, 1.0), 0.8), &fuyard()), Intention::SeTapir(en(2.0, 1.0)));
        // On le frappe.
        let frappe = Percu { pos: en(2.0, 0.0), l_a_frappe_depuis: Some(0.1), ..joueur(1, 0.0) };
        let touche = Observation { pv: 0.1, hasard: 0.2, ..obs_en(9.5, coin, vec![frappe]) };
        assert!(matches!(c.tick(&touche, &fuyard()), Intention::Forcer(lieu) if lieu.x > 6.0));
    }

    /// Tapi, la menace partie : « elle se calme sur place ».
    #[test]
    fn tapi_il_se_calme_quand_la_menace_est_partie() {
        let coin = en(7.0, 7.0);
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![]), &fuyard());
        c.tick(&blesse(1.0, coin, en(11.0, 7.0), 0.2), &fuyard());
        assert_eq!(c.tick(&blesse(2.2, coin, en(11.0, 7.0), 0.2), &fuyard()), Intention::SeTapir(en(11.0, 7.0)));
        let seul = |temps| Observation { pv: 0.2, ..obs_en(temps, coin, vec![]) };
        assert_eq!(c.tick(&seul(8.0), &fuyard()), Intention::SeTapir(en(11.0, 7.0)));
        // Plus de menace depuis 10 s : il vaque là, sans rentrer à son ancien poste.
        assert_eq!(c.tick(&seul(13.0), &fuyard()), Intention::Repos);
        assert_eq!(c.etat(), Etat::Repos);
    }

    /// Il force le passage « pendant 4 s au plus », ou jusqu'à la sortie, puis reprend sa fuite ;
    /// « pas de nouvelle réaction avant 3 s ».
    #[test]
    fn le_passage_force_il_reprend_sa_fuite() {
        let (coin, menace) = (Vec3::zero(), en(4.0, 0.0));
        let mut c = Cerveau::default();
        c.tick(&blesse(0.0, coin, menace, 0.8), &fuyard());
        let Intention::Forcer(sortie) = c.tick(&blesse(1.2, coin, menace, 0.8), &fuyard()) else { panic!() };
        // En route, il garde le même but.
        assert_eq!(c.tick(&blesse(2.0, en(3.0, 2.0), menace, 0.8), &fuyard()), Intention::Forcer(sortie));
        // Arrivé : il fuit de nouveau.
        assert_eq!(c.tick(&blesse(3.0, sortie, menace, 0.8), &fuyard()), Intention::Fuir(menace));
        // Bloqué par un joueur : il renonce après 4 s, et ne retire pas de réaction tout de suite.
        let mut c = Cerveau::default();
        c.tick(&blesse(0.0, coin, menace, 0.8), &fuyard());
        c.tick(&blesse(1.2, coin, menace, 0.8), &fuyard());
        assert!(matches!(c.tick(&blesse(5.0, coin, menace, 0.8), &fuyard()), Intention::Forcer(_)));
        assert_eq!(c.tick(&blesse(5.3, coin, menace, 0.8), &fuyard()), Intention::Fuir(menace));
        assert_eq!(c.tick(&blesse(6.0, coin, menace, 0.8), &fuyard()), Intention::Fuir(menace));
        // 3 s après sa dernière réaction (et de nouveau bloqué depuis 1 s) : il en tire une autre.
        assert_eq!(c.tick(&blesse(8.4, coin, menace, 0.2), &fuyard()), Intention::SeTapir(menace));
    }

    // --- IA.9 : appel à l'aide et déroute ---

    /// Un archer : fragile, il appelle à l'aide quand on le frappe sous 75 % de ses PV.
    fn archer() -> Profil {
        Profil {
            appel_pv: Some(0.75),
            appel_delai: Some(6.0),
            reactions: HashMap::from([(Evenement::Touche, vec![(Reaction::AppelerALAide, 1.0)])]),
            ..orc()
        }
    }

    /// Un guerrier : il défend ceux qui appellent, et la mort du chef le met en déroute 4 s.
    fn guerrier() -> Profil {
        Profil {
            defend: Some(true),
            defend_duree: Some(6.0),
            deroute: Some(4.0),
            reactions: HashMap::from([(Evenement::ChefTombe, vec![(Reaction::Deroute, 1.0)])]),
            ..orc()
        }
    }

    /// Lui, à `pv`, face au joueur 1 à 3 blocs qui vient (ou non) de le frapper.
    fn frappe(temps: f64, pv: f32, vient_de_frapper: bool) -> Observation {
        let agresseur = Percu { l_a_frappe_depuis: vient_de_frapper.then_some(0.05), ..joueur(1, 3.0) };
        Observation { pv, ..obs(temps, vec![agresseur]) }
    }

    /// « Les fragiles crient À l'aide quand on les frappe : l'archer sous 75 % de ses PV. » Le
    /// cri dit qui l'a frappé, et où il est.
    #[test]
    fn l_archer_frappe_appelle_a_l_aide() {
        let mut c = Cerveau::default();
        c.tick(&frappe(0.0, 1.0, false), &archer());
        // Un premier coup le laisse à 80 % : il ne crie pas.
        c.tick(&frappe(1.0, 0.8, true), &archer());
        assert!(c.cris().is_empty());
        // Le suivant le fait passer sous 75 %.
        assert_eq!(c.tick(&frappe(2.0, 0.7, true), &archer()), Intention::Combattre(1));
        assert_eq!(c.cris(), vec![Message::Appel { agresseur: 1, lieu: en(3.0, 0.0) }]);
        // Un guerrier, lui, ne crie pas.
        let mut c = Cerveau::default();
        c.tick(&frappe(0.0, 1.0, false), &guerrier());
        c.tick(&frappe(1.0, 0.5, true), &guerrier());
        assert!(c.cris().is_empty());
    }

    /// « Au plus une fois toutes les 6 s. »
    #[test]
    fn un_appel_toutes_les_six_secondes_au_plus() {
        let mut c = Cerveau::default();
        c.tick(&frappe(0.0, 1.0, false), &archer());
        c.tick(&frappe(1.0, 0.7, true), &archer());
        assert_eq!(c.cris().len(), 1);
        c.tick(&frappe(3.0, 0.6, true), &archer());
        assert!(c.cris().is_empty());
        c.tick(&frappe(8.0, 0.5, true), &archer());
        assert_eq!(c.cris().len(), 1);
        // Sans nouveau coup, pas de nouveau cri.
        c.tick(&frappe(20.0, 0.5, false), &archer());
        assert!(c.cris().is_empty());
    }

    /// « Les brutes et le chef défendent : s'ils entendent l'appel, ils prennent l'agresseur pour
    /// cible (même s'ils se battaient contre un autre joueur). » Sans le voir, ils courent là où
    /// l'appel le dit ; après 6 s, ils choisissent de nouveau comme d'habitude.
    #[test]
    fn le_guerrier_qui_entend_l_appel_prend_l_agresseur_pour_cible() {
        let autre = joueur(2, 4.0);
        let agresseur = Percu { pos: en(-20.0, 0.0), ..joueur(1, 0.0) };
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![autre.clone()]), &guerrier()), Intention::Combattre(2));
        c.recevoir(Message::Appel { agresseur: 1, lieu: en(-20.0, 0.0) }, 1.0);
        // Il ne voit pas l'agresseur : il quitte son combat et court là où on l'appelle.
        assert_eq!(c.tick(&obs(1.0, vec![autre.clone()]), &guerrier()), Intention::AllerVoir(en(-20.0, 0.0)));
        // Il le voit : c'est sa cible, même si l'autre joueur est plus près.
        assert_eq!(c.tick(&obs(2.0, vec![autre.clone(), agresseur.clone()]), &guerrier()), Intention::Combattre(1));
        assert_eq!(c.tick(&obs(5.0, vec![autre.clone(), agresseur.clone()]), &guerrier()), Intention::Combattre(1));
        // Passé 6 s, il rechoisit comme d'habitude (au reciblage) : le plus proche.
        assert_eq!(c.tick(&obs(11.0, vec![autre, agresseur]), &guerrier()), Intention::Combattre(2));
    }

    /// « Les autres créatures qui l'entendent viennent seulement voir. »
    #[test]
    fn celui_qui_ne_defend_pas_vient_seulement_voir() {
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![]), &archer());
        c.recevoir(Message::Appel { agresseur: 1, lieu: en(-20.0, 0.0) }, 1.0);
        assert_eq!(c.tick(&obs(1.0, vec![]), &archer()), Intention::AllerVoir(en(-20.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
        // En plein combat, il ne lâche pas sa cible pour autant.
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![joueur(2, 4.0)]), &archer());
        c.recevoir(Message::Appel { agresseur: 1, lieu: en(-20.0, 0.0) }, 1.0);
        assert_eq!(c.tick(&obs(1.0, vec![joueur(2, 4.0)]), &archer()), Intention::Combattre(2));
    }

    /// « Quand le chef tombe […] les créatures qui l'entendent sont en déroute : brute 4 s. En
    /// déroute, une créature fuit l'ennemi le plus proche, même si on la frappe. Ensuite elle se
    /// reprend et va voir où était la menace. »
    #[test]
    fn la_mort_du_chef_met_sa_garde_en_deroute() {
        let chef_tombe = en(0.0, 10.0);
        let mut c = Cerveau::default();
        assert_eq!(c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &guerrier()), Intention::Combattre(1));
        c.recevoir(Message::ChefTombe { lieu: chef_tombe }, 1.0);
        assert_eq!(c.tick(&obs(1.0, vec![joueur(1, 3.0), joueur(2, 9.0)]), &guerrier()), Intention::Fuir(en(3.0, 0.0)));
        assert_eq!(c.etat(), Etat::Deroute);
        assert_eq!(c.cible(), None);
        // On le frappe : il fuit toujours.
        assert_eq!(c.tick(&frappe(3.0, 0.5, true), &guerrier()), Intention::Fuir(en(3.0, 0.0)));
        // Sans plus voir personne, il fuit encore là où était la menace.
        assert_eq!(c.tick(&obs_en(4.5, en(-12.0, 0.0), vec![]), &guerrier()), Intention::Fuir(en(3.0, 0.0)));
        // 4 s après : il se reprend et va voir où était la menace.
        assert_eq!(c.tick(&obs_en(5.5, en(-15.0, 0.0), vec![]), &guerrier()), Intention::AllerVoir(en(3.0, 0.0)));
        assert_eq!(c.etat(), Etat::Enquete);
    }

    /// Sans ennemi en vue, il fuit l'endroit où le chef est tombé ; celui dont le profil dit « tient
    /// bon » (déroute de 0 s, ou pas de réaction) continue.
    #[test]
    fn deroute_sans_ennemi_en_vue_et_ceux_qui_tiennent_bon() {
        let chef_tombe = en(0.0, 10.0);
        let mut c = Cerveau::default();
        c.tick(&obs(0.0, vec![]), &guerrier());
        c.recevoir(Message::ChefTombe { lieu: chef_tombe }, 1.0);
        assert_eq!(c.tick(&obs(1.0, vec![]), &guerrier()), Intention::Fuir(chef_tombe));
        for tient_bon in [Profil { deroute: Some(0.0), ..guerrier() }, Profil { deroute: Some(4.0), ..orc() }] {
            let mut c = Cerveau::default();
            c.tick(&obs(0.0, vec![joueur(1, 3.0)]), &tient_bon);
            c.recevoir(Message::ChefTombe { lieu: chef_tombe }, 1.0);
            assert_eq!(c.tick(&obs(1.0, vec![joueur(1, 3.0)]), &tient_bon), Intention::Combattre(1));
        }
    }

    /// `menace.md`, principe 1 : « les créatures ne trichent pas ». Deux cerveaux qui reçoivent les
    /// mêmes observations décident pareil : rien d'autre n'entre dans la décision.
    #[test]
    fn memes_observations_memes_decisions() {
        let suite = [
            obs(0.0, vec![joueur(1, 10.0)]),
            obs(1.0, vec![]),
            obs_en(3.5, en(6.0, 0.0), vec![]),
            obs_en(5.0, en(9.0, 0.0), vec![]),
            obs_en(12.0, en(9.0, 0.0), vec![]),
        ];
        let (mut a, mut b) = (Cerveau::default(), Cerveau::default());
        for o in &suite {
            assert_eq!(a.tick(o, &orc()), b.tick(o, &orc()));
        }
    }
}
