#!/usr/bin/env python3
"""
Tauri Icons Replacer
将源图片按尺寸替换 Tauri icons 目录下的所有图标文件（PNG/ICO/ICNS）。用户迭代品牌图标

用法:
  python replace-icons.py <source_image> <icons_dir> [--exclude tray_,backup_]

示例:
  python replace-icons.py logo.png ./src-tauri/icons
  
  # 自定义排除前缀
  python replace-icons.py logo.png ./src-tauri/icons --exclude tray_,debug_
"""

import argparse
import os
import sys

from PIL import Image

ICO_SIZES = [16, 32, 48, 64, 128, 256]
ICNS_SIZE = 1024
SUPPORTED_EXTS = (".png", ".ico", ".icns")


def resize_fit(img, tw, th):
    img_ratio = img.width / img.height
    target_ratio = tw / th
    if img_ratio > target_ratio:
        new_w = tw
        new_h = int(tw / img_ratio)
    else:
        new_h = th
        new_w = int(th * img_ratio)
    resized = img.resize((new_w, new_h), Image.LANCZOS)
    canvas = Image.new("RGBA", (tw, th), (0, 0, 0, 0))
    offset_x = (tw - new_w) // 2
    offset_y = (th - new_h) // 2
    canvas.paste(resized, (offset_x, offset_y), resized)
    return canvas


def is_excluded(filename, prefixes):
    return any(filename.startswith(p) for p in prefixes)


def replace_icons(src_path, icons_dir, exclude_prefixes):
    if not os.path.isfile(src_path):
        print(f"Error: source image not found: {src_path}")
        sys.exit(1)
    if not os.path.isdir(icons_dir):
        print(f"Error: icons directory not found: {icons_dir}")
        sys.exit(1)

    src_img = Image.open(src_path).convert("RGBA")
    print(f"Source image: {src_img.width}x{src_img.height} {src_img.mode}")

    replaced = 0
    skipped = 0

    for root, _dirs, files in os.walk(icons_dir):
        for f in sorted(files):
            if is_excluded(f, exclude_prefixes):
                skipped += 1
                continue
            if not f.lower().endswith(SUPPORTED_EXTS):
                continue

            path = os.path.join(root, f)
            rel = os.path.relpath(path, icons_dir)

            try:
                if f.lower().endswith(".png"):
                    target = Image.open(path)
                    tw, th = target.size
                    target.close()
                    result = resize_fit(src_img, tw, th)
                    result.save(path, "PNG")
                    print(f"  OK  {rel} ({tw}x{th})")

                elif f.lower().endswith(".ico"):
                    images = [resize_fit(src_img, s, s) for s in ICO_SIZES]
                    images[0].save(
                        path,
                        format="ICO",
                        sizes=[(s, s) for s in ICO_SIZES],
                        append_images=images[1:],
                    )
                    print(f"  OK  {rel} (ICO {ICO_SIZES})")

                elif f.lower().endswith(".icns"):
                    result = resize_fit(src_img, ICNS_SIZE, ICNS_SIZE)
                    result.save(path, format="ICNS")
                    print(f"  OK  {rel} (ICNS {ICNS_SIZE}x{ICNS_SIZE})")

                replaced += 1
            except Exception as e:
                print(f"  SKIP {rel} - {e}")
                skipped += 1

    print(f"\nDone: {replaced} replaced, {skipped} skipped")


def main():
    parser = argparse.ArgumentParser(
        description="Replace Tauri icons with a new source image, preserving aspect ratio and color."
    )
    parser.add_argument("source", help="Path to the source image (PNG)")
    parser.add_argument("icons_dir", help="Path to the Tauri icons directory")
    parser.add_argument(
        "--exclude",
        default="tray_",
        help="Comma-separated filename prefixes to skip (default: tray_)",
    )
    args = parser.parse_args()

    prefixes = [p.strip() for p in args.exclude.split(",") if p.strip()]
    replace_icons(args.source, args.icons_dir, prefixes)


if __name__ == "__main__":
    main()
