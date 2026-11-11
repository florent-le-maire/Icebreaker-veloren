//! Icebreaker : le panneau typé de l'interface (jalon N, décision 0019).
//! Le serveur n'envoie plus des phrases et des boutons, mais des blocs que le
//! client range toujours au même endroit : en-tête, onglets, chiffres, jauge,
//! objets, lignes, actions, message. Un panneau se compare (`PartialEq`) : le
//! serveur ne le renvoie que s'il a changé.

use crate::comp::{Content, ItemKey};
use serde::{Deserialize, Serialize};
use vek::*;

/// Un texte écrit tel quel.
pub fn texte(texte: impl Into<String>) -> Content { Content::Plain(texte.into()) }

/// La teinte d'un chiffre, d'une jauge ou d'une pastille.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ton {
    #[default]
    Normal,
    /// L'or : ce qui se compte en pièces, ce qui est au joueur.
    Or,
    Bon,
    Mauvais,
    Info,
    Discret,
}

/// Un chiffre qui compte, en grand, et sa légende dessous.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chiffre {
    pub valeur: Content,
    pub legende: Content,
    pub ton: Ton,
}

impl Chiffre {
    pub fn nouveau(valeur: impl Into<String>, legende: impl Into<String>, ton: Ton) -> Self {
        Self {
            valeur: texte(valeur),
            legende: texte(legende),
            ton,
        }
    }
}

/// Une jauge : sa légende à gauche, son texte à droite, sa part remplie (0 à 1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Jauge {
    pub legende: Content,
    pub texte: Content,
    pub part: f32,
    pub ton: Ton,
}

/// Un bouton : la commande de chat qu'il envoie, s'il est l'action principale,
/// et s'il est permis (sinon il est grisé, la raison est dans le message du
/// panneau). `couleur` : une pastille de cette couleur à la place du libellé
/// (balise).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub libelle: Content,
    pub commande: Vec<String>,
    pub principale: bool,
    pub permise: bool,
    pub couleur: Option<Rgb<u8>>,
}

impl Action {
    pub fn nouvelle(libelle: impl Into<String>, commande: &[&str]) -> Self {
        Self {
            libelle: texte(libelle),
            commande: commande.iter().map(|mot| mot.to_string()).collect(),
            principale: false,
            permise: true,
            couleur: None,
        }
    }

    pub fn principale(mut self) -> Self {
        self.principale = true;
        self
    }

    pub fn permise(mut self, permise: bool) -> Self {
        self.permise = permise;
        self
    }

    pub fn couleur(mut self, couleur: Rgb<u8>) -> Self {
        self.couleur = Some(couleur);
        self
    }
}

/// Une ligne de liste : un personnage, une destination, un inscrit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ligne {
    /// Nom d'icône du HUD (classe, bâtiment), ou vide.
    pub icone: String,
    pub titre: Content,
    pub sous_titre: Content,
    /// L'état, en un mot dans un cadre.
    pub pastille: Option<(Content, Ton)>,
    /// Le cadenas (réserver) : fermé ou non, et la commande qu'il envoie.
    pub cadenas: Option<(bool, Vec<String>)>,
    pub action: Option<Action>,
    /// Cadre doré : la ligne du joueur.
    pub choisie: bool,
    /// En gris : rien à y faire.
    pub estompee: bool,
}

impl Ligne {
    pub fn nouvelle(titre: impl Into<String>) -> Self {
        Self {
            icone: String::new(),
            titre: texte(titre),
            sous_titre: texte(""),
            pastille: None,
            cadenas: None,
            action: None,
            choisie: false,
            estompee: false,
        }
    }

    pub fn icone(mut self, icone: impl Into<String>) -> Self {
        self.icone = icone.into();
        self
    }

    pub fn sous_titre(mut self, sous_titre: impl Into<String>) -> Self {
        self.sous_titre = texte(sous_titre);
        self
    }

    pub fn pastille(mut self, mot: impl Into<String>, ton: Ton) -> Self {
        self.pastille = Some((texte(mot), ton));
        self
    }

    pub fn cadenas(mut self, ferme: bool, commande: &[&str]) -> Self {
        self.cadenas = Some((ferme, commande.iter().map(|mot| mot.to_string()).collect()));
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    pub fn choisie(mut self) -> Self {
        self.choisie = true;
        self
    }

    pub fn estompee(mut self) -> Self {
        self.estompee = true;
        self
    }
}

/// Un onglet et ses lignes ; le client change d'onglet sans rien demander.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Onglet {
    pub libelle: Content,
    pub lignes: Vec<Ligne>,
}

/// La sorte d'un message : sa couleur et son mot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sorte {
    #[default]
    Info,
    Reussite,
    Refus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Panneau {
    /// Nom d'icône du HUD (celles de `vue_village.ron`), ou vide.
    pub icone: String,
    pub titre: Content,
    /// Ce que c'est, en une ligne.
    pub description: Content,
    /// Vide : pas d'onglets, les `lignes` sont montrées.
    pub onglets: Vec<Onglet>,
    /// L'onglet montré à l'ouverture.
    pub onglet: u8,
    pub chiffres: Vec<Chiffre>,
    pub jauge: Option<Jauge>,
    /// Des objets et leur quantité, en cases.
    pub objets: Vec<(ItemKey, u32)>,
    pub lignes: Vec<Ligne>,
    pub actions: Vec<Action>,
    /// Sur combien de colonnes les actions se rangent.
    pub colonnes: u8,
    /// Pourquoi une action est grisée, ou ce qu'il faut savoir.
    pub message: Option<(Content, Sorte)>,
    /// Montré hors du plan du village (choix de la couleur de la balise).
    pub libre: bool,
}

impl Panneau {
    pub fn nouveau(
        icone: impl Into<String>,
        titre: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            icone: icone.into(),
            titre: texte(titre),
            description: texte(description),
            onglets: Vec::new(),
            onglet: 0,
            chiffres: Vec::new(),
            jauge: None,
            objets: Vec::new(),
            lignes: Vec::new(),
            actions: Vec::new(),
            colonnes: 1,
            message: None,
            libre: false,
        }
    }

    pub fn message(mut self, message: impl Into<String>, sorte: Sorte) -> Self {
        self.message = Some((texte(message), sorte));
        self
    }
}

/// Ce que le joueur fait dans le plan du village.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Demande {
    /// Le plan s'ouvre : le serveur envoie les bâtiments, puis ce qui change.
    Ouvrir,
    /// Un clic sur le bâtiment à cette position du monde : son panneau.
    Batiment(Vec2<f32>),
    /// Le panneau se ferme, le plan reste.
    FermerPanneau,
    /// Le plan se ferme : plus rien n'est envoyé.
    Fermer,
}

/// Un bâtiment ou un terrain du plan du village.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Batiment {
    pub nom: Content,
    /// Son emprise, en blocs du monde.
    pub emprise: Aabr<i32>,
    pub altitude: i32,
    /// Sa sorte (`vue_village.ron`) : son icône et sa couleur.
    pub sorte: String,
    /// Ce qu'il faut en savoir sans l'ouvrir (« 420 or », « fini dans 4 h »).
    pub etat: Content,
}

/// L'expédition en préparation au village.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Expedition {
    pub destination: String,
    /// Secondes avant le départ, une fois tout le monde prêt.
    pub depart_dans: Option<f32>,
    /// Chaque inscrit : pseudo, prêt ou non.
    pub membres: Vec<(String, bool)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_action_est_permise_et_secondaire_par_defaut() {
        let action = Action::nouvelle("Verser 10 or", &["caisse", "10"]);
        assert!(action.permise && !action.principale);
        assert_eq!(action.commande, vec!["caisse".to_string(), "10".to_string()]);
        let grisee = action.clone().principale().permise(false);
        assert!(grisee.principale && !grisee.permise);
    }

    #[test]
    fn deux_panneaux_pareils_sont_egaux_et_un_chiffre_change_les_distingue() {
        let panneau = |or: u64| {
            let mut p = Panneau::nouveau("lingot", "Caisse", "Elle paie les chantiers.");
            p.chiffres.push(Chiffre::nouveau(format!("{or} or"), "dans la caisse", Ton::Or));
            p
        };
        assert_eq!(panneau(420), panneau(420));
        assert_ne!(panneau(420), panneau(470));
    }
}
