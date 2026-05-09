use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};

use crate::theme;

pub const STATUS_BAR_HEIGHT: f32 = 24.0;

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(STATUS_BAR_HEIGHT))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_3()
            .bg(theme::panel())
            .border_t_1()
            .border_color(theme::border())
            .text_color(theme::text_muted())
            .text_xs()
            .child(div().child("Ready"))
            .child(div().child("0 agents"))
    }
}
