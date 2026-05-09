use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*, px};

use crate::theme;

pub const TITLE_BAR_HEIGHT: f32 = 32.0;
pub const TRAFFIC_LIGHTS_RESERVED: f32 = 72.0;

pub struct TitleBar {
    pub project_name: SharedString,
}

impl TitleBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            project_name: "Gesttalt".into(),
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(TITLE_BAR_HEIGHT))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .pl(px(TRAFFIC_LIGHTS_RESERVED))
            .pr_3()
            .bg(theme::panel())
            .border_b_1()
            .border_color(theme::border())
            .text_color(theme::text_muted())
            .text_sm()
            .child(self.project_name.clone())
    }
}
