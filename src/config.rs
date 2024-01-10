use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::fs;

use dirs;
use log::{debug, error};

use crate::command::{SortDirection, SortKey};
use crate::model::playable::Playable;
use crate::queue;
use crate::serialization::{Serializer, CBOR, TOML};

pub const CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";
pub const CACHE_VERSION: u16 = 1;
pub const DEFAULT_COMMAND_KEY: char = ':';

/// The playback state when ncspot is started.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Default,
}

/// The focussed library tab when ncspot is started.
#[derive(Clone, Serialize, Deserialize, Debug, Hash, strum_macros::EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum LibraryTab {
    Tracks,
    Albums,
    Artists,
    Playlists,
    Podcasts,
    Browse,
}

/// The format used to represent tracks in a list.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TrackFormat {
    pub left: Option<String>,
    pub center: Option<String>,
    pub right: Option<String>,
}

impl TrackFormat {
    pub fn default() -> Self {
        Self {
            left: Some(String::from("%artists - %title")),
            center: Some(String::from("%album")),
            right: Some(String::from("%saved %duration")),
        }
    }
}

/// The configuration of ncspot.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ConfigValues {
    pub command_key: Option<char>,
    pub initial_screen: Option<String>,
    pub flip_status_indicators: Option<bool>,
    pub audio_cache: Option<bool>,
    pub audio_cache_size: Option<u32>,
    pub backend: Option<String>,
    pub backend_device: Option<String>,
    pub volnorm: Option<bool>,
    pub volnorm_pregain: Option<f64>,
    pub bitrate: Option<u32>,
    pub gapless: Option<bool>,
    pub shuffle: Option<bool>,
    pub repeat: Option<queue::RepeatSetting>,
    pub playback_state: Option<PlaybackState>,
    pub track_format: Option<TrackFormat>,
    pub statusbar_format: Option<String>,
    pub library_tabs: Option<Vec<LibraryTab>>,
    pub hide_display_names: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortingOrder {
    pub key: SortKey,
    pub direction: SortDirection,
}

/// The runtime state of the music queue.
#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct QueueState {
    pub current_track: Option<usize>,
    pub random_order: Option<Vec<usize>>,
    pub track_progress: std::time::Duration,
    pub queue: Vec<Playable>,
}

/// Runtime state that should be persisted accross sessions.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserState {
    pub volume: u16,
    pub shuffle: bool,
    pub repeat: queue::RepeatSetting,
    pub queuestate: QueueState,
    pub playlist_orders: HashMap<String, SortingOrder>,
    pub cache_version: u16,
    pub playback_state: PlaybackState,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            volume: u16::MAX,
            shuffle: false,
            repeat: queue::RepeatSetting::None,
            queuestate: QueueState::default(),
            playlist_orders: HashMap::new(),
            cache_version: 0,
            playback_state: PlaybackState::Default,
        }
    }
}

/// The complete configuration (state + user configuration) of ncspot.
pub struct Config {
    /// Configuration set by the user, read only.
    values: RwLock<ConfigValues>,
    /// Runtime state which can't be edited by the user, read/write.
    state: RwLock<UserState>,
}

impl Config {
    /// Generate the configuration from the user configuration file and the runtime state file.
    pub fn new() -> Self {
        let values = {
            let path = config_path("config.toml");
            TOML.load_or_generate_default(path, || Ok(ConfigValues::default()), false)
                .expect("There is an error in your configuration file")
        };

        let mut userstate = {
            let path = config_path("userstate.cbor");
            CBOR.load_or_generate_default(path, || Ok(UserState::default()), true)
                .expect("could not load user state")
        };

        if let Some(shuffle) = values.shuffle {
            userstate.shuffle = shuffle;
        }

        if let Some(repeat) = values.repeat {
            userstate.repeat = repeat;
        }

        if let Some(playback_state) = values.playback_state.clone() {
            userstate.playback_state = playback_state;
        }

        Self {
            values: RwLock::new(values),
            state: RwLock::new(userstate),
        }
    }

    pub fn values(&self) -> RwLockReadGuard<ConfigValues> {
        self.values.read().expect("can't readlock config values")
    }

    pub fn state(&self) -> RwLockReadGuard<UserState> {
        self.state.read().expect("can't readlock user state")
    }

    pub fn with_state_mut<F>(&self, cb: F)
    where
        F: Fn(RwLockWriteGuard<UserState>),
    {
        let state_guard = self.state.write().expect("can't writelock user state");
        cb(state_guard);
    }

    pub fn save_state(&self) {
        // update cache version number
        self.with_state_mut(|mut state| state.cache_version = CACHE_VERSION);

        let path = config_path("userstate.cbor");
        debug!("saving user state to {}", path.display());
        if let Err(e) = CBOR.write(path, self.state().clone()) {
            error!("Could not save user state: {}", e);
        }
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
