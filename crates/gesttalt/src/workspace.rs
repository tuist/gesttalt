use gpui::{
    Bounds, Context, DragMoveEvent, Entity, IntoElement, Pixels, Point, Render, Window, canvas,
    div, prelude::*, px,
};

use crate::dock::{Dock, DockPosition, DraggedDock, RESIZE_HANDLE_SIZE};
use crate::status_bar::StatusBar;
use crate::theme;
use crate::title_bar::TitleBar;

const MIN_HORIZONTAL_DOCK_SIZE: Pixels = px(120.0);
const MAX_HORIZONTAL_DOCK_SIZE: Pixels = px(600.0);
const MIN_BOTTOM_DOCK_SIZE: Pixels = px(80.0);
const MAX_BOTTOM_DOCK_SIZE: Pixels = px(600.0);

pub struct Workspace {
    title_bar: Entity<TitleBar>,
    left_dock: Entity<Dock>,
    right_dock: Entity<Dock>,
    bottom_dock: Entity<Dock>,
    status_bar: Entity<StatusBar>,
    main_bounds: Bounds<Pixels>,
    previous_drag_position: Option<Point<Pixels>>,
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

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::background())
            .text_color(theme::text())
            .child(self.title_bar.clone())
            .child(
                div()
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
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(theme::text_muted())
                                    .child("Center"),
                            )
                            .child(self.bottom_dock.clone()),
                    )
                    .child(self.right_dock.clone()),
            )
            .child(self.status_bar.clone())
    }
}
