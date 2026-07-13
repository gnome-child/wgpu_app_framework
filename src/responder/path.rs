use super::Scope;

/// The question a responder surface asks of its semantic boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Traversal {
    /// Serve the active task, from its exact frame toward broader owners.
    Task,
    /// Inspect an object, from its containing domain toward its exact facet.
    Inspection,
}

/// Ordered command-owning semantic boundaries.
///
/// Layers are stored broad-to-exact. Traversal chooses how that one ordering is
/// consumed; it does not rebuild or re-rank the path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Path {
    layers: Vec<Scope>,
    task_frame: usize,
}

impl Path {
    pub(crate) fn single(scope: Scope) -> Self {
        Self {
            layers: vec![scope],
            task_frame: 0,
        }
    }

    pub(crate) fn new(layers: impl IntoIterator<Item = Scope>, task_frame: usize) -> Self {
        let layers = layers.into_iter().collect::<Vec<_>>();
        let task_frame = task_frame.min(layers.len().saturating_sub(1));
        Self { layers, task_frame }
    }

    #[cfg(test)]
    pub(crate) fn task_scope(&self) -> Option<Scope> {
        self.layers.get(self.task_frame).copied()
    }

    pub(crate) fn scopes(&self, traversal: Traversal) -> impl Iterator<Item = Scope> + '_ {
        self.ordinals(traversal).map(|ordinal| self.layers[ordinal])
    }

    /// Source-layer order for this question. Section projection and command
    /// claiming consume these same ordinals so their ordering cannot drift.
    pub(crate) fn ordinals(&self, traversal: Traversal) -> impl Iterator<Item = usize> + '_ {
        let count = match traversal {
            Traversal::Task if self.layers.is_empty() => 0,
            Traversal::Task => self.task_frame + 1,
            Traversal::Inspection => self.layers.len(),
        };
        (0..count).map(move |offset| match traversal {
            Traversal::Task => self.task_frame - offset,
            Traversal::Inspection => offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{interaction, responder};

    #[test]
    fn one_path_supports_task_and_inspection_questions() {
        let table = responder::Scope::contextual(Some(interaction::Id::new("table")), None);
        let row = responder::Scope::contextual(Some(interaction::Id::new("row")), None);
        let facet = responder::Scope::contextual(Some(interaction::Id::new("facet")), None);
        let detail = responder::Scope::contextual(Some(interaction::Id::new("detail")), None);
        let path = Path::new([table, row, facet, detail], 2);

        assert_eq!(
            path.scopes(Traversal::Inspection).collect::<Vec<_>>(),
            vec![table, row, facet, detail]
        );
        assert_eq!(
            path.scopes(Traversal::Task).collect::<Vec<_>>(),
            vec![facet, row, table]
        );
        assert_eq!(path.task_scope(), Some(facet));
        assert_eq!(
            path.ordinals(Traversal::Inspection).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            path.ordinals(Traversal::Task).collect::<Vec<_>>(),
            vec![2, 1, 0]
        );
    }
}
