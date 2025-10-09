use crate::loader::app::*;
use autherium_rs::AuthResponse;
use regex::Regex;
use std::sync::mpsc;
use std::thread;

impl crate::loader::app::MyApp {
    pub fn verify_license_async(&mut self) {
        self.failed_reason = String::new();

        // Create channel for communication
        let (tx, rx) = mpsc::channel();
        self.license_receiver = Some(rx);

        let license = self.license.clone();
        let autherium_url = self.autherium_url.clone();
        let product_id = self.product_id.clone();

        // Spawn background thread for license verification
        thread::spawn(move || {
            let autherium = autherium_rs::Autherium::new(&autherium_url).unwrap();
            match autherium.authenticate(&license, product_id) {
                Ok(response) => match response {
                    AuthResponse::Success {
                        license_start,
                        license_duration,
                        time_remaining: _,
                    } => {
                        let _ = tx.send(LicenseResult::Success(license_start, license_duration));
                        return;
                    }
                    AuthResponse::Error { error } => {
                        tx.send(LicenseResult::Error(error.to_string())).unwrap();
                        return;
                    }
                },
                Err(_) => {
                    tx.send(LicenseResult::Error("Failed to authenticate!".to_string()))
                        .unwrap();
                    return;
                }
            }
        });
    }

    pub fn check_license_result(&mut self) {
        if let Some(ref receiver) = self.license_receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    self.license_receiver = None;

                    match result {
                        LicenseResult::Success(license_start, license_duration) => {
                            self.ui_state = UiState::Verified;
                            self.license_timing = (license_start, license_duration);
                        }
                        LicenseResult::Error(error) => {
                            self.failed_reason = error;
                            self.license = String::new();
                            self.ui_state = UiState::LicenseInput;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Still waiting for result
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Thread panicked or channel closed
                    self.license_receiver = None;
                    self.failed_reason = "Error: 0xA011".to_string();
                    self.license = String::new();
                    self.ui_state = UiState::Error;
                }
            }
        }
    }

    pub fn license_regex(&self) -> bool {
        let regex = Regex::new(r"^[A-Z0-9]{16}").unwrap();
        regex.is_match(&self.license)
    }
}
