#!/usr/bin/env python3

from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
RESOURCES = ROOT / "crates" / "gesttalt" / "resources"
WINDOWS_RESOURCES = RESOURCES / "windows"
SOURCE = RESOURCES / "app-icon-source.png"

PNG_SIZES = {
    "app-icon.png": 512,
    "app-icon@2x.png": 1024,
}

ICO_SIZES = [(16, 16), (20, 20), (24, 24), (32, 32), (40, 40), (48, 48), (64, 64), (128, 128), (256, 256)]
ICNS_ICONSET_SPECS = {
    "icon_16x16.png": 16,
    "icon_16x16@2x.png": 32,
    "icon_32x32.png": 32,
    "icon_32x32@2x.png": 64,
    "icon_128x128.png": 128,
    "icon_128x128@2x.png": 256,
    "icon_256x256.png": 256,
    "icon_256x256@2x.png": 512,
    "icon_512x512.png": 512,
    "icon_512x512@2x.png": 1024,
}


def crop_to_square(image: Image.Image) -> Image.Image:
    if image.width == image.height:
        return image

    size = min(image.width, image.height)
    left = (image.width - size) // 2
    top = (image.height - size) // 2
    return image.crop((left, top, left + size, top + size))


def resized(image: Image.Image, size: int) -> Image.Image:
    return image.resize((size, size), Image.Resampling.LANCZOS)


def build_icns(image: Image.Image, destination: Path) -> None:
    iconutil = shutil.which("iconutil")
    if iconutil:
        with tempfile.TemporaryDirectory() as temp_dir:
            iconset = Path(temp_dir) / "app-icon.iconset"
            iconset.mkdir()
            for filename, size in ICNS_ICONSET_SPECS.items():
                resized(image, size).save(iconset / filename)

            subprocess.run(
                [iconutil, "-c", "icns", str(iconset), "-o", str(destination)],
                check=True,
            )
        return

    resized(image, 1024).save(destination)


def main() -> None:
    if not SOURCE.exists():
        raise SystemExit(f"missing source icon: {SOURCE}")

    RESOURCES.mkdir(parents=True, exist_ok=True)
    WINDOWS_RESOURCES.mkdir(parents=True, exist_ok=True)

    image = crop_to_square(Image.open(SOURCE).convert("RGBA"))

    for filename, size in PNG_SIZES.items():
        resized(image, size).save(RESOURCES / filename)

    image.save(WINDOWS_RESOURCES / "app-icon.ico", sizes=ICO_SIZES)
    build_icns(image, RESOURCES / "app-icon.icns")


if __name__ == "__main__":
    main()
