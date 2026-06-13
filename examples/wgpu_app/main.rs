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

        cx.set_action_state(
            ACTIVATE_PANEL,
            action::Context {
                window,
                target: Some(PANEL_A),
            },
            action::State {
                enabled: true,
                active: self.selected == Some(PANEL_A),
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
                active: self.selected == Some(PANEL_B),
            },
        );

        let root = ui::control::panel(ROOT)
            .with_background(paint::Color::BLACK)
            .with_padding(layout::Insets::splat(16.0))
            .with_child(ui::control::button(PANEL_A, ACTIVATE_PANEL))
            .with_child(ui::control::button(PANEL_B, ACTIVATE_PANEL));

        tree.set_root(root);
    }
}
