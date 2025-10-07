use eframe::egui::{self, include_image, vec2, Color32, ImageButton, Pos2, Rect, RichText, Stroke, Style};
use core::f32;
use std::sync::mpsc;


#[derive(Default, Debug)]
pub(crate) struct MyApp {
    pub ui_state: UiState,
    pub frame: u64,
    pub load: bool,
    pub license: String,
    pub failed_reason: String,
    pub license_timing: (u64,u64),
    // Channel for async license verification
    pub license_receiver: Option<mpsc::Receiver<LicenseResult>>,
    pub autherium_url: String,
}
#[derive(Default, PartialEq, Debug)]
pub enum UiState{
    Verifying,
    #[default]
    LicenseInput,
    Verified,
    Error
}

// Result type for license verification
#[derive(Debug, Clone)]
pub enum LicenseResult {
    Success(u64,u64),
    Error(String),
}

fn hsv_to_color32(h: f32, s: f32, v: f32) -> Color32 {
    let c = v * s; // Chroma
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;
    
    let (r_prime, g_prime, b_prime) = if h_prime >= 0.0 && h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime >= 1.0 && h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime >= 2.0 && h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime >= 3.0 && h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime >= 4.0 && h_prime < 5.0 {
        (x, 0.0, c)
    } else if h_prime >= 5.0 && h_prime < 6.0 {
        (c, 0.0, x)
    } else {
        (0.0, 0.0, 0.0) // Fallback for invalid hue
    };
    
    // Convert to 0-255 range
    let r = ((r_prime + m) * 255.0).round() as u8;
    let g = ((g_prime + m) * 255.0).round() as u8;
    let b = ((b_prime + m) * 255.0).round() as u8;
    
    Color32::from_rgb(r, g, b)
}

impl MyApp {
    fn color_cycle(&self) -> Color32{
        hsv_to_color32(self.frame as f32 % 360.0,1.0,1.0)
    }
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
            offset: [0,0],
            blur: 0,
            spread: 0,
            color: Color32::BLACK,
        };

        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0,0],
            blur: 0,
            spread: 0,
            color: Color32::BLACK,
        };

        visuals.widgets.hovered.bg_stroke = Stroke::new(0.1,Color32::BLACK);
        visuals.widgets.active.bg_stroke = Stroke::new(0.1,Color32::BLACK);
        visuals.widgets.inactive.bg_stroke = Stroke::new(0.1,Color32::BLACK);

        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(110,0,0);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(32, 16, 16);

        
        visuals.extreme_bg_color = Color32::BLACK;

        visuals.panel_fill = Color32::BLACK;

        visuals.window_stroke = egui::Stroke{width:0.5, color:Color32::from_rgb(54,1,63)};

        ctx.set_visuals(visuals);
        
        

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                .fill(Color32::BLACK.lerp_to_gamma(Color32::from_rgba_unmultiplied(0, 0, 0, 125),0.5))
                .corner_radius(15.0)
                .stroke(egui::Stroke{width:1.0, color:Color32::BLACK})
            )
            .show(ctx, |ui| {
            match self.ui_state{
                UiState::Verifying => {
                    ctx.style_mut(|s|{s.spacing.item_spacing = vec2(16.0, 64.0);s.spacing.indent=16.0;});
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Verifying license...").size(16.0).color(Color32::LIGHT_BLUE));
                        ui.add(egui::Spinner::new().size(50.0).color(self.color_cycle()));
                    });
                },
                UiState::LicenseInput => {
                    ctx.style_mut(|s|{*s = Style::default()});
                    if self.failed_reason.is_empty(){
                        if let Ok(license) = std::fs::read_to_string("license.txt"){
                            self.license = license;
                        }
                    }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space(480.0);
                        let exit = ui.add_sized([25.0,25.0],ImageButton::new(include_image!("../../assets/exit.png")).frame(false));
                        if exit.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if exit.clicked(){
                            self.load = false;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    
                    if self.license_regex() {
                            self.verify_license_async();
                            self.ui_state = UiState::Verifying;
                        }
                    else {
                        ui.vertical_centered(|ui|{
                            ui.add_space(5.0);
                            if ui.label(RichText::new("License Key").size(24.0).color(Color32::LIGHT_BLUE)).hovered(){
                                ui.ctx().set_cursor_icon(egui::CursorIcon::default()); //remove edit cursor on title
                            };
                            ui.add_space(10.0);
                            let text_edit = egui::TextEdit::singleline(&mut self.license);
                            ui.add(text_edit);
                            self.license = self.license.trim().to_string();
                            if !self.license.is_empty() && !self.license_regex() {
                                self.failed_reason = String::new();
                                ui.label(RichText::new("License not in correct format!").size(16.0).color(Color32::LIGHT_RED));
                            }
                            ui.add_space(35.0);
                            ui.horizontal(|ui| {
                                ui.add_space(175.0);
                                let discord = ui.add_sized([50.0,50.0],ImageButton::new(include_image!("../../assets/discord.png")).frame(false));
                                if discord.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if discord.clicked(){
                                    ctx.open_url(egui::OpenUrl::new_tab( "https://google.com"));
                                }
                                ui.add_space(50.0);
                                let website = ui.add_sized([50.0,50.0],ImageButton::new(include_image!("../../assets/internet.png")).frame(false));
                                if website.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if website.clicked(){
                                    ctx.open_url(egui::OpenUrl::new_tab( "https://google.com"));
                                }
                            });

                        });
                    }
                    
                    if !self.failed_reason.is_empty() {
                        ui.label(RichText::new(format!("Failed: {}", self.failed_reason)).size(16.0).color(Color32::LIGHT_RED));
                    }
                },
                UiState::Verified => {
                    ctx.style_mut(|s|{*s = Style::default()});
                    if ui.button("Load").clicked() {
                        self.load = true;
                        if let Err(e) = std::fs::write("license.txt", &self.license) {
                            eprintln!("Failed to write license.txt: {}", e);
                        }
                        let ctx = ui.ctx().clone();
                        std::thread::spawn(move || {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        });
                    }
                    if ui.button("Exit").clicked() {
                        self.load = false;
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    let time_remaining = (((self.license_timing.0 + self.license_timing.1) as i64 - std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64) as f32).max(0.0);
                    ui.label(format!("Time remaining: {} days {} hours {} minutes",
                        (time_remaining / 60.0 / 60.0 / 24.0).floor(),
                        ((time_remaining / 60.0 / 60.0)%24.0).floor(),
                        ((time_remaining / 60.0 )%60.0).floor(),
                    ));
                    ui.add(egui::widgets::ProgressBar::new(time_remaining / self.license_timing.1 as f32 )
                    .text(RichText::new(format!("Time remaining: {} days {} hours {} minutes",
                        (time_remaining / 60.0 / 60.0 / 24.0).floor(),
                        ((time_remaining / 60.0 / 60.0)%24.0).floor(),
                        ((time_remaining / 60.0 )%60.0).floor(),
                    )).color(Color32::BLACK))
                    .corner_radius(0.0)
                    .fill(Color32::WHITE)
                );
                    ctx.request_repaint();
                },
                UiState::Error => {
                    ctx.style_mut(|s|{*s = Style::default()});
                    ui.vertical_centered(|ui|{
                        ui.add_space(50.0);
                        ui.label(RichText::new(self.failed_reason.clone()).size(24.0).color(Color32::DARK_RED));
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