#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Invalidation {
    #[default]
    Paint,
    Layout,
    Rebuild,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Effect {
    #[default]
    None,
    Paint,
    Layout,
    Rebuild,
    CloseFloatingPanel,
    OpenFileDialog,
    SaveFileDialog,
    Batch(Vec<Effect>),
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

    pub fn invalidation(&self) -> Option<Invalidation> {
        match self {
            Self::Paint => Some(Invalidation::Paint),
            Self::Layout => Some(Invalidation::Layout),
            Self::Rebuild => Some(Invalidation::Rebuild),
            Self::Batch(effects) => effects.iter().filter_map(Effect::invalidation).max(),
            _ => None,
        }
    }

    pub fn contains_invalidation(&self) -> bool {
        self.invalidation().is_some()
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

    collapse_invalidations(effects);
}

fn collapse_invalidations(effects: &mut Vec<Effect>) {
    let Some(invalidation) = effects.iter().filter_map(Effect::invalidation).max() else {
        return;
    };
    effects.retain(|effect| effect.invalidation().is_none());
    effects.push(match invalidation {
        Invalidation::Paint => Effect::Paint,
        Invalidation::Layout => Effect::Layout,
        Invalidation::Rebuild => Effect::Rebuild,
    });
}

#[cfg(test)]
mod tests {
    use super::{Effect, Invalidation};

    #[test]
    fn invalidation_effects_merge_by_max_depth() {
        assert_eq!(
            Effect::Paint.then(Effect::Layout).invalidation(),
            Some(Invalidation::Layout)
        );
        assert_eq!(
            Effect::Layout.then(Effect::Paint).invalidation(),
            Some(Invalidation::Layout)
        );
        assert_eq!(
            Effect::Paint
                .then(Effect::Rebuild)
                .then(Effect::Layout)
                .invalidation(),
            Some(Invalidation::Rebuild)
        );
    }

    #[test]
    fn effect_composition_laws_hold_through_10000_deterministic_cases() {
        fn effect(index: usize) -> Effect {
            match index % 7 {
                0 => Effect::None,
                1 => Effect::Paint,
                2 => Effect::Layout,
                3 => Effect::Rebuild,
                4 => Effect::CloseFloatingPanel,
                5 => Effect::OpenFileDialog,
                _ => Effect::SaveFileDialog,
            }
        }

        for case in 0..10_000_usize {
            let a = effect(case);
            let b = effect(case / 7 + 3);
            let c = effect(case / 49 + 5);
            let left = a.clone().then(b.clone()).then(c.clone());
            let right = a.clone().then(b.clone().then(c.clone()));

            assert_eq!(left, right, "effect associativity case {case}");
            assert_eq!(
                a.clone().then(Effect::None),
                a,
                "right identity case {case}"
            );
            assert_eq!(Effect::None.then(b.clone()), b, "left identity case {case}");

            let expected_invalidation = [a, b, c].iter().filter_map(Effect::invalidation).max();
            assert_eq!(
                left.invalidation(),
                expected_invalidation,
                "maximum invalidation case {case}"
            );
        }
    }
}
