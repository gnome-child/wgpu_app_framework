use wgpu_l3::{Action, action, app, geometry::area, layout, paint, ui, window};

const PREPARE_WORKSPACE: action::Id = action::Id::new("prepare_workspace");
const RUN_TASK: action::Id = action::Id::new("run_task");
const ROOT: ui::Id = ui::Id::new("root");
const PREPARE_BUTTON: ui::Id = ui::Id::new("prepare_button");
const RUN_BUTTON: ui::Id = ui::Id::new("run_button");

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(App::default())
}

#[derive(Default)]
struct App {
    window: Option<window::Id>,
    workspace_ready: bool,
    run_count: u32,
}

impl app::Application for App {
    fn started(&mut self, cx: &mut app::Context<'_>) {
        cx.register_action(Action::new(PREPARE_WORKSPACE, "Prepare Workspace"));
        cx.register_action(Action::new(RUN_TASK, "Run Task"));

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

        if let ui::Event::ActionInvoked { action, .. } = event {
            match action {
                PREPARE_WORKSPACE => {
                    self.workspace_ready = true;
                    cx.request_redraw(window);
                }
                RUN_TASK => {
                    self.run_count += 1;
                    cx.request_redraw(window);
                }
                _ => {}
            }
        }
    }

    fn view(&mut self, cx: &mut app::Context<'_>, window: window::Id, tree: &mut ui::Tree) {
        if Some(window) != self.window {
            return;
        }

        cx.action(window, PREPARE_WORKSPACE)
            .enabled(true)
            .active(self.workspace_ready);
        cx.action(window, RUN_TASK)
            .enabled(self.workspace_ready)
            .active(false);

        let root = ui::control::panel(ROOT)
            .with_background(paint::Color::BLACK)
            .with_padding(layout::Insets::splat(16.0))
            .with_child(ui::control::labeled_button(
                PREPARE_BUTTON,
                PREPARE_WORKSPACE,
                "Prepare workspace",
            ))
            .with_child(ui::control::labeled_button(
                RUN_BUTTON, RUN_TASK, "Run task",
            ));

        tree.set_root(root);
    }
}
