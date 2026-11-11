#![deny(unsafe_code)]
#![deny(clippy::clone_on_ref_ptr)]

use client_i18n::LocalizationHandle;
use common::{clock::Clock, comp};
use std::{
    io,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};
use tokio::runtime::Runtime;
use tracing::{error, info};
use veloren_client::{
    Client, ClientType, Credentials, Event, StaticCredentialProvider, addr::ConnectionArgs,
};
use voxygen_i18n_helpers::localize_chat_message;

const TPS: u64 = 10; // Low value is okay, just reading messages.

fn read_input() -> String {
    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read input");

    buffer.trim().to_string()
}

fn main() {
    // Initialize logging.
    common_frontend::init_stdout(None);

    info!("loading localisation");

    let localisation = LocalizationHandle::load_expect("en");

    info!("Starting chat-cli...");

    // Set up an fps clock.
    let mut clock = Clock::new(Duration::from_secs_f64(1.0 / TPS as f64));

    println!("Enter your username");
    let username = read_input();

    println!("Enter the server address");
    let server_addr = read_input();

    println!("Enter your password");
    let password = read_input();

    let runtime = Arc::new(Runtime::new().unwrap());
    let runtime2 = Arc::clone(&runtime);
    let addr = ConnectionArgs::Tcp {
        prefer_ipv6: false,
        hostname: server_addr,
    };

    let provider = StaticCredentialProvider {
        credentials_by_auth: [
            (
                Some("https://auth.veloren.net".to_string()),
                Credentials::UsernamePassword(username.to_string(), password.to_string()),
            ),
            // Icebreaker : accepte aussi les serveurs sans authentification
            // (`--no-auth`, ceux du lanceur).
            (
                None,
                Credentials::UsernamePassword(username.to_string(), password.to_string()),
            ),
        ]
        .into(),
    };

    // Create a client.
    let mut client = runtime
        .block_on(Client::new(
            addr,
            runtime2,
            None,
            &provider,
            &|_| {},
            |_| {},
            Default::default(),
            // Icebreaker : `ICEBREAKER_JOUER` pour entrer dans le monde (ligne `!jouer`).
            if std::env::var_os("ICEBREAKER_JOUER").is_some() {
                ClientType::Game
            } else {
                ClientType::ChatOnly
            },
        ))
        .expect("Failed to create client instance");

    println!("Server info: {:?}", client.server_info());

    let mut player_printed = false;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            let msg = read_input();
            tx.send(msg).unwrap();
        }
    });

    // Icebreaker : `!jouer` entre dans le monde avec le premier personnage du
    // compte (créé s'il n'y en a pas), pour tester les commandes qui en veulent un.
    let mut joining = false;
    let mut created = false;
    // Icebreaker : le personnage du registre à jouer (ou un fantôme) une fois sorti
    // du monde (jalon R).
    let mut registre_next: Option<Option<common::character::CharacterId>> = None;
    // Icebreaker : les bâtiments du plan du village reçus en dernier (`!panneau <sorte>`).
    let mut batiments: Vec<common::icebreaker_panneau::Batiment> = Vec::new();

    loop {
        for msg in rx.try_iter() {
            // Icebreaker : « /commande args » part en commande, pour tester les
            // commandes du serveur et des plugins.
            if msg.is_empty() {
                continue;
            } else if msg == "!jouer" {
                client.load_character_list();
                joining = true;
            } else if msg == "!fantome" {
                // Icebreaker : entrer en fantôme (spectateur), comme au village avec le
                // registre (jalon R).
                client.request_spectate(common::ViewDistances {
                    terrain: 5,
                    entity: 5,
                });
            } else if msg == "!batiments" {
                // Icebreaker : ouvre le plan du village (nom, emprise, état des bâtiments), puis
                // `!panneau x y` ; le serveur envoie ensuite ce qui change (jalon N). `!fermer`
                // ferme le plan.
                client.request_icebreaker_plan(common::icebreaker_panneau::Demande::Ouvrir);
            } else if let Some(rest) = msg.strip_prefix("!panneau ") {
                // `!panneau x y`, ou `!panneau <sorte>` (caisse, charrette, depart…) d'après la
                // dernière liste de bâtiments reçue.
                let mut xy = rest.split_whitespace().filter_map(|w| w.parse::<f32>().ok());
                let ou = match (xy.next(), xy.next()) {
                    (Some(x), Some(y)) => Some(vek::Vec2::new(x, y)),
                    _ => batiments
                        .iter()
                        .find(|b| b.sorte == rest.trim())
                        .map(|b| (b.emprise.min + b.emprise.max).as_::<f32>() / 2.0),
                };
                match ou {
                    Some(ou) => client.request_icebreaker_plan(
                        common::icebreaker_panneau::Demande::Batiment(ou),
                    ),
                    None => println!("Pas de bâtiment « {} » (d'abord !batiments)", rest.trim()),
                }
            } else if msg == "!fermer" {
                client.request_icebreaker_plan(common::icebreaker_panneau::Demande::Fermer);
            } else if msg == "!echange" {
                // Icebreaker : l'échange en cours, s'il y en a un.
                println!("Échange en cours : {}", client.pending_trade().is_some());
            } else if msg == "!etat" {
                // Icebreaker : PV, à terre ou mort (jalon T).
                use veloren_client::WorldExt;
                let ecs = client.state().ecs();
                let healths = ecs.read_storage::<comp::Health>();
                let states = ecs.read_storage::<comp::CharacterState>();
                let health = healths.get(client.entity());
                println!(
                    "État : PV {:?}/{:?}, à terre {}, mort {}, position {:?}",
                    health.map(|h| h.current()),
                    health.map(|h| h.maximum()),
                    comp::is_downed(health, states.get(client.entity())),
                    health.is_some_and(|h| h.is_dead),
                    client.position().map(|p| p.map(|e| e.round())),
                );
            } else if msg == "!relever" || msg == "!fouiller" {
                // Icebreaker : relever le joueur à terre ou fouiller le sac le plus proche
                // (jalon T).
                use veloren_client::{Join, WorldExt};
                let cible = {
                    let ecs = client.state().ecs();
                    let me = client.position().unwrap_or_default();
                    let healths = ecs.read_storage::<comp::Health>();
                    let states = ecs.read_storage::<comp::CharacterState>();
                    let items = ecs.read_storage::<comp::PickupItem>();
                    (&ecs.entities(), &ecs.read_storage::<comp::Pos>())
                        .join()
                        .filter(|(e, _)| *e != client.entity())
                        .filter(|(e, _)| {
                            if msg == "!relever" {
                                comp::is_downed(healths.get(*e), states.get(*e))
                            } else {
                                items.get(*e).is_some_and(|i| common::icebreaker_mort::est_sac(i.item()))
                            }
                        })
                        .min_by_key(|(_, p)| (p.0.distance(me) * 100.0) as i64)
                        .map(|(e, p)| (e, p.0.distance(me)))
                };
                match cible {
                    Some((e, d)) => {
                        println!("Cible à {d:.1} blocs");
                        if msg == "!relever" { client.help_downed(e) } else { client.pick_up(e) }
                    },
                    None => println!("Aucune cible"),
                }
            } else if msg == "!balises" || msg == "!ramasser" {
                // Icebreaker : les balises lumineuses posées (couleur, distance) ; « !ramasser »
                // ramasse la plus proche, comme E (jalon J).
                use veloren_client::{Join, WorldExt};
                let proche = {
                    let ecs = client.state().ecs();
                    let me = client.position().unwrap_or_default();
                    let items = ecs.read_storage::<comp::PickupItem>();
                    let lumieres = ecs.read_storage::<comp::LightEmitter>();
                    let mut balises = (&ecs.entities(), &ecs.read_storage::<comp::Pos>(), &items, &lumieres)
                        .join()
                        .filter(|(_, _, i, _)| common::icebreaker_balise::est_balise(i.item()))
                        .map(|(e, p, _, l)| (e, p.0.distance(me), l.col))
                        .collect::<Vec<_>>();
                    balises.sort_by(|a, b| a.1.total_cmp(&b.1));
                    for (_, d, col) in &balises {
                        println!("Balise à {d:.1} blocs, couleur {col:?}");
                    }
                    println!("{} balise(s)", balises.len());
                    balises.first().map(|(e, _, _)| *e)
                };
                if msg == "!ramasser" && let Some(e) = proche {
                    client.pick_up(e);
                }
            } else if msg == "!revivre" {
                // Icebreaker : la touche de réapparition après la mort (jalon T).
                client.respawn();
            } else if msg == "!sac" {
                // Icebreaker : ce que porte le personnage.
                use veloren_client::WorldExt;
                let inventories = client.state().ecs().read_storage::<comp::Inventory>();
                if let Some(inventory) = inventories.get(client.entity()) {
                    for item in inventory.slots().flatten() {
                        println!("Sac : {} x {:?}", item.amount(), item.item_definition_id());
                    }
                    for item in inventory.equipped_items() {
                        println!("Équipé : {:?}", item.item_definition_id());
                    }
                }
            } else if msg == "!voisins" {
                // Icebreaker : les PNJ à moins de 100 blocs, leur position et leur haut
                // (couché : le haut n'est plus vertical).
                use veloren_client::{Join, WorldExt};
                let ecs = client.state().ecs();
                let me = ecs.read_storage::<comp::Pos>().get(client.entity()).map(|p| p.0);
                let stats = ecs.read_storage::<comp::Stats>();
                for (entity, pos, ori, _) in (
                    &ecs.entities(),
                    &ecs.read_storage::<comp::Pos>(),
                    &ecs.read_storage::<comp::Ori>(),
                    &ecs.read_storage::<comp::Body>(),
                )
                    .join()
                    .filter(|(_, pos, _, _)| me.is_some_and(|me| me.distance(pos.0) < 100.0 && me != pos.0))
                {
                    // X.8 : le nom, pour voir les recrues de la charrette.
                    let nom = stats.get(entity).map(|s| format!("{:?}", s.name)).unwrap_or_default();
                    println!("Voisin {nom} en {:?} : haut {:?}", pos.0.map(|e| e.round()), ori.up().map(|e| (e * 10.0).round() / 10.0));
                }
            } else if let Some((action, pos)) = msg.split_once(' ').and_then(|(action, rest)| {
                let mut xyz = rest.split_whitespace().filter_map(|w| w.parse().ok());
                Some((action, vek::Vec3::new(xyz.next()?, xyz.next()?, xyz.next()?)))
            }) && matches!(action, "!ouvrir" | "!bloc")
            {
                // Icebreaker : `!ouvrir x y z` ramasse ou ouvre le bloc (serrure,
                // coffre), `!bloc x y z` dit ce que le client y voit.
                if action == "!ouvrir" {
                    client.collect_block(pos);
                } else {
                    use common::vol::ReadVol;
                    let block = client.state().terrain().get(pos).ok().copied();
                    println!("Bloc {pos:?} : {:?}", block.map(|b| (b.kind(), b.get_sprite())));
                }
            } else if let Some(rest) = msg.strip_prefix("!sprites ") {
                // Icebreaker : `!sprites x y r` liste les sprites à moins de r blocs (et ±25 en
                // hauteur), pour voir les enseignes des bâtiments (X.4).
                use common::vol::ReadVol;
                let mut xyr = rest.split_whitespace().filter_map(|w| w.parse::<i32>().ok());
                if let (Some(x), Some(y), Some(r)) = (xyr.next(), xyr.next(), xyr.next()) {
                    let terrain = client.state().terrain();
                    let z0 = client.position().map_or(0, |p| p.z as i32);
                    for dx in -r..=r {
                        for dy in -r..=r {
                            for dz in -25..=25 {
                                let pos = vek::Vec3::new(x + dx, y + dy, z0 + dz);
                                if let Some(sprite) = terrain.get(pos).ok().and_then(|b| b.get_sprite())
                                    && sprite != common::terrain::SpriteKind::Empty
                                {
                                    let dessous = terrain.get(pos - vek::Vec3::unit_z()).ok().map(|b| b.kind());
                                    println!("Sprite {sprite:?} en {pos:?}, dessous {dessous:?}");
                                }
                            }
                        }
                    }
                }
            } else if let Some(command) = msg.strip_prefix('/') {
                let mut words = command.split_whitespace().map(str::to_string);
                if let Some(name) = words.next() {
                    client.send_command(name, words.collect());
                }
            } else {
                client.send_chat(msg)
            }
        }

        if let Some(next) = registre_next
            && client.presence().is_none()
        {
            let vd = common::ViewDistances {
                terrain: 5,
                entity: 5,
            };
            match next {
                Some(id) => {
                    println!("Registre : on joue le personnage {id:?}");
                    client.request_character(id, vd);
                },
                None => client.request_spectate(vd),
            }
            registre_next = None;
        }

        if joining && !client.character_list().loading {
            let first = client.character_list().characters.first().and_then(|c| c.character.id);
            if let Some(id) = first {
                println!("Entrée dans le monde avec le personnage {id:?}");
                client.request_character(id, common::ViewDistances {
                    terrain: 5,
                    entity: 5,
                });
                joining = false;
            } else if !created {
                let body = comp::body::humanoid::Body::random();
                client.create_character(
                    username.clone(),
                    Some("common.items.weapons.sword.starter".to_string()),
                    None,
                    body.into(),
                    false,
                    None,
                );
                client.load_character_list();
                created = true;
            }
        }

        let events = match client.tick(comp::ControllerInputs::default(), clock.game_dt()) {
            Ok(events) => events,
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            },
        };

        const SHOW_NAME: bool = false;
        for event in events {
            // Icebreaker : le bilan de retour se distingue du panneau par son préfixe (N.8).
            let bilan = matches!(event, Event::IcebreakerBilan(_));
            match event {
                Event::Chat(m) => println!(
                    "{}",
                    localize_chat_message(
                        &m,
                        &client.lookup_msg_context(&m),
                        &localisation.read(),
                        SHOW_NAME,
                    )
                    .1
                ),
                Event::Disconnect => {}, // TODO
                Event::IcebreakerFantome => println!("Registre ouvert : !fantome pour entrer"),
                Event::IcebreakerBaliseRecharge(reste) => println!("Balise : recharge {reste} s"),
                Event::StartSpectate(pos) => println!("Fantôme en {:?}", pos.map(|e| e.round())),
                Event::IcebreakerBuildings(buildings) => {
                    if buildings.is_empty() {
                        println!("Aucun bâtiment : pas de village ici");
                    }
                    batiments = buildings.clone();
                    for b in buildings {
                        let center = b.emprise.center();
                        println!(
                            "Bâtiment {:?} ({}) : centre {} {}, altitude {}, état {:?}",
                            b.nom, b.sorte, center.x, center.y, b.altitude, b.etat
                        );
                    }
                },
                Event::IcebreakerExpedition(expedition) => match expedition {
                    Some(e) => println!(
                        "Expédition vers {} : départ {:?}, inscrits {:?}",
                        e.destination, e.depart_dans, e.membres
                    ),
                    None => println!("Expédition : aucune"),
                },
                Event::IcebreakerJouer(next) => {
                    client.request_remove_character();
                    registre_next = Some(next);
                },
                // Icebreaker : le panneau typé (jalon N), bloc par bloc.
                Event::IcebreakerPanel(p) | Event::IcebreakerBilan(p) => {
                    let text = |c: &comp::Content| match c {
                        comp::Content::Plain(s) => s.clone(),
                        other => format!("{other:?}"),
                    };
                    let bouton = |a: &common::icebreaker_panneau::Action| {
                        format!(
                            "[{}{}{}] /{}",
                            text(&a.libelle),
                            if a.principale { " *" } else { "" },
                            if a.permise { "" } else { " (grisé)" },
                            a.commande.join(" ")
                        )
                    };
                    let ligne = |l: &common::icebreaker_panneau::Ligne| {
                        format!(
                            "  - {} | {} | icône {:?} | {} | cadenas {:?} | {}",
                            text(&l.titre),
                            text(&l.sous_titre),
                            l.icone,
                            l.pastille.as_ref().map_or(String::new(), |(mot, _)| text(mot)),
                            l.cadenas.as_ref().map(|(ferme, _)| *ferme),
                            l.action.as_ref().map_or(String::new(), bouton),
                        )
                    };
                    println!(
                        "{} « {} » : {}",
                        if bilan { "Bilan" } else { "Panneau" },
                        text(&p.titre),
                        text(&p.description)
                    );
                    for c in &p.chiffres {
                        println!("  {} ({})", text(&c.valeur), text(&c.legende));
                    }
                    if let Some(j) = &p.jauge {
                        println!("  jauge {} : {} ({:.0} %)", text(&j.legende), text(&j.texte), j.part * 100.0);
                    }
                    for (objet, n) in &p.objets {
                        println!("  {n} x {objet:?}");
                    }
                    for (i, o) in p.onglets.iter().enumerate() {
                        println!("  onglet {}{}", text(&o.libelle), if i == usize::from(p.onglet) { " (ouvert)" } else { "" });
                        o.lignes.iter().for_each(|l| println!("  {}", ligne(l)));
                    }
                    p.lignes.iter().for_each(|l| println!("{}", ligne(l)));
                    p.actions.iter().for_each(|a| println!("  {}", bouton(a)));
                    if let Some((message, sorte)) = &p.message {
                        println!("  message ({sorte:?}) : {}", text(message));
                    }
                },
                Event::DisconnectionNotification(time) => {
                    let message = match time {
                        0 => String::from("Goodbye!"),
                        _ => format!("Connection lost. Kicking in {} seconds", time),
                    };

                    println!("{}", message)
                },
                _ => {},
            }
        }

        // Clean up the server after a tick.
        client.cleanup();

        // Wait for the next tick.
        clock.tick();

        if !player_printed {
            println!("Players online: {:?}", client.players().collect::<Vec<_>>());
            player_printed = true;
        }
    }
}
