use gpui::{
    Bounds, Context, DragMoveEvent, Entity, FocusHandle, Focusable, IntoElement, MouseButton,
    ParentElement, Pixels, Point, Render, Styled, Window, actions, canvas, deferred, div,
    prelude::*, px,
};

use crate::command_palette::{
    Command, CommandHandler, CommandPalette, Toggle as ToggleCommandPalette,
};
use crate::dock::{Dock, DockPosition, DraggedDock, RESIZE_HANDLE_SIZE};
use crate::settings::SettingsView;
use crate::status_bar::StatusBar;
use crate::theme;
use crate::title_bar::TitleBar;

const MIN_HORIZONTAL_DOCK_SIZE: Pixels = px(120.0);
const MAX_HORIZONTAL_DOCK_SIZE: Pixels = px(600.0);
const MIN_BOTTOM_DOCK_SIZE: Pixels = px(80.0);
const MAX_BOTTOM_DOCK_SIZE: Pixels = px(600.0);

actions!(
    workspace,
    [
        /// Opens the settings view in the workspace center.
        OpenSettings,
        /// Returns the workspace center to the default view.
        CloseSettings,
    ]
);

enum CenterView {
    Default,
    Settings(Entity<SettingsView>),
}

pub struct Workspace {
    title_bar: Entity<TitleBar>,
    left_dock: Entity<Dock>,
    right_dock: Entity<Dock>,
    bottom_dock: Entity<Dock>,
    status_bar: Entity<StatusBar>,
    main_bounds: Bounds<Pixels>,
    previous_drag_position: Option<Point<Pixels>>,
    focus_handle: FocusHandle,
    command_palette: Option<Entity<CommandPalette>>,
    center: CenterView,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            title_bar: cx.new(TitleBar::new),
            left_dock: cx.new(|_| Dock::new(DockPosition::Left)),
            right_dock: cx.new(|_| Dock::new(DockPosition::Right)),
            bottom_dock: cx.new(|_| Dock::new(DockPosition::Bottom)),
            status_bar: cx.new(|_| StatusBar::new()),
            main_bounds: Bounds::default(),
            previous_drag_position: None,
            focus_handle: cx.focus_handle(),
            command_palette: None,
            center: CenterView::Default,
        }
    }

    fn handle_dock_drag(
        &mut self,
        event: &DragMoveEvent<DraggedDock>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.previous_drag_position == Some(event.event.position) {
            return;
        }
        self.previous_drag_position = Some(event.event.position);

        let position = event.drag(cx).0;
        let cursor = event.event.position;
        let bounds = self.main_bounds;

        match position {
            DockPosition::Left => {
                let new_size = (cursor.x - bounds.left())
                    .max(MIN_HORIZONTAL_DOCK_SIZE)
                    .min(MAX_HORIZONTAL_DOCK_SIZE);
                self.left_dock.update(cx, |dock, cx| {
                    dock.set_size(new_size);
                    cx.notify();
                });
            }
            DockPosition::Right => {
                let new_size = (bounds.right() - cursor.x)
                    .max(MIN_HORIZONTAL_DOCK_SIZE)
                    .min(MAX_HORIZONTAL_DOCK_SIZE);
                self.right_dock.update(cx, |dock, cx| {
                    dock.set_size(new_size);
                    cx.notify();
                });
            }
            DockPosition::Bottom => {
                let new_size = (bounds.bottom() - cursor.y - RESIZE_HANDLE_SIZE)
                    .max(MIN_BOTTOM_DOCK_SIZE)
                    .min(MAX_BOTTOM_DOCK_SIZE);
                self.bottom_dock.update(cx, |dock, cx| {
                    dock.set_size(new_size);
                    cx.notify();
                });
            }
        }
    }

    fn toggle_command_palette(
        &mut self,
        _: &ToggleCommandPalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_palette.is_some() {
            self.dismiss_command_palette(window, cx);
        } else {
            self.open_command_palette(window, cx);
        }
    }

    fn open_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = cx.entity().downgrade();
        let commands = default_commands();
        let palette = cx.new(|cx| CommandPalette::new(workspace, commands, cx));
        window.focus(&palette.read(cx).focus_handle());
        self.command_palette = Some(palette);
        cx.notify();
    }

    pub fn dismiss_command_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.command_palette.take().is_some() {
            window.focus(&self.focus_handle);
            cx.notify();
        }
    }

    fn open_settings(&mut self, _: &OpenSettings, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.center, CenterView::Settings(_)) {
            return;
        }
        let view = cx.new(SettingsView::new);
        self.center = CenterView::Settings(view);
        cx.notify();
    }

    fn close_settings(&mut self, _: &CloseSettings, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.center, CenterView::Default) {
            return;
        }
        self.center = CenterView::Default;
        cx.notify();
    }

    fn render_default_center(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(theme::text_muted())
            .child("Center")
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let bounds_canvas = canvas(
            move |bounds, _window, cx| {
                entity.update(cx, |this, _| {
                    this.main_bounds = bounds;
                });
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();

        let palette_overlay = self.command_palette.as_ref().map(|palette| {
            deferred(
                div()
                    .absolute()
                    .inset_0()
                    .bg(theme::modal_backdrop())
                    .flex()
                    .flex_col()
                    .items_center()
                    .pt(px(96.))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| this.dismiss_command_palette(window, cx)),
                    )
                    .child(palette.clone()),
            )
            .with_priority(2)
        });

        let body = match &self.center {
            CenterView::Settings(view) => div()
                .relative()
                .flex_1()
                .overflow_hidden()
                .child(bounds_canvas)
                .child(view.clone().into_any_element()),
            CenterView::Default => div()
                .relative()
                .flex_1()
                .flex()
                .flex_row()
                .overflow_hidden()
                .child(bounds_canvas)
                .on_drag_move(cx.listener(Self::handle_dock_drag))
                .child(self.left_dock.clone())
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(self.render_default_center())
                        .child(self.bottom_dock.clone()),
                )
                .child(self.right_dock.clone()),
        };

        div()
            .key_context("Workspace")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_command_palette))
            .on_action(cx.listener(Self::open_settings))
            .on_action(cx.listener(Self::close_settings))
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::background())
            .text_color(theme::text())
            .child(self.title_bar.clone())
            .child(body)
            .child(self.status_bar.clone())
            .when_some(palette_overlay, |this, overlay| this.child(overlay))
    }
}

fn default_commands() -> Vec<Command> {
    vec![
        Command {
            name: "Settings: Open".into(),
            keybinding: None,
            action: open_settings_handler(),
        },
        Command {
            name: "Settings: Close".into(),
            keybinding: None,
            action: close_settings_handler(),
        },
        Command {
            name: "Workspace: Reset Docks".into(),
            keybinding: None,
            action: reset_docks_handler(),
        },
    ]
}

fn open_settings_handler() -> CommandHandler {
    Box::new(|workspace, window, cx| {
        workspace.open_settings(&OpenSettings, window, cx);
    })
}

fn close_settings_handler() -> CommandHandler {
    Box::new(|workspace, window, cx| {
        workspace.close_settings(&CloseSettings, window, cx);
    })
}

fn reset_docks_handler() -> CommandHandler {
    Box::new(|workspace, _, cx| {
        workspace.left_dock.update(cx, |dock, cx| {
            dock.set_size(px(240.0));
            cx.notify();
        });
        workspace.right_dock.update(cx, |dock, cx| {
            dock.set_size(px(240.0));
            cx.notify();
        });
        workspace.bottom_dock.update(cx, |dock, cx| {
            dock.set_size(px(200.0));
            cx.notify();
        });
    })
}
