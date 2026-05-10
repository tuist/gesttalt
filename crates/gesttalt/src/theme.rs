use gpui::{Hsla, Rgba, hsla, rgb};

pub fn background() -> Rgba {
    rgb(0x1e1e2e)
}

pub fn panel() -> Rgba {
    rgb(0x181825)
}

pub fn elevated() -> Rgba {
    rgb(0x11111b)
}

pub fn border() -> Rgba {
    rgb(0x313244)
}

pub fn text() -> Rgba {
    rgb(0xcdd6f4)
}

pub fn text_muted() -> Rgba {
    rgb(0x9399b2)
}

pub fn accent() -> Rgba {
    rgb(0x89b4fa)
}

pub fn selection() -> Rgba {
    rgb(0x45475a)
}

pub fn hover() -> Rgba {
    rgb(0x313244)
}

pub fn modal_backdrop() -> Hsla {
    hsla(0.0, 0.0, 0.0, 0.45)
}
