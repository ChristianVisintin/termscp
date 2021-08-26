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
// Locals
use super::{ConfigClient, FileTransferActivity, LogLevel, LogRecord};
use crate::system::environment;
use crate::system::sshkey_storage::SshKeyStorage;
use crate::utils::path;
// Ext
use std::env;
use std::path::{Path, PathBuf};
use tuirealm::Update;

const LOG_CAPACITY: usize = 256;

impl FileTransferActivity {
    /// ### log
    ///
    /// Add message to log events
    pub(super) fn log(&mut self, level: LogLevel, msg: String) {
        // Log to file
        match level {
            LogLevel::Error => error!("{}", msg),
            LogLevel::Info => info!("{}", msg),
            LogLevel::Warn => warn!("{}", msg),
        }
        // Create log record
        let record: LogRecord = LogRecord::new(level, msg);
        //Check if history overflows the size
        if self.log_records.len() + 1 > LOG_CAPACITY {
            self.log_records.pop_back(); // Start cleaning events from back
        }
        // Eventually push front the new record
        self.log_records.push_front(record);
        // Update log
        let msg = self.update_logbox();
        self.update(msg);
    }

    /// ### log_and_alert
    ///
    /// Add message to log events and also display it as an alert
    pub(super) fn log_and_alert(&mut self, level: LogLevel, msg: String) {
        self.mount_error(msg.as_str());
        self.log(level, msg);
        // Update log
        let msg = self.update_logbox();
        self.update(msg);
    }

    /// ### init_config_client
    ///
    /// Initialize configuration client if possible.
    /// This function doesn't return errors.
    pub(super) fn init_config_client() -> ConfigClient {
        match environment::init_config_dir() {
            Ok(termscp_dir) => match termscp_dir {
                Some(termscp_dir) => {
                    // Make configuration file path and ssh keys path
                    let (config_path, ssh_keys_path): (PathBuf, PathBuf) =
                        environment::get_config_paths(termscp_dir.as_path());
                    match ConfigClient::new(config_path.as_path(), ssh_keys_path.as_path()) {
                        Ok(config_client) => config_client,
                        Err(_) => ConfigClient::degraded(),
                    }
                }
                None => ConfigClient::degraded(),
            },
            Err(_) => ConfigClient::degraded(),
        }
    }

    /// ### make_ssh_storage
    ///
    /// Make ssh storage from `ConfigClient` if possible, empty otherwise (empty is implicit if degraded)
    pub(super) fn make_ssh_storage(cli: &ConfigClient) -> SshKeyStorage {
        SshKeyStorage::storage_from_config(cli)
    }

    /// ### setup_text_editor
    ///
    /// Set text editor to use
    pub(super) fn setup_text_editor(&self) {
        env::set_var("EDITOR", self.config().get_text_editor());
    }

    /// ### read_input_event
    ///
    /// Read one event.
    /// Returns whether at least one event has been handled
    pub(super) fn read_input_event(&mut self) -> bool {
        if let Ok(Some(event)) = self.context().input_hnd().read_event() {
            // Handle event
            let msg = self.view.on(event);
            self.update(msg);
            // Return true
            true
        } else {
            // Error
            false
        }
    }

    /// ### local_to_abs_path
    ///
    /// Convert a path to absolute according to local explorer
    pub(super) fn local_to_abs_path(&self, path: &Path) -> PathBuf {
        path::absolutize(self.local().wrkdir.as_path(), path)
    }

    /// ### remote_to_abs_path
    ///
    /// Convert a path to absolute according to remote explorer
    pub(super) fn remote_to_abs_path(&self, path: &Path) -> PathBuf {
        path::absolutize(self.remote().wrkdir.as_path(), path)
    }
}
