use crate::config::{user_cache_directory, user_configuration_directory};

#[cfg(unix)]
use crate::utils::user_runtime_directory;

/// Print platform info like which platform directories will be used.
pub fn info() {
    let user_configuration_directory = user_configuration_directory().to_string_lossy().to_string();
    let user_cache_directory = user_cache_directory().to_string_lossy().to_string();
    #[cfg(unix)]
    let user_runtime_directory = user_runtime_directory().to_string_lossy().to_string();

    println!("USER_CONFIGURATION_PATH {}", user_configuration_directory);
    println!("USER_CACHE_PATH {}", user_cache_directory);
    #[cfg(unix)]
    println!("USER_RUNTIME_PATH {}", user_runtime_directory);
}
