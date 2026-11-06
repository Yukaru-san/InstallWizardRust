# UI contrast checks

The shared Rust theme and the semantic web wizard use the same foreground and
background palette. Contrast is measured using WCAG relative luminance in sRGB,
not by eyeballing the screenshots.

Normal text uses a minimum of **4.5:1**. Interactive outlines, checkbox boundaries,
and focus indicators use a minimum of **3:1**. Large text could use 3:1, but this
palette also exceeds 4.5:1 for headings. Selection uses an outline plus a tinted
surface; errors/success also use visible text and distinct icons. No gradients.

Run `python3 tools/check-contrast.py` after changing the Rust palette. It reads
actual source tokens and fails if a required pairing falls below its threshold.

| UI role | Contrast | Minimum | Result |
| --- | ---: | ---: | --- |
| Main text | 15.05:1 | 4.5:1 | Pass |
| Main text on canvas | 14.13:1 | 4.5:1 | Pass |
| Help and placeholder text | 6.15:1 | 4.5:1 | Pass |
| Help text on canvas | 5.77:1 | 4.5:1 | Pass |
| Primary button | 6.35:1 | 4.5:1 | Pass |
| Primary button hover/pressed | 8.42:1 | 4.5:1 | Pass |
| Selected text/tab | 5.64:1 | 4.5:1 | Pass |
| Neutral hovered text | 13.36:1 | 4.5:1 | Pass |
| Error text | 7.66:1 | 4.5:1 | Pass |
| Success text | 8.16:1 | 4.5:1 | Pass |
| Input/checkbox boundary | 3.57:1 | 3.0:1 | Pass |
| Input boundary on canvas | 3.35:1 | 3.0:1 | Pass |
| Focus outline | 6.35:1 | 3.0:1 | Pass |
| Focus outline on canvas | 5.96:1 | 3.0:1 | Pass |
| Hover/focus input boundary | 5.64:1 | 3.0:1 | Pass |

The language selector uses the same outlined selection treatment; focused buttons
have a separate 2 px outer ring, and the HTML wizard uses a 3 px focus outline.
The focus color contrasts against both white content and the surrounding canvas.
The input/checkbox stroke is deliberately darker than decorative separators.

These checks cover enabled controls and readable content in normal, hover,
pressed, selected, focus, error and success states. Disabled controls are excluded
from WCAG contrast requirements. Pale card dividers and user-supplied decorative
icons are not interactive boundaries. This is a contrast check, not certification
of full WCAG conformance or a substitute for screen-reader testing.

References: [WCAG contrast minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html),
[non-text contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html),
[focus appearance](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html).
