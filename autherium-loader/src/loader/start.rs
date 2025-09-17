use std::{sync::{atomic::AtomicI64, Arc}, thread::JoinHandle};

use autherium_rs::Autherium;



pub fn start(callback_target: Option<Arc<AtomicI64>>){
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([520.0, 240.0])
            .with_decorations(false)
            .with_active(true)
            .with_taskbar(true)
            .with_resizable(false),
        centered: true,
        ..Default::default()
    };
    
    // Create the MyApp instance
    let app = crate::loader::app::MyApp {
        autherium_url: "http://localhost:8080".into(),
        ..Default::default()
    };

    

    // Run the native window (if you still want to show the UI)
    eframe::run_native(
        "Replace me!!!!!!!",
        options,
        Box::new(|_| Ok(Box::new(app))),
    ).unwrap();

    let license = std::fs::read_to_string("license.txt").unwrap_or_else(|_| "License file not found.".to_string());
    autherium_rs::register_callback(Autherium::new("http://localhost:8080","app_id").unwrap(), license, callback_target);
}

pub fn error(e:&str){

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
        .with_inner_size([320.0, 240.0])
        .with_decorations(false)
        .with_active(true)
        .with_taskbar(true)
        ,//.with_icon(IconData::default()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Replace me!!!!!!!",
        options,
        Box::new(|_cc| Ok({
            let mut app = Box::<crate::loader::app::MyApp>::default();
            app.ui_state = crate::loader::app::UiState::Error;
            app.failed_reason = e.to_string();
            app
        })),
    ).unwrap();
}