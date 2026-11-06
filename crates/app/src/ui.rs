use eframe::egui::{self, Color32, RichText};
use installwizard_core::{Manifest, Progress, Target};

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
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "cjk-fallback".into(),
            egui::FontData::from_static(include_bytes!(
                "../../../assets/fonts/InstallWizard-CJK.ttf"
            ))
            .into(),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .push("cjk-fallback".into());
        }
        context.set_fonts(fonts);
        let mut style = (*context.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = Color32::from_rgb(246, 248, 248);
        style.visuals.window_fill = Color32::WHITE;
        style.visuals.override_text_color = None;
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(32, 45, 50);
        style.visuals.selection.bg_fill = Color32::from_rgb(19, 107, 98);
        style.visuals.selection.stroke.color = Color32::WHITE;
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(231, 237, 237);
        style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(39, 61, 66);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(212, 230, 226);
        style.spacing.item_spacing = egui::vec2(12.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 9.0);
        style.spacing.interact_size.y = 36.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(28.0));
        context.set_theme(egui::Theme::Light);
        context.all_styles_mut(|value| *value = style.clone());
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

    fn draw(&mut self, context: &egui::Context) -> Vec<Action> {
        let mut actions = Vec::new();
        egui::TopBottomPanel::top("top").show(context, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("InstallWizard").size(20.0).strong());
                ui.separator();
                ui.label(self.text(
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
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(!self.de, "EN").clicked() {
                        self.de = false;
                    }
                    if ui.selectable_label(self.de, "DE").clicked() {
                        self.de = true;
                    }
                });
            });
            ui.add_space(6.0);
        });
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.add_space(6.0);
            if self.busy {
                if let Some(progress) = &self.progress {
                    let ratio = if progress.total == 0 {
                        0.0
                    } else {
                        progress.completed as f32 / progress.total as f32
                    };
                    ui.add(egui::ProgressBar::new(ratio).show_percentage());
                    ui.label(&progress.path);
                } else {
                    ui.label(self.text("Paket wird vorbereitet…", "Preparing package…"));
                }
                if ui.button(self.text("Abbrechen", "Cancel")).clicked() {
                    actions.push(Action::Cancel);
                }
            } else if !self.message.is_empty() {
                ui.colored_label(
                    if self.error {
                        Color32::from_rgb(158, 36, 43)
                    } else {
                        Color32::from_rgb(19, 89, 80)
                    },
                    &self.message,
                );
            } else {
                ui.label(
                    RichText::new(self.text(
                        "Deine Dateien. Deine Ordnerstruktur.",
                        "Your files. Your folder structure.",
                    ))
                    .color(Color32::from_rgb(75, 96, 103)),
                );
            }
            ui.add_space(6.0);
        });
        egui::CentralPanel::default().show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(18.0);
                if self.wizard.is_some() {
                    self.draw_wizard(ui, &mut actions);
                } else {
                    self.draw_creator(ui, &mut actions);
                }
                ui.add_space(24.0);
            });
        });
        actions
    }

    fn draw_creator(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.heading(self.text(
            "Aus Dateien wird ein Paket.",
            "Turn files into one package.",
        ));
        ui.label(self.text(
            "Ordnerstruktur festlegen. Plattform wählen. Installer weitergeben.",
            "Arrange folders. Choose platforms. Share your installer.",
        ));
        ui.add_space(18.0);
        ui.add_enabled_ui(!self.busy, |ui| {
            ui.horizontal_wrapped(|ui| {
                let labels = if self.de {
                    ["Dateien", "Name & Icon", "Export"]
                } else {
                    ["Files", "Name & icon", "Export"]
                };
                for (index, label) in labels.iter().enumerate() {
                    if ui
                        .selectable_label(self.step == index, format!("{}  {label}", index + 1))
                        .clicked()
                    {
                        self.step = index;
                    }
                }
            });
            ui.separator();
            ui.add_space(12.0);
            match self.step {
                0 => self.draw_sources(ui, actions),
                1 => self.draw_details(ui, actions),
                _ => self.draw_export(ui, actions),
            }
            ui.add_space(20.0);
            ui.horizontal_wrapped(|ui| {
                if self.step > 0 && ui.button(self.text("Zurück", "Back")).clicked() {
                    self.step -= 1;
                }
                if self.step < 2 && ui.button(self.text("Weiter", "Continue")).clicked() {
                    self.step += 1;
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui
                        .button(self.text("Projekt öffnen", "Open project"))
                        .clicked()
                    {
                        actions.push(Action::OpenProject);
                    }
                    if ui
                        .button(self.text("Projekt speichern", "Save project"))
                        .clicked()
                    {
                        actions.push(Action::SaveProject);
                    }
                }
            });
        });
    }

    fn draw_sources(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
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
            if ui
                .button(self.text("Struktur prüfen", "Check structure"))
                .clicked()
            {
                actions.push(Action::Preview);
            }
        });
        ui.add_space(12.0);
        if self.sources.is_empty() {
            ui.add_space(20.0);
            ui.label(
                RichText::new(self.text(
                    "Welche Dateien möchtest du weitergeben?",
                    "Which files would you like to share?",
                ))
                .size(19.0)
                .strong(),
            );
            ui.label(self.text("Füge einen Ordner hinzu, um seine Struktur zu übernehmen. Für einzelne Dateien legst du den Zielpfad selbst fest.",
                "Add a folder to preserve its structure. Set a package path for each individual file."));
            ui.add_space(20.0);
        } else {
            let de = self.de;
            let mut remove = None;
            let mut changed = false;
            let narrow = ui.available_width() < 650.0;
            egui::ScrollArea::vertical()
                .id_salt("sources")
                .max_height(350.0)
                .show(ui, |ui| {
                    for (index, source) in self.sources.iter_mut().enumerate() {
                        ui.push_id(index, |ui| {
                            if narrow {
                                ui.label(if de { "Quelle" } else { "Source" });
                                ui.label(&source.source);
                                let label = ui.label(if de {
                                    "Pfad im Paket"
                                } else {
                                    "Path inside package"
                                });
                                changed |= ui
                                    .add(
                                        egui::TextEdit::singleline(&mut source.destination)
                                            .desired_width(ui.available_width())
                                            .min_size(egui::vec2(0.0, 36.0))
                                            .hint_text("assets/file.txt"),
                                    )
                                    .labelled_by(label.id)
                                    .changed();
                                if ui.button(if de { "Entfernen" } else { "Remove" }).clicked() {
                                    remove = Some(index);
                                }
                            } else {
                                let width = ui.available_width();
                                let mut label = None;
                                ui.horizontal(|ui| {
                                    ui.add_sized(
                                        [width * 0.4, 20.0],
                                        egui::Label::new(if de { "Quelle" } else { "Source" }),
                                    );
                                    label = Some(
                                        ui.label(if de {
                                            "Pfad im Paket"
                                        } else {
                                            "Path inside package"
                                        })
                                        .id,
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add_sized(
                                        [width * 0.4, 36.0],
                                        egui::Label::new(&source.source).truncate(),
                                    )
                                    .on_hover_text(&source.source);
                                    changed |= ui
                                        .add(
                                            egui::TextEdit::singleline(&mut source.destination)
                                                .desired_width(
                                                    (ui.available_width() - 130.0).max(80.0),
                                                ),
                                        )
                                        .labelled_by(label.expect("destination label"))
                                        .changed();
                                    if ui.button(if de { "Entfernen" } else { "Remove" }).clicked()
                                    {
                                        remove = Some(index);
                                    }
                                });
                            }
                            ui.separator();
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
            }
        }
        ui.label(RichText::new(self.text("Zielpfade sind relativ zum gewählten Installationsordner; Unterordner mit / trennen.",
            "Package paths are relative to the extraction folder; use / for subfolders.")).color(Color32::from_rgb(75, 96, 103)));
        if !self.preview.is_empty() {
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
        let label = ui.label(self.text("Paketname", "Package name"));
        ui.add(
            egui::TextEdit::singleline(&mut self.name)
                .desired_width(420.0_f32.min(ui.available_width())),
        )
        .labelled_by(label.id);
        ui.add_space(16.0);
        ui.label(self.text(
            "Icon (optional, PNG oder ICO)",
            "Icon (optional, PNG or ICO)",
        ));
        self.show_icon(ui);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(self.text("Icon auswählen", "Choose icon"))
                .clicked()
            {
                actions.push(Action::Icon);
            }
            if self.icon.is_some()
                && ui
                    .button(self.text("Icon entfernen", "Remove icon"))
                    .clicked()
            {
                actions.push(Action::RemoveIcon);
            }
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
        }
    }

    fn draw_export(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.label(
            RichText::new(if self.name.is_empty() {
                self.text("Noch kein Paketname", "Package name is missing")
            } else {
                &self.name
            })
            .size(20.0)
            .strong(),
        );
        ui.label(format!(
            "{} {}",
            self.sources.len(),
            self.text("Quellen", "sources")
        ));
        ui.add_space(14.0);
        ui.checkbox(
            &mut self.web,
            if self.de {
                "Web-Wizard als Website-Paket exportieren"
            } else {
                "Export a web wizard website package"
            },
        );
        if !self.web {
            for target in Target::ALL {
                let mut selected = self.targets.contains(&target);
                if ui.checkbox(&mut selected, target.id()).changed() {
                    if selected {
                        self.targets.push(target);
                    } else {
                        self.targets.retain(|t| *t != target);
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.add_space(16.0);
            let label = ui.label(self.text("Ausgabeordner", "Output folder"));
            ui.horizontal_wrapped(|ui| {
                ui.text_edit_singleline(&mut self.output)
                    .labelled_by(label.id);
                if ui.button(self.text("Auswählen", "Browse")).clicked() {
                    actions.push(Action::Output);
                }
            });
            egui::CollapsingHeader::new(self.text("Weitere Optionen", "More options")).show(
                ui,
                |ui| {
                    let label = ui.label(self.text(
                        "Vorbereitete Laufzeitprogramme (für andere Plattformen)",
                        "Runtime templates (for other platforms)",
                    ));
                    ui.horizontal_wrapped(|ui| {
                        ui.text_edit_singleline(&mut self.templates)
                            .labelled_by(label.id);
                        if ui.button(self.text("Auswählen", "Browse")).clicked() {
                            actions.push(Action::Templates);
                        }
                    });
                    ui.checkbox(
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
        ui.label(self.text("Dateien bleiben in deinem Browser. Maximal 500 MB pro Paket. Downloads erscheinen nach dem Erstellen.",
            "Files stay in your browser. Up to 500 MB per package. Downloads appear after building."));
        ui.add_space(20.0);
        let enabled = !self.name.trim().is_empty()
            && !self.sources.is_empty()
            && (self.web || !self.targets.is_empty());
        let button = egui::Button::new(
            RichText::new(self.text("Paket erstellen", "Build package")).color(Color32::WHITE),
        )
        .fill(Color32::from_rgb(19, 107, 98));
        if ui.add_enabled(enabled, button).clicked() {
            actions.push(Action::Build);
        }
        for (index, name) in self.downloads.iter().enumerate() {
            if ui.button(format!("Download: {name}")).clicked() {
                actions.push(Action::Download(index));
            }
        }
    }

    fn draw_wizard(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        self.show_icon(ui);
        let manifest = self.wizard.as_ref().expect("wizard manifest");
        ui.heading(&manifest.name);
        ui.label(self.text("Wähle einen Ordner. Der Wizard entpackt alle Dateien mit ihrer ursprünglichen Struktur.",
            "Choose a folder. The wizard extracts every file with its original folder structure."));
        ui.add_space(16.0);
        egui::CollapsingHeader::new(format!(
            "{} {}",
            manifest.entries.len(),
            self.text("Einträge anzeigen", "entries — show contents")
        ))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for entry in &manifest.entries {
                        ui.monospace(format!(
                            "{}{}",
                            entry.path,
                            if entry.directory { "/" } else { "" }
                        ));
                    }
                });
        });
        ui.add_space(16.0);
        ui.add_enabled_ui(!self.busy, |ui| {
            let label = ui.label(self.text("Zielordner", "Destination folder"));
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.output)
                        .desired_width(ui.available_width().min(420.0)),
                )
                .labelled_by(label.id);
                if ui.button(self.text("Auswählen", "Browse")).clicked() {
                    actions.push(Action::Output);
                }
            });
            ui.checkbox(
                &mut self.overwrite,
                if self.de {
                    "Vorhandene Dateien ausdrücklich ersetzen"
                } else {
                    "Explicitly replace existing files"
                },
            );
            ui.add_space(16.0);
            let button = egui::Button::new(
                RichText::new(self.text("Dateien entpacken", "Extract files"))
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(19, 107, 98));
            if ui
                .add_enabled(!self.output.trim().is_empty(), button)
                .clicked()
            {
                actions.push(Action::Build);
            }
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
