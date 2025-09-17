use std::{sync::Mutex, fs, path::Path};
use actix_web::{error, post, web, App, HttpResponse, HttpServer, Responder, Result};
use serde_derive::{Serialize, Deserialize};
use rand::{distr::Alphanumeric, Rng};

const LICENSES_FILE: &str = "licenses.json";
const BANNED_HWIDS_FILE: &str = "banned_hwids.json";

struct State {
    pub licenses: Mutex<Vec<License>>,
    pub banned_hwids: Mutex<Vec<String>>,
}

impl State {
    pub fn new() -> Self {
        let licenses = Self::load_licenses().unwrap_or_else(|_| Vec::new());
        let banned_hwids = Self::load_banned_hwids().unwrap_or_else(|_| Vec::new());
        
        Self {
            licenses: Mutex::new(licenses),
            banned_hwids: Mutex::new(banned_hwids),
        }
    }

    fn load_licenses() -> Result<Vec<License>, Box<dyn std::error::Error>> {
        if !Path::new(LICENSES_FILE).exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(LICENSES_FILE)?;
        let licenses: Vec<License> = serde_json::from_str(&data)?;
        Ok(licenses)
    }

    fn load_banned_hwids() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if !Path::new(BANNED_HWIDS_FILE).exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(BANNED_HWIDS_FILE)?;
        let banned_hwids: Vec<String> = serde_json::from_str(&data)?;
        Ok(banned_hwids)
    }

    pub fn save_licenses(&self) -> Result<(), Box<dyn std::error::Error>> {
        let licenses = self.licenses.lock().unwrap();
        let data = serde_json::to_string_pretty(&*licenses)?;
        fs::write(LICENSES_FILE, data)?;
        Ok(())
    }

    pub fn save_banned_hwids(&self) -> Result<(), Box<dyn std::error::Error>> {
        let banned_hwids = self.banned_hwids.lock().unwrap();
        let data = serde_json::to_string_pretty(&*banned_hwids)?;
        fs::write(BANNED_HWIDS_FILE, data)?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct AuthRequest {
    license: String,
    hwid: String,
}

#[derive(Serialize, Default)]
struct AuthResponse {
    license_start: u64,
    license_duration: u64,
    time_remaining: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CreateRequest {
    days: u64,
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateResponse {
    license: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HwidRequest {
    hwid: String,
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct License {
    pub key: String,
    pub used: bool,
    pub start: u64,
    pub duration: u64,
}

impl License {
    pub fn new(key: String) -> Self {
        Self {
            key,
            used: false,
            start: 0,
            duration: 0,
        }
    }

    pub fn set_days(&mut self, days: u64) -> Self {
        self.duration = days * 24 * 60 * 60; // days * hours * minutes * seconds
        self.clone()
    }

    pub fn start(&mut self) {
        self.used = true;
        self.start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

#[post("/create-license")]
async fn create_license(req: web::Json<CreateRequest>, state: web::Data<State>) -> Result<impl Responder> {
    if req.key != "super_secret_key" {
        //fixme
        return Err(error::InternalError::from_response(
            "Invalid API key.",
            HttpResponse::Unauthorized().json(ErrorResponse::new("Invalid API key.")),
        )
        .into());
    }

    if req.days > std::u64::MAX / (24 * 60 * 60) {
        return Err(error::InternalError::from_response(
            "Invalid number of days.",
            HttpResponse::BadRequest().json(ErrorResponse::new("Invalid number of days.")),
        )
        .into());
    }

    let mut licenses = state.licenses.lock().unwrap();
    //generate license with valid regex
    let regex = regex::Regex::new(r"^[A-Z0-9]{16}").unwrap();
    let mut s;
    let mut regen_counter = 0;
    loop {
        if regen_counter > 100 {
            return Err(error::InternalError::from_response(
                "Failed to generate a unique license key.",
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::new("Failed to generate a unique license key.")),
            )
            .into());
        }
        s = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>()
            .to_uppercase();
        assert!(
            regex.is_match(&s),
            "Generated license key does not match regex"
        );
        if licenses.iter().any(|license| license.key == s) {
            //regenerate license if it already exists, and add 1 to the regen counter
            regen_counter += 1;
        } else {
            break;
        }
    }
    let mut license = License::new(s.clone()).set_days(req.days);
    license.duration = 60;
    licenses.push(license);
    
    // Save to file after modification
    drop(licenses); // Release the lock before saving
    if let Err(e) = state.save_licenses() {
        eprintln!("Failed to save licenses: {}", e);
    }
    
    Ok(HttpResponse::Created().json(CreateResponse { license: s }))
}

#[post("/auth")]
async fn auth(req: web::Json<AuthRequest>, state: web::Data<State>) -> Result<impl Responder> {
    dbg!(&req);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    if state
        .banned_hwids
        .lock()
        .unwrap()
        .iter()
        .find(|entry| **entry == req.hwid)
        .is_some()
    {
        return Ok(HttpResponse::Unauthorized().json(ErrorResponse::new("Your HWID is banned.")));
    }

    let regex = regex::Regex::new(r"^[A-Z0-9]{16}").unwrap();

    if !regex.is_match(&req.license) {
        return Err(error::InternalError::from_response(
            "Not a valid license.",
            HttpResponse::Unauthorized().json(ErrorResponse::new("Not a valid license.")),
        )
        .into());
    }

    let mut save_needed = false;
    let result = {
        let mut licenses = state.licenses.lock().unwrap();
        if let Some(license) = licenses.iter_mut().find(|entry| entry.key == req.license) {
            if !license.used {
                license.start();
                save_needed = true;
            }
            let time_remaining = (license.start + license.duration) as i64
                - std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
            if time_remaining <= 0 {
                Err(error::InternalError::from_response(
                    "Your license has expired.",
                    HttpResponse::Unauthorized().json(ErrorResponse::new("Your license has expired.")),
                )
                .into())
            } else {
                Ok(HttpResponse::Ok().json(AuthResponse {
                    license_start: license.start,
                    license_duration: license.duration,
                    time_remaining,
                }))
            }
        } else {
            Err(error::InternalError::from_response(
                "Not a valid license.",
                HttpResponse::Unauthorized().json(ErrorResponse::new("Not a valid license.")),
            )
            .into())
        }
    };

    // Save if license was activated
    if save_needed {
        if let Err(e) = state.save_licenses() {
            eprintln!("Failed to save licenses: {}", e);
        }
    }

    result
}

#[post("/ban-hwid")]
async fn ban_hwid(req: web::Json<HwidRequest>, state: web::Data<State>) -> Result<impl Responder> {
    if req.key != "super_secret_key" {
        //fixme
        return Err(error::InternalError::from_response(
            "Invalid API key.",
            HttpResponse::Unauthorized().json(ErrorResponse::new("Invalid API key.")),
        )
        .into());
    }
    
    {
        let mut banned_hwids = state.banned_hwids.lock().unwrap();
        if !banned_hwids.contains(&req.hwid) {
            banned_hwids.push(req.hwid.clone());
        }
    }
    
    // Save to file after modification
    if let Err(e) = state.save_banned_hwids() {
        eprintln!("Failed to save banned HWIDs: {}", e);
    }
    
    Ok(HttpResponse::Ok().json(ErrorResponse::new("HWID banned successfully.")))
}

#[post("/unban-hwid")]
async fn unban_hwid(req: web::Json<HwidRequest>, state: web::Data<State>) -> Result<impl Responder> {
    if req.key != "super_secret_key" {
        //fixme
        return Err(error::InternalError::from_response(
            "Invalid API key.",
            HttpResponse::Unauthorized().json(ErrorResponse::new("Invalid API key.")),
        )
        .into());
    }
    
    {
        let mut banned_hwids = state.banned_hwids.lock().unwrap();
        dbg!(&banned_hwids);
        dbg!(&req.hwid);
        if let Some(pos) = banned_hwids.iter().position(|x| *x == req.hwid) {
            banned_hwids.remove(pos);
        }
        dbg!(&banned_hwids);
    }
    
    // Save to file after modification
    if let Err(e) = state.save_banned_hwids() {
        eprintln!("Failed to save banned HWIDs: {}", e);
    }
    
    Ok(HttpResponse::Ok().json(ErrorResponse::new("HWID unbanned successfully.")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(State::new());

    HttpServer::new(move || {
        let auth_json_config = web::JsonConfig::default().limit(4096).error_handler(|err, _| {
            error::InternalError::from_response(
                err,
                HttpResponse::BadRequest()
                    .json(ErrorResponse::new("The auth payload was malformed."))
                    .into(),
            )
            .into()
        });

        App::new().service(
            web::scope("/api/v1")
                .app_data(auth_json_config)
                .app_data(state.clone())
                .service(auth)
                .service(create_license)
                .service(ban_hwid)
                .service(unban_hwid),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}