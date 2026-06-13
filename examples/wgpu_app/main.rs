use wgpu_l3::{
    app,
    geometry::{Rect, area, point, rect},
    paint, window,
};

fn main() -> app::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    app::run(App::default())
}

#[derive(Default)]
struct App {
    window: Option<window::Id>,
}

impl app::Application for App {
    fn started(&mut self, cx: &mut app::Context<'_>) {
        let window = cx.open_window(window::Options {
            title: "wgpu_l3".to_owned(),
            inner_area: area::physical(512, 512),
            canvas_color: paint::Color::BLACK,
        });

        self.window = Some(window);
    }

    fn redraw(&mut self, cx: &mut app::Context<'_>, window: window::Id, scene: &mut paint::Scene) {
        if Some(window) != self.window {
            return;
        }

        scene.clear(paint::Color::BLACK);

        let Some(area) = cx.window_logical_area(window) else {
            return;
        };

        scene.push_quad(paint::Quad {
            rect: Rect {
                origin: point::logical(0.0, 0.0),
                area,
                radius: rect::Radius::none(),
            },
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::Solid(paint::Color::RED))),
                stroke: None,
                tint: None,
            },
        });
    }
}
