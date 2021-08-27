//! # transfer
//!
//! This module exposes all the file transfers supported by termscp

// -- import
use super::{FileTransfer, FileTransferError, FileTransferErrorType, ProtocolParams};

// -- modules
mod ftp;
mod s3;
mod scp;
mod sftp;

// -- export
pub use self::s3::S3FileTransfer;
pub use ftp::FtpFileTransfer;
pub use scp::ScpFileTransfer;
pub use sftp::SftpFileTransfer;
