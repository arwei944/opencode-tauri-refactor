"""为 OpenCode Tauri 生成有效的图标文件。

生成一个简洁的蓝色方形图标，包含 "OC" 字样。
输出:
  - src-tauri/icons/icon.png   (256x256 RGBA)
  - src-tauri/icons/icon.ico   (256x256, 多分辨率)
"""

from PIL import Image, ImageDraw, ImageFont
import os
import struct

# 颜色方案：OpenCode 品牌色（深蓝紫渐变）
COLOR_BG_TOP = (52, 73, 187, 255)     # 蓝色
COLOR_BG_BOTTOM = (74, 58, 217, 255)  # 紫色
COLOR_FG = (255, 255, 255, 255)       # 白色
COLOR_BORDER = (30, 41, 95, 255)      # 深色边框

OUT_DIR = r"C:\Users\Administrator\opencode-tauri-refactor\src-tauri\icons"
SIZE = 256


def make_square_icon(size: int) -> Image.Image:
    """生成圆角方形图标的 RGBA 图像。"""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 圆角矩形半径
    radius = int(size * 0.2)

    # 渐变背景 (从顶到底)
    for y in range(size):
        t = y / max(size - 1, 1)
        r = int(COLOR_BG_TOP[0] * (1 - t) + COLOR_BG_BOTTOM[0] * t)
        g = int(COLOR_BG_TOP[1] * (1 - t) + COLOR_BG_BOTTOM[1] * t)
        b = int(COLOR_BG_TOP[2] * (1 - t) + COLOR_BG_BOTTOM[2] * t)
        draw.line([(0, y), (size, y)], fill=(r, g, b, 255))

    # 应用圆角蒙版
    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle([(0, 0), (size - 1, size - 1)], radius=radius, fill=255)
    rounded = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    rounded.paste(img, (0, 0), mask)
    img = rounded

    # 边框
    border_draw = ImageDraw.Draw(img)
    border_draw.rounded_rectangle(
        [(1, 1), (size - 2, size - 2)],
        radius=radius,
        outline=COLOR_BORDER,
        width=2,
    )

    # 中心文字 "OC"（OpenCode）
    try:
        # 尝试加载系统中文字体
        font_paths = [
            r"C:\Windows\Fonts\segoeuib.ttf",
            r"C:\Windows\Fonts\seguisb.ttf",
            r"C:\Windows\Fonts\arialbd.ttf",
            r"C:\Windows\Fonts\arial.ttf",
        ]
        font = None
        for fp in font_paths:
            if os.path.exists(fp):
                font = ImageFont.truetype(fp, int(size * 0.45))
                break
        if font is None:
            font = ImageFont.load_default()
    except Exception:
        font = ImageFont.load_default()

    text = "OC"
    bbox = border_draw.textbbox((0, 0), text, font=font)
    text_w = bbox[2] - bbox[0]
    text_h = bbox[3] - bbox[1]
    x = (size - text_w) // 2 - bbox[0]
    y = (size - text_h) // 2 - bbox[1]
    border_draw.text((x, y), text, fill=COLOR_FG, font=font)

    return img


def write_png(img: Image.Image, path: str) -> None:
    img.save(path, format="PNG", optimize=True)
    print(f"  写入 {path} ({os.path.getsize(path)} 字节)")


def write_ico(img: Image.Image, path: str) -> None:
    """直接用 Pillow 保存为 ICO（包含多分辨率）。"""
    sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    img.save(path, format="ICO", sizes=sizes)
    print(f"  写入 {path} ({os.path.getsize(path)} 字节)")


def main() -> None:
    os.makedirs(OUT_DIR, exist_ok=True)
    print("生成 OpenCode Tauri 图标:")

    base = make_square_icon(SIZE)
    write_png(base, os.path.join(OUT_DIR, "icon.png"))
    write_ico(base, os.path.join(OUT_DIR, "icon.ico"))

    # 顺便生成 Tauri 2 推荐的多分辨率 PNG 资源
    # （虽然 tauri.conf.json 现在只引用 icon.ico / icon.png，
    #  但 macOS / Linux 平台标准要求这些名称存在）
    tauri_sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
    }
    for name, sz in tauri_sizes.items():
        resized = base.resize((sz, sz), Image.LANCZOS)
        write_png(resized, os.path.join(OUT_DIR, name))

    # macOS icns 占位（最小有效 ICNS）
    write_minimal_icns(base, os.path.join(OUT_DIR, "icon.icns"))

    print("完成。")


def write_minimal_icns(base_img: Image.Image, path: str) -> None:
    """写一个最小的 macOS ICNS 文件，包含 256x256 PNG。"""
    # 256x256 PNG 图标类型代码: 'ic08'
    png_bytes_256 = base_img.resize((256, 256), Image.LANCZOS)
    import io

    buf = io.BytesIO()
    png_bytes_256.save(buf, format="PNG")
    png_data = buf.getvalue()

    # ICNS 头: magic 'icns' + 总长度 (big-endian)
    inner = b"ic08" + struct.pack(">I", 8 + len(png_data)) + png_data
    total_len = 8 + len(inner)
    with open(path, "wb") as f:
        f.write(b"icns" + struct.pack(">I", total_len) + inner)
    print(f"  写入 {path} ({os.path.getsize(path)} 字节)")


if __name__ == "__main__":
    main()
