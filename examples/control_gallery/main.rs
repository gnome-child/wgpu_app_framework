use wgpu_l3::scratch::control_gallery;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    if std::env::args().any(|arg| arg == "--smoke") {
        return control_gallery::smoke();
    }

    control_gallery::run(control_gallery::State::default())?;
    Ok(())
}
