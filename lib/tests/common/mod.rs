#[cfg(test)]
pub fn init() {
    use env_logger;
    use std::env;

    env::set_var("RUST_LOG", "DEBUG");
    env_logger::builder().try_init().unwrap_or_default();
}
