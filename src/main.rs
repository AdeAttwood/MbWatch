mod config;

use std::process::Command;
use std::thread;

use config::ImapStoreConfig;
use imap::{ImapConnection, Session};

fn connect(config: &ImapStoreConfig, mailbox: &String) -> Option<Session<Box<dyn ImapConnection>>> {
    let mut client_builder = imap::ClientBuilder::new(config.host.clone(), config.port())
        .mode(imap::ConnectionMode::AutoTls)
        .tls_kind(imap::TlsKind::Rust);

    // For now disable skipping tls verification if we have a cert file. I will need to refactor
    // this to setup the TPC connection manually to support this. As we have passed in a
    // certificate we are assuming its all good. If we really want this to be secure for now we
    // need to setup the certificates in the system root store.
    if config.cert_file.is_some() {
        log::warn!("Skipping tls verification for {}", &config.host);
        client_builder = client_builder.danger_skip_tls_verify(true);
    }

    let client = client_builder
        .connect()
        .expect("Could not connect to the server");

    let mut session = client
        .login(&config.user, config.password())
        .expect("Unable to login please ensure yor credentials are correct");

    let capabilities = session.capabilities().expect("Unable to get capabilities");
    if !capabilities.has_str("IDLE") {
        log::info!(
            "Skipping connection, {} dose not support idle connections",
            &config.host
        );

        return None;
    }

    session.select(mailbox).expect("Unable to select folder");

    Some(session)
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .expect("Unable to create the logger");

    let home = match std::env::var("HOME") {
        Ok(home) => home,
        Err(_) => panic!("No HOME env var. This is required to find your config file"),
    };

    let config = config::from_file(&format!("{home}/.mbsyncrc"));
    let mbwatch_group = match config.find_group("mbwatch") {
        Some(group) => group,
        None => panic!("Unable to find mbwatch group in your mbsync config"),
    };

    let mut watchers = Vec::new();

    for (channel_name, mailbox) in &mbwatch_group.channels {
        let channel = match config.find_channel(channel_name) {
            Some(channel) => channel,
            None => panic!("Unable to find channel {}", channel_name),
        };

        if mailbox.is_empty() {
            panic!("No mailbox defined for channel {}", channel_name);
        }

        let imap_store = match config.find_imap_store(&channel.far) {
            Some(store) => store,
            None => panic!("Unable to find store {}", &channel.far),
        };

        let channel_name = channel.name.clone();
        let mailbox = mailbox.clone();
        watchers.push(thread::spawn(move || {
            let mut session = match connect(&imap_store, &mailbox) {
                Some(session) => session,
                None => {
                    log::error!("Unable to connect to channel {}", channel_name);
                    return;
                }
            };

            log::info!(
                "Watching for messages on channel {} in mailbox {}",
                channel_name,
                mailbox
            );

            loop {
                let result = session
                    .idle()
                    .wait_while(imap::extensions::idle::stop_on_any);

                if result.is_err() {
                    log::error!("Error while idling: {:?}", result);

                    thread::sleep(std::time::Duration::from_secs(10));
                    session = connect(&imap_store, &mailbox).unwrap();

                    continue;
                }

                Command::new("mbsync")
                    .args(["--all", &format!("{}:{}", channel_name, mailbox)])
                    .output()
                    .expect("Unable to sync mail");

                log::info!("Synced changes for {} in mailbox {}", channel_name, mailbox);
            }
        }));
    }

    for watcher in watchers {
        watcher.join().expect("Unable to join thread");
    }
}
