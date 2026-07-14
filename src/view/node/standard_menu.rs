use super::Node;
use crate::{
    command::{self, Standard, menu::Category},
    context::Context as CommandContext,
    responder, state,
};

#[derive(Clone)]
pub(crate) struct Extension {
    kind: ExtensionKind,
    nodes: Vec<Node>,
}

#[derive(Clone, Copy)]
enum ExtensionKind {
    ItemsBefore(Standard),
    ItemsAfter(Standard),
    SectionBefore(Standard),
    SectionAfter(Standard),
    ReplaceSection(Standard),
    AppendSection(Category),
    ReplaceCategory(Category),
}

#[derive(Clone)]
struct ProjectedCategory {
    category: Category,
    id: &'static str,
    label: &'static str,
    sections: Vec<ProjectedSection>,
}

#[derive(Clone)]
struct ProjectedSection {
    entries: Vec<ProjectedEntry>,
    authored_after: Option<Standard>,
}

#[derive(Clone)]
enum ProjectedEntry {
    Catalog {
        standard: Option<Standard>,
        node: Option<Node>,
    },
    Authored {
        node: Node,
        after: Option<Standard>,
    },
}

#[derive(Clone, Copy)]
struct Location {
    category: usize,
    section: usize,
    entry: usize,
}

impl ProjectedEntry {
    fn standard(&self) -> Option<Standard> {
        match self {
            Self::Catalog { standard, .. } => *standard,
            Self::Authored { .. } => None,
        }
    }

    fn authored_after(&self) -> Option<Standard> {
        match self {
            Self::Catalog { .. } => None,
            Self::Authored { after, .. } => *after,
        }
    }

    fn into_node(self) -> Option<Node> {
        match self {
            Self::Catalog { node, .. } => node,
            Self::Authored { node, .. } => Some(node),
        }
    }
}

impl Extension {
    pub(crate) fn items_before(anchor: Standard, nodes: Vec<Node>) -> Self {
        Self::anchored(ExtensionKind::ItemsBefore(anchor), anchor, nodes)
    }

    pub(crate) fn items_after(anchor: Standard, nodes: Vec<Node>) -> Self {
        Self::anchored(ExtensionKind::ItemsAfter(anchor), anchor, nodes)
    }

    pub(crate) fn section_before(anchor: Standard, nodes: Vec<Node>) -> Self {
        Self::anchored(ExtensionKind::SectionBefore(anchor), anchor, nodes)
    }

    pub(crate) fn section_after(anchor: Standard, nodes: Vec<Node>) -> Self {
        Self::anchored(ExtensionKind::SectionAfter(anchor), anchor, nodes)
    }

    pub(crate) fn replace_section(anchor: Standard, nodes: Vec<Node>) -> Self {
        Self::anchored(ExtensionKind::ReplaceSection(anchor), anchor, nodes)
    }

    pub(crate) fn append_section(category: Category, nodes: Vec<Node>) -> Self {
        Self {
            kind: ExtensionKind::AppendSection(category),
            nodes,
        }
    }

    pub(crate) fn replace_category(category: Category, nodes: Vec<Node>) -> Self {
        Self {
            kind: ExtensionKind::ReplaceCategory(category),
            nodes,
        }
    }

    fn anchored(kind: ExtensionKind, anchor: Standard, nodes: Vec<Node>) -> Self {
        assert!(
            command::menu::standard_is_placed(anchor),
            "standard-menu extension anchor has no cultural slot"
        );
        Self { kind, nodes }
    }

    pub(super) fn resolve_commands(
        &mut self,
        registry: &command::Registry,
        chain: &mut responder::Chain<'_, impl state::State>,
        cx: &CommandContext,
    ) {
        for node in &mut self.nodes {
            node.resolve_commands(registry, chain, cx);
            node.prune_hidden_commands();
        }
    }
}

pub(super) fn project(projection: &command::BarProjection, extensions: &[Extension]) -> Vec<Node> {
    let mut categories = projection
        .catalog()
        .iter()
        .map(|category| ProjectedCategory {
            category: category.category(),
            id: category.id(),
            label: category.label(),
            sections: Vec::new(),
        })
        .collect::<Vec<_>>();

    for category in projection.categories() {
        let projected = category_mut(&mut categories, category.category());
        projected.sections = category
            .sections()
            .iter()
            .map(|section| ProjectedSection {
                entries: section
                    .iter()
                    .map(|entry| ProjectedEntry::Catalog {
                        standard: entry.standard(),
                        node: entry
                            .action()
                            .map(|action| Node::resolved_bar_action(action, entry.show_shortcut())),
                    })
                    .collect(),
                authored_after: None,
            })
            .collect();
    }

    for extension in extensions {
        apply_extension(&mut categories, extension);
    }

    categories
        .into_iter()
        .filter_map(|category| {
            let sections = category
                .sections
                .into_iter()
                .filter_map(|section| {
                    let nodes = section
                        .entries
                        .into_iter()
                        .filter_map(ProjectedEntry::into_node)
                        .collect::<Vec<_>>();
                    (!nodes.is_empty()).then_some(nodes)
                })
                .collect::<Vec<_>>();
            if sections.is_empty() {
                return None;
            }
            let mut menu = Node::menu(category.id, category.label);
            for (section_index, section) in sections.into_iter().enumerate() {
                if section_index > 0 {
                    menu.push_child(Node::separator());
                }
                for node in section {
                    menu.push_child(node);
                }
            }
            Some(menu)
        })
        .collect()
}

fn apply_extension(categories: &mut [ProjectedCategory], extension: &Extension) {
    match extension.kind {
        ExtensionKind::ItemsBefore(anchor) => {
            let location = location(categories, anchor);
            let section = &mut categories[location.category].sections[location.section];
            section.entries.splice(
                location.entry..location.entry,
                authored_entries(&extension.nodes, None),
            );
        }
        ExtensionKind::ItemsAfter(anchor) => {
            let location = location(categories, anchor);
            let section = &mut categories[location.category].sections[location.section];
            let mut index = location.entry + 1;
            while section
                .entries
                .get(index)
                .is_some_and(|entry| entry.authored_after() == Some(anchor))
            {
                index += 1;
            }
            section.entries.splice(
                index..index,
                authored_entries(&extension.nodes, Some(anchor)),
            );
        }
        ExtensionKind::SectionBefore(anchor) => {
            let location = location(categories, anchor);
            let category = &mut categories[location.category];
            category.sections.insert(
                location.section,
                ProjectedSection {
                    entries: authored_entries(&extension.nodes, None),
                    authored_after: None,
                },
            );
        }
        ExtensionKind::SectionAfter(anchor) => {
            let location = location(categories, anchor);
            let category = &mut categories[location.category];
            let mut index = location.section + 1;
            while category
                .sections
                .get(index)
                .is_some_and(|section| section.authored_after == Some(anchor))
            {
                index += 1;
            }
            category.sections.insert(
                index,
                ProjectedSection {
                    entries: authored_entries(&extension.nodes, None),
                    authored_after: Some(anchor),
                },
            );
        }
        ExtensionKind::ReplaceSection(anchor) => {
            let location = location(categories, anchor);
            let section = &mut categories[location.category].sections[location.section];
            let mut markers = section
                .entries
                .iter()
                .filter_map(|entry| {
                    entry.standard().map(|standard| ProjectedEntry::Catalog {
                        standard: Some(standard),
                        node: None,
                    })
                })
                .collect::<Vec<_>>();
            markers.extend(authored_entries(&extension.nodes, None));
            section.entries = markers;
        }
        ExtensionKind::AppendSection(category) => {
            category_mut(categories, category)
                .sections
                .push(ProjectedSection {
                    entries: authored_entries(&extension.nodes, None),
                    authored_after: None,
                });
        }
        ExtensionKind::ReplaceCategory(category) => {
            category_mut(categories, category).sections = vec![ProjectedSection {
                entries: authored_entries(&extension.nodes, None),
                authored_after: None,
            }];
        }
    }
}

fn authored_entries(nodes: &[Node], authored_after: Option<Standard>) -> Vec<ProjectedEntry> {
    nodes
        .iter()
        .cloned()
        .map(|node| ProjectedEntry::Authored {
            node,
            after: authored_after,
        })
        .collect()
}

fn category_mut(
    categories: &mut [ProjectedCategory],
    category: Category,
) -> &mut ProjectedCategory {
    categories
        .iter_mut()
        .find(|candidate| candidate.category == category)
        .unwrap_or_else(|| panic!("menu extension references an unregistered custom category"))
}

fn location(categories: &[ProjectedCategory], anchor: Standard) -> Location {
    for (category_index, category) in categories.iter().enumerate() {
        for (section_index, section) in category.sections.iter().enumerate() {
            if let Some(entry_index) = section
                .entries
                .iter()
                .position(|entry| entry.standard() == Some(anchor))
            {
                return Location {
                    category: category_index,
                    section: section_index,
                    entry: entry_index,
                };
            }
        }
    }
    panic!("standard-menu extension anchor is absent from the platform topology")
}
