use std::fs;
use std::process::Command;

#[derive(Debug, Default, Clone)]
pub struct ImapStoreConfig {
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub pass: Option<String>,
    pub pass_command: Option<String>,
    pub cert_file: Option<String>,
}

impl ImapStoreConfig {
    pub fn port(&self) -> u16 {
        if let Some(port) = self.port {
            return port;
        }

        993
    }
    pub fn password(&self) -> String {
        if let Some(pass) = &self.pass {
            return pass.to_string();
        }

        if let Some(command) = &self.pass_command {
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .expect("Failed to execute password command");

            let pass = String::from_utf8(output.stdout).expect("Failed to parse password");
            return pass.trim().to_string();
        }

        "".to_string()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChannelConfig {
    pub name: String,
    pub near: String,
    pub far: String,
}

#[derive(Debug, Default, Clone)]
pub struct GroupConfig {
    pub name: String,
    pub channels: Vec<(String, String)>,
}

#[derive(Debug, Default)]
pub struct Config {
    pub channels: Vec<ChannelConfig>,
    pub imap_stores: Vec<ImapStoreConfig>,
    pub groups: Vec<GroupConfig>,
}

impl Config {
    pub fn find_imap_store(&self, name: &str) -> Option<ImapStoreConfig> {
        self.imap_stores
            .iter()
            .find(|store| format!(":{}:", store.name) == name)
            .map(|store| store.clone())
    }

    pub fn find_channel(&self, name: &str) -> Option<ChannelConfig> {
        self.channels
            .iter()
            .find(|channel| channel.name == name)
            .map(|channel| channel.clone())
    }

    pub fn find_group(&self, name: &str) -> Option<GroupConfig> {
        self.groups
            .iter()
            .find(|group| group.name == name)
            .map(|group| group.clone())
    }
}

enum ConfigLine {
    ImapStore(String),
    Host(String),
    Port(u16),
    User(String),
    Pass(String),
    PassCommand(String),
    CertFile(String),

    Channel(String),
    Near(String),
    Far(String),

    Group(String),
    Channels(String),

    End,
}

impl TryFrom<&str> for ConfigLine {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (key, value) = split_on_first_word(value);

        match key {
            "IMAPStore" => Ok(ConfigLine::ImapStore(value)),
            "Host" => Ok(ConfigLine::Host(value)),
            "Port" => Ok(ConfigLine::Port(value.parse().unwrap())),
            "User" => Ok(ConfigLine::User(value)),
            "Pass" => Ok(ConfigLine::Pass(remove_quotes(value))),
            "PassCmd" => Ok(ConfigLine::PassCommand(remove_quotes(value))),
            "CertificateFile" => Ok(ConfigLine::CertFile(value)),

            "Channel" => Ok(ConfigLine::Channel(value)),
            "Near" => Ok(ConfigLine::Near(value)),
            "Far" => Ok(ConfigLine::Far(value)),

            "Group" => Ok(ConfigLine::Group(value)),
            "Channels" => Ok(ConfigLine::Channels(value)),

            _ => {
                if value == "" {
                    return Ok(ConfigLine::End);
                }

                Err(())
            }
        }
    }
}

fn remove_quotes(s: String) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        s.strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap()
            .to_string()
    } else {
        s.to_string()
    }
}

fn split_on_first_word(s: &str) -> (&str, String) {
    if let Some(pos) = s.find(char::is_whitespace) {
        let (first_word, rest) = s.split_at(pos);
        let rest = rest.trim_start();
        (first_word, rest.to_string())
    } else {
        (s, "".to_string())
    }
}

pub fn from_file(config_path: &str) -> Config {
    let mut config = Config::default();

    for line in fs::read_to_string(config_path).unwrap().lines() {
        let config_line = match ConfigLine::try_from(line) {
            Ok(config_line) => config_line,
            Err(_) => continue,
        };

        match config_line {
            ConfigLine::ImapStore(name) => {
                let mut imap_store_config = ImapStoreConfig::default();
                imap_store_config.name = name;
                config.imap_stores.push(imap_store_config);
            }
            ConfigLine::Host(host) => {
                let store = config.imap_stores.last_mut().unwrap();
                store.host = host;
            }
            ConfigLine::Port(port) => {
                let store = config.imap_stores.last_mut().unwrap();
                store.port = Some(port);
            }
            ConfigLine::User(user) => {
                let store = config.imap_stores.last_mut().unwrap();
                store.user = user;
            }
            ConfigLine::Pass(pass) => {
                let store = config.imap_stores.last_mut().unwrap();
                store.pass = Some(pass);
            }
            ConfigLine::PassCommand(command) => {
                let store = config.imap_stores.last_mut().unwrap();
                store.pass_command = Some(command);
            }
            ConfigLine::CertFile(cert_file) => {
                let store = config.imap_stores.last_mut().unwrap();
                store.cert_file = Some(cert_file);
            }

            ConfigLine::Channel(name) => {
                let mut channel_config = ChannelConfig::default();
                channel_config.name = name;
                config.channels.push(channel_config)
            }
            ConfigLine::Far(far) => {
                let channel = config.channels.last_mut().unwrap();
                channel.far = far;
            }
            ConfigLine::Near(near) => {
                let channel = config.channels.last_mut().unwrap();
                channel.near = near;
            }

            ConfigLine::Group(name) => {
                let mut group = GroupConfig::default();
                group.name = name;

                config.groups.push(group);
            }
            ConfigLine::Channels(channels) => {
                let group = config.groups.last_mut().unwrap();
                group.channels = channels
                    .split(',')
                    .map(|channel| {
                        let mut iter = channel.trim().split(':');

                        (
                            iter.next().unwrap_or("").to_string(),
                            iter.next().unwrap_or("").to_string(),
                        )
                    })
                    .collect();
            }

            ConfigLine::End => {}
        }
    }

    config
}
