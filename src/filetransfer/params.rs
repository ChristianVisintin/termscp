//! ## Params
//!
//! file transfer parameters

/**
 * MIT License
 *
 * termscp - Copyright (c) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use super::FileTransferProtocol;

use std::path::{Path, PathBuf};

/// ### FileTransferParams
///
/// Holds connection parameters for file transfers
#[derive(Debug, Clone)]
pub struct FileTransferParams {
    pub protocol: FileTransferProtocol,
    pub params: ProtocolParams,
    pub entry_directory: Option<PathBuf>,
}

/// ## ProtocolParams
///
/// Container for protocol params
#[derive(Debug, Clone)]
pub enum ProtocolParams {
    Generic(GenericProtocolParams),
    AwsS3(AwsS3Params),
}

/// ## GenericProtocolParams
///
/// Protocol params used by most common protocols
#[derive(Debug, Clone)]
pub struct GenericProtocolParams {
    pub address: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// ## AwsS3Params
///
/// Connection parameters for AWS S3 protocol
#[derive(Debug, Clone)]
pub struct AwsS3Params {
    pub bucket_name: String,
    pub region: String,
    pub profile: Option<String>,
}

impl FileTransferParams {
    /// ### new
    ///
    /// Instantiates a new `FileTransferParams`
    pub fn new(protocol: FileTransferProtocol, params: ProtocolParams) -> Self {
        Self {
            protocol,
            params,
            entry_directory: None,
        }
    }

    /// ### protocol
    ///
    /// Set protocol for params
    pub fn protocol(mut self, protocol: FileTransferProtocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// ### entry_directory
    ///
    /// Set entry directory
    pub fn entry_directory<P: AsRef<Path>>(mut self, dir: Option<P>) -> Self {
        self.entry_directory = dir.map(|x| x.as_ref().to_path_buf());
        self
    }
}

impl Default for FileTransferParams {
    fn default() -> Self {
        Self::new(FileTransferProtocol::Sftp, ProtocolParams::default())
    }
}

impl Default for ProtocolParams {
    fn default() -> Self {
        Self::Generic(GenericProtocolParams::default())
    }
}

impl ProtocolParams {
    /// ### generic_params
    ///
    /// Retrieve generic parameters from protocol params if any
    pub fn generic_params(&self) -> Option<&GenericProtocolParams> {
        match self {
            ProtocolParams::Generic(params) => Some(params),
            _ => None,
        }
    }

    pub fn mut_generic_params(&mut self) -> Option<&mut GenericProtocolParams> {
        match self {
            ProtocolParams::Generic(params) => Some(params),
            _ => None,
        }
    }

    /// ### s3_params
    ///
    /// Retrieve AWS S3 parameters if any
    pub fn s3_params(&self) -> Option<&AwsS3Params> {
        match self {
            ProtocolParams::AwsS3(params) => Some(params),
            _ => None,
        }
    }
}

// -- Generic protocol params

impl Default for GenericProtocolParams {
    fn default() -> Self {
        Self {
            address: "localhost".to_string(),
            port: 22,
            username: None,
            password: None,
        }
    }
}

impl GenericProtocolParams {
    /// ### address
    ///
    /// Set address to params
    pub fn address<S: AsRef<str>>(mut self, address: S) -> Self {
        self.address = address.as_ref().to_string();
        self
    }

    /// ### port
    ///
    /// Set port to params
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// ### username
    ///
    /// Set username for params
    pub fn username<S: AsRef<str>>(mut self, username: Option<S>) -> Self {
        self.username = username.map(|x| x.as_ref().to_string());
        self
    }

    /// ### password
    ///
    /// Set password for params
    pub fn password<S: AsRef<str>>(mut self, password: Option<S>) -> Self {
        self.password = password.map(|x| x.as_ref().to_string());
        self
    }
}

// -- S3 params

impl AwsS3Params {
    /// ### new
    ///
    /// Instantiates a new `AwsS3Params` struct
    pub fn new<S: AsRef<str>>(bucket: S, region: S, profile: Option<S>) -> Self {
        Self {
            bucket_name: bucket.as_ref().to_string(),
            region: region.as_ref().to_string(),
            profile: profile.map(|x| x.as_ref().to_string()),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_filetransfer_params() {
        let params: FileTransferParams =
            FileTransferParams::new(FileTransferProtocol::Sftp, ProtocolParams::default())
                .protocol(FileTransferProtocol::Scp)
                .entry_directory(Some(&Path::new("/tmp")));
        assert_eq!(
            params.params.generic_params().unwrap().address.as_str(),
            "localhost"
        );
        assert_eq!(params.protocol, FileTransferProtocol::Scp);
        assert_eq!(
            params.entry_directory.as_deref().unwrap(),
            Path::new("/tmp")
        );
    }

    #[test]
    fn params_default() {
        let params: GenericProtocolParams = ProtocolParams::default()
            .generic_params()
            .unwrap()
            .to_owned();
        assert_eq!(params.address.as_str(), "localhost");
        assert_eq!(params.port, 22);
        assert!(params.username.is_none());
        assert!(params.password.is_none());
    }

    #[test]
    fn params_aws_s3() {
        let params: AwsS3Params = AwsS3Params::new("omar", "eu-west-1", Some("test"));
        assert_eq!(params.bucket_name.as_str(), "omar");
        assert_eq!(params.region.as_str(), "eu-west-1");
        assert_eq!(params.profile.as_deref().unwrap(), "test");
    }

    #[test]
    fn references() {
        let mut params = ProtocolParams::AwsS3(AwsS3Params::new("omar", "eu-west-1", Some("test")));
        assert!(params.s3_params().is_some());
        assert!(params.generic_params().is_none());
        assert!(params.mut_generic_params().is_none());
        let mut params = ProtocolParams::default();
        assert!(params.s3_params().is_none());
        assert!(params.generic_params().is_some());
        assert!(params.mut_generic_params().is_some());
    }
}
