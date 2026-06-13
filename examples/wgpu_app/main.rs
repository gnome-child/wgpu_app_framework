use wgpu_l3::{Action, action, app, geometry::area, layout, paint, ui, window};

const ACTIVATE_PANEL: action::Id = action::Id::new("activate_panel");
const ROOT: ui::Id = ui::Id::new("root");
const PANEL_A: ui::Id = ui::Id::new("panel_a");
const PANEL_B: ui::Id = ui::Id::new("panel_b");

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(App::default())
}

#[derive(Default)]
struct App {
    window: Option<window::Id>,
    selected: Option<ui::Id>,
    panel_a_invoked: bool,
}

impl app::Application for App {
    fn started(&mut self, cx: &mut app::Context<'_>) {
        cx.register_action(Action::new(ACTIVATE_PANEL, "Activate Panel"));

        let window = cx.open_window(window::Options {
            title: "wgpu_l3".to_owned(),
            inner_area: area::physical(512, 512),
            canvas_color: paint::Color::BLACK,
        });

        self.window = Some(window);
    }

    fn event(&mut self, cx: &mut app::Context<'_>, window: window::Id, event: ui::Event) {
        if Some(window) != self.window {
            return;
        }

        if let ui::Event::ActionInvoked {
            action: ACTIVATE_PANEL,
            context,
            ..
        } = event
        {
            self.selected = context.target;
            self.panel_a_invoked |= context.target == Some(PANEL_A);
            cx.request_redraw(window);
        }
    }

    fn view(&mut self, cx: &mut app::Context<'_>, window: window::Id, tree: &mut ui::Tree) {
        if Some(window) != self.window {
            return;
        }

        let focused = cx.focused(window);

        cx.set_action_state(
            ACTIVATE_PANEL,
            action::Context {
                window,
                target: Some(PANEL_A),
            },
            action::State {
                enabled: true,
                active: focused == Some(PANEL_A),
            },
        );
        cx.set_action_state(
            ACTIVATE_PANEL,
            action::Context {
                window,
                target: Some(PANEL_B),
            },
            action::State {
                enabled: self.panel_a_invoked,
                active: focused == Some(PANEL_B),
            },
        );

        let root = ui::Node::container(ROOT, layout::Axis::Vertical)
            .with_background(paint::Color::BLACK)
            .with_padding(layout::Insets::splat(16.0))
            .with_child(self.panel(cx, window, PANEL_A))
            .with_child(self.panel(cx, window, PANEL_B));

        tree.set_root(root);
    }
}

impl App {
    fn panel(&self, cx: &app::Context<'_>, window: window::Id, id: ui::Id) -> ui::Node {
        ui::Node::leaf(id)
            .with_action(ACTIVATE_PANEL)
            .with_background(self.panel_color(cx, window, id))
            .with_disabled_background(paint::Color::rgb(0.12, 0.12, 0.12))
    }

    fn panel_color(&self, cx: &app::Context<'_>, window: window::Id, id: ui::Id) -> paint::Color {
        if self.selected == Some(id) {
            return paint::Color::rgb(0.10, 0.55, 0.28);
        }

        if cx.focused(window) == Some(id) {
            return paint::Color::rgb(0.12, 0.32, 0.72);
        }

        if cx.hovered(window) == Some(id) {
            return paint::Color::rgb(0.78, 0.18, 0.14);
        }

        paint::Color::rgb(0.22, 0.24, 0.28)
    }
}
