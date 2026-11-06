use std::{io::Cursor, sync::Arc};

use eframe::egui;
use installwizard_core::{
    Cancellation, Entry, Error, InputEntry, Manifest, Target, create_archive, memory_artifact,
    normalize_icon, web_bundle,
};
use js_sys::{Array, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::ui::{Action, App, Notice, UiSource};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = InstallWizard, js_name = chooseSources)]
    fn choose_sources(folder: bool);
    #[wasm_bindgen(js_namespace = InstallWizard, js_name = chooseIcon)]
    fn choose_icon();
    #[wasm_bindgen(catch, js_namespace = InstallWizard, js_name = sourceMetadata)]
    fn source_metadata(config: &str) -> std::result::Result<String, JsValue>;
    #[wasm_bindgen(js_namespace = InstallWizard, js_name = takeEvents)]
    fn take_events() -> String;
    #[wasm_bindgen(js_namespace = InstallWizard)]
    fn begin(config: &str);
    #[wasm_bindgen(js_namespace = InstallWizard)]
    fn cancel();
    #[wasm_bindgen(js_namespace = InstallWizard)]
    fn download(index: usize);
}

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    icon_png: Option<Vec<u8>>,
    sources: Vec<UiSource>,
    targets: Vec<Target>,
    web: bool,
}

#[derive(Deserialize)]
struct Event {
    kind: String,
    #[serde(default)]
    sources: Vec<SourceEvent>,
    #[serde(default)]
    message: String,
    #[serde(default)]
    bytes: Vec<u8>,
}
#[derive(Deserialize)]
struct SourceEvent {
    source: String,
    destination: String,
    size: u64,
    directory: bool,
}

/// Start the same Rust UI in a browser canvas. JavaScript only handles browser
/// file APIs, workers, and downloads; package creation runs in Rust/WASM.
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> std::result::Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing window"))?;
    let canvas = window
        .document()
        .and_then(|d| d.get_element_by_id(canvas_id))
        .ok_or_else(|| JsValue::from_str("Missing canvas"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let de = window
        .navigator()
        .language()
        .is_some_and(|language| language.starts_with("de"));
    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(move |context| {
                let mut app = App::new(de, &context.egui_ctx);
                app.targets = vec![Target::WindowsX64];
                Ok(Box::new(app))
            }),
        )
        .await
}

impl App {
    pub fn poll(&mut self, context: &egui::Context) {
        match serde_json::from_str::<Vec<Event>>(&take_events()) {
            Ok(events) => {
                for event in events {
                    match event.kind.as_str() {
                        "sources" => {
                            for source in event.sources {
                                self.sources.push(UiSource {
                                    source: source.source,
                                    destination: source.destination,
                                });
                                self.preview.clear();
                            }
                        }
                        "icon" => match normalize_icon(&event.bytes) {
                            Ok(icon) => {
                                self.icon = Some(icon);
                                self.icon_texture = None;
                            }
                            Err(error) => self.notice(Notice::Error(error.to_string())),
                        },
                        "error" => self.notice(Notice::Error(event.message)),
                        "done" => self.notice(Notice::Done(
                            self.text(
                                "Paket erstellt. Wähle einen Download.",
                                "Package ready. Choose a download.",
                            )
                            .into(),
                        )),
                        "download" => self.downloads.push(event.message),
                        "status" => self.message = event.message,
                        _ => {}
                    }
                }
            }
            Err(error) => self.notice(Notice::Error(error.to_string())),
        }
        context.request_repaint_after(std::time::Duration::from_millis(100));
    }

    pub fn handle(&mut self, action: Action, _: &egui::Context) {
        match action {
            Action::AddFiles => choose_sources(false),
            Action::AddFolder => choose_sources(true),
            Action::Icon => choose_icon(),
            Action::RemoveIcon => {
                self.icon = None;
                self.icon_texture = None;
            }
            Action::Preview => {
                let config = Config {
                    name: self.name.clone(),
                    icon_png: None,
                    sources: self.sources.clone(),
                    targets: self.targets.clone(),
                    web: self.web,
                };
                let entries = serde_json::to_string(&config)
                    .map_err(Error::from)
                    .and_then(|json| {
                        source_metadata(&json).map_err(|e| Error::Invalid(format!("{e:?}")))
                    })
                    .and_then(|json| {
                        serde_json::from_str::<Vec<SourceEvent>>(&json).map_err(Error::from)
                    })
                    .map(|sources| {
                        sources
                            .into_iter()
                            .map(|source| Entry {
                                path: source.destination,
                                directory: source.directory,
                                size: source.size,
                                executable: false,
                                sha256: if source.directory {
                                    String::new()
                                } else {
                                    "0".repeat(64)
                                },
                            })
                            .collect::<Vec<_>>()
                    });
                match entries.and_then(|entries| {
                    let manifest = Manifest {
                        version: 1,
                        name: "validation".into(),
                        icon_png: None,
                        entries,
                    };
                    manifest.validate()?;
                    Ok(manifest
                        .entries
                        .into_iter()
                        .map(|entry| {
                            format!("{}{}", entry.path, if entry.directory { "/" } else { "" })
                        })
                        .collect())
                }) {
                    Ok(entries) => self.notice(Notice::Preview(entries)),
                    Err(error) => self.notice(Notice::Error(error.to_string())),
                }
            }
            Action::Build => {
                let config = Config {
                    name: self.name.clone(),
                    icon_png: self.icon.clone(),
                    sources: self.sources.clone(),
                    targets: self.targets.clone(),
                    web: self.web,
                };
                match serde_json::to_string(&config) {
                    Ok(config) => {
                        self.busy = true;
                        self.error = false;
                        self.downloads.clear();
                        self.message.clear();
                        begin(&config);
                    }
                    Err(error) => self.notice(Notice::Error(error.to_string())),
                }
            }
            Action::Cancel => {
                cancel();
                self.notice(Notice::Error(
                    self.text("Abgebrochen.", "Cancelled.").into(),
                ));
            }
            Action::Download(index) => download(index),
            _ => {}
        }
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// Worker entry: compress once and return a reusable payload plus manifest.
#[wasm_bindgen]
pub fn prepare_payload(config: &str, files: JsValue) -> std::result::Result<JsValue, JsValue> {
    let config: Config = serde_json::from_str(config).map_err(js_error)?;
    let mut entries = Vec::new();
    let mut contents: Vec<Arc<[u8]>> = Vec::new();
    let mut total = 0_u64;
    for file in Array::from(&files) {
        let path = Reflect::get(&file, &"path".into())?
            .as_string()
            .ok_or_else(|| js_error("Missing package path"))?;
        let directory = Reflect::get(&file, &"directory".into())?
            .as_bool()
            .unwrap_or(false);
        let data = Uint8Array::new(&Reflect::get(&file, &"data".into())?).to_vec();
        total += data.len() as u64;
        if total > 500_000_000 {
            return Err(js_error("Browser packages are limited to 500 MB"));
        }
        entries.push(InputEntry {
            path,
            directory,
            size: data.len() as u64,
            executable: false,
        });
        contents.push(Arc::from(data));
    }
    let mut output = Cursor::new(Vec::new());
    let manifest = create_archive(
        &mut output,
        &config.name,
        config.icon_png,
        &entries,
        |index| Ok(Box::new(Cursor::new(contents[index].clone()))),
        &Cancellation::default(),
        |_| {},
    )
    .map_err(js_error)?;
    let result = js_sys::Object::new();
    Reflect::set(
        &result,
        &"manifest".into(),
        &serde_json::to_string(&manifest).map_err(js_error)?.into(),
    )?;
    Reflect::set(
        &result,
        &"payload".into(),
        &Uint8Array::from(output.get_ref().as_slice()),
    )?;
    Ok(result.into())
}

#[wasm_bindgen]
pub fn export_artifact(
    template: &[u8],
    payload: &[u8],
    manifest: &str,
    target: &str,
) -> std::result::Result<Vec<u8>, JsValue> {
    let manifest: Manifest = serde_json::from_str(manifest).map_err(js_error)?;
    if target == "web" {
        let mut output = Cursor::new(Vec::new());
        web_bundle(&mut output, payload, &manifest).map_err(js_error)?;
        return Ok(output.into_inner());
    }
    memory_artifact(
        template,
        payload,
        &manifest,
        Target::parse(target).map_err(js_error)?,
    )
    .map_err(js_error)
}
