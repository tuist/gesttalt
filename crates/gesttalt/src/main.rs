use anyhow::Result;
use gpui::{
    App, AppContext, Application, Bounds, Size, TitlebarOptions, WindowBounds, WindowOptions,
    point, px, size,
};

mod dock;
mod status_bar;
mod theme;
mod title_bar;
mod workspace;

use workspace::Workspace;

fn main() -> Result<()> {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(9.0), px(9.0))),
                }),
                window_min_size: Some(Size {
                    width: px(640.),
                    height: px(400.),
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Workspace::new),
        )
        .expect("failed to open window");

        cx.activate(true);
    });

    Ok(())
}
