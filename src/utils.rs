use std::{fmt::Write, path::PathBuf};
use dirs;

/// Returns a human readable String of a Duration
///
/// Example: `3h 12m 53s`
pub fn format_duration(d: &std::time::Duration) -> String {
    let mut s = String::new();
    let mut append_unit = |value, unit| {
        if value > 0 {
            let _ = write!(s, "{value}{unit}");
        }
    };

    let seconds = d.as_secs() % 60;
    let minutes = (d.as_secs() / 60) % 60;
    let hours = (d.as_secs() / 60) / 60;

    append_unit(hours, "h ");
    append_unit(minutes, "m ");
    append_unit(seconds, "s ");

    s.trim_end().to_string()
}

/// Returns a human readable String of milliseconds in the HH:MM:SS format.
pub fn ms_to_hms(duration: u32) -> String {
    let mut formated_time = String::new();

    let total_seconds = duration / 1000;
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;

    if hours > 0 {
        formated_time.push_str(&format!("{hours}:{minutes:02}:"));
    } else {
        formated_time.push_str(&format!("{minutes}:"));
    }
    formated_time.push_str(&format!("{seconds:02}"));

    formated_time
}

/// Return the path to the current user's runtime directory. This function does
/// not guarantee correct ownership or permissions of the directory.
#[cfg(unix)]
pub fn user_runtime_directory() -> PathBuf {
    let user_runtime_directory = dirs::runtime_dir();
    let mut system_runtime_directory = PathBuf::from("/tmp/");

    if let Some(mut runtime_dir) = user_runtime_directory {
        runtime_dir.push("ncspot");
        runtime_dir
    } else if system_runtime_directory.exists() {
        system_runtime_directory.push(format!("ncspot-{}", unsafe { libc::getuid() }));
        system_runtime_directory
    } else {
        user_runtime_directory.unwrap()
    }
}
