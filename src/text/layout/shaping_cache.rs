use std::hash::Hash;
use std::num::NonZeroUsize;

use glyphon::FontSystem;
use lru::LruCache;

pub(in crate::text) struct ShapingCache<K, V> {
    entries: LruCache<K, V>,
}

pub(super) struct Shaped<V> {
    pub(super) value: V,
    pub(super) cache_hit: bool,
}

impl<K, V> ShapingCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub(super) fn new(capacity: NonZeroUsize) -> Self {
        Self {
            entries: LruCache::new(capacity),
        }
    }

    pub(super) fn get(&mut self, key: &K) -> Option<V> {
        self.entries.get(key).cloned()
    }

    pub(super) fn shape_required(
        &mut self,
        font_system: &mut FontSystem,
        key: K,
        retain: bool,
        prepare: impl FnOnce(&mut FontSystem, &K) -> V,
    ) -> Shaped<V> {
        if let Some(shaped) = self.cached(&key, retain) {
            return shaped;
        }

        let value = prepare(font_system, &key);
        self.admit(key, value, retain)
    }

    pub(super) fn shape_optional(
        &mut self,
        font_system: &mut FontSystem,
        key: K,
        retain: bool,
        prepare: impl FnOnce(&mut FontSystem, &K) -> Option<V>,
    ) -> Option<Shaped<V>> {
        if let Some(shaped) = self.cached(&key, retain) {
            return Some(shaped);
        }

        let value = prepare(font_system, &key)?;
        Some(self.admit(key, value, retain))
    }

    fn cached(&mut self, key: &K, retain: bool) -> Option<Shaped<V>> {
        if !retain {
            return None;
        }
        self.get(key).map(|value| Shaped {
            value,
            cache_hit: true,
        })
    }

    fn admit(&mut self, key: K, value: V, retain: bool) -> Shaped<V> {
        if retain {
            self.entries.put(key, value.clone());
        }
        Shaped {
            value,
            cache_hit: false,
        }
    }

    #[cfg(test)]
    pub(in crate::text) fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(in crate::text) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::ShapingCache;

    #[test]
    fn shaped_buffer_caches_share_one_mechanics_owner() {
        for (name, source) in [
            ("area line", include_str!("text_area.rs")),
            ("field surface", include_str!("field.rs")),
            ("inline", include_str!("inline.rs")),
        ] {
            assert!(
                source.contains("ShapingCache"),
                "{name} cache must use the text-layout shaping cache owner"
            );
        }
        assert!(!include_str!("text_area.rs").contains("LruCache<LineDisplayKey"));
        assert!(!include_str!("field.rs").contains("use lru::LruCache"));
        assert!(!include_str!("inline.rs").contains("use lru::LruCache"));
    }

    #[test]
    fn shaping_cardinality_and_cache_admission_are_structural() {
        let mut font_system = glyphon::FontSystem::new();
        let mut cache = ShapingCache::new(NonZeroUsize::new(2).unwrap());

        let first = cache.shape_required(&mut font_system, "required", true, |_, _| 1);
        assert_eq!(first.value, 1);
        assert!(!first.cache_hit);
        let cached = cache.shape_required(&mut font_system, "required", true, |_, _| 2);
        assert_eq!(cached.value, 1);
        assert!(cached.cache_hit);

        assert!(
            cache
                .shape_optional(&mut font_system, "optional", true, |_, _| None)
                .is_none()
        );
        assert_eq!(cache.len(), 1, "absence must not enter the cache");
        let optional = cache
            .shape_optional(&mut font_system, "optional", true, |_, _| Some(3))
            .expect("test preparation supplies a value");
        assert_eq!(optional.value, 3);
        assert!(!optional.cache_hit);
        let cached = cache
            .shape_optional(&mut font_system, "optional", true, |_, _| Some(4))
            .expect("cached optional preparation remains present");
        assert_eq!(cached.value, 3);
        assert!(cached.cache_hit);
    }
}
