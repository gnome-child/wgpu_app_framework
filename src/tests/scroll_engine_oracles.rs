use std::collections::{BTreeMap, BTreeSet};

const EPSILON: f64 = 1.0e-9;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn plus(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    fn minus(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    fn is_near(self, other: Self) -> bool {
        (self.x - other.x).abs() <= EPSILON && (self.y - other.y).abs() <= EPSILON
    }

    fn is_zero(self) -> bool {
        self.is_near(Self::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AxisConfiguration {
    lower: f64,
    upper: f64,
    page: f64,
    step: f64,
    page_step: f64,
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct OracleAxis {
    configuration: AxisConfiguration,
    revision: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct AxisOutcome {
    applied: f64,
    remaining: f64,
}

impl OracleAxis {
    fn new(lower: f64, upper: f64, page: f64, value: f64) -> Self {
        let mut axis = Self {
            configuration: AxisConfiguration {
                lower,
                upper,
                page,
                step: 1.0,
                page_step: page,
                value: lower,
            },
            revision: 0,
        };
        axis.configuration.value = axis.clamp(value);
        axis
    }

    fn maximum(self) -> f64 {
        (self.configuration.upper - self.configuration.page).max(self.configuration.lower)
    }

    fn value(self) -> f64 {
        self.configuration.value
    }

    fn clamp(self, value: f64) -> f64 {
        value.clamp(self.configuration.lower, self.maximum())
    }

    fn apply(&mut self, delta: f64) -> AxisOutcome {
        let before = self.value();
        let after = self.clamp(before + delta);
        self.configuration.value = after;
        if (after - before).abs() > EPSILON {
            self.revision += 1;
        }
        let applied = after - before;
        AxisOutcome {
            applied,
            remaining: delta - applied,
        }
    }

    fn set(&mut self, value: f64) -> AxisOutcome {
        self.apply(value - self.value())
    }

    fn configure(&mut self, configuration: AxisConfiguration) -> Result<(), &'static str> {
        if ![
            configuration.lower,
            configuration.upper,
            configuration.page,
            configuration.step,
            configuration.page_step,
            configuration.value,
        ]
        .into_iter()
        .all(f64::is_finite)
            || configuration.upper < configuration.lower
            || configuration.page < 0.0
            || configuration.step < 0.0
            || configuration.page_step < 0.0
        {
            return Err("invalid axis configuration");
        }

        let before = self.configuration;
        self.configuration = configuration;
        self.configuration.value = self.clamp(configuration.value);
        if self.configuration != before {
            self.revision += 1;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MotionDefect {
    None,
    QuantizeEachUpdate,
    DropHorizontal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MotionAdapter {
    x: OracleAxis,
    y: OracleAxis,
    defect: MotionDefect,
}

impl MotionAdapter {
    fn new(defect: MotionDefect) -> Self {
        Self {
            x: OracleAxis::new(-1_000.0, 1_000.0, 0.0, 0.0),
            y: OracleAxis::new(-1_000.0, 1_000.0, 0.0, 0.0),
            defect,
        }
    }

    fn update(&mut self, mut delta: Vector) -> Vector {
        match self.defect {
            MotionDefect::None => {}
            MotionDefect::QuantizeEachUpdate => {
                delta.x = delta.x.round();
                delta.y = delta.y.round();
            }
            MotionDefect::DropHorizontal => delta.x = 0.0,
        }
        self.x.apply(delta.x);
        self.y.apply(delta.y);
        Vector::new(self.x.value(), self.y.value())
    }
}

fn validate_fractional_diagonal_motion(defect: MotionDefect) -> Result<(), &'static str> {
    let mut adapter = MotionAdapter::new(defect);
    let trace = [
        (Vector::new(0.25, 0.40), Vector::new(0.25, 0.40)),
        (Vector::new(0.25, 0.35), Vector::new(0.50, 0.75)),
        (Vector::new(-0.10, 0.50), Vector::new(0.40, 1.25)),
    ];
    for (delta, expected) in trace {
        if !adapter.update(delta).is_near(expected) {
            return Err("fractional diagonal position diverged");
        }
    }
    Ok(())
}

#[test]
fn scrolling_oracle_preserves_fractional_diagonal_motion_continuously() {
    validate_fractional_diagonal_motion(MotionDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_quantized_and_single_axis_motion_adapters() {
    assert_eq!(
        validate_fractional_diagonal_motion(MotionDefect::QuantizeEachUpdate),
        Err("fractional diagonal position diverged")
    );
    assert_eq!(
        validate_fractional_diagonal_motion(MotionDefect::DropHorizontal),
        Err("fractional diagonal position diverged")
    );
}

#[test]
fn axis_configuration_oracle_is_atomic_clamped_and_revisioned_once() {
    let mut axis = OracleAxis::new(0.0, 100.0, 20.0, 40.0);
    let revision = axis.revision;
    axis.configure(AxisConfiguration {
        lower: -10.0,
        upper: 50.0,
        page: 15.0,
        step: 2.5,
        page_step: 12.0,
        value: 90.0,
    })
    .unwrap();

    assert_eq!(axis.value(), 35.0);
    assert_eq!(axis.revision, revision + 1);
    assert_eq!(axis.configuration.step, 2.5);
    assert_eq!(axis.configuration.page_step, 12.0);

    let unchanged_revision = axis.revision;
    axis.configure(axis.configuration).unwrap();
    assert_eq!(axis.revision, unchanged_revision);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum InputSource {
    Wheel,
    Touchpad,
    Touchscreen,
    Scrollbar,
    Keyboard,
    Reveal,
    Programmatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Pixel,
    Line,
    Page,
    Absolute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Begin,
    Update,
    End,
    Cancel,
    Deceleration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SessionEvent {
    source: InputSource,
    unit: Unit,
    timestamp_us: u64,
    phase: Phase,
    delta: Vector,
    terminal_velocity: Vector,
}

impl SessionEvent {
    fn new(source: InputSource, timestamp_us: u64, phase: Phase, delta: Vector) -> Self {
        Self {
            source,
            unit: Unit::Pixel,
            timestamp_us,
            phase,
            delta,
            terminal_velocity: Vector::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionDefect {
    None,
    LoseTerminalVelocity,
    KeepKineticOnDirectInput,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct SessionModel {
    active: Option<InputSource>,
    kinetic_velocity: Option<Vector>,
    position: Vector,
    last_timestamp_us: Option<u64>,
    last_unit: Option<Unit>,
}

impl SessionModel {
    fn handle(&mut self, defect: SessionDefect, event: SessionEvent) -> Result<(), &'static str> {
        if self
            .last_timestamp_us
            .is_some_and(|timestamp| event.timestamp_us < timestamp)
        {
            return Err("session timestamps regressed");
        }
        self.last_timestamp_us = Some(event.timestamp_us);
        self.last_unit = Some(event.unit);

        match event.phase {
            Phase::Begin => {
                if defect != SessionDefect::KeepKineticOnDirectInput {
                    self.kinetic_velocity = None;
                }
                self.active = Some(event.source);
            }
            Phase::Update => {
                if self.active != Some(event.source) {
                    return Err("update did not match active source");
                }
                self.position = self.position.plus(event.delta);
            }
            Phase::End => {
                if self.active != Some(event.source) {
                    return Err("end did not match active source");
                }
                self.active = None;
                if defect != SessionDefect::LoseTerminalVelocity
                    && !event.terminal_velocity.is_zero()
                {
                    self.kinetic_velocity = Some(event.terminal_velocity);
                }
            }
            Phase::Cancel => {
                if self.active != Some(event.source) {
                    return Err("cancel did not match active source");
                }
                self.active = None;
                self.kinetic_velocity = None;
            }
            Phase::Deceleration => {
                if self.kinetic_velocity.is_none() {
                    return Err("deceleration had no terminal velocity");
                }
                self.position = self.position.plus(event.delta);
            }
        }
        Ok(())
    }
}

fn validate_session_lifecycle(defect: SessionDefect) -> Result<(), &'static str> {
    let mut session = SessionModel::default();
    session.handle(
        defect,
        SessionEvent::new(InputSource::Touchpad, 100, Phase::Begin, Vector::default()),
    )?;
    session.handle(
        defect,
        SessionEvent::new(
            InputSource::Touchpad,
            110,
            Phase::Update,
            Vector::new(0.75, 4.25),
        ),
    )?;
    let mut end = SessionEvent::new(InputSource::Touchpad, 120, Phase::End, Vector::default());
    end.terminal_velocity = Vector::new(0.5, 3.0);
    session.handle(defect, end)?;
    if session.kinetic_velocity != Some(Vector::new(0.5, 3.0)) {
        return Err("terminal velocity was not retained");
    }
    session.handle(
        defect,
        SessionEvent::new(
            InputSource::Touchpad,
            130,
            Phase::Deceleration,
            Vector::new(0.25, 1.5),
        ),
    )?;
    session.handle(
        defect,
        SessionEvent::new(InputSource::Wheel, 140, Phase::Begin, Vector::default()),
    )?;
    if session.kinetic_velocity.is_some() {
        return Err("direct input did not interrupt kinetic motion");
    }
    session.handle(
        defect,
        SessionEvent::new(InputSource::Wheel, 150, Phase::Cancel, Vector::default()),
    )?;
    if session.active.is_some() || session.kinetic_velocity.is_some() {
        return Err("cancel left session state active");
    }
    Ok(())
}

#[test]
fn scrolling_oracle_models_begin_update_end_cancel_deceleration_and_interruption() {
    validate_session_lifecycle(SessionDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_lost_velocity_and_uninterrupted_kinetic_adapters() {
    assert_eq!(
        validate_session_lifecycle(SessionDefect::LoseTerminalVelocity),
        Err("terminal velocity was not retained")
    );
    assert_eq!(
        validate_session_lifecycle(SessionDefect::KeepKineticOnDirectInput),
        Err("direct input did not interrupt kinetic motion")
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandoffDefect {
    None,
    SwallowRemainder,
    CoupleAxes,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct OracleViewport {
    x: OracleAxis,
    y: OracleAxis,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ViewportOutcome {
    applied: Vector,
    remaining: Vector,
}

impl OracleViewport {
    fn with_values(x_max: f64, x: f64, y_max: f64, y: f64) -> Self {
        Self {
            x: OracleAxis::new(0.0, x_max, 0.0, x),
            y: OracleAxis::new(0.0, y_max, 0.0, y),
        }
    }

    fn apply(&mut self, delta: Vector, coupled: bool) -> ViewportOutcome {
        if coupled {
            let x_can_apply = self.x.clamp(self.x.value() + delta.x) != self.x.value();
            let y_can_apply = self.y.clamp(self.y.value() + delta.y) != self.y.value();
            if !x_can_apply || !y_can_apply {
                return ViewportOutcome {
                    applied: Vector::default(),
                    remaining: delta,
                };
            }
        }
        let x = self.x.apply(delta.x);
        let y = self.y.apply(delta.y);
        ViewportOutcome {
            applied: Vector::new(x.applied, y.applied),
            remaining: Vector::new(x.remaining, y.remaining),
        }
    }
}

fn dispatch_nested(
    defect: HandoffDefect,
    viewports: &mut [OracleViewport],
    delta: Vector,
) -> Vec<ViewportOutcome> {
    let mut remaining = delta;
    let mut outcomes = Vec::new();
    for viewport in viewports {
        let mut outcome = viewport.apply(remaining, defect == HandoffDefect::CoupleAxes);
        if defect == HandoffDefect::SwallowRemainder {
            outcome.remaining = Vector::default();
        }
        remaining = outcome.remaining;
        outcomes.push(outcome);
        if remaining.is_zero() {
            break;
        }
    }
    outcomes
}

fn validate_nested_handoff(defect: HandoffDefect) -> Result<(), &'static str> {
    let mut chain = [
        OracleViewport::with_values(100.0, 90.0, 100.0, 100.0),
        OracleViewport::with_values(20.0, 20.0, 100.0, 60.0),
        OracleViewport::with_values(100.0, 10.0, 100.0, 100.0),
    ];
    let outcomes = dispatch_nested(defect, &mut chain, Vector::new(30.0, 40.0));
    let expected = [
        ViewportOutcome {
            applied: Vector::new(10.0, 0.0),
            remaining: Vector::new(20.0, 40.0),
        },
        ViewportOutcome {
            applied: Vector::new(0.0, 40.0),
            remaining: Vector::new(20.0, 0.0),
        },
        ViewportOutcome {
            applied: Vector::new(20.0, 0.0),
            remaining: Vector::default(),
        },
    ];
    if outcomes != expected {
        return Err("nested applied/remainder trace diverged");
    }
    Ok(())
}

#[test]
fn scrolling_oracle_hands_each_axis_to_ancestors_independently() {
    validate_nested_handoff(HandoffDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_swallowed_remainders_and_coupled_axes() {
    assert_eq!(
        validate_nested_handoff(HandoffDefect::SwallowRemainder),
        Err("nested applied/remainder trace diverged")
    );
    assert_eq!(
        validate_nested_handoff(HandoffDefect::CoupleAxes),
        Err("nested applied/remainder trace diverged")
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceDefect {
    None,
    QuantizeContinuous,
    IgnoreAbsoluteSources,
    ReverseKeyboard,
}

fn source_result(source: InputSource, defect: SourceDefect) -> f64 {
    let mut axis = OracleAxis::new(0.0, 100.0, 20.0, 0.0);
    match source {
        InputSource::Wheel => {
            axis.apply(12.0);
        }
        InputSource::Touchpad => {
            axis.apply(if defect == SourceDefect::QuantizeContinuous {
                0.625_f64.round()
            } else {
                0.625
            });
        }
        InputSource::Touchscreen => {
            axis.apply(18.0);
        }
        InputSource::Scrollbar => {
            if defect != SourceDefect::IgnoreAbsoluteSources {
                axis.set(42.0);
            }
        }
        InputSource::Keyboard => {
            axis.apply(if defect == SourceDefect::ReverseKeyboard {
                -axis.configuration.step
            } else {
                axis.configuration.step
            });
        }
        InputSource::Reveal => {
            if defect != SourceDefect::IgnoreAbsoluteSources {
                axis.set(reveal_axis(axis.value(), axis.maximum(), 50.0, 75.0, 10.0));
            }
        }
        InputSource::Programmatic => {
            if defect != SourceDefect::IgnoreAbsoluteSources {
                axis.set(64.0);
            }
        }
    }
    axis.value()
}

fn validate_all_sources(defect: SourceDefect) -> Result<(), &'static str> {
    let expected = [
        (InputSource::Wheel, 12.0),
        (InputSource::Touchpad, 0.625),
        (InputSource::Touchscreen, 18.0),
        (InputSource::Scrollbar, 42.0),
        (InputSource::Keyboard, 1.0),
        (InputSource::Reveal, 35.0),
        (InputSource::Programmatic, 64.0),
    ];
    for (source, expected) in expected {
        if (source_result(source, defect) - expected).abs() > EPSILON {
            return Err(match source {
                InputSource::Touchpad => "continuous source diverged",
                InputSource::Keyboard => "keyboard source diverged",
                InputSource::Scrollbar | InputSource::Reveal | InputSource::Programmatic => {
                    "absolute source diverged"
                }
                InputSource::Wheel | InputSource::Touchscreen => "direct source diverged",
            });
        }
    }
    Ok(())
}

#[test]
fn scrolling_oracle_normalizes_every_input_source_into_one_axis_law() {
    validate_all_sources(SourceDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_source_specific_bypasses() {
    assert_eq!(
        validate_all_sources(SourceDefect::QuantizeContinuous),
        Err("continuous source diverged")
    );
    assert_eq!(
        validate_all_sources(SourceDefect::IgnoreAbsoluteSources),
        Err("absolute source diverged")
    );
    assert_eq!(
        validate_all_sources(SourceDefect::ReverseKeyboard),
        Err("keyboard source diverged")
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisPolicy {
    Always,
    Automatic,
    Never,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChromePresentation {
    Overlay,
    Consuming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolicyDefect {
    None,
    SinglePass,
    OverlayConsumesSpace,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ContainerLayout {
    horizontal: bool,
    vertical: bool,
    viewport: Vector,
    introduction_passes: usize,
}

fn policy_shows(policy: AxisPolicy, overflow: bool) -> bool {
    match policy {
        AxisPolicy::Always => true,
        AxisPolicy::Automatic => overflow,
        AxisPolicy::Never | AxisPolicy::External => false,
    }
}

fn layout_container(
    horizontal_policy: AxisPolicy,
    vertical_policy: AxisPolicy,
    presentation: ChromePresentation,
    content: Vector,
    available: Vector,
    thickness: f64,
    defect: PolicyDefect,
) -> ContainerLayout {
    let consumes = presentation == ChromePresentation::Consuming
        || defect == PolicyDefect::OverlayConsumesSpace;
    let mut horizontal = false;
    let mut vertical = false;
    let mut introduction_passes = 0;
    let pass_limit = if defect == PolicyDefect::SinglePass {
        1
    } else {
        3
    };

    for _ in 0..pass_limit {
        let viewport = Vector::new(
            available.x - if consumes && vertical { thickness } else { 0.0 },
            available.y
                - if consumes && horizontal {
                    thickness
                } else {
                    0.0
                },
        );
        let next_horizontal =
            horizontal || policy_shows(horizontal_policy, content.x > viewport.x + EPSILON);
        let next_vertical =
            vertical || policy_shows(vertical_policy, content.y > viewport.y + EPSILON);
        if next_horizontal == horizontal && next_vertical == vertical {
            break;
        }
        horizontal = next_horizontal;
        vertical = next_vertical;
        introduction_passes += 1;
    }

    ContainerLayout {
        horizontal,
        vertical,
        viewport: Vector::new(
            available.x - if consumes && vertical { thickness } else { 0.0 },
            available.y
                - if consumes && horizontal {
                    thickness
                } else {
                    0.0
                },
        ),
        introduction_passes,
    }
}

fn validate_policy_convergence(defect: PolicyDefect) -> Result<(), &'static str> {
    let result = layout_container(
        AxisPolicy::Automatic,
        AxisPolicy::Automatic,
        ChromePresentation::Consuming,
        Vector::new(95.0, 110.0),
        Vector::new(100.0, 100.0),
        10.0,
        defect,
    );
    if !result.horizontal || !result.vertical || result.viewport != Vector::new(90.0, 90.0) {
        return Err("cross-axis scrollbar convergence diverged");
    }
    if result.introduction_passes > 2 {
        return Err("scrollbar convergence exceeded two introduction passes");
    }

    let overlay = layout_container(
        AxisPolicy::Automatic,
        AxisPolicy::Automatic,
        ChromePresentation::Overlay,
        Vector::new(120.0, 110.0),
        Vector::new(100.0, 100.0),
        10.0,
        defect,
    );
    if overlay.viewport != Vector::new(100.0, 100.0) {
        return Err("overlay presentation consumed layout space");
    }

    let policy_cases = [
        (AxisPolicy::Always, false, true),
        (AxisPolicy::Automatic, true, true),
        (AxisPolicy::Never, true, false),
        (AxisPolicy::External, true, false),
    ];
    for (policy, overflow, expected) in policy_cases {
        if policy_shows(policy, overflow) != expected {
            return Err("per-axis policy diverged");
        }
    }
    Ok(())
}

#[test]
fn scrolling_oracle_separates_axis_policy_presentation_and_monotonic_convergence() {
    validate_policy_convergence(PolicyDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_single_pass_and_overlay_consumption_adapters() {
    assert_eq!(
        validate_policy_convergence(PolicyDefect::SinglePass),
        Err("cross-axis scrollbar convergence diverged")
    );
    assert_eq!(
        validate_policy_convergence(PolicyDefect::OverlayConsumesSpace),
        Err("overlay presentation consumed layout space")
    );
}

fn reveal_axis(
    current: f64,
    maximum: f64,
    viewport_extent: f64,
    target_start: f64,
    target_extent: f64,
) -> f64 {
    let target_end = target_start + target_extent;
    let visible_start = current;
    let visible_end = current + viewport_extent;
    let delta = if target_extent > viewport_extent || target_start < visible_start {
        target_start - visible_start
    } else if target_end > visible_end {
        target_end - visible_end
    } else {
        0.0
    };
    (current + delta).clamp(0.0, maximum)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RevealDefect {
    None,
    ChildOnly,
    AlignStart,
}

fn nested_reveal(defect: RevealDefect) -> (Vector, Vector) {
    let target = Vector::new(140.0, 140.0);
    let target_extent = Vector::new(10.0, 10.0);
    let inner = if defect == RevealDefect::AlignStart {
        target
    } else {
        Vector::new(
            reveal_axis(0.0, 300.0, 100.0, target.x, target_extent.x),
            reveal_axis(0.0, 300.0, 100.0, target.y, target_extent.y),
        )
    };
    if defect == RevealDefect::ChildOnly {
        return (inner, Vector::default());
    }

    let inner_origin_in_outer = Vector::new(150.0, 180.0);
    let target_in_outer = inner_origin_in_outer.plus(target.minus(inner));
    let outer = Vector::new(
        reveal_axis(0.0, 400.0, 200.0, target_in_outer.x, target_extent.x),
        reveal_axis(0.0, 400.0, 200.0, target_in_outer.y, target_extent.y),
    );
    (inner, outer)
}

fn validate_nested_reveal(defect: RevealDefect) -> Result<(), &'static str> {
    let (inner, outer) = nested_reveal(defect);
    if inner != Vector::new(50.0, 50.0) {
        return Err("inner reveal was not minimal");
    }
    if outer != Vector::new(50.0, 80.0) {
        return Err("focus reveal did not traverse every ancestor");
    }
    Ok(())
}

#[test]
fn scrolling_oracle_reveals_focus_minimally_through_nested_ancestors() {
    validate_nested_reveal(RevealDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_child_only_and_align_start_reveal_adapters() {
    assert_eq!(
        validate_nested_reveal(RevealDefect::ChildOnly),
        Err("focus reveal did not traverse every ancestor")
    );
    assert_eq!(
        validate_nested_reveal(RevealDefect::AlignStart),
        Err("inner reveal was not minimal")
    );
}

type ItemKey = u64;
type SlotId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Item {
    key: ItemKey,
    revision: u64,
}

impl Item {
    const fn new(key: ItemKey, revision: u64) -> Self {
        Self { key, revision }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Setup(SlotId),
    Bind { slot: SlotId, item: Item },
    Unbind { slot: SlotId, item: Item },
    Teardown(SlotId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListDefect {
    None,
    AllowDuplicate,
    IdentityByPosition,
    IgnoreSameKeyRevision,
    LeakDepartedLogicalState,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LogicalItems {
    focused: Option<ItemKey>,
    captured: Option<ItemKey>,
    editors: BTreeSet<ItemKey>,
    popup_anchors: BTreeSet<ItemKey>,
}

impl LogicalItems {
    fn retain_members(&mut self, keys: &BTreeSet<ItemKey>) {
        if self.focused.is_some_and(|key| !keys.contains(&key)) {
            self.focused = None;
        }
        if self.captured.is_some_and(|key| !keys.contains(&key)) {
            self.captured = None;
        }
        self.editors.retain(|key| keys.contains(key));
        self.popup_anchors.retain(|key| keys.contains(key));
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct FactoryModel {
    bindings: BTreeMap<ItemKey, (SlotId, Item)>,
    free_slots: Vec<SlotId>,
    next_slot: SlotId,
    lifecycle: Vec<Lifecycle>,
}

impl FactoryModel {
    fn synchronize(
        &mut self,
        previous: &[Item],
        next: &[Item],
        defect: ListDefect,
    ) -> Result<(), &'static str> {
        let mut unique = BTreeSet::new();
        if defect != ListDefect::AllowDuplicate && next.iter().any(|item| !unique.insert(item.key))
        {
            return Err("duplicate stable key");
        }

        if defect == ListDefect::IdentityByPosition {
            self.synchronize_by_position(next);
            return Ok(());
        }

        let next_keys = next.iter().map(|item| item.key).collect::<BTreeSet<_>>();
        for item in previous
            .iter()
            .copied()
            .filter(|item| !next_keys.contains(&item.key))
        {
            if let Some((slot, bound)) = self.bindings.remove(&item.key) {
                self.lifecycle.push(Lifecycle::Unbind { slot, item: bound });
                self.free_slots.push(slot);
            }
        }

        for item in next.iter().copied() {
            if let Some((slot, bound)) = self.bindings.get_mut(&item.key) {
                if bound.revision != item.revision && defect != ListDefect::IgnoreSameKeyRevision {
                    self.lifecycle.push(Lifecycle::Unbind {
                        slot: *slot,
                        item: *bound,
                    });
                    *bound = item;
                    self.lifecycle.push(Lifecycle::Bind { slot: *slot, item });
                }
                continue;
            }

            let slot = if let Some(slot) = self.free_slots.pop() {
                slot
            } else {
                self.next_slot += 1;
                self.lifecycle.push(Lifecycle::Setup(self.next_slot));
                self.next_slot
            };
            self.bindings.insert(item.key, (slot, item));
            self.lifecycle.push(Lifecycle::Bind { slot, item });
        }
        Ok(())
    }

    fn synchronize_by_position(&mut self, next: &[Item]) {
        let mut previous_by_slot = self
            .bindings
            .values()
            .copied()
            .collect::<Vec<(SlotId, Item)>>();
        previous_by_slot.sort_by_key(|(slot, _)| *slot);
        self.bindings.clear();
        for (index, item) in next.iter().copied().enumerate() {
            let (slot, previous) = previous_by_slot.get(index).copied().unwrap_or_else(|| {
                self.next_slot += 1;
                self.lifecycle.push(Lifecycle::Setup(self.next_slot));
                (self.next_slot, item)
            });
            if previous != item {
                self.lifecycle.push(Lifecycle::Unbind {
                    slot,
                    item: previous,
                });
                self.lifecycle.push(Lifecycle::Bind { slot, item });
            }
            self.bindings.insert(item.key, (slot, item));
        }
    }

    fn teardown_all(&mut self) {
        let bindings = std::mem::take(&mut self.bindings);
        for (_, (slot, item)) in bindings {
            self.lifecycle.push(Lifecycle::Unbind { slot, item });
            self.lifecycle.push(Lifecycle::Teardown(slot));
        }
        for slot in self.free_slots.drain(..) {
            self.lifecycle.push(Lifecycle::Teardown(slot));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListModel {
    items: Vec<Item>,
    logical: LogicalItems,
    factory: FactoryModel,
    defect: ListDefect,
}

impl ListModel {
    fn new(items: Vec<Item>, defect: ListDefect) -> Self {
        let mut model = Self {
            items: Vec::new(),
            logical: LogicalItems::default(),
            factory: FactoryModel::default(),
            defect,
        };
        model.replace(items).unwrap();
        model.factory.lifecycle.clear();
        model
    }

    fn replace(&mut self, next: Vec<Item>) -> Result<(), &'static str> {
        let previous = self.items.clone();
        self.factory.synchronize(&previous, &next, self.defect)?;

        if self.defect == ListDefect::IdentityByPosition {
            let remap = |key: ItemKey| {
                previous
                    .iter()
                    .position(|item| item.key == key)
                    .and_then(|position| next.get(position))
                    .map(|item| item.key)
            };
            self.logical.focused = self.logical.focused.and_then(remap);
            self.logical.captured = self.logical.captured.and_then(remap);
        } else if self.defect != ListDefect::LeakDepartedLogicalState {
            let keys = next.iter().map(|item| item.key).collect();
            self.logical.retain_members(&keys);
        }
        self.items = next;
        Ok(())
    }
}

fn validate_list_lifecycle(defect: ListDefect) -> Result<(), &'static str> {
    let a = Item::new(1, 1);
    let b = Item::new(2, 1);
    let c = Item::new(3, 1);
    let mut model = ListModel::new(vec![a, b, c], defect);
    model.logical.focused = Some(b.key);
    model.logical.captured = Some(b.key);
    model.logical.editors.insert(b.key);
    model.logical.popup_anchors.insert(b.key);

    model.replace(vec![c, a, b])?;
    if model.logical.focused != Some(b.key) || !model.factory.lifecycle.is_empty() {
        return Err("reorder did not preserve stable item and slot identity");
    }

    let b_revised = Item::new(b.key, 2);
    model.replace(vec![c, a, b_revised])?;
    let b_slot = model.factory.bindings[&b.key].0;
    let expected_revision_lifecycle = [
        Lifecycle::Unbind {
            slot: b_slot,
            item: b,
        },
        Lifecycle::Bind {
            slot: b_slot,
            item: b_revised,
        },
    ];
    if model.factory.lifecycle.as_slice() != expected_revision_lifecycle {
        return Err("same-key revision did not rebind exactly one slot");
    }

    model.factory.lifecycle.clear();
    model.replace(vec![c, a])?;
    if model.logical.focused.is_some()
        || model.logical.captured.is_some()
        || !model.logical.editors.is_empty()
        || !model.logical.popup_anchors.is_empty()
    {
        return Err("departed item retained logical interaction state");
    }
    if model.factory.lifecycle
        != [Lifecycle::Unbind {
            slot: b_slot,
            item: b_revised,
        }]
    {
        return Err("deletion touched an unaffected binding");
    }

    let d = Item::new(4, 1);
    model.factory.lifecycle.clear();
    model.replace(vec![c, a, d])?;
    if model.factory.lifecycle
        != [Lifecycle::Bind {
            slot: b_slot,
            item: d,
        }]
    {
        return Err("entering item did not reuse the departed slot");
    }

    let before_duplicate = model.clone();
    if model.replace(vec![c, c, a]).is_ok() {
        return Err("duplicate stable key was accepted");
    }
    if defect == ListDefect::None && model != before_duplicate {
        return Err("rejected mutation changed list state");
    }

    model.factory.lifecycle.clear();
    model.factory.teardown_all();
    let teardown_count = model
        .factory
        .lifecycle
        .iter()
        .filter(|event| matches!(event, Lifecycle::Teardown(_)))
        .count();
    if teardown_count != 3 {
        return Err("teardown did not retire every slot exactly once");
    }
    Ok(())
}

#[test]
fn scrolling_oracle_separates_list_membership_item_identity_and_slot_lifecycle() {
    validate_list_lifecycle(ListDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_list_identity_revision_duplicate_and_cleanup_defects() {
    assert_eq!(
        validate_list_lifecycle(ListDefect::IdentityByPosition),
        Err("reorder did not preserve stable item and slot identity")
    );
    assert_eq!(
        validate_list_lifecycle(ListDefect::IgnoreSameKeyRevision),
        Err("same-key revision did not rebind exactly one slot")
    );
    assert_eq!(
        validate_list_lifecycle(ListDefect::LeakDepartedLogicalState),
        Err("departed item retained logical interaction state")
    );
    assert_eq!(
        validate_list_lifecycle(ListDefect::AllowDuplicate),
        Err("duplicate stable key was accepted")
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnchorDefect {
    None,
    IgnoreMeasurementCorrection,
}

fn anchored_scroll_after_measurement(defect: AnchorDefect) -> f64 {
    let before = [(1, 20.0), (2, 20.0), (3, 20.0)];
    let after = [(1, 30.0), (2, 20.0), (3, 20.0)];
    let anchor_key = 3;
    let within_item = 5.0;
    let prefix = |items: &[(ItemKey, f64)]| {
        items
            .iter()
            .take_while(|(key, _)| *key != anchor_key)
            .map(|(_, extent)| extent)
            .sum::<f64>()
    };
    let before_scroll = prefix(&before) + within_item;
    if defect == AnchorDefect::IgnoreMeasurementCorrection {
        before_scroll
    } else {
        before_scroll + prefix(&after) - prefix(&before)
    }
}

fn validate_anchor_correction(defect: AnchorDefect) -> Result<(), &'static str> {
    if (anchored_scroll_after_measurement(defect) - 55.0).abs() > EPSILON {
        return Err("variable-extent correction moved the stable visible anchor");
    }
    Ok(())
}

#[test]
fn scrolling_oracle_corrects_variable_extent_measurement_around_a_stable_anchor() {
    validate_anchor_correction(AnchorDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_missing_variable_extent_anchor_correction() {
    assert_eq!(
        validate_anchor_correction(AnchorDefect::IgnoreMeasurementCorrection),
        Err("variable-extent correction moved the stable visible anchor")
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AccessibleAction {
    StepBackward,
    StepForward,
    PageBackward,
    PageForward,
    ToStart,
    ToEnd,
    SetValue,
}

#[derive(Debug, Clone, PartialEq)]
struct AccessibleAxis {
    lower: f64,
    upper: f64,
    page: f64,
    value: f64,
    actions: BTreeSet<AccessibleAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessibilityDefect {
    None,
    ProjectResidentValue,
    OmitActions,
}

fn accessible_projection(defect: AccessibilityDefect) -> AccessibleAxis {
    let actions = if defect == AccessibilityDefect::OmitActions {
        BTreeSet::new()
    } else {
        [
            AccessibleAction::StepBackward,
            AccessibleAction::StepForward,
            AccessibleAction::PageBackward,
            AccessibleAction::PageForward,
            AccessibleAction::ToStart,
            AccessibleAction::ToEnd,
            AccessibleAction::SetValue,
        ]
        .into_iter()
        .collect()
    };
    AccessibleAxis {
        lower: 0.0,
        upper: 500.0,
        page: 100.0,
        value: if defect == AccessibilityDefect::ProjectResidentValue {
            100.0
        } else {
            125.0
        },
        actions,
    }
}

fn validate_accessible_projection(defect: AccessibilityDefect) -> Result<(), &'static str> {
    let projection = accessible_projection(defect);
    if projection.lower != 0.0
        || projection.upper != 500.0
        || projection.page != 100.0
        || projection.value != 125.0
    {
        return Err("accessible range/value did not project the canonical adjustment");
    }
    if projection.actions.len() != 7 {
        return Err("accessible scrolling actions were incomplete");
    }
    Ok(())
}

#[test]
fn scrolling_oracle_projects_accessible_range_value_page_and_actions() {
    validate_accessible_projection(AccessibilityDefect::None).unwrap();
}

#[test]
fn scrolling_oracle_rejects_stale_accessible_values_and_missing_actions() {
    assert_eq!(
        validate_accessible_projection(AccessibilityDefect::ProjectResidentValue),
        Err("accessible range/value did not project the canonical adjustment")
    );
    assert_eq!(
        validate_accessible_projection(AccessibilityDefect::OmitActions),
        Err("accessible scrolling actions were incomplete")
    );
}

#[test]
fn scrolling_oracle_preserves_source_units_and_monotonic_timestamps() {
    let units = [Unit::Pixel, Unit::Line, Unit::Page, Unit::Absolute];
    let mut session = SessionModel::default();
    for (index, unit) in units.into_iter().enumerate() {
        let timestamp = (index as u64) * 10;
        let mut begin = SessionEvent::new(
            InputSource::Touchscreen,
            timestamp,
            Phase::Begin,
            Vector::default(),
        );
        begin.unit = unit;
        session.handle(SessionDefect::None, begin).unwrap();
        assert_eq!(session.last_unit, Some(unit));

        let mut cancel = SessionEvent::new(
            InputSource::Touchscreen,
            timestamp + 1,
            Phase::Cancel,
            Vector::default(),
        );
        cancel.unit = unit;
        session.handle(SessionDefect::None, cancel).unwrap();
        assert_eq!(session.last_unit, Some(unit));
    }

    assert_eq!(
        session.handle(
            SessionDefect::None,
            SessionEvent::new(
                InputSource::Touchscreen,
                20,
                Phase::Begin,
                Vector::default(),
            ),
        ),
        Err("session timestamps regressed")
    );
}
