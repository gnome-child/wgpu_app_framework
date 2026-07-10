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
    pub(super) fn new(capacity: usize, owner: &str) -> Self {
        Self {
            entries: LruCache::new(
                NonZeroUsize::new(capacity)
                    .unwrap_or_else(|| panic!("{owner} shaping cache capacity must be non-zero")),
            ),
        }
    }

    pub(super) fn get(&mut self, key: &K) -> Option<V> {
        self.entries.get(key).cloned()
    }

    pub(super) fn shape(
        &mut self,
        font_system: &mut FontSystem,
        key: K,
        retain: bool,
        prepare: impl FnOnce(&mut FontSystem, &K) -> Option<V>,
    ) -> Option<Shaped<V>> {
        if retain && let Some(value) = self.get(&key) {
            return Some(Shaped {
                value,
                cache_hit: true,
            });
        }

        let value = prepare(font_system, &key)?;
        if retain {
            self.entries.put(key, value.clone());
        }
        Some(Shaped {
            value,
            cache_hit: false,
        })
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
}
