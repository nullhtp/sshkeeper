pub mod actions;
pub mod key_setup;
mod system;
pub mod transfer;

pub use system::SystemSshBackend;

use crate::model::Connection;
use anyhow::Result;

pub trait SshBackend {
    fn connect(&self, profile: &Connection) -> Result<()>;
}
