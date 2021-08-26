//! ## FileTransferActivity
//!
//! `filetransfer_activiy` is the module which implements the Filetransfer activity, which is the main activity afterall

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
// locals
use super::{FileTransferActivity, FsEntry, LogLevel, SelectedEntry};

impl FileTransferActivity {
    pub(crate) fn action_local_delete(&mut self) {
        match self.get_local_selected_entries() {
            SelectedEntry::One(entry) => {
                // Delete file
                self.local_remove_file(&entry);
                // Reload
                self.reload_local_dir();
            }
            SelectedEntry::Many(entries) => {
                // Iter files
                for entry in entries.iter() {
                    // Delete file
                    self.local_remove_file(entry);
                }
                // Reload entries
                self.reload_local_dir();
            }
            SelectedEntry::None => {}
        }
    }

    pub(crate) fn action_remote_delete(&mut self) {
        match self.get_remote_selected_entries() {
            SelectedEntry::One(entry) => {
                // Delete file
                self.remote_remove_file(&entry);
                // Reload
                self.reload_remote_dir();
            }
            SelectedEntry::Many(entries) => {
                // Iter files
                for entry in entries.iter() {
                    // Delete file
                    self.remote_remove_file(entry);
                }
                // Reload entries
                self.reload_remote_dir();
            }
            SelectedEntry::None => {}
        }
    }

    pub(crate) fn local_remove_file(&mut self, entry: &FsEntry) {
        match self.host.remove(entry) {
            Ok(_) => {
                // Log
                self.log(
                    LogLevel::Info,
                    format!("Removed file \"{}\"", entry.get_abs_path().display()),
                );
            }
            Err(err) => {
                self.log_and_alert(
                    LogLevel::Error,
                    format!(
                        "Could not delete file \"{}\": {}",
                        entry.get_abs_path().display(),
                        err
                    ),
                );
            }
        }
    }

    pub(crate) fn remote_remove_file(&mut self, entry: &FsEntry) {
        match self.client.remove(entry) {
            Ok(_) => {
                self.log(
                    LogLevel::Info,
                    format!("Removed file \"{}\"", entry.get_abs_path().display()),
                );
            }
            Err(err) => {
                self.log_and_alert(
                    LogLevel::Error,
                    format!(
                        "Could not delete file \"{}\": {}",
                        entry.get_abs_path().display(),
                        err
                    ),
                );
            }
        }
    }
}
