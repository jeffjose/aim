pub mod connection;
pub mod protocol;
pub mod file_transfer;
pub mod shell;
pub mod server;

pub use connection::{AdbConnection, ConnectionPool};
pub use protocol::{AdbMessage, AdbProtocol};
pub use file_transfer::{FileTransfer, TransferProgress};
pub use shell::{ShellCommand, ShellOutput};
pub use server::AdbServer;

// Re-export commonly used types
pub use crate::error::Result;