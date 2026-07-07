use wgpu_l3::text_editor;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    if std::env::args().any(|arg| arg == "--smoke") {
        return text_editor::smoke();
    }

    text_editor::run(text_editor::State::default())?;
    Ok(())
}
