use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub id: String,
    pub hostname: String,
    #[serde(rename = "backendServer")]
    pub backend_server: String,
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
        let entries = serde_json::from_str(&content)?;
        Ok(Self { entries })
    }

    pub fn get_by_hostname(&self, hostname: &str) -> Option<ServerEntry> {
        self.entries
            .iter()
            .find(|s| s.hostname == hostname)
            .cloned()
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)?;
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
        entries.retain(|s| s.hostname != hostname);
        let removed = entries.len() != len;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn update_server(&self, hostname: &str, new_entry: ServerEntry) -> Result<bool> {
        let mut entries = self.entries.clone();
        if let Some(entry) = entries.iter_mut().find(|s| s.hostname == hostname) {
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