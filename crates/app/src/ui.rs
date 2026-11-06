use eframe::egui::{self, RichText};
use installwizard_core::{Manifest, Progress, Target};

use crate::{
    components::{self as c, Icon},
    theme,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UiSource {
    pub source: String,
    pub destination: String,
}

#[derive(Clone, Copy)]
pub enum Action {
    AddFiles,
    AddFolder,
    Icon,
    RemoveIcon,
    Output,
    #[cfg(not(target_arch = "wasm32"))]
    Templates,
    #[cfg(not(target_arch = "wasm32"))]
    OpenProject,
    #[cfg(not(target_arch = "wasm32"))]
    SaveProject,
    Preview,
    Build,
    Cancel,
    Download(usize),
}

pub enum Notice {
    #[cfg(not(target_arch = "wasm32"))]
    Progress(Progress),
    Done(String),
    Error(String),
    Preview(Vec<String>),
}

pub struct App {
    pub de: bool,
    pub name: String,
    pub sources: Vec<UiSource>,
    pub icon: Option<Vec<u8>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub icon_path: Option<String>,
    pub targets: Vec<Target>,
    pub output: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub templates: String,
    pub web: bool,
    pub overwrite: bool,
    pub wizard: Option<Manifest>,
    pub busy: bool,
    pub message: String,
    pub error: bool,
    pub progress: Option<Progress>,
    pub preview: Vec<String>,
    pub step: usize,
    pub downloads: Vec<String>,
    pub icon_texture: Option<egui::TextureHandle>,
    #[cfg(not(target_arch = "wasm32"))]
    pub platform: crate::native_ui::NativeState,
}

impl App {
    pub fn new(de: bool, context: &egui::Context) -> Self {
        theme::apply(context);
        Self {
            de,
            name: String::new(),
            sources: vec![],
            icon: None,
            #[cfg(not(target_arch = "wasm32"))]
            icon_path: None,
            targets: vec![],
            output: "output".into(),
            #[cfg(not(target_arch = "wasm32"))]
            templates: String::new(),
            web: false,
            overwrite: false,
            wizard: None,
            busy: false,
            message: String::new(),
            error: false,
            progress: None,
            preview: vec![],
            step: 0,
            downloads: vec![],
            icon_texture: None,
            #[cfg(not(target_arch = "wasm32"))]
            platform: Default::default(),
        }
    }

    pub fn text<'a>(&self, de: &'a str, en: &'a str) -> &'a str {
        if self.de { de } else { en }
    }

    pub fn notice(&mut self, notice: Notice) {
        match notice {
            #[cfg(not(target_arch = "wasm32"))]
            Notice::Progress(progress) => {
                self.progress = Some(progress);
            }
            Notice::Done(message) => {
                self.busy = false;
                self.error = false;
                self.message = message;
                self.progress = None;
            }
            Notice::Error(error) => {
                self.busy = false;
                self.error = true;
                self.message = error;
                self.progress = None;
            }
            Notice::Preview(entries) => {
                self.preview = entries;
                self.error = false;
                self.message.clear();
            }
        }
    }

    fn status_message(&self) -> &str {
        let unavailable = self
            .message
            .starts_with("Runtime template is not published: ")
            || self.message.starts_with("No runtime catalog published.")
            || (self.message.starts_with("Missing ") && self.message.contains(" template."));
        if self.error && unavailable {
            self.text(
                "Für die gewählte Plattform kann derzeit kein Installer erstellt werden. Wähle eine andere Plattform oder „Web-Wizard“ und erstelle das Paket erneut.",
                "The selected platform is not available yet. Choose another platform or Web-Wizard, then build the package again.",
            )
        } else {
            &self.message
        }
    }

    fn draw(&mut self, context: &egui::Context) -> Vec<Action> {
        let mut actions = Vec::new();
        let narrow = context.content_rect().width() < 600.0;
        egui::TopBottomPanel::top("top")
            .frame(
                egui::Frame::new()
                    .fill(theme::SURFACE)
                    .inner_margin(egui::Margin::symmetric(if narrow { 16 } else { 28 }, 12)),
            )
            .show(context, |ui| {
                c::centered(ui, theme::CONTENT_WIDTH, |ui| {
                    ui.horizontal(|ui| {
                        c::icon(ui, Icon::Package, 25.0, theme::ACCENT);
                        ui.label(RichText::new("InstallWizard").font(theme::semibold(18.0)));
                        if !narrow {
                            ui.add_space(8.0);
                            c::caption(
                                ui,
                                self.text(
                                    if self.wizard.is_some() {
                                        "Entpacken"
                                    } else {
                                        "Ersteller"
                                    },
                                    if self.wizard.is_some() {
                                        "Extract"
                                    } else {
                                        "Creator"
                                    },
                                ),
                            );
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if c::language(ui, "EN", !self.de).clicked() {
                                self.de = false;
                            }
                            if c::language(ui, "DE", self.de).clicked() {
                                self.de = true;
                            }
                        });
                    });
                });
            });
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(theme::SURFACE)
                    .inner_margin(egui::Margin::symmetric(if narrow { 16 } else { 28 }, 10)),
            )
            .show(context, |ui| {
                c::centered(ui, theme::CONTENT_WIDTH, |ui| {
                    if self.busy {
                        ui.horizontal(|ui| {
                            ui.label(self.text("Paket wird vorbereitet…", "Preparing package…"));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button(self.text("Abbrechen", "Cancel")).clicked() {
                                        actions.push(Action::Cancel);
                                    }
                                },
                            );
                        });
                        if let Some(progress) = &self.progress {
                            let ratio = if progress.total == 0 {
                                0.0
                            } else {
                                progress.completed as f32 / progress.total as f32
                            };
                            ui.add(
                                egui::ProgressBar::new(ratio)
                                    .fill(theme::ACCENT)
                                    .show_percentage(),
                            );
                            c::caption(ui, &progress.path);
                        }
                    } else if !self.message.is_empty() {
                        egui::ScrollArea::vertical()
                            .id_salt("status-scroll")
                            .max_height(108.0)
                            .show(ui, |ui| {
                                ui.horizontal_top(|ui| {
                                    let color = if self.error {
                                        theme::ERROR
                                    } else {
                                        theme::SUCCESS
                                    };
                                    c::icon(
                                        ui,
                                        if self.error { Icon::Close } else { Icon::Check },
                                        18.0,
                                        color,
                                    );
                                    ui.colored_label(color, self.status_message());
                                });
                            });
                    } else {
                        c::caption(
                            ui,
                            self.text(
                                "Deine Dateien. Deine Ordnerstruktur.",
                                "Your files. Your folder structure.",
                            ),
                        );
                    }
                })
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::CANVAS)
                    .inner_margin(egui::Margin::symmetric(
                        if narrow { 16 } else { 28 },
                        if narrow { 24 } else { 32 },
                    )),
            )
            .show(context, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("main-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let width = if self.wizard.is_some() {
                            760.0
                        } else {
                            theme::CONTENT_WIDTH
                        };
                        c::centered(ui, width, |ui| {
                            if self.wizard.is_some() {
                                self.draw_wizard(ui, &mut actions);
                            } else {
                                self.draw_creator(ui, &mut actions);
                            }
                            ui.add_space(16.0);
                        });
                    });
            });
        actions
    }

    fn draw_creator(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.heading(self.text(
            "Aus Dateien wird ein Paket.",
            "Turn files into one package.",
        ));
        ui.label(
            RichText::new(self.text(
                "Ordnerstruktur festlegen. Plattform wählen. Installer weitergeben.",
                "Arrange folders. Choose platforms. Share your installer.",
            ))
            .color(theme::MUTED),
        );
        ui.add_space(16.0);
        ui.add_enabled_ui(!self.busy, |ui| {
            let labels = if self.de {
                ["Dateien", "Name & Icon", "Export"]
            } else {
                ["Files", "Name & icon", "Export"]
            };
            let width = (ui.available_width() - 16.0) / 3.0;
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                ui.spacing_mut().button_padding.x = 8.0;
                ui.horizontal(|ui| {
                    for (index, label) in labels.iter().enumerate() {
                        if c::segment(
                            ui,
                            &format!("{}  {label}", index + 1),
                            self.step == index,
                            width,
                        )
                        .clicked()
                        {
                            self.step = index;
                        }
                    }
                });
            });
            ui.add_space(16.0);
            let padding = if ui.available_width() < 600.0 { 18 } else { 26 };
            c::sheet().inner_margin(padding).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                match self.step {
                    0 => self.draw_sources(ui, actions),
                    1 => self.draw_details(ui, actions),
                    _ => self.draw_export(ui, actions),
                }
            });
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if self.step > 0 && ui.button(self.text("Zurück", "Back")).clicked() {
                    self.step -= 1;
                }
                #[cfg(not(target_arch = "wasm32"))]
                ui.menu_button(self.text("Projekt", "Project"), |ui| {
                    if ui
                        .button(self.text("Projekt öffnen", "Open project"))
                        .clicked()
                    {
                        actions.push(Action::OpenProject);
                        ui.close();
                    }
                    if ui
                        .button(self.text("Projekt speichern", "Save project"))
                        .clicked()
                    {
                        actions.push(Action::SaveProject);
                        ui.close();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.step < 2 {
                        if c::primary(ui, self.text("Weiter", "Continue"), true).clicked() {
                            self.step += 1;
                        }
                    } else if c::primary(
                        ui,
                        self.text("Paket erstellen", "Build package"),
                        self.build_ready(),
                    )
                    .clicked()
                    {
                        actions.push(Action::Build);
                    }
                });
            });
        });
    }

    fn build_ready(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.sources.is_empty()
            && (self.web || !self.targets.is_empty())
    }

    fn draw_sources(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        c::heading(ui, self.text("Dateien", "Files"));
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(self.text("Dateien hinzufügen", "Add files"))
                .clicked()
            {
                actions.push(Action::AddFiles);
            }
            if ui
                .button(self.text("Ordner hinzufügen", "Add folder"))
                .clicked()
            {
                actions.push(Action::AddFolder);
            }
        });
        if self.sources.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                c::icon(ui, Icon::Folder, 38.0, theme::MUTED);
                ui.add_space(8.0);
                ui.label(RichText::new(self.text("Welche Dateien möchtest du weitergeben?", "Which files would you like to share?"))
                    .font(theme::semibold(17.0)));
                ui.add(egui::Label::new(RichText::new(self.text(
                    "Füge einen Ordner hinzu, um seine Struktur zu übernehmen. Für einzelne Dateien legst du den Zielpfad selbst fest.",
                    "Add a folder to preserve its structure. Set a package path for each individual file.")).color(theme::MUTED)).wrap());
            });
            ui.add_space(20.0);
            return;
        }
        ui.add_space(16.0);
        let de = self.de;
        let narrow = ui.available_width() < 650.0;
        let mut changed = false;
        let mut remove = None;
        egui::ScrollArea::vertical()
            .id_salt("sources")
            .max_height(360.0)
            .show(ui, |ui| {
                for (index, source) in self.sources.iter_mut().enumerate() {
                    ui.push_id(index, |ui| {
                        #[cfg(target_arch = "wasm32")]
                        let display = source
                            .source
                            .split_once(" · ")
                            .map_or(source.source.as_str(), |(_, name)| name);
                        #[cfg(not(target_arch = "wasm32"))]
                        let display = source.source.as_str();
                        if narrow {
                            ui.horizontal(|ui| {
                                c::caption(ui, if de { "Quelle" } else { "Source" });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if c::icon_button(
                                            ui,
                                            Icon::Close,
                                            if de {
                                                "Quelle entfernen"
                                            } else {
                                                "Remove source"
                                            },
                                        )
                                        .clicked()
                                        {
                                            remove = Some(index);
                                        }
                                    },
                                );
                            });
                            ui.add(egui::Label::new(display).wrap());
                            let label = c::caption(
                                ui,
                                if de {
                                    "Pfad im Paket"
                                } else {
                                    "Path inside package"
                                },
                            );
                            changed |= ui
                                .add(
                                    c::field(&mut source.destination, ui.available_width())
                                        .hint_text("assets/file.txt"),
                                )
                                .labelled_by(label.id)
                                .changed();
                        } else {
                            let source_width = ui.available_width() * 0.4;
                            let mut label = None;
                            ui.horizontal(|ui| {
                                ui.allocate_ui(egui::vec2(source_width, 18.0), |ui| {
                                    ui.set_min_width(source_width);
                                    c::caption(ui, if de { "Quelle" } else { "Source" });
                                });
                                label = Some(
                                    c::caption(
                                        ui,
                                        if de {
                                            "Pfad im Paket"
                                        } else {
                                            "Path inside package"
                                        },
                                    )
                                    .id,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(source_width, 42.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.set_min_size(egui::vec2(source_width, 42.0));
                                        c::icon(ui, Icon::File, 19.0, theme::MUTED);
                                        ui.add(egui::Label::new(display).truncate())
                                            .on_hover_text(display);
                                    },
                                );
                                let width = (ui.available_width() - 52.0).max(80.0);
                                changed |= ui
                                    .add(c::field(&mut source.destination, width))
                                    .labelled_by(label.expect("destination label"))
                                    .changed();
                                if c::icon_button(
                                    ui,
                                    Icon::Close,
                                    if de {
                                        "Quelle entfernen"
                                    } else {
                                        "Remove source"
                                    },
                                )
                                .clicked()
                                {
                                    remove = Some(index);
                                }
                            });
                        }
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);
                    });
                }
            });
        if changed {
            self.preview.clear();
            self.message.clear();
        }
        if let Some(index) = remove {
            self.sources.remove(index);
            self.preview.clear();
            self.message.clear();
        }
        c::caption(
            ui,
            self.text(
                "Zielpfade sind relativ zum Installationsordner. Unterordner mit / trennen.",
                "Package paths are relative to the extraction folder. Use / for subfolders.",
            ),
        );
        if ui
            .button(self.text("Struktur prüfen", "Check structure"))
            .clicked()
        {
            actions.push(Action::Preview);
        }
        if !self.preview.is_empty() {
            ui.add_space(8.0);
            egui::CollapsingHeader::new(
                self.text("Geprüfter Paketinhalt", "Validated package contents"),
            )
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("preview")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for path in &self.preview {
                            ui.monospace(path);
                        }
                    });
            });
        }
    }

    fn draw_details(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        c::heading(ui, self.text("Name & Icon", "Name & icon"));
        ui.add_space(12.0);
        let label = ui.label(
            RichText::new(self.text("Paketname", "Package name")).font(theme::semibold(14.0)),
        );
        ui.add(
            c::field(&mut self.name, ui.available_width()).hint_text(if self.de {
                "Mein Projekt"
            } else {
                "My project"
            }),
        )
        .labelled_by(label.id);
        c::caption(
            ui,
            self.text(
                "Dieser Name erscheint im InstallWizard.",
                "This name appears in the InstallWizard.",
            ),
        );
        ui.add_space(18.0);
        ui.separator();
        ui.add_space(14.0);
        ui.horizontal_top(|ui| {
            self.show_icon(ui);
            ui.vertical(|ui| {
                ui.label(RichText::new(self.text("Icon", "Icon")).font(theme::semibold(15.0)));
                c::caption(
                    ui,
                    self.text("Optional · PNG oder ICO", "Optional · PNG or ICO"),
                );
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(self.text("Icon auswählen", "Choose icon"))
                        .clicked()
                    {
                        actions.push(Action::Icon);
                    }
                    if self.icon.is_some() && ui.button(self.text("Entfernen", "Remove")).clicked()
                    {
                        actions.push(Action::RemoveIcon);
                    }
                });
            });
        });
    }

    fn show_icon(&mut self, ui: &mut egui::Ui) {
        if self.icon_texture.is_none()
            && let Some(bytes) = &self.icon
            && let Ok(image) = image::load_from_memory(bytes)
        {
            let image = image.to_rgba8();
            self.icon_texture = Some(ui.ctx().load_texture(
                "package-icon",
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                ),
                Default::default(),
            ));
        }
        if let Some(texture) = &self.icon_texture {
            ui.image((texture.id(), egui::vec2(64.0, 64.0)));
        } else {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 12.0, theme::CANVAS);
            Icon::Package.paint(ui.painter(), rect.shrink(15.0), theme::MUTED);
        }
    }

    fn draw_export(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        c::heading(ui, self.text("Export", "Export"));
        ui.add_space(8.0);
        ui.label(
            RichText::new(if self.name.is_empty() {
                self.text("Paketname fehlt", "Package name is missing")
            } else {
                &self.name
            })
            .font(theme::semibold(17.0)),
        );
        c::caption(
            ui,
            &format!(
                "{} {}",
                self.sources.len(),
                if self.sources.len() == 1 {
                    self.text("Quelle", "source")
                } else {
                    self.text("Quellen", "sources")
                }
            ),
        );
        ui.add_space(16.0);
        let width = ((ui.available_width() - 8.0) / 2.0).min(220.0);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.horizontal(|ui| {
                if c::segment(ui, "Installer", !self.web, width).clicked() {
                    self.web = false;
                }
                if c::segment(ui, "Web-Wizard", self.web, width).clicked() {
                    self.web = true;
                }
            });
        });
        ui.add_space(14.0);
        if self.web {
            ui.label(self.text(
                "Website-Paket mit ZIP-Download und vollständiger Ordnerstruktur.",
                "A website package with a ZIP download and the complete folder structure.",
            ));
        } else {
            ui.label(
                RichText::new(self.text("Plattformen", "Platforms")).font(theme::semibold(14.0)),
            );
            for (name, format, x64, arm) in [
                ("Windows", ".exe", Target::WindowsX64, Target::WindowsArm64),
                ("Linux", "ELF", Target::LinuxX64, Target::LinuxArm64),
                ("macOS", ".app.zip", Target::MacosX64, Target::MacosArm64),
            ] {
                ui.horizontal(|ui| {
                    let width = (ui.available_width() * 0.4).max(90.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, 40.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_min_width(width);
                            ui.label(name);
                            if ui.available_width() > 80.0 {
                                c::caption(ui, format);
                            }
                        },
                    );
                    self.target_checkbox(ui, x64, "x64");
                    self.target_checkbox(ui, arm, "ARM64");
                });
                ui.separator();
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.add_space(18.0);
            let label = ui.label(
                RichText::new(self.text("Ausgabeordner", "Output folder"))
                    .font(theme::semibold(14.0)),
            );
            ui.horizontal(|ui| {
                ui.add(c::field(
                    &mut self.output,
                    (ui.available_width() - 52.0).max(80.0),
                ))
                .labelled_by(label.id);
                if c::icon_button(
                    ui,
                    Icon::Folder,
                    self.text("Ausgabeordner auswählen", "Choose output folder"),
                )
                .clicked()
                {
                    actions.push(Action::Output);
                }
            });
            ui.add_space(8.0);
            egui::CollapsingHeader::new(self.text("Weitere Optionen", "More options")).show(
                ui,
                |ui| {
                    let label =
                        ui.label(self.text("Vorbereitete Laufzeitprogramme", "Runtime templates"));
                    ui.horizontal(|ui| {
                        ui.add(c::field(
                            &mut self.templates,
                            (ui.available_width() - 52.0).max(80.0),
                        ))
                        .labelled_by(label.id);
                        if c::icon_button(
                            ui,
                            Icon::Folder,
                            self.text("Vorlagen auswählen", "Choose templates"),
                        )
                        .clicked()
                        {
                            actions.push(Action::Templates);
                        }
                    });
                    c::checkbox(
                        ui,
                        &mut self.overwrite,
                        if self.de {
                            "Vorhandene Exportdateien ersetzen"
                        } else {
                            "Replace existing export files"
                        },
                    );
                },
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            ui.add_space(16.0);
            c::caption(
                ui,
                self.text(
                    "Dateien bleiben in deinem Browser. Maximal 500 MB pro Paket.",
                    "Files stay in your browser. Up to 500 MB per package.",
                ),
            );
        }
        if !self.downloads.is_empty() {
            ui.add_space(16.0);
            ui.separator();
            ui.label(
                RichText::new(self.text("Downloads", "Downloads")).font(theme::semibold(15.0)),
            );
            for (index, name) in self.downloads.iter().enumerate() {
                if ui.button(name).clicked() {
                    actions.push(Action::Download(index));
                }
            }
        }
    }

    fn target_checkbox(&mut self, ui: &mut egui::Ui, target: Target, label: &str) {
        let mut selected = self.targets.contains(&target);
        let response = c::checkbox(ui, &mut selected, label);
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Checkbox,
                ui.is_enabled(),
                selected,
                target.id(),
            )
        });
        if response.changed() {
            if selected {
                self.targets.push(target);
            } else {
                self.targets.retain(|item| *item != target);
            }
        }
    }

    fn draw_wizard(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let padding = if ui.available_width() < 600.0 { 20 } else { 32 };
        c::sheet().inner_margin(padding).show(ui,|ui| {
            self.show_icon(ui);
            ui.add_space(8.0);
            let manifest = self.wizard.as_ref().expect("wizard manifest");
            ui.heading(&manifest.name);
            ui.label(RichText::new(self.text("Wähle einen Ordner. Der Wizard entpackt alle Dateien mit ihrer ursprünglichen Struktur.",
                "Choose a folder. The wizard extracts every file with its original folder structure.")).color(theme::MUTED));
            ui.add_space(12.0);
            egui::CollapsingHeader::new(format!("{} {}",manifest.entries.len(),self.text("Einträge anzeigen", "entries — show contents")))
                .show(ui,|ui| {
                    egui::ScrollArea::vertical().id_salt("wizard-contents").max_height(200.0).show(ui,|ui| {
                        for entry in &manifest.entries { ui.monospace(format!("{}{}",entry.path,if entry.directory { "/" } else { "" })); }
                    });
                });
            ui.add_space(14.0);
            ui.separator();
            ui.add_space(14.0);
            ui.add_enabled_ui(!self.busy,|ui| {
                let label = ui.label(RichText::new(self.text("Zielordner", "Destination folder")).font(theme::semibold(14.0)));
                ui.horizontal(|ui| {
                    ui.add(c::field(&mut self.output,(ui.available_width()-52.0).max(80.0))).labelled_by(label.id);
                    if c::icon_button(ui,Icon::Folder,self.text("Zielordner auswählen", "Choose destination folder")).clicked() { actions.push(Action::Output); }
                });
                ui.add_space(4.0);
                c::checkbox(ui,&mut self.overwrite,if self.de { "Vorhandene Dateien ausdrücklich ersetzen" } else { "Explicitly replace existing files" });
                ui.add_space(14.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {
                    if c::primary(ui,self.text("Dateien entpacken", "Extract files"),!self.output.trim().is_empty()).clicked() { actions.push(Action::Build); }
                });
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        self.poll(context);
        for action in self.draw(context) {
            self.handle(action, context);
        }
    }
}
