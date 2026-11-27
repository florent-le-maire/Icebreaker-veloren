//! Icebreaker CA : la carte 3D de l'expédition, côté session. Branchement mince : la logique est dans
//! `common::icebreaker_carte` (ce qu'on découvre, la vue, le dessin). Ici on lui donne le terrain chargé
//! autour du joueur, la souris et la molette, et on rend au HUD l'image à montrer.

use crate::hud::icebreaker_carte::Ouverte;
use client::Client;
use common::{
    comp,
    icebreaker_carte::{Bloc, Carnet, Image, Objectif, Reglages, Vue, dessiner},
    vol::{FilledVox, ReadVol},
};
use specs::WorldExt;
use std::sync::Arc;
use vek::{Rgb, Vec2, Vec3};

/// Couleur d'un bloc plein qui n'en a pas.
const SANS_COULEUR: Rgb<u8> = Rgb::new(110, 110, 110);
/// Secondes au moins entre deux dessins de la carte ouverte.
const ENTRE_DESSINS: f32 = 0.05;

/// Sans bouger, on ne regarde autour de soi qu'une période sur tant.
const SUR_PLACE: f32 = 6.0;

/// Pourquoi la carte ne s'ouvre pas (clé du message à dire au joueur).
pub const HORS_EXPEDITION: &str = "icebreaker-carte-hors-expedition";
pub const ABSENTE: &str = "icebreaker-carte-absente";

pub struct Carte {
    reglages: Reglages,
    carnet: Carnet,
    ouverte: bool,
    vue: Vue,
    /// Secondes depuis la dernière découverte, et d'où elle a été faite.
    attente: f32,
    explore_depuis: Option<Vec3<i32>>,
    /// Le dernier dessin, sa version, d'où il a été fait ; `a_dessiner` : il n'est plus à jour.
    image: Option<Arc<Image>>,
    version: u64,
    dessine_depuis: Vec3<f32>,
    depuis_dessin: f32,
    a_dessiner: bool,
}

impl Carte {
    pub fn charger() -> Self {
        let reglages = Reglages::charger();
        Self {
            vue: Vue::nouvelle(&reglages),
            reglages,
            carnet: Carnet::default(),
            ouverte: false,
            attente: f32::MAX,
            explore_depuis: None,
            image: None,
            version: 0,
            dessine_depuis: Vec3::zero(),
            depuis_dessin: f32::MAX,
            a_dessiner: true,
        }
    }

    pub fn ouverte(&self) -> bool { self.ouverte }

    /// Le serveur dit les objectifs du lieu (`None` : pas en expédition, la carte se ferme).
    pub fn recevoir(&mut self, objectifs: Option<Vec<Objectif>>) {
        self.carnet.recevoir(objectifs);
        self.explore_depuis = None;
        self.a_dessiner = true;
        if !self.carnet.en_expedition() {
            self.ouverte = false;
        }
    }

    /// Ouvre ou ferme la carte ; elle s'ouvre tournée dans le sens où le personnage regarde. `Err` : la
    /// clé du message qui dit pourquoi elle ne s'ouvre pas.
    pub fn basculer(&mut self, client: &Client) -> Result<(), &'static str> {
        if self.ouverte {
            self.ouverte = false;
            return Ok(());
        }
        if !self.carnet.en_expedition() {
            return Err(HORS_EXPEDITION);
        }
        let dans_le_sac = client
            .inventories()
            .get(client.entity())
            .is_some_and(|inventaire| inventaire.slots().flatten().any(common::icebreaker_carte::est_carte));
        if !dans_le_sac {
            return Err(ABSENTE);
        }
        self.vue = Vue::nouvelle(&self.reglages);
        self.vue.face_a(regard(client));
        self.ouverte = true;
        self.a_dessiner = true;
        Ok(())
    }

    pub fn fermer(&mut self) { self.ouverte = false; }

    /// La souris fait tourner la carte ouverte.
    pub fn souris(&mut self, delta: Vec2<f32>) {
        self.vue.tourner(delta, &self.reglages);
        self.a_dessiner = true;
    }

    /// La molette zoome (crans positifs : plus près).
    pub fn molette(&mut self, crans: f32) {
        self.vue.zoomer(crans, &self.reglages);
        self.a_dessiner = true;
    }

    /// Pour les captures sans les mains : la carte tournée à ce lacet (degrés).
    pub fn tourner_a(&mut self, lacet: f32) {
        self.vue.lacet = lacet.to_radians();
        self.a_dessiner = true;
    }

    /// Chaque image du jeu : en expédition on découvre ce qui est autour de soi, carte ouverte ou non ;
    /// ouverte, on la redessine quand elle a changé ou qu'on a bougé.
    pub fn tic(&mut self, dt: f32, client: &Client) {
        if !self.carnet.en_expedition() {
            return;
        }
        let Some(pos) = client.position() else {
            return;
        };
        self.attente = (self.attente + dt).min(f32::MAX / 2.0);
        // La tête : les pieds peuvent être dans un bloc bas (dalle, marche).
        let tete = (pos + Vec3::unit_z()).map(|e| e.floor() as i32);
        // Sans bouger, on regarde quand même de temps en temps : le terrain finit de se charger.
        let periode = self.reglages.periode;
        if self.attente >= periode && (self.explore_depuis != Some(tete) || self.attente >= periode * SUR_PLACE) {
            self.attente = 0.0;
            self.explore_depuis = Some(tete);
            let terrain = client.state().terrain();
            let bloc = |p: Vec3<i32>| match terrain.get(p) {
                Ok(bloc) if bloc.is_filled() => Bloc::Plein(bloc.get_color().unwrap_or(SANS_COULEUR)),
                Ok(_) => Bloc::Air,
                Err(_) => Bloc::Inconnu,
            };
            if self.carnet.exploration.explorer(tete, bloc, &self.reglages) {
                self.a_dessiner = true;
            }
        }
        if !self.ouverte {
            return;
        }
        self.depuis_dessin = (self.depuis_dessin + dt).min(f32::MAX / 2.0);
        if pos.distance_squared(self.dessine_depuis) > 0.01 {
            self.a_dessiner = true;
        }
        if (self.a_dessiner || self.image.is_none()) && self.depuis_dessin >= ENTRE_DESSINS {
            self.image = Some(Arc::new(dessiner(
                &self.carnet.exploration,
                self.carnet.objectifs(),
                pos,
                regard(client),
                &self.vue,
                &self.reglages,
            )));
            self.version += 1;
            self.dessine_depuis = pos;
            self.depuis_dessin = 0.0;
            self.a_dessiner = false;
        }
    }

    /// Ce que le HUD montre de la carte ouverte.
    pub fn vue_hud(&self) -> Option<Ouverte> {
        let image = self.image.clone().filter(|_| self.ouverte)?;
        let objectifs = self.carnet.objectifs();
        Some(Ouverte {
            image,
            version: self.version,
            objectifs: objectifs.len(),
            restants: objectifs.iter().filter(|objectif| !objectif.fait).count(),
        })
    }
}

/// Où regarde le personnage, à plat.
fn regard(client: &Client) -> Vec2<f32> {
    client
        .state()
        .ecs()
        .read_storage::<comp::Ori>()
        .get(client.entity())
        .map_or(Vec2::unit_y(), |ori| ori.look_vec().xy())
}
