use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

use installwizard_core::{Cancellation, Error, Package, Result, normalize_icon};
use installwizard_native::{
    ExportOptions, Overwrite, Project, Source, build, collect, export_web, extract, host_target,
};

use crate::ui::{Action, App, Notice, UiSource};

#[derive(Default)]
pub struct NativeState {
    input: Option<(PathBuf, Package)>,
    receiver: Option<mpsc::Receiver<Notice>>,
    cancel: Cancellation,
}

pub fn launch(package: Option<(PathBuf, Package)>) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 800.0])
            .with_min_inner_size([390.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "InstallWizard",
        options,
        Box::new(move |context| {
            let language = sys_locale::get_locale().unwrap_or_default();
            let mut app = App::new(language.starts_with("de"), &context.egui_ctx);
            app.targets = vec![host_target()];
            if let Some((_, package)) = &package {
                app.wizard = Some(package.manifest.clone());
                app.icon = package.manifest.icon_png.clone();
                app.output.clear();
            }
            app.platform.input = package;
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| {
        Error::Invalid(format!(
            "GUI could not start: {e}. Use --cli for terminal mode."
        ))
    })
}

impl App {
    pub fn poll(&mut self, context: &eframe::egui::Context) {
        let notices = self
            .platform
            .receiver
            .as_ref()
            .map(|receiver| receiver.try_iter().collect::<Vec<_>>())
            .unwrap_or_default();
        for notice in notices {
            self.notice(notice);
        }
        if self.busy {
            context.request_repaint_after(Duration::from_millis(50));
        }
    }

    fn project(&self) -> Project {
        Project {
            version: 1,
            name: self.name.clone(),
            icon: self.icon_path.as_ref().map(PathBuf::from),
            sources: self
                .sources
                .iter()
                .map(|s| Source {
                    source: s.source.clone().into(),
                    destination: s.destination.clone(),
                })
                .collect(),
            targets: self.targets.clone(),
        }
    }

    pub fn handle(&mut self, action: Action, context: &eframe::egui::Context) {
        match action {
            Action::AddFiles => {
                if let Some(paths) = rfd::FileDialog::new().pick_files() {
                    for path in paths {
                        self.add_source(path);
                    }
                }
            }
            Action::AddFolder => {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.add_source(path);
                }
            }
            Action::Icon => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG / ICO", &["png", "ico"])
                    .pick_file()
                {
                    match std::fs::metadata(&path).map_err(Error::from).and_then(|m| {
                        if m.len() > 8 * 1024 * 1024 {
                            return Err(Error::Invalid("Icon exceeds 8 MiB".into()));
                        }
                        normalize_icon(&std::fs::read(&path)?)
                    }) {
                        Ok(icon) => {
                            self.icon = Some(icon);
                            self.icon_path = Some(path.to_string_lossy().into_owned());
                            self.icon_texture = None;
                        }
                        Err(error) => self.notice(Notice::Error(error.to_string())),
                    }
                }
            }
            Action::RemoveIcon => {
                self.icon = None;
                self.icon_path = None;
                self.icon_texture = None;
            }
            Action::Output => {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.output = path.to_string_lossy().into_owned();
                }
            }
            Action::Templates => {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.templates = path.to_string_lossy().into_owned();
                }
            }
            Action::OpenProject => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("InstallWizard project", &["json"])
                    .pick_file()
                {
                    match Project::load(&path) {
                        Ok(project) => {
                            self.name = project.name;
                            self.targets = project.targets;
                            self.sources = project
                                .sources
                                .into_iter()
                                .map(|s| UiSource {
                                    source: s.source.to_string_lossy().into_owned(),
                                    destination: s.destination,
                                })
                                .collect();
                            self.icon_path = project.icon.map(|p| p.to_string_lossy().into_owned());
                            self.icon = self
                                .icon_path
                                .as_ref()
                                .and_then(|path| std::fs::read(path).ok())
                                .and_then(|bytes| normalize_icon(&bytes).ok());
                            self.icon_texture = None;
                            self.preview.clear();
                        }
                        Err(error) => self.notice(Notice::Error(error.to_string())),
                    }
                }
            }
            Action::SaveProject => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("installwizard.json")
                    .save_file()
                {
                    match self.project().save(&path) {
                        Ok(()) => self.notice(Notice::Done(
                            self.text("Projekt gespeichert.", "Project saved.").into(),
                        )),
                        Err(error) => self.notice(Notice::Error(error.to_string())),
                    }
                }
            }
            Action::Preview => match collect(&self.project().sources) {
                Ok(prepared) => self.notice(Notice::Preview(
                    prepared
                        .entries
                        .iter()
                        .map(|e| format!("{}{}", e.path, if e.directory { "/" } else { "" }))
                        .collect(),
                )),
                Err(error) => self.notice(Notice::Error(error.to_string())),
            },
            Action::Cancel => self.platform.cancel.cancel(),
            Action::Build => self.start_job(context),
            Action::Download(index) => {
                let _ = index;
            }
        }
    }

    fn add_source(&mut self, path: PathBuf) {
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            self.sources.push(UiSource {
                source: path.to_string_lossy().into_owned(),
                destination: name.into(),
            });
            self.preview.clear();
        }
    }

    fn start_job(&mut self, context: &eframe::egui::Context) {
        let project = self.project();
        let input = self.platform.input.clone();
        let options = ExportOptions {
            output: PathBuf::from(&self.output),
            templates: if self.templates.is_empty() {
                None
            } else {
                Some(self.templates.clone().into())
            },
            overwrite: self.overwrite,
        };
        let web = self.web;
        let cancel = Cancellation::default();
        self.platform.cancel = cancel.clone();
        let (sender, receiver) = mpsc::channel();
        self.platform.receiver = Some(receiver);
        self.busy = true;
        self.message.clear();
        self.progress = None;
        let context = context.clone();
        std::thread::spawn(move || {
            let mut last = Instant::now() - Duration::from_secs(1);
            let progress = |progress| {
                if last.elapsed() > Duration::from_millis(40) {
                    let _ = sender.send(Notice::Progress(progress));
                    context.request_repaint();
                    last = Instant::now();
                }
            };
            let result: Result<String> = (|| {
                if let Some((path, package)) = input {
                    extract(
                        &mut File::open(path)?,
                        &package,
                        Path::new(&options.output),
                        if options.overwrite {
                            Overwrite::Replace
                        } else {
                            Overwrite::Refuse
                        },
                        &cancel,
                        progress,
                    )?;
                    Ok(format!("Completed: {}", options.output.display()))
                } else {
                    let paths = if web {
                        vec![export_web(&project, &options, &cancel, progress)?]
                    } else {
                        build(&project, &options, &cancel, progress)?
                    };
                    Ok(paths
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join("\n"))
                }
            })();
            let _ = sender.send(match result {
                Ok(message) => Notice::Done(message),
                Err(error) => Notice::Error(error.to_string()),
            });
            context.request_repaint();
        });
    }
}
