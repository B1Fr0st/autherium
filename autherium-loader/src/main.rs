pub mod loader;
fn main() {
    crate::loader::start::start(None);
    crate::loader::start::error("Failure!!");
}
