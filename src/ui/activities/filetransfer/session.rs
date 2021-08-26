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
// Locals
use super::{FileTransferActivity, LogLevel};
use crate::filetransfer::{FileTransferError, FileTransferErrorType};
use crate::fs::{FsEntry, FsFile};
use crate::host::HostError;
use crate::utils::fmt::fmt_millis;

// Ext
use bytesize::ByteSize;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;

/// ## TransferErrorReason
///
/// Describes the reason that caused an error during a file transfer
#[derive(Error, Debug)]
enum TransferErrorReason {
    #[error("File transfer aborted")]
    Abrupted,
    #[error("Failed to seek file: {0}")]
    CouldNotRewind(std::io::Error),
    #[error("I/O error on localhost: {0}")]
    LocalIoError(std::io::Error),
    #[error("Host error: {0}")]
    HostError(HostError),
    #[error("I/O error on remote: {0}")]
    RemoteIoError(std::io::Error),
    #[error("File transfer error: {0}")]
    FileTransferError(FileTransferError),
}

/// ## TransferPayload
///
/// Represents the entity to send or receive during a transfer.
/// - File: describes an individual `FsFile` to send
/// - Any: Can be any kind of `FsEntry`, but just one
/// - Many: a list of `FsEntry`
#[derive(Debug)]
pub(super) enum TransferPayload {
    File(FsFile),
    Any(FsEntry),
    Many(Vec<FsEntry>),
}

impl FileTransferActivity {
    /// ### connect
    ///
    /// Connect to remote
    pub(super) fn connect(&mut self) {
        let params = self.context().ft_params().unwrap().clone();
        let addr: String = params.address.clone();
        let entry_dir: Option<PathBuf> = params.entry_directory.clone();
        // Connect to remote
        match self.client.connect(
            params.address,
            params.port,
            params.username,
            params.password,
        ) {
            Ok(welcome) => {
                if let Some(banner) = welcome {
                    // Log welcome
                    self.log(
                        LogLevel::Info,
                        format!("Established connection with '{}': \"{}\"", addr, banner),
                    );
                }
                // Try to change directory to entry directory
                let mut remote_chdir: Option<PathBuf> = None;
                if let Some(entry_directory) = &entry_dir {
                    remote_chdir = Some(entry_directory.clone());
                }
                if let Some(entry_directory) = remote_chdir {
                    self.remote_changedir(entry_directory.as_path(), false);
                }
                // Set state to explorer
                self.umount_wait();
                self.reload_remote_dir();
                // Update file lists
                self.update_local_filelist();
                self.update_remote_filelist();
            }
            Err(err) => {
                // Set popup fatal error
                self.umount_wait();
                self.mount_fatal(&err.to_string());
            }
        }
    }

    /// ### disconnect
    ///
    /// disconnect from remote
    pub(super) fn disconnect(&mut self) {
        let params = self.context().ft_params().unwrap();
        let msg: String = format!("Disconnecting from {}…", params.address);
        // Show popup disconnecting
        self.mount_wait(msg.as_str());
        // Disconnect
        let _ = self.client.disconnect();
        // Quit
        self.exit_reason = Some(super::ExitReason::Disconnect);
    }

    /// ### disconnect_and_quit
    ///
    /// disconnect from remote and then quit
    pub(super) fn disconnect_and_quit(&mut self) {
        self.disconnect();
        self.exit_reason = Some(super::ExitReason::Quit);
    }

    /// ### reload_remote_dir
    ///
    /// Reload remote directory entries and update browser
    pub(super) fn reload_remote_dir(&mut self) {
        // Get current entries
        if let Ok(wrkdir) = self.client.pwd() {
            self.remote_scan(wrkdir.as_path());
            // Set wrkdir
            self.remote_mut().wrkdir = wrkdir;
        }
    }

    /// ### reload_local_dir
    ///
    /// Reload local directory entries and update browser
    pub(super) fn reload_local_dir(&mut self) {
        let wrkdir: PathBuf = self.host.pwd();
        self.local_scan(wrkdir.as_path());
        self.local_mut().wrkdir = wrkdir;
    }

    /// ### local_scan
    ///
    /// Scan current local directory
    fn local_scan(&mut self, path: &Path) {
        match self.host.scan_dir(path) {
            Ok(files) => {
                // Set files and sort (sorting is implicit)
                self.local_mut().set_files(files);
            }
            Err(err) => {
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not scan current directory: {}", err),
                );
            }
        }
    }

    /// ### remote_scan
    ///
    /// Scan current remote directory
    fn remote_scan(&mut self, path: &Path) {
        match self.client.list_dir(path) {
            Ok(files) => {
                // Set files and sort (sorting is implicit)
                self.remote_mut().set_files(files);
            }
            Err(err) => {
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not scan current directory: {}", err),
                );
            }
        }
    }

    /// ### filetransfer_send
    ///
    /// Send fs entry to remote.
    /// If dst_name is Some, entry will be saved with a different name.
    /// If entry is a directory, this applies to directory only
    pub(super) fn filetransfer_send(
        &mut self,
        payload: TransferPayload,
        curr_remote_path: &Path,
        dst_name: Option<String>,
    ) -> Result<(), String> {
        // Use different method based on payload
        match payload {
            TransferPayload::Any(entry) => {
                self.filetransfer_send_any(&entry, curr_remote_path, dst_name)
            }
            TransferPayload::File(file) => {
                self.filetransfer_send_file(&file, curr_remote_path, dst_name)
            }
            TransferPayload::Many(entries) => {
                self.filetransfer_send_many(entries, curr_remote_path)
            }
        }
    }

    /// ### filetransfer_send_file
    ///
    /// Send one file to remote at specified path.
    fn filetransfer_send_file(
        &mut self,
        file: &FsFile,
        curr_remote_path: &Path,
        dst_name: Option<String>,
    ) -> Result<(), String> {
        // Reset states
        self.transfer.reset();
        // Calculate total size of transfer
        let total_transfer_size: usize = file.size;
        self.transfer.full.init(total_transfer_size);
        // Mount progress bar
        self.mount_progress_bar(format!("Uploading {}…", file.abs_path.display()));
        // Get remote path
        let file_name: String = file.name.clone();
        let mut remote_path: PathBuf = PathBuf::from(curr_remote_path);
        let remote_file_name: PathBuf = match dst_name {
            Some(s) => PathBuf::from(s.as_str()),
            None => PathBuf::from(file_name.as_str()),
        };
        remote_path.push(remote_file_name);
        // Send
        let result = self.filetransfer_send_one(file, remote_path.as_path(), file_name);
        // Umount progress bar
        self.umount_progress_bar();
        // Return result
        result.map_err(|x| x.to_string())
    }

    /// ### filetransfer_send_any
    ///
    /// Send a `TransferPayload` of type `Any`
    fn filetransfer_send_any(
        &mut self,
        entry: &FsEntry,
        curr_remote_path: &Path,
        dst_name: Option<String>,
    ) -> Result<(), String> {
        // Reset states
        self.transfer.reset();
        // Calculate total size of transfer
        let total_transfer_size: usize = self.get_total_transfer_size_local(entry);
        self.transfer.full.init(total_transfer_size);
        // Mount progress bar
        self.mount_progress_bar(format!("Uploading {}…", entry.get_abs_path().display()));
        // Send recurse
        self.filetransfer_send_recurse(entry, curr_remote_path, dst_name);
        // Umount progress bar
        self.umount_progress_bar();
        Ok(())
    }

    /// ### filetransfer_send_many
    ///
    /// Send many entries to remote
    fn filetransfer_send_many(
        &mut self,
        entries: Vec<FsEntry>,
        curr_remote_path: &Path,
    ) -> Result<(), String> {
        // Reset states
        self.transfer.reset();
        // Calculate total size of transfer
        let total_transfer_size: usize = entries
            .iter()
            .map(|x| self.get_total_transfer_size_local(x))
            .sum();
        self.transfer.full.init(total_transfer_size);
        // Mount progress bar
        self.mount_progress_bar(format!("Uploading {} entries…", entries.len()));
        // Send recurse
        entries
            .iter()
            .for_each(|x| self.filetransfer_send_recurse(x, curr_remote_path, None));
        // Umount progress bar
        self.umount_progress_bar();
        Ok(())
    }

    fn filetransfer_send_recurse(
        &mut self,
        entry: &FsEntry,
        curr_remote_path: &Path,
        dst_name: Option<String>,
    ) {
        // Write popup
        let file_name: String = match entry {
            FsEntry::Directory(dir) => dir.name.clone(),
            FsEntry::File(file) => file.name.clone(),
        };
        // Get remote path
        let mut remote_path: PathBuf = PathBuf::from(curr_remote_path);
        let remote_file_name: PathBuf = match dst_name {
            Some(s) => PathBuf::from(s.as_str()),
            None => PathBuf::from(file_name.as_str()),
        };
        remote_path.push(remote_file_name);
        // Match entry
        match entry {
            FsEntry::File(file) => {
                if let Err(err) = self.filetransfer_send_one(file, remote_path.as_path(), file_name)
                {
                    // Log error
                    self.log_and_alert(
                        LogLevel::Error,
                        format!("Failed to upload file {}: {}", file.name, err),
                    );
                    // If transfer was abrupted or there was an IO error on remote, remove file
                    if matches!(
                        err,
                        TransferErrorReason::Abrupted | TransferErrorReason::RemoteIoError(_)
                    ) {
                        // Stat file on remote and remove it if exists
                        match self.client.stat(remote_path.as_path()) {
                            Err(err) => self.log(
                                LogLevel::Error,
                                format!(
                                    "Could not remove created file {}: {}",
                                    remote_path.display(),
                                    err
                                ),
                            ),
                            Ok(entry) => {
                                if let Err(err) = self.client.remove(&entry) {
                                    self.log(
                                        LogLevel::Error,
                                        format!(
                                            "Could not remove created file {}: {}",
                                            remote_path.display(),
                                            err
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            FsEntry::Directory(dir) => {
                // Create directory on remote first
                match self.client.mkdir(remote_path.as_path()) {
                    Ok(_) => {
                        self.log(
                            LogLevel::Info,
                            format!("Created directory \"{}\"", remote_path.display()),
                        );
                    }
                    Err(err) if err.kind() == FileTransferErrorType::DirectoryAlreadyExists => {
                        self.log(
                            LogLevel::Info,
                            format!(
                                "Directory \"{}\" already exists on remote",
                                remote_path.display()
                            ),
                        );
                    }
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!(
                                "Failed to create directory \"{}\": {}",
                                remote_path.display(),
                                err
                            ),
                        );
                        return;
                    }
                }
                // Get files in dir
                match self.host.scan_dir(dir.abs_path.as_path()) {
                    Ok(entries) => {
                        // Iterate over files
                        for entry in entries.iter() {
                            // If aborted; break
                            if self.transfer.aborted() {
                                break;
                            }
                            // Send entry; name is always None after first call
                            self.filetransfer_send_recurse(entry, remote_path.as_path(), None);
                        }
                    }
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!(
                                "Could not scan directory \"{}\": {}",
                                dir.abs_path.display(),
                                err
                            ),
                        );
                    }
                }
            }
        }
        // Scan dir on remote
        self.reload_remote_dir();
        // If aborted; show popup
        if self.transfer.aborted() {
            // Log abort
            self.log_and_alert(
                LogLevel::Warn,
                format!("Upload aborted for \"{}\"!", entry.get_abs_path().display()),
            );
        }
    }

    /// ### filetransfer_send_file
    ///
    /// Send local file and write it to remote path
    fn filetransfer_send_one(
        &mut self,
        local: &FsFile,
        remote: &Path,
        file_name: String,
    ) -> Result<(), TransferErrorReason> {
        // Upload file
        // Try to open local file
        match self.host.open_file_read(local.abs_path.as_path()) {
            Ok(mut fhnd) => match self.client.send_file(local, remote) {
                Ok(mut rhnd) => {
                    // Write file
                    let file_size: usize =
                        fhnd.seek(std::io::SeekFrom::End(0)).unwrap_or(0) as usize;
                    // Init transfer
                    self.transfer.partial.init(file_size);
                    // rewind
                    if let Err(err) = fhnd.seek(std::io::SeekFrom::Start(0)) {
                        return Err(TransferErrorReason::CouldNotRewind(err));
                    }
                    // Write remote file
                    let mut total_bytes_written: usize = 0;
                    let mut last_progress_val: f64 = 0.0;
                    let mut last_input_event_fetch: Option<Instant> = None;
                    // While the entire file hasn't been completely written,
                    // Or filetransfer has been aborted
                    while total_bytes_written < file_size && !self.transfer.aborted() {
                        // Handle input events (each 500ms) or if never fetched before
                        if last_input_event_fetch.is_none()
                            || last_input_event_fetch
                                .unwrap_or_else(Instant::now)
                                .elapsed()
                                .as_millis()
                                >= 500
                        {
                            // Read events
                            self.read_input_event();
                            // Reset instant
                            last_input_event_fetch = Some(Instant::now());
                        }
                        // Read till you can
                        let mut buffer: [u8; 65536] = [0; 65536];
                        let delta: usize = match fhnd.read(&mut buffer) {
                            Ok(bytes_read) => {
                                total_bytes_written += bytes_read;
                                if bytes_read == 0 {
                                    continue;
                                } else {
                                    let mut delta: usize = 0;
                                    while delta < bytes_read {
                                        // Write bytes
                                        match rhnd.write(&buffer[delta..bytes_read]) {
                                            Ok(bytes) => {
                                                delta += bytes;
                                            }
                                            Err(err) => {
                                                return Err(TransferErrorReason::RemoteIoError(
                                                    err,
                                                ));
                                            }
                                        }
                                    }
                                    delta
                                }
                            }
                            Err(err) => {
                                return Err(TransferErrorReason::LocalIoError(err));
                            }
                        };
                        // Increase progress
                        self.transfer.partial.update_progress(delta);
                        self.transfer.full.update_progress(delta);
                        // Draw only if a significant progress has been made (performance improvement)
                        if last_progress_val < self.transfer.partial.calc_progress() - 0.01 {
                            // Draw
                            self.update_progress_bar(format!("Uploading \"{}\"…", file_name));
                            self.view();
                            last_progress_val = self.transfer.partial.calc_progress();
                        }
                    }
                    // Finalize stream
                    if let Err(err) = self.client.on_sent(rhnd) {
                        self.log(
                            LogLevel::Warn,
                            format!("Could not finalize remote stream: \"{}\"", err),
                        );
                    }
                    // if upload was abrupted, return error
                    if self.transfer.aborted() {
                        return Err(TransferErrorReason::Abrupted);
                    }
                    self.log(
                        LogLevel::Info,
                        format!(
                            "Saved file \"{}\" to \"{}\" (took {} seconds; at {}/s)",
                            local.abs_path.display(),
                            remote.display(),
                            fmt_millis(self.transfer.partial.started().elapsed()),
                            ByteSize(self.transfer.partial.calc_bytes_per_second()),
                        ),
                    );
                }
                Err(err) => return Err(TransferErrorReason::FileTransferError(err)),
            },
            Err(err) => return Err(TransferErrorReason::HostError(err)),
        }
        Ok(())
    }

    /// ### filetransfer_recv
    ///
    /// Recv fs entry from remote.
    /// If dst_name is Some, entry will be saved with a different name.
    /// If entry is a directory, this applies to directory only
    pub(super) fn filetransfer_recv(
        &mut self,
        payload: TransferPayload,
        local_path: &Path,
        dst_name: Option<String>,
    ) -> Result<(), String> {
        match payload {
            TransferPayload::Any(entry) => self.filetransfer_recv_any(&entry, local_path, dst_name),
            TransferPayload::File(file) => self.filetransfer_recv_file(&file, local_path),
            TransferPayload::Many(entries) => self.filetransfer_recv_many(entries, local_path),
        }
    }

    /// ### filetransfer_recv_any
    ///
    /// Recv fs entry from remote.
    /// If dst_name is Some, entry will be saved with a different name.
    /// If entry is a directory, this applies to directory only
    fn filetransfer_recv_any(
        &mut self,
        entry: &FsEntry,
        local_path: &Path,
        dst_name: Option<String>,
    ) -> Result<(), String> {
        // Reset states
        self.transfer.reset();
        // Calculate total transfer size
        let total_transfer_size: usize = self.get_total_transfer_size_remote(entry);
        self.transfer.full.init(total_transfer_size);
        // Mount progress bar
        self.mount_progress_bar(format!("Downloading {}…", entry.get_abs_path().display()));
        // Receive
        self.filetransfer_recv_recurse(entry, local_path, dst_name);
        // Umount progress bar
        self.umount_progress_bar();
        Ok(())
    }

    /// ### filetransfer_recv_file
    ///
    /// Receive a single file from remote.
    fn filetransfer_recv_file(&mut self, entry: &FsFile, local_path: &Path) -> Result<(), String> {
        // Reset states
        self.transfer.reset();
        // Calculate total transfer size
        let total_transfer_size: usize = entry.size;
        self.transfer.full.init(total_transfer_size);
        // Mount progress bar
        self.mount_progress_bar(format!("Downloading {}…", entry.abs_path.display()));
        // Receive
        let result = self.filetransfer_recv_one(local_path, entry, entry.name.clone());
        // Umount progress bar
        self.umount_progress_bar();
        // Return result
        result.map_err(|x| x.to_string())
    }

    /// ### filetransfer_send_many
    ///
    /// Send many entries to remote
    fn filetransfer_recv_many(
        &mut self,
        entries: Vec<FsEntry>,
        curr_remote_path: &Path,
    ) -> Result<(), String> {
        // Reset states
        self.transfer.reset();
        // Calculate total size of transfer
        let total_transfer_size: usize = entries
            .iter()
            .map(|x| self.get_total_transfer_size_remote(x))
            .sum();
        self.transfer.full.init(total_transfer_size);
        // Mount progress bar
        self.mount_progress_bar(format!("Downloading {} entries…", entries.len()));
        // Send recurse
        entries
            .iter()
            .for_each(|x| self.filetransfer_recv_recurse(x, curr_remote_path, None));
        // Umount progress bar
        self.umount_progress_bar();
        Ok(())
    }

    fn filetransfer_recv_recurse(
        &mut self,
        entry: &FsEntry,
        local_path: &Path,
        dst_name: Option<String>,
    ) {
        // Write popup
        let file_name: String = match entry {
            FsEntry::Directory(dir) => dir.name.clone(),
            FsEntry::File(file) => file.name.clone(),
        };
        // Match entry
        match entry {
            FsEntry::File(file) => {
                // Get local file
                let mut local_file_path: PathBuf = PathBuf::from(local_path);
                let local_file_name: String = match dst_name {
                    Some(n) => n,
                    None => file.name.clone(),
                };
                local_file_path.push(local_file_name.as_str());
                // Download file
                if let Err(err) =
                    self.filetransfer_recv_one(local_file_path.as_path(), file, file_name)
                {
                    self.log_and_alert(
                        LogLevel::Error,
                        format!("Could not download file {}: {}", file.name, err),
                    );
                    // If transfer was abrupted or there was an IO error on remote, remove file
                    if matches!(
                        err,
                        TransferErrorReason::Abrupted | TransferErrorReason::LocalIoError(_)
                    ) {
                        // Stat file
                        match self.host.stat(local_file_path.as_path()) {
                            Err(err) => self.log(
                                LogLevel::Error,
                                format!(
                                    "Could not remove created file {}: {}",
                                    local_file_path.display(),
                                    err
                                ),
                            ),
                            Ok(entry) => {
                                if let Err(err) = self.host.remove(&entry) {
                                    self.log(
                                        LogLevel::Error,
                                        format!(
                                            "Could not remove created file {}: {}",
                                            local_file_path.display(),
                                            err
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            FsEntry::Directory(dir) => {
                // Get dir name
                let mut local_dir_path: PathBuf = PathBuf::from(local_path);
                match dst_name {
                    Some(name) => local_dir_path.push(name),
                    None => local_dir_path.push(dir.name.as_str()),
                }
                // Create directory on local
                match self.host.mkdir_ex(local_dir_path.as_path(), true) {
                    Ok(_) => {
                        // Apply file mode to directory
                        #[cfg(any(
                            target_family = "unix",
                            target_os = "macos",
                            target_os = "linux"
                        ))]
                        if let Some((owner, group, others)) = dir.unix_pex {
                            if let Err(err) = self.host.chmod(
                                local_dir_path.as_path(),
                                (owner.as_byte(), group.as_byte(), others.as_byte()),
                            ) {
                                self.log(
                                    LogLevel::Error,
                                    format!(
                                        "Could not apply file mode {:?} to \"{}\": {}",
                                        (owner.as_byte(), group.as_byte(), others.as_byte()),
                                        local_dir_path.display(),
                                        err
                                    ),
                                );
                            }
                        }
                        self.log(
                            LogLevel::Info,
                            format!("Created directory \"{}\"", local_dir_path.display()),
                        );
                        // Get files in dir
                        match self.client.list_dir(dir.abs_path.as_path()) {
                            Ok(entries) => {
                                // Iterate over files
                                for entry in entries.iter() {
                                    // If transfer has been aborted; break
                                    if self.transfer.aborted() {
                                        break;
                                    }
                                    // Receive entry; name is always None after first call
                                    // Local path becomes local_dir_path
                                    self.filetransfer_recv_recurse(
                                        entry,
                                        local_dir_path.as_path(),
                                        None,
                                    );
                                }
                            }
                            Err(err) => {
                                self.log_and_alert(
                                    LogLevel::Error,
                                    format!(
                                        "Could not scan directory \"{}\": {}",
                                        dir.abs_path.display(),
                                        err
                                    ),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        self.log(
                            LogLevel::Error,
                            format!(
                                "Failed to create directory \"{}\": {}",
                                local_dir_path.display(),
                                err
                            ),
                        );
                    }
                }
            }
        }
        // Reload directory on local
        self.reload_local_dir();
        // if aborted; show alert
        if self.transfer.aborted() {
            // Log abort
            self.log_and_alert(
                LogLevel::Warn,
                format!(
                    "Download aborted for \"{}\"!",
                    entry.get_abs_path().display()
                ),
            );
        }
    }

    /// ### filetransfer_recv_one
    ///
    /// Receive file from remote and write it to local path
    fn filetransfer_recv_one(
        &mut self,
        local: &Path,
        remote: &FsFile,
        file_name: String,
    ) -> Result<(), TransferErrorReason> {
        // Try to open local file
        match self.host.open_file_write(local) {
            Ok(mut local_file) => {
                // Download file from remote
                match self.client.recv_file(remote) {
                    Ok(mut rhnd) => {
                        let mut total_bytes_written: usize = 0;
                        // Init transfer
                        self.transfer.partial.init(remote.size);
                        // Write local file
                        let mut last_progress_val: f64 = 0.0;
                        let mut last_input_event_fetch: Option<Instant> = None;
                        // While the entire file hasn't been completely read,
                        // Or filetransfer has been aborted
                        while total_bytes_written < remote.size && !self.transfer.aborted() {
                            // Handle input events (each 500 ms) or is None
                            if last_input_event_fetch.is_none()
                                || last_input_event_fetch
                                    .unwrap_or_else(Instant::now)
                                    .elapsed()
                                    .as_millis()
                                    >= 500
                            {
                                // Read events
                                self.read_input_event();
                                // Reset instant
                                last_input_event_fetch = Some(Instant::now());
                            }
                            // Read till you can
                            let mut buffer: [u8; 65536] = [0; 65536];
                            let delta: usize = match rhnd.read(&mut buffer) {
                                Ok(bytes_read) => {
                                    total_bytes_written += bytes_read;
                                    if bytes_read == 0 {
                                        continue;
                                    } else {
                                        let mut delta: usize = 0;
                                        while delta < bytes_read {
                                            // Write bytes
                                            match local_file.write(&buffer[delta..bytes_read]) {
                                                Ok(bytes) => delta += bytes,
                                                Err(err) => {
                                                    return Err(TransferErrorReason::LocalIoError(
                                                        err,
                                                    ));
                                                }
                                            }
                                        }
                                        delta
                                    }
                                }
                                Err(err) => {
                                    return Err(TransferErrorReason::RemoteIoError(err));
                                }
                            };
                            // Set progress
                            self.transfer.partial.update_progress(delta);
                            self.transfer.full.update_progress(delta);
                            // Draw only if a significant progress has been made (performance improvement)
                            if last_progress_val < self.transfer.partial.calc_progress() - 0.01 {
                                // Draw
                                self.update_progress_bar(format!("Downloading \"{}\"", file_name));
                                self.view();
                                last_progress_val = self.transfer.partial.calc_progress();
                            }
                        }
                        // Finalize stream
                        if let Err(err) = self.client.on_recv(rhnd) {
                            self.log(
                                LogLevel::Warn,
                                format!("Could not finalize remote stream: \"{}\"", err),
                            );
                        }
                        // If download was abrupted, return Error
                        if self.transfer.aborted() {
                            return Err(TransferErrorReason::Abrupted);
                        }
                        // Apply file mode to file
                        #[cfg(any(
                            target_family = "unix",
                            target_os = "macos",
                            target_os = "linux"
                        ))]
                        if let Some((owner, group, others)) = remote.unix_pex {
                            if let Err(err) = self
                                .host
                                .chmod(local, (owner.as_byte(), group.as_byte(), others.as_byte()))
                            {
                                self.log(
                                    LogLevel::Error,
                                    format!(
                                        "Could not apply file mode {:?} to \"{}\": {}",
                                        (owner.as_byte(), group.as_byte(), others.as_byte()),
                                        local.display(),
                                        err
                                    ),
                                );
                            }
                        }
                        // Log
                        self.log(
                            LogLevel::Info,
                            format!(
                                "Saved file \"{}\" to \"{}\" (took {} seconds; at {}/s)",
                                remote.abs_path.display(),
                                local.display(),
                                fmt_millis(self.transfer.partial.started().elapsed()),
                                ByteSize(self.transfer.partial.calc_bytes_per_second()),
                            ),
                        );
                    }
                    Err(err) => return Err(TransferErrorReason::FileTransferError(err)),
                }
            }
            Err(err) => return Err(TransferErrorReason::HostError(err)),
        }
        Ok(())
    }

    /// ### local_changedir
    ///
    /// Change directory for local
    pub(super) fn local_changedir(&mut self, path: &Path, push: bool) {
        // Get current directory
        let prev_dir: PathBuf = self.local().wrkdir.clone();
        // Change directory
        match self.host.change_wrkdir(path) {
            Ok(_) => {
                self.log(
                    LogLevel::Info,
                    format!("Changed directory on local: {}", path.display()),
                );
                // Reload files
                self.reload_local_dir();
                // Push prev_dir to stack
                if push {
                    self.local_mut().pushd(prev_dir.as_path())
                }
            }
            Err(err) => {
                // Report err
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not change working directory: {}", err),
                );
            }
        }
    }

    pub(super) fn remote_changedir(&mut self, path: &Path, push: bool) {
        // Get current directory
        let prev_dir: PathBuf = self.remote().wrkdir.clone();
        // Change directory
        match self.client.as_mut().change_dir(path) {
            Ok(_) => {
                self.log(
                    LogLevel::Info,
                    format!("Changed directory on remote: {}", path.display()),
                );
                // Update files
                self.reload_remote_dir();
                // Push prev_dir to stack
                if push {
                    self.remote_mut().pushd(prev_dir.as_path())
                }
            }
            Err(err) => {
                // Report err
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not change working directory: {}", err),
                );
            }
        }
    }

    /// ### download_file_as_temp
    ///
    /// Download provided file as a temporary file
    pub(super) fn download_file_as_temp(&mut self, file: &FsFile) -> Result<PathBuf, String> {
        let tmpfile: PathBuf = match self.cache.as_ref() {
            Some(cache) => {
                let mut p: PathBuf = cache.path().to_path_buf();
                p.push(file.name.as_str());
                p
            }
            None => {
                return Err(String::from(
                    "Could not create tempfile: cache not available",
                ))
            }
        };
        // Download file
        match self.filetransfer_recv(
            TransferPayload::File(file.clone()),
            tmpfile.as_path(),
            Some(file.name.clone()),
        ) {
            Err(err) => Err(format!(
                "Could not download {} to temporary file: {}",
                file.abs_path.display(),
                err
            )),
            Ok(()) => Ok(tmpfile),
        }
    }

    // -- transfer sizes

    /// ### get_total_transfer_size_local
    ///
    /// Get total size of transfer for localhost
    fn get_total_transfer_size_local(&mut self, entry: &FsEntry) -> usize {
        match entry {
            FsEntry::File(file) => file.size,
            FsEntry::Directory(dir) => {
                // List dir
                match self.host.scan_dir(dir.abs_path.as_path()) {
                    Ok(files) => files
                        .iter()
                        .map(|x| self.get_total_transfer_size_local(x))
                        .sum(),
                    Err(err) => {
                        self.log(
                            LogLevel::Error,
                            format!(
                                "Could not list directory {}: {}",
                                dir.abs_path.display(),
                                err
                            ),
                        );
                        0
                    }
                }
            }
        }
    }

    /// ### get_total_transfer_size_remote
    ///
    /// Get total size of transfer for remote host
    fn get_total_transfer_size_remote(&mut self, entry: &FsEntry) -> usize {
        match entry {
            FsEntry::File(file) => file.size,
            FsEntry::Directory(dir) => {
                // List directory
                match self.client.list_dir(dir.abs_path.as_path()) {
                    Ok(files) => files
                        .iter()
                        .map(|x| self.get_total_transfer_size_remote(x))
                        .sum(),
                    Err(err) => {
                        self.log(
                            LogLevel::Error,
                            format!(
                                "Could not list directory {}: {}",
                                dir.abs_path.display(),
                                err
                            ),
                        );
                        0
                    }
                }
            }
        }
    }
}
