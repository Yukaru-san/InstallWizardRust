---
name: InstallWizard
description: Clean, simple, modern file packaging and extraction with flat colors.
colors:
  primary: "#136b62"
  primary-hover: "#0e574f"
  primary-soft: "#e8f4f1"
  canvas: "#f6f8f9"
  surface: "#ffffff"
  text: "#1c2830"
  muted: "#53646e"
  border: "#dde4e8"
  input-border: "#7a8a93"
  error: "#9e242b"
  success: "#135950"
  web-active: "#09473f"
  web-divider: "#edf0f2"
  web-selection: "#c8e5de"
  web-selection-text: "#173c36"
typography:
  headline:
    fontFamily: "InstallWizard Sans Semibold"
    fontSize: "28px"
    fontWeight: 600
  title:
    fontFamily: "InstallWizard Sans Semibold"
    fontSize: "21px"
    fontWeight: 600
  body:
    fontFamily: "InstallWizard Sans Regular"
    fontSize: "15px"
    fontWeight: 400
  label:
    fontFamily: "InstallWizard Sans Semibold"
    fontSize: "14px"
    fontWeight: 600
  caption:
    fontFamily: "InstallWizard Sans Regular"
    fontSize: "13px"
    fontWeight: 400
  web-body:
    fontFamily: 'system-ui, -apple-system, "Segoe UI", sans-serif'
    fontSize: "15px"
    lineHeight: 1.6
  web-headline:
    fontFamily: 'system-ui, -apple-system, "Segoe UI", sans-serif'
    fontSize: "28px"
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: "-0.025em"
rounded:
  checkbox: "3px"
  control: "8px"
  icon: "12px"
  sheet: "14px"
spacing:
  item: "12px"
  button: "10px 16px"
  field: "10px 12px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.surface}"
    rounded: "{rounded.control}"
    padding: "{spacing.button}"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  sheet:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.sheet}"
  field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "{spacing.field}"
  web-download:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.surface}"
    rounded: "{rounded.control}"
    padding: "10px 20px"
  web-container:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.sheet}"
    padding: "36px"
---

# Design System: InstallWizard

## Overview

**Creative North Star: “The File Workbench”** — the existing product's practical, readable workspace. The user's visual commitment is clean, simple and modern across supported platforms, using flat colors with no gradients. Filenames, package paths and task controls lead the hierarchy; teal marks actions and selection.

The native and browser creator share Rust egui components and embedded UI fonts. Native extraction reuses those components in a narrower view. The generated HTML download wizard matches the palette and form language while using semantic HTML and system fonts.

## Colors

Primary teal identifies Continue, Build package, Extract files and Download ZIP; its darker hover state confirms interaction. Pale teal distinguishes selected steps and export modes. White sheets sit on the cool canvas; dark text and muted help text retain a clear hierarchy. Thin neutral borders divide content; stronger input borders identify editable fields. Error and success colors accompany explicit text and small status symbols.

Enabled text targets WCAG AA contrast of at least 4.5:1; interactive boundaries and focus indicators target at least 3:1. The source-token audit in [docs/CONTRAST.md](docs/CONTRAST.md), maintained by `tools/check-contrast.py`, reports a minimum 5.64:1 for normal text and 3.35:1 for control boundaries; teal focus reaches 5.96:1 against the canvas. These measurements cover enabled states. Disabled controls are excluded from these contrast requirements; pale separators and decorative artwork are not interactive boundaries. This contrast audit does not establish full WCAG conformance or replace screen-reader testing.

## Typography

Rust embeds renamed static Inter instances: InstallWizard Sans Regular (400) and Semibold (600), optical size 14. No font service is required. The renamed Noto Sans JP subset remains a fallback in proportional, semibold and monospace families. Preserve this glyph coverage and font license notices.

Main headings are 28px semibold; section headings 21px; application identity 18px; summaries and empty-state prompts 17px; body 15px; labels 14px; captions 13px. Package contents use egui monospace. The HTML wizard uses the system stack, a 28px semibold heading (26px on narrow screens), 15px body and 13px metadata/monospace paths.

## Layout

Rust uses a centered workspace capped at 960px, persistent top identity/language bar, scrolling content and separate bottom status panel. Native extraction caps content at 760px. Outer horizontal margins are 28px, reduced to 16px below 600px viewport width; central vertical padding is 32px, reduced to 24px. Creator sheets use 26px internal padding, reduced to 18px below 600px available width. Extraction sheets use 32px/20px.

Three equal-width steps lead Files → Name & icon → Export. Back and the native Project menu remain secondary, with the primary next/build action aligned right. File mappings place source and package path side by side; below 650px available UI width they stack with visible labels. Add-file/folder controls wrap. Installer targets use three OS rows (Windows, Linux, macOS), each with x64 and ARM64 choices. Installer and Web-Wizard use paired segments.

HTML brand and main container cap at 672px. Body padding is 36px 20px 48px, and the sheet uses 36px. At a 480px viewport or narrower, body padding becomes 24px 16px, sheet padding 24px 20px, and the download fills the content width. Long package names and listed paths wrap; contents scroll at 280px maximum height.

## Elevation & Depth

The workspace is flat: solid canvas and sheets, thin borders and separators. Do not introduce gradients or decorative shadows. There is no authored shadow scale or transition system; unspecified egui popup/window behavior remains framework-owned.

## Shapes

Sheets have 14px corners, controls 8px, optional-icon wells 12px and checkboxes 3px. Inputs have a 1px stronger neutral stroke; sheets use a 1px pale border. Standard Rust controls have a 40px minimum interaction height, primary actions and fields 42px, and segments 44px; compact language choices are 36px. HTML download targets are at least 44px tall.

## Components

**Actions and navigation.** Primary actions use white semibold labels on teal, with darker hover/active fills. Selected steps, export segments and language choices use pale teal, teal text and a 1px teal outline, so selection remains visible beyond the fill color alone. Rust primary, segment, language and icon controls draw a 2px teal keyboard-focus outline outside their bounds. HTML interactive elements use a 3px teal focus-visible outline with 4px offset; native disclosure behavior remains intact.

**Fields and file mappings.** Single-line fields use 12px horizontal/10px vertical padding and a 42px minimum height. Associate visible labels via `labelled_by`, including stacked mapping fields. Source names truncate on desktop with full-path hover text. Mapping changes and removal clear stale validated previews; Check structure rebuilds the listing.

**Icons.** Small authored strokes represent package, folder, file, close and check. Decorative symbols support adjacent text; remove controls retain accessible labels. User-provided PNG/ICO artwork is preserved and shown at 64px rather than replaced with a generated mark.

**Status and recovery.** Busy work disables editing and retains progress/cancel feedback in the bottom panel. Determinate progress shows percentage and current path; errors use text plus color. Missing runtime-template errors explain in English/German that users can choose another platform or Web-Wizard and build again. Build requires sources, a name and an export target; extraction requires a destination.

**HTML download wizard.** A semantic main contains package identity, metadata, direct ZIP download, native contents disclosure and an alert region. The download remains usable without JavaScript; `noscript` explains extraction. Manifest-loaded content supports English/German and preserves user artwork. The browser creator's HTML shell supplies a matching loading/error surface until Rust/WASM starts.

## Do's and Don'ts

- **Do** reuse shared Rust tokens/components for creator and extraction on native and WASM.
- **Do** keep source files and paths inside the package distinct and visibly labelled.
- **Do** preserve EN/DE text, CJK fallback, keyboard focus, cancellation and actionable errors.
- **Do** retain at least 4.5:1 text and 3:1 interactive-boundary/focus contrast, with selected outlines and text/icons for state meaning; update the source-token audit when changing colors.
- **Do** preserve the semantic HTML download/disclosure and user-provided icons.
- **Don't** add gradients, decorative shadows or a new brand identity.
- **Don't** replace informative file/path content with decoration or retain a stale preview.
- **Don't** assume the HTML wizard's system typography or focus width is an exact Rust default.
