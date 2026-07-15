mod choice;
mod indicator;
mod slider;
mod text_area;
mod text_box;
pub(super) mod text_surface;
mod viewport_chrome;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::icon as icons;

use super::super::{composition, geometry, keymap, layout, overlay, theme::Theme, view};
use super::{
    Brush, Clip, Icon, Material, Offset, Outline, Pane, Quad, Scene, Shadow, Style, Text,
    TextAlign, TextStyle, TextWrap, Visuals,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    Base,
    Chrome,
    Floating,
}

#[derive(Clone, Copy)]
enum PropertySample {
    CommitBaseline,
    Current,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct RetainedStats {
    pub(super) commits_created: usize,
    pub(super) nodes_added: usize,
    pub(super) nodes_removed: usize,
    pub(super) node_paints: usize,
    pub(super) node_reuses: usize,
    pub(super) track_paints: usize,
    pub(super) chrome_paints: usize,
}

pub(super) struct Retained {
    theme: Option<Theme>,
    retained_nodes: HashSet<composition::tree::NodeId>,
    frames: HashMap<composition::tree::NodeId, CachedFrame>,
    tracks: HashMap<(composition::tree::NodeId, usize), CachedTrack>,
    chrome: HashMap<(composition::tree::NodeId, usize), CachedChrome>,
    nodes: HashMap<composition::tree::NodeId, Arc<super::Node>>,
    commits: HashMap<CommitKey, Arc<super::Commit>>,
    properties: HashMap<CommitKey, super::Properties>,
    overlays: Vec<overlay::Draft>,
    next_property_serial: super::PropertySerial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CommitKey {
    Base,
    Popup(composition::tree::NodeId),
}

#[derive(Clone)]
struct CachedFrame {
    key: layout::FrameSceneKey,
    visual: super::visual::NodeState,
    body: Scene,
    late: Vec<viewport_chrome::Projection>,
}

#[derive(Clone)]
struct CachedTrack {
    key: layout::table::Track,
    body: Scene,
}

#[derive(Clone)]
struct CachedChrome {
    key: layout::Chrome,
    visual: super::visual::Scrollbar,
    late: Vec<viewport_chrome::Projection>,
}

#[derive(Default)]
struct Seen {
    frames: HashSet<composition::tree::NodeId>,
    tracks: HashSet<(composition::tree::NodeId, usize)>,
    chrome: HashSet<(composition::tree::NodeId, usize)>,
}

#[derive(Default)]
struct RetainedWork {
    seen: Seen,
    stats: RetainedStats,
}

struct LayerFragments {
    frames: Vec<Fragment>,
    tracks: Vec<Fragment>,
    late: Vec<viewport_chrome::Projection>,
}

struct Fragment {
    owner: composition::tree::NodeId,
    clip: Option<Clip>,
    scrolls: Vec<composition::tree::NodeId>,
    body: Scene,
}

impl Default for Retained {
    fn default() -> Self {
        Self {
            theme: None,
            retained_nodes: HashSet::new(),
            frames: HashMap::new(),
            tracks: HashMap::new(),
            chrome: HashMap::new(),
            nodes: HashMap::new(),
            commits: HashMap::new(),
            properties: HashMap::new(),
            overlays: Vec::new(),
            next_property_serial: super::PropertySerial::INITIAL,
        }
    }
}

impl Retained {
    pub(super) fn paint(
        &mut self,
        layout: &layout::Layout,
        clear: super::Color,
        theme: &Theme,
        visuals: &Visuals,
        interaction: Option<&super::super::interaction::Interaction>,
    ) -> (
        Arc<super::Commit>,
        super::Properties,
        Vec<overlay::Draft>,
        RetainedStats,
    ) {
        if self.theme.as_ref() != Some(theme) {
            self.frames.clear();
            self.tracks.clear();
            self.chrome.clear();
            self.theme = Some(theme.clone());
        }

        let mut work = RetainedWork::default();
        let mut builder = commit_builder(layout, clear, None);
        for layer in [Layer::Base, Layer::Chrome] {
            let fragments = self.layer_fragments(layout, layer, None, theme, visuals, &mut work);
            append_layer_fragments(&mut builder, fragments);
        }
        let base_key = CommitKey::Base;
        let previous_base = self.commits.get(&base_key).cloned();
        let commit = builder
            .finish(previous_base.as_ref(), &mut self.nodes)
            .expect("retained base commit must satisfy the scene contract");
        work.stats.commits_created = usize::from(
            previous_base
                .as_ref()
                .is_none_or(|previous| !Arc::ptr_eq(previous, &commit)),
        );
        self.commits.insert(base_key, Arc::clone(&commit));
        let properties = self
            .property_snapshot(
                base_key,
                &commit,
                layout,
                visuals,
                interaction,
                PropertySample::CommitBaseline,
            )
            .expect("freshly painted base properties must satisfy their resident topology");

        let mut entries = Vec::new();
        let mut seen_commits = HashSet::from([base_key]);
        for panel in root_floating_panels(layout) {
            let id = match panel.interaction_id() {
                Some(id) => id,
                None => continue,
            };
            let mut builder = commit_builder(layout, clear, Some(panel));
            let fragments = self.layer_fragments(
                layout,
                Layer::Floating,
                Some(panel),
                theme,
                visuals,
                &mut work,
            );
            append_layer_fragments(&mut builder, fragments);
            let commit_key = CommitKey::Popup(panel.node_id());
            let previous_popup = self.commits.get(&commit_key).cloned();
            let popup_commit = builder
                .finish(previous_popup.as_ref(), &mut self.nodes)
                .expect("retained popup commit must satisfy the scene contract");
            work.stats.commits_created = work.stats.commits_created.saturating_add(usize::from(
                previous_popup
                    .as_ref()
                    .is_none_or(|previous| !Arc::ptr_eq(previous, &popup_commit)),
            ));
            let popup_properties = self
                .property_snapshot(
                    commit_key,
                    &popup_commit,
                    layout,
                    visuals,
                    interaction,
                    PropertySample::CommitBaseline,
                )
                .expect("freshly painted popup properties must satisfy their resident topology");
            let panel_scene = popup_commit
                .compatibility_scene(&popup_properties)
                .expect("the public popup scene snapshot receives its own property state");
            self.commits.insert(commit_key, Arc::clone(&popup_commit));
            seen_commits.insert(commit_key);
            if !panel_scene.is_empty() {
                entries.push(
                    overlay::Draft::retained(
                        id,
                        panel.rect(),
                        popup_commit,
                        popup_properties,
                        panel_scene,
                    )
                    .prefer(overlay::Preference::NativePopup)
                    .placement(panel.popup_placement())
                    .context_fingerprint(panel.popup_context())
                    .popup_material_preference(popup_material_preference(panel))
                    .popup_border(theme.floating_panel().border())
                    .text_caret_rect(text_caret_rect_for_panel(layout, panel))
                    .accepts_input(panel.panel_accepts_input())
                    .force_group_at_full_opacity(panel.force_overlay_group()),
                );
            }
        }

        self.frames.retain(|id, _| work.seen.frames.contains(id));
        self.tracks.retain(|key, _| work.seen.tracks.contains(key));
        self.chrome.retain(|key, _| work.seen.chrome.contains(key));
        self.nodes.retain(|id, _| work.seen.frames.contains(id));
        self.commits.retain(|key, _| seen_commits.contains(key));
        self.properties.retain(|key, _| seen_commits.contains(key));
        work.stats.nodes_added = work.seen.frames.difference(&self.retained_nodes).count();
        work.stats.nodes_removed = self.retained_nodes.difference(&work.seen.frames).count();
        self.retained_nodes = work.seen.frames;
        self.overlays = entries.clone();

        (commit, properties, entries, work.stats)
    }

    pub(super) fn tick_properties(
        &mut self,
        layout: &layout::Layout,
        visuals: &Visuals,
        interaction: Option<&super::super::interaction::Interaction>,
    ) -> Option<(Arc<super::Commit>, super::Properties, Vec<overlay::Draft>)> {
        let base_key = CommitKey::Base;
        let commit = self.commits.get(&base_key).cloned()?;
        let properties = self
            .property_snapshot(
                base_key,
                &commit,
                layout,
                visuals,
                interaction,
                PropertySample::Current,
            )
            .ok()?;
        let keyed =
            self.overlays
                .iter()
                .cloned()
                .filter_map(|draft| {
                    let commit = Arc::clone(draft.commit());
                    let key = self.commits.iter().find_map(|(key, candidate)| {
                        Arc::ptr_eq(candidate, &commit).then_some(*key)
                    })?;
                    Some((draft, key, commit))
                })
                .collect::<Vec<_>>();
        let overlays = keyed
            .into_iter()
            .map(|(draft, key, popup_commit)| {
                let popup_properties = self
                    .property_snapshot(
                        key,
                        &popup_commit,
                        layout,
                        visuals,
                        interaction,
                        PropertySample::Current,
                    )
                    .ok()?;
                let scene = popup_commit
                    .compatibility_scene(&popup_properties)
                    .expect("cached popup properties remain compatible with their commit");
                Some(draft.with_property_state(popup_properties, scene))
            })
            .collect::<Option<Vec<_>>>()?;
        self.overlays = overlays.clone();
        Some((commit, properties, overlays))
    }

    fn property_snapshot(
        &mut self,
        key: CommitKey,
        commit: &super::Commit,
        layout: &layout::Layout,
        visuals: &Visuals,
        interaction: Option<&super::super::interaction::Interaction>,
        sample: PropertySample,
    ) -> Result<super::Properties, super::commit::ContractError> {
        let mut values = layout
            .scroll_projections()
            .iter()
            .filter_map(|projection| {
                let declaration = commit
                    .nodes()
                    .iter()
                    .find(|node| node.id() == projection.node())?
                    .scroll()?;
                let offset = match sample {
                    PropertySample::CommitBaseline => declaration.baseline(),
                    PropertySample::Current => interaction
                        .map(super::super::interaction::Interaction::scroll)
                        .map(|scroll| scroll.offset(projection.target()))
                        .map(|offset| projection.viewport().resolve(offset))
                        .unwrap_or_else(|| declaration.baseline()),
                };
                Some(super::PropertyValue::ScrollOffset {
                    node: projection.node(),
                    x: offset.x() as f32,
                    y: offset.y() as f32,
                })
            })
            .collect::<Vec<_>>();
        let mut scrollbar_nodes = HashSet::new();
        for chrome in layout.chrome() {
            let node = chrome.owner();
            if !scrollbar_nodes.insert(node)
                || !commit.nodes().iter().any(|candidate| {
                    candidate.id() == node && candidate.declares(super::PropertyKind::Scrollbar)
                })
            {
                continue;
            }
            let (opacity, thickness) = layout
                .chrome()
                .iter()
                .filter(|candidate| candidate.owner() == node)
                .map(|candidate| visuals.scrollbar(candidate.target()))
                .fold((0.0_f32, 1.0_f32), |(opacity, thickness), visual| {
                    (
                        opacity.max(visual.opacity()),
                        thickness.max(visual.thickness() as f32),
                    )
                });
            values.push(super::PropertyValue::Scrollbar {
                node,
                opacity,
                thickness,
            });
        }
        let previous = self.properties.get(&key);
        let (properties, advanced) =
            super::Properties::snapshot(commit, previous, self.next_property_serial, values)?;
        if advanced {
            self.next_property_serial = self.next_property_serial.next();
        }
        self.properties.insert(key, properties.clone());
        Ok(properties)
    }

    fn layer_fragments(
        &mut self,
        layout: &layout::Layout,
        layer: Layer,
        panel: Option<&layout::Frame>,
        theme: &Theme,
        visuals: &Visuals,
        work: &mut RetainedWork,
    ) -> LayerFragments {
        let RetainedWork { seen, stats } = work;
        let mut frames = Vec::new();
        let mut tracks = Vec::new();
        let mut late = Vec::new();

        for frame in layout.frames().iter().filter(|frame| {
            layer_for(frame) == layer
                && panel.is_none_or(|panel| frame_belongs_to_panel(frame, panel))
        }) {
            seen.frames.insert(frame.node_id());
            let key = frame.scene_key();
            let visual = visuals.node_state(frame.target());
            let cached = self
                .frames
                .get(&frame.node_id())
                .filter(|cached| cached.key == key && cached.visual == visual);
            let cached = if let Some(cached) = cached {
                stats.node_reuses = stats.node_reuses.saturating_add(1);
                cached.clone()
            } else {
                let mut body = Scene::new_with_clear(layout.size(), super::Color::rgba(0, 0, 0, 0));
                let mut frame_late = Vec::new();
                let clip = frame
                    .clip()
                    .map(|clip| Clip::new(clip.rect()).with_rounding(clip.rounding()));
                paint_frame(frame, &mut body, theme, visuals, &mut frame_late, clip);
                let cached = CachedFrame {
                    key,
                    visual,
                    body,
                    late: frame_late,
                };
                self.frames.insert(frame.node_id(), cached.clone());
                stats.node_paints = stats.node_paints.saturating_add(1);
                cached
            };
            let clip = frame
                .clip()
                .map(|clip| Clip::new(clip.rect()).with_rounding(clip.rounding()));
            frames.push(Fragment {
                owner: frame.node_id(),
                clip,
                scrolls: layout.scroll_ancestry(frame.node_id()).to_vec(),
                body: cached.body,
            });
            late.extend(cached.late);
        }

        let mut track_ordinals = HashMap::new();
        for track in layout.table_tracks().iter().filter(|track| {
            table_track_layer_for(track) == layer
                && panel.is_none_or(|panel| table_track_belongs_to_panel(layout, track, panel))
        }) {
            let ordinal = track_ordinals.entry(track.table_node()).or_insert(0_usize);
            let cache_key = (track.table_node(), *ordinal);
            *ordinal = ordinal.saturating_add(1);
            seen.tracks.insert(cache_key);
            let cached = self
                .tracks
                .get(&cache_key)
                .filter(|cached| cached.key == *track)
                .cloned()
                .unwrap_or_else(|| {
                    let mut body =
                        Scene::new_with_clear(layout.size(), super::Color::rgba(0, 0, 0, 0));
                    paint_table_track(track, &mut body, theme);
                    let cached = CachedTrack {
                        key: track.clone(),
                        body,
                    };
                    self.tracks.insert(cache_key, cached.clone());
                    stats.track_paints = stats.track_paints.saturating_add(1);
                    cached
                });
            let clip = track
                .clip()
                .map(|clip| Clip::new(clip.rect()).with_rounding(clip.rounding()));
            tracks.push(Fragment {
                owner: track.owner_node(),
                clip,
                scrolls: layout.scroll_ancestry(track.owner_node()).to_vec(),
                body: cached.body,
            });
        }

        let mut chrome_ordinals = HashMap::new();
        for chrome in layout.chrome().iter().filter(|chrome| {
            chrome_layer_for(layout, chrome) == layer
                && panel.is_none_or(|panel| chrome_belongs_to_panel(layout, chrome, panel))
        }) {
            let ordinal = chrome_ordinals.entry(chrome.owner()).or_insert(0_usize);
            let cache_key = (chrome.owner(), *ordinal);
            *ordinal = ordinal.saturating_add(1);
            seen.chrome.insert(cache_key);
            let visual = visuals.scrollbar(chrome.target());
            let cached = self
                .chrome
                .get(&cache_key)
                .filter(|cached| cached.key == *chrome && cached.visual == visual)
                .cloned()
                .unwrap_or_else(|| {
                    let mut chrome_late = Vec::new();
                    project_chrome(chrome, theme, visuals, &mut chrome_late);
                    let cached = CachedChrome {
                        key: chrome.clone(),
                        visual,
                        late: chrome_late,
                    };
                    self.chrome.insert(cache_key, cached.clone());
                    stats.chrome_paints = stats.chrome_paints.saturating_add(1);
                    cached
                });
            late.extend(cached.late);
        }

        late.sort_by_key(viewport_chrome::Projection::layer_order);
        LayerFragments {
            frames,
            tracks,
            late,
        }
    }
}

fn commit_builder(
    layout: &layout::Layout,
    clear: super::Color,
    panel: Option<&layout::Frame>,
) -> super::CommitBuilder {
    let frames = layout
        .frames()
        .iter()
        .filter(|frame| match panel {
            Some(panel) => {
                layer_for(frame) == Layer::Floating && frame_belongs_to_panel(frame, panel)
            }
            None => layer_for(frame) != Layer::Floating,
        })
        .collect::<Vec<_>>();
    let included = frames
        .iter()
        .map(|frame| frame.node_id())
        .collect::<HashSet<_>>();
    let mut builder = super::CommitBuilder::new(layout.size(), clear);
    for frame in frames {
        builder.register(
            frame.node_id(),
            frame.parent().filter(|parent| included.contains(parent)),
            frame.content_revision(),
            frame.rect(),
        );
    }
    for projection in layout.scroll_projections() {
        let Some(declaration) = super::ScrollDeclaration::new(
            projection.viewport().visible_content(),
            projection.resident_bounds(),
            projection.viewport().resolved_scroll(),
        ) else {
            continue;
        };
        builder.declare_scroll(projection.node(), declaration);
    }
    builder
}

fn append_layer_fragments(target: &mut super::CommitBuilder, fragments: LayerFragments) {
    append_scoped_fragments(target, fragments.frames);
    append_scoped_fragments(target, fragments.tracks);
    let mut late = fragments.late;
    late.sort_by_key(viewport_chrome::Projection::layer_order);
    for projection in late {
        projection.append_to_commit(target);
    }
}

fn append_scoped_fragments(target: &mut super::CommitBuilder, fragments: Vec<Fragment>) {
    let mut active_clip = None;
    let mut active_scrolls = Vec::new();
    for fragment in fragments {
        if active_scrolls != fragment.scrolls {
            switch_commit_clip_scope(target, &mut active_clip, None);
            let shared = active_scrolls
                .iter()
                .zip(&fragment.scrolls)
                .take_while(|(left, right)| left == right)
                .count();
            for _ in shared..active_scrolls.len() {
                target.pop_scroll();
            }
            active_scrolls.truncate(shared);
            for scroll in fragment.scrolls.iter().skip(shared) {
                target.push_scroll(*scroll);
                active_scrolls.push(*scroll);
            }
        }
        switch_commit_clip_scope(target, &mut active_clip, fragment.clip);
        target.append_fragment(fragment.owner, &fragment.body);
    }
    switch_commit_clip_scope(target, &mut active_clip, None);
    for _ in 0..active_scrolls.len() {
        target.pop_scroll();
    }
}

fn switch_commit_clip_scope(
    target: &mut super::CommitBuilder,
    active: &mut Option<Clip>,
    next: Option<Clip>,
) {
    if *active == next {
        return;
    }
    if active.take().is_some() {
        target.pop_clip();
    }
    if let Some(clip) = next {
        target.push_clip(clip);
        *active = Some(clip);
    }
}

#[cfg(test)]
pub(super) fn paint_layout_with_theme(
    layout: &layout::Layout,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
) -> Vec<overlay::Draft> {
    for layer in [Layer::Base, Layer::Chrome] {
        let mut late_chrome = Vec::new();

        paint_frames_with_shared_clip(
            layout
                .frames()
                .iter()
                .filter(|frame| layer_for(frame) == layer),
            scene,
            theme,
            visuals,
            &mut late_chrome,
        );

        paint_table_tracks_with_shared_clip(
            layout
                .table_tracks()
                .iter()
                .filter(|track| table_track_layer_for(track) == layer),
            scene,
            theme,
        );

        for chrome in layout
            .chrome()
            .iter()
            .filter(|chrome| chrome_layer_for(layout, chrome) == layer)
        {
            project_chrome(chrome, theme, visuals, &mut late_chrome);
        }

        viewport_chrome::paint(scene, late_chrome);
    }

    paint_overlay_entries(layout, scene.clear(), theme, visuals)
}

#[cfg(test)]
fn paint_overlay_entries(
    layout: &layout::Layout,
    clear: super::Color,
    theme: &Theme,
    visuals: &Visuals,
) -> Vec<overlay::Draft> {
    root_floating_panels(layout)
        .into_iter()
        .filter_map(|panel| {
            let id = panel.interaction_id()?;
            let mut scene = Scene::new_with_clear(layout.size(), clear);
            let mut late_chrome = Vec::new();

            paint_frames_with_shared_clip(
                layout
                    .frames()
                    .iter()
                    .filter(|frame| layer_for(frame) == Layer::Floating)
                    .filter(|frame| frame_belongs_to_panel(frame, panel)),
                &mut scene,
                theme,
                visuals,
                &mut late_chrome,
            );

            paint_table_tracks_with_shared_clip(
                layout
                    .table_tracks()
                    .iter()
                    .filter(|track| table_track_belongs_to_panel(layout, track, panel)),
                &mut scene,
                theme,
            );

            for chrome in layout
                .chrome()
                .iter()
                .filter(|chrome| chrome_belongs_to_panel(layout, chrome, panel))
            {
                project_chrome(chrome, theme, visuals, &mut late_chrome);
            }

            viewport_chrome::paint(&mut scene, late_chrome);

            (!scene.is_empty()).then(|| {
                overlay::Draft::new(id, panel.rect(), scene)
                    .prefer(overlay::Preference::NativePopup)
                    .placement(panel.popup_placement())
                    .context_fingerprint(panel.popup_context())
                    .popup_material_preference(popup_material_preference(panel))
                    .popup_border(theme.floating_panel().border())
                    .text_caret_rect(text_caret_rect_for_panel(layout, panel))
                    .accepts_input(panel.panel_accepts_input())
                    .force_group_at_full_opacity(panel.force_overlay_group())
            })
        })
        .collect()
}

fn frame_belongs_to_panel(frame: &layout::Frame, panel: &layout::Frame) -> bool {
    frame.node_id() == panel.node_id() || frame.is_descendant_of(panel)
}

fn text_caret_rect_for_panel(
    layout: &layout::Layout,
    panel: &layout::Frame,
) -> Option<geometry::Rect> {
    layout
        .frames()
        .iter()
        .filter(|frame| frame_belongs_to_panel(frame, panel))
        .find_map(layout::Frame::text_caret_rect)
}

fn popup_material_preference(panel: &layout::Frame) -> overlay::PopupMaterialPreference {
    match panel.native_popup_material_preference() {
        view::NativePopupMaterialPreference::System => overlay::PopupMaterialPreference::System,
        view::NativePopupMaterialPreference::OpaqueFallback => {
            overlay::PopupMaterialPreference::OpaqueFallback
        }
        view::NativePopupMaterialPreference::NoAccent => overlay::PopupMaterialPreference::NoAccent,
    }
}

fn root_floating_panels(layout: &layout::Layout) -> Vec<&layout::Frame> {
    layout.root_floating_panels().collect()
}

fn chrome_belongs_to_panel(
    layout: &layout::Layout,
    chrome: &layout::Chrome,
    panel: &layout::Frame,
) -> bool {
    layout
        .frames()
        .iter()
        .rev()
        .find(|frame| frame.node_id() == chrome.owner())
        .is_some_and(|frame| frame.node_id() == panel.node_id() || frame.is_descendant_of(panel))
}

fn table_track_belongs_to_panel(
    layout: &layout::Layout,
    track: &layout::table::Track,
    panel: &layout::Frame,
) -> bool {
    layout
        .frames()
        .iter()
        .find(|frame| frame.node_id() == track.table_node())
        .is_some_and(|frame| frame_belongs_to_panel(frame, panel))
}

fn table_track_layer_for(track: &layout::table::Track) -> Layer {
    if track.is_floating_layer() {
        Layer::Floating
    } else {
        Layer::Base
    }
}

fn paint_table_track(track: &layout::table::Track, scene: &mut Scene, theme: &Theme) {
    let rule = match track.axis() {
        layout::table::Axis::Column => {
            super::Rule::vertical(track.rule_rect(), theme.menu().separator, 1)
        }
        layout::table::Axis::Row => {
            super::Rule::horizontal(track.rule_rect(), theme.menu().separator, 1)
        }
    };
    scene.push_rule(rule);
}

#[cfg(test)]
fn paint_frames_with_shared_clip<'a>(
    frames: impl IntoIterator<Item = &'a layout::Frame>,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
    late_chrome: &mut Vec<viewport_chrome::Projection>,
) {
    let mut active_clip = None;
    for frame in frames {
        let clip = frame
            .clip()
            .map(|clip| Clip::new(clip.rect()).with_rounding(clip.rounding()));
        switch_clip_scope(scene, &mut active_clip, clip);
        paint_frame(frame, scene, theme, visuals, late_chrome, clip);
    }
    switch_clip_scope(scene, &mut active_clip, None);
}

#[cfg(test)]
fn paint_table_tracks_with_shared_clip<'a>(
    tracks: impl IntoIterator<Item = &'a layout::table::Track>,
    scene: &mut Scene,
    theme: &Theme,
) {
    let mut active_clip = None;
    for track in tracks {
        let clip = track
            .clip()
            .map(|clip| Clip::new(clip.rect()).with_rounding(clip.rounding()));
        switch_clip_scope(scene, &mut active_clip, clip);
        paint_table_track(track, scene, theme);
    }
    switch_clip_scope(scene, &mut active_clip, None);
}

#[cfg(test)]
fn switch_clip_scope(scene: &mut Scene, active: &mut Option<Clip>, next: Option<Clip>) {
    if *active == next {
        return;
    }
    if active.take().is_some() {
        scene.pop_clip();
    }
    if let Some(clip) = next {
        scene.push_clip(clip);
        *active = Some(clip);
    }
}

#[cfg(test)]
mod clip_scope_tests {
    use super::*;

    #[test]
    fn identical_contiguous_members_share_one_clip_scope() {
        let mut scene = Scene::new(geometry::Size::new(100, 80));
        let clip = Clip::new(geometry::Rect::new(0, 0, 50, 40));
        let mut active = None;

        switch_clip_scope(&mut scene, &mut active, Some(clip));
        switch_clip_scope(&mut scene, &mut active, Some(clip));
        switch_clip_scope(&mut scene, &mut active, None);

        assert_eq!(
            scene
                .primitives()
                .iter()
                .filter(|item| matches!(item, super::super::Primitive::Clip(_)))
                .count(),
            1
        );
        assert_eq!(
            scene
                .primitives()
                .iter()
                .filter(|item| matches!(item, super::super::Primitive::PopClip))
                .count(),
            1
        );
    }

    #[test]
    fn delete_shortcut_icon_resolves_to_a_real_font_glyph() {
        assert!(
            shortcut_icon(keymap::ShortcutIcon::Delete)
                .glyph()
                .is_some(),
            "Delete shortcut chrome must use the icon font rather than a Unicode text run"
        );
    }
}

fn paint_frame(
    frame: &layout::Frame,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
    late_chrome: &mut Vec<viewport_chrome::Projection>,
    clip: Option<Clip>,
) {
    if is_floating_panel_role(frame.role()) {
        paint_floating_panel_shadow(frame, scene, theme);
    }

    let rounding = role_rounding(frame, theme);
    if is_floating_panel_role(frame.role()) {
        paint_floating_panel_material(frame, scene, theme, clip);
        paint_floating_panel_border(frame, scene, theme);
        paint_auxiliary_panel_icon(frame, scene, theme);
    } else if let Some(fill) = frame.background() {
        paint_brush_quad(scene, frame.rect(), fill, rounding);
    } else if let Some(fill) = role_fill(frame, theme) {
        scene.push_quad(Quad::new(frame.rect(), fill).with_rounding(rounding));
    }

    if let Some(tint) = visual_tint_for(frame, theme, visuals) {
        scene.push_quad(
            Quad::new(visual_tint_rect_for(frame), tint)
                .with_rounding(visual_tint_rounding_for(frame, rounding, theme)),
        );
    }

    if frame.table_row().is_some_and(|row| row.index() % 2 == 1) {
        scene.push_quad(Quad::new(frame.rect(), theme.table().alternate_row_tint));
    }
    match frame.role() {
        view::Role::Binding if frame.is_menu_row() => paint_menu_row(frame, scene, theme),
        _ if frame.is_palette_row() => paint_palette_row(frame, scene, theme),
        view::Role::Separator => {
            paint_menu_separator(frame, scene, theme);
        }
        view::Role::SectionHeader => {
            paint_section_header(frame, scene, theme);
        }
        view::Role::Checkbox | view::Role::Radio => {
            choice::paint(frame, scene, theme, visuals);
        }
        view::Role::Slider => {
            slider::paint(frame, scene, theme, visuals);
        }
        view::Role::TextArea => {
            text_area::paint(frame, scene, theme, visuals);
        }
        view::Role::TextBox => {
            if frame.text_area_layout().is_some() {
                text_area::paint(frame, scene, theme, visuals);
            } else {
                text_box::paint_selection(frame, scene, theme);
            }
        }
        _ => {}
    }

    if frame.is_menu_row()
        || frame.is_palette_row()
        || frame.role() == view::Role::Separator
        || frame.role() == view::Role::SectionHeader
        || is_floating_panel_role(frame.role())
    {
        // Floating panes and menu rows paint their own content.
    } else if frame.role() == view::Role::TextBox && frame.text_area_layout().is_some() {
        // Inactive table TextBoxes use their column display projection above.
    } else if frame.role() == view::Role::TextBox && text_box::paint_text(frame, scene) {
        // TextBox contents use the shaped field viewport so glyphs, selection,
        // caret, and horizontal scroll share one coordinate system.
    } else if frame.role() == view::Role::TextArea {
        // TextArea contents, including selectable table text, use their one
        // resolved shaped viewport above.
    } else if let Some(value) = text_for(frame) {
        scene.push_text(
            Text::new(
                text_rect_for(frame, theme),
                value,
                text_color_for(frame, theme),
                text_wrap_for(frame),
            )
            .with_style(text_style_for(frame, theme))
            .with_align(text_align_for(frame))
            .with_overflow(frame.world_text_overflow().unwrap_or_default()),
        );
    }

    paint_table_sort_indicator(frame, scene, theme);

    if frame.role() == view::Role::TextBox {
        if let (Some(rect), Some(hint)) =
            (frame.input_indicator_rect(), frame.input_indicator_hint())
        {
            indicator::paint(rect, hint, scene, theme);
        }
        text_box::paint_caret(frame, scene, theme, visuals);
    }

    if let Some(color) = focus_outline_color_for(frame, theme) {
        late_chrome.push(viewport_chrome::Projection::outline(
            frame.node_id(),
            viewport_chrome::Scope::new(clip),
            Outline::new(focus_outline_rect_for(frame), color)
                .with_width(theme.focus().width as f32)
                .with_offset(theme.focus().offset)
                .with_rounding(focus_outline_rounding_for(frame, rounding, theme)),
        ));
    } else if let Some(color) = outline_color_for(frame, theme) {
        scene.push_outline(
            Outline::new(focus_outline_rect_for(frame), color)
                .with_width(theme.focus().width as f32)
                .with_rounding(focus_outline_rounding_for(frame, rounding, theme)),
        );
    }
}

fn layer_for(frame: &layout::Frame) -> Layer {
    if frame.is_floating_layer() {
        return Layer::Floating;
    }

    match frame.role() {
        view::Role::MenuBar | view::Role::Menu => Layer::Chrome,
        view::Role::FloatingPanel | view::Role::Separator => Layer::Floating,
        view::Role::Binding if frame.is_menu_row() => Layer::Floating,
        _ if frame.is_palette_row() => Layer::Floating,
        _ => Layer::Base,
    }
}

fn role_fill(frame: &layout::Frame, theme: &Theme) -> Option<super::Color> {
    if let Some(part) = frame.table_part()
        && part != view::TablePart::Action
    {
        return match part {
            view::TablePart::HeaderBand
            | view::TablePart::Header
            | view::TablePart::HeaderControl => visible_fill(theme.table().header_background),
            view::TablePart::Cell | view::TablePart::PassiveToggle | view::TablePart::Toggle => {
                visible_fill(theme.table().cell_background)
            }
            view::TablePart::Action => unreachable!("table actions retain control chrome"),
        };
    }

    match frame.role() {
        view::Role::Root => Some(theme.surfaces().root),
        view::Role::MenuBar => Some(theme.menu().bar_background),
        view::Role::Menu => visible_fill(theme.menu().title_background),
        view::Role::FloatingPanel => None,
        view::Role::Binding if frame.is_menu_row() => visible_fill(theme.menu().row_background),
        view::Role::Label if frame.is_palette_row() => visible_fill(theme.menu().row_background),
        view::Role::Binding => visible_fill(if frame.is_enabled() {
            theme.control().background
        } else {
            theme.control().disabled_background
        }),
        view::Role::Separator => None,
        view::Role::TextArea => Some(theme.text_input().area_background),
        view::Role::Button => visible_fill(theme.control().button_background),
        view::Role::Checkbox | view::Role::Radio => visible_fill(theme.choice().background),
        view::Role::Slider => visible_fill(theme.slider().background),
        view::Role::TextBox => Some(theme.text_input().field_background),
        view::Role::Scroll | view::Role::VirtualList => None,
        view::Role::Panel => visible_fill(theme.surfaces().panel),
        view::Role::SectionHeader | view::Role::Label | view::Role::Stack | view::Role::Table => {
            None
        }
    }
}

fn visible_fill(color: super::Color) -> Option<super::Color> {
    (color.channels().3 > 0).then_some(color)
}

fn project_chrome(
    chrome: &layout::Chrome,
    theme: &Theme,
    visuals: &Visuals,
    late_chrome: &mut Vec<viewport_chrome::Projection>,
) {
    project_scrollbar(chrome, theme, visuals, late_chrome);
}

fn project_scrollbar(
    chrome: &layout::Chrome,
    theme: &Theme,
    visuals: &Visuals,
    late_chrome: &mut Vec<viewport_chrome::Projection>,
) {
    let theme_scrollbar = theme.scrollbar();
    let visual = visuals.scrollbar(chrome.target());
    let base_thickness = match theme_scrollbar.metrics.policy {
        crate::theme::ScrollbarPolicy::GutterAlways => theme_scrollbar.metrics.thickness.max(1),
        crate::theme::ScrollbarPolicy::OverlayAuto => {
            theme_scrollbar.appearance.overlay_thickness.max(1)
        }
    };
    let maximum_thickness = theme_scrollbar
        .appearance
        .hover_thickness
        .max(base_thickness);
    let track = chrome.track_with_thickness(base_thickness);
    let thumb = chrome.thumb_with_thickness(base_thickness);
    let axis = chrome.axis();
    let (edge, track_start, track_extent, thumb_start, thumb_extent) = match axis {
        crate::interaction::ScrollbarAxis::Vertical => (
            track.right(),
            track.y(),
            track.height(),
            thumb.y(),
            thumb.height(),
        ),
        crate::interaction::ScrollbarAxis::Horizontal => (
            track.bottom(),
            track.x(),
            track.width(),
            thumb.x(),
            thumb.width(),
        ),
    };
    let track_projection = super::ContentProjection::ScrollbarTrack {
        axis,
        edge,
        base_thickness,
        maximum_thickness,
    };
    let thumb_projection = super::ContentProjection::ScrollbarThumb {
        axis,
        edge,
        base_thickness,
        maximum_thickness,
        baseline_start: thumb_start,
        baseline_extent: thumb_extent,
        baseline_position: thumb_start.saturating_sub(track_start),
        travel: track_extent.saturating_sub(thumb_extent),
        maximum_offset: chrome.maximum_offset(),
    };
    let appearance = theme_scrollbar.appearance;
    let scope = viewport_chrome::Scope::new(
        chrome
            .clips()
            .iter()
            .map(|clip| Clip::new(clip.rect()).with_rounding(clip.rounding())),
    );
    let mut projection = viewport_chrome::Projection::scrollbar(chrome.owner(), scope);

    if appearance.track.channels().3 > 0 {
        projection.push_scrollbar_quad(
            Quad::new(track, appearance.track).with_rounding(appearance.rounding),
            track_projection,
        );
    }
    if appearance.thumb.channels().3 > 0 {
        projection.push_scrollbar_quad(
            Quad::new(thumb, appearance.thumb).with_rounding(appearance.rounding),
            thumb_projection,
        );
    }

    let tint = if visual.pressed() {
        appearance.thumb_pressed_tint
    } else if visual.hovered() {
        appearance.thumb_hover_tint
    } else {
        super::Color::rgba(0, 0, 0, 0)
    };
    if tint.channels().3 > 0 {
        projection.push_scrollbar_quad(
            Quad::new(thumb, tint).with_rounding(appearance.rounding),
            thumb_projection,
        );
    }
    if !projection.is_empty() {
        late_chrome.push(projection);
    }
}

fn chrome_layer_for(layout: &layout::Layout, chrome: &layout::Chrome) -> Layer {
    layout
        .frames()
        .iter()
        .rev()
        .find(|frame| frame.node_id() == chrome.owner())
        .map(layer_for)
        .unwrap_or(Layer::Base)
}

fn role_rounding(frame: &layout::Frame, theme: &Theme) -> super::Rounding {
    if frame
        .table_part()
        .is_some_and(|part| part != view::TablePart::Action)
    {
        return super::Rounding::none();
    }

    match frame.role() {
        view::Role::FloatingPanel => theme.floating_panel().rounding,
        view::Role::Menu => theme.control().rounding,
        view::Role::Binding if frame.is_menu_row() => theme.control().rounding,
        _ if frame.is_palette_row() => theme.control().rounding,
        view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox
        | view::Role::Panel => theme.control().rounding,
        _ => super::Rounding::none(),
    }
}

fn is_floating_panel_role(role: view::Role) -> bool {
    role == view::Role::FloatingPanel
}

fn paint_brush_quad(
    scene: &mut Scene,
    rect: geometry::Rect,
    fill: Brush,
    rounding: super::Rounding,
) {
    if fill.is_visible() {
        scene.push_quad(Quad::styled(rect, Style::filled_with(fill)).with_rounding(rounding));
    }
}

fn paint_floating_panel_shadow(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let panel = theme.floating_panel();
    scene.push_shadow(
        Shadow::new(
            frame.rect(),
            panel.shadow,
            panel.shadow_blur,
            panel.shadow_spread,
            Offset::new(0.0, panel.shadow_offset_y),
        )
        .with_rounding(role_rounding(frame, theme)),
    );
}

fn paint_floating_panel_material(
    frame: &layout::Frame,
    scene: &mut Scene,
    theme: &Theme,
    clip: Option<Clip>,
) {
    let panel = theme.floating_panel();
    match &panel.material {
        Material::Solid(brush) => {
            paint_brush_quad(scene, frame.rect(), *brush, role_rounding(frame, theme))
        }
        Material::Glass(_) => scene.push_material_pane(
            frame.node_id(),
            Pane::new(frame.rect(), panel.material.clone())
                .with_rounding(role_rounding(frame, theme)),
            clip,
        ),
    }
}

fn paint_floating_panel_border(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let panel = theme.floating_panel();
    if panel.border.channels().3 == 0 {
        return;
    }

    scene.push_outline(
        Outline::new(frame.rect(), panel.border)
            .with_width(1.0)
            .with_rounding(role_rounding(frame, theme)),
    );
}

fn paint_auxiliary_panel_icon(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(hint) = frame.auxiliary_hint() else {
        return;
    };
    if hint.icon().is_none() {
        return;
    }
    let auxiliary = theme.auxiliary_panel();
    let panel = theme.floating_panel();
    let extent = auxiliary.icon_extent.max(1).min(
        frame
            .rect()
            .height()
            .saturating_sub(panel.padding.max(0).saturating_mul(2)),
    );
    let rect = geometry::Rect::new(
        frame.rect().x().saturating_add(panel.padding.max(0)),
        frame
            .rect()
            .y()
            .saturating_add(frame.rect().height().saturating_sub(extent) / 2),
        extent,
        extent,
    );
    indicator::paint(rect, hint, scene, theme);
}

fn paint_menu_row(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let parts = layout::menu_row_parts(frame.rect(), frame.shortcut_width(), theme);
    let color = text_color_for(frame, theme);

    if frame.checked() == Some(true) {
        scene.push_icon(Icon::new(
            parts.glyph,
            icons::Icon::phosphor(icons::Id::new("check")).with_style(icons::Style::Bold),
            color,
            layout::control_content_extent(theme.menu().row_height, theme) as f32,
        ));
    }

    if let Some(label) = frame.label_text() {
        scene.push_text(
            Text::new(parts.label, label, color, TextWrap::None)
                .with_style(text_style_for(frame, theme))
                .with_align(TextAlign::Start),
        );
    }

    paint_shortcut(frame, parts.shortcut, scene, theme);
}

fn paint_menu_separator(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let parts = layout::menu_row_parts(frame.rect(), frame.shortcut_width(), theme);

    scene.push_rule(super::Rule::horizontal(
        parts.separator,
        theme.menu().separator,
        1,
    ));
}

fn paint_palette_row(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let rect = frame.rect();
    let parts = layout::palette_row_parts(rect, frame.shortcut_width(), theme);
    let color = theme.text().primary;

    if let Some(label) = frame.label_text() {
        scene.push_text(
            Text::new(parts.label, label, color, TextWrap::None)
                .with_style(text_style_for(frame, theme))
                .with_align(TextAlign::Start),
        );
    }

    paint_shortcut(frame, parts.shortcut, scene, theme);
}

fn shortcut_text_color(theme: &Theme) -> super::Color {
    theme.text().muted
}

fn paint_shortcut(frame: &layout::Frame, area: geometry::Rect, scene: &mut Scene, theme: &Theme) {
    let Some(parts) = frame.shortcut_display() else {
        return;
    };
    if parts.is_empty() {
        return;
    }

    let mut x = area.right().saturating_sub(frame.shortcut_content_width());
    let color = shortcut_text_color(theme);
    let style = scene_text_style(layout::shortcut_text_style(theme));
    let gap = layout::shortcut_run_gap(theme);

    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            x = x.saturating_add(gap);
        }

        let width = part.width();
        let rect = geometry::Rect::new(x, area.y(), width, area.height());
        match part.run() {
            keymap::ShortcutRun::Text(value) => {
                scene.push_text(
                    Text::new(rect, value, color, TextWrap::None)
                        .with_style(style)
                        .with_align(TextAlign::Start),
                );
            }
            keymap::ShortcutRun::Icon(icon) => {
                let extent = width.min(area.height()).max(1);
                let icon_rect = geometry::Rect::new(
                    x,
                    area.y()
                        .saturating_add((area.height().saturating_sub(extent)) / 2),
                    extent,
                    extent,
                );
                scene.push_icon(Icon::new(
                    icon_rect,
                    shortcut_icon(*icon),
                    color,
                    extent as f32,
                ));
            }
        }

        x = x.saturating_add(width);
    }
}

fn shortcut_icon(icon: keymap::ShortcutIcon) -> icons::Icon {
    let id = match icon {
        keymap::ShortcutIcon::Control => "caret-up",
        keymap::ShortcutIcon::Shift => "arrow-fat-up",
        keymap::ShortcutIcon::Alt | keymap::ShortcutIcon::Option => "option",
        keymap::ShortcutIcon::Command => "command",
        keymap::ShortcutIcon::Delete => "backspace",
    };

    icons::Icon::phosphor(icons::Id::new(id))
}

fn paint_section_header(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(label) = frame.label_text() else {
        return;
    };

    scene.push_text(
        Text::new(
            frame.rect(),
            layout::section_header_text(label),
            theme.text().muted,
            TextWrap::None,
        )
        .with_style(text_style_for(frame, theme))
        .with_align(theme.command_palette().section_alignment()),
    );
}

fn visual_tint_for(
    frame: &layout::Frame,
    theme: &Theme,
    visuals: &Visuals,
) -> Option<super::Color> {
    if !frame.is_enabled() {
        return None;
    }
    let target_visual = frame
        .target()
        .map(|target| visuals.target(target))
        .unwrap_or_default();

    match frame.table_part() {
        Some(view::TablePart::HeaderControl) => {
            return if target_visual.pressed() || target_visual.active() {
                Some(theme.table().header_pressed_tint)
            } else if target_visual.hovered() || frame.focus_visible() {
                Some(theme.table().header_hover_tint)
            } else {
                None
            };
        }
        Some(
            view::TablePart::HeaderBand
            | view::TablePart::Header
            | view::TablePart::Cell
            | view::TablePart::PassiveToggle
            | view::TablePart::Toggle,
        ) => return None,
        Some(view::TablePart::Action) | None => {}
    }

    if frame.role() == view::Role::Menu {
        if target_visual.pressed() {
            return Some(theme.menu().title_pressed_tint);
        }

        if target_visual.active() {
            return Some(theme.menu().title_active_tint);
        }

        if target_visual.hovered() || frame.focus_visible() {
            return Some(theme.menu().title_hover_tint);
        }

        return None;
    }

    if frame.is_menu_row() || frame.is_palette_row() || frame.provided_row().is_some() {
        return row_highlight_tint_for(frame, theme, visuals);
    }

    if frame.role() == view::Role::TextBox {
        return target_visual
            .active()
            .then_some(theme.control().pressed_tint);
    }

    if !is_control_visual_role(frame.role()) {
        return None;
    }

    if target_visual.pressed() {
        Some(theme.control().pressed_tint)
    } else if target_visual.hovered() {
        Some(theme.control().hover_tint)
    } else {
        None
    }
}

fn row_highlight_tint_for(
    frame: &layout::Frame,
    theme: &Theme,
    visuals: &Visuals,
) -> Option<super::Color> {
    if !frame.is_menu_row() && !frame.is_palette_row() && frame.provided_row().is_none() {
        return None;
    }
    let target_visual = frame
        .target()
        .map(|target| visuals.target(target))
        .unwrap_or_default();

    if target_visual.pressed() {
        return Some(theme.menu().row_pressed_tint);
    }

    let highlighted = target_visual.hovered()
        || frame.focus_visible()
        || frame.is_selected()
        || frame.is_active_item()
        || (frame.is_palette_row() && target_visual.selected());

    highlighted.then_some(theme.menu().row_hover_tint)
}

fn visual_tint_rect_for(frame: &layout::Frame) -> geometry::Rect {
    frame.rect()
}

fn visual_tint_rounding_for(
    _frame: &layout::Frame,
    default: super::Rounding,
    _theme: &Theme,
) -> super::Rounding {
    default
}

fn is_control_visual_role(role: view::Role) -> bool {
    matches!(role, view::Role::Binding | view::Role::Button)
}

fn text_for(frame: &layout::Frame) -> Option<&str> {
    frame.label_text().or_else(|| frame.text())
}

fn paint_table_sort_indicator(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(direction) = frame
        .table_header_presentation()
        .and_then(crate::table::HeaderPresentation::sort_direction)
    else {
        return;
    };
    let icon = match direction {
        crate::table::SortDirection::Ascending => "caret-up",
        crate::table::SortDirection::Descending => "caret-down",
    };
    let rect = layout::table_sort_indicator_rect(frame.rect(), theme);
    scene.push_icon(Icon::new(
        rect,
        icons::Icon::phosphor(icons::Id::new(icon)),
        text_color_for(frame, theme),
        rect.width().min(rect.height()).max(1) as f32,
    ));
}

fn text_rect_for(frame: &layout::Frame, theme: &Theme) -> geometry::Rect {
    if let Some(part) = frame.table_part() {
        return match part {
            view::TablePart::HeaderBand => frame.rect(),
            view::TablePart::PassiveToggle | view::TablePart::Toggle => {
                layout::table_choice_label_rect(frame.rect(), theme)
            }
            view::TablePart::Action => frame.rect(),
            view::TablePart::Header | view::TablePart::HeaderControl => {
                layout::table_header_label_rect(
                    frame.rect(),
                    frame
                        .table_header_presentation()
                        .is_some_and(|presentation| presentation.sort_direction().is_some()),
                    theme,
                )
            }
            view::TablePart::Cell => {
                if frame
                    .text_box()
                    .is_some_and(crate::view::TextBox::is_active)
                {
                    frame.text_box_text_rect()
                } else {
                    layout::table_content_rect(frame.rect(), theme)
                }
            }
        };
    }

    match frame.role() {
        view::Role::Checkbox | view::Role::Radio => layout::choice_label_rect(frame.rect(), theme),
        view::Role::Slider => layout::slider_label_rect(frame.rect(), frame.label_width(), theme),
        view::Role::TextBox => frame.text_box_text_rect(),
        _ => frame.rect(),
    }
}

fn text_color_for(frame: &layout::Frame, theme: &Theme) -> super::Color {
    if !frame.is_enabled() {
        return theme.text().muted;
    }
    if frame.table_part() == Some(view::TablePart::PassiveToggle) {
        return theme.text().muted;
    }
    if frame.table_part() == Some(view::TablePart::Cell) {
        return theme.text().primary;
    }

    match frame.role() {
        view::Role::TextBox
            if frame
                .text_box()
                .is_some_and(|text_box| text_box.text().is_empty()) =>
        {
            theme.text_input().placeholder
        }
        view::Role::SectionHeader => theme.text().muted,
        view::Role::TextArea | view::Role::TextBox => theme.text_input().foreground,
        view::Role::Separator => theme.menu().separator,
        _ => theme.text().primary,
    }
}

fn text_style_for(frame: &layout::Frame, theme: &Theme) -> TextStyle {
    if frame.table_part().is_some() || frame.is_auxiliary_text() {
        return scene_text_style(theme.typography().interface());
    }

    scene_text_style(layout::label_style_for(
        frame.role(),
        frame.binding_source(),
        theme,
    ))
}

fn scene_text_style(style: crate::theme::TypeStyle) -> TextStyle {
    TextStyle::new(style.size(), style.weight())
}

fn text_wrap_for(frame: &layout::Frame) -> TextWrap {
    if frame.table_header_presentation().is_some() {
        return TextWrap::None;
    }
    if let Some(wrap) = frame.world_text_wrap() {
        return match wrap {
            view::Wrap::None => TextWrap::None,
            view::Wrap::Word => TextWrap::WordOrGlyph,
        };
    }
    if matches!(
        frame.role(),
        view::Role::Checkbox | view::Role::Radio | view::Role::Slider
    ) {
        return TextWrap::None;
    }

    match frame.text_wrap() {
        Some(view::Wrap::None) => TextWrap::None,
        Some(view::Wrap::Word) | None => TextWrap::WordOrGlyph,
    }
}

fn text_align_for(frame: &layout::Frame) -> TextAlign {
    if frame.table_part() == Some(view::TablePart::HeaderControl) {
        return TextAlign::Start;
    }

    if let Some(align) = frame.world_text_align() {
        return match align {
            view::Align::Start | view::Align::Stretch => TextAlign::Start,
            view::Align::Center => TextAlign::Center,
            view::Align::End => TextAlign::End,
        };
    }

    match frame.role() {
        view::Role::Button | view::Role::Menu => TextAlign::Center,
        _ => TextAlign::Start,
    }
}

fn outline_color_for(frame: &layout::Frame, theme: &Theme) -> Option<super::Color> {
    matches!(
        frame.role(),
        view::Role::TextArea
            | view::Role::Button
            | view::Role::Slider
            | view::Role::TextBox
            | view::Role::Panel
    )
    .then_some(theme.focus().outline)
    .and_then(visible_fill)
}

fn focus_outline_color_for(frame: &layout::Frame, theme: &Theme) -> Option<super::Color> {
    (frame.focus_visible() && is_focus_outline_role(frame.role())).then_some(theme.focus().color)
}

fn focus_outline_rect_for(frame: &layout::Frame) -> geometry::Rect {
    if frame.table_part() == Some(view::TablePart::Cell)
        && frame
            .text_box()
            .is_some_and(crate::view::TextBox::is_active)
    {
        let rect = frame.rect();
        return geometry::Rect::new(
            rect.x().saturating_add(1),
            rect.y().saturating_add(1),
            rect.width().saturating_sub(2),
            rect.height().saturating_sub(2),
        );
    }

    match frame.role() {
        view::Role::Checkbox | view::Role::Radio | view::Role::Slider => frame.active_rect(),
        _ => frame.rect(),
    }
}

fn focus_outline_rounding_for(
    frame: &layout::Frame,
    default: super::Rounding,
    theme: &Theme,
) -> super::Rounding {
    match frame.role() {
        view::Role::Radio | view::Role::Slider => super::Rounding::relative(1.0),
        view::Role::Checkbox => theme.control().rounding,
        _ => default,
    }
}

fn is_focus_outline_role(role: view::Role) -> bool {
    matches!(
        role,
        view::Role::Menu
            | view::Role::Binding
            | view::Role::TextArea
            | view::Role::Button
            | view::Role::Checkbox
            | view::Role::Radio
            | view::Role::Slider
            | view::Role::TextBox
    )
}
