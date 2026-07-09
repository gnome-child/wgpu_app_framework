mod color;
mod material;
mod paint;
mod presentation;
mod primitive;
mod visual;

pub use color::Color;
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
pub(crate) use visual::Visuals;
pub(crate) use visual::{Scalar as VisualScalar, Target as TargetVisual};

use super::{geometry, layout, overlay, theme::Theme};

const DEFAULT_CLEAR: Color = Color::rgb(17, 18, 20);

#[derive(Debug, Clone)]
pub struct Scene {
    size: geometry::Size,
    clear: Color,
    primitives: Vec<Primitive>,
}

impl Scene {
    #[cfg(test)]
    pub(crate) fn paint(layout: &layout::Layout) -> Self {
        Self::paint_with_clear(layout, DEFAULT_CLEAR)
    }

    #[cfg(test)]
    pub(crate) fn paint_with_theme(layout: &layout::Layout, theme: &Theme) -> Self {
        Self::paint_with_clear_and_theme(layout, theme.surfaces().canvas, theme)
    }

    #[cfg(test)]
    pub(crate) fn paint_with_clear(layout: &layout::Layout, clear: Color) -> Self {
        let theme = Theme::default();
        Self::paint_with_clear_and_theme(layout, clear, &theme)
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
        Self::new_with_clear(size, DEFAULT_CLEAR)
    }

    pub fn new_with_clear(size: geometry::Size, clear: Color) -> Self {
        Self {
            size,
            clear,
            primitives: Vec::new(),
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
    }

    pub(crate) fn append_ghost_scene_with_opacity(&mut self, scene: &Scene, opacity: f32) {
        let mut ghost = scene.clone();
        ghost.primitives = ghost
            .primitives
            .iter()
            .map(ghost_primitive)
            .collect::<Vec<_>>();
        self.append_scene_with_opacity(&ghost, opacity);
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

    pub(super) fn push_pane(&mut self, pane: Pane) {
        if pane.rect().width() > 0 && pane.rect().height() > 0 {
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

fn ghost_primitive(primitive: &Primitive) -> Primitive {
    match primitive {
        Primitive::Pane(pane) => Primitive::Pane(pane.without_backdrop_sampling()),
        Primitive::Group(group) => {
            let primitives = group.primitives().iter().map(ghost_primitive).collect();
            Primitive::Group(
                Group::new(primitives, group.opacity()).expect("existing group is visible"),
            )
        }
        _ => primitive.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            !glass.surface_layers().is_empty(),
            "ghost panes keep paint-only surface layers"
        );
    }
}
