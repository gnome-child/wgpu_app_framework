mod binding;
mod control;
mod element;
mod layout;
mod menu;
pub mod panel;
mod root;
mod scroll;
mod trigger;
mod ui;

pub use binding::Binding;
pub use control::{Button, Checkbox, Label, Radio, Separator, Slider, TextArea, TextBox};
pub use element::Element;
pub use layout::{Direction, Layout};
pub use menu::{Menu, MenuBar, StandardMenuBar};
pub use panel::Panel;
pub use root::Root;
pub use scroll::Scroll;
pub use ui::Ui;

use super::view;

pub trait Widget {
    fn into_node(self) -> view::Node;

    /// Marks this widget as the nearest owner of an automatically derived
    /// context menu.
    fn context_menu(self) -> view::Node
    where
        Self: Sized,
    {
        self.into_node().with_context_menu()
    }
}

impl Widget for view::Node {
    fn into_node(self) -> view::Node {
        self
    }
}

/// Marks one widget as the nearest owner of an automatically derived context
/// menu. The widget's binding and exact responder/service targets supply the
/// candidates; this helper supplies no recipe of its own.
pub fn context_menu(widget: impl Widget) -> view::Node {
    widget.context_menu()
}

pub fn view(children: impl FnOnce(&mut Ui)) -> view::View {
    view::View::new(Root::new().children(children).into_node())
}

pub fn view_node(root: impl Widget) -> view::View {
    view::View::new(root.into_node())
}
