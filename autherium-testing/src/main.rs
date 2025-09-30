use autherium_rs::Autherium;

pub fn auth_loop(autherium: &Autherium, license: &String) {
    let mut total = 0.0;
    let s = std::time::Instant::now();
    while s.elapsed().as_secs_f32() < 10.0 {
        let start = std::time::Instant::now();
        let _  = autherium.authenticate(license);
        total += start.elapsed().as_secs_f32();
    }
    println!("Average auth time: {}ms", (total / (s.elapsed().as_secs_f32() * 1000.0)));
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "super_secret_key".to_string();
    let autherium = Autherium::new("http://localhost:8080", "app_id")?;
    let license = autherium.create_license(1, &api_key)?;
    //auth loop
    for _ in 0..100{
        std::thread::spawn({
            let autherium = Autherium::new("http://localhost:8080", "app_id")?;
            let license = license.clone();
            move || {
                auth_loop(&autherium, &license);
            }
        });
    }
    std::thread::sleep(std::time::Duration::from_secs(12));
    Ok(())
}
