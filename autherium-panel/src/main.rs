use eframe::egui;
#[derive(Default)]
struct MyApp {
    autherium_url: String,
    days: String,
    alert: String,
    product_id: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("ban self hwid").clicked() {
                let autherium = autherium_rs::Autherium::new(&self.autherium_url.clone()).unwrap();
                match autherium.ban_hwid(
                    &autherium_rs::Autherium::get_hwid().unwrap().into(),
                    &"super_secret_key".to_string(),
                ) {
                    Ok(_) => self.alert = "banned".into(),
                    Err(e) => self.alert = format!("Error: {}", e),
                }
            }
            if ui.button("unban self hwid").clicked() {
                let autherium = autherium_rs::Autherium::new(&self.autherium_url.clone()).unwrap();
                match autherium.unban_hwid(
                    &autherium_rs::Autherium::get_hwid().unwrap().into(),
                    &"super_secret_key".to_string(),
                ) {
                    Ok(_) => self.alert = "unbanned".into(),
                    Err(e) => self.alert = format!("Error: {}", e),
                }
            }
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.days);
                ui.label("License Days")
            });
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.product_id);
                ui.label("Product ID")
            });
            ui.horizontal(|ui| {
                if ui.button("create license").clicked() {
                    if self.days.parse::<u32>().is_err() {
                        self.alert = "days must be a number".into();
                        return;
                    }
                    let days = self.days.parse::<u32>().unwrap();
                    let autherium =
                        autherium_rs::Autherium::new(&self.autherium_url.clone()).unwrap();
                    match autherium.create_license(
                        days as u64,
                        &"super_secret_key".to_string(),
                        vec![&self.product_id],
                    ) {
                        Ok(license) => self.alert = format!("{} day(s) license: {}", days, license),
                        Err(e) => self.alert = format!("Error: {}", e),
                    }
                }
            });
            ui.label(format!("{}", self.alert));
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 240.0])
            .with_decorations(true)
            .with_active(true)
            .with_taskbar(true)
            .with_resizable(true), //.with_icon(IconData::default()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Replace me!!!!!!!",
        options,
        Box::new(|_cc| {
            Ok(Box::<MyApp>::new(MyApp {
                autherium_url: "http://localhost:8080".into(),
                ..Default::default()
            }))
        }),
    )
    .unwrap();
}
