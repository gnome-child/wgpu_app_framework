#[path = "app/mod.rs"]
mod text_editor;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    if std::env::args().any(|arg| arg == "--smoke") {
        return smoke();
    }

    text_editor::run(text_editor::State::default())?;
    Ok(())
}

fn smoke() -> Result {
    let mut app = text_editor::app(text_editor::State::default());
    app.start();

    let window = app
        .session()
        .windows()
        .first()
        .ok_or_else(|| std::io::Error::other("text editor did not open a window"))?
        .id();
    let rendered = app
        .render_scene(window, text_editor::window_size())
        .ok_or_else(|| std::io::Error::other("text editor did not render"))?;

    if rendered.scene().is_empty() {
        return Err(std::io::Error::other("text editor scene is empty").into());
    }
    if !rendered
        .scene()
        .texts()
        .iter()
        .any(|text| text.value() == "File")
    {
        return Err(std::io::Error::other("text editor menu text did not paint").into());
    }

    Ok(())
}
