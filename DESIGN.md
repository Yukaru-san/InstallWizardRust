---
name: InstallWizard
description: A restrained file workbench for packaging and extracting folders.
colors:
  primary: "#136b62"
  canvas: "#f6f8f8"
  content: "#ffffff"
  control: "#e7eded"
  text: "#202d32"
  control-text: "#273d42"
  secondary-text: "#4b6067"
  hover: "#d4e6e2"
  error: "#9e242b"
  success: "#135950"
  web-canvas: "#f5f7f7"
  web-border: "#dce3e4"
  web-focus: "#16443d"
  web-error: "#a0272c"
  web-selection: "#bfdfd8"
  web-selection-text: "#173c36"
typography:
  headline:
    fontFamily: "egui proportional with bundled CJK fallback"
    fontSize: "28px"
  title:
    fontFamily: "egui proportional with bundled CJK fallback"
    fontSize: "20px"
  body:
    fontFamily: "egui proportional with bundled CJK fallback"
    fontSize: "15px"
  label:
    fontFamily: "egui proportional with bundled CJK fallback"
    fontSize: "15px"
  web-body:
    fontFamily: "system-ui, sans-serif"
    fontSize: "16px"
    lineHeight: 1.6
  web-headline:
    fontFamily: "system-ui, sans-serif"
    fontSize: "2rem"
    lineHeight: 1.2
rounded:
  web-action: "6px"
  web-container: "12px"
spacing:
  item-horizontal: "12px"
  item-vertical: "10px"
  button-horizontal: "14px"
  button-vertical: "9px"
  web-container: "32px"
  web-page: "clamp(20px, 5vw, 64px)"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.content}"
    typography: "{typography.label}"
    padding: "9px 14px"
  button-neutral:
    backgroundColor: "{colors.control}"
    textColor: "{colors.control-text}"
    typography: "{typography.label}"
    padding: "9px 14px"
  button-neutral-hover:
    backgroundColor: "{colors.hover}"
  web-download:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.content}"
    rounded: "{rounded.web-action}"
    padding: "12px 22px"
  web-container:
    backgroundColor: "{colors.content}"
    rounded: "{rounded.web-container}"
    padding: "{spacing.web-container}"
---

# Design System: InstallWizard

## Overview

**Creative North Star: "The File Workbench"**

The implemented interface is a quiet, light workbench: filenames, package paths and task controls carry the visual hierarchy. Teal identifies primary actions and selection; cool neutrals support readable forms and compact navigation. This documents the incumbent Rust egui interface and semantic HTML download wizard rather than a replacement identity.

The shared Rust UI serves the creator and a separate extraction view on native platforms; the browser creator uses the same egui design. The generated web wizard is a smaller semantic HTML surface with its own explicitly recorded spacing, type and background variations.

**Key Characteristics:**

- Light surfaces with restrained teal emphasis.
- Visible source-to-package mappings and labelled fields.
- Compact, wrapping controls and persistent status in the Rust UI.
- A direct ZIP download and optional contents disclosure in the HTML wizard.

## Colors

### Primary

Muted teal (`primary`) marks Build package, Extract files and the HTML download action. It also supplies egui's selection fill with white selected text. The pale `hover` tone is the explicit egui hover fill.

### Neutral

`canvas` backs the Rust panels; `content` is the window fill and HTML main container. `control` and `control-text` define inactive egui controls, while `text` supplies ordinary text. `secondary-text` carries help and idle status. The HTML page uses the distinct `web-canvas` and `web-border` values already present in its stylesheet.

Errors and completed notices use `error` and `success` in egui. HTML errors use `web-error`. HTML keyboard focus uses `web-focus`; text selection uses `web-selection` with `web-selection-text`. These values are surface-specific, not an additional accent palette.

## Typography

Rust uses egui's default proportional family for UI text. Body and buttons share the body scale; headings use the headline scale. The application bar and export summary use the title scale with strong emphasis; the empty-state prompt is a strong proportional label (19px). Package-content paths use egui monospace.

The embedded `assets/fonts/InstallWizard-CJK.ttf` is a renamed Noto Sans JP subset appended as `cjk-fallback` to both proportional and monospace families. Preserve this glyph coverage. The HTML wizard uses the system font stack; its heading and body sizes are recorded separately in frontmatter. No custom display family is introduced.

## Layout

The Rust interface has a compact role/language bar, a vertically scrolling central area and a separate bottom status panel. Item spacing and button padding follow the frontmatter tokens; the minimum interaction height is (36px). Introductory space is (18px); common section gaps are (12px), (16px) and (20px).

Creator steps are Files → Name & icon → Export, with direct step selection and Back/Continue controls. Button groups and step navigation wrap. Source mappings place source and package destination side by side when space permits, with source labels taking 40% of row width. Below (650px) of available UI width, each mapping stacks its Source label, filename, Path inside package field and Remove action. Desktop source names truncate with a full-path hover tooltip. The source list scrolls up to (350px); validated contents scroll up to (200px).

Package-name and wizard destination fields cap at (420px) while fitting available width. The HTML body has a maximum width of (48rem), fluid page padding and a padded main container. Long listed paths wrap anywhere. It uses fluid layout rather than a media-query breakpoint.

## Elevation & Depth

The application surfaces use tonal separation, separators and native egui widget treatments. There is no project-defined shadow scale or motion system. The HTML main container is flat, with a thin border and no shadow. Preserve framework defaults where the Rust code does not override them.

## Shapes

Rust controls retain egui's native shape defaults; no project radius token overrides them. The HTML surface has softly rounded main-container and download-action corners, recorded in frontmatter, plus a thin container border (1px). Optional icons display at (64 × 64px) on both surfaces.

## Components

### Buttons and navigation

Primary Rust actions use white text on teal; neutral actions use the egui widget system and explicit inactive/hover fills. Steps and EN/DE language choices are selectable labels. Busy creator work disables its editing region; extraction likewise disables its destination controls. Native jobs run on threads and browser builds use workers so status can update independently.

### Inputs and source mappings

Package name, package destination, native output/template paths and extraction destination have visible labels associated through egui `labelled_by`. The narrow mapping field keeps the same explicit label as desktop. Changing destination mappings clears the validated preview and its message; removing a source clears the preview. The Check structure action rebuilds the visible content listing.

### Progress and feedback

The bottom Rust panel shows an idle message, inline completion/error text, or busy feedback and Cancel. Determinate progress includes a percentage and current path; preparation has a textual busy state. Build package requires a nonempty name, sources, and either a web export or selected native target. Extract files requires a destination. Error meaning is conveyed by text as well as color.

### HTML download wizard

A semantic main container holds the optional decorative icon, package heading, explanatory paragraph, download link, contents disclosure and alert paragraph. The download anchor remains available without JavaScript, with extraction instructions in `noscript`. The link has an explicit focus-visible outline (3px) and offset (4px); no custom hover or active treatment is defined. The native `details`/`summary` element supplies disclosure behavior. English and German copy follow browser language after manifest loading.

## Do's and Don'ts

### Do:

- **Do** preserve the distinction between source files and paths inside the package.
- **Do** associate visible field labels with their controls and keep mapping labels in stacked layouts.
- **Do** retain native egui behavior where this system has no explicit override.
- **Do** preserve the HTML download link, semantic disclosure and visible keyboard focus.

### Don't:

- **Don't** show a validated preview after a mapping change without checking structure again.
- **Don't** substitute decorative imagery for file and path information.
- **Don't** treat the HTML wizard's surface-specific tokens as exact Rust widget defaults.
