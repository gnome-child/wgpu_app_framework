use std::collections::HashMap;

use super::{Key, Materialization, Provider};

/// Sparse cumulative-height projection for the variable virtual-list path.
/// Unseen rows retain one arithmetic estimate; only measured stable keys are
/// indexed, so distant lookup never walks preceding rows.
#[derive(Clone)]
pub(crate) struct Region {
    estimate: i32,
    len: usize,
    width: Option<i32>,
    measured: HashMap<Key, i32>,
    ordered: Vec<Entry>,
    prefix_delta: Vec<i64>,
    resolved_offset: i64,
    anchor: Option<Anchor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    index: usize,
    key: Key,
    height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Anchor {
    key: Key,
    relative: i32,
    fallback_index: usize,
}

impl Region {
    pub(crate) fn new(estimate: i32) -> Self {
        Self {
            estimate: estimate.max(1),
            len: 0,
            width: None,
            measured: HashMap::new(),
            ordered: Vec::new(),
            prefix_delta: Vec::new(),
            resolved_offset: 0,
            anchor: None,
        }
    }

    pub(crate) fn reconcile(&mut self, provider: &dyn Provider) {
        self.len = provider.len();
        self.measured.retain(|key, _| {
            provider
                .index_of(*key)
                .is_some_and(|index| index < self.len)
        });
        if self
            .anchor
            .is_some_and(|anchor| provider.index_of(anchor.key).is_none())
        {
            self.anchor = None;
        }
        self.rebuild(provider);
        self.resolved_offset = self.resolved_offset.clamp(0, self.max_offset());
    }

    pub(crate) fn request(
        &mut self,
        offset: i32,
        viewport_height: i32,
        overscan: usize,
        pins: Vec<Key>,
        provider: &dyn Provider,
    ) -> Materialization {
        self.reconcile(provider);
        self.resolved_offset = (offset.max(0) as i64).min(self.max_offset());
        if self.len == 0 {
            self.anchor = None;
            return Materialization::new(0..0, pins);
        }

        let visible_start = self.index_at(self.resolved_offset);
        let relative = self
            .resolved_offset
            .saturating_sub(self.offset_of(visible_start))
            .clamp(0, i32::MAX as i64) as i32;
        self.anchor = Some(Anchor {
            key: provider.key(visible_start),
            relative,
            fallback_index: visible_start,
        });
        let bottom = self
            .resolved_offset
            .saturating_add(viewport_height.max(0) as i64);
        let visible_end = self.index_at(bottom).saturating_add(1).min(self.len);
        Materialization::new(
            visible_start.saturating_sub(overscan)
                ..visible_end.saturating_add(overscan).min(self.len),
            pins,
        )
    }

    /// Refines only materialized row geometry and returns the explicit
    /// geometry correction that keeps the first visible stable key anchored.
    /// The session decides whether that correction becomes scroll truth.
    pub(crate) fn refine(
        &mut self,
        measurements: impl IntoIterator<Item = (Key, i32)>,
        provider: &dyn Provider,
    ) -> i32 {
        for (key, height) in measurements {
            if provider.index_of(key).is_some() {
                self.measured.insert(key, height.max(1));
            }
        }
        self.reconcile(provider);
        self.restore_anchor(provider)
    }

    /// Width is the measurement-generation token for wrapping content.
    pub(crate) fn set_width(&mut self, width: i32, provider: &dyn Provider) -> i32 {
        let width = width.max(0);
        if self.width == Some(width) {
            return self.resolved_offset();
        }
        self.width = Some(width);
        self.measured.clear();
        self.reconcile(provider);
        self.restore_anchor(provider)
    }

    pub(crate) fn height_for(&self, key: Key) -> i32 {
        self.measured.get(&key).copied().unwrap_or(self.estimate)
    }

    pub(crate) fn estimate(&self) -> i32 {
        self.estimate
    }

    pub(crate) fn index_for_offset(&self, offset: i32) -> usize {
        self.index_at(offset.max(0) as i64)
    }

    pub(crate) fn offset_for_index(&self, index: usize) -> i32 {
        self.offset_of(index).clamp(0, i32::MAX as i64) as i32
    }

    pub(crate) fn content_height(&self) -> i32 {
        self.offset_of(self.len).clamp(0, i32::MAX as i64) as i32
    }

    fn resolved_offset(&self) -> i32 {
        self.resolved_offset.clamp(0, i32::MAX as i64) as i32
    }

    #[cfg(test)]
    fn measured_len(&self) -> usize {
        self.measured.len()
    }

    fn restore_anchor(&mut self, provider: &dyn Provider) -> i32 {
        let Some(anchor) = self.anchor else {
            self.resolved_offset = self.resolved_offset.clamp(0, self.max_offset());
            return self.resolved_offset();
        };
        let index = provider
            .index_of(anchor.key)
            .unwrap_or(anchor.fallback_index.min(self.len.saturating_sub(1)));
        let height = self.height_for(anchor.key);
        let relative = anchor.relative.min(height.saturating_sub(1)).max(0);
        self.resolved_offset = self
            .offset_of(index)
            .saturating_add(relative as i64)
            .clamp(0, self.max_offset());
        self.anchor = (index < self.len).then_some(Anchor {
            key: provider.key(index),
            relative,
            fallback_index: index,
        });
        self.resolved_offset()
    }

    fn rebuild(&mut self, provider: &dyn Provider) {
        self.ordered = self
            .measured
            .iter()
            .filter_map(|(key, height)| {
                provider.index_of(*key).map(|index| Entry {
                    index,
                    key: *key,
                    height: *height,
                })
            })
            .collect();
        self.ordered.sort_unstable_by_key(|entry| entry.index);
        self.prefix_delta.clear();
        let mut total = 0_i64;
        for entry in &self.ordered {
            total = total.saturating_add((entry.height - self.estimate) as i64);
            self.prefix_delta.push(total);
        }
    }

    fn max_offset(&self) -> i64 {
        self.offset_of(self.len).max(0)
    }

    fn offset_of(&self, index: usize) -> i64 {
        let index = index.min(self.len);
        let measured_before = self.ordered.partition_point(|entry| entry.index < index);
        let delta = measured_before
            .checked_sub(1)
            .and_then(|prefix| self.prefix_delta.get(prefix).copied())
            .unwrap_or_default();
        (index as i64)
            .saturating_mul(self.estimate as i64)
            .saturating_add(delta)
    }

    fn index_at(&self, offset: i64) -> usize {
        if self.len == 0 {
            return 0;
        }
        let offset = offset.max(0);
        let mut low = 0_usize;
        let mut high = self.len;
        while low < high {
            let middle = low + (high - low) / 2;
            if self.offset_of(middle.saturating_add(1)) <= offset {
                low = middle.saturating_add(1);
            } else {
                high = middle;
            }
        }
        low.min(self.len.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use crate::view;

    struct Records {
        keys: Vec<Key>,
        lookups: Rc<Cell<usize>>,
    }

    impl Records {
        fn new(keys: impl IntoIterator<Item = u64>) -> Self {
            Self {
                keys: keys.into_iter().map(Key::new).collect(),
                lookups: Rc::new(Cell::new(0)),
            }
        }
    }

    impl Provider for Records {
        fn len(&self) -> usize {
            self.keys.len()
        }

        fn key(&self, index: usize) -> Key {
            self.lookups.set(self.lookups.get() + 1);
            self.keys[index]
        }

        fn index_of(&self, key: Key) -> Option<usize> {
            self.lookups.set(self.lookups.get() + 1);
            self.keys.iter().position(|candidate| *candidate == key)
        }

        fn row(&self, index: usize) -> view::Node {
            view::Node::label(index.to_string())
        }
    }

    #[test]
    fn mixed_height_refinement_returns_an_explicit_anchor_correction() {
        let records = Records::new(0..100);
        let mut region = Region::new(20);
        region.request(410, 100, 2, Vec::new(), &records);
        let anchor = region.anchor.expect("visible anchor");
        assert_eq!(anchor.key, Key::new(20));
        let refined = region.refine(
            [(Key::new(2), 50), (Key::new(20), 35), (Key::new(80), 60)],
            &records,
        );
        assert_eq!(refined, 440);
        assert_eq!(region.offset_for_index(20), 430);
        assert_eq!(region.height_for(Key::new(20)), 35);
        assert_eq!(region.height_for(Key::new(80)), 60);
    }

    #[test]
    fn distant_jump_uses_logarithmic_estimated_lookup_and_bounded_request() {
        let records = Records::new(0..1_000_000);
        let mut region = Region::new(24);
        let request = region.request(20_000_000, 240, 3, Vec::new(), &records);
        assert!(request.range.start > 800_000);
        assert!(request.range.len() <= 17);
        assert!(
            records.lookups.get() < 64,
            "distant lookup must not scan rows"
        );
    }

    #[test]
    fn stable_keys_reconcile_measurements_across_reorder_and_deletion() {
        let mut records = Records::new(0..8);
        let mut region = Region::new(20);
        region.request(40, 60, 1, vec![Key::new(7)], &records);
        region.refine([(Key::new(1), 42), (Key::new(6), 31)], &records);
        records.keys.swap(1, 6);
        region.reconcile(&records);
        assert_eq!(region.offset_for_index(2), 51);
        assert_eq!(region.measured_len(), 2);
        records
            .keys
            .retain(|key| *key != Key::new(6) && *key != Key::new(2));
        region.reconcile(&records);
        assert_eq!(region.measured_len(), 1);
        assert!(region.anchor.is_none());
    }

    #[test]
    fn pins_survive_outside_the_bounded_visible_range_and_dedupe() {
        let records = Records::new(0..10_000);
        let mut region = Region::new(20);
        let request = region.request(
            4_000,
            100,
            2,
            vec![Key::new(2), Key::new(9_000), Key::new(2), Key::new(7_000)],
            &records,
        );
        assert!(request.range.len() <= 10);
        assert_eq!(
            request.pins,
            vec![Key::new(2), Key::new(7_000), Key::new(9_000)]
        );
    }

    #[test]
    fn width_change_invalidates_measurements_and_preserves_visible_anchor() {
        let records = Records::new(0..100);
        let mut region = Region::new(20);
        region.request(210, 80, 1, Vec::new(), &records);
        region.refine([(Key::new(2), 50), (Key::new(10), 40)], &records);
        assert_eq!(region.measured_len(), 2);
        let offset = region.set_width(120, &records);
        assert_eq!(region.measured_len(), 0);
        assert_eq!(offset, 210);
        region.refine([(Key::new(10), 70)], &records);
        assert_eq!(region.set_width(120, &records), 210);
        assert_eq!(region.measured_len(), 1);
    }
}
