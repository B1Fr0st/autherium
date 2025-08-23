

pub fn start(){

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
        .with_inner_size([520.0, 240.0])
        .with_decorations(false)
        .with_active(true)
        .with_taskbar(true)
        .with_resizable(false)
        ,//.with_icon(IconData::default()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Replace me!!!!!!!",
        options,
        Box::new(|_cc| Ok(Box::<crate::loader::app::MyApp>::new(crate::loader::app::MyApp{
            autherium_url: "http://localhost:8080".into(),
            ..Default::default()
        }))),
    ).unwrap();
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