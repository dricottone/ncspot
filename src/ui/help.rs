use cursive::view::ViewWrapper;
use cursive::views::{ScrollView, TextView};
use cursive::Cursive;

use crate::command::{Command, MoveAmount, MoveMode};
use crate::commands::CommandResult;
use crate::traits::ViewExt;
use cursive::view::scroll::Scroller;

pub struct HelpView {
    view: ScrollView<TextView>,
}

impl HelpView {
    pub fn new() -> Self {
        let mut text = String::new();
        text.push_str("Playback control:\n");
        text.push_str(" P   play/pause\n");
        text.push_str(" S   stop\n");
        text.push_str(" >   next\n");
        text.push_str(" <   previous\n");
        text.push_str(" c   clear queue\n");
        text.push_str(" b   seek -1000\n");
        text.push_str(" f   seek +1000\n");
        text.push_str(" B   seek -10000\n");
        text.push_str(" F   seek +10000\n");
        text.push_str(" r   toggle repeat mode\n");
        text.push_str(" z   toggle shuffle mode\n");

        text.push_str("\nVolume control:\n");
        text.push_str(" +   increase by 1\n");
        text.push_str(" -   decrease by 1\n");
        text.push_str(" [   increase by 5\n");
        text.push_str(" ]   increase by 5\n");

        text.push_str("\nNavigation:\n");
        text.push_str(" ←, h, Ctrl+a   left 1\n");
        text.push_str(" ↑, k, Ctrl+p   up 1\n");
        text.push_str(" →, l, Ctrl+e   right 1\n");
        text.push_str(" ↓, j, Ctrl+n   down 1\n");
        text.push_str(" PageUp         up 5\n");
        text.push_str(" PageDown       down 5\n");
        text.push_str(" Home           go to top\n");
        text.push_str(" End            go to bottom\n");
        text.push_str(" p              go to playing\n");
        text.push_str(" F1             show queue tab\n");
        text.push_str(" F2             show search tab\n");
        text.push_str(" F3             show library tab\n");
        text.push_str(" Backspace      back\n");

        text.push_str("\nDisplay control:\n");
        text.push_str(" Ctrl+l   redraw screen\n");
        text.push_str(" :        begin entering a command\n");
        text.push_str(" /        begin searching\n");

        text.push_str("\nLibrary actions:\n");
        text.push_str(" Enter   play\n");
        text.push_str(" .       play next\n");
        text.push_str(" Space   add to queue\n");
        text.push_str(" s       save/favorite\n");
        text.push_str(" a       show album for selection\n");
        text.push_str(" A       show artist for selection\n");
        text.push_str(" m       show similar to selection\n");
        text.push_str(" M       show similar to playing\n");
        text.push_str(" o       show context menu for selection\n");
        text.push_str(" O       show context menu for playing\n");
        text.push_str(" U       update library\n");
        text.push_str(" q       quit\n");

        text.push_str("\nQueue actions:\n");
        text.push_str(" Shift+↑   swap selection and previous song\n");
        text.push_str(" Shift+↓   swap selection and next song\n");

        text.push_str("\nSearch actions:\n");
        text.push_str(" n   go to next\n");
        text.push_str(" N   go to previous\n");

        Self {
            view: ScrollView::new(TextView::new(text)),
        }
    }
}

impl ViewWrapper for HelpView {
    wrap_impl!(self.view: ScrollView<TextView>);
}

impl ViewExt for HelpView {
    fn title(&self) -> String {
        "Help".to_string()
    }

    fn on_command(&mut self, _s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        match cmd {
            Command::Help => Ok(CommandResult::Consumed(None)),
            Command::Move(mode, amount) => {
                let scroller = self.view.get_scroller_mut();
                let viewport = scroller.content_viewport();
                match mode {
                    MoveMode::Up => {
                        match amount {
                            MoveAmount::Extreme => {
                                self.view.scroll_to_top();
                            }
                            MoveAmount::Float(scale) => {
                                let amount = (viewport.height() as f32) * scale;
                                scroller
                                    .scroll_to_y(viewport.top().saturating_sub(amount as usize));
                            }
                            MoveAmount::Integer(amount) => scroller
                                .scroll_to_y(viewport.top().saturating_sub(*amount as usize)),
                        };
                        Ok(CommandResult::Consumed(None))
                    }
                    MoveMode::Down => {
                        match amount {
                            MoveAmount::Extreme => {
                                self.view.scroll_to_bottom();
                            }
                            MoveAmount::Float(scale) => {
                                let amount = (viewport.height() as f32) * scale;
                                scroller
                                    .scroll_to_y(viewport.bottom().saturating_add(amount as usize));
                            }
                            MoveAmount::Integer(amount) => scroller
                                .scroll_to_y(viewport.bottom().saturating_add(*amount as usize)),
                        };
                        Ok(CommandResult::Consumed(None))
                    }
                    _ => Ok(CommandResult::Consumed(None)),
                }
            }
            _ => Ok(CommandResult::Ignored),
        }
    }
}
