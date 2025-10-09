use autherium_rs::Autherium;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "super_secret_key".to_string();
    let autherium = Autherium::new("http://localhost:8080")?;
    let license = "VAEL73BATD8EW2UG".to_string();
    dbg!(autherium.authenticate(&license, "farlight84".to_string()));
    Ok(())
}
