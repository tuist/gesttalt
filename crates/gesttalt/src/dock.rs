use gpui::{
    Context, Empty, IntoElement, MouseButton, MouseDownEvent, Pixels, Render, Window, deferred,
    div, prelude::*, px,
};

use crate::theme;

pub const RESIZE_HANDLE_SIZE: Pixels = px(6.0);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DockPosition {
    Left,
    Right,
    Bottom,
}

#[derive(Clone)]
pub struct DraggedDock(pub DockPosition);

impl Render for DraggedDock {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct Dock {
    position: DockPosition,
    size: Pixels,
}

impl Dock {
    pub fn new(position: DockPosition) -> Self {
        let size = match position {
            DockPosition::Left | DockPosition::Right => px(240.0),
            DockPosition::Bottom => px(200.0),
        };
        Self { position, size }
    }

    pub fn size(&self) -> Pixels {
        self.size
    }

    pub fn set_size(&mut self, size: Pixels) {
        self.size = size;
    }

    pub fn position(&self) -> DockPosition {
        self.position
    }
}

impl Render for Dock {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let position = self.position;

        let handle = div()
            .id("resize-handle")
            .occlude()
            .on_drag(DraggedDock(position), |dragged, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| dragged.clone())
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_, _: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                }),
            );

        let handle = match position {
            DockPosition::Left => handle
                .absolute()
                .top(px(0.0))
                .right(-(RESIZE_HANDLE_SIZE / 2.0))
                .h_full()
                .w(RESIZE_HANDLE_SIZE)
                .cursor_col_resize(),
            DockPosition::Right => handle
                .absolute()
                .top(px(0.0))
                .left(-(RESIZE_HANDLE_SIZE / 2.0))
                .h_full()
                .w(RESIZE_HANDLE_SIZE)
                .cursor_col_resize(),
            DockPosition::Bottom => handle
                .absolute()
                .left(px(0.0))
                .top(-(RESIZE_HANDLE_SIZE / 2.0))
                .w_full()
                .h(RESIZE_HANDLE_SIZE)
                .cursor_row_resize(),
        };

        let base = div()
            .relative()
            .flex_none()
            .bg(theme::panel())
            .border_color(theme::border())
            .text_color(theme::text_muted());

        let body = match position {
            DockPosition::Left => base.w(self.size).h_full().border_r_1(),
            DockPosition::Right => base.w(self.size).h_full().border_l_1(),
            DockPosition::Bottom => base.h(self.size).w_full().border_t_1(),
        };

        body.child(deferred(handle))
    }
}
