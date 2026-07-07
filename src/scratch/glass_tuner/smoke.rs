use std::{error::Error, io};

use super::super::{geometry, view};
use super::{
    AcrylicToken, State,
    command::{SetToken, TogglePanel},
    view::window_size,
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn smoke() -> Result {
    let mut app = super::app(State::default());
    app.start();

    let window = app
        .session()
        .windows()
        .first()
        .ok_or_else(|| io::Error::other("glass tuner did not open a window"))?
        .id();
    let size = window_size();

    let initial = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("glass tuner did not render"))?;
    if initial.scene().filters().is_empty() {
        return Err(io::Error::other("open tuner panel did not emit a filter").into());
    }
    if !initial
        .scene()
        .texts()
        .iter()
        .any(|text| text.value().contains("noise-opacity = 0.022"))
    {
        return Err(io::Error::other("acrylic tuner did not paint default TOML values").into());
    }
    if initial.scene().filters().iter().any(|filter| {
        filter
            .ops()
            .iter()
            .any(|op| matches!(op, super::super::scene::FilterOp::Refraction(_)))
    }) {
        return Err(io::Error::other("default acrylic tuner emitted refraction").into());
    }

    click_role_with_label(
        &mut app,
        window,
        size,
        view::node::Role::Button,
        "Hide panel",
    )?;
    if app.state().panel_open {
        return Err(io::Error::other("toggle button did not hide panel").into());
    }

    let hidden = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("glass tuner did not render hidden panel"))?;
    if !hidden.scene().filters().is_empty() {
        return Err(io::Error::other("hidden tuner panel still emitted filters").into());
    }

    app.invoke(app.trigger::<TogglePanel>(()))
        .output
        .map_err(|error| io::Error::other(format!("toggle command failed: {error:?}")))?;
    app.invoke(app.trigger::<SetToken>((AcrylicToken::NoiseOpacity, 0.04)))
        .output
        .map_err(|error| io::Error::other(format!("set token command failed: {error:?}")))?;

    if (app.state().noise_opacity - 0.04).abs() > f64::EPSILON {
        return Err(io::Error::other("set token did not update noise opacity").into());
    }

    let tuned = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("glass tuner did not render tuned panel"))?;
    if !tuned
        .scene()
        .texts()
        .iter()
        .any(|text| text.value().contains("noise-opacity = 0.040"))
    {
        return Err(io::Error::other("tuned TOML noise opacity did not paint").into());
    }

    Ok(())
}

fn click_role_with_label(
    app: &mut super::super::Runtime<State, (), super::super::View>,
    window: super::super::window::Id,
    size: geometry::Size,
    role: view::node::Role,
    label: &str,
) -> Result {
    let rendered = app
        .render_scene(window, size)
        .ok_or_else(|| io::Error::other("glass tuner did not render before click"))?;
    let rect = rendered
        .layout()
        .find_role(role)
        .into_iter()
        .find(|frame| frame.label_text().unwrap_or_default() == label)
        .ok_or_else(|| io::Error::other(format!("missing {role:?} with label {label:?}")))?
        .rect();
    let point = center(rect);

    app.pointer_down_at(window, size, point)?;
    app.render_scene(window, size)
        .ok_or_else(|| io::Error::other("glass tuner did not present after pointer down"))?;
    app.pointer_up_at(window, size, point)?;

    Ok(())
}

fn center(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(rect.height() / 2),
    )
}
