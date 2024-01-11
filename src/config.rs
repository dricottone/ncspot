use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard};
use std::fs;

use dirs;

use crate::serialization::{Serializer, TOML};

pub const CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";

/// The configuration of ncspot.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ConfigValues {
    pub flip_status_indicators: Option<bool>,
    pub statusbar_format: Option<String>,
}

/// The complete configuration (state + user configuration) of ncspot.
pub struct Config {
    /// Configuration set by the user, read only.
    values: RwLock<ConfigValues>,
}

impl Config {
    /// Generate the configuration from the user configuration file and the runtime state file.
    pub fn new() -> Self {
        let values = {
            let path = config_path("config.toml");
            TOML.load_or_generate_default(path, || Ok(ConfigValues::default()), false)
                .expect("There is an error in your configuration file")
        };

        Self {
            values: RwLock::new(values),
        }
    }

    pub fn values(&self) -> RwLockReadGuard<ConfigValues> {
        self.values.read().expect("can't readlock config values")
    }
}

/// Return the path to the current user's configuration directory. This
/// function does not guarantee correct permissions or ownership of the
/// directory.
pub fn user_configuration_directory() -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    path.push("ncspot");
    path
}

/// Return the path to the current user's cache directory. This function does
/// not guarantee correct permissions or ownership of the directory.
pub fn user_cache_directory() -> PathBuf {
    let mut path = dirs::cache_dir().unwrap();
    path.push("ncspot");
    path
}

/// Force create the configuration directory at the default project location,
/// removing anything that isn't a directory but has the same name. Return the
/// path to the configuration file inside the directory. This doesn't create
/// the file, only the containing directory.
pub fn config_path(file: &str) -> PathBuf {
    let mut path = user_configuration_directory();
    if path.exists() && !path.is_dir() {
        fs::remove_file(&path).expect("unable to remove old config file");
    }
    if !path.exists() {
        fs::create_dir_all(&path).expect("can't create config folder");
    }
    path.push(file);
    path
}

/// Create the cache directory at the default project location, preserving it
/// if it already exists, and return the path to the cache file inside the
/// directory. This doesn't create the file, only the containing directory.
pub fn cache_path(file: &str) -> PathBuf {
    let mut path = user_cache_directory();
    if !path.exists() {
        fs::create_dir_all(&path).expect("can't create cache folder");
    }
    path.push(file);
    path
}
