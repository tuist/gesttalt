use anyhow::{Context, Result, bail};
use icns::{IconFamily, IconType, Image as IcnsImage, PixelFormat};
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use image::{
    DynamicImage, ImageFormat, RgbaImage,
    imageops::{FilterType, crop_imm, resize},
};
use std::{env, fs, path::PathBuf};

const PNG_SIZES: &[(&str, u32)] = &[("app-icon.png", 512), ("app-icon@2x.png", 1024)];
const ICO_SIZES: &[u32] = &[16, 20, 24, 32, 40, 48, 64, 128, 256];
const ICNS_SPECS: &[(u32, IconType)] = &[
    (16, IconType::RGBA32_16x16),
    (32, IconType::RGBA32_16x16_2x),
    (32, IconType::RGBA32_32x32),
    (64, IconType::RGBA32_32x32_2x),
    (128, IconType::RGBA32_128x128),
    (256, IconType::RGBA32_128x128_2x),
    (256, IconType::RGBA32_256x256),
    (512, IconType::RGBA32_256x256_2x),
    (512, IconType::RGBA32_512x512),
    (1024, IconType::RGBA32_512x512_2x),
];

fn main() -> Result<()> {
    match env::args().nth(1).as_deref() {
        None | Some("generate-app-icons") => generate_app_icons(),
        Some(command) => bail!("unknown xtask command: {command}"),
    }
}

fn generate_app_icons() -> Result<()> {
    let resources_dir = repo_root().join("crates/gesttalt/resources");
    let windows_resources_dir = resources_dir.join("windows");
    let source_path = resources_dir.join("app-icon-source.png");

    let image = image::open(&source_path)
        .with_context(|| format!("opening source icon {}", source_path.display()))?
        .into_rgba8();
    let image = crop_to_square(trim_transparent_padding(image));

    fs::create_dir_all(&resources_dir)
        .with_context(|| format!("creating {}", resources_dir.display()))?;
    fs::create_dir_all(&windows_resources_dir)
        .with_context(|| format!("creating {}", windows_resources_dir.display()))?;

    for &(filename, size) in PNG_SIZES {
        let output_path = resources_dir.join(filename);
        write_png(&output_path, &resize_icon(&image, size))?;
    }

    write_ico(&windows_resources_dir.join("app-icon.ico"), &image)?;
    write_icns(&resources_dir.join("app-icon.icns"), &image)?;

    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask crate should live under crates/")
        .to_path_buf()
}

fn trim_transparent_padding(image: RgbaImage) -> RgbaImage {
    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found_opaque_pixel = false;

    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] == 0 {
            continue;
        }

        found_opaque_pixel = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if !found_opaque_pixel {
        return image;
    }

    crop_imm(&image, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1).to_image()
}

fn crop_to_square(image: RgbaImage) -> RgbaImage {
    if image.width() == image.height() {
        return image;
    }

    let size = image.width().min(image.height());
    let left = (image.width() - size) / 2;
    let top = (image.height() - size) / 2;

    crop_imm(&image, left, top, size, size).to_image()
}

fn resize_icon(image: &RgbaImage, size: u32) -> RgbaImage {
    resize(image, size, size, FilterType::Lanczos3)
}

fn write_png(path: &PathBuf, image: &RgbaImage) -> Result<()> {
    DynamicImage::ImageRgba8(image.clone())
        .save_with_format(path, ImageFormat::Png)
        .with_context(|| format!("writing PNG {}", path.display()))
}

fn write_ico(path: &PathBuf, image: &RgbaImage) -> Result<()> {
    let mut icon_dir = IconDir::new(ResourceType::Icon);
    for &size in ICO_SIZES {
        let resized = resize_icon(image, size);
        let icon_image = IconImage::from_rgba_data(size, size, resized.into_raw());
        icon_dir.add_entry(
            IconDirEntry::encode(&icon_image)
                .with_context(|| format!("encoding ICO entry {size}x{size}"))?,
        );
    }

    let file =
        fs::File::create(path).with_context(|| format!("creating ICO {}", path.display()))?;
    icon_dir
        .write(file)
        .with_context(|| format!("writing ICO {}", path.display()))
}

fn write_icns(path: &PathBuf, image: &RgbaImage) -> Result<()> {
    let mut icon_family = IconFamily::new();
    for &(size, icon_type) in ICNS_SPECS {
        let resized = resize_icon(image, size);
        let icon_image = IcnsImage::from_data(PixelFormat::RGBA, size, size, resized.into_raw())
            .with_context(|| format!("encoding ICNS pixels for {icon_type:?}"))?;
        icon_family
            .add_icon_with_type(&icon_image, icon_type)
            .with_context(|| format!("adding ICNS icon {icon_type:?}"))?;
    }

    let file =
        fs::File::create(path).with_context(|| format!("creating ICNS {}", path.display()))?;
    icon_family
        .write(file)
        .with_context(|| format!("writing ICNS {}", path.display()))
}
