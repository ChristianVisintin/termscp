//! ## AuthActivity
//!
//! `auth_activity` is the module which implements the authentication activity

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
use super::{AuthActivity, FileTransferParams, FileTransferProtocol};
use crate::filetransfer::params::{AwsS3Params, GenericProtocolParams, ProtocolParams};

impl AuthActivity {
    /// ### protocol_opt_to_enum
    ///
    /// Convert radio index for protocol into a `FileTransferProtocol`
    pub(super) fn protocol_opt_to_enum(protocol: usize) -> FileTransferProtocol {
        match protocol {
            1 => FileTransferProtocol::Scp,
            2 => FileTransferProtocol::Ftp(false),
            3 => FileTransferProtocol::Ftp(true),
            4 => FileTransferProtocol::AwsS3,
            _ => FileTransferProtocol::Sftp,
        }
    }

    /// ### protocol_enum_to_opt
    ///
    /// Convert `FileTransferProtocol` enum into radio group index
    pub(super) fn protocol_enum_to_opt(protocol: FileTransferProtocol) -> usize {
        match protocol {
            FileTransferProtocol::Sftp => 0,
            FileTransferProtocol::Scp => 1,
            FileTransferProtocol::Ftp(false) => 2,
            FileTransferProtocol::Ftp(true) => 3,
            FileTransferProtocol::AwsS3 => 4,
        }
    }

    /// ### get_default_port_for_protocol
    ///
    /// Get the default port for protocol
    pub(super) fn get_default_port_for_protocol(protocol: FileTransferProtocol) -> u16 {
        match protocol {
            FileTransferProtocol::Sftp | FileTransferProtocol::Scp => 22,
            FileTransferProtocol::Ftp(_) => 21,
            FileTransferProtocol::AwsS3 => 22, // Doesn't matter, since not used
        }
    }

    /// ### is_port_standard
    ///
    /// Returns whether the port is standard or not
    pub(super) fn is_port_standard(port: u16) -> bool {
        port < 1024
    }

    /// ### check_minimum_window_size
    ///
    /// Check minimum window size window
    pub(super) fn check_minimum_window_size(&mut self, height: u16) {
        if height < 25 {
            // Mount window error
            self.mount_size_err();
        } else {
            self.umount_size_err();
        }
    }

    /// ### collect_host_params
    ///
    /// Collect host params as `FileTransferParams`
    pub(super) fn collect_host_params(&self) -> Result<FileTransferParams, &'static str> {
        let protocol: FileTransferProtocol = self.get_protocol();
        match protocol {
            FileTransferProtocol::AwsS3 => self.collect_s3_host_params(protocol),
            protocol => self.collect_generic_host_params(protocol),
        }
    }

    /// ### collect_generic_host_params
    ///
    /// Get input values from fields or return an error if fields are invalid to work as generic
    pub(super) fn collect_generic_host_params(
        &self,
        protocol: FileTransferProtocol,
    ) -> Result<FileTransferParams, &'static str> {
        let (address, port, username, password): (String, u16, String, String) =
            self.get_generic_params_input();
        if address.is_empty() {
            return Err("Invalid host");
        }
        if port == 0 {
            return Err("Invalid port");
        }
        Ok(FileTransferParams {
            protocol,
            params: ProtocolParams::Generic(
                GenericProtocolParams::default()
                    .address(address)
                    .port(port)
                    .username(match username.is_empty() {
                        true => None,
                        false => Some(username),
                    })
                    .password(match password.is_empty() {
                        true => None,
                        false => Some(password),
                    }),
            ),
            entry_directory: None,
        })
    }

    /// ### collect_s3_host_params
    ///
    /// Get input values from fields or return an error if fields are invalid to work as aws s3
    pub(super) fn collect_s3_host_params(
        &self,
        protocol: FileTransferProtocol,
    ) -> Result<FileTransferParams, &'static str> {
        let (bucket, region, profile): (String, String, Option<String>) =
            self.get_s3_params_input();
        if bucket.is_empty() {
            return Err("Invalid bucket");
        }
        if region.is_empty() {
            return Err("Invalid region");
        }
        Ok(FileTransferParams {
            protocol,
            params: ProtocolParams::AwsS3(AwsS3Params::new(bucket, region, profile)),
            entry_directory: None,
        })
    }
}
