use actix_web::{App, HttpResponse, HttpServer, Responder, Result, error, post, web};
use regex::Regex;
use std::{fs, path::Path, sync::Mutex};

use rand::{Rng, distr::Alphanumeric};

mod types;
use types::*;

const LICENSES_FILE: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("LICENSES_FILE").unwrap_or_else(|_| "./config/licenses.json".to_string())
});
const BANNED_HWIDS_FILE: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("BANNED_HWIDS_FILE").unwrap_or_else(|_| "./config/banned_hwids.json".to_string())
});
const ARCHIVE_FILE: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("ARCHIVE_FILE").unwrap_or_else(|_| "./config/expired_licenses.json".to_string())
});
const LICENSE_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^[A-Z0-9]{16}").unwrap());
const API_KEY: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("API_KEY").unwrap_or_else(|_| "super_secret_key".to_string())
});
const LICENSE_REGEN_LIMIT: u32 = 100;

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
        if !Path::new(LICENSES_FILE.as_str()).exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(LICENSES_FILE.as_str())?;
        let licenses: Vec<License> = serde_json::from_str(&data)?;
        // add expired licenses to an archive file
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (expired, active): (Vec<License>, Vec<License>) = licenses
            .into_iter()
            .partition(|license| license.used && (license.start + license.duration) <= now);
        if !expired.is_empty() {
            let mut archive = if Path::new(&ARCHIVE_FILE.as_str()).exists() {
                let archive_data = fs::read_to_string(ARCHIVE_FILE.as_str())?;
                serde_json::from_str::<Vec<License>>(&archive_data)?
            } else {
                Vec::new()
            };
            archive.extend(expired);
            let archive_data = serde_json::to_string_pretty(&archive)?;
            fs::write(ARCHIVE_FILE.as_str(), archive_data)?;
        }
        let licenses = active;
        // Save the active licenses back to the licenses file
        let data = serde_json::to_string_pretty(&licenses)?;
        fs::write(LICENSES_FILE.as_str(), data)?;
        Ok(licenses)
    }

    fn load_banned_hwids() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if !Path::new(&BANNED_HWIDS_FILE.as_str()).exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&BANNED_HWIDS_FILE.as_str())?;
        let banned_hwids: Vec<String> = serde_json::from_str(&data)?;
        Ok(banned_hwids)
    }

    pub fn save_licenses(&self) -> Result<(), Box<dyn std::error::Error>> {
        let licenses = self.licenses.lock().unwrap();
        let data = serde_json::to_string_pretty(&*licenses)?;
        fs::write(&LICENSES_FILE.as_str(), data)?;
        Ok(())
    }

    pub fn save_banned_hwids(&self) -> Result<(), Box<dyn std::error::Error>> {
        let banned_hwids = self.banned_hwids.lock().unwrap();
        let data = serde_json::to_string_pretty(&*banned_hwids)?;
        fs::write(BANNED_HWIDS_FILE.as_str(), data)?;
        Ok(())
    }

    pub fn archive_license(&self, license: &License) {
        let mut archive = if Path::new(ARCHIVE_FILE.as_str()).exists() {
            let archive_data = fs::read_to_string(ARCHIVE_FILE.as_str()).unwrap_or_default();
            serde_json::from_str::<Vec<License>>(&archive_data).unwrap_or_default()
        } else {
            Vec::new()
        };
        archive.push(license.clone());
        let archive_data = serde_json::to_string_pretty(&archive).unwrap_or_default();
        fs::write(ARCHIVE_FILE.as_str(), archive_data).unwrap_or_default();
    }
}

#[post("/create-license")]
async fn create_license(
    req: web::Json<CreateRequest>,
    state: web::Data<State>,
) -> Result<impl Responder> {
    if req.key != API_KEY.as_str() {
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
    let mut s;
    let mut regen_counter = 0;
    loop {
        if regen_counter > LICENSE_REGEN_LIMIT {
            return Err(error::InternalError::from_response(
                "Failed to generate a unique license key.",
                HttpResponse::InternalServerError().json(ErrorResponse::new(
                    "Failed to generate a unique license key.",
                )),
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
            LICENSE_REGEX.is_match(&s),
            "Generated license key does not match regex"
        );
        if licenses.iter().any(|license| license.key == s) {
            //regenerate license if it already exists, and add 1 to the regen counter
            regen_counter += 1;
        } else {
            break;
        }
    }
    let license = License::new(s.clone(), &req.product_ids).set_days(req.days);
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

    if !LICENSE_REGEX.is_match(&req.license) {
        return Err(error::InternalError::from_response(
            "Not a valid license.",
            HttpResponse::Unauthorized().json(ErrorResponse::new("Not a valid license.")),
        )
        .into());
    }

    let mut save_needed = false;
    let result = {
        let mut licenses = state.licenses.lock().unwrap();
        if let Some(license) = licenses
            .iter_mut()
            .find(|entry| entry.key == req.license && entry.product_ids.contains(&req.product_id))
        {
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
                //remove expired license from db to save on search time, add it to archive file
                let key = license.key.clone(); //borrow checker
                state.archive_license(&license);
                licenses.retain(|l| l.key != key);
                save_needed = true;
                Err(error::InternalError::from_response(
                    "Your license has expired.",
                    HttpResponse::Unauthorized()
                        .json(ErrorResponse::new("Your license has expired.")),
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
    if req.key != API_KEY.as_str() {
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
async fn unban_hwid(
    req: web::Json<HwidRequest>,
    state: web::Data<State>,
) -> Result<impl Responder> {
    if req.key != API_KEY.as_str() {
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
    // if let Err(e) = std::env::var("API_KEY") {
    //     eprintln!("API_KEY environment variable is not set: {}", e);
    //     eprintln!("Please set it before running the server.");
    //     std::process::exit(1);
    // }
    let state = web::Data::new(State::new());

    HttpServer::new(move || {
        let auth_json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _| {
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
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
