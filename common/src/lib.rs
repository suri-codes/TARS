pub mod dirs;
mod errors;
pub mod types;
pub use errors::*;
mod client;
pub use client::*;
pub const DAEMON_ADDR: &str = "0.0.0.0:42069";
