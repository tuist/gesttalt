use gpui::{
    Context, Entity, IntoElement, MouseButton, ParentElement, Render, Styled, Window, div,
    prelude::*, px,
};

use crate::auto_update::{AutoUpdateStatus, AutoUpdater};
use crate::theme;
use crate::title_bar::TitleBar;

pub const STATUS_BAR_HEIGHT: f32 = 24.0;

pub struct StatusBar {
    auto_updater: Entity<AutoUpdater>,
    title_bar: Entity<TitleBar>,
    _auto_update_subscription: gpui::Subscription,
    _title_bar_subscription: gpui::Subscription,
}

impl StatusBar {
    pub fn new(
        auto_updater: Entity<AutoUpdater>,
        title_bar: Entity<TitleBar>,
        cx: &mut Context<Self>,
    ) -> Self {
        let _auto_update_subscription = cx.observe(&auto_updater, |_, _, cx| cx.notify());
        let _title_bar_subscription = cx.observe(&title_bar, |_, _, cx| cx.notify());
        Self {
            auto_updater,
            title_bar,
            _auto_update_subscription,
            _title_bar_subscription,
        }
    }

    fn render_update_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.auto_updater.read(cx).status().clone();
        let label = match &status {
            AutoUpdateStatus::Idle => {
                format!("v{}", self.auto_updater.read(cx).current_version())
            }
            AutoUpdateStatus::Checking => "Checking for updates".to_string(),
            AutoUpdateStatus::UpToDate => "Up to date".to_string(),
            AutoUpdateStatus::Available {
                version,
                asset_name,
                ..
            } => match asset_name {
                Some(asset_name) => format!("Update v{version} available: {asset_name}"),
                None => format!("Update v{version} available"),
            },
            AutoUpdateStatus::Downloading { version } => {
                format!("Downloading update v{version}")
            }
            AutoUpdateStatus::Installing { version } => {
                format!("Installing update v{version}")
            }
            AutoUpdateStatus::Updated { version } => format!("Restart to update v{version}"),
            AutoUpdateStatus::Errored(error) => format!("Update check failed: {error}"),
        };

        let is_clickable = matches!(
            status,
            AutoUpdateStatus::Available { .. } | AutoUpdateStatus::Updated { .. }
        );
        let mut element = div().child(label);
        if is_clickable {
            let updater = self.auto_updater.clone();
            element = element
                .cursor_pointer()
                .text_color(theme::accent())
                .hover(|style| style.bg(theme::hover()))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    updater.update(cx, |updater, cx| match updater.status() {
                        AutoUpdateStatus::Available { .. } => updater.install_update(cx),
                        AutoUpdateStatus::Updated { .. } => updater.restart(cx),
                        _ => {}
                    });
                });
        }
        element
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.auto_updater.read(cx).status().clone();
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
            .child(
                if self.title_bar.read(cx).show_update_in_status_bar(&status) {
                    self.render_update_status(cx).into_any_element()
                } else {
                    div()
                        .child(format!("v{}", self.auto_updater.read(cx).current_version()))
                        .into_any_element()
                },
            )
            .child(div().child("0 agents"))
    }
}
