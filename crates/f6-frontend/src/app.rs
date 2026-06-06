use std::sync::Mutex;

use egui::{
    Align, Button, Color32, CornerRadius, Frame, Key, Layout, Margin, RichText, Stroke, TextEdit,
    TextStyle, Vec2,
};
use f6_types::{LegalEntityTIN, report::TINReport};

const PAGE_BACKGROUND: Color32 = Color32::from_rgb(15, 15, 15);
const CARD_BACKGROUND: Color32 = Color32::from_rgb(24, 24, 24);
const INPUT_BACKGROUND: Color32 = Color32::from_rgb(36, 36, 36);
const BORDER_COLOR: Color32 = Color32::from_rgb(46, 46, 46);
const PRIMARY_TEXT: Color32 = Color32::from_rgb(245, 245, 245);
const MUTED_TEXT: Color32 = Color32::from_rgb(168, 168, 168);
const BUTTON_TEXT: Color32 = Color32::from_rgb(12, 12, 12);
const BUTTON_BACKGROUND: Color32 = Color32::from_rgb(250, 250, 250);
const BUTTON_BACKGROUND_DISABLED: Color32 = Color32::from_rgb(92, 92, 92);
const ERROR_TEXT: Color32 = Color32::from_rgb(255, 126, 126);

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
    fn apply_theme(ui: &mut egui::Ui) {
        let visuals = ui.visuals_mut();
        visuals.override_text_color = Some(PRIMARY_TEXT);
        visuals.panel_fill = PAGE_BACKGROUND;
        visuals.extreme_bg_color = INPUT_BACKGROUND;
        visuals.faint_bg_color = CARD_BACKGROUND;
        visuals.code_bg_color = INPUT_BACKGROUND;
        visuals.window_fill = CARD_BACKGROUND;
        visuals.widgets.noninteractive.bg_fill = CARD_BACKGROUND;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_COLOR);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, PRIMARY_TEXT);
        visuals.widgets.inactive.bg_fill = INPUT_BACKGROUND;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_COLOR);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, PRIMARY_TEXT);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 42, 42);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(70, 70, 70));
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, PRIMARY_TEXT);
        visuals.widgets.active.bg_fill = Color32::from_rgb(48, 48, 48);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(92, 92, 92));
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, PRIMARY_TEXT);
        visuals.selection.bg_fill = Color32::from_rgb(84, 84, 84);
        visuals.selection.stroke = Stroke::new(1.0, PRIMARY_TEXT);
        visuals.hyperlink_color = PRIMARY_TEXT;

        let spacing = ui.spacing_mut();
        spacing.item_spacing = Vec2::new(12.0, 12.0);
        spacing.button_padding = Vec2::new(20.0, 14.0);
        spacing.interact_size.y = 48.0;
        spacing.text_edit_width = 320.0;
    }

    fn sanitize_tin_input(tin_string: &mut String) {
        let sanitized = tin_string
            .chars()
            .filter(char::is_ascii_digit)
            .take(12)
            .collect::<String>();

        if sanitized != *tin_string {
            *tin_string = sanitized;
        }
    }

    fn is_valid_tin_input(tin_string: &str) -> bool {
        matches!(tin_string.len(), 10 | 12) && tin_string.chars().all(|ch| ch.is_ascii_digit())
    }

    pub fn advance(&mut self) {
        match self {
            Self::Landing {
                tin: tin_string,
                error: error_string,
                clicked,
            } => {
                Self::sanitize_tin_input(tin_string);

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

    #[expect(clippy::too_many_lines)]
    pub fn main_ui(&mut self, ui: &mut egui::Ui) {
        Self::apply_theme(ui);

        let body_text_size = ui.text_style_height(&TextStyle::Body);
        match self {
            Self::Landing {
                tin: tin_string,
                error: error_string,
                clicked,
            } => {
                let page_width = ui.available_width().min(980.0);
                let top_padding = (ui.available_height() * 0.18).clamp(56.0, 180.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(top_padding);

                    ui.allocate_ui_with_layout(
                        Vec2::new(page_width, 0.0),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.label(
                                RichText::new("InfraTrace")
                                    .size(body_text_size - 1.0)
                                    .color(MUTED_TEXT)
                                    .strong(),
                            );

                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("Поиск и визуализация\nинфраструктуры компаний\nпо ИНН")
                                    .size(body_text_size + 24.0)
                                    .color(PRIMARY_TEXT)
                                    .strong(),
                            );

                            ui.add_space(14.0);
                            ui.label(
                                RichText::new(
                                    "Введите ИНН, чтобы получить карточку компании, связанные домены и сетевую инфраструктуру в одном отчёте.",
                                )
                                .size(body_text_size + 3.0)
                                .color(MUTED_TEXT),
                            );

                            ui.add_space(32.0);

                            Frame::new()
                                .fill(CARD_BACKGROUND)
                                .stroke(Stroke::new(1.0, BORDER_COLOR))
                                .corner_radius(CornerRadius::same(20))
                                .inner_margin(Margin::same(24))
                                .show(ui, |ui| {
                                    let wide_layout = ui.available_width() >= 560.0;
                                    let is_valid = Self::is_valid_tin_input(tin_string);

                                    let submit_button = |enabled: bool| {
                                        Button::new(
                                            RichText::new("Найти")
                                                .size(body_text_size + 2.0)
                                                .color(if enabled { BUTTON_TEXT } else { PRIMARY_TEXT })
                                                .strong(),
                                        )
                                        .fill(if enabled {
                                            BUTTON_BACKGROUND
                                        } else {
                                            BUTTON_BACKGROUND_DISABLED
                                        })
                                        .stroke(Stroke::NONE)
                                        .corner_radius(CornerRadius::same(12))
                                        .min_size(Vec2::new(140.0, 54.0))
                                    };

                                    if wide_layout {
                                        ui.horizontal(|ui| {
                                            let input_width = (ui.available_width() - 152.0).max(220.0);
                                            let input_response = ui.add_sized(
                                                Vec2::new(input_width, 54.0),
                                                TextEdit::singleline(tin_string)
                                                    .hint_text("Введите ИНН")
                                                    .font(TextStyle::Heading)
                                                    .desired_width(f32::INFINITY),
                                            );
                                            let clicked_now = ui.add_enabled(is_valid, submit_button(is_valid)).clicked();
                                            let enter_pressed = input_response.lost_focus()
                                                && ui.input(|input| input.key_pressed(Key::Enter));

                                            *clicked = clicked_now || (enter_pressed && is_valid);
                                        });
                                    } else {
                                        let input_response = ui.add_sized(
                                            Vec2::new(ui.available_width(), 54.0),
                                            TextEdit::singleline(tin_string)
                                                .hint_text("Введите ИНН")
                                                .font(TextStyle::Heading)
                                                .desired_width(f32::INFINITY),
                                        );
                                        let enter_pressed = input_response.lost_focus()
                                            && ui.input(|input| input.key_pressed(Key::Enter));
                                        let clicked_now = ui
                                            .add_enabled_ui(is_valid, |ui| {
                                                ui.add_sized(
                                                    Vec2::new(ui.available_width(), 54.0),
                                                    submit_button(is_valid),
                                                )
                                            })
                                            .inner
                                            .clicked();

                                        *clicked = is_valid && (clicked_now || enter_pressed);
                                    }

                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("ИНН: 10 или 12 цифр")
                                            .size(body_text_size - 1.0)
                                            .color(MUTED_TEXT),
                                    );

                                    if let Some(error) = error_string {
                                        ui.add_space(2.0);
                                        ui.label(
                                            RichText::new(error.as_str())
                                                .size(body_text_size)
                                                .color(ERROR_TEXT),
                                        );
                                    }
                                });
                        },
                    );
                });
            }

            Self::Query {
                tin,
                report,
                report_queried: _,
                back_clicked,
            } => {
                let page_width = ui.available_width().min(980.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);

                    Frame::new()
                        .fill(CARD_BACKGROUND)
                        .stroke(Stroke::new(1.0, BORDER_COLOR))
                        .corner_radius(CornerRadius::same(20))
                        .inner_margin(Margin::same(24))
                        .show(ui, |ui| {
                            ui.set_max_width(page_width);

                            ui.horizontal(|ui| {
                                if report.is_none() {
                                    ui.spinner();
                                }

                                ui.label(
                                    RichText::new(format!("Поиск по ИНН: {tin}"))
                                        .size(body_text_size + 4.0)
                                        .strong(),
                                );

                                if report.is_some() {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        *back_clicked = ui
                                            .add(
                                                Button::new(RichText::new("Назад").strong())
                                                    .corner_radius(CornerRadius::same(10)),
                                            )
                                            .clicked();
                                    });
                                }
                            });

                            ui.add_space(20.0);

                            if let Some(TINReport {
                                tin: _,
                                name,
                                domains,
                                ip_addrs,
                            }) = report
                            {
                                ui.label(
                                    RichText::new(format!("Название: {name}"))
                                        .monospace()
                                        .color(PRIMARY_TEXT),
                                );
                                ui.label(
                                    RichText::new(format!("Домены и поддомены: {domains:?}"))
                                        .monospace()
                                        .color(MUTED_TEXT),
                                );
                                ui.label(
                                    RichText::new(format!("IP-адреса: {ip_addrs:?}"))
                                        .monospace()
                                        .color(MUTED_TEXT),
                                );
                            }
                        });
                });
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.advance();

        egui::CentralPanel::default()
            .frame(Frame::new().fill(PAGE_BACKGROUND))
            .show_inside(ui, |ui| self.main_ui(ui));
    }
}
