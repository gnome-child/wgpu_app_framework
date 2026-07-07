use crate::text;

use super::{command, input};

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
    key: input::Key,
    modifiers: input::Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBoxShortcut {
    ClearSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::scratch) struct ShortcutDisplay {
    runs: Vec<ShortcutRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::scratch) enum ShortcutRun {
    Text(String),
    Icon(ShortcutIcon),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::scratch) enum ShortcutIcon {
    Control,
    Shift,
    Alt,
    Option,
    Command,
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
    pub(crate) fn for_platform(self, _platform: Platform) -> ResolvedDisplayStyle {
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
        key: input::Key,
        modifiers: input::Modifiers,
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

    pub(in crate::scratch) fn display_parts(
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

    pub fn edit_for_key(
        self,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> Option<text::edit::Edit> {
        match self.platform {
            Platform::Mac => mac_edit_for_key(key, modifiers),
            Platform::Windows | Platform::Linux => windows_edit_for_key(key, modifiers),
        }
    }

    pub fn text_box_shortcut_for_key(
        self,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> Option<TextBoxShortcut> {
        let clear = match self.platform {
            Platform::Mac => [
                ConcreteChord::new(
                    input::Key::Character('a'),
                    input::Modifiers::new(true, false, false, true),
                ),
                ConcreteChord::new(
                    input::Key::Character('\\'),
                    input::Modifiers::new(false, true, false, false),
                ),
            ],
            Platform::Windows | Platform::Linux => [
                ConcreteChord::new(
                    input::Key::Character('a'),
                    input::Modifiers::new(true, true, false, false),
                ),
                ConcreteChord::new(
                    input::Key::Character('\\'),
                    input::Modifiers::new(false, true, false, false),
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
        use input::Key;

        let primary = self.primary_modifier();
        let primary_shift = with_shift(primary);
        match (self.platform, standard) {
            (Platform::Mac, Standard::CloseWindow) => vec![ConcreteChord::new(
                Key::Character('w'),
                input::Modifiers::new(false, false, false, true),
            )],
            (Platform::Windows | Platform::Linux, Standard::CloseWindow) => {
                vec![ConcreteChord::new(
                    Key::F4,
                    input::Modifiers::new(false, false, true, false),
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

    fn primary_modifier(self) -> input::Modifiers {
        match self.platform {
            Platform::Mac => input::Modifiers::new(false, false, false, true),
            Platform::Windows | Platform::Linux => input::Modifiers::new(false, true, false, false),
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::current()
    }
}

impl ConcreteChord {
    pub const fn new(key: input::Key, modifiers: input::Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub const fn key(self) -> input::Key {
        self.key
    }

    pub const fn modifiers(self) -> input::Modifiers {
        self.modifiers
    }

    pub fn matches_key(self, key: input::Key, modifiers: input::Modifiers) -> bool {
        self.key.normalized() == key.normalized() && self.modifiers == modifiers
    }

    fn display(self, platform: Platform, style: DisplayStyle) -> String {
        match style.for_platform(platform) {
            ResolvedDisplayStyle::Symbols => self.symbol_display_text(platform),
            ResolvedDisplayStyle::Text => self.text_display(false),
        }
    }

    fn display_parts(self, platform: Platform, style: DisplayStyle) -> ShortcutDisplay {
        match style.for_platform(platform) {
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
        runs.push(ShortcutRun::Text(key_text(self.key, false)));

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

    pub(in crate::scratch) fn runs(&self) -> &[ShortcutRun] {
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
    pub(in crate::scratch) const fn fallback_text(self) -> &'static str {
        match self {
            Self::Control => "Ctrl",
            Self::Shift => "Shift",
            Self::Alt => "Alt",
            Self::Option => "Option",
            Self::Command => "Command",
        }
    }
}

fn with_shift(modifiers: input::Modifiers) -> input::Modifiers {
    input::Modifiers::new(
        true,
        modifiers.control(),
        modifiers.alt(),
        modifiers.super_key(),
    )
}

fn key_symbol(key: input::Key) -> String {
    match key.normalized() {
        input::Key::Tab => "⇥".to_owned(),
        input::Key::Enter => "↩".to_owned(),
        input::Key::Space => "Space".to_owned(),
        input::Key::Escape => "⎋".to_owned(),
        input::Key::Backspace => "⌫".to_owned(),
        input::Key::Delete => "⌦".to_owned(),
        input::Key::ArrowLeft => "←".to_owned(),
        input::Key::ArrowRight => "→".to_owned(),
        input::Key::ArrowUp => "↑".to_owned(),
        input::Key::ArrowDown => "↓".to_owned(),
        input::Key::Home => "↖".to_owned(),
        input::Key::End => "↘".to_owned(),
        input::Key::PageUp => "⇞".to_owned(),
        input::Key::PageDown => "⇟".to_owned(),
        input::Key::F4 => "F4".to_owned(),
        input::Key::Character(' ') => "Space".to_owned(),
        input::Key::Character(character) => character.to_uppercase().collect(),
        input::Key::Other => "?".to_owned(),
    }
}

fn key_text(key: input::Key, glyph_keys: bool) -> String {
    if glyph_keys {
        match key.normalized() {
            input::Key::ArrowLeft
            | input::Key::ArrowRight
            | input::Key::ArrowUp
            | input::Key::ArrowDown
            | input::Key::Home
            | input::Key::End
            | input::Key::PageUp
            | input::Key::PageDown => return key_symbol(key),
            _ => {}
        }
    }

    match key.normalized() {
        input::Key::Tab => "Tab".to_owned(),
        input::Key::Enter => "Enter".to_owned(),
        input::Key::Space => "Space".to_owned(),
        input::Key::Escape => "Esc".to_owned(),
        input::Key::Backspace => "Backspace".to_owned(),
        input::Key::Delete => "Delete".to_owned(),
        input::Key::ArrowLeft => "Left".to_owned(),
        input::Key::ArrowRight => "Right".to_owned(),
        input::Key::ArrowUp => "Up".to_owned(),
        input::Key::ArrowDown => "Down".to_owned(),
        input::Key::Home => "Home".to_owned(),
        input::Key::End => "End".to_owned(),
        input::Key::PageUp => "PageUp".to_owned(),
        input::Key::PageDown => "PageDown".to_owned(),
        input::Key::F4 => "F4".to_owned(),
        input::Key::Character(' ') => "Space".to_owned(),
        input::Key::Character(character) => character.to_uppercase().collect(),
        input::Key::Other => "?".to_owned(),
    }
}

fn windows_edit_for_key(key: input::Key, modifiers: input::Modifiers) -> Option<text::edit::Edit> {
    if modifiers.alt() || modifiers.super_key() {
        return None;
    }

    let key = key.normalized();
    let control = modifiers.control();
    let extend = modifiers.shift();

    match key {
        input::Key::Backspace if control => Some(text::edit::Edit::delete_word_backward()),
        input::Key::Backspace => Some(text::edit::Edit::backspace()),
        input::Key::Delete if control => Some(text::edit::Edit::delete_word_forward()),
        input::Key::Delete => Some(text::edit::Edit::delete()),
        input::Key::Enter if !control => Some(text::edit::Edit::insert_line_break()),
        input::Key::ArrowLeft => Some(motion_edit(
            if control {
                text::edit::Motion::WordPrevious
            } else {
                text::edit::Motion::VisualLeft
            },
            extend,
        )),
        input::Key::ArrowRight => Some(motion_edit(
            if control {
                text::edit::Motion::WordNext
            } else {
                text::edit::Motion::VisualRight
            },
            extend,
        )),
        input::Key::ArrowUp if !control => Some(motion_edit(text::edit::Motion::VisualUp, extend)),
        input::Key::ArrowDown if !control => {
            Some(motion_edit(text::edit::Motion::VisualDown, extend))
        }
        input::Key::Home => Some(motion_edit(
            if control {
                text::edit::Motion::DocumentStart
            } else {
                text::edit::Motion::LineStart
            },
            extend,
        )),
        input::Key::End => Some(motion_edit(
            if control {
                text::edit::Motion::DocumentEnd
            } else {
                text::edit::Motion::LineEnd
            },
            extend,
        )),
        input::Key::PageUp if !control => Some(motion_edit(text::edit::Motion::PageUp, extend)),
        input::Key::PageDown if !control => Some(motion_edit(text::edit::Motion::PageDown, extend)),
        input::Key::Tab
        | input::Key::Space
        | input::Key::Escape
        | input::Key::Enter
        | input::Key::ArrowUp
        | input::Key::ArrowDown
        | input::Key::PageUp
        | input::Key::PageDown
        | input::Key::F4
        | input::Key::Character(_)
        | input::Key::Other => None,
    }
}

fn mac_edit_for_key(key: input::Key, modifiers: input::Modifiers) -> Option<text::edit::Edit> {
    if modifiers.control() || (modifiers.alt() && modifiers.super_key()) {
        return None;
    }

    let key = key.normalized();
    let option = modifiers.alt();
    let command = modifiers.super_key();
    let extend = modifiers.shift();

    match key {
        input::Key::Backspace if option => Some(text::edit::Edit::delete_word_backward()),
        input::Key::Backspace => Some(text::edit::Edit::backspace()),
        input::Key::Delete if option => Some(text::edit::Edit::delete_word_forward()),
        input::Key::Delete => Some(text::edit::Edit::delete()),
        input::Key::Enter if !option && !command => Some(text::edit::Edit::insert_line_break()),
        input::Key::ArrowLeft => Some(motion_edit(
            if command {
                text::edit::Motion::LineStart
            } else if option {
                text::edit::Motion::WordPrevious
            } else {
                text::edit::Motion::VisualLeft
            },
            extend,
        )),
        input::Key::ArrowRight => Some(motion_edit(
            if command {
                text::edit::Motion::LineEnd
            } else if option {
                text::edit::Motion::WordNext
            } else {
                text::edit::Motion::VisualRight
            },
            extend,
        )),
        input::Key::ArrowUp if command => {
            Some(motion_edit(text::edit::Motion::DocumentStart, extend))
        }
        input::Key::ArrowDown if command => {
            Some(motion_edit(text::edit::Motion::DocumentEnd, extend))
        }
        input::Key::ArrowUp if !option => Some(motion_edit(text::edit::Motion::VisualUp, extend)),
        input::Key::ArrowDown if !option => {
            Some(motion_edit(text::edit::Motion::VisualDown, extend))
        }
        input::Key::Home | input::Key::End => None,
        input::Key::PageUp if !option && !command => {
            Some(motion_edit(text::edit::Motion::PageUp, extend))
        }
        input::Key::PageDown if !option && !command => {
            Some(motion_edit(text::edit::Motion::PageDown, extend))
        }
        input::Key::Tab
        | input::Key::Space
        | input::Key::Escape
        | input::Key::Enter
        | input::Key::ArrowUp
        | input::Key::ArrowDown
        | input::Key::PageUp
        | input::Key::PageDown
        | input::Key::F4
        | input::Key::Character(_)
        | input::Key::Other => None,
    }
}

fn motion_edit(motion: text::edit::Motion, extend: bool) -> text::edit::Edit {
    if extend {
        text::edit::Edit::extend_position(motion)
    } else {
        text::edit::Edit::move_position(motion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scratch::command::{KeyChord, Standard};

    #[test]
    fn primary_resolves_per_platform() {
        let chord = KeyChord::new("Primary+Z");
        assert!(Profile::windows().matches(
            chord,
            input::Key::Character('z'),
            input::Modifiers::new(false, true, false, false)
        ));
        assert!(!Profile::windows().matches(
            chord,
            input::Key::Character('z'),
            input::Modifiers::new(false, false, false, true)
        ));
        assert!(Profile::mac().matches(
            chord,
            input::Key::Character('z'),
            input::Modifiers::new(false, false, false, true)
        ));
    }

    #[test]
    fn standard_redo_has_multiple_windows_chords_and_one_display() {
        let chord = KeyChord::standard(Standard::Redo);
        assert!(Profile::windows().matches(
            chord,
            input::Key::Character('y'),
            input::Modifiers::new(false, true, false, false)
        ));
        assert!(Profile::windows().matches(
            chord,
            input::Key::Character('z'),
            input::Modifiers::new(true, true, false, false)
        ));
        assert_eq!(
            Profile::windows().display(chord, DisplayStyle::Default),
            "Ctrl+Y"
        );
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
    }

    #[test]
    fn text_edit_motion_mapping_is_profile_owned() {
        assert_eq!(
            Profile::windows().edit_for_key(
                input::Key::ArrowLeft,
                input::Modifiers::new(false, true, false, false),
            ),
            Some(text::edit::Edit::move_position(
                text::edit::Motion::WordPrevious
            ))
        );
        assert_eq!(
            Profile::mac().edit_for_key(
                input::Key::ArrowLeft,
                input::Modifiers::new(false, false, true, false),
            ),
            Some(text::edit::Edit::move_position(
                text::edit::Motion::WordPrevious
            ))
        );
        assert_eq!(
            Profile::mac().edit_for_key(
                input::Key::ArrowLeft,
                input::Modifiers::new(false, false, false, true),
            ),
            Some(text::edit::Edit::move_position(
                text::edit::Motion::LineStart
            ))
        );
        assert_eq!(
            Profile::mac().edit_for_key(input::Key::Home, input::Modifiers::default()),
            None
        );
    }
}
