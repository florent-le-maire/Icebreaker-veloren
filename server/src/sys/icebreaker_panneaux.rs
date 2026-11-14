use crate::{
    client::Client,
    icebreaker_buildings::{self, Building, IcebreakerBuildings, Place, Poste},
    icebreaker_expedition,
    icebreaker_panneaux::{self, IcebreakerPanneaux, PERIODE},
    icebreaker_registre::IcebreakerRegistre,
    icebreaker_salle,
    icebreaker_services::IcebreakerServices,
    icebreaker_treasury::IcebreakerTreasury,
    icebreaker_village::{self, IcebreakerVillage},
    icebreaker_warehouse::IcebreakerWarehouse,
};
use common::{
    comp::{Health, Inventory, ItemKey, Player, Pos, Presence, PresenceKind},
    icebreaker_competences::{IcebreakerCompetences, SalleArmes},
    resources::{Time, TimeOfDay},
    uid::Uid,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::ServerGeneral;
use specs::{Entities, Read, ReadStorage, Write};

/// Icebreaker : le plan du village et le panneau d'un bâtiment (jalon N). Pour
/// chaque joueur qui a le plan ouvert, les bâtiments et le panneau sont
/// reconstruits quand il vient d'agir (une commande), sinon toutes les
/// `PERIODE` secondes, et ne lui sont envoyés que s'ils ont changé : un clic a
/// sa réponse au tic suivant, et rien ne part quand rien ne bouge.
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        Read<'a, TimeOfDay>,
        Write<'a, IcebreakerPanneaux>,
        Read<'a, IcebreakerBuildings>,
        Read<'a, IcebreakerWarehouse>,
        Read<'a, IcebreakerTreasury>,
        Read<'a, IcebreakerServices>,
        Read<'a, IcebreakerVillage>,
        Write<'a, IcebreakerRegistre>,
        ReadStorage<'a, Client>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, Presence>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Inventory>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, IcebreakerCompetences>,
        // Icebreaker O : ce qui est en piste au lieu d'entraînement.
        ReadStorage<'a, common::comp::Agent>,
        ReadStorage<'a, common::comp::Alignment>,
    );

    const NAME: &'static str = "icebreaker_panneaux";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            time,
            time_of_day,
            mut panneaux,
            buildings,
            warehouse,
            treasury,
            services,
            village,
            mut registre,
            clients,
            uids,
            presences,
            players,
            positions,
            inventories,
            healths,
            competences,
            agents,
            alignements,
        ): Self::SystemData,
    ) {
        if panneaux.ouverts.is_empty() {
            return;
        }
        let (now, carte) = (time.0, time_of_day.0);
        // Ceux qui sont partis, ou revenus à l'écran des personnages, n'ont plus rien d'ouvert.
        panneaux
            .ouverts
            .retain(|entity, _| entities.is_alive(*entity) && clients.contains(*entity) && presences.contains(*entity));
        let config = icebreaker_village::config();
        let places = buildings.places(village.open_plots());
        let pieces = crate::icebreaker_services::coins();
        let salle = SalleArmes::actuel();
        let prix_soin = u64::from(services.heal_price());
        let inscrits = icebreaker_expedition::resume(now).map(|expedition| expedition.membres.len());

        for (&entity, ouvert) in panneaux.ouverts.iter_mut() {
            if !ouvert.a_revoir && now - ouvert.vu < PERIODE {
                continue;
            }
            ouvert.a_revoir = false;
            ouvert.vu = now;
            let Some(client) = clients.get(entity) else {
                continue;
            };
            // Hors du village (un emplacement de donjon de la campagne) : aucun
            // bâtiment, ce qui fait quitter le plan au client (X.1).
            let au_village = positions
                .get(entity)
                .is_some_and(|pos| buildings.in_village(pos.0.xy()));
            let liste = if au_village {
                icebreaker_buildings::liste(
                    &places,
                    &village,
                    &config,
                    treasury.coins(),
                    prix_soin,
                    registre.charrette().len(),
                    inscrits,
                    carte,
                )
            } else {
                Vec::new()
            };
            if ouvert.batiments.as_ref() != Some(&liste) {
                client.send_fallible(ServerGeneral::IcebreakerBuildings(liste.clone()));
                ouvert.batiments = Some(liste);
            }
            let Some(place) = ouvert.place.filter(|_| au_village) else {
                continue;
            };

            let fantome = !matches!(
                presences.get(entity).map(|presence| presence.kind),
                Some(PresenceKind::Character(_))
            );
            let bourse = inventories
                .get(entity)
                .map_or(0, |inventory| inventory.item_count(&pieces));
            let panneau = match place {
                Place::Plot(rang) => match village.chantier(rang) {
                    Some(chantier) => icebreaker_panneaux::chantier(
                        icebreaker_buildings::hours_left(chantier.fin, carte),
                        config.heures_chantier,
                    ),
                    None => icebreaker_panneaux::terrain(
                        rang,
                        u64::from(config.prix_maison),
                        config.heures_chantier,
                        treasury.coins(),
                    ),
                },
                Place::Building(Building::Warehouse) => icebreaker_panneaux::entrepot(
                    warehouse
                        .load_items()
                        .iter()
                        .map(|item| (ItemKey::from(item), item.amount()))
                        .collect(),
                    fantome,
                ),
                Place::Building(Building::Treasury) => {
                    icebreaker_panneaux::caisse(treasury.coins(), bourse, u64::from(config.prix_maison))
                },
                Place::Building(Building::Intendance) => {
                    icebreaker_panneaux::intendance(bourse, fantome)
                },
                Place::Building(Building::Infirmary) => icebreaker_panneaux::infirmerie(
                    prix_soin,
                    bourse,
                    healths
                        .get(entity)
                        .filter(|_| !fantome)
                        .map(|health| (health.current(), health.maximum())),
                ),
                Place::Building(Building::Departure) => {
                    let Some(uid) = uids.get(entity) else {
                        continue;
                    };
                    icebreaker_expedition::panneau_pour(*uid, fantome, now)
                },
                Place::Building(Building::Charrette) => icebreaker_buildings::charrette_panneau(
                    &mut registre,
                    carte,
                    presences.get(entity),
                    players.get(entity),
                    &presences,
                    &players,
                ),
                // Icebreaker O : les deux panneaux du lieu d'entraînement.
                Place::Arene(Poste::Personnage) => crate::icebreaker_arene::panneau_personnage(
                    competences.get(entity),
                    inventories.get(entity),
                    &crate::icebreaker_classes::classes(),
                    ouvert.salle_message.clone(),
                ),
                Place::Arene(Poste::Arene) => {
                    use specs::{Join, LendJoin};
                    let en_piste = (&agents, (&alignements).maybe()).join().fold(
                        (0, 0),
                        |(ennemis, allies), (_, alignement)| match alignement {
                            Some(common::comp::Alignment::Owned(_)) => (ennemis, allies + 1),
                            _ => (ennemis + 1, allies),
                        },
                    );
                    crate::icebreaker_arene::panneau_arene(
                        &common::icebreaker_arene::config(),
                        en_piste,
                        ouvert.arene_choix,
                        ouvert.salle_message.clone(),
                    )
                },
                Place::Building(Building::SalleArmes) => icebreaker_salle::panneau(
                    competences.get(entity),
                    inventories.get(entity),
                    bourse,
                    fantome,
                    &salle,
                    ouvert.salle_message.clone(),
                ),
            };
            if ouvert.panneau.as_ref() != Some(&panneau) {
                client.send_fallible(ServerGeneral::IcebreakerPanel(panneau.clone()));
                ouvert.panneau = Some(panneau);
            }
        }
    }
}
