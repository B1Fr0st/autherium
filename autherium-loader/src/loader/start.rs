use std::sync::{Arc, atomic::AtomicI64};

use autherium_rs::Autherium;

pub fn start(
    window_name: &str,
    autherium_url: &str,
    product_id: &str,
    discord_url: &str,
    website_url: &str,
    callback_target: Option<Arc<AtomicI64>>,
) {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 240.0])
            .with_decorations(false)
            .with_active(true)
            .with_taskbar(true)
            .with_resizable(false)
            .with_transparent(true),
        centered: true,
        ..Default::default()
    };

    // Create the MyApp instance
    let app = crate::loader::app::MyApp {
        autherium_url: autherium_url.to_string(),
        product_id: product_id.to_string(),
        discord_url: discord_url.to_string(),
        website_url: website_url.to_string(),
        ..Default::default()
    };

    // Run the native window (if you still want to show the UI)
    eframe::run_native(window_name, options, Box::new(|_| Ok(Box::new(app)))).unwrap();

    let license = std::fs::read_to_string("license.key")
        .unwrap_or_else(|_| "License file not found.".to_string());
    autherium_rs::register_callback(
        Autherium::new(&autherium_url).unwrap(),
        product_id.into(),
        license,
        callback_target,
    );
}

pub fn error(window_name: &str, e: &str) {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_decorations(false)
            .with_active(true)
            .with_taskbar(true)
            .with_transparent(true), //.with_icon(IconData::default()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        window_name,
        options,
        Box::new(|_cc| {
            Ok({
                let mut app = Box::<crate::loader::app::MyApp>::default();
                app.ui_state = crate::loader::app::UiState::Error;
                app.failed_reason = e.to_string();
                app
            })
        }),
    )
    .unwrap();
}
