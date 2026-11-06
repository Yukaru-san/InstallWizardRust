#!/usr/bin/env python3
"""Check the actual Rust palette against WCAG text and non-text thresholds."""
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / 'crates/app/src/theme.rs').read_text()
colors = {name: (255, 255, 255) for name in re.findall(r'pub const (\w+): Color32 = Color32::WHITE;', source)}
for name, r, g, b in re.findall(r'pub const (\w+): Color32 = Color32::from_rgb\((\d+), (\d+), (\d+)\)', source):
    colors[name] = (int(r), int(g), int(b))

def luminance(color):
    channels = [channel / 255 for channel in color]
    linear = [channel / 12.92 if channel <= .04045 else ((channel + .055) / 1.055) ** 2.4 for channel in channels]
    return sum(channel * weight for channel, weight in zip(linear, [.2126, .7152, .0722]))

def contrast(foreground, background):
    values = sorted([luminance(foreground), luminance(background)])
    return (values[1] + .05) / (values[0] + .05)

pairs = [
    ('Main text', 'TEXT', 'SURFACE', 4.5),
    ('Main text on canvas', 'TEXT', 'CANVAS', 4.5),
    ('Help and placeholder text', 'MUTED', 'SURFACE', 4.5),
    ('Help text on canvas', 'MUTED', 'CANVAS', 4.5),
    ('Primary button', 'SURFACE', 'ACCENT', 4.5),
    ('Primary button hover/pressed', 'SURFACE', 'ACCENT_HOVER', 4.5),
    ('Selected text/tab', 'ACCENT', 'ACCENT_SOFT', 4.5),
    ('Neutral hovered text', 'TEXT', 'ACCENT_SOFT', 4.5),
    ('Error text', 'ERROR', 'SURFACE', 4.5),
    ('Success text', 'SUCCESS', 'SURFACE', 4.5),
    ('Input/checkbox boundary', 'INPUT_BORDER', 'SURFACE', 3.0),
    ('Input boundary on canvas', 'INPUT_BORDER', 'CANVAS', 3.0),
    ('Focus outline', 'ACCENT', 'SURFACE', 3.0),
    ('Focus outline on canvas', 'ACCENT', 'CANVAS', 3.0),
    ('Hover/focus input boundary', 'ACCENT', 'ACCENT_SOFT', 3.0),
]
failed = []
print('| UI role | Contrast | Minimum | Result |')
print('| --- | ---: | ---: | --- |')
for role, foreground, background, threshold in pairs:
    ratio = contrast(colors[foreground], colors[background])
    passed = ratio >= threshold
    print(f'| {role} | {ratio:.2f}:1 | {threshold:.1f}:1 | {"Pass" if passed else "FAIL"} |')
    if not passed:
        failed.append(role)
if failed:
    sys.exit('Insufficient contrast: ' + ', '.join(failed))
