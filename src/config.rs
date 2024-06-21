use std::fs;
use std::process::Command;

#[derive(Debug, Default, Clone)]
pub struct ImapStoreConfig {
    pub name: String,
    pub host: String,
    pub user: String,
    pub pass: Option<String>,
    pub pass_command: Option<String>,
}

impl ImapStoreConfig {
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

#[derive(Debug, Default)]
pub struct ChannelConfig {
    pub name: String,
    pub near: String,
    pub far: String,
}

#[derive(Debug, Default)]
pub struct Config {
    pub channels: Vec<ChannelConfig>,
    pub imap_stores: Vec<ImapStoreConfig>,
}

impl Config {
    pub fn find_imap_store(&self, name: &str) -> Option<ImapStoreConfig> {
        self.imap_stores
            .iter()
            .find(|store| format!(":{}:", store.name) == name)
            .map(|store| store.clone())
    }
}

enum ConfigLine {
    ImapStore(String),
    Host(String),
    User(String),
    Pass(String),
    PassCommand(String),

    Channel(String),
    Near(String),
    Far(String),

    End,
}

impl TryFrom<&str> for ConfigLine {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (key, value) = split_on_first_word(value);

        match key {
            "IMAPStore" => Ok(ConfigLine::ImapStore(value)),
            "Host" => Ok(ConfigLine::Host(value)),
            "User" => Ok(ConfigLine::User(value)),
            "Pass" => Ok(ConfigLine::Pass(remove_quotes(value))),
            "PassCmd" => Ok(ConfigLine::PassCommand(remove_quotes(value))),

            "Channel" => Ok(ConfigLine::Channel(value)),
            "Near" => Ok(ConfigLine::Near(value)),
            "Far" => Ok(ConfigLine::Far(value)),

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
                config.imap_stores.push(imap_store_config)
            }
            ConfigLine::Host(host) => {
                let store = config.imap_stores.last_mut().unwrap();
                store.host = host;
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

            ConfigLine::End => {}
        }
    }

    config
}
