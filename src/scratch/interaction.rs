use std::any::TypeId;

use crate::text;

use super::{context::Source, draft, session};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Interaction {
    open_menu: Option<Menu>,
    pointer: Pointer,
    scroll: Scroll,
    text_input: draft::Input,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Menu {
    id: Id,
    label: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pointer {
    hovered: Option<Target>,
    pressed: Option<Target>,
    capture: Option<Capture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target {
    kind: Kind,
    identity: Identity,
    label: String,
    captures: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    target: Target,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Scroll {
    offsets: Vec<ScrollEntry>,
    reveal_requests: Vec<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScrollEntry {
    target: Target,
    offset: ScrollOffset,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollOffset {
    x: i32,
    y: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollDelta {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Identity {
    Element(Id),
    CommandPath {
        command_type: TypeId,
        source: Source,
        path: Vec<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Menu,
    Command,
    TextArea,
    Popup,
    Label,
}

impl Interaction {
    pub fn open_menu(&self) -> Option<&Menu> {
        self.open_menu.as_ref()
    }

    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    pub fn scroll(&self) -> &Scroll {
        &self.scroll
    }

    pub fn text_input(&self) -> &draft::Input {
        &self.text_input
    }

    pub(super) fn open_menu_with(&mut self, menu: Menu) -> bool {
        let changed = self.open_menu.as_ref() != Some(&menu);
        self.open_menu = Some(menu);
        changed
    }

    pub(super) fn toggle_menu(&mut self, menu: Menu) -> bool {
        if self.open_menu.as_ref() == Some(&menu) {
            self.open_menu = None;
        } else {
            self.open_menu = Some(menu);
        }

        true
    }

    pub(super) fn close_menu(&mut self) -> bool {
        let changed = self.open_menu.is_some();
        self.open_menu = None;
        changed
    }

    pub(super) fn pointer_move(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.hovered != target;
        self.pointer.hovered = target;
        changed
    }

    pub(super) fn pointer_down(&mut self, target: Target) -> bool {
        let changed = self.pointer.hovered.as_ref() != Some(&target)
            || self.pointer.pressed.as_ref() != Some(&target)
            || self.pointer.capture.as_ref().map(Capture::target)
                != target.captures().then_some(&target);
        self.pointer.hovered = Some(target.clone());
        self.pointer.pressed = Some(target.clone());
        self.pointer.capture = target.captures().then(|| Capture::new(target));
        changed
    }

    pub(super) fn pointer_up(&mut self, target: Option<Target>) -> bool {
        let changed = self.pointer.pressed.is_some()
            || self.pointer.capture.is_some()
            || self.pointer.hovered != target;
        self.pointer.pressed = None;
        self.pointer.capture = None;
        self.pointer.hovered = target;
        changed
    }

    pub(super) fn pointer_left(&mut self) -> bool {
        let changed = self.pointer.hovered.is_some()
            || (self.pointer.capture.is_none() && self.pointer.pressed.is_some());
        self.pointer.hovered = None;
        if self.pointer.capture.is_none() {
            self.pointer.pressed = None;
        }
        changed
    }

    pub(super) fn cancel_pointer(&mut self) -> bool {
        let changed = self.pointer.pressed.is_some() || self.pointer.capture.is_some();
        self.pointer.pressed = None;
        self.pointer.capture = None;
        changed
    }

    pub(super) fn scroll_by(&mut self, target: Target, delta: ScrollDelta) -> bool {
        self.scroll.scroll_by(target, delta)
    }

    pub(super) fn scroll_to(&mut self, target: Target, offset: ScrollOffset) -> bool {
        self.scroll.scroll_to(target, offset)
    }

    pub(super) fn reveal_scroll(&mut self, target: Target) -> bool {
        self.scroll.reveal(target)
    }

    pub(super) fn clear_scroll_reveal(&mut self, target: &Target) -> bool {
        self.scroll.clear_reveal(target)
    }

    pub(super) fn set_text_preedit(&mut self, target: Target, preedit: text::Preedit) -> bool {
        self.text_input.set_preedit(target, preedit)
    }

    pub(super) fn edit_text_draft(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> draft::Change {
        self.text_input.edit(target, base, edit)
    }

    pub(super) fn clear_text_input(&mut self) -> bool {
        self.text_input.clear()
    }

    pub(super) fn clear_text_input_unless(&mut self, target: &Target) -> bool {
        self.text_input.clear_unless(target)
    }
}

impl Menu {
    pub fn new(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl From<&'static str> for Id {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl Pointer {
    pub fn hovered(&self) -> Option<&Target> {
        self.hovered.as_ref()
    }

    pub fn pressed(&self) -> Option<&Target> {
        self.pressed.as_ref()
    }

    pub fn capture(&self) -> Option<&Capture> {
        self.capture.as_ref()
    }
}

impl Capture {
    fn new(target: Target) -> Self {
        Self { target }
    }

    pub fn target(&self) -> &Target {
        &self.target
    }
}

impl Scroll {
    pub fn offset(&self, target: &Target) -> ScrollOffset {
        self.offsets
            .iter()
            .find(|entry| &entry.target == target)
            .map(|entry| entry.offset)
            .unwrap_or_default()
    }

    pub fn should_reveal(&self, target: &Target) -> bool {
        self.reveal_requests.iter().any(|request| request == target)
    }

    fn scroll_by(&mut self, target: Target, delta: ScrollDelta) -> bool {
        if delta.is_zero() {
            return false;
        }

        let Some(index) = self.offsets.iter().position(|entry| entry.target == target) else {
            let offset = ScrollOffset::default().scrolled_by(delta);
            if offset.is_zero() {
                return false;
            }

            self.offsets.push(ScrollEntry { target, offset });
            return true;
        };

        let before = self.offsets[index].offset;
        let offset = before.scrolled_by(delta);
        if offset.is_zero() {
            self.offsets.remove(index);
        } else {
            self.offsets[index].offset = offset;
        }

        before != offset
    }

    fn scroll_to(&mut self, target: Target, offset: ScrollOffset) -> bool {
        let Some(index) = self.offsets.iter().position(|entry| entry.target == target) else {
            if offset.is_zero() {
                return false;
            }

            self.offsets.push(ScrollEntry { target, offset });
            return true;
        };

        let before = self.offsets[index].offset;
        if offset.is_zero() {
            self.offsets.remove(index);
        } else {
            self.offsets[index].offset = offset;
        }

        before != offset
    }

    fn reveal(&mut self, target: Target) -> bool {
        if self.should_reveal(&target) {
            return false;
        }

        self.reveal_requests.push(target);
        true
    }

    fn clear_reveal(&mut self, target: &Target) -> bool {
        let Some(index) = self
            .reveal_requests
            .iter()
            .position(|request| request == target)
        else {
            return false;
        };

        self.reveal_requests.remove(index);
        true
    }
}

impl ScrollOffset {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    fn scrolled_by(self, delta: ScrollDelta) -> Self {
        Self {
            x: self.x.saturating_add(delta.x),
            y: self.y.saturating_add(delta.y),
        }
    }

    fn is_zero(self) -> bool {
        self.x == 0 && self.y == 0
    }
}

impl ScrollDelta {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn horizontal(x: i32) -> Self {
        Self { x, y: 0 }
    }

    pub fn vertical(y: i32) -> Self {
        Self { x: 0, y }
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    fn is_zero(self) -> bool {
        self.x == 0 && self.y == 0
    }
}

impl Target {
    pub fn menu(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Menu, id, label)
    }

    pub fn command_element(id: impl Into<Id>, command_name: &'static str) -> Self {
        Self {
            kind: Kind::Command,
            identity: Identity::Element(id.into()),
            label: command_name.to_owned(),
            captures: false,
        }
    }

    pub fn command_path(
        command_type: TypeId,
        command_name: &'static str,
        source: Source,
        path: impl Into<Vec<usize>>,
    ) -> Self {
        Self {
            kind: Kind::Command,
            identity: Identity::CommandPath {
                command_type,
                source,
                path: path.into(),
            },
            label: command_name.to_owned(),
            captures: false,
        }
    }

    pub fn text_area(focus: session::Focus) -> Self {
        Self::text_area_id(focus.target())
    }

    pub fn text_area_id(id: impl Into<Id>) -> Self {
        let id = id.into();
        Self {
            kind: Kind::TextArea,
            identity: Identity::Element(id),
            label: id.as_str().to_owned(),
            captures: true,
        }
    }

    pub fn popup(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Popup, id, label)
    }

    pub fn label(id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self::new(Kind::Label, id, label)
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn label_text(&self) -> &str {
        &self.label
    }

    pub fn captures(&self) -> bool {
        self.captures
    }

    pub fn with_capture(mut self) -> Self {
        self.captures = true;
        self
    }

    fn new(kind: Kind, id: impl Into<Id>, label: impl Into<String>) -> Self {
        Self {
            kind,
            identity: Identity::Element(id.into()),
            label: label.into(),
            captures: false,
        }
    }
}
