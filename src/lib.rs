use clap::builder::PathBufValueParser;
use librespot_playback::audio_backend;

/// Return the [Command](clap::Command) that models the program's command line arguments. The
/// command can be used to parse the actual arguments passed to the program, or to automatically
/// generate a man page using clap's mangen package.
pub fn program_arguments() -> clap::Command {
    let backends = {
        let backends: Vec<&str> = audio_backend::BACKENDS.iter().map(|b| b.0).collect();
        format!("Audio backends: {}", backends.join(", "))
    };

    clap::Command::new("ncspot")
        .version(env!("VERSION"))
        .author("Henrik Friedrichsen <henrik@affekt.org> and contributors")
        .about("cross-platform ncurses Spotify client")
        .after_help(backends)
        .arg(
            clap::Arg::new("debug")
                .short('d')
                .long("debug")
                .value_name("FILE")
                .value_parser(PathBufValueParser::new())
                .help("Enable debug logging to the specified file"),
        )
        .subcommands([clap::Command::new("info").about("Print platform information like paths")])
}
