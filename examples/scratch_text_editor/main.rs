fn main()
-> Result<(), wgpu_l3::scratch::platform::RunError<wgpu_l3::scratch::platform::NativeError>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    wgpu_l3::scratch::text_editor::run(wgpu_l3::scratch::text_editor::State::default())
}
