use crate::text;

use super::{command, keyboard};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Mac,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    Default,
    Symbols,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    platform: Platform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConcreteChord {
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBoxShortcut {
    ClearSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TextOperation {
    Selection(text::selection::Operation),
    Edit(text::Edit),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShortcutDisplay {
    runs: Vec<ShortcutRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ShortcutRun {
    Text(String),
    Icon(ShortcutIcon),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ShortcutIcon {
    Control,
    Shift,
    Alt,
    Option,
    Command,
    Delete,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Windows
        }
    }
}

impl DisplayStyle {
    pub(crate) fn resolve(self) -> ResolvedDisplayStyle {
        match self {
            Self::Default => ResolvedDisplayStyle::Symbols,
            Self::Symbols => ResolvedDisplayStyle::Symbols,
            Self::Text => ResolvedDisplayStyle::Text,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedDisplayStyle {
    Symbols,
    Text,
}

impl Profile {
    pub const fn new(platform: Platform) -> Self {
        Self { platform }
    }

    pub const fn windows() -> Self {
        Self::new(Platform::Windows)
    }

    pub const fn mac() -> Self {
        Self::new(Platform::Mac)
    }

    pub const fn linux() -> Self {
        Self::new(Platform::Linux)
    }

    pub fn current() -> Self {
        Self::new(Platform::current())
    }

    pub const fn platform(self) -> Platform {
        self.platform
    }

    pub fn matches(
        self,
        chord: command::KeyChord,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> bool {
        self.chords(chord)
            .into_iter()
            .any(|chord| chord.matches_key(key, modifiers))
    }

    pub fn display(self, chord: command::KeyChord, style: DisplayStyle) -> String {
        self.chords(chord)
            .first()
            .map(|chord| chord.display(self.platform, style))
            .unwrap_or_else(|| chord.as_str().to_owned())
    }

    pub(crate) fn display_parts(
        self,
        chord: command::KeyChord,
        style: DisplayStyle,
    ) -> ShortcutDisplay {
        self.chords(chord)
            .first()
            .map(|chord| chord.display_parts(self.platform, style))
            .unwrap_or_else(|| ShortcutDisplay::text(chord.as_str()))
    }

    pub fn chords(self, chord: command::KeyChord) -> Vec<ConcreteChord> {
        match chord.kind() {
            command::KeyChordKind::Chord(chord) => {
                chord.resolve(self.platform).into_iter().collect()
            }
            command::KeyChordKind::Standard(standard) => self.standard_chords(standard),
        }
    }

    pub(crate) fn text_operation_for_key(
        self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Option<TextOperation> {
        match self.platform {
            Platform::Mac => mac_text_operation_for_key(key, modifiers),
            Platform::Windows | Platform::Linux => windows_text_operation_for_key(key, modifiers),
        }
    }

    pub fn text_box_shortcut_for_key(
        self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Option<TextBoxShortcut> {
        let clear = match self.platform {
            Platform::Mac => [
                ConcreteChord::new(
                    keyboard::Key::Character('a'),
                    keyboard::Modifiers::new(true, false, false, true),
                ),
                ConcreteChord::new(
                    keyboard::Key::Character('\\'),
                    keyboard::Modifiers::new(false, true, false, false),
                ),
            ],
            Platform::Windows | Platform::Linux => [
                ConcreteChord::new(
                    keyboard::Key::Character('a'),
                    keyboard::Modifiers::new(true, true, false, false),
                ),
                ConcreteChord::new(
                    keyboard::Key::Character('\\'),
                    keyboard::Modifiers::new(false, true, false, false),
                ),
            ],
        };

        clear
            .into_iter()
            .any(|chord| chord.matches_key(key, modifiers))
            .then_some(TextBoxShortcut::ClearSelection)
    }

    pub fn text_box_shortcut_for_chord(self, chord: command::KeyChord) -> Option<TextBoxShortcut> {
        self.chords(chord)
            .into_iter()
            .find_map(|chord| self.text_box_shortcut_for_key(chord.key(), chord.modifiers()))
    }

    fn standard_chords(self, standard: command::Standard) -> Vec<ConcreteChord> {
        use command::Standard;
        use keyboard::Key;

        let primary = self.primary_modifier();
        let primary_shift = with_shift(primary);
        match (self.platform, standard) {
            (Platform::Mac, Standard::CloseWindow) => vec![ConcreteChord::new(
                Key::Character('w'),
                keyboard::Modifiers::new(false, false, false, true),
            )],
            (Platform::Windows | Platform::Linux, Standard::CloseWindow) => {
                vec![ConcreteChord::new(
                    Key::F4,
                    keyboard::Modifiers::new(false, false, true, false),
                )]
            }
            (Platform::Windows | Platform::Linux, Standard::Redo) => vec![
                ConcreteChord::new(Key::Character('y'), primary),
                ConcreteChord::new(Key::Character('z'), primary_shift),
            ],
            (_, Standard::Redo) => {
                vec![ConcreteChord::new(Key::Character('z'), primary_shift)]
            }
            (_, Standard::Undo) => vec![ConcreteChord::new(Key::Character('z'), primary)],
            (_, Standard::Cut) => vec![ConcreteChord::new(Key::Character('x'), primary)],
            (_, Standard::Copy) => vec![ConcreteChord::new(Key::Character('c'), primary)],
            (_, Standard::Paste) => vec![ConcreteChord::new(Key::Character('v'), primary)],
            (_, Standard::Delete) => vec![ConcreteChord::new(
                Key::Delete,
                keyboard::Modifiers::new(false, false, false, false),
            )],
            (_, Standard::SelectAll) => vec![ConcreteChord::new(Key::Character('a'), primary)],
            (_, Standard::New) => vec![ConcreteChord::new(Key::Character('n'), primary)],
            (_, Standard::Open) => vec![ConcreteChord::new(Key::Character('o'), primary)],
            (_, Standard::Save) => vec![ConcreteChord::new(Key::Character('s'), primary)],
            (_, Standard::SaveAs) => vec![ConcreteChord::new(Key::Character('s'), primary_shift)],
            (_, Standard::CommandPalette) => {
                vec![ConcreteChord::new(Key::Character('p'), primary_shift)]
            }
        }
    }

    fn primary_modifier(self) -> keyboard::Modifiers {
        match self.platform {
            Platform::Mac => keyboard::Modifiers::new(false, false, false, true),
            Platform::Windows | Platform::Linux => {
                keyboard::Modifiers::new(false, true, false, false)
            }
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::current()
    }
}

impl ConcreteChord {
    pub const fn new(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub const fn key(self) -> keyboard::Key {
        self.key
    }

    pub const fn modifiers(self) -> keyboard::Modifiers {
        self.modifiers
    }

    pub fn matches_key(self, key: keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
        self.key.normalized() == key.normalized() && self.modifiers == modifiers
    }

    fn display(self, platform: Platform, style: DisplayStyle) -> String {
        match style.resolve() {
            ResolvedDisplayStyle::Symbols => self.symbol_display_text(platform),
            ResolvedDisplayStyle::Text => self.text_display(false),
        }
    }

    fn display_parts(self, platform: Platform, style: DisplayStyle) -> ShortcutDisplay {
        match style.resolve() {
            ResolvedDisplayStyle::Symbols => self.symbol_display_parts(platform),
            ResolvedDisplayStyle::Text => ShortcutDisplay::text(self.text_display(false)),
        }
    }

    fn symbol_display_text(self, platform: Platform) -> String {
        self.symbol_display_parts(platform).fallback_text()
    }

    fn symbol_display_parts(self, platform: Platform) -> ShortcutDisplay {
        let mut runs = Vec::new();
        if self.modifiers.control() {
            runs.push(ShortcutRun::Icon(ShortcutIcon::Control));
        }
        if self.modifiers.alt() {
            runs.push(ShortcutRun::Icon(match platform {
                Platform::Mac => ShortcutIcon::Option,
                Platform::Windows | Platform::Linux => ShortcutIcon::Alt,
            }));
        }
        if self.modifiers.shift() {
            runs.push(ShortcutRun::Icon(ShortcutIcon::Shift));
        }
        if self.modifiers.super_key() {
            match platform {
                Platform::Mac => runs.push(ShortcutRun::Icon(ShortcutIcon::Command)),
                Platform::Windows | Platform::Linux => {
                    runs.push(ShortcutRun::Text("Super".to_owned()));
                }
            }
        }
        match self.key.normalized() {
            keyboard::Key::Delete => runs.push(ShortcutRun::Icon(ShortcutIcon::Delete)),
            _ => runs.push(ShortcutRun::Text(key_text(self.key, true))),
        }

        ShortcutDisplay::chord(runs)
    }

    fn text_display(self, glyph_keys: bool) -> String {
        let mut parts = Vec::new();
        if self.modifiers.control() {
            parts.push("Ctrl".to_owned());
        }
        if self.modifiers.alt() {
            parts.push("Alt".to_owned());
        }
        if self.modifiers.shift() {
            parts.push("Shift".to_owned());
        }
        if self.modifiers.super_key() {
            parts.push("Super".to_owned());
        }
        parts.push(key_text(self.key, glyph_keys));
        parts.join("+")
    }
}

impl ShortcutDisplay {
    fn text(value: impl Into<String>) -> Self {
        Self {
            runs: vec![ShortcutRun::Text(value.into())],
        }
    }

    fn chord(runs: Vec<ShortcutRun>) -> Self {
        let mut separated = Vec::with_capacity(runs.len().saturating_mul(2).saturating_sub(1));
        for (index, run) in runs.into_iter().enumerate() {
            if index > 0 {
                separated.push(ShortcutRun::Text("+".to_owned()));
            }
            separated.push(run);
        }

        Self { runs: separated }
    }

    pub(crate) fn runs(&self) -> &[ShortcutRun] {
        &self.runs
    }

    fn fallback_text(&self) -> String {
        self.runs
            .iter()
            .map(|run| match run {
                ShortcutRun::Text(value) => value.clone(),
                ShortcutRun::Icon(icon) => icon.fallback_text().to_owned(),
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

impl ShortcutIcon {
    pub(crate) const fn fallback_text(self) -> &'static str {
        match self {
            Self::Control => "Ctrl",
            Self::Shift => "Shift",
            Self::Alt => "Alt",
            Self::Option => "Option",
            Self::Command => "Command",
            Self::Delete => "⌦",
        }
    }
}

fn with_shift(modifiers: keyboard::Modifiers) -> keyboard::Modifiers {
    keyboard::Modifiers::new(
        true,
        modifiers.control(),
        modifiers.alt(),
        modifiers.super_key(),
    )
}

fn key_symbol(key: keyboard::Key) -> String {
    match key.normalized() {
        keyboard::Key::Tab => "⇥".to_owned(),
        keyboard::Key::Enter => "↩".to_owned(),
        keyboard::Key::Space => "Space".to_owned(),
        keyboard::Key::Escape => "⎋".to_owned(),
        keyboard::Key::Backspace => "⌫".to_owned(),
        keyboard::Key::Delete => "⌦".to_owned(),
        keyboard::Key::ArrowLeft => "←".to_owned(),
        keyboard::Key::ArrowRight => "→".to_owned(),
        keyboard::Key::ArrowUp => "↑".to_owned(),
        keyboard::Key::ArrowDown => "↓".to_owned(),
        keyboard::Key::Home => "↖".to_owned(),
        keyboard::Key::End => "↘".to_owned(),
        keyboard::Key::PageUp => "⇞".to_owned(),
        keyboard::Key::PageDown => "⇟".to_owned(),
        keyboard::Key::F2 => "F2".to_owned(),
        keyboard::Key::F4 => "F4".to_owned(),
        keyboard::Key::F10 => "F10".to_owned(),
        keyboard::Key::ContextMenu => "Menu".to_owned(),
        keyboard::Key::Character(' ') => "Space".to_owned(),
        keyboard::Key::Character(character) => character.to_uppercase().collect(),
        keyboard::Key::Other => "?".to_owned(),
    }
}

fn key_text(key: keyboard::Key, glyph_keys: bool) -> String {
    if glyph_keys {
        match key.normalized() {
            keyboard::Key::ArrowLeft
            | keyboard::Key::ArrowRight
            | keyboard::Key::ArrowUp
            | keyboard::Key::ArrowDown
            | keyboard::Key::Home
            | keyboard::Key::End
            | keyboard::Key::PageUp
            | keyboard::Key::PageDown => return key_symbol(key),
            _ => {}
        }
    }

    match key.normalized() {
        keyboard::Key::Tab => "Tab".to_owned(),
        keyboard::Key::Enter => "Enter".to_owned(),
        keyboard::Key::Space => "Space".to_owned(),
        keyboard::Key::Escape => "Esc".to_owned(),
        keyboard::Key::Backspace => "Backspace".to_owned(),
        keyboard::Key::Delete => "Delete".to_owned(),
        keyboard::Key::ArrowLeft => "Left".to_owned(),
        keyboard::Key::ArrowRight => "Right".to_owned(),
        keyboard::Key::ArrowUp => "Up".to_owned(),
        keyboard::Key::ArrowDown => "Down".to_owned(),
        keyboard::Key::Home => "Home".to_owned(),
        keyboard::Key::End => "End".to_owned(),
        keyboard::Key::PageUp => "PageUp".to_owned(),
        keyboard::Key::PageDown => "PageDown".to_owned(),
        keyboard::Key::F2 => "F2".to_owned(),
        keyboard::Key::F4 => "F4".to_owned(),
        keyboard::Key::F10 => "F10".to_owned(),
        keyboard::Key::ContextMenu => "Menu".to_owned(),
        keyboard::Key::Character(' ') => "Space".to_owned(),
        keyboard::Key::Character(character) => character.to_uppercase().collect(),
        keyboard::Key::Other => "?".to_owned(),
    }
}

fn windows_text_operation_for_key(
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<TextOperation> {
    if modifiers.alt() || modifiers.super_key() {
        return None;
    }

    let key = key.normalized();
    let control = modifiers.control();
    let extend = modifiers.shift();

    match key {
        keyboard::Key::Backspace if control => {
            Some(TextOperation::Edit(text::Edit::delete_word_backward()))
        }
        keyboard::Key::Backspace => Some(TextOperation::Edit(text::Edit::backspace())),
        keyboard::Key::Delete if control => {
            Some(TextOperation::Edit(text::Edit::delete_word_forward()))
        }
        keyboard::Key::Delete => Some(TextOperation::Edit(text::Edit::delete())),
        keyboard::Key::Enter if !control => {
            Some(TextOperation::Edit(text::Edit::insert_line_break()))
        }
        keyboard::Key::ArrowLeft => Some(motion_operation(
            if control {
                text::selection::Motion::WordPrevious
            } else {
                text::selection::Motion::VisualLeft
            },
            extend,
        )),
        keyboard::Key::ArrowRight => Some(motion_operation(
            if control {
                text::selection::Motion::WordNext
            } else {
                text::selection::Motion::VisualRight
            },
            extend,
        )),
        keyboard::Key::ArrowUp if !control => {
            Some(motion_operation(text::selection::Motion::VisualUp, extend))
        }
        keyboard::Key::ArrowDown if !control => Some(motion_operation(
            text::selection::Motion::VisualDown,
            extend,
        )),
        keyboard::Key::Home => Some(motion_operation(
            if control {
                text::selection::Motion::DocumentStart
            } else {
                text::selection::Motion::LineStart
            },
            extend,
        )),
        keyboard::Key::End => Some(motion_operation(
            if control {
                text::selection::Motion::DocumentEnd
            } else {
                text::selection::Motion::LineEnd
            },
            extend,
        )),
        keyboard::Key::PageUp if !control => {
            Some(motion_operation(text::selection::Motion::PageUp, extend))
        }
        keyboard::Key::PageDown if !control => {
            Some(motion_operation(text::selection::Motion::PageDown, extend))
        }
        keyboard::Key::Tab
        | keyboard::Key::Space
        | keyboard::Key::Escape
        | keyboard::Key::Enter
        | keyboard::Key::ArrowUp
        | keyboard::Key::ArrowDown
        | keyboard::Key::PageUp
        | keyboard::Key::PageDown
        | keyboard::Key::F2
        | keyboard::Key::F4
        | keyboard::Key::F10
        | keyboard::Key::ContextMenu
        | keyboard::Key::Character(_)
        | keyboard::Key::Other => None,
    }
}

fn mac_text_operation_for_key(
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<TextOperation> {
    if modifiers.control() || (modifiers.alt() && modifiers.super_key()) {
        return None;
    }

    let key = key.normalized();
    let option = modifiers.alt();
    let command = modifiers.super_key();
    let extend = modifiers.shift();

    match key {
        keyboard::Key::Backspace if option => {
            Some(TextOperation::Edit(text::Edit::delete_word_backward()))
        }
        keyboard::Key::Backspace => Some(TextOperation::Edit(text::Edit::backspace())),
        keyboard::Key::Delete if option => {
            Some(TextOperation::Edit(text::Edit::delete_word_forward()))
        }
        keyboard::Key::Delete => Some(TextOperation::Edit(text::Edit::delete())),
        keyboard::Key::Enter if !option && !command => {
            Some(TextOperation::Edit(text::Edit::insert_line_break()))
        }
        keyboard::Key::ArrowLeft => Some(motion_operation(
            if command {
                text::selection::Motion::LineStart
            } else if option {
                text::selection::Motion::WordPrevious
            } else {
                text::selection::Motion::VisualLeft
            },
            extend,
        )),
        keyboard::Key::ArrowRight => Some(motion_operation(
            if command {
                text::selection::Motion::LineEnd
            } else if option {
                text::selection::Motion::WordNext
            } else {
                text::selection::Motion::VisualRight
            },
            extend,
        )),
        keyboard::Key::ArrowUp if command => Some(motion_operation(
            text::selection::Motion::DocumentStart,
            extend,
        )),
        keyboard::Key::ArrowDown if command => Some(motion_operation(
            text::selection::Motion::DocumentEnd,
            extend,
        )),
        keyboard::Key::ArrowUp if !option => {
            Some(motion_operation(text::selection::Motion::VisualUp, extend))
        }
        keyboard::Key::ArrowDown if !option => Some(motion_operation(
            text::selection::Motion::VisualDown,
            extend,
        )),
        keyboard::Key::Home | keyboard::Key::End => None,
        keyboard::Key::PageUp if !option && !command => {
            Some(motion_operation(text::selection::Motion::PageUp, extend))
        }
        keyboard::Key::PageDown if !option && !command => {
            Some(motion_operation(text::selection::Motion::PageDown, extend))
        }
        keyboard::Key::Tab
        | keyboard::Key::Space
        | keyboard::Key::Escape
        | keyboard::Key::Enter
        | keyboard::Key::ArrowUp
        | keyboard::Key::ArrowDown
        | keyboard::Key::PageUp
        | keyboard::Key::PageDown
        | keyboard::Key::F2
        | keyboard::Key::F4
        | keyboard::Key::F10
        | keyboard::Key::ContextMenu
        | keyboard::Key::Character(_)
        | keyboard::Key::Other => None,
    }
}

fn motion_operation(motion: text::selection::Motion, extend: bool) -> TextOperation {
    TextOperation::Selection(if extend {
        text::selection::Operation::extend_position(motion)
    } else {
        text::selection::Operation::move_position(motion)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{KeyChord, Standard};

    #[test]
    fn primary_resolves_per_platform() {
        let chord = KeyChord::new("Primary+Z");
        assert!(Profile::windows().matches(
            chord,
            keyboard::Key::Character('z'),
            keyboard::Modifiers::new(false, true, false, false)
        ));
        assert!(!Profile::windows().matches(
            chord,
            keyboard::Key::Character('z'),
            keyboard::Modifiers::new(false, false, false, true)
        ));
        assert!(Profile::mac().matches(
            chord,
            keyboard::Key::Character('z'),
            keyboard::Modifiers::new(false, false, false, true)
        ));
    }

    #[test]
    fn standard_redo_has_multiple_windows_chords_and_one_display() {
        let chord = KeyChord::standard(Standard::Redo);
        assert!(Profile::windows().matches(
            chord,
            keyboard::Key::Character('y'),
            keyboard::Modifiers::new(false, true, false, false)
        ));
        assert!(Profile::windows().matches(
            chord,
            keyboard::Key::Character('z'),
            keyboard::Modifiers::new(true, true, false, false)
        ));
        assert_eq!(
            Profile::windows().display(chord, DisplayStyle::Default),
            "Ctrl+Y"
        );
    }

    #[test]
    fn every_standard_role_keeps_its_cross_platform_chord_projection() {
        use Standard::{
            CloseWindow, CommandPalette, Copy, Cut, Delete, New, Open, Paste, Redo, Save, SaveAs,
            SelectAll, Undo,
        };

        let cases = [
            (Undo, "Ctrl+Z", "Command+Z", 1, 1),
            (Redo, "Ctrl+Y", "Shift+Command+Z", 2, 1),
            (Cut, "Ctrl+X", "Command+X", 1, 1),
            (Copy, "Ctrl+C", "Command+C", 1, 1),
            (Paste, "Ctrl+V", "Command+V", 1, 1),
            (Delete, "⌦", "⌦", 1, 1),
            (SelectAll, "Ctrl+A", "Command+A", 1, 1),
            (New, "Ctrl+N", "Command+N", 1, 1),
            (Open, "Ctrl+O", "Command+O", 1, 1),
            (Save, "Ctrl+S", "Command+S", 1, 1),
            (SaveAs, "Ctrl+Shift+S", "Shift+Command+S", 1, 1),
            (CloseWindow, "Alt+F4", "Command+W", 1, 1),
            (CommandPalette, "Ctrl+Shift+P", "Shift+Command+P", 1, 1),
        ];

        for (standard, windows, mac, windows_count, mac_count) in cases {
            let chord = KeyChord::standard(standard);
            assert_eq!(
                Profile::windows().display(chord, DisplayStyle::Default),
                windows,
                "Windows display for {standard:?}"
            );
            assert_eq!(
                Profile::linux().display(chord, DisplayStyle::Default),
                windows,
                "Linux display for {standard:?}"
            );
            assert_eq!(
                Profile::mac().display(chord, DisplayStyle::Default),
                mac,
                "macOS display for {standard:?}"
            );
            assert_eq!(Profile::windows().chords(chord).len(), windows_count);
            assert_eq!(Profile::linux().chords(chord).len(), windows_count);
            assert_eq!(Profile::mac().chords(chord).len(), mac_count);
        }
    }

    #[test]
    fn mac_formatting_uses_canonical_symbol_order() {
        let redo = KeyChord::standard(Standard::Redo);
        assert_eq!(
            Profile::mac().display(redo, DisplayStyle::Default),
            "Shift+Command+Z"
        );

        let chord = KeyChord::new("Command+Shift+Option+Control+Z");
        assert_eq!(
            Profile::mac().display(chord, DisplayStyle::Symbols),
            "Ctrl+Option+Shift+Command+Z"
        );

        let alt = KeyChord::new("Alt+F4");
        assert_eq!(
            Profile::windows().display(alt, DisplayStyle::Default),
            "Alt+F4"
        );
        assert_eq!(
            Profile::windows().display(alt, DisplayStyle::Text),
            "Alt+F4"
        );
    }

    #[test]
    fn symbol_display_uses_icon_runs_with_plus_separators() {
        let display =
            Profile::windows().display_parts(KeyChord::new("Ctrl+C"), DisplayStyle::Default);

        assert_eq!(
            display.runs(),
            &[
                ShortcutRun::Icon(ShortcutIcon::Control),
                ShortcutRun::Text("+".to_owned()),
                ShortcutRun::Text("C".to_owned()),
            ]
        );

        let delete = Profile::windows()
            .display_parts(KeyChord::standard(Standard::Delete), DisplayStyle::Default);
        assert_eq!(
            delete.runs(),
            &[ShortcutRun::Icon(ShortcutIcon::Delete)],
            "the Delete key uses the same icon-run path as shortcut modifiers"
        );
    }

    #[test]
    fn text_operation_motion_mapping_is_profile_owned() {
        assert_eq!(
            Profile::windows().text_operation_for_key(
                keyboard::Key::ArrowLeft,
                keyboard::Modifiers::new(false, true, false, false),
            ),
            Some(TextOperation::Selection(
                text::selection::Operation::move_position(text::selection::Motion::WordPrevious)
            ))
        );
        assert_eq!(
            Profile::mac().text_operation_for_key(
                keyboard::Key::ArrowLeft,
                keyboard::Modifiers::new(false, false, true, false),
            ),
            Some(TextOperation::Selection(
                text::selection::Operation::move_position(text::selection::Motion::WordPrevious)
            ))
        );
        assert_eq!(
            Profile::mac().text_operation_for_key(
                keyboard::Key::ArrowLeft,
                keyboard::Modifiers::new(false, false, false, true),
            ),
            Some(TextOperation::Selection(
                text::selection::Operation::move_position(text::selection::Motion::LineStart)
            ))
        );
        assert_eq!(
            Profile::mac()
                .text_operation_for_key(keyboard::Key::Home, keyboard::Modifiers::default()),
            None
        );
    }
}
