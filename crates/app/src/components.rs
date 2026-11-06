//! Small visual primitives shared by creator and extraction views.
use eframe::egui::{self, Align, Color32, Response, RichText, Stroke, Ui};

use crate::theme;

#[derive(Clone, Copy)]
pub enum Icon {
    Package,
    Folder,
    File,
    Close,
    Check,
}

impl Icon {
    pub fn paint(self, painter: &egui::Painter, rect: egui::Rect, color: Color32) {
        let point = |x: f32, y: f32| {
            egui::pos2(
                rect.left() + rect.width() * x,
                rect.top() + rect.height() * y,
            )
        };
        let stroke = Stroke::new((rect.width() / 14.0).clamp(1.4, 2.0), color);
        let line = |a, b| {
            painter.line_segment([a, b], stroke);
        };
        match self {
            Self::Package => {
                for (a, b) in [
                    ((0.16, 0.28), (0.5, 0.1)),
                    ((0.5, 0.1), (0.84, 0.28)),
                    ((0.84, 0.28), (0.84, 0.72)),
                    ((0.84, 0.72), (0.5, 0.9)),
                    ((0.5, 0.9), (0.16, 0.72)),
                    ((0.16, 0.72), (0.16, 0.28)),
                    ((0.16, 0.28), (0.5, 0.46)),
                    ((0.5, 0.46), (0.84, 0.28)),
                    ((0.5, 0.46), (0.5, 0.9)),
                    ((0.33, 0.19), (0.67, 0.37)),
                ] {
                    line(point(a.0, a.1), point(b.0, b.1));
                }
            }
            Self::Folder => {
                let points = [
                    (0.1, 0.28),
                    (0.1, 0.17),
                    (0.4, 0.17),
                    (0.5, 0.28),
                    (0.9, 0.28),
                    (0.9, 0.81),
                    (0.1, 0.81),
                    (0.1, 0.28),
                ];
                for pair in points.windows(2) {
                    line(point(pair[0].0, pair[0].1), point(pair[1].0, pair[1].1));
                }
            }
            Self::File => {
                for (a, b) in [
                    ((0.24, 0.1), (0.6, 0.1)),
                    ((0.6, 0.1), (0.8, 0.3)),
                    ((0.8, 0.3), (0.8, 0.9)),
                    ((0.8, 0.9), (0.24, 0.9)),
                    ((0.24, 0.9), (0.24, 0.1)),
                    ((0.6, 0.1), (0.6, 0.3)),
                    ((0.6, 0.3), (0.8, 0.3)),
                    ((0.38, 0.5), (0.65, 0.5)),
                    ((0.38, 0.67), (0.65, 0.67)),
                ] {
                    line(point(a.0, a.1), point(b.0, b.1));
                }
            }
            Self::Close => {
                line(point(0.28, 0.28), point(0.72, 0.72));
                line(point(0.72, 0.28), point(0.28, 0.72));
            }
            Self::Check => {
                line(point(0.2, 0.51), point(0.42, 0.72));
                line(point(0.42, 0.72), point(0.82, 0.27));
            }
        }
    }
}

pub fn icon(ui: &mut Ui, kind: Icon, size: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    kind.paint(ui.painter(), rect, color);
}

pub fn icon_button(ui: &mut Ui, kind: Icon, label: &str) -> Response {
    let response = ui
        .add(
            egui::Button::new("")
                .min_size(egui::vec2(40.0, 40.0))
                .frame(false),
        )
        .on_hover_text(label);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    kind.paint(
        ui.painter(),
        response.rect.shrink(10.0),
        if response.hovered() {
            theme::ACCENT
        } else {
            theme::MUTED
        },
    );
    focus_outline(ui, &response);
    response
}

fn focus_outline(ui: &Ui, response: &Response) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.expand(3.0),
            10.0,
            Stroke::new(2.0_f32, theme::ACCENT),
            egui::StrokeKind::Outside,
        );
    }
}

pub fn primary(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    let response = ui
        .scope(|ui| {
            let widgets = &mut ui.visuals_mut().widgets;
            widgets.inactive.weak_bg_fill = theme::ACCENT;
            widgets.inactive.bg_fill = theme::ACCENT;
            widgets.hovered.weak_bg_fill = theme::ACCENT_HOVER;
            widgets.hovered.bg_fill = theme::ACCENT_HOVER;
            widgets.active.weak_bg_fill = theme::ACCENT_HOVER;
            widgets.active.bg_fill = theme::ACCENT_HOVER;
            ui.add_enabled(
                enabled,
                egui::Button::new(RichText::new(label).color(theme::SURFACE))
                    .stroke(Stroke::NONE)
                    .min_size(egui::vec2(128.0, 42.0)),
            )
        })
        .inner;
    focus_outline(ui, &response);
    response
}

pub fn language(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let response = ui.add(
        egui::Button::new(RichText::new(label).size(13.0).color(if selected {
            theme::ACCENT
        } else {
            theme::MUTED
        }))
        .selected(selected)
        .fill(if selected {
            theme::ACCENT_SOFT
        } else {
            Color32::TRANSPARENT
        })
        .stroke(if selected {
            Stroke::new(1.0_f32, theme::ACCENT)
        } else {
            Stroke::NONE
        })
        .min_size(egui::vec2(38.0, 36.0)),
    );
    focus_outline(ui, &response);
    response
}

pub fn segment(ui: &mut Ui, label: &str, selected: bool, width: f32) -> Response {
    let response = ui.add_sized(
        [width, 44.0],
        egui::Button::new(RichText::new(label).color(if selected {
            theme::ACCENT
        } else {
            theme::MUTED
        }))
        .selected(selected)
        .fill(if selected {
            theme::ACCENT_SOFT
        } else {
            Color32::TRANSPARENT
        })
        .stroke(if selected {
            Stroke::new(1.0_f32, theme::ACCENT)
        } else {
            Stroke::NONE
        }),
    );
    focus_outline(ui, &response);
    response
}

pub fn caption(ui: &mut Ui, label: &str) -> Response {
    ui.label(RichText::new(label).size(13.0).color(theme::MUTED))
}

pub fn heading(ui: &mut Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .font(theme::semibold(21.0))
            .color(theme::TEXT),
    );
}

pub fn field<'a>(value: &'a mut String, width: f32) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(value)
        .desired_width((width - 24.0).max(24.0))
        .min_size(egui::vec2(0.0, 42.0))
        .margin(egui::Margin::symmetric(12, 10))
}

pub fn sheet() -> egui::Frame {
    egui::Frame::new()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(14)
}

/// Constrain reading/interaction width without relying on desktop window size.
pub fn centered<R>(ui: &mut Ui, max_width: f32, content: impl FnOnce(&mut Ui) -> R) -> R {
    let available = ui.available_rect_before_wrap();
    let width = available.width().min(max_width);
    let left = available.left() + (available.width() - width) * 0.5;
    let rect = egui::Rect::from_min_max(
        egui::pos2(left, available.top()),
        egui::pos2(left + width, available.bottom()),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(Align::Min)),
        |ui| {
            ui.set_width(width);
            content(ui)
        },
    )
    .inner
}

pub fn checkbox(ui: &mut Ui, value: &mut bool, label: &str) -> Response {
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        for widget in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            widget.corner_radius = egui::CornerRadius::same(3);
        }
        ui.checkbox(value, label)
    })
    .inner
}
