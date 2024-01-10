use std::sync::Arc;

use cursive::view::ViewWrapper;
use cursive::Cursive;

use crate::command::Command;
use crate::commands::CommandResult;
use crate::library::Library;
use crate::model::playlist::Playlist;
use crate::queue::Queue;
use crate::traits::ViewExt;
use crate::ui::listview::ListView;

pub struct PlaylistsView {
    list: ListView<Playlist>,
}

impl PlaylistsView {
    pub fn new(queue: Arc<Queue>, library: Arc<Library>) -> Self {
        Self {
            list: ListView::new(library.playlists.clone(), queue, library.clone()),
        }
    }
}

impl ViewWrapper for PlaylistsView {
    wrap_impl!(self.list: ListView<Playlist>);
}

impl ViewExt for PlaylistsView {
    fn title(&self) -> String {
        "Playlists".to_string()
    }

    fn on_command(&mut self, s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        self.list.on_command(s, cmd)
    }
}
