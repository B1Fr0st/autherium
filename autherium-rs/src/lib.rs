pub struct Autherium {
    base_url: String,
    app_id: String,
    hwid: String,
    client: reqwest::blocking::Client,
    license_regex: Regex,
}

use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Serialize)]
pub struct AuthRequest {
    license: String,
    hwid: String,
    app_id: String,
}

#[derive(Deserialize,Clone, Debug)]
#[serde(untagged)]
pub enum AuthResponse {
    Success{
        license_start: u64,
        license_duration: u64,
        time_remaining: i64
    },
    Error{
        error: String
    },
}

#[derive(Serialize, Clone, Debug)]
struct CreateRequest {
    days: u64,
    key: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum CreateResponse {
    License { license: String },
    Error { error: String },
}

impl Autherium {
    pub fn new(base_url:&str, app_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::ClientBuilder::new().build()?;
        Ok(Self {
            base_url: base_url.into(),
            app_id: app_id.into(),
            hwid: Self::get_hwid()?, // Placeholder HWID
            client,
            license_regex: Regex::new(r"^[A-Z0-9]{16}$").unwrap(),
        })
    }

    pub fn get_hwid() -> Result<String, Box<dyn std::error::Error>> {
        use machineid_rs::{IdBuilder, Encryption, HWIDComponent};
        Ok(IdBuilder::new(Encryption::SHA256)
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::DriveSerial)
            .add_component(HWIDComponent::CPUCores)
            .add_component(HWIDComponent::CPUID)
            .build("Autherium")?)
    }

    pub fn check_license_format(&self, license: &str) -> bool {
        self.license_regex.is_match(license)
    }

    pub fn authenticate(&self, license: &String) -> Result<AuthResponse, Box<dyn std::error::Error>> {
        if !self.check_license_format(&license) {
            return Err("Invalid license format".into());
        }

        let request = AuthRequest {
            license: license.clone(),
            hwid: self.hwid.clone(),
            app_id: self.app_id.clone(),
        };

        let response = self.client.post(&format!("{}/api/v1/auth", self.base_url))
            .json(&request)
            .send()?;

        let response_body: AuthResponse = response.json()?;

        match response_body {
            AuthResponse::Success { license_start, license_duration, time_remaining } => {
                Ok(AuthResponse::Success { license_start, license_duration, time_remaining })
            }
            AuthResponse::Error { error } => {
                Err(format!("Authentication failed: {}", error).into())
            }
        }
    }

    pub fn create_license(&self, days: u64, key:&String) -> Result<String, Box<dyn std::error::Error>> {
        let request = CreateRequest { days, key: key.clone() };

        let response = self.client.post(&format!("{}/api/v1/create-license", self.base_url))
            .json(&request)
            .send()?;

        let response_body: CreateResponse = response.json()?;

        match response_body {
            CreateResponse::License { license } => Ok(license),
            CreateResponse::Error { error } => Err(format!("Failed to create license: {}", error).into()),
        }
    }

    pub fn ban_hwid(&self, hwid: &String, key: &String) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.post(&format!("{}/api/v1/ban-hwid", self.base_url))
            .json(&serde_json::json!({ "hwid": hwid, "key": key }))
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: serde_json::Value = response.json()?;
            Err(format!("Failed to ban HWID: {}", error["error"]).into())
        }
    }
    pub fn unban_hwid(&self, hwid: &String, key: &String) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.post(&format!("{}/api/v1/unban-hwid", self.base_url))
            .json(&serde_json::json!({ "hwid": hwid, "key": key }))
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: serde_json::Value = response.json()?;
            Err(format!("Failed to unban HWID: {}", error["error"]).into())
        }
    }

}