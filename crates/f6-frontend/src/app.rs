use std::net::IpAddr;
use std::sync::Mutex;

use egui::{
    Align, Button, Color32, CornerRadius, Frame, Key, Layout, Margin, RichText, ScrollArea, Stroke,
    TextEdit, TextStyle, Vec2,
};
use f6_types::LegalEntityTIN;
use f6_types::report::{InfrastructureGroup, InfrastructureKind, TINReport};

const PAGE_BACKGROUND: Color32 = Color32::from_rgb(15, 15, 15);
const CARD_BACKGROUND: Color32 = Color32::from_rgb(23, 23, 23);
const INPUT_BACKGROUND: Color32 = Color32::from_rgb(36, 36, 36);
const MUTED_PANEL: Color32 = Color32::from_rgb(28, 28, 28);
const CHIP_BACKGROUND: Color32 = Color32::from_rgb(31, 31, 31);
const BORDER_COLOR: Color32 = Color32::from_rgb(46, 46, 46);
const CHIP_BORDER: Color32 = Color32::from_rgb(58, 58, 58);
const PRIMARY_TEXT: Color32 = Color32::from_rgb(245, 245, 245);
const MUTED_TEXT: Color32 = Color32::from_rgb(174, 174, 174);
const BUTTON_TEXT: Color32 = Color32::from_rgb(12, 12, 12);
const BUTTON_BACKGROUND: Color32 = Color32::from_rgb(250, 250, 250);
const BUTTON_BACKGROUND_DISABLED: Color32 = Color32::from_rgb(92, 92, 92);
const ERROR_TEXT: Color32 = Color32::from_rgb(255, 126, 126);
const OWN_BADGE: Color32 = Color32::from_rgb(34, 84, 61);
const HOSTING_BADGE: Color32 = Color32::from_rgb(49, 63, 111);
const UNKNOWN_BADGE: Color32 = Color32::from_rgb(71, 71, 71);
const PAGE_MAX_WIDTH: f32 = 1040.0;

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
        error: Option<String>,
        report_queried: bool,
        back_clicked: bool,
    },
}

static REPORT_BUFFER: Mutex<Option<Result<TINReport, String>>> = Mutex::new(None);

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
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(43, 43, 43);
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

    fn sanitize_tin_input(tin: &mut String) {
        let sanitized = tin
            .chars()
            .filter(char::is_ascii_digit)
            .take(10)
            .collect::<String>();

        if sanitized != *tin {
            *tin = sanitized;
        }
    }

    fn is_valid_tin_input(tin: &str) -> bool {
        tin.len() == 10 && tin.chars().all(|ch| ch.is_ascii_digit())
    }

    fn primary_button(text: &str, enabled: bool, body_text_size: f32) -> Button<'_> {
        Button::new(
            RichText::new(text)
                .size(body_text_size + 1.0)
                .color(if enabled { BUTTON_TEXT } else { PRIMARY_TEXT })
                .strong(),
        )
        .fill(if enabled {
            BUTTON_BACKGROUND
        } else {
            BUTTON_BACKGROUND_DISABLED
        })
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(140.0, 54.0))
    }

    fn card_frame() -> Frame {
        Frame::new()
            .fill(CARD_BACKGROUND)
            .stroke(Stroke::new(1.0, BORDER_COLOR))
            .corner_radius(CornerRadius::same(16))
            .inner_margin(Margin::same(24))
    }

    fn section_frame() -> Frame {
        Frame::new()
            .fill(CARD_BACKGROUND)
            .stroke(Stroke::new(1.0, BORDER_COLOR))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::same(18))
    }

    fn subtle_frame() -> Frame {
        Frame::new()
            .fill(MUTED_PANEL)
            .stroke(Stroke::new(1.0, BORDER_COLOR))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(16))
    }

    fn chip(ui: &mut egui::Ui, text: impl Into<String>, color: Color32, monospace: bool) {
        let mut rich_text = RichText::new(text.into()).color(color);
        if monospace {
            rich_text = rich_text.monospace();
        }

        Frame::new()
            .fill(CHIP_BACKGROUND)
            .stroke(Stroke::new(1.0, CHIP_BORDER))
            .corner_radius(CornerRadius::same(255))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                if monospace {
                    ui.set_min_width(112.0);
                }
                ui.label(rich_text);
            });
    }

    fn domain_row(ui: &mut egui::Ui, domain: &str) {
        Frame::new()
            .fill(CHIP_BACKGROUND)
            .stroke(Stroke::new(1.0, CHIP_BORDER))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(domain).color(PRIMARY_TEXT));
            });
    }

    fn badge(ui: &mut egui::Ui, kind: &InfrastructureKind) {
        let (label, color) = match kind {
            InfrastructureKind::Own => ("Собственная", OWN_BADGE),
            InfrastructureKind::Hosting => ("Хостинг / облако", HOSTING_BADGE),
            InfrastructureKind::Unknown => ("Неизвестно", UNKNOWN_BADGE),
        };

        Frame::new()
            .fill(color)
            .stroke(Stroke::new(1.0, Color32::from_white_alpha(28)))
            .corner_radius(CornerRadius::same(255))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.label(RichText::new(label).color(PRIMARY_TEXT).strong());
            });
    }

    fn section_header(ui: &mut egui::Ui, title: &str, count: usize, body_text_size: f32) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(title)
                    .size(body_text_size + 1.0)
                    .color(PRIMARY_TEXT)
                    .strong(),
            );
            ui.label(
                RichText::new(count.to_string())
                    .size(body_text_size - 1.0)
                    .color(MUTED_TEXT),
            );
        });
    }

    fn stat_card(ui: &mut egui::Ui, label: &str, value: impl ToString, body_text_size: f32) {
        Self::subtle_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(value.to_string())
                    .size(body_text_size + 12.0)
                    .strong()
                    .color(PRIMARY_TEXT),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(label)
                    .size(body_text_size - 1.0)
                    .color(MUTED_TEXT),
            );
        });
    }

    fn metadata_row(ui: &mut egui::Ui, label: &str, value: Option<&String>) {
        let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
            return;
        };

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(label).color(MUTED_TEXT));
            ui.label(RichText::new(value).color(PRIMARY_TEXT));
        });
    }

    fn infrastructure_group_card(
        ui: &mut egui::Ui,
        group: &InfrastructureGroup,
        body_text_size: f32,
    ) {
        Self::subtle_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                let title = match (group.asn, group.prefix.as_deref()) {
                    (Some(asn), Some(prefix)) => format!("AS{asn} / {prefix}"),
                    (Some(asn), None) => format!("AS{asn}"),
                    (None, Some(prefix)) => prefix.to_owned(),
                    (None, None) => "Сеть без ASN".to_owned(),
                };

                ui.label(
                    RichText::new(title)
                        .size(body_text_size + 4.0)
                        .strong()
                        .color(PRIMARY_TEXT),
                );
                Self::badge(ui, &group.kind);
            });

            if let Some(summary) = group.netname.as_ref().or(group.as_holder.as_ref()) {
                ui.label(
                    RichText::new(summary)
                        .size(body_text_size)
                        .color(MUTED_TEXT),
                );
            }

            ui.add_space(10.0);
            Self::metadata_row(ui, "AS holder", group.as_holder.as_ref());
            Self::metadata_row(ui, "Netname", group.netname.as_ref());
            Self::metadata_row(ui, "Описание", group.description.as_ref());
            Self::metadata_row(ui, "Страна", group.country.as_ref());
            Self::metadata_row(ui, "Maintainer", group.maintainer.as_ref());

            ui.add_space(8.0);
            ui.label(RichText::new(&group.reason).color(MUTED_TEXT));

            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                for ip_addr in &group.ip_addrs {
                    Self::chip(ui, ip_addr.to_string(), PRIMARY_TEXT, true);
                }
            });
        });
    }

    fn domain_section(ui: &mut egui::Ui, domains: &[String], body_text_size: f32) {
        Self::section_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            Self::section_header(ui, "Домены", domains.len(), body_text_size);
            ui.add_space(10.0);

            if domains.is_empty() {
                ui.label(RichText::new("Домены не найдены").color(MUTED_TEXT));
                return;
            }

            for (index, domain) in domains.iter().enumerate() {
                Self::domain_row(ui, domain);
                if index + 1 != domains.len() {
                    ui.add_space(8.0);
                }
            }
        });
    }

    fn ip_section(ui: &mut egui::Ui, ip_addrs: &[IpAddr], body_text_size: f32) {
        Self::section_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            Self::section_header(ui, "Все найденные IP", ip_addrs.len(), body_text_size);
            ui.add_space(10.0);

            if ip_addrs.is_empty() {
                ui.label(RichText::new("IP-адреса не найдены").color(MUTED_TEXT));
                return;
            }

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                for ip_addr in ip_addrs {
                    Self::chip(ui, ip_addr.to_string(), PRIMARY_TEXT, true);
                }
            });
        });
    }

    fn landing_ui(
        ui: &mut egui::Ui,
        tin_string: &mut String,
        error_string: &mut Option<String>,
        clicked: &mut bool,
        body_text_size: f32,
    ) {
        let page_width = ui.available_width().min(980.0);
        let top_padding = (ui.available_height() * 0.18).clamp(48.0, 170.0);

        ui.vertical_centered(|ui| {
            ui.add_space(top_padding);
            ui.allocate_ui_with_layout(Vec2::new(page_width, 0.0), Layout::top_down(Align::Min), |ui| {
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
                    .size(body_text_size + 2.0)
                    .color(MUTED_TEXT),
                );

                ui.add_space(32.0);
                Self::card_frame().show(ui, |ui| {
                    let wide_layout = ui.available_width() >= 560.0;
                    let is_valid = Self::is_valid_tin_input(tin_string);

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
                            let clicked_now = ui
                                .add_enabled(
                                    is_valid,
                                    Self::primary_button("Найти", is_valid, body_text_size),
                                )
                                .clicked();
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
                                    Self::primary_button("Найти", is_valid, body_text_size),
                                )
                            })
                            .inner
                            .clicked();

                        *clicked = is_valid && (clicked_now || enter_pressed);
                    }

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("ИНН организации: 10 цифр")
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
            });
        });
    }

    fn report_ui(
        ui: &mut egui::Ui,
        tin: LegalEntityTIN,
        report: Option<&TINReport>,
        error: Option<&String>,
        back_clicked: &mut bool,
        body_text_size: f32,
    ) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(28.0);
                ui.vertical_centered(|ui| {
                    ui.set_max_width(PAGE_MAX_WIDTH);

                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ui.set_width(ui.available_width().min(PAGE_MAX_WIDTH));

                        *back_clicked = ui
                            .add(
                                Button::new(RichText::new("Назад").strong())
                                    .corner_radius(CornerRadius::same(8)),
                            )
                            .clicked();

                        ui.add_space(12.0);
                        Self::card_frame().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new(format!("Отчёт по ИНН {tin}"))
                                    .size(body_text_size - 1.0)
                                    .color(MUTED_TEXT),
                            );

                            let title = report.map_or("Загрузка отчёта", |report| report.name.as_str());
                            ui.label(
                                RichText::new(title)
                                    .size(body_text_size + 14.0)
                                    .strong()
                                    .color(PRIMARY_TEXT),
                            );
                        });

                        if let Some(error) = error {
                            ui.add_space(16.0);
                            Self::subtle_frame().show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new("Не удалось получить отчёт")
                                        .size(body_text_size + 3.0)
                                        .strong()
                                        .color(ERROR_TEXT),
                                );
                                ui.label(RichText::new(error).color(MUTED_TEXT));
                            });
                            return;
                        }

                        let Some(report) = report else {
                            ui.add_space(16.0);
                            Self::subtle_frame().show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new("Ищем инфраструктуру")
                                                .size(body_text_size + 3.0)
                                                .strong()
                                                .color(PRIMARY_TEXT),
                                        );
                                        ui.label(
                                            RichText::new(
                                                "Получаем домены, резолвим IP и группируем сети через RIPEstat.",
                                            )
                                            .color(MUTED_TEXT),
                                        );
                                    });
                                });
                            });
                            return;
                        };

                        ui.add_space(16.0);
                        Self::report_contents(ui, report, body_text_size);
                        ui.add_space(28.0);
                    });
                });
            });
    }

    fn report_contents(ui: &mut egui::Ui, report: &TINReport, body_text_size: f32) {
        let mut domains = report.domains.iter().cloned().collect::<Vec<_>>();
        domains.sort_unstable();

        let mut ip_addrs = report.ip_addrs.iter().copied().collect::<Vec<_>>();
        ip_addrs.sort_unstable();

        let mut groups = report.infrastructure_groups.clone();
        groups.sort_by(|left, right| {
            (left.asn, left.prefix.as_deref()).cmp(&(right.asn, right.prefix.as_deref()))
        });

        let own_count = groups
            .iter()
            .filter(|group| group.kind == InfrastructureKind::Own)
            .count();
        let hosting_count = groups
            .iter()
            .filter(|group| group.kind == InfrastructureKind::Hosting)
            .count();
        let unknown_count = groups
            .iter()
            .filter(|group| group.kind == InfrastructureKind::Unknown)
            .count();

        let summary_label = "Хостинги / Собственные / Неизвестно";
        let summary_value = format!("{hosting_count} / {own_count} / {unknown_count}");

        Self::stat_card(ui, "Домены", domains.len(), body_text_size);
        Self::stat_card(ui, "IP-адреса", ip_addrs.len(), body_text_size);
        Self::stat_card(ui, "Группы", groups.len(), body_text_size);
        Self::stat_card(ui, summary_label, summary_value.as_str(), body_text_size);

        ui.add_space(18.0);
        Self::section_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            Self::section_header(ui, "Инфраструктура", groups.len(), body_text_size);
            ui.add_space(10.0);

            if groups.is_empty() {
                ui.label(
                    RichText::new("Не удалось сгруппировать IP-инфраструктуру").color(MUTED_TEXT),
                );
            } else {
                for (index, group) in groups.iter().enumerate() {
                    Self::infrastructure_group_card(ui, group, body_text_size);
                    if index + 1 != groups.len() {
                        ui.add_space(10.0);
                    }
                }
            }
        });

        ui.add_space(14.0);
        Self::domain_section(ui, &domains, body_text_size);
        ui.add_space(14.0);
        Self::ip_section(ui, &ip_addrs, body_text_size);
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
                            *REPORT_BUFFER.lock().unwrap() = None;
                            *self = Self::Query {
                                tin,
                                report: None,
                                error: None,
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
                error,
                report_queried,
                back_clicked,
            } => {
                if !*report_queried {
                    debug_assert!(report.is_none());
                    debug_assert!(error.is_none());
                    debug_assert!(REPORT_BUFFER.lock().unwrap().is_none());

                    let tin = *tin;
                    let future = async move {
                        let result = match reqwest::get(format!(
                            "http://localhost:8080/report/{tin}"
                        ))
                        .await
                        {
                            Ok(response) => response.json::<TINReport>().await.map_err(|error| {
                                format!("Не удалось прочитать ответ backend: {error}")
                            }),
                            Err(error) => Err(format!(
                                "Backend недоступен на localhost:8080 или вернул сетевую ошибку: {error}"
                            )),
                        };
                        *REPORT_BUFFER.lock().unwrap() = Some(result);

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
                if let Some(result) = lock.take() {
                    match result {
                        Ok(received_report) => *report = Some(received_report),
                        Err(message) => *error = Some(message),
                    }
                }
                drop(lock);

                if *back_clicked {
                    *REPORT_BUFFER.lock().unwrap() = None;
                    *self = Self::default();
                }
            }
        }
    }

    pub fn main_ui(&mut self, ui: &mut egui::Ui) {
        Self::apply_theme(ui);
        let body_text_size = ui.text_style_height(&TextStyle::Body);

        match self {
            Self::Landing {
                tin: tin_string,
                error: error_string,
                clicked,
            } => Self::landing_ui(ui, tin_string, error_string, clicked, body_text_size),
            Self::Query {
                tin,
                report,
                error,
                report_queried: _,
                back_clicked,
            } => Self::report_ui(
                ui,
                *tin,
                report.as_ref(),
                error.as_ref(),
                back_clicked,
                body_text_size,
            ),
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
