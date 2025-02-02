use std::sync::Arc;

use cursive::view::ViewWrapper;
use cursive::Cursive;

use crate::command::Command;
use crate::commands::CommandResult;
use crate::library::Library;
use crate::queue::Queue;
use crate::traits::ViewExt;
use crate::ui::browse::BrowseView;
use crate::ui::listview::ListView;
use crate::ui::playlists::PlaylistsView;
use crate::ui::tabbedview::TabbedView;

pub struct LibraryView {
    tabs: TabbedView,
    display_name: Option<String>,
}

impl LibraryView {
    pub fn new(queue: Arc<Queue>, library: Arc<Library>) -> Self {
        let mut tabview = TabbedView::new();
        tabview.add_tab("Tracks", ListView::new(library.tracks.clone(), queue.clone(), library.clone()));
        tabview.add_tab("Albums", ListView::new(library.albums.clone(), queue.clone(), library.clone()));
        tabview.add_tab("Artists", ListView::new(library.artists.clone(), queue.clone(), library.clone()));
        tabview.add_tab("Playlists", PlaylistsView::new(queue.clone(), library.clone()));
        tabview.add_tab("Podcasts", ListView::new(library.shows.clone(), queue.clone(), library.clone()));
        tabview.add_tab("Browse", BrowseView::new(queue.clone(), library.clone()));

        Self {
            tabs: tabview,
            display_name: library.display_name.clone(),
        }
    }
}

impl ViewWrapper for LibraryView {
    wrap_impl!(self.tabs: TabbedView);
}

impl ViewExt for LibraryView {
    fn title(&self) -> String {
        if let Some(name) = &self.display_name {
            format!("Library of {name}")
        } else {
            "Library".to_string()
        }
    }

    fn on_command(&mut self, s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        self.tabs.on_command(s, cmd)
    }
}
