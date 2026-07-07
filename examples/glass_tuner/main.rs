pub use wgpu_l3::*;

#[path = "app/mod.rs"]
mod glass_tuner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if std::env::args().any(|arg| arg == "--smoke") {
        return smoke();
    }

    glass_tuner::run(glass_tuner::State::default())?;
    Ok(())
}

fn smoke() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = glass_tuner::app(glass_tuner::State::default());
    app.start();

    let window = app
        .session()
        .windows()
        .first()
        .ok_or_else(|| std::io::Error::other("glass tuner did not open a window"))?
        .id();
    let rendered = app
        .render_scene(window, glass_tuner::window_size())
        .ok_or_else(|| std::io::Error::other("glass tuner did not render"))?;

    if rendered.scene().is_empty() {
        return Err(std::io::Error::other("glass tuner scene is empty").into());
    }
    if !rendered
        .scene()
        .texts()
        .iter()
        .any(|text| text.value().contains("[floating-panel]"))
    {
        return Err(std::io::Error::other("glass tuner TOML preview did not paint").into());
    }

    Ok(())
}
