#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    None,
    Repaint,
    ClosePopup,
    OpenFileDialog,
    SaveFileDialog,
    Batch(Vec<Effect>),
}

impl Default for Effect {
    fn default() -> Self {
        Self::None
    }
}

impl Effect {
    pub fn then(self, next: Self) -> Self {
        let mut effects = Vec::new();
        collect_effects(self, &mut effects);
        collect_effects(next, &mut effects);

        match effects.len() {
            0 => Self::None,
            1 => effects.pop().expect("length was checked"),
            _ => Self::Batch(effects),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn contains(&self, effect: &Effect) -> bool {
        match self {
            Self::Batch(effects) => effects.iter().any(|item| item.contains(effect)),
            _ => self == effect,
        }
    }
}

fn collect_effects(effect: Effect, effects: &mut Vec<Effect>) {
    match effect {
        Effect::None => {}
        Effect::Batch(batch) => {
            for effect in batch {
                collect_effects(effect, effects);
            }
        }
        effect => {
            if !effects.contains(&effect) {
                effects.push(effect);
            }
        }
    }
}
