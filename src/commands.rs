use std::sync::Arc;
use std::time::Duration;

use crate::application::send_command;
use crate::command::{
    Command, GotoMode, JumpMode, MoveAmount, MoveMode, SeekDirection, ShiftMode, TargetMode,
};
use crate::events::EventManager;
use crate::ext_traits::CursiveExt;
use crate::fs::cache_path;
use crate::library::Library;
use crate::queue::{Queue, RepeatSetting};
use crate::spotify::{Spotify, VOLUME_PERCENT};
use crate::traits::{IntoBoxedViewExt, ViewExt};
use crate::ui::contextmenu::ContextMenu;
use crate::ui::help::HelpView;
use crate::ui::layout::Layout;
use crate::ui::modal::Modal;
use crate::ui::search_results::SearchResultsView;

use cursive::event::{Event, Key};
use cursive::traits::View;
use cursive::views::Dialog;
use cursive::Cursive;
use log::{debug, info};

pub enum CommandResult {
    Consumed(Option<String>),
    View(Box<dyn ViewExt>),
    Modal(Box<dyn View>),
    Ignored,
}

pub struct CommandManager {
    spotify: Spotify,
    queue: Arc<Queue>,
    library: Arc<Library>,
    events: EventManager,
}

impl CommandManager {
    pub fn new(
        spotify: Spotify,
        queue: Arc<Queue>,
        library: Arc<Library>,
        events: EventManager,
    ) -> Self {
        Self {
            spotify,
            queue,
            library,
            events,
        }
    }

    fn handle_default_commands(
        &self,
        s: &mut Cursive,
        cmd: &Command,
    ) -> Result<Option<String>, String> {
        match cmd {
            Command::Noop => Ok(None),
            Command::Quit => {
                s.quit();
                Ok(None)
            }
            Command::Redraw => {
                info!("Redrawing screen");
                s.clear();
                Ok(None)
            }
            Command::Stop => {
                self.queue.stop();
                Ok(None)
            }
            Command::Previous => {
                if self.spotify.get_current_progress() < Duration::from_secs(5) {
                    self.queue.previous();
                } else {
                    self.spotify.seek(0);
                }
                Ok(None)
            }
            Command::Next => {
                self.queue.next(true);
                Ok(None)
            }
            Command::Clear => {
                let queue = self.queue.clone();
                let confirmation = Dialog::text("Clear queue?")
                    .button("Yes", move |s| {
                        s.pop_layer();
                        queue.clear()
                    })
                    .dismiss_button("No");
                s.add_layer(Modal::new(confirmation));
                Ok(None)
            }
            Command::UpdateLibrary => {
                self.library.update_library();
                Ok(None)
            }
            Command::TogglePlay => {
                self.queue.toggleplayback();
                Ok(None)
            }
            Command::Shuffle(mode) => {
                let mode = mode.unwrap_or_else(|| !self.queue.get_shuffle());
                self.queue.set_shuffle(mode);
                Ok(None)
            }
            Command::Repeat(mode) => {
                let mode = mode.unwrap_or_else(|| match self.queue.get_repeat() {
                    RepeatSetting::None => RepeatSetting::RepeatPlaylist,
                    RepeatSetting::RepeatPlaylist => RepeatSetting::RepeatTrack,
                    RepeatSetting::RepeatTrack => RepeatSetting::None,
                });

                self.queue.set_repeat(mode);
                Ok(None)
            }
            Command::Seek(direction) => {
                match *direction {
                    SeekDirection::Relative(rel) => self.spotify.seek_relative(rel),
                    SeekDirection::Absolute(abs) => self.spotify.seek(abs),
                }
                Ok(None)
            }
            Command::VolumeUp(amount) => {
                let volume = self
                    .spotify
                    .volume()
                    .saturating_add(VOLUME_PERCENT * amount);
                self.spotify.set_volume(volume);
                Ok(None)
            }
            Command::VolumeDown(amount) => {
                let volume = self
                    .spotify
                    .volume()
                    .saturating_sub(VOLUME_PERCENT * amount);
                debug!("vol {}", volume);
                self.spotify.set_volume(volume);
                Ok(None)
            }
            Command::Help => {
                let view = Box::new(HelpView::new());
                s.call_on_name("main", move |v: &mut Layout| v.push_view(view));
                Ok(None)
            }
            Command::Search(term) => {
                let view = if !term.is_empty() {
                    Some(SearchResultsView::new(
                        term.clone(),
                        self.events.clone(),
                        self.queue.clone(),
                        self.library.clone(),
                    ))
                } else {
                    None
                };
                s.call_on_name("main", |v: &mut Layout| {
                    v.set_screen("search");
                    if let Some(results) = view {
                        v.push_view(results.into_boxed_view_ext())
                    }
                });
                Ok(None)
            }
            Command::Logout => {
                self.spotify.shutdown();

                let mut credentials_path = cache_path("librespot");
                credentials_path.push("credentials.json");
                std::fs::remove_file(credentials_path).unwrap();

                s.quit();
                Ok(None)
            }
            Command::Execute(cmd) => {
                log::info!("Executing command: {}", cmd);
                let cmd = std::ffi::CString::new(cmd.clone()).unwrap();
                let result = unsafe { libc::system(cmd.as_ptr()) };
                log::info!("Exit code: {}", result);
                Ok(None)
            }
            Command::Reconnect => {
                self.spotify.shutdown();
                Ok(None)
            }

            Command::Queue
            | Command::PlayNext
            | Command::Play
            | Command::Focus(_)
            | Command::Back
            | Command::Open(_)
            | Command::Goto(_)
            | Command::Move(_, _)
            | Command::Shift(_, _)
            | Command::Jump(_)
            | Command::ShowRecommendations(_)
            | Command::Sort(_, _) => Err(format!(
                "The command \"{}\" is unsupported in this view",
                cmd.basename()
            )),
        }
    }

    fn handle_callbacks(&self, s: &mut Cursive, cmd: &Command) -> Result<Option<String>, String> {
        let local = if let Some(mut contextmenu) = s.find_name::<ContextMenu>("contextmenu") {
            contextmenu.on_command(s, cmd)?
        } else {
            s.on_layout(|siv, mut l| l.on_command(siv, cmd))?
        };

        if let CommandResult::Consumed(output) = local {
            Ok(output)
        } else if let CommandResult::Modal(modal) = local {
            s.add_layer(modal);
            Ok(None)
        } else if let CommandResult::View(view) = local {
            s.call_on_name("main", move |v: &mut Layout| {
                v.push_view(view);
            });

            Ok(None)
        } else {
            self.handle_default_commands(s, cmd)
        }
    }


    pub fn handle(&self, s: &mut Cursive, cmd: Command) {
        let result = self.handle_callbacks(s, &cmd);

        s.call_on_name("main", |v: &mut Layout| {
            v.set_result(result);
        });

        s.on_event(Event::Refresh);
    }

    pub fn register_keybindings(&self, cursive: &mut Cursive) {
        cursive.add_global_callback(Event::Char('q'), move |siv| send_command(siv, Command::Quit));

        cursive.add_global_callback(Event::CtrlChar('l'), move |siv| send_command(siv, Command::Redraw));
        cursive.add_global_callback(Event::Char('P'), move |siv| send_command(siv, Command::TogglePlay));
        cursive.add_global_callback(Event::Char('U'), move |siv| send_command(siv, Command::UpdateLibrary));
        cursive.add_global_callback(Event::Char('S'), move |siv| send_command(siv, Command::Stop));
        cursive.add_global_callback(Event::Char('<'), move |siv| send_command(siv, Command::Previous));
        cursive.add_global_callback(Event::Char('>'), move |siv| send_command(siv, Command::Next));
        cursive.add_global_callback(Event::Char('c'), move |siv| send_command(siv, Command::Clear));

        cursive.add_global_callback(Event::Char(' '), move |siv| send_command(siv, Command::Queue));
        cursive.add_global_callback(Event::Char(' '), move |siv| send_command(siv, Command::Move(MoveMode::Down, Default::default())));
        cursive.add_global_callback(Event::Char('.'), move |siv| send_command(siv, Command::PlayNext));
        cursive.add_global_callback(Event::Char('.'), move |siv| send_command(siv, Command::Move(MoveMode::Down, Default::default())));

        cursive.add_global_callback(Event::Key(Key::Enter), move |siv| send_command(siv, Command::Play));
        cursive.add_global_callback(Event::Char('n'), move |siv| send_command(siv, Command::Jump(JumpMode::Next)));
        cursive.add_global_callback(Event::Char('N'), move |siv| send_command(siv, Command::Jump(JumpMode::Previous)));
        cursive.add_global_callback(Event::Char('f'), move |siv| send_command(siv, Command::Seek(SeekDirection::Relative(1000))));
        cursive.add_global_callback(Event::Char('b'), move |siv| send_command(siv, Command::Seek(SeekDirection::Relative(-1000))));
        cursive.add_global_callback(Event::Char('F'), move |siv| send_command(siv, Command::Seek(SeekDirection::Relative(10000))));
        cursive.add_global_callback(Event::Char('B'), move |siv| send_command(siv, Command::Seek(SeekDirection::Relative(-10000))));
        cursive.add_global_callback(Event::Char('+'), move |siv| send_command(siv, Command::VolumeUp(1)));
        cursive.add_global_callback(Event::Char(']'), move |siv| send_command(siv, Command::VolumeUp(5)));
        cursive.add_global_callback(Event::Char('-'), move |siv| send_command(siv, Command::VolumeDown(1)));
        cursive.add_global_callback(Event::Char('['), move |siv| send_command(siv, Command::VolumeDown(5)));

        cursive.add_global_callback(Event::Char('r'), move |siv| send_command(siv, Command::Repeat(None)));
        cursive.add_global_callback(Event::Char('z'), move |siv| send_command(siv, Command::Shuffle(None)));

        cursive.add_global_callback(Event::Key(Key::F1), move |siv| send_command(siv, Command::Focus("queue".into())));
        cursive.add_global_callback(Event::Key(Key::F2), move |siv| send_command(siv, Command::Focus("search".into())));
        cursive.add_global_callback(Event::Key(Key::F3), move |siv| send_command(siv, Command::Focus("library".into())));
        cursive.add_global_callback(Event::Char('?'), move |siv| send_command(siv, Command::Help));
        cursive.add_global_callback(Event::Key(Key::Backspace), move |siv| send_command(siv, Command::Back));

        cursive.add_global_callback(Event::Char('o'), move |siv| send_command(siv, Command::Open(TargetMode::Selected)));
        cursive.add_global_callback(Event::Char('O'), move |siv| send_command(siv, Command::Open(TargetMode::Current)));
        cursive.add_global_callback(Event::Char('a'), move |siv| send_command(siv, Command::Goto(GotoMode::Album)));
        cursive.add_global_callback(Event::Char('A'), move |siv| send_command(siv, Command::Goto(GotoMode::Artist)));

        cursive.add_global_callback(Event::Char('m'), move |siv| send_command(siv, Command::ShowRecommendations(TargetMode::Selected)));
        cursive.add_global_callback(Event::Char('M'), move |siv| send_command(siv, Command::ShowRecommendations(TargetMode::Current)));

        cursive.add_global_callback(Event::Key(Key::Up), move |siv| send_command(siv, Command::Move(MoveMode::Up, Default::default())));
        cursive.add_global_callback(Event::Char('p'), move |siv| send_command(siv, Command::Move(MoveMode::Playing, Default::default())));
        cursive.add_global_callback(Event::Key(Key::Down), move |siv| send_command(siv, Command::Move(MoveMode::Down, Default::default())));
        cursive.add_global_callback(Event::Key(Key::Left), move |siv| send_command(siv, Command::Move(MoveMode::Left, Default::default())));
        cursive.add_global_callback(Event::Key(Key::Right), move |siv| send_command(siv, Command::Move(MoveMode::Right, Default::default())));
        cursive.add_global_callback(Event::Key(Key::PageUp), move |siv| send_command(siv, Command::Move(MoveMode::Up, MoveAmount::Integer(5))));
        cursive.add_global_callback(Event::Key(Key::PageDown), move |siv| send_command(siv, Command::Move(MoveMode::Down, MoveAmount::Integer(5))));
        cursive.add_global_callback(Event::Key(Key::Home), move |siv| send_command(siv, Command::Move(MoveMode::Up, MoveAmount::Extreme)));
        cursive.add_global_callback(Event::Key(Key::End), move |siv| send_command(siv, Command::Move(MoveMode::Down, MoveAmount::Extreme)));
        cursive.add_global_callback(Event::Char('k'), move |siv| send_command(siv, Command::Move(MoveMode::Up, Default::default())));
        cursive.add_global_callback(Event::Char('j'), move |siv| send_command(siv, Command::Move(MoveMode::Down, Default::default())));
        cursive.add_global_callback(Event::Char('h'), move |siv| send_command(siv, Command::Move(MoveMode::Left, Default::default())));
        cursive.add_global_callback(Event::Char('l'), move |siv| send_command(siv, Command::Move(MoveMode::Right, Default::default())));

        cursive.add_global_callback(Event::CtrlChar('p'), move |siv| send_command(siv, Command::Move(MoveMode::Up, Default::default())));
        cursive.add_global_callback(Event::CtrlChar('n'), move |siv| send_command(siv, Command::Move(MoveMode::Down, Default::default())));
        cursive.add_global_callback(Event::CtrlChar('a'), move |siv| send_command(siv, Command::Move(MoveMode::Left, Default::default())));
        cursive.add_global_callback(Event::CtrlChar('e'), move |siv| send_command(siv, Command::Move(MoveMode::Right, Default::default())));

        cursive.add_global_callback(Event::Shift(Key::Up), move |siv| send_command(siv, Command::Shift(ShiftMode::Up, None)));
        cursive.add_global_callback(Event::Shift(Key::Down), move |siv| send_command(siv, Command::Shift(ShiftMode::Down, None)));
    }
}
