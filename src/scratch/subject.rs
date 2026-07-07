/// Human-facing command-palette subject segment.
///
/// `name` is stable text for grouping, display/debug output, and future
/// serialization. It is not a routing identity; retained `NodeId`s and command
/// claims own runtime identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Segment {
    name: String,
    label: String,
}

/// Subject ancestry from outermost to nearest presented subject.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    segments: Vec<Segment>,
}

impl Segment {
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
        }
    }

    pub(in crate::scratch) fn from_label(label: &str) -> Self {
        Self::new(stable_name(label), label)
    }

    pub(in crate::scratch) fn from_name(name: &str) -> Self {
        Self::new(name, title_label(name))
    }

    pub fn application() -> Self {
        Self::new("application", "Application")
    }

    pub fn window() -> Self {
        Self::new("window", "Window")
    }

    pub fn system() -> Self {
        Self::new("system", "System")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Path {
    pub fn new(segments: impl Into<Vec<Segment>>) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    pub fn application() -> Self {
        Self::new([Segment::application()])
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn nearest(&self) -> Option<&Segment> {
        self.segments.last()
    }

    pub fn nearest_at(&self, depth: usize) -> Option<&Segment> {
        self.segments
            .len()
            .checked_sub(depth.saturating_add(1))
            .and_then(|index| self.segments.get(index))
    }
}

fn stable_name(label: &str) -> String {
    let mut name = String::new();
    let mut pending_separator = false;

    for ch in label.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            if pending_separator && !name.is_empty() {
                name.push('-');
            }
            name.push(ch);
            pending_separator = false;
        } else {
            pending_separator = !name.is_empty();
        }
    }

    if name.is_empty() {
        format!("subject-{:016x}", fnv1a64(label))
    } else {
        name
    }
}

fn title_label(name: &str) -> String {
    let mut label = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch == '_' || ch == '-' || ch == '.' {
            capitalize = true;
            label.push(' ');
            continue;
        }
        if capitalize {
            label.extend(ch.to_uppercase());
            capitalize = false;
        } else {
            label.push(ch);
        }
    }
    if label.is_empty() {
        "Subject".to_owned()
    } else {
        label
    }
}

fn fnv1a64(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
