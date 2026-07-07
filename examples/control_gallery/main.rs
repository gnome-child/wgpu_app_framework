pub use wgpu_l3::*;

#[path = "app/mod.rs"]
mod control_gallery;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    if std::env::args().any(|arg| arg == "--smoke") {
        return smoke();
    }

    control_gallery::run(control_gallery::State::default())?;
    Ok(())
}

fn smoke() -> Result {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app
        .session()
        .windows()
        .first()
        .ok_or_else(|| std::io::Error::other("control gallery did not open a window"))?
        .id();
    let rendered = app
        .render_scene(window, control_gallery::window_size())
        .ok_or_else(|| std::io::Error::other("control gallery did not render"))?;

    if rendered.scene().is_empty() {
        return Err(std::io::Error::other("control gallery scene is empty").into());
    }
    if !rendered
        .scene()
        .texts()
        .iter()
        .any(|text| text.value() == "Interactive Controls")
    {
        return Err(std::io::Error::other("control gallery heading did not paint").into());
    }

    Ok(())
}
