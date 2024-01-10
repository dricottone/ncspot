use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

use cursive::theme::{BaseColor, BorderStyle, Palette, PaletteColor, Theme};
use cursive::traits::Nameable;
use cursive::{CbSink, Cursive, CursiveRunner};
use log::{info, trace};

#[cfg(unix)]
use futures::stream::StreamExt;
#[cfg(unix)]
use signal_hook::{consts::SIGHUP, consts::SIGTERM};
#[cfg(unix)]
use signal_hook_tokio::Signals;

use crate::command::Command;
use crate::commands::CommandManager;
use crate::config::Config;
use crate::events::{Event, EventManager};
use crate::library::Library;
use crate::queue::Queue;
use crate::spotify::{PlayerEvent, Spotify};
use crate::ui::create_cursive;
use crate::{authentication, ui};
use crate::{queue, spotify};

/// Set up the global logger to log to `filename`.
pub fn setup_logging(filename: &Path) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Trace)
        // Set runtime log level for modules
        .level_for("librespot", log::LevelFilter::Debug)
        .level_for("cursive_buffered_backend", log::LevelFilter::Debug)
        // Output to stdout, files, and other Dispatch configurations
        .chain(fern::log_file(filename)?)
        // Apply globally
        .apply()?;
    Ok(())
}

#[cfg(unix)]
async fn handle_signals(cursive_callback_sink: CbSink) {
    let mut signals = Signals::new([SIGTERM, SIGHUP]).expect("could not register signal handler");

    while let Some(signal) = signals.next().await {
        info!("Caught {}, cleaning up and closing", signal);
        match signal {
            SIGTERM | SIGHUP => {
                cursive_callback_sink
                    .send(Box::new(|cursive| {
                        if let Some(data) = cursive.user_data::<UserData>().cloned() {
                            data.cmd.handle(cursive, Command::Quit);
                        }
                    }))
                    .expect("can't send callback to cursive");
            }
            _ => unreachable!(),
        }
    }
}

pub type UserData = Rc<UserDataInner>;
pub struct UserDataInner {
    pub cmd: CommandManager,
}

/// The global Tokio runtime for running asynchronous tasks.
pub static ASYNC_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// The representation of an ncspot application.
pub struct Application {
    /// The music queue which controls playback order.
    queue: Arc<Queue>,
    /// Internally shared
    spotify: Spotify,
    /// Internally shared
    event_manager: EventManager,
    /// The object to render to the terminal.
    cursive: CursiveRunner<Cursive>,
}

pub fn default_theme() -> Theme {
    let mut palette = Palette::default();
    let borders = BorderStyle::Simple;

    palette[PaletteColor::Background] = BaseColor::Black.dark();
    palette[PaletteColor::View] = BaseColor::Black.dark();
    palette[PaletteColor::Primary] = BaseColor::White.light();
    palette[PaletteColor::Secondary] = BaseColor::Black.light();
    palette[PaletteColor::TitlePrimary] = BaseColor::Green.dark();
    palette[PaletteColor::HighlightText] = BaseColor::White.light();
    palette[PaletteColor::Highlight] = BaseColor::Black.light();
    palette[PaletteColor::HighlightInactive] = BaseColor::Black.dark();
    palette.set_color("playing", BaseColor::Green.dark());
    palette.set_color("playing_selected", BaseColor::Green.dark());
    palette.set_color("playing_bg", BaseColor::Black.light());
    palette.set_color("error", BaseColor::White.light());
    palette.set_color("error_bg", BaseColor::Red.dark());
    palette.set_color("statusbar_progress", BaseColor::Green.dark());
    palette.set_color("statusbar_progress_bg", BaseColor::Black.light());
    palette.set_color("statusbar", BaseColor::Black.dark());
    palette.set_color("statusbar_bg", BaseColor::Green.dark());
    palette.set_color("cmdline", BaseColor::White.light());
    palette.set_color("cmdline_bg", BaseColor::Black.dark());
    palette.set_color("search_match", BaseColor::Yellow.dark());

    Theme {
        shadow: false,
        palette,
        borders,
    }
}

impl Application {
    /// Create a new ncspot application.
    pub fn new() -> Result<Self, String> {
        // Things here may cause the process to abort; we must do them before creating curses
        // windows otherwise the error message will not be seen by a user

        ASYNC_RUNTIME
            .set(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            )
            .unwrap();

        let configuration = Arc::new(Config::new());
        let credentials = authentication::get_credentials()?;

        // DON'T USE STDOUT AFTER THIS CALL!
        let mut cursive = create_cursive().map_err(|error| error.to_string())?;

        let theme = default_theme();
        cursive.set_theme(theme.clone());

        #[cfg(all(unix, feature = "pancurses_backend"))]
        cursive.add_global_callback(cursive::event::Event::CtrlChar('z'), |_s| unsafe {
            libc::raise(libc::SIGTSTP);
        });

        let event_manager = EventManager::new(cursive.cb_sink().clone());

        let spotify =
            spotify::Spotify::new(event_manager.clone(), credentials, configuration.clone());

        let library = Arc::new(Library::new(
            event_manager.clone(),
            spotify.clone(),
            configuration.clone(),
        ));

        let queue = Arc::new(queue::Queue::new(
            spotify.clone(),
            configuration.clone()
        ));

        let mut cmd_manager = CommandManager::new(
            spotify.clone(),
            queue.clone(),
            library.clone(),
            configuration.clone(),
            event_manager.clone(),
        );

        cmd_manager.register_all();
        cmd_manager.register_keybindings(&mut cursive);

        cursive.set_user_data(Rc::new(UserDataInner { cmd: cmd_manager }));

        let search =
            ui::search::SearchView::new(event_manager.clone(), queue.clone(), library.clone());

        let libraryview = ui::library::LibraryView::new(queue.clone(), library.clone());

        let queueview = ui::queue::QueueView::new(queue.clone(), library.clone());

        let status = ui::statusbar::StatusBar::new(queue.clone(), Arc::clone(&library));

        let mut layout =
            ui::layout::Layout::new(status, &event_manager, theme)
                .screen("search", search.with_name("search"))
                .screen("library", libraryview.with_name("library"))
                .screen("queue", queueview);
        layout.set_screen("library");

        cursive.add_fullscreen_layer(layout.with_name("main"));

        #[cfg(unix)]
        let cursive_callback_sink = cursive.cb_sink().clone();

        #[cfg(unix)]
        ASYNC_RUNTIME.get().unwrap().spawn(async {
            handle_signals(cursive_callback_sink).await;
        });

        Ok(Self {
            queue,
            spotify,
            event_manager,
            cursive,
        })
    }

    /// Start the application and run the event loop.
    pub fn run(&mut self) {
        // cursive event loop
        while self.cursive.is_running() {
            self.cursive.step();
            for event in self.event_manager.msg_iter() {
                match event {
                    Event::Player(state) => {
                        trace!("event received: {:?}", state);
                        self.spotify.update_status(state.clone());

                        if state == PlayerEvent::FinishedTrack {
                            self.queue.next(false);
                        }
                    }
                    Event::Queue(event) => {
                        self.queue.handle_event(event);
                    }
                    Event::SessionDied => self.spotify.start_worker(None),
                }
            }
        }
    }
}
