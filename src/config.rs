use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default)]
pub struct SettingsLoadOptions<'a> {
    pub config_path: Option<&'a str>,
    pub profile: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingsResolution {
    pub config_path: String,
    pub profile: Option<String>,
    pub profile_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedSettings {
    pub settings: Settings,
    pub resolution: SettingsResolution,
}

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
    pub const ENV_CONFIG_PATH: &'static str = "BLOCKCHAIN_SIM_CONFIG_PATH";
    pub const ENV_PROFILE: &'static str = "BLOCKCHAIN_SIM_PROFILE";
    pub const ENV_APP_INITIAL_NODE_COUNT: &'static str = "BLOCKCHAIN_SIM_APP_INITIAL_NODE_COUNT";
    pub const ENV_NETWORK_DIFFICULTY: &'static str = "BLOCKCHAIN_SIM_NETWORK_DIFFICULTY";
    pub const ENV_NETWORK_BLOCK_TIME_SECONDS: &'static str =
        "BLOCKCHAIN_SIM_NETWORK_BLOCK_TIME_SECONDS";
    pub const ENV_PERSISTENCE_STATE_PATH: &'static str = "BLOCKCHAIN_SIM_PERSISTENCE_STATE_PATH";
    pub const ENV_API_BIND_HOST: &'static str = "BLOCKCHAIN_SIM_API_BIND_HOST";
    pub const ENV_API_BIND_PORT: &'static str = "BLOCKCHAIN_SIM_API_BIND_PORT";

    pub fn load_default() -> Result<Self, String> {
        Self::load(SettingsLoadOptions::default())
    }

    pub fn load(options: SettingsLoadOptions<'_>) -> Result<Self, String> {
        let loaded = Self::load_with_resolution(options)?;
        Ok(loaded.settings)
    }

    pub fn load_with_resolution(
        options: SettingsLoadOptions<'_>,
    ) -> Result<LoadedSettings, String> {
        let config_path = Self::resolve_config_path(options.config_path);
        let profile = Self::resolve_profile(options.profile);
        let profile_path = profile
            .as_ref()
            .map(|profile| Self::profile_path_for(Path::new(&config_path), profile));

        let mut settings = Self::default();
        settings.merge_file_if_exists(Path::new(&config_path))?;

        if let Some(profile_path) = profile_path.as_deref() {
            settings.merge_file_if_exists(&profile_path)?;
        }

        settings.apply_env_overrides()?;
        settings.validate()?;
        Ok(LoadedSettings {
            settings,
            resolution: SettingsResolution {
                config_path,
                profile,
                profile_path: profile_path.map(|path| path.to_string_lossy().to_string()),
            },
        })
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        Self::load(SettingsLoadOptions {
            config_path: Some(path),
            profile: None,
        })
    }

    pub fn load_with_path(path: Option<&str>) -> Result<Self, String> {
        Self::load(SettingsLoadOptions {
            config_path: path,
            profile: None,
        })
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

    fn resolve_config_path(config_path: Option<&str>) -> String {
        config_path
            .map(|value| value.to_string())
            .or_else(|| Self::read_env_string(Self::ENV_CONFIG_PATH).ok().flatten())
            .unwrap_or_else(|| Self::DEFAULT_CONFIG_PATH.to_string())
    }

    fn resolve_profile(profile: Option<&str>) -> Option<String> {
        profile
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| Self::read_env_string(Self::ENV_PROFILE).ok().flatten())
    }

    fn profile_path_for(config_path: &Path, profile: &str) -> PathBuf {
        let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
        config_dir.join(format!("{}.toml", profile))
    }

    fn merge_file_if_exists(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path)
            .map_err(|err| format!("Config read error ({}): {}", path.display(), err))?;
        let raw: RawSettings = toml::from_str(&content)
            .map_err(|err| format!("Config parse error ({}): {}", path.display(), err))?;
        self.apply_raw(raw);
        Ok(())
    }

    fn apply_raw(&mut self, raw: RawSettings) {
        if let Some(app) = raw.app {
            if let Some(initial_node_count) = app.initial_node_count {
                self.app.initial_node_count = initial_node_count;
            }
        }

        if let Some(network) = raw.network {
            if let Some(difficulty) = network.difficulty {
                self.network.difficulty = difficulty;
            }
            if let Some(block_time_seconds) = network.block_time_seconds {
                self.network.block_time_seconds = block_time_seconds;
            }
        }

        if let Some(persistence) = raw.persistence {
            if let Some(state_path) = persistence.state_path {
                self.persistence.state_path = state_path;
            }
        }

        if let Some(api) = raw.api {
            if let Some(bind_host) = api.bind_host {
                self.api.bind_host = bind_host;
            }
            if let Some(bind_port) = api.bind_port {
                self.api.bind_port = bind_port;
            }
        }
    }

    fn apply_env_overrides(&mut self) -> Result<(), String> {
        if let Some(initial_node_count) = Self::read_env_usize(Self::ENV_APP_INITIAL_NODE_COUNT)? {
            self.app.initial_node_count = initial_node_count;
        }
        if let Some(difficulty) = Self::read_env_usize(Self::ENV_NETWORK_DIFFICULTY)? {
            self.network.difficulty = difficulty;
        }
        if let Some(block_time_seconds) = Self::read_env_u64(Self::ENV_NETWORK_BLOCK_TIME_SECONDS)?
        {
            self.network.block_time_seconds = block_time_seconds;
        }
        if let Some(state_path) = Self::read_env_string(Self::ENV_PERSISTENCE_STATE_PATH)? {
            self.persistence.state_path = state_path;
        }
        if let Some(bind_host) = Self::read_env_string(Self::ENV_API_BIND_HOST)? {
            self.api.bind_host = bind_host;
        }
        if let Some(bind_port) = Self::read_env_u16(Self::ENV_API_BIND_PORT)? {
            self.api.bind_port = bind_port;
        }
        Ok(())
    }

    fn read_env_string(key: &str) -> Result<Option<String>, String> {
        match env::var(key) {
            Ok(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(format!("Environment variable {} must not be empty", key));
                }
                Ok(Some(trimmed.to_string()))
            }
            Err(env::VarError::NotPresent) => Ok(None),
            Err(err) => Err(format!(
                "Environment variable {} could not be read: {}",
                key, err
            )),
        }
    }

    fn read_env_usize(key: &str) -> Result<Option<usize>, String> {
        match env::var(key) {
            Ok(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(format!("Environment variable {} must not be empty", key));
                }
                let parsed = trimmed.parse::<usize>().map_err(|err| {
                    format!("Environment variable {} is not a valid usize: {}", key, err)
                })?;
                Ok(Some(parsed))
            }
            Err(env::VarError::NotPresent) => Ok(None),
            Err(err) => Err(format!(
                "Environment variable {} could not be read: {}",
                key, err
            )),
        }
    }

    fn read_env_u64(key: &str) -> Result<Option<u64>, String> {
        match env::var(key) {
            Ok(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(format!("Environment variable {} must not be empty", key));
                }
                let parsed = trimmed.parse::<u64>().map_err(|err| {
                    format!("Environment variable {} is not a valid u64: {}", key, err)
                })?;
                Ok(Some(parsed))
            }
            Err(env::VarError::NotPresent) => Ok(None),
            Err(err) => Err(format!(
                "Environment variable {} could not be read: {}",
                key, err
            )),
        }
    }

    fn read_env_u16(key: &str) -> Result<Option<u16>, String> {
        match env::var(key) {
            Ok(value) => {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(format!("Environment variable {} must not be empty", key));
                }
                let parsed = trimmed.parse::<u16>().map_err(|err| {
                    format!("Environment variable {} is not a valid u16: {}", key, err)
                })?;
                Ok(Some(parsed))
            }
            Err(env::VarError::NotPresent) => Ok(None),
            Err(err) => Err(format!(
                "Environment variable {} could not be read: {}",
                key, err
            )),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawSettings {
    app: Option<RawAppSettings>,
    network: Option<RawNetworkSettings>,
    persistence: Option<RawPersistenceSettings>,
    api: Option<RawApiSettings>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawAppSettings {
    initial_node_count: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawNetworkSettings {
    difficulty: Option<usize>,
    block_time_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawPersistenceSettings {
    state_path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawApiSettings {
    bind_host: Option<String>,
    bind_port: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::{Settings, SettingsLoadOptions};
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}_{}.toml", name, std::process::id(), stamp))
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}_{}", name, std::process::id(), stamp))
    }

    fn write_text_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directories should be created");
        }
        fs::write(path, content).expect("file should be written");
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn set_env_var(key: &str, value: &str) -> Option<String> {
        let previous = env::var(key).ok();
        env::set_var(key, value);
        previous
    }

    fn clear_config_env_vars() -> Vec<(&'static str, Option<String>)> {
        let keys = [
            Settings::ENV_CONFIG_PATH,
            Settings::ENV_PROFILE,
            Settings::ENV_APP_INITIAL_NODE_COUNT,
            Settings::ENV_NETWORK_DIFFICULTY,
            Settings::ENV_NETWORK_BLOCK_TIME_SECONDS,
            Settings::ENV_PERSISTENCE_STATE_PATH,
            Settings::ENV_API_BIND_HOST,
            Settings::ENV_API_BIND_PORT,
        ];

        let mut previous_values = Vec::with_capacity(keys.len());
        for key in keys {
            previous_values.push((key, env::var(key).ok()));
            env::remove_var(key);
        }
        previous_values
    }

    fn restore_env_var(key: &str, previous: Option<String>) {
        match previous {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
    }

    fn restore_config_env_vars(previous_values: Vec<(&'static str, Option<String>)>) {
        for (key, previous) in previous_values {
            restore_env_var(key, previous);
        }
    }

    #[test]
    fn missing_config_file_should_fall_back_to_defaults() {
        let guard = env_lock().lock().expect("env lock should be acquired");
        let previous_values = clear_config_env_vars();
        let path = unique_temp_path("missing_config");
        let settings = Settings::load_from_file(path.to_string_lossy().as_ref())
            .expect("defaults should load");
        restore_config_env_vars(previous_values);
        drop(guard);

        assert_eq!(settings.app.initial_node_count, 5);
        assert_eq!(settings.network.difficulty, 2);
        assert_eq!(settings.network.block_time_seconds, 60);
        assert_eq!(settings.persistence.state_path, "./data/network_state.json");
        assert_eq!(settings.api.bind_host, "0.0.0.0");
        assert_eq!(settings.api.bind_port, 3000);
    }

    #[test]
    fn config_file_should_override_defaults() {
        let guard = env_lock().lock().expect("env lock should be acquired");
        let previous_values = clear_config_env_vars();
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

        restore_config_env_vars(previous_values);
        drop(guard);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn profile_file_should_override_default_file_values() {
        let guard = env_lock().lock().expect("env lock should be acquired");
        let previous_values = clear_config_env_vars();
        let root = unique_temp_dir("profile_override");
        let default_path = root.join("config/default.toml");
        let profile_path = root.join("config/dev.toml");

        write_text_file(
            &default_path,
            r#"
[app]
initial_node_count = 4

[network]
difficulty = 2
block_time_seconds = 90

[api]
bind_port = 3000
"#,
        );
        write_text_file(
            &profile_path,
            r#"
[network]
difficulty = 8

[api]
bind_port = 5050
"#,
        );

        let settings = Settings::load(SettingsLoadOptions {
            config_path: Some(default_path.to_string_lossy().as_ref()),
            profile: Some("dev"),
        })
        .expect("profile config should load");

        assert_eq!(settings.app.initial_node_count, 4);
        assert_eq!(settings.network.difficulty, 8);
        assert_eq!(settings.network.block_time_seconds, 90);
        assert_eq!(settings.api.bind_port, 5050);

        restore_config_env_vars(previous_values);
        drop(guard);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn env_should_override_profile_and_default_values() {
        let guard = env_lock().lock().expect("env lock should be acquired");
        let previous_values = clear_config_env_vars();
        let root = unique_temp_dir("env_override");
        let default_path = root.join("config/default.toml");
        let profile_path = root.join("config/prod.toml");

        write_text_file(
            &default_path,
            r#"
[network]
difficulty = 2
block_time_seconds = 60

[api]
bind_port = 3000
"#,
        );
        write_text_file(
            &profile_path,
            r#"
[network]
difficulty = 5
"#,
        );

        let prev_diff = set_env_var(Settings::ENV_NETWORK_DIFFICULTY, "11");
        let prev_block_time = set_env_var(Settings::ENV_NETWORK_BLOCK_TIME_SECONDS, "15");
        let prev_port = set_env_var(Settings::ENV_API_BIND_PORT, "8088");

        let settings = Settings::load(SettingsLoadOptions {
            config_path: Some(default_path.to_string_lossy().as_ref()),
            profile: Some("prod"),
        })
        .expect("env override should load");

        restore_env_var(Settings::ENV_NETWORK_DIFFICULTY, prev_diff);
        restore_env_var(Settings::ENV_NETWORK_BLOCK_TIME_SECONDS, prev_block_time);
        restore_env_var(Settings::ENV_API_BIND_PORT, prev_port);
        restore_config_env_vars(previous_values);
        drop(guard);

        assert_eq!(settings.network.difficulty, 11);
        assert_eq!(settings.network.block_time_seconds, 15);
        assert_eq!(settings.api.bind_port, 8088);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cli_options_should_take_precedence_over_environment() {
        let guard = env_lock().lock().expect("env lock should be acquired");
        let previous_values = clear_config_env_vars();
        let root = unique_temp_dir("cli_precedence");
        let env_default_path = root.join("config/default.toml");
        let env_profile_path = root.join("config/dev.toml");
        let cli_default_path = root.join("alt/default.toml");
        let cli_profile_path = root.join("alt/qa.toml");

        write_text_file(
            &env_default_path,
            r#"
[network]
difficulty = 2
"#,
        );
        write_text_file(
            &env_profile_path,
            r#"
[network]
difficulty = 5
"#,
        );
        write_text_file(
            &cli_default_path,
            r#"
[network]
difficulty = 7
"#,
        );
        write_text_file(
            &cli_profile_path,
            r#"
[network]
difficulty = 9
"#,
        );

        let prev_config_path = set_env_var(
            Settings::ENV_CONFIG_PATH,
            env_default_path.to_string_lossy().as_ref(),
        );
        let prev_profile = set_env_var(Settings::ENV_PROFILE, "dev");

        let settings = Settings::load(SettingsLoadOptions {
            config_path: Some(cli_default_path.to_string_lossy().as_ref()),
            profile: Some("qa"),
        })
        .expect("cli options should override env selectors");

        restore_env_var(Settings::ENV_CONFIG_PATH, prev_config_path);
        restore_env_var(Settings::ENV_PROFILE, prev_profile);
        restore_config_env_vars(previous_values);
        drop(guard);

        assert_eq!(settings.network.difficulty, 9);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_env_value_should_return_error() {
        let guard = env_lock().lock().expect("env lock should be acquired");
        let previous_values = clear_config_env_vars();
        let previous = set_env_var(Settings::ENV_API_BIND_PORT, "not-a-port");

        let result = Settings::load_default();

        restore_env_var(Settings::ENV_API_BIND_PORT, previous);
        restore_config_env_vars(previous_values);
        drop(guard);

        assert!(result.is_err());
    }
}
