//! Small immutable collections used by retained presentation products.
//!
//! These collections are deliberately private. They carry structural sharing
//! between immutable generations; they do not own invalidation, identity, or
//! scheduling. `Sequence` is an implicit treap suited to residency edits at
//! either end, while `RadixMap` is a hash-radix trie with a fixed-depth update
//! path. An edit therefore allocates only the changed path and never copies the
//! resident population.

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Weak},
};

const RADIX_BITS: usize = 4;
const RADIX: usize = 1 << RADIX_BITS;
const RADIX_MASK: u64 = (RADIX as u64) - 1;
const RADIX_LEVELS: usize = 64 / RADIX_BITS;

/// Deterministic heap priority for a semantic sequence key. The priority is a
/// shape currency only; semantic identity remains owned by the caller.
pub(crate) fn sequence_priority(key: u64) -> u64 {
    let mut value = key.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Clone, Default)]
pub(crate) struct Sequence<T> {
    root: Option<Arc<SequenceNode<T>>>,
}

pub(crate) struct SequenceNode<T> {
    value: T,
    priority: u64,
    left: Option<Arc<SequenceNode<T>>>,
    right: Option<Arc<SequenceNode<T>>>,
    len: usize,
}

pub(crate) struct SequenceIter<'a, T> {
    stack: Vec<&'a SequenceNode<T>>,
    next: Option<&'a SequenceNode<T>>,
}

pub(crate) struct SequenceMapCache<T, U> {
    nodes: HashMap<usize, (Weak<SequenceNode<T>>, Weak<SequenceNode<U>>)>,
}

impl<T, U> Default for SequenceMapCache<T, U> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
}

#[allow(
    dead_code,
    reason = "PC-005 integrates the persistent collections into scene and renderer in the next boundary"
)]
impl<T: Clone> Sequence<T> {
    pub(crate) fn from_entries(entries: impl IntoIterator<Item = (u64, T)>) -> Self {
        let root = entries.into_iter().fold(None, |root, (priority, value)| {
            merge(root, Some(sequence_node(value, priority, None, None)))
        });
        Self { root }
    }

    pub(crate) fn from_root(root: Option<Arc<SequenceNode<T>>>) -> Self {
        Self { root }
    }

    pub(crate) fn root(&self) -> Option<&Arc<SequenceNode<T>>> {
        self.root.as_ref()
    }

    pub(crate) fn len(&self) -> usize {
        sequence_len(&self.root)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub(crate) fn get(&self, mut index: usize) -> Option<&T> {
        let mut node = self.root.as_deref()?;
        loop {
            let left = sequence_len(&node.left);
            if index < left {
                node = node.left.as_deref()?;
            } else if index == left {
                return Some(&node.value);
            } else {
                index = index.saturating_sub(left + 1);
                node = node.right.as_deref()?;
            }
        }
    }

    pub(crate) fn iter(&self) -> SequenceIter<'_, T> {
        SequenceIter {
            stack: Vec::new(),
            next: self.root.as_deref(),
        }
    }

    /// Applies the typed virtual-residency edit without visiting the retained
    /// middle. `front` and `back` must already be in final presentation order.
    pub(crate) fn edit_ends(
        &self,
        remove_front: usize,
        remove_back: usize,
        front: impl IntoIterator<Item = (u64, T)>,
        back: impl IntoIterator<Item = (u64, T)>,
    ) -> Self {
        let (_, after_front) = split(self.root.clone(), remove_front.min(self.len()));
        let keep = sequence_len(&after_front).saturating_sub(remove_back);
        let (middle, _) = split(after_front, keep);
        let front = Self::from_entries(front).root;
        let back = Self::from_entries(back).root;
        Self {
            root: merge(merge(front, middle), back),
        }
    }

    /// Maps one immutable sequence generation while memoizing source subtree
    /// identity. An unchanged subtree is returned by one weak-cache lookup;
    /// only newly allocated source paths invoke `map_value` and rebuild target
    /// paths.
    pub(crate) fn map_reusing<U: Clone>(
        &self,
        cache: &mut SequenceMapCache<T, U>,
        mut map_value: impl FnMut(&T) -> U,
    ) -> Sequence<U> {
        cache
            .nodes
            .retain(|_, (source, target)| source.strong_count() > 0 && target.strong_count() > 0);
        Sequence::from_root(map_sequence_node(self.root.as_ref(), cache, &mut map_value))
    }
}

fn map_sequence_node<T: Clone, U: Clone>(
    source: Option<&Arc<SequenceNode<T>>>,
    cache: &mut SequenceMapCache<T, U>,
    map_value: &mut impl FnMut(&T) -> U,
) -> Option<Arc<SequenceNode<U>>> {
    let source = source?;
    let key = Arc::as_ptr(source) as usize;
    if let Some((cached_source, cached_target)) = cache.nodes.get(&key)
        && let (Some(cached_source), Some(cached_target)) =
            (cached_source.upgrade(), cached_target.upgrade())
        && Arc::ptr_eq(&cached_source, source)
    {
        return Some(cached_target);
    }
    let left = map_sequence_node(source.left.as_ref(), cache, map_value);
    let right = map_sequence_node(source.right.as_ref(), cache, map_value);
    let target = sequence_node(map_value(&source.value), source.priority, left, right);
    cache
        .nodes
        .insert(key, (Arc::downgrade(source), Arc::downgrade(&target)));
    Some(target)
}

#[allow(
    dead_code,
    reason = "PC-005 renderer memoization consumes the persistent node accessors"
)]
impl<T> SequenceNode<T> {
    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn priority(&self) -> u64 {
        self.priority
    }

    pub(crate) fn left(&self) -> Option<&Arc<Self>> {
        self.left.as_ref()
    }

    pub(crate) fn right(&self) -> Option<&Arc<Self>> {
        self.right.as_ref()
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> Iterator for SequenceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.next.take() {
            self.stack.push(node);
            self.next = node.left.as_deref();
        }
        let node = self.stack.pop()?;
        self.next = node.right.as_deref();
        Some(&node.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

fn sequence_len<T>(node: &Option<Arc<SequenceNode<T>>>) -> usize {
    node.as_ref().map_or(0, |node| node.len)
}

pub(crate) fn sequence_node<T>(
    value: T,
    priority: u64,
    left: Option<Arc<SequenceNode<T>>>,
    right: Option<Arc<SequenceNode<T>>>,
) -> Arc<SequenceNode<T>> {
    Arc::new(SequenceNode {
        len: 1 + sequence_len(&left) + sequence_len(&right),
        value,
        priority,
        left,
        right,
    })
}

fn merge<T: Clone>(
    left: Option<Arc<SequenceNode<T>>>,
    right: Option<Arc<SequenceNode<T>>>,
) -> Option<Arc<SequenceNode<T>>> {
    match (left, right) {
        (None, right) => right,
        (left, None) => left,
        (Some(left), Some(right)) if left.priority >= right.priority => {
            let merged = merge(left.right.clone(), Some(right));
            Some(sequence_node(
                left.value.clone(),
                left.priority,
                left.left.clone(),
                merged,
            ))
        }
        (Some(left), Some(right)) => {
            let merged = merge(Some(left), right.left.clone());
            Some(sequence_node(
                right.value.clone(),
                right.priority,
                merged,
                right.right.clone(),
            ))
        }
    }
}

fn split<T: Clone>(
    root: Option<Arc<SequenceNode<T>>>,
    count: usize,
) -> (Option<Arc<SequenceNode<T>>>, Option<Arc<SequenceNode<T>>>) {
    let Some(root) = root else {
        return (None, None);
    };
    let left_len = sequence_len(&root.left);
    if count <= left_len {
        let (left, middle) = split(root.left.clone(), count);
        let right = Some(sequence_node(
            root.value.clone(),
            root.priority,
            middle,
            root.right.clone(),
        ));
        (left, right)
    } else {
        let (middle, right) = split(root.right.clone(), count.saturating_sub(left_len + 1));
        let left = Some(sequence_node(
            root.value.clone(),
            root.priority,
            root.left.clone(),
            middle,
        ));
        (left, right)
    }
}

#[derive(Clone, Default)]
pub(crate) struct RadixMap<K, V> {
    root: Option<Arc<RadixNode<K, V>>>,
    len: usize,
}

enum RadixNode<K, V> {
    Branch(Box<[Option<Arc<RadixNode<K, V>>>; RADIX]>),
    Bucket(Arc<[(K, V)]>),
}

#[allow(
    dead_code,
    reason = "PC-005 scene indices consume the persistent map incrementally"
)]
impl<K, V> RadixMap<K, V>
where
    K: Copy + Eq + Hash,
    V: Clone,
{
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn get(&self, key: K) -> Option<&V> {
        let hash = persistent_hash(&key);
        radix_get(self.root.as_deref()?, 0, hash, key)
    }

    pub(crate) fn contains_key(&self, key: K) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn insert(&self, key: K, value: V) -> (Self, bool) {
        let hash = persistent_hash(&key);
        let (root, inserted) = radix_insert(self.root.clone(), 0, hash, key, value);
        (
            Self {
                root: Some(root),
                len: self.len.saturating_add(usize::from(inserted)),
            },
            inserted,
        )
    }

    pub(crate) fn remove(&self, key: K) -> (Self, bool) {
        let hash = persistent_hash(&key);
        let (root, removed) = radix_remove(self.root.clone(), 0, hash, key);
        (
            Self {
                root,
                len: self.len.saturating_sub(usize::from(removed)),
            },
            removed,
        )
    }

    #[cfg(test)]
    fn root(&self) -> Option<&Arc<RadixNode<K, V>>> {
        self.root.as_ref()
    }
}

fn persistent_hash<K: Hash>(key: &K) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

fn radix_slot(hash: u64, level: usize) -> usize {
    ((hash >> (level * RADIX_BITS)) & RADIX_MASK) as usize
}

fn empty_children<K, V>() -> Box<[Option<Arc<RadixNode<K, V>>>; RADIX]> {
    Box::new(std::array::from_fn(|_| None))
}

fn radix_get<K: Copy + Eq, V>(
    node: &RadixNode<K, V>,
    level: usize,
    hash: u64,
    key: K,
) -> Option<&V> {
    match node {
        RadixNode::Bucket(entries) => entries
            .iter()
            .find_map(|(candidate, value)| (*candidate == key).then_some(value)),
        RadixNode::Branch(children) => {
            let child = children.get(radix_slot(hash, level))?.as_deref()?;
            radix_get(child, level + 1, hash, key)
        }
    }
}

fn radix_insert<K, V>(
    node: Option<Arc<RadixNode<K, V>>>,
    level: usize,
    hash: u64,
    key: K,
    value: V,
) -> (Arc<RadixNode<K, V>>, bool)
where
    K: Copy + Eq,
    V: Clone,
{
    if level == RADIX_LEVELS {
        let mut entries = match node.as_deref() {
            Some(RadixNode::Bucket(entries)) => entries.to_vec(),
            Some(RadixNode::Branch(_)) => unreachable!("terminal radix level cannot branch"),
            None => Vec::new(),
        };
        if let Some((_, existing)) = entries.iter_mut().find(|(candidate, _)| *candidate == key) {
            *existing = value;
            return (Arc::new(RadixNode::Bucket(entries.into())), false);
        }
        entries.push((key, value));
        return (Arc::new(RadixNode::Bucket(entries.into())), true);
    }

    let mut children = match node.as_deref() {
        Some(RadixNode::Branch(children)) => children.clone(),
        Some(RadixNode::Bucket(_)) => unreachable!("nonterminal radix level cannot be a bucket"),
        None => empty_children(),
    };
    let slot = radix_slot(hash, level);
    let (child, inserted) = radix_insert(children[slot].clone(), level + 1, hash, key, value);
    children[slot] = Some(child);
    (Arc::new(RadixNode::Branch(children)), inserted)
}

fn radix_remove<K, V>(
    node: Option<Arc<RadixNode<K, V>>>,
    level: usize,
    hash: u64,
    key: K,
) -> (Option<Arc<RadixNode<K, V>>>, bool)
where
    K: Copy + Eq,
    V: Clone,
{
    let Some(node) = node else {
        return (None, false);
    };
    if level == RADIX_LEVELS {
        let RadixNode::Bucket(entries) = node.as_ref() else {
            unreachable!("terminal radix level must contain a bucket");
        };
        let Some(index) = entries.iter().position(|(candidate, _)| *candidate == key) else {
            return (Some(node), false);
        };
        let mut entries = entries.to_vec();
        entries.remove(index);
        return if entries.is_empty() {
            (None, true)
        } else {
            (Some(Arc::new(RadixNode::Bucket(entries.into()))), true)
        };
    }

    let RadixNode::Branch(existing) = node.as_ref() else {
        unreachable!("nonterminal radix level must contain a branch");
    };
    let slot = radix_slot(hash, level);
    let (child, removed) = radix_remove(existing[slot].clone(), level + 1, hash, key);
    if !removed {
        return (Some(node), false);
    }
    let mut children = existing.clone();
    children[slot] = child;
    if children.iter().all(Option::is_none) {
        (None, true)
    } else {
        (Some(Arc::new(RadixNode::Branch(children))), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn priorities(count: usize) -> impl Iterator<Item = (u64, usize)> {
        (0..count).map(|value| (sequence_priority(value as u64), value))
    }

    fn nodes<T>(root: Option<&Arc<SequenceNode<T>>>, target: &mut Vec<usize>) {
        let Some(root) = root else {
            return;
        };
        target.push(Arc::as_ptr(root) as usize);
        nodes(root.left(), target);
        nodes(root.right(), target);
    }

    #[test]
    fn end_edit_preserves_the_retained_middle_as_shared_subtrees() {
        let initial = Sequence::from_entries(priorities(256));
        let next = initial.edit_ends(1, 0, [], priorities(257).skip(256));
        assert_eq!(next.len(), 256);
        assert_eq!(
            next.iter().copied().collect::<Vec<_>>(),
            (1..257).collect::<Vec<_>>()
        );

        let mut before = Vec::new();
        let mut after = Vec::new();
        nodes(initial.root(), &mut before);
        nodes(next.root(), &mut after);
        let shared = before.iter().filter(|node| after.contains(node)).count();
        let rebuilt = before.len().saturating_sub(shared);
        assert!(
            rebuilt <= 32,
            "a one-row edit rebuilt more than a logarithmic path: shared={shared} rebuilt={rebuilt}"
        );
    }

    #[test]
    fn radix_updates_preserve_unrelated_branches_and_exact_membership() {
        let initial = (0_u64..256).fold(RadixMap::default(), |map, key| map.insert(key, key * 2).0);
        let root = Arc::clone(initial.root().expect("populated radix root"));
        let (inserted, added) = initial.insert(10_000, 7);
        assert!(added);
        assert_eq!(inserted.len(), 257);
        assert_eq!(inserted.get(10_000), Some(&7));
        assert_eq!(inserted.get(100), Some(&200));
        assert!(!Arc::ptr_eq(
            &root,
            inserted.root().expect("updated radix root")
        ));

        let (removed, did_remove) = inserted.remove(10_000);
        assert!(did_remove);
        assert_eq!(removed.len(), 256);
        assert!(!removed.contains_key(10_000));
        for key in 0_u64..256 {
            assert_eq!(removed.get(key), Some(&(key * 2)));
        }
    }

    #[test]
    fn mapped_sequence_reuses_unchanged_target_subtrees() {
        let initial = Sequence::from_entries(priorities(256));
        let mut cache = SequenceMapCache::default();
        let mut mapped_values = 0_usize;
        let first = initial.map_reusing(&mut cache, |value| {
            mapped_values += 1;
            value * 2
        });
        assert_eq!(mapped_values, 256);

        let next = initial.edit_ends(1, 0, [], priorities(257).skip(256));
        mapped_values = 0;
        let second = next.map_reusing(&mut cache, |value| {
            mapped_values += 1;
            value * 2
        });
        assert!(
            mapped_values <= 32,
            "one end edit remapped resident overlap: mapped={mapped_values}"
        );
        assert_eq!(
            second.iter().copied().collect::<Vec<_>>(),
            (1..257).map(|value| value * 2).collect::<Vec<_>>()
        );
        assert_eq!(first.len(), second.len());
    }
}
