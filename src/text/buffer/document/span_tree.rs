use std::{
    io::{self, Write},
    ops::Range,
    sync::Arc,
};

pub(in crate::text) const TARGET_LEAF_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceKind {
    Original,
    Add,
}

#[derive(Debug)]
struct SourceBuffer {
    #[cfg_attr(not(test), allow(dead_code))]
    kind: SourceKind,
    text: Arc<str>,
}

#[derive(Debug, Clone)]
struct SourceSpan {
    source: Arc<SourceBuffer>,
    start: usize,
    len: usize,
}

impl SourceSpan {
    fn text(&self) -> &str {
        &self.source.text[self.start..self.start + self.len]
    }

    fn slice(&self, range: Range<usize>) -> Self {
        debug_assert!(range.start <= range.end);
        debug_assert!(range.end <= self.len);
        Self {
            source: self.source.clone(),
            start: self.start + range.start,
            len: range.end - range.start,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Summary {
    bytes: usize,
    newlines: usize,
    height: u8,
}

#[derive(Debug)]
enum NodeKind {
    Leaf {
        span: SourceSpan,
        newline_offsets: Arc<[u32]>,
    },
    Branch {
        left: Arc<Node>,
        right: Arc<Node>,
    },
}

#[derive(Debug)]
struct Node {
    summary: Summary,
    kind: NodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LineBounds {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) ending_len: usize,
}

#[derive(Debug, Clone, Default)]
pub(super) struct SpanTree {
    root: Option<Arc<Node>>,
}

impl SpanTree {
    pub(super) fn from_original(text: Arc<str>) -> Self {
        Self::from_source(SourceKind::Original, text)
    }

    pub(super) fn from_addition(text: Arc<str>) -> Self {
        Self::from_source(SourceKind::Add, text)
    }

    fn from_source(kind: SourceKind, text: Arc<str>) -> Self {
        if text.is_empty() {
            return Self::default();
        }

        let source = Arc::new(SourceBuffer { kind, text });
        let mut leaves = Vec::new();
        let mut start = 0;
        while start < source.text.len() {
            let mut end = (start + TARGET_LEAF_BYTES).min(source.text.len());
            while end > start && !source.text.is_char_boundary(end) {
                end -= 1;
            }
            if end == start {
                end = (start + 1).min(source.text.len());
                while end < source.text.len() && !source.text.is_char_boundary(end) {
                    end += 1;
                }
            }
            leaves.push(leaf(SourceSpan {
                source: source.clone(),
                start,
                len: end - start,
            }));
            start = end;
        }

        Self {
            root: build_balanced(&leaves),
        }
    }

    pub(super) fn len(&self) -> usize {
        summary(self.root.as_ref()).bytes
    }

    pub(super) fn line_count(&self) -> usize {
        summary(self.root.as_ref()).newlines.saturating_add(1)
    }

    pub(super) fn line_start(&self, line: usize) -> usize {
        let line = line.min(self.line_count().saturating_sub(1));
        if line == 0 {
            0
        } else {
            self.root
                .as_ref()
                .and_then(|root| nth_newline(root, line - 1, 0))
                .map(|offset| offset + 1)
                .unwrap_or(self.len())
        }
    }

    pub(super) fn line_bounds(&self, line: usize) -> LineBounds {
        let line = line.min(self.line_count().saturating_sub(1));
        let start = self.line_start(line);
        let Some(newline) = self
            .root
            .as_ref()
            .and_then(|root| nth_newline(root, line, 0))
        else {
            return LineBounds {
                start,
                end: self.len(),
                ending_len: 0,
            };
        };
        let crlf = newline > start && self.byte_at(newline - 1) == Some(b'\r');
        LineBounds {
            start,
            end: newline - usize::from(crlf),
            ending_len: if crlf { 2 } else { 1 },
        }
    }

    pub(super) fn line_and_local_for_index(&self, index: usize) -> (usize, usize) {
        let index = index.min(self.len());
        let line = self
            .root
            .as_ref()
            .map(|root| count_newlines_before(root, index))
            .unwrap_or(0)
            .min(self.line_count().saturating_sub(1));
        let bounds = self.line_bounds(line);
        (
            line,
            index
                .saturating_sub(bounds.start)
                .min(bounds.end - bounds.start),
        )
    }

    pub(super) fn line_starts(&self) -> Vec<usize> {
        let mut starts = Vec::with_capacity(self.line_count());
        starts.push(0);
        let mut base = 0;
        self.for_each_leaf(&mut |span, newline_offsets| {
            starts.extend(
                newline_offsets
                    .iter()
                    .map(|offset| base + *offset as usize + 1),
            );
            base += span.len;
            Ok(())
        })
        .expect("collecting line starts cannot fail");
        starts
    }

    pub(super) fn text(&self) -> String {
        let mut text = String::with_capacity(self.len());
        self.for_each_span(|span| text.push_str(span));
        text
    }

    pub(super) fn text_for_range(&self, range: Range<usize>) -> String {
        let start = range.start.min(self.len());
        let end = range.end.max(start).min(self.len());
        let (left, tail) = self.split_at(start);
        let _ = left;
        let (middle, _) = tail.split_at(end - start);
        middle.text()
    }

    pub(super) fn replace(&self, range: Range<usize>, inserted: Self) -> Self {
        let start = range.start.min(self.len());
        let end = range.end.max(start).min(self.len());
        let (left, tail) = self.split_at(start);
        let (_, right) = tail.split_at(end - start);
        Self::concat(Self::concat(left, inserted), right)
    }

    pub(super) fn split_at(&self, index: usize) -> (Self, Self) {
        let index = index.min(self.len());
        let (left, right) = split_node(self.root.as_ref(), index);
        (Self { root: left }, Self { root: right })
    }

    pub(super) fn concat(left: Self, right: Self) -> Self {
        Self {
            root: join(left.root, right.root),
        }
    }

    pub(super) fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.for_each_leaf(&mut |span, _| writer.write_all(span.text().as_bytes()))
    }

    #[cfg(test)]
    pub(super) fn source_lengths(&self) -> (usize, usize) {
        let mut original = 0usize;
        let mut add = 0usize;
        self.for_each_leaf(&mut |span, _| {
            match span.source.kind {
                SourceKind::Original => original += span.len,
                SourceKind::Add => add += span.len,
            }
            Ok(())
        })
        .expect("counting source spans cannot fail");
        (original, add)
    }

    pub(super) fn for_each_span(&self, mut visit: impl FnMut(&str)) {
        self.for_each_leaf(&mut |span, _| {
            visit(span.text());
            Ok(())
        })
        .expect("infallible span visitor cannot fail");
    }

    fn for_each_leaf(
        &self,
        visit: &mut dyn FnMut(&SourceSpan, &[u32]) -> io::Result<()>,
    ) -> io::Result<()> {
        if let Some(root) = &self.root {
            visit_leaves(root, visit)?;
        }
        Ok(())
    }

    fn byte_at(&self, index: usize) -> Option<u8> {
        (index < self.len())
            .then(|| byte_at(self.root.as_ref()?, index))
            .flatten()
    }

    #[cfg(debug_assertions)]
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

fn leaf(span: SourceSpan) -> Arc<Node> {
    let newline_offsets = span
        .text()
        .bytes()
        .enumerate()
        .filter_map(|(index, byte)| (byte == b'\n').then_some(index as u32))
        .collect::<Arc<[_]>>();
    Arc::new(Node {
        summary: Summary {
            bytes: span.len,
            newlines: newline_offsets.len(),
            height: 1,
        },
        kind: NodeKind::Leaf {
            span,
            newline_offsets,
        },
    })
}

fn branch(left: Arc<Node>, right: Arc<Node>) -> Arc<Node> {
    Arc::new(Node {
        summary: Summary {
            bytes: left.summary.bytes.saturating_add(right.summary.bytes),
            newlines: left.summary.newlines.saturating_add(right.summary.newlines),
            height: left
                .summary
                .height
                .max(right.summary.height)
                .saturating_add(1),
        },
        kind: NodeKind::Branch { left, right },
    })
}

fn build_balanced(nodes: &[Arc<Node>]) -> Option<Arc<Node>> {
    match nodes.len() {
        0 => None,
        1 => Some(nodes[0].clone()),
        len => {
            let middle = len / 2;
            Some(branch(
                build_balanced(&nodes[..middle]).expect("left half is non-empty"),
                build_balanced(&nodes[middle..]).expect("right half is non-empty"),
            ))
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

fn split_node(node: Option<&Arc<Node>>, index: usize) -> (Option<Arc<Node>>, Option<Arc<Node>>) {
    let Some(node) = node else {
        return (None, None);
    };
    if index == 0 {
        return (None, Some(node.clone()));
    }
    if index >= node.summary.bytes {
        return (Some(node.clone()), None);
    }

    match &node.kind {
        NodeKind::Leaf { span, .. } => {
            let source_index = span.start + index;
            debug_assert!(span.source.text.is_char_boundary(source_index));
            (
                Some(leaf(span.slice(0..index))),
                Some(leaf(span.slice(index..span.len))),
            )
        }
        NodeKind::Branch { left, right } => {
            if index < left.summary.bytes {
                let (before, after) = split_node(Some(left), index);
                (before, join(after, Some(right.clone())))
            } else if index == left.summary.bytes {
                (Some(left.clone()), Some(right.clone()))
            } else {
                let (before, after) = split_node(Some(right), index - left.summary.bytes);
                (join(Some(left.clone()), before), after)
            }
        }
    }
}

fn nth_newline(node: &Arc<Node>, mut index: usize, base: usize) -> Option<usize> {
    if index >= node.summary.newlines {
        return None;
    }
    match &node.kind {
        NodeKind::Leaf {
            newline_offsets, ..
        } => newline_offsets
            .get(index)
            .map(|offset| base + *offset as usize),
        NodeKind::Branch { left, right } => {
            if index < left.summary.newlines {
                nth_newline(left, index, base)
            } else {
                index -= left.summary.newlines;
                nth_newline(right, index, base + left.summary.bytes)
            }
        }
    }
}

fn count_newlines_before(node: &Arc<Node>, index: usize) -> usize {
    let index = index.min(node.summary.bytes);
    match &node.kind {
        NodeKind::Leaf {
            newline_offsets, ..
        } => newline_offsets.partition_point(|offset| (*offset as usize) < index),
        NodeKind::Branch { left, right } => {
            if index <= left.summary.bytes {
                count_newlines_before(left, index)
            } else {
                left.summary.newlines + count_newlines_before(right, index - left.summary.bytes)
            }
        }
    }
}

fn byte_at(node: &Arc<Node>, index: usize) -> Option<u8> {
    match &node.kind {
        NodeKind::Leaf { span, .. } => span.text().as_bytes().get(index).copied(),
        NodeKind::Branch { left, right } => {
            if index < left.summary.bytes {
                byte_at(left, index)
            } else {
                byte_at(right, index - left.summary.bytes)
            }
        }
    }
}

fn visit_leaves(
    node: &Arc<Node>,
    visit: &mut dyn FnMut(&SourceSpan, &[u32]) -> io::Result<()>,
) -> io::Result<()> {
    match &node.kind {
        NodeKind::Leaf {
            span,
            newline_offsets,
        } => visit(span, newline_offsets),
        NodeKind::Branch { left, right } => {
            visit_leaves(left, visit)?;
            visit_leaves(right, visit)
        }
    }
}

#[cfg(debug_assertions)]
fn validate(node: &Arc<Node>) -> Summary {
    let actual = match &node.kind {
        NodeKind::Leaf {
            span,
            newline_offsets,
        } => {
            debug_assert!(span.start + span.len <= span.source.text.len());
            debug_assert!(span.source.text.is_char_boundary(span.start));
            debug_assert!(span.source.text.is_char_boundary(span.start + span.len));
            let offsets = span
                .text()
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index as u32))
                .collect::<Vec<_>>();
            debug_assert_eq!(offsets.as_slice(), newline_offsets.as_ref());
            Summary {
                bytes: span.len,
                newlines: offsets.len(),
                height: 1,
            }
        }
        NodeKind::Branch { left, right } => {
            let left = validate(left);
            let right = validate(right);
            debug_assert!(left.height.abs_diff(right.height) <= 1);
            Summary {
                bytes: left.bytes + right.bytes,
                newlines: left.newlines + right.newlines,
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
        NodeKind::Leaf { .. } => leaves.push(Arc::as_ptr(node)),
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
    fn source_span_tree_splits_replaces_and_streams_without_flattening() {
        let original: Arc<str> = Arc::from("alpha\nbeta\ngamma");
        let tree = SpanTree::from_original(original);
        let inserted = SpanTree::from_addition(Arc::from("BETA\nDELTA"));
        let edited = tree.replace("alpha\n".len().."alpha\nbeta".len(), inserted);

        assert_eq!(edited.text(), "alpha\nBETA\nDELTA\ngamma");
        assert_eq!(edited.line_starts(), vec![0, 6, 11, 17]);
        assert_eq!(edited.source_lengths(), ("alpha\n\ngamma".len(), 10));
        let mut written = Vec::new();
        edited
            .write_to(&mut written)
            .expect("span stream should write");
        assert_eq!(String::from_utf8(written).unwrap(), edited.text());
        edited.assert_invariants();
    }

    #[test]
    fn source_span_tree_reports_crlf_as_endings_not_line_content() {
        let tree = SpanTree::from_original(Arc::from("a\r\nb\n\r\nc"));

        assert_eq!(tree.line_count(), 4);
        assert_eq!(tree.line_starts(), vec![0, 3, 5, 7]);
        assert_eq!(
            tree.line_bounds(0),
            LineBounds {
                start: 0,
                end: 1,
                ending_len: 2,
            }
        );
        assert_eq!(tree.line_bounds(1).ending_len, 1);
        assert_eq!(tree.line_bounds(2).end, tree.line_bounds(2).start);
        assert_eq!(tree.line_and_local_for_index(1), (0, 1));
        assert_eq!(tree.line_and_local_for_index(2), (0, 1));
        assert_eq!(tree.line_and_local_for_index(3), (1, 0));
    }

    #[test]
    fn source_span_tree_clone_is_constant_and_edits_share_untouched_leaves() {
        let text: Arc<str> = Arc::from("x".repeat(TARGET_LEAF_BYTES * 4));
        let tree = SpanTree::from_original(text);
        let clone = tree.clone();
        assert!(tree.shares_root_with(&clone));

        let edited = clone.replace(
            tree.len() - 1..tree.len(),
            SpanTree::from_addition(Arc::from("!")),
        );
        assert_eq!(edited.len(), tree.len());
        assert!(tree.shared_leaf_count(&edited) >= 3);
        assert!(!tree.shares_root_with(&edited));
        edited.assert_invariants();
    }

    #[test]
    fn source_span_tree_line_queries_cross_leaf_boundaries() {
        let prefix = "x".repeat(TARGET_LEAF_BYTES - 1);
        let text: Arc<str> = Arc::from(format!("{prefix}\r\nsecond\nthird"));
        let tree = SpanTree::from_original(text);

        assert_eq!(tree.line_count(), 3);
        assert_eq!(tree.line_start(1), prefix.len() + 2);
        assert_eq!(tree.line_bounds(0).end, prefix.len());
        assert_eq!(
            tree.text_for_range(tree.line_bounds(1).start..tree.line_bounds(1).end),
            "second"
        );
        tree.assert_invariants();
    }

    #[test]
    fn source_span_tree_random_edits_match_string_model() {
        let mut tree = SpanTree::default();
        let mut model = String::new();
        let mut random = 0x9e37_79b9_7f4a_7c15_u64;
        let insertions = ["", "a", "xyz", "\n", "\r\n", "two\nlines"];

        for operation in 0..10_000 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let start = (random as usize) % (model.len() + 1);
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let end = start + (random as usize) % (model.len() - start + 1);
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let inserted = insertions[(random as usize) % insertions.len()];

            model.replace_range(start..end, inserted);
            tree = tree.replace(start..end, SpanTree::from_addition(Arc::from(inserted)));

            assert_eq!(tree.text(), model, "operation {operation}");
            assert_eq!(
                tree.line_starts(),
                std::iter::once(0)
                    .chain(
                        model
                            .bytes()
                            .enumerate()
                            .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
                    )
                    .collect::<Vec<_>>(),
                "operation {operation}"
            );
            tree.assert_invariants();
        }
    }
}
