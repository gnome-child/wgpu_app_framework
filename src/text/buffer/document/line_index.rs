use std::sync::Arc;

use super::super::{LineId, LineLayoutIdentity, next_line_id};

const TARGET_LEAF_LINES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LineMeta {
    pub(super) id: LineId,
    pub(super) revision: u64,
}

impl LineMeta {
    pub(super) fn new(revision: u64) -> Self {
        Self {
            id: next_line_id(),
            revision,
        }
    }

    pub(super) fn identity(self) -> LineLayoutIdentity {
        LineLayoutIdentity {
            id: self.id,
            revision: self.revision,
        }
    }

    pub(super) fn with_revision(self, revision: u64) -> Self {
        Self { revision, ..self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Summary {
    lines: usize,
    height: u8,
}

#[derive(Debug)]
enum NodeKind {
    Leaf(Arc<[LineMeta]>),
    Branch { left: Arc<Node>, right: Arc<Node> },
}

#[derive(Debug)]
struct Node {
    summary: Summary,
    kind: NodeKind,
}

#[derive(Debug, Clone)]
pub(super) struct LineIndex {
    root: Arc<Node>,
}

impl LineIndex {
    pub(super) fn new(line_count: usize, revision: u64) -> Self {
        let lines = (0..line_count.max(1))
            .map(|_| LineMeta::new(revision))
            .collect::<Vec<_>>();
        Self::from_lines(lines)
    }

    pub(super) fn from_lines(lines: Vec<LineMeta>) -> Self {
        let lines = if lines.is_empty() {
            vec![LineMeta::new(0)]
        } else {
            lines
        };
        let first_end = lines.len().min(TARGET_LEAF_LINES);
        let root = lines[first_end..]
            .chunks(TARGET_LEAF_LINES)
            .fold(leaf(Arc::from(&lines[..first_end])), |root, chunk| {
                join_nodes(root, leaf(Arc::from(chunk)))
            });
        Self { root }
    }

    pub(super) fn len(&self) -> usize {
        self.root.summary.lines
    }

    pub(super) fn get(&self, line: usize) -> Option<LineMeta> {
        (line < self.root.summary.lines).then(|| get(&self.root, line))
    }

    pub(super) fn get_clamped(&self, line: usize) -> LineMeta {
        get(&self.root, line.min(self.root.summary.lines - 1))
    }

    pub(super) fn find(&self, id: LineId) -> Option<(usize, LineMeta)> {
        find(&self.root, id, 0)
    }

    pub(super) fn replace(
        &self,
        start: usize,
        old_line_count: usize,
        replacement: Vec<LineMeta>,
    ) -> Self {
        let start = start.min(self.len());
        let end = start.saturating_add(old_line_count).min(self.len());
        let (left, tail) = split_node(Some(&self.root), start);
        let (_, right) = split_node(tail.as_ref(), end - start);
        let middle = tree_for_lines(&replacement);
        let root =
            join(join(left, middle), right).unwrap_or_else(|| leaf(Arc::from([LineMeta::new(0)])));
        Self { root }
    }

    #[cfg(test)]
    pub(super) fn assert_invariants(&self) {
        validate(&self.root);
    }

    #[cfg(test)]
    pub(super) fn shares_root_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.root, &other.root)
    }

    #[cfg(test)]
    pub(super) fn shared_leaf_count(&self, other: &Self) -> usize {
        let mut left = Vec::new();
        let mut right = Vec::new();
        collect_leaf_ptrs(&self.root, &mut left);
        collect_leaf_ptrs(&other.root, &mut right);
        left.iter().filter(|ptr| right.contains(ptr)).count()
    }
}

fn leaf(lines: Arc<[LineMeta]>) -> Arc<Node> {
    debug_assert!(!lines.is_empty());
    let node = Arc::new(Node {
        summary: Summary {
            lines: lines.len(),
            height: 1,
        },
        kind: NodeKind::Leaf(lines),
    });
    debug_assert_local_summary(&node);
    node
}

fn branch(left: Arc<Node>, right: Arc<Node>) -> Arc<Node> {
    debug_assert!(left.summary.height.abs_diff(right.summary.height) <= 1);
    let node = Arc::new(Node {
        summary: Summary {
            lines: left.summary.lines + right.summary.lines,
            height: left.summary.height.max(right.summary.height) + 1,
        },
        kind: NodeKind::Branch { left, right },
    });
    debug_assert_local_summary(&node);
    node
}

fn debug_assert_local_summary(_node: &Arc<Node>) {
    #[cfg(debug_assertions)]
    match &_node.kind {
        NodeKind::Leaf(lines) => {
            debug_assert_eq!(_node.summary.lines, lines.len());
            debug_assert_eq!(_node.summary.height, 1);
        }
        NodeKind::Branch { left, right } => {
            debug_assert_eq!(
                _node.summary.lines,
                left.summary.lines + right.summary.lines
            );
            debug_assert_eq!(
                _node.summary.height,
                left.summary.height.max(right.summary.height) + 1
            );
        }
    }
}

fn tree_for_lines(lines: &[LineMeta]) -> Option<Arc<Node>> {
    lines
        .chunks(TARGET_LEAF_LINES)
        .map(|chunk| leaf(Arc::from(chunk)))
        .reduce(join_nodes)
}

fn get(node: &Arc<Node>, line: usize) -> LineMeta {
    match &node.kind {
        NodeKind::Leaf(lines) => lines[line],
        NodeKind::Branch { left, right } => {
            if line < left.summary.lines {
                get(left, line)
            } else {
                get(right, line - left.summary.lines)
            }
        }
    }
}

fn find(node: &Arc<Node>, id: LineId, base: usize) -> Option<(usize, LineMeta)> {
    match &node.kind {
        NodeKind::Leaf(lines) => lines
            .iter()
            .copied()
            .enumerate()
            .find(|(_, line)| line.id == id)
            .map(|(index, line)| (base + index, line)),
        NodeKind::Branch { left, right } => {
            find(left, id, base).or_else(|| find(right, id, base + left.summary.lines))
        }
    }
}

fn join(left: Option<Arc<Node>>, right: Option<Arc<Node>>) -> Option<Arc<Node>> {
    match (left, right) {
        (None, right) => right,
        (left, None) => left,
        (Some(left), Some(right)) => Some(join_nodes(left, right)),
    }
}

fn join_nodes(left: Arc<Node>, right: Arc<Node>) -> Arc<Node> {
    if left.summary.height > right.summary.height.saturating_add(1)
        && let NodeKind::Branch {
            left: left_left,
            right: left_right,
        } = &left.kind
    {
        return balance(left_left.clone(), join_nodes(left_right.clone(), right));
    }
    if right.summary.height > left.summary.height.saturating_add(1)
        && let NodeKind::Branch {
            left: right_left,
            right: right_right,
        } = &right.kind
    {
        return balance(join_nodes(left, right_left.clone()), right_right.clone());
    }
    branch(left, right)
}

fn balance(left: Arc<Node>, right: Arc<Node>) -> Arc<Node> {
    if left.summary.height > right.summary.height.saturating_add(1)
        && let NodeKind::Branch {
            left: left_left,
            right: left_right,
        } = &left.kind
    {
        if left_left.summary.height >= left_right.summary.height {
            return branch(left_left.clone(), branch(left_right.clone(), right));
        }
        if let NodeKind::Branch {
            left: middle_left,
            right: middle_right,
        } = &left_right.kind
        {
            return branch(
                branch(left_left.clone(), middle_left.clone()),
                branch(middle_right.clone(), right),
            );
        }
    }
    if right.summary.height > left.summary.height.saturating_add(1)
        && let NodeKind::Branch {
            left: right_left,
            right: right_right,
        } = &right.kind
    {
        if right_right.summary.height >= right_left.summary.height {
            return branch(branch(left, right_left.clone()), right_right.clone());
        }
        if let NodeKind::Branch {
            left: middle_left,
            right: middle_right,
        } = &right_left.kind
        {
            return branch(
                branch(left, middle_left.clone()),
                branch(middle_right.clone(), right_right.clone()),
            );
        }
    }
    branch(left, right)
}

fn split_node(node: Option<&Arc<Node>>, line: usize) -> (Option<Arc<Node>>, Option<Arc<Node>>) {
    let Some(node) = node else {
        return (None, None);
    };
    if line == 0 {
        return (None, Some(node.clone()));
    }
    if line >= node.summary.lines {
        return (Some(node.clone()), None);
    }

    match &node.kind {
        NodeKind::Leaf(lines) => (
            Some(leaf(Arc::from(&lines[..line]))),
            Some(leaf(Arc::from(&lines[line..]))),
        ),
        NodeKind::Branch { left, right } => {
            if line < left.summary.lines {
                let (before, after) = split_node(Some(left), line);
                (before, join(after, Some(right.clone())))
            } else if line == left.summary.lines {
                (Some(left.clone()), Some(right.clone()))
            } else {
                let (before, after) = split_node(Some(right), line - left.summary.lines);
                (join(Some(left.clone()), before), after)
            }
        }
    }
}

#[cfg(test)]
fn validate(node: &Arc<Node>) -> Summary {
    let actual = match &node.kind {
        NodeKind::Leaf(lines) => {
            debug_assert!(!lines.is_empty());
            Summary {
                lines: lines.len(),
                height: 1,
            }
        }
        NodeKind::Branch { left, right } => {
            let left = validate(left);
            let right = validate(right);
            debug_assert!(left.height.abs_diff(right.height) <= 1);
            Summary {
                lines: left.lines + right.lines,
                height: left.height.max(right.height) + 1,
            }
        }
    };
    debug_assert_eq!(node.summary, actual);
    actual
}

#[cfg(test)]
fn collect_leaf_ptrs(node: &Arc<Node>, leaves: &mut Vec<*const Node>) {
    match &node.kind {
        NodeKind::Leaf(_) => leaves.push(Arc::as_ptr(node)),
        NodeKind::Branch { left, right } => {
            collect_leaf_ptrs(left, leaves);
            collect_leaf_ptrs(right, leaves);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_index_clone_and_edit_path_share_untouched_metadata() {
        let index = LineIndex::new(400, 0);
        let clone = index.clone();
        assert!(index.shares_root_with(&clone));

        let original = index.get(5).unwrap();
        let edited = clone.replace(5, 1, vec![original.with_revision(1)]);

        assert_eq!(edited.get(5).unwrap().id, original.id);
        assert_eq!(edited.get(5).unwrap().revision, 1);
        assert_eq!(edited.get(300), index.get(300));
        assert!(index.shared_leaf_count(&edited) >= 2);
        edited.assert_invariants();
    }

    #[test]
    fn line_index_replace_preserves_only_explicit_line_identities() {
        let index = LineIndex::new(3, 0);
        let first = index.get(0).unwrap();
        let third = index.get(2).unwrap();
        let inserted = vec![first.with_revision(1), LineMeta::new(1)];
        let edited = index.replace(0, 2, inserted);

        assert_eq!(edited.len(), 3);
        assert_eq!(edited.get(0).unwrap().id, first.id);
        assert_eq!(edited.get(2).unwrap(), third);
        assert_eq!(edited.find(third.id), Some((2, third)));
        edited.assert_invariants();
    }

    #[test]
    fn line_index_remains_nonempty_when_every_line_is_removed() {
        let index = LineIndex::new(3, 0).replace(0, 3, Vec::new());

        assert_eq!(index.len(), 1);
        assert_eq!(index.get_clamped(usize::MAX), index.get(0).unwrap());
        index.assert_invariants();
    }
}
