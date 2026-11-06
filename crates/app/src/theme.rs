//! Shared visual tokens. Native and WASM render the same fonts and controls.
use eframe::egui::{self, Color32, FontFamily, FontId, Stroke};

pub const CANVAS: Color32 = Color32::from_rgb(246, 248, 249);
pub const SURFACE: Color32 = Color32::WHITE;
pub const TEXT: Color32 = Color32::from_rgb(28, 40, 48);
pub const MUTED: Color32 = Color32::from_rgb(83, 100, 110);
pub const ACCENT: Color32 = Color32::from_rgb(19, 107, 98);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(14, 87, 79);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(232, 244, 241);
pub const BORDER: Color32 = Color32::from_rgb(221, 228, 232);
pub const INPUT_BORDER: Color32 = Color32::from_rgb(122, 138, 147);
pub const ERROR: Color32 = Color32::from_rgb(158, 36, 43);
pub const SUCCESS: Color32 = Color32::from_rgb(19, 89, 80);
pub const CONTENT_WIDTH: f32 = 960.0;

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("ui-semibold".into()))
}

pub fn apply(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "ui-regular".into(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/InstallWizard-Sans-Regular.ttf"
        ))
        .into(),
    );
    fonts.font_data.insert(
        "ui-semibold".into(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/InstallWizard-Sans-Semibold.ttf"
        ))
        .into(),
    );
    fonts.font_data.insert(
        "cjk-fallback".into(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/InstallWizard-CJK.ttf"
        ))
        .into(),
    );
    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(0, "ui-regular".into());
    proportional.push("cjk-fallback".into());
    let mut semibold = proportional.clone();
    semibold[0] = "ui-semibold".into();
    fonts
        .families
        .insert(FontFamily::Name("ui-semibold".into()), semibold);
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("cjk-fallback".into());
    context.set_fonts(fonts);
    let mut style = (*context.style()).clone();
    style.visuals = egui::Visuals::light();
    style.visuals.panel_fill = CANVAS;
    style.visuals.window_fill = SURFACE;
    style.visuals.override_text_color = None;
    style.visuals.weak_text_color = Some(MUTED);
    style.visuals.selection.bg_fill = ACCENT_SOFT;
    style.visuals.selection.stroke = Stroke::new(2.0_f32, ACCENT);
    style.visuals.text_edit_bg_color = Some(SURFACE);
    style.visuals.widgets.noninteractive.fg_stroke.color = TEXT;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    style.visuals.widgets.inactive.bg_fill = SURFACE;
    style.visuals.widgets.inactive.weak_bg_fill = SURFACE;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, INPUT_BORDER);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.5_f32, TEXT);
    style.visuals.widgets.hovered.bg_fill = ACCENT_SOFT;
    style.visuals.widgets.hovered.weak_bg_fill = ACCENT_SOFT;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, TEXT);
    style.visuals.widgets.active.bg_fill = ACCENT_SOFT;
    style.visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
    style.visuals.widgets.active.bg_stroke = Stroke::new(2.0_f32, ACCENT);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, ACCENT);
    style.visuals.widgets.open = style.visuals.widgets.hovered;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::same(8);
    }
    style.spacing.item_spacing = egui::vec2(12.0, 12.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.interact_size.y = 40.0;
    style.spacing.icon_width = 18.0;
    style.spacing.icon_spacing = 9.0;
    style.spacing.window_margin = egui::Margin::same(20);
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(15.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, self::semibold(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, self::semibold(28.0));
    context.set_theme(egui::Theme::Light);
    context.all_styles_mut(|value| *value = style.clone());
}
