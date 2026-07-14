#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id {
    name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Icon {
    id: Id,
    style: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Glyph {
    family: &'static str,
    codepoint: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Style {
    #[default]
    Regular,
    Filled,
    Light,
    Thin,
    Bold,
    Duotone,
}

impl Id {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    pub const fn as_str(self) -> &'static str {
        self.name
    }
}

impl Icon {
    pub const fn phosphor(id: Id) -> Self {
        Self {
            id,
            style: Style::Regular,
        }
    }

    pub const fn id(self) -> Id {
        self.id
    }

    pub const fn style(self) -> Style {
        self.style
    }

    pub const fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn glyph(self) -> Option<Glyph> {
        phosphor_icon(self.id.as_str(), self.style)
            .ok()
            .and_then(|icon| Glyph::new(icon.family, icon.codepoint))
    }
}

fn phosphor_icon(name: &str, style: Style) -> Result<iconflow::IconRef, iconflow::IconError> {
    let iconflow_style = style.to_iconflow();

    iconflow::try_icon(
        iconflow::Pack::Phosphor,
        name,
        iconflow_style,
        iconflow::Size::Regular,
    )
    .or_else(|_| {
        let Some(suffix) = style.phosphor_suffix() else {
            return iconflow::try_icon(
                iconflow::Pack::Phosphor,
                name,
                iconflow_style,
                iconflow::Size::Regular,
            );
        };
        let name = format!("{name}-{suffix}");

        iconflow::try_icon(
            iconflow::Pack::Phosphor,
            &name,
            iconflow_style,
            iconflow::Size::Regular,
        )
    })
}

impl Glyph {
    pub const fn new(family: &'static str, codepoint: u32) -> Option<Self> {
        if family.is_empty() || codepoint == 0 {
            None
        } else {
            Some(Self { family, codepoint })
        }
    }

    pub const fn family(self) -> &'static str {
        self.family
    }

    pub const fn codepoint(self) -> u32 {
        self.codepoint
    }

    pub fn character(self) -> Option<char> {
        char::from_u32(self.codepoint)
    }
}

impl Style {
    fn to_iconflow(self) -> iconflow::Style {
        match self {
            Self::Regular => iconflow::Style::Regular,
            Self::Filled => iconflow::Style::Filled,
            Self::Light => iconflow::Style::Light,
            Self::Thin => iconflow::Style::Thin,
            Self::Bold => iconflow::Style::Bold,
            Self::Duotone => iconflow::Style::Duotone,
        }
    }

    const fn phosphor_suffix(self) -> Option<&'static str> {
        match self {
            Self::Regular => None,
            Self::Filled => Some("fill"),
            Self::Light => Some("light"),
            Self::Thin => Some("thin"),
            Self::Bold => Some("bold"),
            Self::Duotone => Some("duotone"),
        }
    }
}

pub(crate) fn font_bytes() -> impl Iterator<Item = &'static [u8]> {
    iconflow::fonts().iter().map(|font| font.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phosphor_icon_resolves_to_font_glyph() {
        let glyph = Icon::phosphor(Id::new("check"))
            .glyph()
            .expect("check icon should resolve");

        assert_eq!(glyph.family(), "Phosphor Regular");
        assert!(glyph.codepoint() > 0);
        assert!(glyph.character().is_some());
    }

    #[test]
    fn missing_icon_resolves_to_none() {
        let icon = Icon::phosphor(Id::new("missing-framework-icon"));

        assert_eq!(icon.glyph(), None);
    }

    #[test]
    fn icon_style_can_be_selected() {
        let icon = Icon::phosphor(Id::new("check")).with_style(Style::Bold);
        let glyph = icon.glyph().expect("bold check icon should resolve");

        assert_eq!(icon.style(), Style::Bold);
        assert_eq!(glyph.family(), "Phosphor Bold");
    }

    #[test]
    fn icon_owner_exposes_every_embedded_font_as_nonempty_bytes() {
        let fonts = font_bytes().collect::<Vec<_>>();

        assert!(!fonts.is_empty());
        assert!(fonts.iter().all(|font| !font.is_empty()));
    }
}
