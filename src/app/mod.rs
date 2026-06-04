use egui::{RichText, ScrollArea, TextEdit, TextStyle};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use crate::api::Api;
use crate::api::fns::FnsApi;
use crate::key_registry::fns::FnsApiKey;
use crate::types::LegalEntityTIN;

pub const APP_NAME: &str = env!("CARGO_CRATE_NAME");

static RESPONSE_BUFFER: Mutex<Option<Result<serde_json::Value, reqwest::Error>>> = Mutex::new(None);

#[derive(Debug)]
#[must_use]
pub enum App {
    Landing {
        tin: String,
        error: Option<String>,
        clicked: bool,
    },

    Query {
        tin: LegalEntityTIN,
        fns_api_key: FnsApiKey,
        response_json: Option<Result<String, String>>,
    },
}

impl Default for App {
    fn default() -> Self {
        Self::Landing {
            tin: String::new(),
            error: None,
            clicked: false,
        }
    }
}

impl App {
    pub fn advance(&mut self) {
        match self {
            Self::Landing {
                tin: tin_string,
                error: error_string,
                clicked,
            } => {
                if tin_string.trim().is_empty() {
                    *error_string = None;
                } else {
                    match tin_string
                        .parse::<u64>()
                        .ok()
                        .and_then(|n| LegalEntityTIN::try_new(n).ok())
                    {
                        Some(parsed_tin) if *clicked => {
                            *self = Self::Query {
                                tin: parsed_tin,
                                fns_api_key: FnsApiKey::from_str(
                                    "d720bff6d7647a52f1db847e4760ee823af5e57d",
                                )
                                .unwrap(),
                                response_json: None,
                            }
                        }
                        Some(_) => *error_string = None,
                        None => *error_string = Some("ИНН введён некорректно".into()),
                    }
                }
            }

            Self::Query {
                tin,
                fns_api_key,
                response_json,
            } => {
                let api_key = fns_api_key.clone();
                let tin = *tin;

                let lock = RESPONSE_BUFFER.lock().unwrap();

                if lock.is_none() {
                    cfg_select! {
                        not(target_arch = "wasm32") => {
                            tokio::task::spawn(async move {
                                let json_value = FnsApi.fetch_egr(api_key, tin).await;
                                *RESPONSE_BUFFER.lock().unwrap() = Some(json_value);
                            });
                        }

                        target_arch = "wasm32" => {
                            wasm_bindgen_futures::spawn_local(async move {
                                let json_value = FnsApi.fetch_egr(api_key, tin).await;
                                *RESPONSE_BUFFER.lock().unwrap() = Some(json_value);
                            });
                        }
                    }
                }

                if let Some(response) = &*lock {
                    match response {
                        Err(error) => *response_json = Some(Err(error.to_string())),
                        Ok(value) => {
                            *response_json =
                                Some(Ok(serde_json::to_string_pretty(&value).unwrap()));
                        }
                    }
                }
            }
        }
    }

    pub fn main_ui(&mut self, ui: &mut egui::Ui) {
        let body_text_size = ui.text_style_height(&TextStyle::Body);
        match self {
            Self::Landing {
                tin: tin_string,
                error: error_string,
                clicked,
            } => {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.2);

                    let heading =
                        RichText::from("Поиск и визуализация \n инфраструктуры компаний \n по ИНН")
                            .size(body_text_size + 12.0)
                            .strong();
                    ui.label(heading);

                    ui.label(
                        "
                        Введите ИНН, чтобы получить карточку компании, связанные
                        домены и сетевую инфраструктуру в одном отчёте
                        ",
                    );

                    let inn_edit_field = TextEdit::singleline(tin_string)
                        .code_editor()
                        .hint_text("ИНН")
                        .desired_width(150.0);
                    ui.add(inn_edit_field);

                    *clicked = ui.button("Найти").clicked();

                    if let Some(error) = error_string {
                        ui.colored_label(ui.visuals().warn_fg_color, error);
                    }
                });
            }

            Self::Query {
                response_json, tin, ..
            } => {
                ui.strong(format!("Поиск по ИНН: {tin}"));

                match response_json {
                    Some(Ok(response_json)) => {
                        ScrollArea::vertical().show(ui, |ui| {
                            let response_text_field = TextEdit::multiline(response_json)
                                .interactive(false)
                                .code_editor()
                                .desired_width(f32::INFINITY);
                            ui.add(response_text_field);
                        });
                    }

                    Some(Err(error)) => {
                        ui.colored_label(ui.visuals().error_fg_color, error);
                    }

                    None => {
                        ui.weak("Поиск в FNS API...");
                        ui.request_repaint_after(Duration::from_secs(1));
                    }
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.advance();

        ui.set_zoom_factor(1.5);

        egui::CentralPanel::default().show_inside(ui, |ui| self.main_ui(ui));
    }
}
