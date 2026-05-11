use anyhow::Result;
use gpui::{
    App, AppContext, Application, Bounds, Focusable, KeyBinding, Menu, MenuItem, Size,
    TitlebarOptions, WindowBounds, WindowOptions, actions, point, px, size,
};

mod auto_update;
mod command_palette;
mod dev_tools;
mod dock;
mod settings;
mod status_bar;
mod theme;
mod title_bar;
mod workspace;

use workspace::Workspace;

actions!(
    gesttalt,
    [
        /// Quits the application.
        Quit,
    ]
);

#[cfg(target_os = "macos")]
const COMMAND_PALETTE_KEY: &str = "cmd-k";
#[cfg(not(target_os = "macos"))]
const COMMAND_PALETTE_KEY: &str = "ctrl-k";

#[cfg(target_os = "macos")]
const QUIT_KEY: &str = "cmd-q";
#[cfg(not(target_os = "macos"))]
const QUIT_KEY: &str = "ctrl-q";

fn main() -> Result<()> {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        cx.on_action(|_: &Quit, cx| cx.quit());

        cx.bind_keys([
            KeyBinding::new(QUIT_KEY, Quit, None),
            KeyBinding::new(COMMAND_PALETTE_KEY, command_palette::Toggle, None),
            KeyBinding::new("up", command_palette::SelectPrev, Some("CommandPalette")),
            KeyBinding::new("down", command_palette::SelectNext, Some("CommandPalette")),
            KeyBinding::new("enter", command_palette::Confirm, Some("CommandPalette")),
            KeyBinding::new("escape", command_palette::Dismiss, Some("CommandPalette")),
        ]);

        cx.set_menus(vec![Menu {
            name: "Gesttalt".into(),
            items: vec![MenuItem::action("Quit Gesttalt", Quit)],
        }]);

        let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
        let window = cx
            .open_window(
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

        window
            .update(cx, |workspace, window, cx| {
                window.focus(&workspace.focus_handle(cx));
            })
            .ok();

        cx.activate(true);
    });

    Ok(())
}
