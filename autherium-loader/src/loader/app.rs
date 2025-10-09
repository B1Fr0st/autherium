use core::f32;
use eframe::egui::{
    self, Color32, ImageButton, RichText, Stroke, Style, Vec2, include_image, vec2,
};
use std::sync::mpsc;

#[derive(Default, Debug)]
pub(crate) struct MyApp {
    pub ui_state: UiState,
    pub frame: u64,
    pub load: bool,
    pub license: String,
    pub failed_reason: String,
    pub license_timing: (u64, u64),
    // Channel for async license verification
    pub license_receiver: Option<mpsc::Receiver<LicenseResult>>,
    pub autherium_url: String,
    pub product_id: String,
    pub discord_url: String,
    pub website_url: String,
}
#[derive(Default, PartialEq, Debug)]
pub enum UiState {
    Verifying,
    #[default]
    LicenseInput,
    Verified,
    Error,
}

// Result type for license verification
#[derive(Debug, Clone)]
pub enum LicenseResult {
    Success(u64, u64),
    Error(String),
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        // Check for async license verification result
        self.check_license_result();
        self.frame += 1;

        let mut visuals = egui::Visuals::default();

        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 0],
            blur: 0,
            spread: 0,
            color: Color32::BLACK,
        };

        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 0],
            blur: 0,
            spread: 0,
            color: Color32::BLACK,
        };

        // visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        // visuals.widgets.hovered.fg_stroke = Stroke::NONE;
        // visuals.widgets.active.bg_stroke = Stroke::NONE;
        // visuals.widgets.active.fg_stroke = Stroke::NONE;
        // visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        // visuals.widgets.inactive.fg_stroke = Stroke::NONE;

        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(110, 0, 0);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(32, 16, 16);

        visuals.extreme_bg_color = Color32::from_rgba_unmultiplied(255, 255, 255, 25);

        visuals.panel_fill = Color32::BLACK;

        visuals.window_stroke = egui::Stroke {
            width: 0.5,
            color: Color32::from_rgb(54, 1, 63),
        };

        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(
                        Color32::BLACK
                            .lerp_to_gamma(Color32::from_rgba_unmultiplied(0, 0, 0, 125), 0.5),
                    )
                    .corner_radius(15.0)
                    .stroke(egui::Stroke {
                        width: 1.0,
                        color: Color32::BLACK,
                    }),
            )
            .show(ctx, |ui| {
                match self.ui_state {
                    UiState::Verifying => {
                        ctx.style_mut(|s| {
                            s.spacing.item_spacing = vec2(16.0, 64.0);
                            s.spacing.indent = 16.0;
                        });
                        ui.vertical_centered(|ui| {
                            ui.add_space(90.0);
                            ui.add(egui::Spinner::new().size(50.0).color(Color32::GRAY));
                        });
                    }
                    UiState::LicenseInput => {
                        ctx.style_mut(|s| {
                            let mut style = Style::default();
                            let mut visuals = egui::Visuals::default();
                            style.visuals = visuals;
                            *s = style;
                        });
                        if self.failed_reason.is_empty() {
                            if let Ok(license) = std::fs::read_to_string("license.key") {
                                self.license = license;
                            }
                        }
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_space(480.0);
                            let exit = ui.add_sized(
                                [25.0, 25.0],
                                ImageButton::new(include_image!("../../assets/exit.png"))
                                    .frame(false),
                            );
                            if exit.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if exit.clicked() {
                                self.load = false;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });

                        if self.license_regex() {
                            self.verify_license_async();
                            self.ui_state = UiState::Verifying;
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(5.0);
                                if ui
                                    .label(RichText::new("thrum").size(24.0).color(Color32::GRAY))
                                    .hovered()
                                {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::default()); //remove edit cursor on title
                                };
                                ui.add_space(10.0);
                                let text_edit = egui::TextEdit::singleline(&mut self.license);
                                ui.add(text_edit);
                                self.license = self.license.trim().to_string();
                                if !self.license.is_empty() && !self.license_regex() {
                                    self.failed_reason =
                                        "License not in correct format!".to_string();
                                }
                                if self.license.is_empty()
                                    && self.failed_reason == "License not in correct format!"
                                {
                                    self.failed_reason.clear();
                                }
                                ui.add_space(35.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(175.0);
                                    let discord = ui.add_sized(
                                        [50.0, 50.0],
                                        ImageButton::new(include_image!(
                                            "../../assets/discord.png"
                                        ))
                                        .frame(false),
                                    );
                                    if discord.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if discord.clicked() {
                                        ctx.open_url(egui::OpenUrl::new_tab(
                                            self.discord_url.clone(),
                                        ));
                                    }
                                    ui.add_space(50.0);
                                    let website = ui.add_sized(
                                        [50.0, 50.0],
                                        ImageButton::new(include_image!(
                                            "../../assets/internet.png"
                                        ))
                                        .frame(false),
                                    );
                                    if website.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if website.clicked() {
                                        ctx.open_url(egui::OpenUrl::new_tab(
                                            self.website_url.clone(),
                                        ));
                                    }
                                });
                            });
                        }

                        if !self.failed_reason.is_empty() {
                            ui.label(
                                RichText::new(format!("{}", self.failed_reason))
                                    .size(16.0)
                                    .color(Color32::LIGHT_RED),
                            );
                        }
                    }
                    UiState::Verified => {
                        ctx.style_mut(|s| {
                            let mut style = Style::default();
                            style.visuals.extreme_bg_color =
                                Color32::from_rgba_unmultiplied(0, 0, 0, 0);
                            *s = style
                        });
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_space(480.0);
                            let exit = ui.add_sized(
                                [25.0, 25.0],
                                ImageButton::new(include_image!("../../assets/exit.png"))
                                    .frame(false),
                            );
                            if exit.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if exit.clicked() {
                                self.load = false;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                        ui.vertical_centered(|ui| {
                            let time_remaining =
                                (((self.license_timing.0 + self.license_timing.1) as i64
                                    - std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs() as i64) as f32)
                                    .max(0.0);
                            ui.label(
                                RichText::new(format!(
                                    "Time remaining: {} days {} hours {} minutes",
                                    (time_remaining / 60.0 / 60.0 / 24.0).floor(),
                                    ((time_remaining / 60.0 / 60.0) % 24.0).floor(),
                                    ((time_remaining / 60.0) % 60.0).floor(),
                                ))
                                .color(Color32::GRAY)
                                .size(20.0),
                            );
                            ui.add_space(25.0);
                            ui.add(
                                egui::widgets::ProgressBar::new(
                                    time_remaining / self.license_timing.1 as f32,
                                )
                                .corner_radius(0.0)
                                .desired_width(350.0)
                                .fill(Color32::GRAY),
                            );
                            ui.add_space(25.0);
                            if ui
                                .add(
                                    egui::widgets::Button::new(RichText::new("Load").size(50.0))
                                        .min_size(Vec2::new(100.0, 50.0))
                                        .fill(Color32::TRANSPARENT)
                                        .stroke((1.0, Color32::GRAY)),
                                )
                                .clicked()
                            {
                                self.load = true;
                                if let Err(e) = std::fs::write("license.key", &self.license) {
                                    eprintln!("Failed to write license.key: {}", e);
                                }
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                        ctx.request_repaint();
                    }
                    UiState::Error => {
                        ctx.style_mut(|s| *s = Style::default());
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label(
                                RichText::new(self.failed_reason.clone())
                                    .size(24.0)
                                    .color(Color32::LIGHT_RED),
                            );
                            ui.add_space(100.0);
                            if ui.button("Exit").clicked() {
                                self.load = false;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    }
                }
            });

        if ctx.input(|i| i.viewport().close_requested()) && !self.load {
            std::process::exit(0);
        }
    }
}
