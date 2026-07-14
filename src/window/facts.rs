use crate::{color, geometry};

use super::{Id, Kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    id: Id,
    title: String,
    inner_size: geometry::Size,
    canvas_color: color::Color,
    kind: Kind,
}

impl Facts {
    pub(crate) fn new(
        id: Id,
        title: impl Into<String>,
        inner_size: geometry::Size,
        canvas_color: color::Color,
        kind: Kind,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            inner_size,
            canvas_color,
            kind,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn inner_size(&self) -> geometry::Size {
        self.inner_size
    }

    pub fn canvas_color(&self) -> color::Color {
        self.canvas_color
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub(crate) fn set_inner_size(&mut self, inner_size: geometry::Size) {
        self.inner_size = inner_size;
    }

    pub(crate) fn replace_preserving_inner_size(&mut self, facts: &Self) {
        let inner_size = self.inner_size;
        self.clone_from(facts);
        self.inner_size = inner_size;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_window_layer_wraps_the_facts_owner() {
        for (name, source) in [
            ("host", include_str!("../host/window.rs")),
            ("shell", include_str!("../shell/window.rs")),
            ("session", include_str!("../session/window.rs")),
            ("platform", include_str!("../platform/backend.rs")),
        ] {
            assert_window_struct_uses_facts(name, source, "Window");
            if name == "session" {
                assert_window_struct_uses_facts(name, source, "WindowSnapshot");
            }
        }
    }

    fn assert_window_struct_uses_facts(layer: &str, source: &str, type_name: &str) {
        let marker = format!("pub struct {type_name} {{");
        let body = source
            .split_once(&marker)
            .and_then(|(_, rest)| rest.split_once("\n}"))
            .map(|(body, _)| body)
            .unwrap_or_else(|| panic!("{layer} {type_name} declaration should exist"));
        assert!(
            body.contains("facts:"),
            "{layer} {type_name} must wrap window::Facts"
        );
        for duplicate in [
            "id:",
            "title:",
            "size:",
            "inner_size:",
            "canvas_color:",
            "kind:",
        ] {
            for line in body.lines() {
                let field = line.trim().trim_start_matches("pub(super) ");
                assert!(
                    !field.starts_with(duplicate),
                    "{layer} {type_name} must not restore duplicate fact field {duplicate}"
                );
            }
        }
    }
}
