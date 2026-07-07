use wgpu_l3::scratch::glass_tuner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if std::env::args().any(|arg| arg == "--smoke") {
        return glass_tuner::smoke();
    }

    glass_tuner::run(glass_tuner::State::default())?;
    Ok(())
}
