use gpui::{
    App, Context, FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseButton, ParentElement,
    Render, SharedString, Styled, WeakEntity, Window, actions, div, prelude::*, px, rems,
};

use crate::theme;
use crate::workspace::Workspace;

actions!(
    command_palette,
    [
        /// Toggles the command palette modal.
        Toggle,
        /// Selects the next command in the list.
        SelectNext,
        /// Selects the previous command in the list.
        SelectPrev,
        /// Invokes the highlighted command.
        Confirm,
        /// Closes the command palette without invoking anything.
        Dismiss,
    ]
);

pub type CommandHandler = Box<dyn Fn(&mut Workspace, &mut Window, &mut Context<Workspace>)>;

pub struct Command {
    pub name: SharedString,
    pub keybinding: Option<SharedString>,
    pub action: CommandHandler,
}

pub struct CommandPalette {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    query: String,
    selected: usize,
    commands: Vec<Command>,
    visible_indices: Vec<usize>,
}

impl CommandPalette {
    pub fn new(
        workspace: WeakEntity<Workspace>,
        commands: Vec<Command>,
        cx: &mut Context<Self>,
    ) -> Self {
        let visible_indices = (0..commands.len()).collect();
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            query: String::new(),
            selected: 0,
            commands,
            visible_indices,
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    fn refresh_matches(&mut self) {
        let query = self.query.trim().to_lowercase();
        self.visible_indices = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| {
                if query.is_empty() {
                    true
                } else {
                    cmd.name.to_lowercase().contains(&query)
                }
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = 0;
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        if self.visible_indices.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.visible_indices.len();
        cx.notify();
    }

    fn select_prev(&mut self, _: &SelectPrev, _: &mut Window, cx: &mut Context<Self>) {
        if self.visible_indices.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.visible_indices.len() - 1
        } else {
            self.selected - 1
        };
        cx.notify();
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let Some(&command_index) = self.visible_indices.get(self.selected) else {
            return;
        };
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let Some(command) = self.commands.get(command_index) else {
            return;
        };
        let action = command.action.as_ref();
        workspace.update(cx, |workspace, cx| {
            workspace.dismiss_command_palette(cx);
            (action)(workspace, window, cx);
        });
    }

    fn dismiss(&mut self, _: &Dismiss, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(workspace) = self.workspace.upgrade() {
            workspace.update(cx, |workspace, cx| workspace.dismiss_command_palette(cx));
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform {
            return;
        }
        match event.keystroke.key.as_str() {
            "backspace" => {
                if self.query.pop().is_some() {
                    self.refresh_matches();
                    cx.notify();
                }
                return;
            }
            "up" | "down" | "enter" | "escape" | "tab" | "left" | "right" | "home" | "end" => {
                return;
            }
            _ => {}
        }
        let Some(typed) = event.keystroke.key_char.as_ref() else {
            return;
        };
        let mut changed = false;
        for ch in typed.chars() {
            if !ch.is_control() {
                self.query.push(ch);
                changed = true;
            }
        }
        if changed {
            self.refresh_matches();
            cx.notify();
        }
    }
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let placeholder = self.query.is_empty();
        let query_text: SharedString = if placeholder {
            "Type a command…".into()
        } else {
            self.query.clone().into()
        };
        let query_color = if placeholder {
            theme::text_muted()
        } else {
            theme::text()
        };

        let visible: Vec<(SharedString, Option<SharedString>, bool)> = self
            .visible_indices
            .iter()
            .enumerate()
            .map(|(row_idx, &cmd_idx)| {
                let cmd = &self.commands[cmd_idx];
                (
                    cmd.name.clone(),
                    cmd.keybinding.clone(),
                    row_idx == self.selected,
                )
            })
            .collect();

        let empty_message = visible.is_empty().then(|| {
            div()
                .px_3()
                .py_4()
                .text_color(theme::text_muted())
                .text_sm()
                .child("No matching commands")
        });

        div()
            .key_context("CommandPalette")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .w(rems(34.))
            .max_h(rems(28.))
            .bg(theme::elevated())
            .border_1()
            .border_color(theme::border())
            .rounded_lg()
            .shadow_lg()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .px_3()
                    .py_2p5()
                    .border_b_1()
                    .border_color(theme::border())
                    .text_color(query_color)
                    .text_sm()
                    .child(query_text),
            )
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .py_1()
                    .when_some(empty_message, |this, msg| this.child(msg))
                    .children(visible.into_iter().enumerate().map(
                        |(row_idx, (name, keybinding, selected))| {
                            let row = div()
                                .id(("command-palette-row", row_idx))
                                .px_3()
                                .py_1p5()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .text_sm()
                                .cursor_pointer()
                                .child(div().child(name));
                            let row = if selected {
                                row.bg(theme::selection()).text_color(theme::text())
                            } else {
                                row.text_color(theme::text_muted())
                                    .hover(|s| s.bg(theme::hover()).text_color(theme::text()))
                            };
                            let row = row.when_some(keybinding, |this, kb| {
                                this.child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(theme::panel())
                                        .text_color(theme::text_muted())
                                        .text_xs()
                                        .child(kb),
                                )
                            });
                            row.on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, cx| {
                                    this.selected = row_idx;
                                    this.confirm(&Confirm, window, cx);
                                }),
                            )
                        },
                    )),
            )
    }
}
