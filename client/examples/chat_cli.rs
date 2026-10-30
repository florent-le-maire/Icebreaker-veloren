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

    loop {
        for msg in rx.try_iter() {
            // Icebreaker : « /commande args » part en commande, pour tester les
            // commandes du serveur et des plugins.
            if msg.is_empty() {
                continue;
            } else if msg == "!jouer" {
                client.load_character_list();
                joining = true;
            } else if let Some(command) = msg.strip_prefix('/') {
                let mut words = command.split_whitespace().map(str::to_string);
                if let Some(name) = words.next() {
                    client.send_command(name, words.collect());
                }
            } else {
                client.send_chat(msg)
            }
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
