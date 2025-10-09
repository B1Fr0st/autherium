use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct AuthRequest {
    pub license: String,
    pub product_id: String,
    pub hwid: String,
}

#[derive(Serialize, Default)]
pub struct AuthResponse {
    pub license_start: u64,
    pub license_duration: u64,
    pub time_remaining: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateRequest {
    pub days: u64,
    pub key: String,
    pub product_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateResponse {
    pub license: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HwidRequest {
    pub hwid: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct License {
    pub key: String,
    pub used: bool,
    pub start: u64,
    pub duration: u64,
    pub product_ids: Vec<String>,
}

impl License {
    pub fn new(key: String, product_ids: &Vec<String>) -> Self {
        Self {
            key,
            used: false,
            start: 0,
            duration: 0,
            product_ids: product_ids.clone(),
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
