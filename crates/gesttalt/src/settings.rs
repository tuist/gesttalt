use gpui::{
    AnyElement, Context, IntoElement, MouseButton, ParentElement, Render, SharedString, Styled,
    Window, div, prelude::*, px,
};

use crate::dev_tools::{DevTool, default_dev_tools};
use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Appearance,
    Agents,
    DevTools,
}

impl SettingsSection {
    fn title(self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Agents => "Agents",
            SettingsSection::DevTools => "Dev Tools",
        }
    }

    fn description(self) -> &'static str {
        match self {
            SettingsSection::General => "Workspace defaults, project root, and startup behavior.",
            SettingsSection::Appearance => "Theme, fonts, and chrome density.",
            SettingsSection::Agents => "CLI coding-agent adapters and their default models.",
            SettingsSection::DevTools => {
                "Internal tools you can attach to the workspace to debug Gesttalt itself."
            }
        }
    }

    fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::General,
            SettingsSection::Appearance,
            SettingsSection::Agents,
            SettingsSection::DevTools,
        ]
    }
}

pub struct SettingsView {
    selected: SettingsSection,
    dev_tools: Vec<DevTool>,
    active_dev_tool: Option<usize>,
}

impl SettingsView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            selected: SettingsSection::General,
            dev_tools: default_dev_tools(),
            active_dev_tool: None,
        }
    }

    fn select(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        if self.selected != section {
            self.selected = section;
            self.active_dev_tool = None;
            cx.notify();
        }
    }

    fn render_nav(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut nav = div()
            .w(px(220.))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .p_3()
            .gap_1()
            .border_r_1()
            .border_color(theme::border())
            .bg(theme::panel())
            .child(
                div()
                    .pb_2()
                    .text_color(theme::text_muted())
                    .text_xs()
                    .child("SETTINGS"),
            );

        for (index, &section) in SettingsSection::all().iter().enumerate() {
            let selected = section == self.selected;
            let label = section.title();
            let row = div()
                .id(("settings-nav", index))
                .px_2()
                .py_1p5()
                .rounded_md()
                .text_sm()
                .cursor_pointer()
                .child(label);
            let row = if selected {
                row.bg(theme::selection()).text_color(theme::text())
            } else {
                row.text_color(theme::text_muted())
                    .hover(|s| s.bg(theme::hover()).text_color(theme::text()))
            };
            nav = nav.child(row.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.select(section, cx);
                }),
            ));
        }

        nav
    }

    fn render_page(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.selected {
            SettingsSection::DevTools => self.render_dev_tools(cx).into_any_element(),
            section => self.render_placeholder_page(section).into_any_element(),
        }
    }

    fn render_placeholder_page(&self, section: SettingsSection) -> impl IntoElement {
        let title: SharedString = section.title().into();
        let description: SharedString = section.description().into();
        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .child(
                div()
                    .text_color(theme::text())
                    .text_xl()
                    .child(title.clone()),
            )
            .child(
                div()
                    .text_color(theme::text_muted())
                    .text_sm()
                    .child(description),
            )
            .child(
                div()
                    .mt_4()
                    .p_4()
                    .rounded_md()
                    .border_1()
                    .border_color(theme::border())
                    .bg(theme::panel())
                    .text_color(theme::text_muted())
                    .text_sm()
                    .child(format!("{} settings have not been implemented yet.", title)),
            )
    }

    fn render_dev_tools(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut page = div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .p_8()
            .gap_4()
            .child(
                div()
                    .text_color(theme::text())
                    .text_xl()
                    .child(SettingsSection::DevTools.title()),
            )
            .child(
                div()
                    .text_color(theme::text_muted())
                    .text_sm()
                    .child(SettingsSection::DevTools.description()),
            );

        let mut list = div().mt_2().flex().flex_col().gap_2().child(
            div()
                .text_color(theme::text_muted())
                .text_xs()
                .child("AVAILABLE TOOLS"),
        );

        for (index, tool) in self.dev_tools.iter().enumerate() {
            let active = self.active_dev_tool == Some(index);
            let name = tool.name.clone();
            let description = tool.description.clone();
            let card = div()
                .id(("dev-tool", index))
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(theme::border())
                .bg(theme::panel())
                .cursor_pointer()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(div().text_color(theme::text()).text_sm().child(name))
                        .child({
                            let badge = div().text_xs();
                            let badge = if active {
                                badge.text_color(theme::accent())
                            } else {
                                badge.text_color(theme::text_muted())
                            };
                            badge.child(if active { "Active" } else { "Attach" })
                        }),
                )
                .child(
                    div()
                        .text_color(theme::text_muted())
                        .text_xs()
                        .child(description),
                );

            let card = if active {
                card.border_color(theme::accent())
            } else {
                card.hover(|s| s.bg(theme::hover()))
            };

            list = list.child(card.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_dev_tool = if this.active_dev_tool == Some(index) {
                        None
                    } else {
                        Some(index)
                    };
                    cx.notify();
                }),
            ));
        }

        page = page.child(list);

        if let Some(index) = self.active_dev_tool
            && let Some(tool) = self.dev_tools.get(index)
        {
            page = page.child(
                div()
                    .mt_4()
                    .p_4()
                    .rounded_md()
                    .border_1()
                    .border_color(theme::border())
                    .bg(theme::elevated())
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_color(theme::text())
                            .text_sm()
                            .child(format!("{} — Output", tool.name)),
                    )
                    .child(
                        div()
                            .text_color(theme::text_muted())
                            .text_xs()
                            .child("This tool has no implementation wired up yet. Drop a renderer in crates/gesttalt/src/dev_tools.rs to plug it in."),
                    ),
            );
        }

        page
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(theme::background())
            .child(self.render_nav(cx))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(self.render_page(cx)),
            )
    }
}
