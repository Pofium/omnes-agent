"""Generate OmnesAgent PNG and ICO brand assets from the canonical OA_icon.svg.

Tries CairoSVG (`cairosvg`) or `resvg_py` when installed, and otherwise falls back
to a pure-Python cubic Bezier path flattener rendered at 4x supersampling via Pillow
directly from ``OA_icon.svg`` geometry.
"""

from __future__ import annotations

import re
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Iterable

REPO_ROOT = Path(__file__).resolve().parent.parent
SVG_SOURCE = REPO_ROOT / "frontend" / "desktop" / "assets" / "Logo" / "OA_icon.svg"

PNG_TARGETS: tuple[tuple[Path, int], ...] = (
    (REPO_ROOT / "frontend" / "desktop" / "assets" / "Logo" / "app_icon.png", 512),
    (REPO_ROOT / "frontend" / "desktop" / "assets" / "Logo" / "app_launcher.png", 256),
    (REPO_ROOT / "frontend" / "web" / "assets" / "Logo" / "app_icon.png", 512),
    (REPO_ROOT / "frontend" / "web" / "assets" / "Logo" / "app_launcher.png", 256),
    (REPO_ROOT / "frontend" / "web" / "web" / "favicon.png", 64),
    (REPO_ROOT / "frontend" / "web" / "web" / "icons" / "Icon-192.png", 192),
    (REPO_ROOT / "frontend" / "web" / "web" / "icons" / "Icon-512.png", 512),
    (REPO_ROOT / "frontend" / "web" / "web" / "icons" / "Icon-maskable-192.png", 192),
    (REPO_ROOT / "frontend" / "web" / "web" / "icons" / "Icon-maskable-512.png", 512),
)

ICO_TARGET = REPO_ROOT / "frontend" / "desktop" / "assets" / "app_icon.ico"
ICO_SIZES: list[tuple[int, int]] = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]


def _parse_hex_rgba(hex_color: str) -> tuple[int, int, int, int]:
    """Convert a #RRGGBB color string to an RGBA tuple."""
    clean = hex_color.strip().lstrip("#")
    if len(clean) != 6:
        raise ValueError(f"Unsupported color format: {hex_color}")
    return (int(clean[0:2], 16), int(clean[2:4], 16), int(clean[4:6], 16), 255)


def _flatten_cubic(
    p0: tuple[float, float],
    p1: tuple[float, float],
    p2: tuple[float, float],
    p3: tuple[float, float],
    steps: int = 8,
) -> list[tuple[float, float]]:
    """Sample a cubic Bezier segment into linear points."""
    pts: list[tuple[float, float]] = []
    for i in range(1, steps + 1):
        t = i / steps
        mt = 1.0 - t
        x = (mt**3) * p0[0] + 3 * (mt**2) * t * p1[0] + 3 * mt * (t**2) * p2[0] + (t**3) * p3[0]
        y = (mt**3) * p0[1] + 3 * (mt**2) * t * p1[1] + 3 * mt * (t**2) * p2[1] + (t**3) * p3[1]
        pts.append((x, y))
    return pts


def _parse_path_polygon(d: str, tx: float, ty: float) -> list[tuple[float, float]]:
    """Parse an SVG path containing M, C, L, Z commands into absolute polygon points."""
    tokens = re.findall(r"[A-Za-z]|-?\d+(?:\.\d+)?", d)
    pts: list[tuple[float, float]] = []
    idx = 0
    cur = (0.0, 0.0)
    cmd = ""
    while idx < len(tokens):
        tok = tokens[idx]
        if tok.isalpha():
            cmd = tok
            idx += 1
            if cmd in ("Z", "z"):
                continue
        if cmd == "M":
            x = float(tokens[idx]) + tx
            y = float(tokens[idx + 1]) + ty
            idx += 2
            cur = (x, y)
            pts.append(cur)
            cmd = "L"
        elif cmd == "L":
            x = float(tokens[idx]) + tx
            y = float(tokens[idx + 1]) + ty
            idx += 2
            cur = (x, y)
            pts.append(cur)
        elif cmd == "C":
            p1 = (float(tokens[idx]) + tx, float(tokens[idx + 1]) + ty)
            p2 = (float(tokens[idx + 2]) + tx, float(tokens[idx + 3]) + ty)
            p3 = (float(tokens[idx + 4]) + tx, float(tokens[idx + 5]) + ty)
            idx += 6
            pts.extend(_flatten_cubic(cur, p1, p2, p3, steps=6))
            cur = p3
        else:
            raise ValueError(f"Unsupported SVG path command: {cmd!r}")
    return pts


def _render_via_svg_backend(svg_bytes: bytes, size: int) -> bytes | None:
    """Try rendering SVG bytes directly via cairosvg or resvg_py."""
    try:
        import cairosvg  # type: ignore

        return cairosvg.svg2png(bytestring=svg_bytes, output_width=size, output_height=size)
    except Exception:
        pass

    try:
        import resvg_py  # type: ignore

        return resvg_py.svg_to_bytes(svg_bytes.decode("utf-8"), width=size, height=size)
    except Exception:
        return None


def _render_via_pillow(svg_text: str, size: int):
    """Render OA_icon.svg accurately using 4x supersampling in Pillow."""
    from PIL import Image, ImageDraw  # type: ignore

    root = ET.fromstring(svg_text)
    view_box = [float(v) for v in root.attrib.get("viewBox", "0 0 818 815").split()]
    vb_w, vb_h = view_box[2], view_box[3]

    ss = max(size * 4, 1024)
    scale = min(ss / vb_w, ss / vb_h)
    ox = (ss - vb_w * scale) / 2.0
    oy = (ss - vb_h * scale) / 2.0

    canvas = Image.new("RGBA", (ss, ss), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)

    for elem in root:
        tag = elem.tag.split("}")[-1]
        fill = _parse_hex_rgba(elem.attrib.get("fill", "#FFFFFF"))
        if tag == "circle":
            cx = float(elem.attrib["cx"]) * scale + ox
            cy = float(elem.attrib["cy"]) * scale + oy
            r = float(elem.attrib["r"]) * scale
            draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=fill)
        elif tag == "path":
            tx, ty = 0.0, 0.0
            transform = elem.attrib.get("transform", "")
            match = re.search(r"translate\(\s*([^,\s)]+)[\s,]+([^)]+)\)", transform)
            if match:
                tx = float(match.group(1))
                ty = float(match.group(2))
            raw_pts = _parse_path_polygon(elem.attrib["d"], tx, ty)
            scaled = [(x * scale + ox, y * scale + oy) for x, y in raw_pts]
            if len(scaled) >= 3:
                draw.polygon(scaled, fill=fill)

    return canvas.resize((size, size), Image.Resampling.LANCZOS)


def render_png(svg_bytes: bytes, size: int, output_path: Path) -> None:
    """Render an SVG source into a square PNG at ``output_path``."""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    rendered = _render_via_svg_backend(svg_bytes, size)
    if rendered is not None:
        output_path.write_bytes(rendered)
        return

    img = _render_via_pillow(svg_bytes.decode("utf-8"), size)
    img.save(output_path, format="PNG")


def render_ico(svg_bytes: bytes, ico_path: Path, sizes: Iterable[tuple[int, int]]) -> None:
    """Render a multi-resolution Windows .ico file from the canonical SVG."""
    from io import BytesIO
    from PIL import Image  # type: ignore

    ico_path.parent.mkdir(parents=True, exist_ok=True)
    largest = max(max(w, h) for w, h in sizes)
    rendered = _render_via_svg_backend(svg_bytes, largest)
    if rendered is not None:
        base_img = Image.open(BytesIO(rendered)).convert("RGBA")
    else:
        base_img = _render_via_pillow(svg_bytes.decode("utf-8"), largest)
    base_img.save(ico_path, format="ICO", sizes=list(sizes))


def main() -> None:
    """Generate all PNG and ICO brand icon assets for Desktop and Web."""
    if not SVG_SOURCE.exists():
        raise FileNotFoundError(f"Canonical SVG source not found: {SVG_SOURCE}")

    svg_bytes = SVG_SOURCE.read_bytes()
    for target_path, size in PNG_TARGETS:
        render_png(svg_bytes, size, target_path)
        print(f"Generated PNG ({size}x{size}): {target_path.relative_to(REPO_ROOT)}")

    render_ico(svg_bytes, ICO_TARGET, ICO_SIZES)
    print(f"Generated ICO: {ICO_TARGET.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
