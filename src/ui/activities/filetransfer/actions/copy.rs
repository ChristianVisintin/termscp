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
use super::{FileTransferActivity, LogLevel, SelectedEntry, TransferPayload};

use remotefs::{Entry, RemoteErrorType};
use std::path::{Path, PathBuf};

impl FileTransferActivity {
    /// Copy file on local
    pub(crate) fn action_local_copy(&mut self, input: String) {
        match self.get_local_selected_entries() {
            SelectedEntry::One(entry) => {
                let dest_path: PathBuf = PathBuf::from(input);
                self.local_copy_file(&entry, dest_path.as_path());
                // Reload entries
                self.reload_local_dir();
            }
            SelectedEntry::Many(entries) => {
                // Try to copy each file to Input/{FILE_NAME}
                let base_path: PathBuf = PathBuf::from(input);
                // Iter files
                for entry in entries.iter() {
                    let mut dest_path: PathBuf = base_path.clone();
                    dest_path.push(entry.name());
                    self.local_copy_file(entry, dest_path.as_path());
                }
                // Reload entries
                self.reload_local_dir();
            }
            SelectedEntry::None => {}
        }
    }

    /// Copy file on remote
    pub(crate) fn action_remote_copy(&mut self, input: String) {
        match self.get_remote_selected_entries() {
            SelectedEntry::One(entry) => {
                let dest_path: PathBuf = PathBuf::from(input);
                self.remote_copy_file(entry, dest_path.as_path());
                // Reload entries
                self.reload_remote_dir();
            }
            SelectedEntry::Many(entries) => {
                // Try to copy each file to Input/{FILE_NAME}
                let base_path: PathBuf = PathBuf::from(input);
                // Iter files
                for entry in entries.into_iter() {
                    let mut dest_path: PathBuf = base_path.clone();
                    dest_path.push(entry.name());
                    self.remote_copy_file(entry, dest_path.as_path());
                }
                // Reload entries
                self.reload_remote_dir();
            }
            SelectedEntry::None => {}
        }
    }

    fn local_copy_file(&mut self, entry: &Entry, dest: &Path) {
        match self.host.copy(entry, dest) {
            Ok(_) => {
                self.log(
                    LogLevel::Info,
                    format!(
                        "Copied \"{}\" to \"{}\"",
                        entry.path().display(),
                        dest.display()
                    ),
                );
            }
            Err(err) => self.log_and_alert(
                LogLevel::Error,
                format!(
                    "Could not copy \"{}\" to \"{}\": {}",
                    entry.path().display(),
                    dest.display(),
                    err
                ),
            ),
        }
    }

    fn remote_copy_file(&mut self, entry: Entry, dest: &Path) {
        match self.client.as_mut().copy(entry.path(), dest) {
            Ok(_) => {
                self.log(
                    LogLevel::Info,
                    format!(
                        "Copied \"{}\" to \"{}\"",
                        entry.path().display(),
                        dest.display()
                    ),
                );
            }
            Err(err) => match err.kind {
                RemoteErrorType::UnsupportedFeature => {
                    // If copy is not supported, perform the tricky copy
                    let _ = self.tricky_copy(entry, dest);
                }
                _ => self.log_and_alert(
                    LogLevel::Error,
                    format!(
                        "Could not copy \"{}\" to \"{}\": {}",
                        entry.path().display(),
                        dest.display(),
                        err
                    ),
                ),
            },
        }
    }

    /// Tricky copy will be used whenever copy command is not available on remote host
    pub(super) fn tricky_copy(&mut self, entry: Entry, dest: &Path) -> Result<(), String> {
        // NOTE: VERY IMPORTANT; wait block must be umounted or something really bad will happen
        self.umount_wait();
        // match entry
        match entry {
            Entry::File(entry) => {
                // Create tempfile
                let tmpfile: tempfile::NamedTempFile = match tempfile::NamedTempFile::new() {
                    Ok(f) => f,
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!("Copy failed: could not create temporary file: {}", err),
                        );
                        return Err(String::from("Could not create temporary file"));
                    }
                };
                // Download file
                let name = entry.name.clone();
                let entry_path = entry.path.clone();
                if let Err(err) =
                    self.filetransfer_recv(TransferPayload::File(entry), tmpfile.path(), Some(name))
                {
                    self.log_and_alert(
                        LogLevel::Error,
                        format!("Copy failed: could not download to temporary file: {}", err),
                    );
                    return Err(err);
                }
                // Get local fs entry
                let tmpfile_entry = match self.host.stat(tmpfile.path()) {
                    Ok(e) => e.unwrap_file(),
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!(
                                "Copy failed: could not stat \"{}\": {}",
                                tmpfile.path().display(),
                                err
                            ),
                        );
                        return Err(err.to_string());
                    }
                };
                // Upload file to destination
                let wrkdir = self.remote().wrkdir.clone();
                if let Err(err) = self.filetransfer_send(
                    TransferPayload::File(tmpfile_entry),
                    wrkdir.as_path(),
                    Some(String::from(dest.to_string_lossy())),
                ) {
                    self.log_and_alert(
                        LogLevel::Error,
                        format!(
                            "Copy failed: could not write file {}: {}",
                            entry_path.display(),
                            err
                        ),
                    );
                    return Err(err);
                }
                Ok(())
            }
            Entry::Directory(_) => {
                let tempdir: tempfile::TempDir = match tempfile::TempDir::new() {
                    Ok(d) => d,
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!("Copy failed: could not create temporary directory: {}", err),
                        );
                        return Err(err.to_string());
                    }
                };
                // Get path of dest
                let mut tempdir_path: PathBuf = tempdir.path().to_path_buf();
                tempdir_path.push(entry.name());
                // Download file
                if let Err(err) =
                    self.filetransfer_recv(TransferPayload::Any(entry), tempdir.path(), None)
                {
                    self.log_and_alert(
                        LogLevel::Error,
                        format!("Copy failed: failed to download file: {}", err),
                    );
                    return Err(err);
                }
                // Stat dir
                let tempdir_entry = match self.host.stat(tempdir_path.as_path()) {
                    Ok(e) => e,
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!(
                                "Copy failed: could not stat \"{}\": {}",
                                tempdir.path().display(),
                                err
                            ),
                        );
                        return Err(err.to_string());
                    }
                };
                // Upload to destination
                let wrkdir: PathBuf = self.remote().wrkdir.clone();
                if let Err(err) = self.filetransfer_send(
                    TransferPayload::Any(tempdir_entry),
                    wrkdir.as_path(),
                    Some(String::from(dest.to_string_lossy())),
                ) {
                    self.log_and_alert(
                        LogLevel::Error,
                        format!("Copy failed: failed to send file: {}", err),
                    );
                    return Err(err);
                }
                Ok(())
            }
        }
    }
}
