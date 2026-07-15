mod commit;
mod material;
mod paint;
mod presentation;
mod primitive;
mod region;
mod store;
mod visual;

pub use crate::color::Color;
pub(crate) use commit::{Builder as CommitBuilder, Commit, Node, Properties};
#[cfg(feature = "renderer-debug")]
pub(crate) use commit::{
    Content, ContractError, EffectDeclaration, OpacityDeclaration, PropertyKind, PropertyRef,
    PropertyValue,
};
#[cfg(feature = "renderer-debug")]
pub(crate) use commit::{FixtureCase, renderer_fixture};
pub(crate) use material::GlassBase;
pub use material::{
    BackdropBlur, BackdropEdgeMode, BackdropLayer, Glass, Luminosity, Material, Noise, Refraction,
    SurfaceLayer,
};
pub use presentation::Presentation;
pub use primitive::{
    Axis, Brush, Clip, EdgeMode, Group, Icon, Motion, Offset, Outline, Pane, Primitive, Quad,
    Radius, Rasterization, Rounding, Rule, ScaleMotion, Shadow, Stroke, Style, Text, TextAlign,
    TextStyle, TextSurface, TextViewport, TextWrap, Transform,
};
pub(crate) use region::{
    MaterialCapabilities, MaterialFidelity, MaterialRealizationReport, MaterialRegion,
    MaterialRenderer, RealizedMaterialParts,
};
pub(crate) use store::{PaintStats, Store};
pub(crate) use visual::Visuals;

use super::{composition, geometry, theme};
#[cfg(test)]
use super::{layout, overlay, theme::Theme};

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    size: geometry::Size,
    clear: Color,
    primitives: Vec<Primitive>,
    material_regions: Vec<MaterialRegion>,
}

#[derive(Debug, Clone)]
pub(crate) struct NativePopupRequest {
    scene: Scene,
    accent_tint: Color,
}

#[derive(Debug, Clone)]
pub(crate) struct MaterialResolution {
    scene: Scene,
    fidelity: MaterialFidelity,
    region_fidelity: Vec<(composition::tree::NodeId, MaterialFidelity)>,
}

impl Scene {
    #[cfg(test)]
    pub(crate) fn paint(layout: &layout::Layout) -> Self {
        Self::paint_with_theme(layout, &Theme::default())
    }

    #[cfg(test)]
    pub(crate) fn paint_with_theme(layout: &layout::Layout, theme: &Theme) -> Self {
        Self::paint_with_clear_and_theme(layout, theme.surfaces().canvas, theme)
    }

    #[cfg(test)]
    pub(crate) fn paint_with_clear_and_theme(
        layout: &layout::Layout,
        clear: Color,
        theme: &Theme,
    ) -> Self {
        Self::paint_with_clear_theme_and_visuals(layout, clear, theme, &Visuals::default())
    }

    #[cfg(test)]
    pub(crate) fn paint_with_clear_theme_and_visuals(
        layout: &layout::Layout,
        clear: Color,
        theme: &Theme,
        visuals: &Visuals,
    ) -> Self {
        let (mut scene, entries) =
            Self::paint_parts_with_clear_theme_and_visuals(layout, clear, theme, visuals);

        for entry in entries {
            scene.append_scene_with_opacity(entry.scene(), 1.0);
        }

        scene
    }

    #[cfg(test)]
    pub(crate) fn paint_parts_with_clear_theme_and_visuals(
        layout: &layout::Layout,
        clear: Color,
        theme: &Theme,
        visuals: &Visuals,
    ) -> (Self, Vec<overlay::Draft>) {
        let mut scene = Self::new_with_clear(layout.size(), clear);
        let entries = paint::paint_layout_with_theme(layout, &mut scene, theme, visuals);

        (scene, entries)
    }

    pub fn new(size: geometry::Size) -> Self {
        Self::new_with_clear(size, theme::DEFAULT_CANVAS_COLOR)
    }

    pub fn new_with_clear(size: geometry::Size, clear: Color) -> Self {
        Self {
            size,
            clear,
            primitives: Vec::new(),
            material_regions: Vec::new(),
        }
    }

    pub fn size(&self) -> geometry::Size {
        self.size
    }

    pub fn clear(&self) -> Color {
        self.clear
    }

    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub(crate) fn material_regions(&self) -> &[MaterialRegion] {
        &self.material_regions
    }

    pub(crate) fn legacy_full_window_material_region(&self) -> Option<&MaterialRegion> {
        let [region] = self.material_regions.as_slice() else {
            return None;
        };
        let rect = region.rect();
        (rect.x() == 0
            && rect.y() == 0
            && rect.width() == self.size.width()
            && rect.height() == self.size.height())
        .then_some(region)
    }

    pub(crate) fn resolve_material(
        &self,
        renderer: MaterialRenderer,
        reports: &[MaterialRealizationReport],
    ) -> MaterialResolution {
        let clear = match renderer {
            MaterialRenderer::InFrame => self.clear,
            MaterialRenderer::NativePopup { opaque: false } => Color::rgba(0, 0, 0, 0),
            MaterialRenderer::NativePopup { opaque: true } => {
                native_popup_fallback_clear(&self.primitives).unwrap_or(theme::DEFAULT_CANVAS_COLOR)
            }
        };
        let mut region_fidelity = Vec::new();
        let primitives = self
            .primitives
            .iter()
            .filter_map(|primitive| {
                resolve_material_primitive(primitive, renderer, reports, &mut region_fidelity)
            })
            .collect();
        let fidelity = region_fidelity
            .iter()
            .map(|(_, fidelity)| *fidelity)
            .max()
            .unwrap_or(MaterialFidelity::Full);

        MaterialResolution {
            scene: Self {
                size: self.size,
                clear,
                primitives,
                material_regions: Vec::new(),
            },
            fidelity,
            region_fidelity,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    pub(crate) fn append_scene_with_opacity(&mut self, scene: &Scene, opacity: f32) {
        self.append_scene_with_opacity_mode(scene, opacity, false);
    }

    pub(crate) fn append_scene_with_forced_group(&mut self, scene: &Scene, opacity: f32) {
        self.append_scene_with_opacity_mode(scene, opacity, true);
    }

    fn append_scene_with_opacity_mode(&mut self, scene: &Scene, opacity: f32, force_group: bool) {
        let opacity = opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 {
            return;
        }

        if opacity >= 1.0 && !force_group {
            self.primitives.extend(scene.primitives().iter().cloned());
        } else if let Some(group) = Group::new(scene.primitives().to_vec(), opacity) {
            self.primitives.push(Primitive::Group(group));
        }
        self.material_regions.extend(
            scene
                .material_regions
                .iter()
                .map(|region| region.with_parent_opacity(opacity)),
        );
    }

    pub(crate) fn append_ghost_scene_with_opacity(&mut self, scene: &Scene, opacity: f32) {
        let mut ghost = scene.clone();
        ghost.primitives = ghost
            .primitives
            .iter()
            .map(Primitive::without_backdrop_sampling)
            .collect::<Vec<_>>();
        ghost.material_regions.clear();
        self.append_scene_with_opacity(&ghost, opacity);
    }

    pub(crate) fn native_popup_request(&self, bounds: geometry::Rect) -> NativePopupRequest {
        let dx = -bounds.x();
        let dy = -bounds.y();
        let mut scene = Self::new_with_clear(
            geometry::Size::new(bounds.width(), bounds.height()),
            Color::rgba(0, 0, 0, 0),
        );
        scene.primitives = self
            .primitives
            .iter()
            .cloned()
            .map(|primitive| primitive.translated(dx, dy))
            .collect();
        scene.material_regions = self
            .material_regions
            .iter()
            .map(|region| region.translated(dx, dy))
            .collect();

        NativePopupRequest {
            scene,
            accent_tint: native_popup_accent_tint(&self.primitives)
                .unwrap_or(Color::rgba(28, 28, 30, 192)),
        }
    }

    pub fn quads(&self) -> Vec<&Quad> {
        let mut quads = Vec::new();
        collect_quads(&self.primitives, &mut quads);
        quads
    }

    pub fn rules(&self) -> Vec<&Rule> {
        let mut rules = Vec::new();
        collect_rules(&self.primitives, &mut rules);
        rules
    }

    pub fn texts(&self) -> Vec<&Text> {
        let mut texts = Vec::new();
        collect_texts(&self.primitives, &mut texts);
        texts
    }

    pub fn text_viewports(&self) -> Vec<&TextViewport> {
        let mut text_viewports = Vec::new();
        collect_text_viewports(&self.primitives, &mut text_viewports);
        text_viewports
    }

    pub fn icons(&self) -> Vec<&Icon> {
        let mut icons = Vec::new();
        collect_icons(&self.primitives, &mut icons);
        icons
    }

    pub fn shadows(&self) -> Vec<&Shadow> {
        let mut shadows = Vec::new();
        collect_shadows(&self.primitives, &mut shadows);
        shadows
    }

    pub fn panes(&self) -> Vec<&Pane> {
        let mut panes = Vec::new();
        collect_panes(&self.primitives, &mut panes);
        panes
    }

    pub fn outlines(&self) -> Vec<&Outline> {
        let mut outlines = Vec::new();
        collect_outlines(&self.primitives, &mut outlines);
        outlines
    }

    pub fn clips(&self) -> Vec<&Clip> {
        let mut clips = Vec::new();
        collect_clips(&self.primitives, &mut clips);
        clips
    }

    #[cfg(test)]
    pub(crate) fn groups(&self) -> Vec<&Group> {
        self.primitives
            .iter()
            .filter_map(|primitive| match primitive {
                Primitive::Group(group) => Some(group),
                _ => None,
            })
            .collect()
    }

    pub(super) fn push_quad(&mut self, quad: Quad) {
        if quad.rect().width() > 0 && quad.rect().height() > 0 {
            self.primitives.push(Primitive::Quad(quad));
        }
    }

    pub(super) fn push_rule(&mut self, rule: Rule) {
        if rule.rect().width() > 0 && rule.rect().height() > 0 && rule.thickness_px() > 0 {
            self.primitives.push(Primitive::Rule(rule));
        }
    }

    pub(super) fn push_text(&mut self, text: Text) {
        if !text.value().is_empty() && text.rect().width() > 0 && text.rect().height() > 0 {
            self.primitives.push(Primitive::Text(text));
        }
    }

    pub(super) fn push_text_viewport(&mut self, text: TextViewport) {
        if !text.surfaces().is_empty() && text.rect().width() > 0 && text.rect().height() > 0 {
            self.primitives.push(Primitive::TextViewport(text));
        }
    }

    pub(super) fn push_icon(&mut self, icon: Icon) {
        if icon.rect().width() > 0 && icon.rect().height() > 0 && icon.size() > 0.0 {
            self.primitives.push(Primitive::Icon(icon));
        }
    }

    pub(super) fn push_shadow(&mut self, shadow: Shadow) {
        if shadow.rect().width() > 0
            && shadow.rect().height() > 0
            && shadow.color().channels().3 > 0
        {
            self.primitives.push(Primitive::Shadow(shadow));
        }
    }

    #[cfg(test)]
    pub(super) fn push_pane(&mut self, pane: Pane) {
        if pane.rect().width() > 0 && pane.rect().height() > 0 {
            self.primitives.push(Primitive::Pane(pane));
        }
    }

    pub(super) fn push_material_pane(
        &mut self,
        id: composition::tree::NodeId,
        pane: Pane,
        clip: Option<Clip>,
    ) {
        if pane.rect().width() > 0 && pane.rect().height() > 0 {
            let pane = pane.with_region_id(id);
            self.material_regions
                .push(MaterialRegion::from_pane(id, &pane, clip));
            self.primitives.push(Primitive::Pane(pane));
        }
    }

    pub(super) fn push_clip(&mut self, clip: Clip) {
        if clip.rect().width() > 0 && clip.rect().height() > 0 {
            self.primitives.push(Primitive::Clip(clip));
        }
    }

    pub(super) fn pop_clip(&mut self) {
        self.primitives.push(Primitive::PopClip);
    }

    pub(super) fn push_outline(&mut self, outline: Outline) {
        if outline.rect().width() > 0 && outline.rect().height() > 0 {
            self.primitives.push(Primitive::Outline(outline));
        }
    }
}

impl NativePopupRequest {
    pub(crate) fn scene(&self) -> &Scene {
        &self.scene
    }

    pub(crate) fn accent_tint(&self) -> Color {
        self.accent_tint
    }
}

impl MaterialResolution {
    pub(crate) fn scene(&self) -> &Scene {
        &self.scene
    }

    pub(crate) fn fidelity(&self) -> MaterialFidelity {
        self.fidelity
    }

    pub(crate) fn region_fidelity(&self) -> &[(composition::tree::NodeId, MaterialFidelity)] {
        &self.region_fidelity
    }
}

fn resolve_material_primitive(
    primitive: &Primitive,
    renderer: MaterialRenderer,
    reports: &[MaterialRealizationReport],
    fidelity: &mut Vec<(composition::tree::NodeId, MaterialFidelity)>,
) -> Option<Primitive> {
    match primitive {
        Primitive::Pane(pane) => resolve_material_pane(pane, renderer, reports, fidelity),
        Primitive::Shadow(_) if matches!(renderer, MaterialRenderer::NativePopup { .. }) => None,
        Primitive::Group(group) => {
            let primitives = group
                .primitives()
                .iter()
                .filter_map(|primitive| {
                    resolve_material_primitive(primitive, renderer, reports, fidelity)
                })
                .collect();
            Group::new(primitives, group.opacity()).map(Primitive::Group)
        }
        _ => Some(primitive.clone()),
    }
}

fn resolve_material_pane(
    pane: &Pane,
    renderer: MaterialRenderer,
    reports: &[MaterialRealizationReport],
    fidelity: &mut Vec<(composition::tree::NodeId, MaterialFidelity)>,
) -> Option<Primitive> {
    let Material::Glass(glass) = pane.material() else {
        return Some(Primitive::Pane(pane.clone()));
    };
    let Some(id) = pane.region_id() else {
        return match renderer {
            MaterialRenderer::InFrame => Some(Primitive::Pane(pane.clone())),
            MaterialRenderer::NativePopup { .. } => Some(Primitive::Quad(
                Quad::styled(pane.rect(), Style::filled_with(glass.fallback()))
                    .with_rounding(pane.rounding()),
            )),
        };
    };

    match renderer {
        MaterialRenderer::InFrame => {
            fidelity.push((id, MaterialFidelity::Full));
            Some(Primitive::Pane(pane.clone()))
        }
        MaterialRenderer::NativePopup { .. } => {
            let parts = unique_report_parts(id, reports);
            if parts.backdrop_frost() {
                fidelity.push((id, MaterialFidelity::Frost));
                Some(Primitive::Pane(pane.clone().with_material(
                    pane.material().without_realized_parts(parts),
                )))
            } else {
                fidelity.push((id, MaterialFidelity::Fallback));
                Some(Primitive::Quad(
                    Quad::styled(pane.rect(), Style::filled_with(glass.fallback()))
                        .with_rounding(pane.rounding()),
                ))
            }
        }
    }
}

fn unique_report_parts(
    id: composition::tree::NodeId,
    reports: &[MaterialRealizationReport],
) -> RealizedMaterialParts {
    let mut matching = reports.iter().filter(|report| report.id() == id);
    let Some(report) = matching.next() else {
        return RealizedMaterialParts::none();
    };
    if matching.next().is_some() {
        return RealizedMaterialParts::none();
    }
    report.parts()
}

fn collect_quads<'a>(primitives: &'a [Primitive], quads: &mut Vec<&'a Quad>) {
    for primitive in primitives {
        match primitive {
            Primitive::Quad(quad) => quads.push(quad),
            Primitive::Group(group) => collect_quads(group.primitives(), quads),
            _ => {}
        }
    }
}

fn collect_rules<'a>(primitives: &'a [Primitive], rules: &mut Vec<&'a Rule>) {
    for primitive in primitives {
        match primitive {
            Primitive::Rule(rule) => rules.push(rule),
            Primitive::Group(group) => collect_rules(group.primitives(), rules),
            _ => {}
        }
    }
}

fn collect_texts<'a>(primitives: &'a [Primitive], texts: &mut Vec<&'a Text>) {
    for primitive in primitives {
        match primitive {
            Primitive::Text(text) => texts.push(text),
            Primitive::Group(group) => collect_texts(group.primitives(), texts),
            _ => {}
        }
    }
}

fn collect_text_viewports<'a>(
    primitives: &'a [Primitive],
    text_viewports: &mut Vec<&'a TextViewport>,
) {
    for primitive in primitives {
        match primitive {
            Primitive::TextViewport(text_viewport) => text_viewports.push(text_viewport),
            Primitive::Group(group) => collect_text_viewports(group.primitives(), text_viewports),
            _ => {}
        }
    }
}

fn collect_icons<'a>(primitives: &'a [Primitive], icons: &mut Vec<&'a Icon>) {
    for primitive in primitives {
        match primitive {
            Primitive::Icon(icon) => icons.push(icon),
            Primitive::Group(group) => collect_icons(group.primitives(), icons),
            _ => {}
        }
    }
}

fn collect_shadows<'a>(primitives: &'a [Primitive], shadows: &mut Vec<&'a Shadow>) {
    for primitive in primitives {
        match primitive {
            Primitive::Shadow(shadow) => shadows.push(shadow),
            Primitive::Group(group) => collect_shadows(group.primitives(), shadows),
            _ => {}
        }
    }
}

fn collect_panes<'a>(primitives: &'a [Primitive], panes: &mut Vec<&'a Pane>) {
    for primitive in primitives {
        match primitive {
            Primitive::Pane(pane) => panes.push(pane),
            Primitive::Group(group) => collect_panes(group.primitives(), panes),
            _ => {}
        }
    }
}

fn collect_outlines<'a>(primitives: &'a [Primitive], outlines: &mut Vec<&'a Outline>) {
    for primitive in primitives {
        match primitive {
            Primitive::Outline(outline) => outlines.push(outline),
            Primitive::Group(group) => collect_outlines(group.primitives(), outlines),
            _ => {}
        }
    }
}

fn collect_clips<'a>(primitives: &'a [Primitive], clips: &mut Vec<&'a Clip>) {
    for primitive in primitives {
        match primitive {
            Primitive::Clip(clip) => clips.push(clip),
            Primitive::Group(group) => collect_clips(group.primitives(), clips),
            _ => {}
        }
    }
}

fn native_popup_fallback_clear(primitives: &[Primitive]) -> Option<Color> {
    primitives.iter().find_map(|primitive| match primitive {
        Primitive::Pane(pane) => match pane.material() {
            Material::Glass(glass) => match glass.fallback() {
                Brush::Solid(color) => Some(color),
                Brush::LinearGradient { .. } => None,
            },
            Material::Solid(Brush::Solid(color)) => Some(*color),
            Material::Solid(Brush::LinearGradient { .. }) => None,
        },
        Primitive::Group(group) => native_popup_fallback_clear(group.primitives()),
        _ => None,
    })
}

fn native_popup_accent_tint(primitives: &[Primitive]) -> Option<Color> {
    primitives.iter().find_map(|primitive| match primitive {
        Primitive::Pane(pane) => match pane.material() {
            Material::Glass(glass) => match glass.tint() {
                Some((Brush::Solid(color), opacity)) => {
                    let (r, g, b, _) = color.channels();
                    Some(Color::rgba(
                        r,
                        g,
                        b,
                        (opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
                    ))
                }
                Some((Brush::LinearGradient { .. }, _)) => None,
                None => match glass.fallback() {
                    Brush::Solid(color) => {
                        let (r, g, b, _) = color.channels();
                        Some(Color::rgba(r, g, b, 192))
                    }
                    Brush::LinearGradient { .. } => None,
                },
            },
            Material::Solid(Brush::Solid(color)) => {
                let (r, g, b, a) = color.channels();
                Some(Color::rgba(r, g, b, a))
            }
            Material::Solid(Brush::LinearGradient { .. }) => None,
        },
        Primitive::Group(group) => native_popup_accent_tint(group.primitives()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view;

    fn retained_material_region_ids(view: view::View) -> Vec<composition::tree::NodeId> {
        retained_material_region_ids_by_entry(view)
            .into_iter()
            .flatten()
            .collect()
    }

    fn retained_material_region_ids_by_entry(
        view: view::View,
    ) -> Vec<Vec<composition::tree::NodeId>> {
        let window = crate::window::Id::new(1);
        let mut store = composition::Store::default();
        let composition = store.install(window, view);
        let theme = Theme::default();
        let mut engine = layout::Engine::new();
        let layout = layout::Layout::compose_composition_with_theme_at(
            composition,
            geometry::Size::new(240, 160),
            &mut engine,
            &theme,
            crate::animation::Frame::new(std::time::Instant::now()),
            crate::keymap::Profile::default(),
            layout::PopupSurfaces::InFrame,
        );
        let (_, entries) = Scene::paint_parts_with_clear_theme_and_visuals(
            &layout,
            theme.surfaces().canvas,
            &theme,
            &Visuals::default(),
        );

        entries
            .iter()
            .map(|entry| {
                entry
                    .scene()
                    .material_regions()
                    .iter()
                    .map(MaterialRegion::id)
                    .collect()
            })
            .collect()
    }

    fn retained_material_region_ids_across(
        first: view::View,
        second: view::View,
    ) -> (
        Vec<composition::tree::NodeId>,
        Vec<composition::tree::NodeId>,
    ) {
        let window = crate::window::Id::new(1);
        let mut store = composition::Store::default();
        let theme = Theme::default();
        let mut engine = layout::Engine::new();

        let first_ids = {
            let composition = store.install(window, first);
            let layout = layout::Layout::compose_composition_with_theme_at(
                composition,
                geometry::Size::new(240, 160),
                &mut engine,
                &theme,
                crate::animation::Frame::new(std::time::Instant::now()),
                crate::keymap::Profile::default(),
                layout::PopupSurfaces::InFrame,
            );
            let (_, entries) = Scene::paint_parts_with_clear_theme_and_visuals(
                &layout,
                theme.surfaces().canvas,
                &theme,
                &Visuals::default(),
            );
            entries
                .iter()
                .flat_map(|entry| entry.scene().material_regions().iter())
                .map(MaterialRegion::id)
                .collect()
        };
        let second_ids = {
            let composition = store.install(window, second);
            let layout = layout::Layout::compose_composition_with_theme_at(
                composition,
                geometry::Size::new(240, 160),
                &mut engine,
                &theme,
                crate::animation::Frame::new(std::time::Instant::now()),
                crate::keymap::Profile::default(),
                layout::PopupSurfaces::InFrame,
            );
            let (_, entries) = Scene::paint_parts_with_clear_theme_and_visuals(
                &layout,
                theme.surfaces().canvas,
                &theme,
                &Visuals::default(),
            );
            entries
                .iter()
                .flat_map(|entry| entry.scene().material_regions().iter())
                .map(MaterialRegion::id)
                .collect()
        };

        (first_ids, second_ids)
    }

    fn panel(id: &'static str) -> view::Node {
        view::Node::floating_panel(id).child(view::Node::label(id))
    }

    fn panels(ids: &[&'static str]) -> view::View {
        view::View::new(
            ids.iter()
                .fold(view::Node::root(), |root, id| root.child(panel(id))),
        )
    }

    fn nested_panels() -> view::View {
        view::View::new(view::Node::root().child(
            view::Node::floating_panel("outer").child(
                view::Node::floating_panel("inner").child(view::Node::label("inner content")),
            ),
        ))
    }

    fn simple_scene() -> Scene {
        let mut scene = Scene::new(geometry::Size::new(100, 100));
        scene.push_quad(Quad::new(
            geometry::Rect::new(0, 0, 10, 10),
            Color::rgb(255, 0, 0),
        ));
        scene
    }

    fn glass_pane_scene() -> Scene {
        let mut scene = Scene::new(geometry::Size::new(100, 100));
        scene.push_pane(
            Pane::new(
                geometry::Rect::new(4, 6, 40, 24),
                Material::glass(Glass::panel_dark()),
            )
            .with_rounding(Rounding::fixed(8.0)),
        );
        scene
    }

    #[test]
    fn material_region_identity_is_retained_while_order_is_a_projection() {
        let (first, reordered) =
            retained_material_region_ids_across(panels(&["one", "two"]), panels(&["two", "one"]));

        assert_eq!(first.len(), 2);
        assert_eq!(reordered, vec![first[1], first[0]]);
        assert!(first.iter().all(|id| id.is_retained()));
    }

    #[test]
    fn insertion_before_material_region_does_not_rename_it() {
        let (first, inserted) = retained_material_region_ids_across(
            panels(&["one", "two"]),
            panels(&["zero", "one", "two"]),
        );

        assert_eq!(first.len(), 2);
        assert_eq!(inserted.len(), 3);
        assert_eq!(&inserted[1..], first.as_slice());
        assert_ne!(inserted[0], first[0]);
        assert_ne!(inserted[0], first[1]);
    }

    #[test]
    fn departing_material_region_is_removed_without_renaming_survivors() {
        let (first, removed) =
            retained_material_region_ids_across(panels(&["one", "two"]), panels(&["one"]));

        assert_eq!(first.len(), 2);
        assert_eq!(removed, vec![first[0]]);
    }

    #[test]
    fn one_popup_scene_can_submit_two_independently_retained_material_regions() {
        let entries = retained_material_region_ids_by_entry(nested_panels());
        let [ids] = entries.as_slice() else {
            panic!("nested material owners should share one root popup scene");
        };

        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
        assert!(ids.iter().all(|id| id.is_retained()));
    }

    #[test]
    fn painted_material_region_carries_declaring_geometry_recipe_and_clip() {
        let ids = retained_material_region_ids(panels(&["panel"]));
        assert_eq!(ids.len(), 1);

        let id = retained_material_region_ids(panels(&["manual.panel"]))[0];
        let pane = Pane::new(
            geometry::Rect::new(4, 6, 40, 24),
            Material::glass(Glass::panel_dark()),
        )
        .with_rounding(Rounding::fixed(8.0));
        let clip = Clip::new(geometry::Rect::new(5, 7, 38, 22)).with_rounding(Rounding::fixed(6.0));
        let mut scene = Scene::new(geometry::Size::new(100, 100));
        scene.push_material_pane(id, pane, Some(clip));

        let [region] = scene.material_regions() else {
            panic!("material pane should emit one request");
        };
        assert_eq!(region.id(), id);
        assert_eq!(region.rect(), geometry::Rect::new(4, 6, 40, 24));
        assert_eq!(region.rounding(), Rounding::fixed(8.0));
        assert_eq!(region.clips(), &[clip]);
        assert_eq!(region.opacity(), 1.0);
        assert!(matches!(region.material(), Material::Glass(_)));
    }

    #[test]
    fn material_region_translation_and_parent_opacity_follow_scene_projection() {
        let id = retained_material_region_ids(panels(&["translated.panel"]))[0];
        let pane = Pane::new(
            geometry::Rect::new(4, 6, 40, 24),
            Material::glass(Glass::panel_dark()),
        );
        let clip = Clip::new(geometry::Rect::new(4, 6, 40, 24));
        let mut source = Scene::new(geometry::Size::new(100, 100));
        source.push_material_pane(id, pane, Some(clip));
        let mut faded = Scene::new(geometry::Size::new(100, 100));
        faded.append_scene_with_opacity(&source, 0.5);

        let popup = faded.native_popup_request(geometry::Rect::new(4, 6, 40, 24));
        let [region] = popup.scene().material_regions() else {
            panic!("native scene should retain one translated request");
        };
        assert_eq!(region.id(), id);
        assert_eq!(region.rect(), geometry::Rect::new(0, 0, 40, 24));
        assert_eq!(region.clips()[0].rect(), geometry::Rect::new(0, 0, 40, 24));
        assert_eq!(region.opacity(), 0.5);
        assert_eq!(popup.scene().panes().len(), 1);
    }

    #[test]
    fn opacity_one_overlay_appends_inline_primitives() {
        let source = simple_scene();
        let mut target = Scene::new(geometry::Size::new(100, 100));

        target.append_scene_with_opacity(&source, 1.0);

        assert_eq!(target.groups().len(), 0);
        assert_eq!(target.quads().len(), 1);
    }

    #[test]
    fn mid_opacity_overlay_promotes_to_group() {
        let source = simple_scene();
        let mut target = Scene::new(geometry::Size::new(100, 100));

        target.append_scene_with_opacity(&source, 0.5);

        assert!(
            !target
                .primitives()
                .iter()
                .any(|primitive| matches!(primitive, Primitive::Quad(_)))
        );
        let groups = target.groups();
        let [group] = groups.as_slice() else {
            panic!("expected one group");
        };
        assert_eq!(group.opacity(), 0.5);
        assert_eq!(group.primitives().len(), 1);
    }

    #[test]
    fn forced_opacity_one_overlay_promotes_to_group() {
        let source = simple_scene();
        let mut target = Scene::new(geometry::Size::new(100, 100));

        target.append_scene_with_forced_group(&source, 1.0);

        assert_eq!(target.quads().len(), 1);
        let groups = target.groups();
        let [group] = groups.as_slice() else {
            panic!("expected one group");
        };
        assert_eq!(group.opacity(), 1.0);
        assert_eq!(group.primitives().len(), 1);
    }

    #[test]
    fn ghost_overlay_downgrades_panes_to_non_backdrop_material() {
        let source = glass_pane_scene();
        let mut target = Scene::new(geometry::Size::new(100, 100));

        target.append_ghost_scene_with_opacity(&source, 0.5);

        let panes = target.panes();
        let [pane] = panes.as_slice() else {
            panic!("ghost should keep one pane");
        };
        let Material::Glass(glass) = pane.material() else {
            panic!("ghost pane should keep glass body");
        };
        assert!(
            glass.backdrop_layers().is_empty(),
            "ghost panes must not backdrop-sample"
        );
        assert_eq!(glass.base(), GlassBase::Fallback);
        assert!(
            !glass.surface_layers().is_empty(),
            "ghost panes keep paint-only surface layers"
        );
        assert!(
            target.material_regions().is_empty(),
            "paint-only ghosts must not retain platform material requests"
        );
    }

    #[test]
    fn ghost_overlay_preserves_nested_group_membership_and_opacity() {
        let mut source = glass_pane_scene();
        let Some(group) = Group::new(std::mem::take(&mut source.primitives), 0.75) else {
            panic!("the pane scene should form a visible group");
        };
        source.primitives.push(Primitive::Group(group));
        let mut target = Scene::new(geometry::Size::new(100, 100));

        target.append_ghost_scene_with_opacity(&source, 0.5);

        let [Primitive::Group(outer)] = target.primitives() else {
            panic!("ghost opacity should form one outer group");
        };
        let [Primitive::Group(inner)] = outer.primitives() else {
            panic!("ghost projection should retain the nested group");
        };
        assert_eq!(outer.opacity(), 0.5);
        assert_eq!(inner.opacity(), 0.75);
        assert_eq!(target.panes().len(), 1);
        assert!(target.material_regions().is_empty());
    }

    #[test]
    fn native_popup_request_preserves_glass_until_platform_reports_outcome() {
        let id = retained_material_region_ids(panels(&["native.panel"]))[0];
        let mut source = Scene::new(geometry::Size::new(100, 100));
        source.push_material_pane(
            id,
            Pane::new(
                geometry::Rect::new(4, 6, 40, 24),
                Material::glass(Glass::panel_dark()),
            ),
            None,
        );

        let popup = source.native_popup_request(geometry::Rect::new(4, 6, 40, 24));
        let requested = popup.scene();

        assert_eq!(popup.accent_tint(), Color::rgba(28, 28, 30, 102));
        assert_eq!(requested.size(), geometry::Size::new(40, 24));
        assert_eq!(requested.clear(), Color::rgba(0, 0, 0, 0));
        assert_eq!(requested.panes().len(), 1);

        let resolution = requested.resolve_material(
            MaterialRenderer::NativePopup { opaque: false },
            &[MaterialRealizationReport::new(
                id,
                RealizedMaterialParts::frost(true),
            )],
        );
        assert_eq!(resolution.fidelity(), MaterialFidelity::Frost);
        let panes = resolution.scene().panes();
        let [pane] = panes.as_slice() else {
            panic!("surface residue should remain after native frost realization");
        };
        let Material::Glass(glass) = pane.material() else {
            panic!("residual material keeps its glass recipe");
        };
        assert!(glass.backdrop_layers().is_empty());
        assert_eq!(glass.base(), GlassBase::Transparent);
        assert!(glass.tint().is_none());
    }

    #[test]
    fn missing_native_report_resolves_to_opaque_fallback() {
        let id = retained_material_region_ids(panels(&["fallback.panel"]))[0];
        let mut source = Scene::new(geometry::Size::new(40, 24));
        source.push_material_pane(
            id,
            Pane::new(
                geometry::Rect::new(0, 0, 40, 24),
                Material::glass(Glass::panel_dark()),
            ),
            None,
        );

        let resolution =
            source.resolve_material(MaterialRenderer::NativePopup { opaque: true }, &[]);
        let fallback = resolution.scene();

        assert_eq!(resolution.fidelity(), MaterialFidelity::Fallback);
        assert_eq!(fallback.size(), geometry::Size::new(40, 24));
        assert_eq!(fallback.clear(), Color::rgb(28, 28, 30));
        assert!(
            fallback.panes().is_empty(),
            "fallback body must not be framework glass"
        );
        let quads = fallback.quads();
        let [quad] = quads.as_slice() else {
            panic!("fallback should render one solid body quad");
        };
        assert_eq!(quad.rect(), geometry::Rect::new(0, 0, 40, 24));
    }

    #[test]
    fn report_order_does_not_change_region_identity_or_residual_order() {
        let ids = retained_material_region_ids(panels(&["one", "two"]));
        let mut source =
            Scene::new_with_clear(geometry::Size::new(80, 24), Color::rgba(0, 0, 0, 0));
        for (index, id) in ids.iter().copied().enumerate() {
            source.push_material_pane(
                id,
                Pane::new(
                    geometry::Rect::new((index as i32) * 40, 0, 40, 24),
                    Material::glass(Glass::panel_dark()),
                ),
                None,
            );
        }
        let forward = source.resolve_material(
            MaterialRenderer::NativePopup { opaque: false },
            &[
                MaterialRealizationReport::new(ids[0], RealizedMaterialParts::frost(false)),
                MaterialRealizationReport::new(ids[1], RealizedMaterialParts::frost(true)),
            ],
        );
        let reverse = source.resolve_material(
            MaterialRenderer::NativePopup { opaque: false },
            &[
                MaterialRealizationReport::new(ids[1], RealizedMaterialParts::frost(true)),
                MaterialRealizationReport::new(ids[0], RealizedMaterialParts::frost(false)),
            ],
        );

        assert_eq!(forward.region_fidelity(), reverse.region_fidelity());
        assert_eq!(
            forward.region_fidelity(),
            &[
                (ids[0], MaterialFidelity::Frost),
                (ids[1], MaterialFidelity::Frost)
            ]
        );
        assert_eq!(
            forward
                .scene()
                .panes()
                .iter()
                .map(|pane| pane.rect())
                .collect::<Vec<_>>(),
            reverse
                .scene()
                .panes()
                .iter()
                .map(|pane| pane.rect())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn missing_stale_and_duplicate_reports_never_consume_requested_material() {
        let ids = retained_material_region_ids(panels(&["live", "stale"]));
        let mut source = Scene::new(geometry::Size::new(40, 24));
        source.push_material_pane(
            ids[0],
            Pane::new(
                geometry::Rect::new(0, 0, 40, 24),
                Material::glass(Glass::panel_dark()),
            ),
            None,
        );

        for reports in [
            vec![],
            vec![MaterialRealizationReport::new(
                ids[1],
                RealizedMaterialParts::frost(true),
            )],
            vec![
                MaterialRealizationReport::new(ids[0], RealizedMaterialParts::frost(true)),
                MaterialRealizationReport::new(ids[0], RealizedMaterialParts::frost(true)),
            ],
        ] {
            let resolution =
                source.resolve_material(MaterialRenderer::NativePopup { opaque: true }, &reports);
            assert_eq!(resolution.fidelity(), MaterialFidelity::Fallback);
            assert!(resolution.scene().panes().is_empty());
            assert_eq!(resolution.scene().quads().len(), 1);
        }
    }

    #[test]
    fn mixed_native_outcomes_preserve_one_final_fidelity_per_requested_region() {
        let ids = retained_material_region_ids(panels(&["frost", "fallback"]));
        let mut source =
            Scene::new_with_clear(geometry::Size::new(80, 24), Color::rgba(0, 0, 0, 0));
        for (index, id) in ids.iter().copied().enumerate() {
            source.push_material_pane(
                id,
                Pane::new(
                    geometry::Rect::new((index as i32) * 40, 0, 40, 24),
                    Material::glass(Glass::panel_dark()),
                ),
                None,
            );
        }

        let resolution = source.resolve_material(
            MaterialRenderer::NativePopup { opaque: false },
            &[MaterialRealizationReport::new(
                ids[0],
                RealizedMaterialParts::frost(false),
            )],
        );

        assert_eq!(resolution.fidelity(), MaterialFidelity::Fallback);
        assert_eq!(
            resolution.region_fidelity(),
            &[
                (ids[0], MaterialFidelity::Frost),
                (ids[1], MaterialFidelity::Fallback)
            ]
        );
        assert_eq!(resolution.scene().panes().len(), 1);
        assert_eq!(resolution.scene().quads().len(), 1);
    }
}
