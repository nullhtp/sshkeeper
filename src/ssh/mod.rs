mod system;
pub mod key_setup;
pub mod transfer;

pub use system::SystemSshBackend;

use crate::model::Connection;
use anyhow::Result;

pub trait SshBackend {
    fn connect(&self, profile: &Connection) -> Result<()>;
}
