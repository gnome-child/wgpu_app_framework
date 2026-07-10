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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default)]
pub(super) struct LineIndex {
    root: Option<Arc<Node>>,
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
        let leaves = lines
            .chunks(TARGET_LEAF_LINES)
            .map(|chunk| leaf(Arc::from(chunk)))
            .collect::<Vec<_>>();
        Self {
            root: build_balanced(&leaves),
        }
    }

    pub(super) fn len(&self) -> usize {
        summary(self.root.as_ref()).lines
    }

    pub(super) fn get(&self, line: usize) -> Option<LineMeta> {
        let root = self.root.as_ref()?;
        (line < root.summary.lines)
            .then(|| get(root, line))
            .flatten()
    }

    pub(super) fn find(&self, id: LineId) -> Option<(usize, LineMeta)> {
        self.root.as_ref().and_then(|root| find(root, id, 0))
    }

    pub(super) fn replace(
        &self,
        start: usize,
        old_line_count: usize,
        replacement: Vec<LineMeta>,
    ) -> Self {
        let start = start.min(self.len());
        let end = start.saturating_add(old_line_count).min(self.len());
        let (left, tail) = self.split_at(start);
        let (_, right) = tail.split_at(end - start);
        let middle = Self::from_optional_lines(replacement);
        let joined = Self::concat(Self::concat(left, middle), right);
        if joined.len() == 0 {
            Self::new(1, 0)
        } else {
            joined
        }
    }

    fn from_optional_lines(lines: Vec<LineMeta>) -> Self {
        if lines.is_empty() {
            Self::default()
        } else {
            let leaves = lines
                .chunks(TARGET_LEAF_LINES)
                .map(|chunk| leaf(Arc::from(chunk)))
                .collect::<Vec<_>>();
            Self {
                root: build_balanced(&leaves),
            }
        }
    }

    fn split_at(&self, line: usize) -> (Self, Self) {
        let (left, right) = split_node(self.root.as_ref(), line.min(self.len()));
        (Self { root: left }, Self { root: right })
    }

    fn concat(left: Self, right: Self) -> Self {
        Self {
            root: join(left.root, right.root),
        }
    }

    #[cfg(test)]
    pub(super) fn assert_invariants(&self) {
        if let Some(root) = &self.root {
            validate(root);
        }
    }

    #[cfg(test)]
    pub(super) fn shares_root_with(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }

    #[cfg(test)]
    pub(super) fn shared_leaf_count(&self, other: &Self) -> usize {
        let mut left = Vec::new();
        let mut right = Vec::new();
        collect_leaf_ptrs(self.root.as_ref(), &mut left);
        collect_leaf_ptrs(other.root.as_ref(), &mut right);
        left.iter().filter(|ptr| right.contains(ptr)).count()
    }
}

fn summary(node: Option<&Arc<Node>>) -> Summary {
    node.map(|node| node.summary).unwrap_or_default()
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

fn build_balanced(nodes: &[Arc<Node>]) -> Option<Arc<Node>> {
    match nodes.len() {
        0 => None,
        1 => Some(nodes[0].clone()),
        len => {
            let middle = len / 2;
            Some(branch(
                build_balanced(&nodes[..middle]).expect("left line-index half is non-empty"),
                build_balanced(&nodes[middle..]).expect("right line-index half is non-empty"),
            ))
        }
    }
}

fn get(node: &Arc<Node>, line: usize) -> Option<LineMeta> {
    match &node.kind {
        NodeKind::Leaf(lines) => lines.get(line).copied(),
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
fn collect_leaf_ptrs(node: Option<&Arc<Node>>, leaves: &mut Vec<*const Node>) {
    let Some(node) = node else {
        return;
    };
    match &node.kind {
        NodeKind::Leaf(_) => leaves.push(Arc::as_ptr(node)),
        NodeKind::Branch { left, right } => {
            collect_leaf_ptrs(Some(left), leaves);
            collect_leaf_ptrs(Some(right), leaves);
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
}
