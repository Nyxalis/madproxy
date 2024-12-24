use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::path::Path;
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MOTD {
    text: String,
    protocol_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub(crate) ip: String,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownHost {
    kick_message: String,
    motd: MOTD
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineServer {
    kick_message: String,
    starting_message: String,
    motd: MOTD
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub listen_addr: String,
    pub unknown_host: UnknownHost,
    pub offline_server: OfflineServer,
    pub auto_start: bool,
    pub panel_link: String,
    pub api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        let unknown_host = UnknownHost {
            kick_message: "§bRust Minecraft Proxy\n\n§cInvalid Address".to_string(),
            motd: MOTD { text: "§cUnknown host!\n§7Please use a valid address to connect.".to_string(), protocol_name: "§crust-minecraft-proxy".to_string(),  }
        };

        let offline_server = OfflineServer {
            kick_message: "§bRust Minecraft Proxy\n\n§cServer is offline".to_string(),
            starting_message: "§bRust Minecraft Proxy\n\n§eServer is starting...".to_string(),
            motd: MOTD { text: "§cServer is offline!\n§7Please try again later.".to_string(), protocol_name: "§cServer Offline".to_string() }
        };

        Self {
            listen_addr: "0.0.0.0:25565".to_string(),
            unknown_host,
            offline_server,
            auto_start: true,
            panel_link: "https://your.panel.link".to_string(),
            api_key: "your_api_key".to_string(),
        }
    }
}

impl Config {
    pub fn load_or_init(path: &Path) -> Config {
        if path.exists() {
            serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
        } else {
            info!("Configuration file does not exist. Use defaults.");
            let default = Config::default();
            trace!("Default configuration: {:?}", default);
            let string = serde_yaml::to_string(&default).unwrap();
            fs::write(path, &string).unwrap();
            default
        }
    }

    pub fn get_listen_addr(&self) -> String {
        self.listen_addr.clone()
    }

    pub fn get_unknown_host_kick_msg(&self) -> String {
        let mut message: String = "{\"text\":\"".to_owned();
        message.push_str(&self.unknown_host.kick_message);
        message.push_str("\"}");
        message
    }

    pub fn get_unknown_host_motd(&self) -> String {
        json!({
            "version": {
                "name": &self.unknown_host.motd.protocol_name,
                "protocol": -1
            },
            "players": {
                "max": 0,
                "online": 0,
                "sample": []
            },
            "description": {
                "text": &self.unknown_host.motd.text
            }
        }).to_string()
    }

    pub fn get_offline_server_kick_msg(&self) -> String {
        let mut message: String = "{\"text\":\"".to_owned();
        message.push_str(&self.offline_server.kick_message);
        message.push_str("\"}");
        message
    }

    pub fn get_offline_server_starting_msg(&self) -> String {
        let mut message: String = "{\"text\":\"".to_owned();
        message.push_str(&self.offline_server.starting_message);
        message.push_str("\"}");
        message
    }

    pub async fn get_offline_server_motd_not_starting(&self, _server_id: &str) -> String {
        json!({
            "version": {
                "name": &self.offline_server.motd.protocol_name,
                "protocol": -1
            },
            "players": {
                "max": 0,
                "online": 0,
                "sample": []
            },
            "description": {
                "text": &self.offline_server.motd.text
            }
        }).to_string()
    }

    pub fn auto_start(&self) -> bool {
        self.auto_start
    }

    pub fn panel_link(&self) -> String {
        self.panel_link.clone()
    }

    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }
}
