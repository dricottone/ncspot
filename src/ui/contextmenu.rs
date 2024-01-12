use std::sync::Arc;

use cursive::view::{Margins, ViewWrapper};
use cursive::views::{Dialog, NamedView, SelectView};
use cursive::Cursive;

use crate::commands::CommandResult;
use crate::ext_traits::SelectViewExt;
use crate::library::Library;
use crate::model::track::Track;
use crate::queue::Queue;
use crate::spotify::PlayerEvent;
use crate::traits::{ListItem, ViewExt};
use crate::ui::layout::Layout;
use crate::ui::modal::Modal;
use crate::command::Command;
use cursive::traits::{Finder, Nameable};

pub struct ContextMenu {
    dialog: Modal<Dialog>,
}


enum ContextMenuAction {
    ShowItem(Box<dyn ListItem>),
    ShowRecommendations(Box<Track>),
    Play(Box<dyn ListItem>),
    PlayNext(Box<dyn ListItem>),
    TogglePlayback,
    Queue(Box<dyn ListItem>),
}

impl ContextMenu {
    pub fn new(item: &dyn ListItem, queue: Arc<Queue>, library: Arc<Library>) -> NamedView<Self> {
        let mut content: SelectView<ContextMenuAction> = SelectView::new();
        let album = item.album(&queue);

        if item.is_playable() {
            if item.is_playing(&queue)
                && queue.get_spotify().get_current_status()
                    == PlayerEvent::Paused(queue.get_spotify().get_current_progress())
            {
                // the item is the current track, but paused
                content.insert_item(0, "Resume", ContextMenuAction::TogglePlayback);
            } else if !item.is_playing(&queue) {
                // the item is not the current track
                content.insert_item(0, "Play", ContextMenuAction::Play(item.as_listitem()));
            } else {
                // the item is the current track and playing
                content.insert_item(0, "Pause", ContextMenuAction::TogglePlayback);
            }
            content.insert_item(
                1,
                "Play next",
                ContextMenuAction::PlayNext(item.as_listitem()),
            );
            content.insert_item(2, "Queue", ContextMenuAction::Queue(item.as_listitem()));
        }

        // Note: currently cannot return None
        for a in item.artists().unwrap().iter() {
            content.add_item(
                format!("Show {}", a.name),
                ContextMenuAction::ShowItem(Box::new(a.clone())),
            )
        }

        if let Some(ref a) = album {
            content.add_item(
                "Show album",
                ContextMenuAction::ShowItem(Box::new(a.clone())),
            );
        }

        if let Some(t) = item.track() {
            content.add_item(
                "Similar tracks",
                ContextMenuAction::ShowRecommendations(Box::new(t)),
            )
        }

        // open detail view of artist/album
        {
            let library = library.clone();
            content.set_on_submit(move |s: &mut Cursive, action: &ContextMenuAction| {
                let queue = queue.clone();
                let library = library.clone();
                s.pop_layer();

                match action {
                    ContextMenuAction::ShowItem(item) => {
                        if let Some(view) = item.open(queue, library) {
                            s.call_on_name("main", move |v: &mut Layout| v.push_view(view));
                        }
                    }
                    ContextMenuAction::ShowRecommendations(item) => {
                        if let Some(view) = item.to_owned().open_recommendations(queue, library) {
                            s.call_on_name("main", move |v: &mut Layout| v.push_view(view));
                        }
                    }
                    ContextMenuAction::Play(item) => item.as_listitem().play(&queue),
                    ContextMenuAction::PlayNext(item) => item.as_listitem().play_next(&queue),
                    ContextMenuAction::TogglePlayback => queue.toggleplayback(),
                    ContextMenuAction::Queue(item) => item.as_listitem().queue(&queue),
                }
            });
        }

        let dialog = Dialog::new()
            .title(item.display_left(&library))
            .dismiss_button("Close")
            .padding(Margins::lrtb(1, 1, 1, 0))
            .content(content.with_name("contextmenu_select"));
        Self {
            dialog: Modal::new_ext(dialog),
        }
        .with_name("contextmenu")
    }
}

impl ViewExt for ContextMenu {
    fn on_command(&mut self, s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        handle_move_command::<ContextMenuAction>(&mut self.dialog, s, cmd, "contextmenu_select")
    }
}

fn handle_move_command<T: 'static>(
    sel: &mut Modal<Dialog>,
    s: &mut Cursive,
    cmd: &Command,
    name: &str,
) -> Result<CommandResult, String> {
    match cmd {
        Command::Back => {
            s.pop_layer();
            Ok(CommandResult::Consumed(None))
        }
        Command::Move(_, _) => sel
            .call_on_name(name, |select: &mut SelectView<T>| {
                select.handle_command(cmd)
            })
            .unwrap_or(Ok(CommandResult::Consumed(None))),
        _ => Ok(CommandResult::Consumed(None)),
    }
}

impl ViewWrapper for ContextMenu {
    wrap_impl!(self.dialog: Modal<Dialog>);
}
