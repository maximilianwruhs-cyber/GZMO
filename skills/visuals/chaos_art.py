#!/usr/bin/env python3
"""
GZMO Visual Engine — Procedural Chaos Art Generator

Generates images driven by chaos engine state (Lorenz coordinates,
tension, energy, valence). Supports multiple visual modes:

  - lorenz: Plot the Lorenz attractor orbit
  - energy: Abstract energy field visualization
  - card: MTG-style card frame with art
  - dice: D20/D6 visual result
  - mood: Valence-driven abstract mood painting
  - sigil: Sacred geometry / chaos sigil

Usage:
  python3 visuals/chaos_art.py <mode> <output_path> [--state JSON_PATH]

Reads CHAOS_STATE.json for engine parameters. Falls back to defaults.
"""

import sys
import json
import math
import random
import os

from PIL import Image, ImageDraw, ImageFilter, ImageFont

# ─── Read chaos state ────────────────────────────────────────────

def load_chaos_state(path=None):
    """Load chaos state from JSON. Falls back to seeded defaults."""
    defaults = {
        "x": 1.0, "y": 2.0, "z": 25.0,
        "tension": 50.0, "energy": 75.0,
        "chaos_val": 0.5, "llm_valence": 0.0,
        "tick": 0,
    }
    if path and os.path.exists(path):
        try:
            with open(path) as f:
                data = json.load(f)
                defaults.update(data)
        except Exception:
            pass
    return defaults


# ─── Color palettes (valence-driven) ─────────────────────────────

def palette_from_valence(valence: float):
    """
    valence < -0.5: dark, aggressive (reds, blacks, deep purples)
    valence ~ 0:    neutral, cool (blues, grays, teals)
    valence > 0.5:  serene, warm (golds, ambers, soft greens)
    """
    if valence < -0.5:
        return [
            (180, 20, 20), (120, 0, 40), (60, 0, 60),
            (200, 50, 30), (40, 0, 20), (255, 80, 0),
            (100, 10, 10), (20, 0, 0),
        ]
    elif valence < 0.0:
        return [
            (40, 80, 140), (20, 50, 100), (70, 90, 120),
            (30, 60, 90), (80, 100, 140), (50, 70, 110),
            (10, 30, 60), (60, 80, 100),
        ]
    elif valence < 0.5:
        return [
            (100, 140, 120), (80, 120, 100), (60, 100, 80),
            (120, 160, 130), (90, 130, 110), (70, 110, 90),
            (50, 90, 70), (110, 150, 120),
        ]
    else:
        return [
            (212, 175, 55), (180, 140, 40), (220, 190, 80),
            (200, 160, 50), (230, 200, 100), (190, 150, 45),
            (160, 130, 35), (240, 210, 120),
        ]


# ─── Lorenz Orbit Visualization ──────────────────────────────────

def render_lorenz(state, w=400, h=400):
    """Plot a section of the Lorenz attractor orbit."""
    img = Image.new('RGB', (w, h), (5, 5, 15))
    draw = ImageDraw.Draw(img)
    palette = palette_from_valence(state.get("llm_valence", 0))

    # Simulate orbit from current state
    x, y, z = state["x"], state["y"], state["z"]
    dt = 0.005
    sigma, rho, beta = 10.0, 28.0, 8.0 / 3.0

    points = []
    for i in range(4000):
        dx = sigma * (y - x) * dt
        dy = (x * (rho - z) - y) * dt
        dz = (x * y - beta * z) * dt
        x += dx; y += dy; z += dz
        # Project 3D → 2D (XZ plane with slight rotation)
        px = int(w/2 + (x * 6) + z * 0.3)
        py = int(h/2 + (z * 4) - 120)
        px = max(0, min(w-1, px))
        py = max(0, min(h-1, py))
        points.append((px, py, i))

    # Draw orbit with fading trail
    for px, py, i in points:
        alpha = min(255, int(50 + (i / 4000) * 200))
        color_idx = (i // 500) % len(palette)
        c = palette[color_idx]
        intensity = alpha / 255.0
        rc = (int(c[0] * intensity), int(c[1] * intensity), int(c[2] * intensity))
        draw.point((px, py), fill=rc)
        # Glow effect
        for dx, dy in [(-1,0),(1,0),(0,-1),(0,1)]:
            npx, npy = px+dx, py+dy
            if 0 <= npx < w and 0 <= npy < h:
                glow = (int(c[0]*intensity*0.3), int(c[1]*intensity*0.3), int(c[2]*intensity*0.3))
                draw.point((npx, npy), fill=glow)

    # Overlay state text
    draw.text((10, h-30), f"Lorenz ({state['x']:.1f}, {state['y']:.1f}, {state['z']:.1f})", fill=(100,100,100))
    draw.text((10, h-15), f"τ:{state['tension']:.0f}% ε:{state['energy']:.0f}%", fill=(80,80,80))

    img = img.filter(ImageFilter.GaussianBlur(radius=0.5))
    return img


# ─── Energy Field Visualization ──────────────────────────────────

def render_energy(state, w=400, h=300):
    """Abstract energy field — concentric waves modulated by chaos."""
    img = Image.new('RGB', (w, h), (0, 0, 0))
    draw = ImageDraw.Draw(img)
    palette = palette_from_valence(state.get("llm_valence", 0))

    cx, cy = w // 2, h // 2
    energy = state.get("energy", 50) / 100.0
    tension = state.get("tension", 50) / 100.0
    chaos = state.get("chaos_val", 0.5)

    for y_pos in range(h):
        for x_pos in range(0, w, 2):  # Step 2 for performance
            dx = x_pos - cx
            dy = y_pos - cy
            dist = math.sqrt(dx*dx + dy*dy)
            angle = math.atan2(dy, dx)

            # Interference pattern
            wave1 = math.sin(dist * 0.05 + state["x"] * 0.3) * energy
            wave2 = math.cos(angle * 3 + state["y"] * 0.2) * tension
            wave3 = math.sin(dist * 0.02 - state["z"] * 0.1) * chaos

            intensity = (wave1 + wave2 + wave3 + 1.5) / 3.0
            intensity = max(0, min(1, intensity))

            ci = int(intensity * (len(palette) - 1))
            c = palette[ci]
            brightness = intensity * 0.8 + 0.2
            rc = (int(c[0]*brightness), int(c[1]*brightness), int(c[2]*brightness))
            draw.point((x_pos, y_pos), fill=rc)
            draw.point((x_pos+1, y_pos), fill=rc)

    img = img.filter(ImageFilter.GaussianBlur(radius=1))
    return img


# ─── Mood Painting ───────────────────────────────────────────────

def render_mood(state, w=400, h=300):
    """Abstract mood visualization — valence/tension driven."""
    img = Image.new('RGB', (w, h), (0, 0, 0))
    draw = ImageDraw.Draw(img)
    palette = palette_from_valence(state.get("llm_valence", 0))

    chaos = state.get("chaos_val", 0.5)
    tension = state.get("tension", 50) / 100.0
    energy = state.get("energy", 50) / 100.0

    # Procedural brush strokes
    rng = random.Random(int(chaos * 100000) + state.get("tick", 0))
    num_strokes = int(40 + tension * 60)

    for _ in range(num_strokes):
        x1 = rng.randint(0, w)
        y1 = rng.randint(0, h)
        length = rng.randint(20, int(80 + energy * 100))
        angle = rng.random() * math.pi * 2
        x2 = int(x1 + math.cos(angle) * length)
        y2 = int(y1 + math.sin(angle) * length)
        width = rng.randint(2, int(5 + tension * 10))
        color = palette[rng.randint(0, len(palette)-1)]
        opacity = rng.random() * 0.6 + 0.2
        rc = (int(color[0]*opacity), int(color[1]*opacity), int(color[2]*opacity))
        draw.line([(x1,y1),(x2,y2)], fill=rc, width=width)

    # Splatter dots
    for _ in range(int(tension * 200)):
        x = rng.randint(0, w-1)
        y = rng.randint(0, h-1)
        r = rng.randint(1, 4)
        c = palette[rng.randint(0, len(palette)-1)]
        draw.ellipse([x-r, y-r, x+r, y+r], fill=c)

    img = img.filter(ImageFilter.GaussianBlur(radius=1.5))
    return img


# ─── Chaos Sigil ─────────────────────────────────────────────────

def render_sigil(state, w=400, h=400):
    """Sacred geometry sigil generated from chaos coordinates."""
    img = Image.new('RGB', (w, h), (5, 5, 10))
    draw = ImageDraw.Draw(img)
    palette = palette_from_valence(state.get("llm_valence", 0))

    cx, cy = w // 2, h // 2
    chaos = state.get("chaos_val", 0.5)
    tension = state.get("tension", 50) / 100.0

    # Number of vertices from chaos
    n_verts = int(3 + chaos * 9)  # 3-12 sided polygon
    radius = min(w, h) // 2 - 30

    # Draw nested polygons
    for layer in range(5):
        r = radius * (1 - layer * 0.18)
        phase = state["x"] * 0.1 + layer * 0.5
        points = []
        for i in range(n_verts):
            angle = (2 * math.pi * i / n_verts) + phase
            px = cx + int(r * math.cos(angle))
            py = cy + int(r * math.sin(angle))
            points.append((px, py))

        c = palette[layer % len(palette)]
        brightness = 0.3 + layer * 0.15
        rc = (int(c[0]*brightness), int(c[1]*brightness), int(c[2]*brightness))

        # Draw polygon edges
        for i in range(len(points)):
            j = (i + 1) % len(points)
            draw.line([points[i], points[j]], fill=rc, width=2)

        # Draw inner connections (every k-th vertex)
        k = max(2, n_verts // 3)
        for i in range(len(points)):
            j = (i + k) % len(points)
            inner_c = (int(c[0]*brightness*0.5), int(c[1]*brightness*0.5), int(c[2]*brightness*0.5))
            draw.line([points[i], points[j]], fill=inner_c, width=1)

    # Central dot
    draw.ellipse([cx-5, cy-5, cx+5, cy+5], fill=palette[0])

    # Radiating lines
    for i in range(n_verts * 2):
        angle = (2 * math.pi * i / (n_verts * 2)) + state["y"] * 0.05
        inner_r = 10
        outer_r = radius + 20
        x1 = cx + int(inner_r * math.cos(angle))
        y1 = cy + int(inner_r * math.sin(angle))
        x2 = cx + int(outer_r * math.cos(angle))
        y2 = cy + int(outer_r * math.sin(angle))
        draw.line([(x1,y1),(x2,y2)], fill=(30,30,40), width=1)

    img = img.filter(ImageFilter.GaussianBlur(radius=0.5))
    return img


# ─── Main ────────────────────────────────────────────────────────

MODES = {
    "lorenz": render_lorenz,
    "energy": render_energy,
    "mood": render_mood,
    "sigil": render_sigil,
}

def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <mode> <output_path> [--state PATH]")
        print(f"Modes: {', '.join(MODES.keys())}")
        sys.exit(1)

    mode = sys.argv[1]
    output = sys.argv[2]
    state_path = None

    if "--state" in sys.argv:
        idx = sys.argv.index("--state")
        if idx + 1 < len(sys.argv):
            state_path = sys.argv[idx + 1]

    if mode not in MODES:
        print(f"Unknown mode: {mode}. Available: {', '.join(MODES.keys())}")
        sys.exit(1)

    state = load_chaos_state(state_path or "CHAOS_STATE.json")
    img = MODES[mode](state)
    img.save(output)
    print(f"OK:{output}")


if __name__ == "__main__":
    main()
