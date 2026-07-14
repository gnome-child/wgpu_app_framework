use crate::{geometry::Rect, theme::Theme, view};

use super::super::{Icon, Scene};

pub(super) fn paint(rect: Rect, hint: &view::Hint, scene: &mut Scene, theme: &Theme) {
    let Some(icon) = hint.icon() else {
        return;
    };
    let auxiliary = theme.auxiliary_panel();
    let color = match hint.tone() {
        view::Tone::Neutral => auxiliary.info,
        view::Tone::Warning => auxiliary.warning,
        view::Tone::Error => auxiliary.error,
    };
    scene.push_icon(Icon::new(rect, icon, color, rect.width().max(0) as f32));
}
