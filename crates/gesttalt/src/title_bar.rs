use gpui::{
    Context, Entity, IntoElement, MouseButton, ParentElement, Render, SharedString, Styled, Window,
    div, prelude::*, px,
};

use crate::auto_update::{AutoUpdateStatus, AutoUpdater};
use crate::theme;

pub const TITLE_BAR_HEIGHT: f32 = 32.0;
pub const TRAFFIC_LIGHTS_RESERVED: f32 = 72.0;

pub struct TitleBar {
    pub project_name: SharedString,
    auto_updater: Entity<AutoUpdater>,
    dismissed_update: Option<AutoUpdateStatus>,
    _auto_update_subscription: gpui::Subscription,
}

impl TitleBar {
    pub fn new(auto_updater: Entity<AutoUpdater>, cx: &mut Context<Self>) -> Self {
        let _auto_update_subscription = cx.observe(&auto_updater, |this, updater, cx| {
            let status = updater.read(cx).status().clone();
            if matches!(
                (&this.dismissed_update, &status),
                (
                    Some(AutoUpdateStatus::Available { .. }),
                    AutoUpdateStatus::Available { .. }
                ) | (
                    Some(AutoUpdateStatus::Updated { .. }),
                    AutoUpdateStatus::Updated { .. }
                ) | (
                    Some(AutoUpdateStatus::Errored(_)),
                    AutoUpdateStatus::Errored(_)
                )
            ) {
            } else {
                this.dismissed_update = None;
            }
            cx.notify();
        });

        Self {
            project_name: "Gesttalt".into(),
            auto_updater,
            dismissed_update: None,
            _auto_update_subscription,
        }
    }

    pub fn show_update_in_status_bar(&self, status: &AutoUpdateStatus) -> bool {
        self.dismissed_update.as_ref() == Some(status)
            && matches!(
                status,
                AutoUpdateStatus::Available { .. }
                    | AutoUpdateStatus::Updated { .. }
                    | AutoUpdateStatus::Errored(_)
            )
    }

    fn dismiss_update(&mut self, _: &gpui::MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.dismissed_update = Some(self.auto_updater.read(cx).status().clone());
        cx.notify();
    }

    fn render_update_button(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let status = self.auto_updater.read(cx).status().clone();
        if self.dismissed_update.as_ref() == Some(&status) {
            return None;
        }

        let (message, tooltip, is_clickable, is_dismissable, warning) = match status {
            AutoUpdateStatus::Checking => (
                "Checking for Gesttalt Updates".to_string(),
                None,
                false,
                false,
                false,
            ),
            AutoUpdateStatus::Available {
                version,
                asset_name,
                ..
            } => (
                "Download Gesttalt Update".to_string(),
                Some(update_tooltip(version, asset_name.as_deref())),
                true,
                true,
                false,
            ),
            AutoUpdateStatus::Downloading { version } => (
                "Downloading Gesttalt Update".to_string(),
                Some(update_tooltip(version, None)),
                false,
                false,
                false,
            ),
            AutoUpdateStatus::Installing { version } => (
                "Installing Gesttalt Update".to_string(),
                Some(update_tooltip(version, None)),
                false,
                false,
                false,
            ),
            AutoUpdateStatus::Updated { version } => (
                "Restart to Update Gesttalt".to_string(),
                Some(update_tooltip(version, None)),
                true,
                true,
                false,
            ),
            AutoUpdateStatus::Errored(error) => (
                "Failed to Update".to_string(),
                Some(error),
                false,
                true,
                true,
            ),
            AutoUpdateStatus::Idle | AutoUpdateStatus::UpToDate => return None,
        };

        let border_color = if warning {
            theme::warning()
        } else {
            theme::border()
        };
        let text_color = if warning {
            theme::warning()
        } else if is_clickable {
            theme::accent()
        } else {
            theme::text_muted()
        };

        let mut main_button = div()
            .id("title-bar-update-button-main")
            .h(px(22.))
            .px_2()
            .flex()
            .items_center()
            .text_color(text_color)
            .child(message);

        if let Some(tooltip) = tooltip {
            main_button = main_button.tooltip(move |_, cx| {
                cx.new(|_| UpdateTooltip {
                    text: tooltip.clone().into(),
                })
                .into()
            });
        }

        if is_clickable {
            let updater = self.auto_updater.clone();
            main_button = main_button
                .cursor_pointer()
                .hover(|style| style.bg(theme::hover()))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    updater.update(cx, |updater, cx| match updater.status() {
                        AutoUpdateStatus::Available { .. } => updater.install_update(cx),
                        AutoUpdateStatus::Updated { .. } => updater.restart(cx),
                        _ => {}
                    });
                });
        }

        let mut button = div()
            .id("title-bar-update-button")
            .mr_2()
            .h(px(22.))
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .rounded_sm()
            .border_1()
            .border_color(border_color)
            .text_xs()
            .overflow_hidden()
            .child(main_button);

        if is_dismissable {
            button = button.child(
                div()
                    .id("dismiss-update-button")
                    .h_full()
                    .px_2()
                    .flex()
                    .items_center()
                    .border_l_1()
                    .border_color(border_color)
                    .cursor_pointer()
                    .text_color(theme::text_muted())
                    .hover(|style| style.text_color(theme::text()))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::dismiss_update))
                    .child("x"),
            );
        }

        Some(button)
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(TITLE_BAR_HEIGHT))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pl(px(TRAFFIC_LIGHTS_RESERVED))
            .pr_3()
            .bg(theme::panel())
            .border_b_1()
            .border_color(theme::border())
            .text_color(theme::text_muted())
            .text_sm()
            .child(self.project_name.clone())
            .when_some(self.render_update_button(cx), |this, button| {
                this.child(button)
            })
    }
}

fn update_tooltip(version: semver::Version, asset_name: Option<&str>) -> String {
    match asset_name {
        Some(asset_name) => format!("Update to Version: {version} ({asset_name})"),
        None => format!("Update to Version: {version}"),
    }
}

struct UpdateTooltip {
    text: SharedString,
}

impl Render for UpdateTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .max_w(px(420.))
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(theme::border())
            .bg(theme::elevated())
            .text_color(theme::text_muted())
            .text_xs()
            .child(self.text.clone())
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::*;

    #[test]
    fn update_tooltip_includes_version() {
        assert_eq!(
            update_tooltip(Version::new(1, 2, 3), None),
            "Update to Version: 1.2.3"
        );
    }

    #[test]
    fn update_tooltip_includes_asset_name_when_present() {
        assert_eq!(
            update_tooltip(
                Version::new(1, 2, 3),
                Some("gesttalt-v1.2.3-macos-aarch64.tar.gz")
            ),
            "Update to Version: 1.2.3 (gesttalt-v1.2.3-macos-aarch64.tar.gz)"
        );
    }
}
