pub mod dirs;
mod errors;
pub mod types;
pub use errors::*;
mod client;
pub use client::*;
pub mod logging;
pub mod provider;
pub const DAEMON_ADDR: &str = "127.0.0.1:42069";
