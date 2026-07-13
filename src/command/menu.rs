#![allow(
    dead_code,
    reason = "pure topology is consumed by the derived bar in checkpoint 4"
)]

use std::{
    any::TypeId,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

use super::Standard;
use crate::keymap::Platform;

/// Stable identity and registration metadata for a top-level menu category.
///
/// Standard categories are constants. A custom category uses the application
/// marker type as identity and registers its label exactly once.
#[derive(Clone, Copy, Debug)]
pub struct Category {
    identity: CategoryIdentity,
    identity_name: &'static str,
    label: Option<&'static str>,
    position: CategoryPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CategoryIdentity {
    File,
    Edit,
    View,
    Tools,
    Window,
    Help,
    Custom(TypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CategoryPosition {
    Standard,
    CustomDefault,
    Before(CategoryIdentity),
    After(CategoryIdentity),
}

/// Typed static placement for a command in the conventional bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement(PlacementKind);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::command) enum PlacementKind {
    Category(Category),
    Before(Standard),
    After(Standard),
    SectionBefore(Standard),
    SectionAfter(Standard),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Section {
    Document,
    Persistence,
    SaveItem,
    History,
    Clipboard,
    Selection,
    Deletion,
    Pasteboard,
}

#[derive(Clone, Copy, Debug)]
struct Slot {
    category: CategoryIdentity,
    section: Section,
    ordinal: usize,
    show_shortcut: bool,
}

#[derive(Clone)]
pub(in crate::command) struct Item {
    pub(in crate::command) command_type: TypeId,
    pub(in crate::command) command_name: &'static str,
    pub(in crate::command) command_type_name: &'static str,
    pub(in crate::command) standard: Option<Standard>,
    pub(in crate::command) placement: Option<Placement>,
    pub(in crate::command) suppressed: bool,
    pub(in crate::command) shortcut_visibility: Option<bool>,
}

pub(in crate::command) struct Topology {
    categories: Vec<TopologyCategory>,
}

pub(in crate::command) struct TopologyCategory {
    category: Category,
    label: &'static str,
    sections: Vec<Vec<TopologyEntry>>,
}

#[derive(Clone, Copy)]
pub(in crate::command) struct TopologyEntry {
    command_type: TypeId,
    standard: Option<Standard>,
    show_shortcut: bool,
}

impl Category {
    pub const FILE: Self = Self::standard(CategoryIdentity::File, "File");
    pub const EDIT: Self = Self::standard(CategoryIdentity::Edit, "Edit");
    pub const VIEW: Self = Self::standard(CategoryIdentity::View, "View");
    pub const TOOLS: Self = Self::standard(CategoryIdentity::Tools, "Tools");
    pub const WINDOW: Self = Self::standard(CategoryIdentity::Window, "Window");
    pub const HELP: Self = Self::standard(CategoryIdentity::Help, "Help");

    const fn standard(identity: CategoryIdentity, label: &'static str) -> Self {
        Self {
            identity,
            identity_name: label,
            label: Some(label),
            position: CategoryPosition::Standard,
        }
    }

    /// References a custom category by marker type.
    pub fn of<T: 'static>() -> Self {
        Self {
            identity: CategoryIdentity::Custom(TypeId::of::<T>()),
            identity_name: std::any::type_name::<T>(),
            label: None,
            position: CategoryPosition::CustomDefault,
        }
    }

    /// Declares a custom category. Register this value once with
    /// [`crate::command::Registry::menu_category`].
    pub fn new<T: 'static>(label: &'static str) -> Self {
        Self {
            label: Some(label),
            ..Self::of::<T>()
        }
    }

    /// Places a custom category immediately before a standard category band.
    pub fn before(mut self, anchor: Self) -> Self {
        self.assert_custom_declaration();
        assert!(
            anchor.is_standard(),
            "custom category positions require a standard category anchor"
        );
        self.position = CategoryPosition::Before(anchor.identity);
        self
    }

    /// Places a custom category immediately after a standard category band.
    pub fn after(mut self, anchor: Self) -> Self {
        self.assert_custom_declaration();
        assert!(
            anchor.is_standard(),
            "custom category positions require a standard category anchor"
        );
        self.position = CategoryPosition::After(anchor.identity);
        self
    }

    pub(in crate::command) fn label(self) -> Option<&'static str> {
        self.label
    }

    pub(in crate::command) fn is_standard(self) -> bool {
        !matches!(self.identity, CategoryIdentity::Custom(_))
    }

    pub(in crate::command) fn is_custom_declaration(self) -> bool {
        !self.is_standard() && self.label.is_some()
    }

    pub(in crate::command) fn same_declaration(self, other: Self) -> bool {
        self.identity == other.identity
            && self.identity_name == other.identity_name
            && self.label == other.label
            && self.position == other.position
    }

    fn assert_custom_declaration(self) {
        assert!(
            self.is_custom_declaration(),
            "category positioning requires Category::new::<T>(label)"
        );
    }
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for Category {}

impl Hash for Category {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

impl Placement {
    pub fn category(category: Category) -> Self {
        Self(PlacementKind::Category(category))
    }

    pub fn before(anchor: Standard) -> Self {
        Self(PlacementKind::Before(anchor))
    }

    pub fn after(anchor: Standard) -> Self {
        Self(PlacementKind::After(anchor))
    }

    pub fn section_before(anchor: Standard) -> Self {
        Self(PlacementKind::SectionBefore(anchor))
    }

    pub fn section_after(anchor: Standard) -> Self {
        Self(PlacementKind::SectionAfter(anchor))
    }

    pub(in crate::command) fn kind(self) -> PlacementKind {
        self.0
    }
}

impl Topology {
    pub(in crate::command) fn resolve(
        platform: Platform,
        registered_categories: &HashMap<Category, Category>,
        mut items: Vec<Item>,
    ) -> Self {
        items.sort_by(|left, right| {
            left.command_name
                .cmp(right.command_name)
                .then_with(|| left.command_type_name.cmp(right.command_type_name))
        });
        validate_placements(&items, registered_categories);
        let mut declared_categories = standard_categories()
            .into_iter()
            .map(|category| (category, category))
            .collect::<HashMap<_, _>>();
        declared_categories.extend(
            registered_categories
                .iter()
                .map(|(identity, declaration)| (*identity, *declaration)),
        );

        let mut category_ids = standard_categories()
            .into_iter()
            .map(|category| category.identity)
            .collect::<Vec<_>>();
        category_ids.extend(
            registered_categories
                .values()
                .map(|category| category.identity),
        );
        category_ids.sort_by(|left, right| {
            compare_categories(
                declared_categories
                    .get(&Category::from_identity(*left))
                    .copied()
                    .expect("category declaration"),
                declared_categories
                    .get(&Category::from_identity(*right))
                    .copied()
                    .expect("category declaration"),
            )
        });
        category_ids.dedup();

        let mut categories = Vec::new();
        for category_id in category_ids {
            let category = declared_categories
                .get(&Category::from_identity(category_id))
                .copied()
                .expect("category declaration");
            let sections = resolve_category(
                platform,
                category_id,
                registered_categories,
                items.iter().filter(|item| !item.suppressed),
            );
            if sections.is_empty() {
                continue;
            }
            categories.push(TopologyCategory {
                category,
                label: category.label().expect("registered category label"),
                sections,
            });
        }

        Self { categories }
    }

    pub(in crate::command) fn categories(&self) -> &[TopologyCategory] {
        &self.categories
    }
}

impl TopologyCategory {
    pub(in crate::command) fn category(&self) -> Category {
        self.category
    }

    pub(in crate::command) fn label(&self) -> &'static str {
        self.label
    }

    pub(in crate::command) fn sections(&self) -> &[Vec<TopologyEntry>] {
        &self.sections
    }
}

impl TopologyEntry {
    pub(in crate::command) fn command_type(self) -> TypeId {
        self.command_type
    }

    pub(in crate::command) fn standard(self) -> Option<Standard> {
        self.standard
    }

    pub(in crate::command) fn show_shortcut(self) -> bool {
        self.show_shortcut
    }
}

impl Category {
    const fn from_identity(identity: CategoryIdentity) -> Self {
        match identity {
            CategoryIdentity::File => Self::FILE,
            CategoryIdentity::Edit => Self::EDIT,
            CategoryIdentity::View => Self::VIEW,
            CategoryIdentity::Tools => Self::TOOLS,
            CategoryIdentity::Window => Self::WINDOW,
            CategoryIdentity::Help => Self::HELP,
            CategoryIdentity::Custom(identity) => Self {
                identity: CategoryIdentity::Custom(identity),
                identity_name: "unregistered custom category",
                label: None,
                position: CategoryPosition::CustomDefault,
            },
        }
    }
}

fn resolve_category<'a>(
    platform: Platform,
    category: CategoryIdentity,
    registered_categories: &HashMap<Category, Category>,
    items: impl Iterator<Item = &'a Item> + Clone,
) -> Vec<Vec<TopologyEntry>> {
    let mut sections = Vec::new();
    let template = template_sections(platform, category);

    for roles in template {
        let role_set = roles.iter().copied().collect::<HashSet<_>>();
        let before_section = items
            .clone()
            .filter(|item| {
                item.placement.is_some_and(|placement| {
                    matches!(placement.kind(), PlacementKind::SectionBefore(anchor) if role_set.contains(&anchor))
                })
            })
            .map(|item| topology_entry(platform, item))
            .collect::<Vec<_>>();
        let after_section = items
            .clone()
            .filter(|item| {
                item.placement.is_some_and(|placement| {
                    matches!(placement.kind(), PlacementKind::SectionAfter(anchor) if role_set.contains(&anchor))
                })
            })
            .map(|item| topology_entry(platform, item))
            .collect::<Vec<_>>();
        if !before_section.is_empty() {
            sections.push(before_section);
        }

        let mut section = Vec::new();
        for role in roles {
            section.extend(
                items
                    .clone()
                    .filter(|item| {
                        item.placement.is_some_and(|placement| {
                            matches!(placement.kind(), PlacementKind::Before(anchor) if anchor == role)
                        })
                    })
                    .map(|item| topology_entry(platform, item)),
            );
            section.extend(
                items
                    .clone()
                    .filter(|item| item.standard == Some(role) && item.placement.is_none())
                    .map(|item| topology_entry(platform, item)),
            );
            section.extend(
                items
                    .clone()
                    .filter(|item| {
                        item.placement.is_some_and(|placement| {
                            matches!(placement.kind(), PlacementKind::After(anchor) if anchor == role)
                        })
                    })
                    .map(|item| topology_entry(platform, item)),
            );
        }
        if !section.is_empty() {
            sections.push(section);
        }
        if !after_section.is_empty() {
            sections.push(after_section);
        }
    }

    let category = Category::from_identity(category);
    let direct = items
        .filter(|item| {
            item.placement.is_some_and(|placement| {
                matches!(placement.kind(), PlacementKind::Category(target) if target == category)
            })
        })
        .map(|item| {
            if !category.is_standard() {
                assert!(
                    registered_categories.contains_key(&category),
                    "menu placement references an unregistered custom category"
                );
            }
            topology_entry(platform, item)
        })
        .collect::<Vec<_>>();
    if !direct.is_empty() {
        sections.push(direct);
    }

    sections
}

fn topology_entry(platform: Platform, item: &Item) -> TopologyEntry {
    TopologyEntry {
        command_type: item.command_type,
        standard: item.standard,
        show_shortcut: item.shortcut_visibility.unwrap_or_else(|| {
            item.standard.is_none_or(|role| {
                standard_slot(platform, role).is_none_or(|slot| slot.show_shortcut)
            })
        }),
    }
}

fn validate_placements(items: &[Item], registered_categories: &HashMap<Category, Category>) {
    for item in items {
        let Some(placement) = item.placement else {
            continue;
        };
        match placement.kind() {
            PlacementKind::Category(category) => {
                assert!(
                    category.is_standard() || registered_categories.contains_key(&category),
                    "menu placement references an unregistered custom category"
                );
            }
            PlacementKind::Before(anchor)
            | PlacementKind::After(anchor)
            | PlacementKind::SectionBefore(anchor)
            | PlacementKind::SectionAfter(anchor) => {
                assert!(
                    standard_is_placed(anchor),
                    "menu placement references a standard role without a virtual slot"
                );
            }
        }
    }
}

pub(in crate::command) fn standard_is_placed(standard: Standard) -> bool {
    [Platform::Windows, Platform::Mac, Platform::Linux]
        .into_iter()
        .any(|platform| standard_slot(platform, standard).is_some())
}

fn standard_slot(platform: Platform, standard: Standard) -> Option<Slot> {
    let category = match standard {
        Standard::New
        | Standard::Open
        | Standard::Save
        | Standard::SaveAs
        | Standard::CloseWindow => CategoryIdentity::File,
        Standard::Undo
        | Standard::Redo
        | Standard::Cut
        | Standard::Copy
        | Standard::Paste
        | Standard::Delete
        | Standard::SelectAll => CategoryIdentity::Edit,
        Standard::CommandPalette => return None,
    };
    let (section, ordinal) = match (platform, standard) {
        (Platform::Mac, Standard::New) => (Section::Document, 0),
        (Platform::Mac, Standard::Open) => (Section::Document, 1),
        (Platform::Mac, Standard::CloseWindow) => (Section::SaveItem, 0),
        (Platform::Mac, Standard::Save) => (Section::SaveItem, 1),
        (Platform::Mac, Standard::SaveAs) => (Section::SaveItem, 2),
        (Platform::Mac, Standard::Undo) => (Section::History, 0),
        (Platform::Mac, Standard::Redo) => (Section::History, 1),
        (Platform::Mac, Standard::Cut) => (Section::Pasteboard, 0),
        (Platform::Mac, Standard::Copy) => (Section::Pasteboard, 1),
        (Platform::Mac, Standard::Paste) => (Section::Pasteboard, 2),
        (Platform::Mac, Standard::Delete) => (Section::Pasteboard, 3),
        (Platform::Mac, Standard::SelectAll) => (Section::Pasteboard, 4),
        (_, Standard::New) => (Section::Document, 0),
        (_, Standard::Open) => (Section::Document, 1),
        (_, Standard::CloseWindow) => (Section::Document, 2),
        (_, Standard::Save) => (Section::Persistence, 0),
        (_, Standard::SaveAs) => (Section::Persistence, 1),
        (_, Standard::Undo) => (Section::History, 0),
        (_, Standard::Redo) => (Section::History, 1),
        (_, Standard::Cut) => (Section::Clipboard, 0),
        (_, Standard::Copy) => (Section::Clipboard, 1),
        (_, Standard::Paste) => (Section::Clipboard, 2),
        (_, Standard::SelectAll) => (Section::Selection, 0),
        (_, Standard::Delete) => (Section::Deletion, 0),
        (_, Standard::CommandPalette) => return None,
    };
    Some(Slot {
        category,
        section,
        ordinal,
        show_shortcut: !(matches!(platform, Platform::Windows | Platform::Linux)
            && standard == Standard::Delete),
    })
}

fn template_sections(platform: Platform, category: CategoryIdentity) -> Vec<Vec<Standard>> {
    let mut slots = [
        Standard::Undo,
        Standard::Redo,
        Standard::Cut,
        Standard::Copy,
        Standard::Paste,
        Standard::Delete,
        Standard::SelectAll,
        Standard::New,
        Standard::Open,
        Standard::Save,
        Standard::SaveAs,
        Standard::CloseWindow,
    ]
    .into_iter()
    .filter_map(|role| standard_slot(platform, role).map(|slot| (role, slot)))
    .filter(|(_, slot)| slot.category == category)
    .collect::<Vec<_>>();
    slots.sort_by_key(|(_, slot)| (section_order(platform, slot.section), slot.ordinal));

    let mut sections = Vec::<Vec<Standard>>::new();
    let mut current = None;
    for (role, slot) in slots {
        if current != Some(slot.section) {
            sections.push(Vec::new());
            current = Some(slot.section);
        }
        sections
            .last_mut()
            .expect("section just inserted")
            .push(role);
    }
    sections
}

fn section_order(platform: Platform, section: Section) -> usize {
    match (platform, section) {
        (_, Section::Document) => 0,
        (_, Section::Persistence | Section::SaveItem) => 1,
        (_, Section::History) => 0,
        (_, Section::Clipboard | Section::Pasteboard) => 1,
        (_, Section::Selection) => 2,
        (_, Section::Deletion) => 3,
    }
}

fn standard_categories() -> [Category; 6] {
    [
        Category::FILE,
        Category::EDIT,
        Category::VIEW,
        Category::TOOLS,
        Category::WINDOW,
        Category::HELP,
    ]
}

fn compare_categories(left: Category, right: Category) -> Ordering {
    category_sort_key(left)
        .cmp(&category_sort_key(right))
        .then_with(|| left.identity_name.cmp(right.identity_name))
}

fn category_sort_key(category: Category) -> i16 {
    let standard = |identity| match identity {
        CategoryIdentity::File => 0,
        CategoryIdentity::Edit => 10,
        CategoryIdentity::View => 20,
        CategoryIdentity::Tools => 40,
        CategoryIdentity::Window => 50,
        CategoryIdentity::Help => 60,
        CategoryIdentity::Custom(_) => 30,
    };
    match category.position {
        CategoryPosition::Standard => standard(category.identity),
        CategoryPosition::CustomDefault => 30,
        CategoryPosition::Before(anchor) => standard(anchor) - 1,
        CategoryPosition::After(anchor) => standard(anchor) + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Command, Registry, Spec};

    macro_rules! unit_command {
        ($name:ident, $command_name:literal) => {
            struct $name;

            impl Command for $name {
                type Args = ();
                type Output = ();

                const NAME: &'static str = $command_name;
            }
        };
    }

    unit_command!(NewCommand, "test.new");
    unit_command!(OpenCommand, "test.open");
    unit_command!(SaveCommand, "test.save");
    unit_command!(SaveAsCommand, "test.save_as");
    unit_command!(CloseCommand, "test.close");
    unit_command!(UndoCommand, "test.undo");
    unit_command!(RedoCommand, "test.redo");
    unit_command!(CutCommand, "test.cut");
    unit_command!(CopyCommand, "test.copy");
    unit_command!(PasteCommand, "test.paste");
    unit_command!(SelectAllCommand, "test.select_all");
    unit_command!(DeleteCommand, "test.delete");
    unit_command!(PaletteCommand, "test.palette");
    unit_command!(AfterOpen, "test.after_open");
    unit_command!(ViewCommand, "test.view");
    unit_command!(ToolsCommand, "test.tools");
    unit_command!(ControlsCommand, "test.controls");
    unit_command!(OtherControlsCommand, "test.other_controls");

    struct Controls;
    struct OtherControls;

    struct ArgumentCommand;

    impl Command for ArgumentCommand {
        type Args = usize;
        type Output = ();

        const NAME: &'static str = "test.argument";
    }

    fn complete_registry(reverse: bool) -> Registry {
        let mut registry = Registry::default();
        if reverse {
            registry
                .register::<DeleteCommand>(Spec::standard(Standard::Delete))
                .register::<SelectAllCommand>(Spec::standard(Standard::SelectAll))
                .register::<PasteCommand>(Spec::standard(Standard::Paste))
                .register::<CopyCommand>(Spec::standard(Standard::Copy))
                .register::<CutCommand>(Spec::standard(Standard::Cut))
                .register::<RedoCommand>(Spec::standard(Standard::Redo))
                .register::<UndoCommand>(Spec::standard(Standard::Undo))
                .register::<CloseCommand>(Spec::standard(Standard::CloseWindow))
                .register::<SaveAsCommand>(Spec::standard(Standard::SaveAs))
                .register::<SaveCommand>(Spec::standard(Standard::Save))
                .register::<OpenCommand>(Spec::standard(Standard::Open))
                .register::<NewCommand>(Spec::standard(Standard::New))
                .register::<PaletteCommand>(Spec::standard(Standard::CommandPalette));
        } else {
            registry
                .register::<NewCommand>(Spec::standard(Standard::New))
                .register::<OpenCommand>(Spec::standard(Standard::Open))
                .register::<SaveCommand>(Spec::standard(Standard::Save))
                .register::<SaveAsCommand>(Spec::standard(Standard::SaveAs))
                .register::<CloseCommand>(Spec::standard(Standard::CloseWindow))
                .register::<UndoCommand>(Spec::standard(Standard::Undo))
                .register::<RedoCommand>(Spec::standard(Standard::Redo))
                .register::<CutCommand>(Spec::standard(Standard::Cut))
                .register::<CopyCommand>(Spec::standard(Standard::Copy))
                .register::<PasteCommand>(Spec::standard(Standard::Paste))
                .register::<SelectAllCommand>(Spec::standard(Standard::SelectAll))
                .register::<DeleteCommand>(Spec::standard(Standard::Delete))
                .register::<PaletteCommand>(Spec::standard(Standard::CommandPalette));
        }
        registry
    }

    fn role_signature(topology: &Topology) -> Vec<(&'static str, Vec<Vec<Standard>>)> {
        topology
            .categories()
            .iter()
            .map(|category| {
                (
                    category.label(),
                    category
                        .sections()
                        .iter()
                        .map(|section| {
                            section
                                .iter()
                                .filter_map(|entry| entry.standard())
                                .collect()
                        })
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn platform_templates_are_pure_and_registration_order_independent() {
        let forward = complete_registry(false);
        let reverse = complete_registry(true);

        let windows = forward.population().menu_topology(Platform::Windows);
        let windows_reverse = reverse.population().menu_topology(Platform::Windows);
        let expected_windows = vec![
            (
                "File",
                vec![
                    vec![Standard::New, Standard::Open, Standard::CloseWindow],
                    vec![Standard::Save, Standard::SaveAs],
                ],
            ),
            (
                "Edit",
                vec![
                    vec![Standard::Undo, Standard::Redo],
                    vec![Standard::Cut, Standard::Copy, Standard::Paste],
                    vec![Standard::SelectAll],
                    vec![Standard::Delete],
                ],
            ),
        ];
        assert_eq!(role_signature(&windows), expected_windows);
        assert_eq!(role_signature(&windows_reverse), expected_windows);

        let mac = forward.population().menu_topology(Platform::Mac);
        assert_eq!(
            role_signature(&mac),
            vec![
                (
                    "File",
                    vec![
                        vec![Standard::New, Standard::Open],
                        vec![Standard::CloseWindow, Standard::Save, Standard::SaveAs],
                    ],
                ),
                (
                    "Edit",
                    vec![
                        vec![Standard::Undo, Standard::Redo],
                        vec![
                            Standard::Cut,
                            Standard::Copy,
                            Standard::Paste,
                            Standard::Delete,
                            Standard::SelectAll,
                        ],
                    ],
                ),
            ]
        );
    }

    #[test]
    fn virtual_standard_slots_anchor_placements_when_the_role_is_absent() {
        let mut registry = Registry::default();
        registry
            .register::<AfterOpen>(
                Spec::new("After Open").placement(Placement::after(Standard::Open)),
            )
            .register::<PaletteCommand>(Spec::standard(Standard::CommandPalette))
            .register::<CopyCommand>(Spec::standard(Standard::Copy).unplaced());

        let topology = registry.population().menu_topology(Platform::Windows);
        assert_eq!(topology.categories().len(), 1);
        let file = &topology.categories()[0];
        assert_eq!(file.category(), Category::FILE);
        assert_eq!(file.sections().len(), 1);
        assert_eq!(file.sections()[0].len(), 1);
        assert_eq!(
            file.sections()[0][0].command_type(),
            TypeId::of::<AfterOpen>()
        );
    }

    #[test]
    fn custom_categories_are_registered_once_and_keep_typed_identity() {
        let mut registry = Registry::default();
        registry
            .menu_category(Category::new::<Controls>("Controls"))
            .register::<ViewCommand>(
                Spec::new("View Action").placement(Placement::category(Category::VIEW)),
            )
            .register::<ControlsCommand>(
                Spec::new("Control Action")
                    .placement(Placement::category(Category::of::<Controls>())),
            )
            .register::<ToolsCommand>(
                Spec::new("Tool Action").placement(Placement::category(Category::TOOLS)),
            );

        let topology = registry.population().menu_topology(Platform::Windows);
        assert_eq!(
            topology
                .categories()
                .iter()
                .map(TopologyCategory::label)
                .collect::<Vec<_>>(),
            vec!["View", "Controls", "Tools"]
        );
        assert_eq!(
            topology.categories()[1].category(),
            Category::of::<Controls>()
        );
    }

    #[test]
    fn equal_visible_labels_never_merge_custom_category_identity() {
        let mut registry = Registry::default();
        registry
            .menu_category(Category::new::<Controls>("Same Label"))
            .menu_category(Category::new::<OtherControls>("Same Label"))
            .register::<ControlsCommand>(
                Spec::new("One").placement(Placement::category(Category::of::<Controls>())),
            )
            .register::<OtherControlsCommand>(
                Spec::new("Two").placement(Placement::category(Category::of::<OtherControls>())),
            );

        let topology = registry.population().menu_topology(Platform::Windows);
        assert_eq!(topology.categories().len(), 2);
        assert_ne!(
            topology.categories()[0].category(),
            topology.categories()[1].category()
        );
        assert_eq!(topology.categories()[0].label(), "Same Label");
        assert_eq!(topology.categories()[1].label(), "Same Label");
    }

    #[test]
    fn shortcut_visibility_is_platform_policy_with_an_explicit_override() {
        let registry = {
            let mut registry = Registry::default();
            registry.register::<DeleteCommand>(Spec::standard(Standard::Delete));
            registry
        };
        let windows = registry.population().menu_topology(Platform::Windows);
        let mac = registry.population().menu_topology(Platform::Mac);
        assert!(!windows.categories()[0].sections()[0][0].show_shortcut());
        assert!(mac.categories()[0].sections()[0][0].show_shortcut());

        let mut overridden = Registry::default();
        overridden
            .register::<DeleteCommand>(Spec::standard(Standard::Delete).show_menu_shortcut(true));
        assert!(
            overridden
                .population()
                .menu_topology(Platform::Windows)
                .categories()[0]
                .sections()[0][0]
                .show_shortcut()
        );
    }

    #[test]
    #[should_panic(expected = "menu placement requires a unit-argument command")]
    fn argument_bearing_commands_cannot_enter_static_topology() {
        let mut registry = Registry::default();
        registry.register::<ArgumentCommand>(
            Spec::new("Argument").placement(Placement::category(Category::FILE)),
        );
    }

    #[test]
    #[should_panic(expected = "conflicting metadata")]
    fn custom_category_identity_rejects_conflicting_declarations() {
        let mut registry = Registry::default();
        registry
            .menu_category(Category::new::<Controls>("Controls"))
            .menu_category(Category::new::<Controls>("Different"));
    }

    #[test]
    #[should_panic(expected = "unregistered custom category")]
    fn placement_cannot_smuggle_in_a_custom_category_label() {
        let mut registry = Registry::default();
        registry.register::<ControlsCommand>(
            Spec::new("Control")
                .placement(Placement::category(Category::new::<Controls>("Controls"))),
        );

        let _ = registry.population().menu_topology(Platform::Windows);
    }
}
