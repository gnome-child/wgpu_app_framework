use wgpu_l3::Document as TextDocument;

pub const STRESS_TEXT: &str = include_str!("../fixtures/unicode_stress_dump.txt");

#[derive(Clone)]
pub struct State {
    pub document: TextDocument,
    pub wrap_text: bool,
    pub show_debug_panel: bool,
    pub last_status: String,
    pub save_generation: u64,
}

impl wgpu_l3::State for State {}

impl Default for State {
    fn default() -> Self {
        Self {
            document: TextDocument::default(),
            wrap_text: true,
            show_debug_panel: false,
            last_status: "ready".to_owned(),
            save_generation: 0,
        }
    }
}
