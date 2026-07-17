use std::{collections::HashSet, sync::Arc};

use thiserror::Error;

use super::super::{composition, interaction, layout};
use super::{Commit, Draw, GeometryRevision, TopologyRevision, commit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Revision(u64);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Residency {
    revision: Revision,
    commit: commit::Revision,
    scroll: composition::tree::NodeId,
    target: interaction::Target,
    nodes: Arc<[Resident]>,
    draw_order: Arc<[composition::tree::NodeId]>,
    minimum: interaction::ScrollOffset,
    maximum: interaction::ScrollOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Resident {
    node: composition::tree::NodeId,
    content: composition::tree::ContentRevision,
    geometry: GeometryRevision,
    topology: TopologyRevision,
}

pub(crate) struct Builder<'a> {
    commit: &'a Commit,
    drawable: Arc<Commit>,
    previous: &'a [Residency],
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum ContractError {
    #[error("scene residency for {0:?} does not contain its scroll node")]
    MissingScrollNode(composition::tree::NodeId),
    #[error("scene residency contains duplicate node {0:?}")]
    DuplicateNode(composition::tree::NodeId),
    #[error("scene residency has an inverted accepted-offset interval")]
    InvertedOffsets,
    #[error("scene residency targets semantic commit {actual:?}, expected {expected:?}")]
    IncompatibleCommit {
        expected: commit::Revision,
        actual: commit::Revision,
    },
}

impl Revision {
    pub(crate) const INITIAL: Self = Self(1);

    pub(crate) fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl Residency {
    #[allow(clippy::too_many_arguments)]
    fn new(
        revision: Revision,
        commit: commit::Revision,
        scroll: composition::tree::NodeId,
        target: interaction::Target,
        nodes: Vec<Resident>,
        draw_order: Vec<composition::tree::NodeId>,
        minimum: interaction::ScrollOffset,
        maximum: interaction::ScrollOffset,
    ) -> Result<Self, ContractError> {
        if minimum
            .axis_cmp(maximum, interaction::ScrollbarAxis::Horizontal)
            .is_gt()
            || minimum
                .axis_cmp(maximum, interaction::ScrollbarAxis::Vertical)
                .is_gt()
        {
            return Err(ContractError::InvertedOffsets);
        }
        let mut unique = HashSet::with_capacity(nodes.len());
        for resident in &nodes {
            if !unique.insert(resident.node) {
                return Err(ContractError::DuplicateNode(resident.node));
            }
        }
        if !unique.contains(&scroll) {
            return Err(ContractError::MissingScrollNode(scroll));
        }
        Ok(Self {
            revision,
            commit,
            scroll,
            target,
            nodes: Arc::from(nodes.into_boxed_slice()),
            draw_order: Arc::from(draw_order.into_boxed_slice()),
            minimum,
            maximum,
        })
    }

    pub(crate) fn revision(&self) -> Revision {
        self.revision
    }

    pub(crate) fn scroll(&self) -> composition::tree::NodeId {
        self.scroll
    }

    pub(crate) fn accepts(&self, offset: interaction::ScrollOffset) -> bool {
        offset.lies_within(self.minimum, self.maximum)
    }

    pub(crate) fn project(&self, offset: interaction::ScrollOffset) -> interaction::ScrollOffset {
        offset.clamped(self.minimum, self.maximum)
    }

    pub(crate) fn require_compatible(&self, commit: &Commit) -> Result<(), ContractError> {
        if self.commit == commit.revision() {
            Ok(())
        } else {
            Err(ContractError::IncompatibleCommit {
                expected: commit.revision(),
                actual: self.commit,
            })
        }
    }

    pub(crate) fn node_ids(&self) -> impl Iterator<Item = composition::tree::NodeId> + '_ {
        self.nodes.iter().map(|resident| resident.node)
    }

    pub(crate) fn draw_order(&self) -> &[composition::tree::NodeId] {
        &self.draw_order
    }

    fn same_snapshot(&self, other: &Self) -> bool {
        self.commit == other.commit
            && self.scroll == other.scroll
            && self.target == other.target
            && self.nodes == other.nodes
            && self.draw_order == other.draw_order
            && self.minimum == other.minimum
            && self.maximum == other.maximum
    }
}

impl<'a> Builder<'a> {
    pub(crate) fn new(
        commit: &'a Commit,
        drawable: Arc<Commit>,
        previous: &'a [Residency],
    ) -> Self {
        Self {
            commit,
            drawable,
            previous,
        }
    }

    pub(crate) fn build(self, layout: &layout::Layout) -> Result<Arc<[Residency]>, ContractError> {
        let mut residencies = Vec::new();
        for projection in layout
            .scroll_projections()
            .iter()
            .filter(|projection| projection.is_scene_drawable())
        {
            if !self
                .drawable
                .nodes()
                .iter()
                .any(|node| node.id() == projection.node())
            {
                continue;
            }
            let Some((minimum, maximum)) = projection.accepted_offsets() else {
                continue;
            };
            let resident_ids = layout
                .resident_node_ids(projection.node())
                .into_iter()
                .collect::<HashSet<_>>();
            let nodes = self
                .drawable
                .nodes()
                .iter()
                .filter(|node| resident_ids.contains(&node.id()))
                .map(|node| Resident {
                    node: node.id(),
                    content: node.content_revision(),
                    geometry: node.geometry_revision(),
                    topology: node.topology_revision(),
                })
                .collect::<Vec<_>>();
            let draw_order = self.drawable.order().map_or_else(
                || nodes.iter().map(|resident| resident.node).collect(),
                |order| {
                    order
                        .iter()
                        .filter_map(|draw| match draw {
                            Draw::Content { node, .. } if resident_ids.contains(node) => {
                                Some(*node)
                            }
                            _ => None,
                        })
                        .collect()
                },
            );
            let previous = self.previous.iter().find(|residency| {
                residency.scroll == projection.node()
                    && residency.require_compatible(self.commit).is_ok()
            });
            let revision =
                previous.map_or(Revision::INITIAL, |residency| residency.revision.next());
            let candidate = Residency::new(
                revision,
                self.commit.revision(),
                projection.node(),
                projection.target().clone(),
                nodes,
                draw_order,
                minimum,
                maximum,
            )?;
            if let Some(previous) = previous.filter(|previous| previous.same_snapshot(&candidate)) {
                residencies.push(previous.clone());
            } else {
                residencies.push(candidate);
            }
        }
        Ok(Arc::from(residencies.into_boxed_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_or_duplicate_resident_membership_is_rejected() {
        let mut next = 1;
        let scroll = composition::tree::NodeId::layout(&mut next);
        let resident = Resident {
            node: scroll,
            content: composition::tree::ContentRevision::INITIAL,
            geometry: GeometryRevision::INITIAL,
            topology: TopologyRevision::INITIAL,
        };
        let target = interaction::Target::scroll("residency.scroll", "Residency");
        assert_eq!(
            Residency::new(
                Revision::INITIAL,
                commit::Revision::INITIAL,
                scroll,
                target.clone(),
                vec![resident],
                vec![scroll],
                interaction::ScrollOffset::new(0, 20),
                interaction::ScrollOffset::new(0, 10),
            ),
            Err(ContractError::InvertedOffsets)
        );
        assert!(matches!(
            Residency::new(
                Revision::INITIAL,
                commit::Revision::INITIAL,
                scroll,
                target,
                vec![resident, resident],
                vec![scroll],
                interaction::ScrollOffset::default(),
                interaction::ScrollOffset::new(0, 10),
            ),
            Err(ContractError::DuplicateNode(node)) if node == scroll
        ));
    }

    #[test]
    fn projection_clamps_each_axis_to_the_complete_interval() {
        let mut next = 1;
        let scroll = composition::tree::NodeId::layout(&mut next);
        let resident = Resident {
            node: scroll,
            content: composition::tree::ContentRevision::INITIAL,
            geometry: GeometryRevision::INITIAL,
            topology: TopologyRevision::INITIAL,
        };
        let residency = Residency::new(
            Revision::INITIAL,
            commit::Revision::INITIAL,
            scroll,
            interaction::Target::scroll("residency.scroll", "Residency"),
            vec![resident],
            vec![scroll],
            interaction::ScrollOffset::new(10, 20),
            interaction::ScrollOffset::new(30, 40),
        )
        .expect("residency fixture");

        assert_eq!(
            residency.project(interaction::ScrollOffset::new(50, 0)),
            interaction::ScrollOffset::new(30, 20)
        );
        assert_eq!(
            residency.project(interaction::ScrollOffset::new(15, 35)),
            interaction::ScrollOffset::new(15, 35)
        );
    }
}
