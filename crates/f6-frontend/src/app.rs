use std::sync::Mutex;

use egui::{Align, Layout, RichText, ScrollArea, TextEdit, TextStyle};
use f6_types::LegalEntityTIN;
use f6_types::report::TINReport;

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
        report: Option<TINReport>,
        report_queried: bool,
        back_clicked: bool,
    },
}

static REPORT_BUFFER: Mutex<Option<TINReport>> = Mutex::new(None);

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
                        Some(tin) if *clicked => {
                            *self = Self::Query {
                                tin,
                                report: None,
                                report_queried: false,
                                back_clicked: false,
                            }
                        }
                        Some(_) => *error_string = None,
                        None => *error_string = Some("ИНН введён некорректно".into()),
                    }
                }
            }

            Self::Query {
                tin,
                report,
                report_queried,
                back_clicked,
            } => {
                if !*report_queried {
                    debug_assert!(report.is_none());
                    debug_assert!(REPORT_BUFFER.lock().unwrap().is_none());

                    let tin = *tin;
                    let future = async move {
                        let report = reqwest::get(format!("http://localhost:8080/report/{tin}"))
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                        *REPORT_BUFFER.lock().unwrap() = Some(report);

                        #[cfg(not(target_arch = "wasm32"))]
                        tracing::debug!("Wrote response to report buffer");
                    };

                    cfg_select! {
                        not(target_arch = "wasm32") => { tokio::task::spawn(future); }
                        target_arch = "wasm32" => { wasm_bindgen_futures::spawn_local(future); }
                    }

                    *report_queried = true;
                }

                let mut lock = REPORT_BUFFER.lock().unwrap();
                if lock.is_some() {
                    *report = lock.take();
                }
                drop(lock);

                if *back_clicked {
                    *self = Self::default();
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
                tin,
                report,
                report_queried: _,
                back_clicked,
            } => {
                ui.horizontal(|ui| {
                    if report.is_none() {
                        ui.spinner();
                    }

                    ui.strong(format!("Поиск по ИНН: {tin}"));

                    if report.is_some() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            *back_clicked = ui.button("Назад").clicked();
                        });
                    }
                });

                if let Some(report) = report {
                    ui.strong(RichText::from(&report.name).size(body_text_size + 8.0));

                    ui.hyperlink_to(
                        "Отчёт в формате PDF",
                        format!("http://localhost:8080/pdf/{tin}.pdf"),
                    );

                    ui.separator();

                    ScrollArea::vertical().show(ui, |ui| {
                        ui.strong("Обнаруженные домены и поддомены:");

                        for (domain, ip_addr) in &report.ip_addrs {
                            ui.monospace(format!("{domain} ({ip_addr})"));
                        }

                        for (asn, as_info) in &report.ripe_info {
                            let holder = as_info
                                .holder
                                .as_ref()
                                .map_or("неизвестен", |holder| holder.as_str());

                            ui.separator();
                            ui.strong(format!("AS {asn} (владелец: {holder})"));
                            for (domain, ip_addr) in &as_info.domains {
                                ui.monospace(format!("{domain} ({ip_addr})"));
                            }
                        }

                        ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
                    });
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
