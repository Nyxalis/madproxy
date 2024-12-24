use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;
use serde_json;
use std::sync::atomic::{AtomicUsize, Ordering};
#[macro_use]
use log::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerEntry {
    pub id: String,
    pub hostnames: Vec<String>,
    #[serde(rename = "backend_server")]
    pub backend_server: String,
    #[serde(skip)]
    pub player_count: AtomicUsize,
}

impl Clone for ServerEntry {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            hostnames: self.hostnames.clone(),
            backend_server: self.backend_server.clone(),
            player_count: AtomicUsize::new(self.player_count.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServersFile {
    servers: Vec<ServerEntry>
}

#[derive(Debug)]
pub struct Servers {
    entries: Vec<ServerEntry>,
}

impl Clone for Servers {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone()
        }
    }
}

impl Servers {
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string("servers.json")?;
        let mut servers_file: ServersFile = serde_json::from_str(&content)?;
        
        // Initialize player counts to 0
        for server in &mut servers_file.servers {
            server.player_count = AtomicUsize::new(0);
        }
        
        Ok(Self { 
            entries: servers_file.servers 
        })
    }

    pub fn increment_player_count(&self, hostname: &str) -> Option<usize> {
        if let Some(server) = self.entries
            .iter()
            .find(|s| s.hostnames.contains(&hostname.to_string())) {
            let new_count = server.player_count.fetch_add(1, Ordering::SeqCst) + 1;
            info!("Player joined {} - Current players: {}", hostname, new_count);
            Some(new_count)
        } else {
            None
        }
    }

    pub fn decrement_player_count(&self, hostname: &str) -> Option<usize> {
        if let Some(server) = self.entries
            .iter()
            .find(|s| s.hostnames.contains(&hostname.to_string())) {
            let current = server.player_count.load(Ordering::SeqCst);
            if current > 0 {
                let new_count = server.player_count.fetch_sub(1, Ordering::SeqCst) - 1;
                info!("Player left {} - Current players: {}", hostname, new_count);
                Some(new_count)
            } else {
                warn!("Attempted to decrement player count below 0 for {}", hostname);
                Some(0)
            }
        } else {
            warn!("Attempted to decrement player count for unknown server: {}", hostname);
            None
        }
    }

    pub fn get_player_count(&self, hostname: &str) -> Option<usize> {
        self.entries
            .iter()
            .find(|s| s.hostnames.contains(&hostname.to_string()))
            .map(|s| s.player_count.load(Ordering::SeqCst))
    }

    pub fn get_by_hostname(&self, hostname: &str) -> Option<ServerEntry> {
        self.entries
            .iter()
            .find(|s| s.hostnames.contains(&hostname.to_string()))
            .cloned()
    }

    pub fn save(&self) -> Result<()> {
        let servers_file = ServersFile {
            servers: self.entries.clone()
        };
        let json = serde_json::to_string_pretty(&servers_file)?;
        fs::write("servers.json", json)?;
        Ok(())
    }

    pub fn add_server(&self, entry: ServerEntry) -> Result<()> {
        let mut entries = self.entries.clone();
        entries.push(entry);
        self.save()?;
        Ok(())
    }

    pub fn remove_server(&self, hostname: &str) -> Result<bool> {
        let mut entries = self.entries.clone();
        let len = entries.len();
        entries.retain(|s| s.hostnames.contains(&hostname.to_string()));
        let removed = entries.len() != len;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn update_server(&self, hostname: &str, new_entry: ServerEntry) -> Result<bool> {
        let mut entries = self.entries.clone();
        if let Some(entry) = entries.iter_mut().find(|s| s.hostnames.contains(&hostname.to_string())) {
            *entry = new_entry;
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_servers(&self) -> Vec<ServerEntry> {
        self.entries.clone()
    }
} 