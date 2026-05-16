#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn prepare_linux_app_icon() {
    use image::{ImageReader, imageops::FilterType};
    use std::path::PathBuf;

    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/app-icon.png");
    let output = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("app_icon.png");

    let image = ImageReader::open(&source)
        .unwrap()
        .decode()
        .unwrap()
        .resize(256, 256, FilterType::Lanczos3);

    image.save(&output).expect("saving Linux app icon");

    println!("cargo:rerun-if-changed={}", source.display());
}

#[cfg(target_os = "windows")]
fn compile_windows_icon() {
    use std::path::PathBuf;

    let icon = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/windows/app-icon.ico");
    let icon_escaped = icon.to_string_lossy().replace('\\', "\\\\");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let rc_path = out_dir.join("gesttalt_resources.rc");

    std::fs::write(&rc_path, format!("1 ICON \"{icon_escaped}\"\n"))
        .expect("writing Windows resource manifest");

    println!("cargo:rerun-if-changed={}", icon.display());

    embed_resource::compile(&rc_path, embed_resource::NONE)
        .manifest_optional()
        .expect("compiling Windows icon resources");
}

fn main() {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    prepare_linux_app_icon();

    #[cfg(target_os = "windows")]
    compile_windows_icon();
}
