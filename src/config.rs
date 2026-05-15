use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub app: AppSettings,
    pub network: NetworkSettings,
    pub persistence: PersistenceSettings,
    pub api: ApiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub initial_node_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    pub difficulty: usize,
    pub block_time_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistenceSettings {
    pub state_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiSettings {
    pub bind_host: String,
    pub bind_port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app: AppSettings::default(),
            network: NetworkSettings::default(),
            persistence: PersistenceSettings::default(),
            api: ApiSettings::default(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            initial_node_count: 5,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            difficulty: 2,
            block_time_seconds: 60,
        }
    }
}

impl Default for PersistenceSettings {
    fn default() -> Self {
        Self {
            state_path: "./data/network_state.json".to_string(),
        }
    }
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            bind_host: "0.0.0.0".to_string(),
            bind_port: 3000,
        }
    }
}

impl Settings {
    pub const DEFAULT_CONFIG_PATH: &'static str = "config/default.toml";

    pub fn load_default() -> Result<Self, String> {
        Self::load_from_file(Self::DEFAULT_CONFIG_PATH)
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let path_ref = Path::new(path);
        if !path_ref.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path_ref)
            .map_err(|err| format!("Config read error ({}): {}", path, err))?;
        let settings: Self = toml::from_str(&content)
            .map_err(|err| format!("Config parse error ({}): {}", path, err))?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn load_with_path(path: Option<&str>) -> Result<Self, String> {
        let target = path.unwrap_or(Self::DEFAULT_CONFIG_PATH);
        Self::load_from_file(target)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.app.initial_node_count == 0 {
            return Err("app.initial_node_count must be greater than zero".to_string());
        }
        if self.network.difficulty == 0 {
            return Err("network.difficulty must be greater than zero".to_string());
        }
        if self.network.block_time_seconds == 0 {
            return Err("network.block_time_seconds must be greater than zero".to_string());
        }
        if self.persistence.state_path.trim().is_empty() {
            return Err("persistence.state_path must not be empty".to_string());
        }
        if self.api.bind_host.trim().is_empty() {
            return Err("api.bind_host must not be empty".to_string());
        }
        if self.api.bind_port == 0 {
            return Err("api.bind_port must be greater than zero".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}_{}.toml", name, std::process::id(), stamp))
    }

    #[test]
    fn missing_config_file_should_fall_back_to_defaults() {
        let path = unique_temp_path("missing_config");
        let settings = Settings::load_from_file(path.to_string_lossy().as_ref())
            .expect("defaults should load");

        assert_eq!(settings.app.initial_node_count, 5);
        assert_eq!(settings.network.difficulty, 2);
        assert_eq!(settings.network.block_time_seconds, 60);
        assert_eq!(settings.persistence.state_path, "./data/network_state.json");
        assert_eq!(settings.api.bind_host, "0.0.0.0");
        assert_eq!(settings.api.bind_port, 3000);
    }

    #[test]
    fn config_file_should_override_defaults() {
        let path = unique_temp_path("override_config");
        let content = r#"
[app]
initial_node_count = 7

[network]
difficulty = 4
block_time_seconds = 30

[persistence]
state_path = "./tmp/state.json"

[api]
bind_host = "127.0.0.1"
bind_port = 4040
"#;
        fs::write(&path, content).expect("temp config should be writable");

        let settings =
            Settings::load_from_file(path.to_string_lossy().as_ref()).expect("config should parse");

        assert_eq!(settings.app.initial_node_count, 7);
        assert_eq!(settings.network.difficulty, 4);
        assert_eq!(settings.network.block_time_seconds, 30);
        assert_eq!(settings.persistence.state_path, "./tmp/state.json");
        assert_eq!(settings.api.bind_host, "127.0.0.1");
        assert_eq!(settings.api.bind_port, 4040);

        let _ = fs::remove_file(path);
    }
}
