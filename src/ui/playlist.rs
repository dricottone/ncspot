use std::sync::{Arc, RwLock};

use cursive::view::ViewWrapper;
use cursive::Cursive;

use crate::command::Command;
use crate::commands::CommandResult;
use crate::library::Library;
use crate::model::playable::Playable;
use crate::model::playlist::Playlist;
use crate::queue::Queue;

use crate::traits::ViewExt;
use crate::ui::listview::ListView;

pub struct PlaylistView {
    playlist: Playlist,
    list: ListView<Playable>,
    library: Arc<Library>,
    queue: Arc<Queue>,
}

impl PlaylistView {
    pub fn new(queue: Arc<Queue>, library: Arc<Library>, playlist: &Playlist) -> Self {
        let mut playlist = playlist.clone();
        playlist.load_tracks(queue.get_spotify());

        let tracks = if let Some(t) = playlist.tracks.as_ref() {
            t.clone()
        } else {
            Vec::new()
        };

        let list = ListView::new(
            Arc::new(RwLock::new(tracks)),
            queue.clone(),
            library.clone(),
        );

        Self {
            playlist,
            list,
            library,
            queue,
        }
    }
}

impl ViewWrapper for PlaylistView {
    wrap_impl!(self.list: ListView<Playable>);
}

impl ViewExt for PlaylistView {
    fn title(&self) -> String {
        self.playlist.name.clone()
    }

    fn title_sub(&self) -> String {
        if let Some(tracks) = self.playlist.tracks.as_ref() {
            let duration_secs = tracks.iter().map(|p| p.duration() as u64 / 1000).sum();
            let duration = std::time::Duration::from_secs(duration_secs);
            format!(
                "{} tracks, {}",
                tracks.len(),
                crate::utils::format_duration(&duration)
            )
        } else {
            "".to_string()
        }
    }

    fn on_command(&mut self, s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        if let Command::Sort(key, direction) = cmd {
            self.playlist.sort(key, direction);
            let tracks = self.playlist.tracks.as_ref().unwrap_or(&Vec::new()).clone();
            self.list = ListView::new(
                Arc::new(RwLock::new(tracks)),
                self.queue.clone(),
                self.library.clone(),
            );
            return Ok(CommandResult::Consumed(None));
        }

        self.list.on_command(s, cmd)
    }
}
