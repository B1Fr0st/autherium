use std::sync::{Arc, atomic::AtomicI64};

pub mod loader;
fn main() {
    let time_remaining = Arc::new(AtomicI64::new(0));
    crate::loader::start::start(
        "thrum",
        "http://localhost:8080",
        "thrum",
        "https://discord.com",
        "https://thrummenu.dev",
        Some(time_remaining.clone()),
    );

    crate::loader::start::error("thrum", "Failure!!");
}
