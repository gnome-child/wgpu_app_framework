#[derive(Debug, Clone)]
pub struct State {
    pub clicks: u32,
    pub wrap: bool,
    pub grid: bool,
    pub mode: Mode,
    pub level: f64,
    pub query: String,
    pub show_advanced: bool,
    pub last_status: String,
    pub records_descending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Design,
    Inspect,
    Preview,
}

impl State {
    pub fn reset(&mut self) {
        *self = Self::default();
        self.last_status = "reset controls".to_owned();
    }
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Design => "Design",
            Self::Inspect => "Inspect",
            Self::Preview => "Preview",
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            clicks: 0,
            wrap: true,
            grid: false,
            mode: Mode::Design,
            level: 42.0,
            query: String::new(),
            show_advanced: true,
            last_status: "ready".to_owned(),
            records_descending: false,
        }
    }
}

impl wgpu_l3::state::State for State {}
