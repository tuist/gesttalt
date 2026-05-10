use gpui::SharedString;

#[derive(Clone)]
pub struct DevTool {
    pub name: SharedString,
    pub description: SharedString,
}

pub fn default_dev_tools() -> Vec<DevTool> {
    vec![
        DevTool {
            name: "Workspace Bounds".into(),
            description: "Inspect the live bounds of the workspace and dock sizes.".into(),
        },
        DevTool {
            name: "Theme Inspector".into(),
            description: "Browse the active theme's color tokens.".into(),
        },
        DevTool {
            name: "Agent Probe".into(),
            description: "Send a tiny prompt to each detected coding-agent CLI.".into(),
        },
        DevTool {
            name: "Keymap Inspector".into(),
            description: "View the registered actions and their key bindings.".into(),
        },
    ]
}
