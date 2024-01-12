use std::sync::Arc;

use cursive::view::{Nameable,Margins};
use cursive::views::{Dialog,NamedView, ScrollView, SelectView};

use crate::library::Library;
use crate::model::artist::Artist;
use crate::queue::Queue;
use crate::traits::ListItem;
use crate::ui::layout::Layout;
use crate::ui::modal::Modal;

pub fn select_artist(
    queue: Arc<Queue>,
    library: Arc<Library>,
    artists: Vec<Artist>
) -> NamedView<Modal<Dialog>> {
    let mut select = SelectView::<Artist>::new();
    for artist in artists {
        select.add_item(artist.name.clone(), artist);
    }
    select.set_on_submit(move |siv, a| {
        siv.pop_layer();
        if let Some(view) = a.open(queue.clone(), library.clone()) {
            siv.call_on_name("main", |v: &mut Layout| v.push_view(view));
        }
        // do something with this view
    });

    let dialog = Dialog::new()
        .title("Select artist")
        .dismiss_button("Close")
        .padding(Margins::lrtb(1, 1, 1, 0))
        .content(ScrollView::new(select.with_name("artist_select")));

    Modal::new_ext(dialog).with_name("selectartist")
}
