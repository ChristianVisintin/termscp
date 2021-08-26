//! ## SFTP_Transfer
//!
//! `sftp_transfer` is the module which provides the implementation for the SFTP file transfer

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
use super::{FileTransfer, FileTransferError, FileTransferErrorType};
use crate::fs::{FsDirectory, FsEntry, FsFile, UnixPex};
use crate::system::sshkey_storage::SshKeyStorage;
use crate::utils::fmt::{fmt_time, shadow_password};

// Includes
use ssh2::{Channel, FileStat, OpenFlags, OpenType, Session, Sftp};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// ## SftpFileTransfer
///
/// SFTP file transfer structure
pub struct SftpFileTransfer {
    session: Option<Session>,
    sftp: Option<Sftp>,
    wrkdir: PathBuf,
    key_storage: SshKeyStorage,
}

impl SftpFileTransfer {
    /// ### new
    ///
    /// Instantiates a new SftpFileTransfer
    pub fn new(key_storage: SshKeyStorage) -> SftpFileTransfer {
        SftpFileTransfer {
            session: None,
            sftp: None,
            wrkdir: PathBuf::from("~"),
            key_storage,
        }
    }

    /// ### get_abs_path
    ///
    /// Get absolute path from path argument and check if it exists
    fn get_remote_path(&self, p: &Path) -> Result<PathBuf, FileTransferError> {
        match p.is_relative() {
            true => {
                let mut root: PathBuf = self.wrkdir.clone();
                root.push(p);
                match self.sftp.as_ref().unwrap().realpath(root.as_path()) {
                    Ok(p) => match self.sftp.as_ref().unwrap().stat(p.as_path()) {
                        Ok(_) => Ok(p),
                        Err(err) => Err(FileTransferError::new_ex(
                            FileTransferErrorType::NoSuchFileOrDirectory,
                            err.to_string(),
                        )),
                    },
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::NoSuchFileOrDirectory,
                        err.to_string(),
                    )),
                }
            }
            false => match self.sftp.as_ref().unwrap().realpath(p) {
                Ok(p) => match self.sftp.as_ref().unwrap().stat(p.as_path()) {
                    Ok(_) => Ok(p),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::NoSuchFileOrDirectory,
                        err.to_string(),
                    )),
                },
                Err(_) => Err(FileTransferError::new(
                    FileTransferErrorType::NoSuchFileOrDirectory,
                )),
            },
        }
    }

    /// ### get_abs_path
    ///
    /// Returns absolute path on remote, but without errors
    fn get_abs_path(&self, p: &Path) -> PathBuf {
        match p.is_relative() {
            true => {
                let mut root: PathBuf = self.wrkdir.clone();
                root.push(p);
                match self.sftp.as_ref().unwrap().realpath(root.as_path()) {
                    Ok(p) => p,
                    Err(_) => root,
                }
            }
            false => PathBuf::from(p),
        }
    }

    /// ### make_fsentry
    ///
    /// Make fsentry from path and metadata
    fn make_fsentry(&mut self, path: &Path, metadata: &FileStat) -> FsEntry {
        // Get common parameters
        let file_name: String = String::from(path.file_name().unwrap().to_str().unwrap_or(""));
        let file_type: Option<String> = path
            .extension()
            .map(|ext| String::from(ext.to_str().unwrap_or("")));
        let uid: Option<u32> = metadata.uid;
        let gid: Option<u32> = metadata.gid;
        let pex: Option<(UnixPex, UnixPex, UnixPex)> = metadata.perm.map(|x| {
            (
                UnixPex::from(((x >> 6) & 0x7) as u8),
                UnixPex::from(((x >> 3) & 0x7) as u8),
                UnixPex::from((x & 0x7) as u8),
            )
        });
        let size: u64 = metadata.size.unwrap_or(0);
        let mut atime: SystemTime = SystemTime::UNIX_EPOCH;
        atime = atime
            .checked_add(Duration::from_secs(metadata.atime.unwrap_or(0)))
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let mut mtime: SystemTime = SystemTime::UNIX_EPOCH;
        mtime = mtime
            .checked_add(Duration::from_secs(metadata.mtime.unwrap_or(0)))
            .unwrap_or(SystemTime::UNIX_EPOCH);
        // Check if symlink
        let is_symlink: bool = metadata.file_type().is_symlink();
        let symlink: Option<Box<FsEntry>> = match is_symlink {
            true => {
                // Read symlink
                match self.sftp.as_ref().unwrap().readlink(path) {
                    Ok(p) => match self.stat(p.as_path()) {
                        Ok(entry) => Some(Box::new(entry)),
                        Err(_) => None, // Ignore errors
                    },
                    Err(_) => None,
                }
            }
            false => None,
        };
        debug!("Follows {} attributes", path.display());
        debug!("Is directory? {}", metadata.is_dir());
        debug!("Is symlink? {}", is_symlink);
        debug!("name: {}", file_name);
        debug!("abs_path: {}", path.display());
        debug!("last_change_time: {}", fmt_time(mtime, "%Y-%m-%dT%H:%M:%S"));
        debug!("last_access_time: {}", fmt_time(mtime, "%Y-%m-%dT%H:%M:%S"));
        debug!("creation_time: {}", fmt_time(mtime, "%Y-%m-%dT%H:%M:%S"));
        debug!("symlink: {:?}", symlink);
        debug!("user: {:?}", uid);
        debug!("group: {:?}", gid);
        debug!("unix_pex: {:?}", pex);
        debug!("---------------------------------------");
        // Is a directory?
        match metadata.is_dir() {
            true => FsEntry::Directory(FsDirectory {
                name: file_name,
                abs_path: PathBuf::from(path),
                last_change_time: mtime,
                last_access_time: atime,
                creation_time: SystemTime::UNIX_EPOCH,
                symlink,
                user: uid,
                group: gid,
                unix_pex: pex,
            }),
            false => FsEntry::File(FsFile {
                name: file_name,
                abs_path: PathBuf::from(path),
                size: size as usize,
                ftype: file_type,
                last_change_time: mtime,
                last_access_time: atime,
                creation_time: SystemTime::UNIX_EPOCH,
                symlink,
                user: uid,
                group: gid,
                unix_pex: pex,
            }),
        }
    }

    /// ### perform_shell_cmd_with
    ///
    /// Perform a shell command, but change directory to specified path first
    fn perform_shell_cmd_with_path(&mut self, cmd: &str) -> Result<String, FileTransferError> {
        self.perform_shell_cmd(format!("cd \"{}\"; {}", self.wrkdir.display(), cmd).as_str())
    }

    /// ### perform_shell_cmd
    ///
    /// Perform a shell command and read the output from shell
    /// This operation is, obviously, blocking.
    fn perform_shell_cmd(&mut self, cmd: &str) -> Result<String, FileTransferError> {
        match self.session.as_mut() {
            Some(session) => {
                // Create channel
                debug!("Running command: {}", cmd);
                let mut channel: Channel = match session.channel_session() {
                    Ok(ch) => ch,
                    Err(err) => {
                        return Err(FileTransferError::new_ex(
                            FileTransferErrorType::ProtocolError,
                            format!("Could not open channel: {}", err),
                        ))
                    }
                };
                // Execute command
                if let Err(err) = channel.exec(cmd) {
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("Could not execute command \"{}\": {}", cmd, err),
                    ));
                }
                // Read output
                let mut output: String = String::new();
                match channel.read_to_string(&mut output) {
                    Ok(_) => {
                        // Wait close
                        let _ = channel.wait_close();
                        debug!("Command output: {}", output);
                        Ok(output)
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        format!("Could not read output: {}", err),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }
}

impl FileTransfer for SftpFileTransfer {
    /// ### connect
    ///
    /// Connect to the remote server
    fn connect(
        &mut self,
        address: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Option<String>, FileTransferError> {
        // Setup tcp stream
        info!("Connecting to {}:{}", address, port);
        let socket_addresses: Vec<SocketAddr> =
            match format!("{}:{}", address, port).to_socket_addrs() {
                Ok(s) => s.collect(),
                Err(err) => {
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::BadAddress,
                        err.to_string(),
                    ))
                }
            };
        let mut tcp: Option<TcpStream> = None;
        // Try addresses
        for socket_addr in socket_addresses.iter() {
            debug!("Trying socket address {}", socket_addr);
            match TcpStream::connect_timeout(socket_addr, Duration::from_secs(30)) {
                Ok(stream) => {
                    tcp = Some(stream);
                    break;
                }
                Err(_) => continue,
            }
        }
        // If stream is None, return connection timeout
        let tcp: TcpStream = match tcp {
            Some(t) => t,
            None => {
                error!("No suitable socket address found; connection timeout");
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    String::from("Connection timeout"),
                ));
            }
        };
        // Create session
        let mut session: Session = match Session::new() {
            Ok(s) => s,
            Err(err) => {
                error!("Could not create session: {}", err);
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    err.to_string(),
                ));
            }
        };
        // Set TCP stream
        session.set_tcp_stream(tcp);
        // Open connection
        debug!("Initializing handshake");
        if let Err(err) = session.handshake() {
            error!("Handshake failed: {}", err);
            return Err(FileTransferError::new_ex(
                FileTransferErrorType::ConnectionError,
                err.to_string(),
            ));
        }
        let username: String = match username {
            Some(u) => u,
            None => String::from(""),
        };
        // Check if it is possible to authenticate using a RSA key
        match self
            .key_storage
            .resolve(address.as_str(), username.as_str())
        {
            Some(rsa_key) => {
                debug!(
                    "Authenticating with user {} and RSA key {}",
                    username,
                    rsa_key.display()
                );
                // Authenticate with RSA key
                if let Err(err) = session.userauth_pubkey_file(
                    username.as_str(),
                    None,
                    rsa_key.as_path(),
                    password.as_deref(),
                ) {
                    error!("Authentication failed: {}", err);
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::AuthenticationFailed,
                        err.to_string(),
                    ));
                }
            }
            None => {
                // Proceeed with username/password authentication
                debug!(
                    "Authenticating with username {} and password {}",
                    username,
                    shadow_password(password.as_deref().unwrap_or(""))
                );
                if let Err(err) = session.userauth_password(
                    username.as_str(),
                    password.unwrap_or_else(|| String::from("")).as_str(),
                ) {
                    error!("Authentication failed: {}", err);
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::AuthenticationFailed,
                        err.to_string(),
                    ));
                }
            }
        }
        // Set blocking to true
        session.set_blocking(true);
        // Get Sftp client
        debug!("Getting SFTP client...");
        let sftp: Sftp = match session.sftp() {
            Ok(s) => s,
            Err(err) => {
                error!("Could not get sftp client: {}", err);
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::ProtocolError,
                    err.to_string(),
                ));
            }
        };
        // Get working directory
        debug!("Getting working directory...");
        self.wrkdir = match sftp.realpath(PathBuf::from(".").as_path()) {
            Ok(p) => p,
            Err(err) => {
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::ProtocolError,
                    err.to_string(),
                ))
            }
        };
        // Set session
        let banner: Option<String> = session.banner().map(String::from);
        self.session = Some(session);
        // Set sftp
        self.sftp = Some(sftp);
        info!(
            "Connection established: {}; working directory {}",
            banner.as_deref().unwrap_or(""),
            self.wrkdir.display()
        );
        Ok(banner)
    }

    /// ### disconnect
    ///
    /// Disconnect from the remote server
    fn disconnect(&mut self) -> Result<(), FileTransferError> {
        info!("Disconnecting from remote...");
        match self.session.as_ref() {
            Some(session) => {
                // Disconnect (greet server with 'Mandi' as they do in Friuli)
                match session.disconnect(None, "Mandi!", None) {
                    Ok(()) => {
                        // Set session and sftp to none
                        self.session = None;
                        self.sftp = None;
                        Ok(())
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ConnectionError,
                        err.to_string(),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### is_connected
    ///
    /// Indicates whether the client is connected to remote
    fn is_connected(&self) -> bool {
        self.session.is_some()
    }

    /// ### pwd
    ///
    /// Print working directory
    fn pwd(&mut self) -> Result<PathBuf, FileTransferError> {
        info!("PWD: {}", self.wrkdir.display());
        match self.sftp {
            Some(_) => Ok(self.wrkdir.clone()),
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### change_dir
    ///
    /// Change working directory
    fn change_dir(&mut self, dir: &Path) -> Result<PathBuf, FileTransferError> {
        match self.sftp.as_ref() {
            Some(_) => {
                // Change working directory
                self.wrkdir = self.get_remote_path(dir)?;
                info!("Changed working directory to {}", self.wrkdir.display());
                Ok(self.wrkdir.clone())
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### copy
    ///
    /// Copy file to destination
    fn copy(&mut self, src: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        // NOTE: use SCP command to perform copy (UNSAFE)
        match self.is_connected() {
            true => {
                let dst: PathBuf = self.get_abs_path(dst);
                info!(
                    "Copying {} to {}",
                    src.get_abs_path().display(),
                    dst.display()
                );
                // Run `cp -rf`
                match self.perform_shell_cmd_with_path(
                    format!(
                        "cp -rf \"{}\" \"{}\"; echo $?",
                        src.get_abs_path().display(),
                        dst.display()
                    )
                    .as_str(),
                ) {
                    Ok(output) =>
                    // Check if output is 0
                    {
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // File copied
                            false => Err(FileTransferError::new_ex(
                                // Could not copy file
                                FileTransferErrorType::FileCreateDenied,
                                format!("\"{}\"", dst.display()),
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        err.to_string(),
                    )),
                }
            }
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### list_dir
    ///
    /// List directory entries
    fn list_dir(&mut self, path: &Path) -> Result<Vec<FsEntry>, FileTransferError> {
        match self.sftp.as_ref() {
            Some(sftp) => {
                // Get path
                let dir: PathBuf = self.get_remote_path(path)?;
                info!("Getting file entries in {}", path.display());
                // Get files
                match sftp.readdir(dir.as_path()) {
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::DirStatFailed,
                        err.to_string(),
                    )),
                    Ok(files) => {
                        // Allocate vector
                        let mut entries: Vec<FsEntry> = Vec::with_capacity(files.len());
                        // Iterate over files
                        for (path, metadata) in files {
                            entries.push(self.make_fsentry(path.as_path(), &metadata));
                        }
                        Ok(entries)
                    }
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### mkdir
    ///
    /// Make directory
    /// In case the directory already exists, it must return an Error of kind `FileTransferErrorType::DirectoryAlreadyExists`
    fn mkdir(&mut self, dir: &Path) -> Result<(), FileTransferError> {
        match self.sftp.as_ref() {
            Some(sftp) => {
                // Make directory
                let path: PathBuf = self.get_abs_path(PathBuf::from(dir).as_path());
                // If directory already exists, return Err
                if sftp.stat(path.as_path()).is_ok() {
                    error!("Directory {} already exists", path.display());
                    return Err(FileTransferError::new(
                        FileTransferErrorType::DirectoryAlreadyExists,
                    ));
                }
                info!("Making directory {}", path.display());
                match sftp.mkdir(path.as_path(), 0o775) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::FileCreateDenied,
                        err.to_string(),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### remove
    ///
    /// Remove a file or a directory
    fn remove(&mut self, file: &FsEntry) -> Result<(), FileTransferError> {
        if self.sftp.is_none() {
            return Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            ));
        }
        // Match if file is a file or a directory
        info!("Removing file {}", file.get_abs_path().display());
        match file {
            FsEntry::File(f) => {
                // Remove file
                match self.sftp.as_ref().unwrap().unlink(f.abs_path.as_path()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::PexError,
                        err.to_string(),
                    )),
                }
            }
            FsEntry::Directory(d) => {
                // Remove recursively
                debug!("{} is a directory; removing all directory entries", d.name);
                // Get directory files
                let directory_content: Vec<FsEntry> = self.list_dir(d.abs_path.as_path())?;
                for entry in directory_content.iter() {
                    if let Err(err) = self.remove(entry) {
                        return Err(err);
                    }
                }
                // Finally remove directory
                match self.sftp.as_ref().unwrap().rmdir(d.abs_path.as_path()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::PexError,
                        err.to_string(),
                    )),
                }
            }
        }
    }

    /// ### rename
    ///
    /// Rename file or a directory
    fn rename(&mut self, file: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        match self.sftp.as_ref() {
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
            Some(sftp) => {
                info!(
                    "Moving {} to {}",
                    file.get_abs_path().display(),
                    dst.display()
                );
                // Resolve destination path
                let abs_dst: PathBuf = self.get_abs_path(dst);
                // Get abs path of entry
                let abs_src: PathBuf = file.get_abs_path();
                match sftp.rename(abs_src.as_path(), abs_dst.as_path(), None) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::FileCreateDenied,
                        err.to_string(),
                    )),
                }
            }
        }
    }

    /// ### stat
    ///
    /// Stat file and return FsEntry
    fn stat(&mut self, path: &Path) -> Result<FsEntry, FileTransferError> {
        match self.sftp.as_ref() {
            Some(sftp) => {
                // Get path
                let dir: PathBuf = self.get_remote_path(path)?;
                info!("Stat file {}", dir.display());
                // Get file
                match sftp.stat(dir.as_path()) {
                    Ok(metadata) => Ok(self.make_fsentry(dir.as_path(), &metadata)),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::NoSuchFileOrDirectory,
                        err.to_string(),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### exec
    ///
    /// Execute a command on remote host
    fn exec(&mut self, cmd: &str) -> Result<String, FileTransferError> {
        info!("Executing command {}", cmd);
        match self.is_connected() {
            true => match self.perform_shell_cmd_with_path(cmd) {
                Ok(output) => Ok(output),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ProtocolError,
                    err.to_string(),
                )),
            },
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### send_file
    ///
    /// Send file to remote
    /// File name is referred to the name of the file as it will be saved
    /// Data contains the file data
    fn send_file(
        &mut self,
        local: &FsFile,
        file_name: &Path,
    ) -> Result<Box<dyn Write>, FileTransferError> {
        match self.sftp.as_ref() {
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
            Some(sftp) => {
                let remote_path: PathBuf = self.get_abs_path(file_name);
                info!(
                    "Sending file {} to {}",
                    local.abs_path.display(),
                    remote_path.display()
                );
                // Calculate file mode
                let mode: i32 = match local.unix_pex {
                    None => 0o644,
                    Some((u, g, o)) => {
                        ((u.as_byte() as i32) << 6)
                            + ((g.as_byte() as i32) << 3)
                            + (o.as_byte() as i32)
                    }
                };
                debug!("File mode {:?}", mode);
                match sftp.open_mode(
                    remote_path.as_path(),
                    OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::TRUNCATE,
                    mode,
                    OpenType::File,
                ) {
                    Ok(file) => Ok(Box::new(BufWriter::with_capacity(65536, file))),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::FileCreateDenied,
                        err.to_string(),
                    )),
                }
            }
        }
    }

    /// ### recv_file
    ///
    /// Receive file from remote with provided name
    fn recv_file(&mut self, file: &FsFile) -> Result<Box<dyn Read>, FileTransferError> {
        match self.sftp.as_ref() {
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
            Some(sftp) => {
                // Get remote file name
                let remote_path: PathBuf = self.get_remote_path(file.abs_path.as_path())?;
                info!("Receiving file {}", remote_path.display());
                // Open remote file
                match sftp.open(remote_path.as_path()) {
                    Ok(file) => Ok(Box::new(BufReader::with_capacity(65536, file))),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::NoSuchFileOrDirectory,
                        err.to_string(),
                    )),
                }
            }
        }
    }

    /// ### on_sent
    ///
    /// Finalize send method. This method must be implemented only if necessary.
    /// The purpose of this method is to finalize the connection with the peer when writing data.
    /// This is necessary for some protocols such as FTP.
    /// You must call this method each time you want to finalize the write of the remote file.
    fn on_sent(&mut self, _writable: Box<dyn Write>) -> Result<(), FileTransferError> {
        Ok(())
    }

    /// ### on_recv
    ///
    /// Finalize recv method. This method must be implemented only if necessary.
    /// The purpose of this method is to finalize the connection with the peer when reading data.
    /// This mighe be necessary for some protocols.
    /// You must call this method each time you want to finalize the read of the remote file.
    fn on_recv(&mut self, _readable: Box<dyn Read>) -> Result<(), FileTransferError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::test_helpers::make_fsentry;
    #[cfg(feature = "with-containers")]
    use crate::utils::test_helpers::{create_sample_file_entry, write_file, write_ssh_key};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_filetransfer_sftp_new() {
        let client: SftpFileTransfer = SftpFileTransfer::new(SshKeyStorage::empty());
        assert!(client.session.is_none());
        assert!(client.sftp.is_none());
        assert_eq!(client.wrkdir, PathBuf::from("~"));
        assert_eq!(client.is_connected(), false);
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_sftp_server() {
        let mut client: SftpFileTransfer = SftpFileTransfer::new(SshKeyStorage::empty());
        // Sample file
        let (entry, file): (FsFile, tempfile::NamedTempFile) = create_sample_file_entry();
        // Connect
        assert!(client
            .connect(
                String::from("127.0.0.1"),
                10022,
                Some(String::from("sftp")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and sftp
        assert!(client.session.is_some());
        assert!(client.sftp.is_some());
        assert_eq!(client.wrkdir, PathBuf::from("/config"));
        assert_eq!(client.is_connected(), true);
        // Pwd
        assert_eq!(client.wrkdir.clone(), client.pwd().ok().unwrap());
        // Stat
        let stat: FsFile = client
            .stat(PathBuf::from("/config/sshd.pid").as_path())
            .ok()
            .unwrap()
            .unwrap_file();
        assert_eq!(stat.name.as_str(), "sshd.pid");
        let stat: FsDirectory = client
            .stat(PathBuf::from("/config").as_path())
            .ok()
            .unwrap()
            .unwrap_dir();
        assert_eq!(stat.name.as_str(), "config");
        // Stat (err)
        assert!(client
            .stat(PathBuf::from("/config/5t0ca220.log").as_path())
            .is_err());
        // List dir (dir has 4 (one is hidden :D) entries)
        assert!(client.list_dir(&Path::new("/config")).unwrap().len() >= 4);
        // Make directory
        assert!(client.mkdir(PathBuf::from("/tmp/omar").as_path()).is_ok());
        // Remake directory (should report already exists)
        assert_eq!(
            client
                .mkdir(PathBuf::from("/tmp/omar").as_path())
                .err()
                .unwrap()
                .kind(),
            FileTransferErrorType::DirectoryAlreadyExists
        );
        // Make directory (err)
        assert!(client
            .mkdir(PathBuf::from("/root/aaaaa/pommlar").as_path())
            .is_err());
        // Change directory
        assert!(client
            .change_dir(PathBuf::from("/tmp/omar").as_path())
            .is_ok());
        // Change directory (err)
        assert!(client
            .change_dir(PathBuf::from("/tmp/oooo/aaaa/eee").as_path())
            .is_err());
        // Copy (not supported)
        assert!(client
            .copy(&FsEntry::File(entry.clone()), PathBuf::from("/").as_path())
            .is_err());
        // Exec
        assert_eq!(client.exec("echo 5").ok().unwrap().as_str(), "5\n");
        // Upload 2 files
        let mut writable = client
            .send_file(&entry, PathBuf::from("omar.txt").as_path())
            .ok()
            .unwrap();
        write_file(&file, &mut writable);
        assert!(client.on_sent(writable).is_ok());
        let mut writable = client
            .send_file(&entry, PathBuf::from("README.md").as_path())
            .ok()
            .unwrap();
        write_file(&file, &mut writable);
        assert!(client.on_sent(writable).is_ok());
        // Upload file (err)
        assert!(client
            .send_file(&entry, PathBuf::from("/ommlar/omarone").as_path())
            .is_err());
        // List dir
        let list: Vec<FsEntry> = client
            .list_dir(PathBuf::from("/tmp/omar").as_path())
            .ok()
            .unwrap();
        assert_eq!(list.len(), 2);
        // Find
        assert_eq!(client.find("*.txt").ok().unwrap().len(), 1);
        assert_eq!(client.find("*.md").ok().unwrap().len(), 1);
        assert_eq!(client.find("*.jpeg").ok().unwrap().len(), 0);
        // Rename
        assert!(client
            .mkdir(PathBuf::from("/tmp/uploads").as_path())
            .is_ok());
        assert!(client
            .rename(
                list.get(0).unwrap(),
                PathBuf::from("/tmp/uploads/README.txt").as_path()
            )
            .is_ok());
        // Rename (err)
        assert!(client
            .rename(list.get(0).unwrap(), PathBuf::from("OMARONE").as_path())
            .is_err());
        let dummy: FsEntry = FsEntry::File(FsFile {
            name: String::from("cucumber.txt"),
            abs_path: PathBuf::from("/cucumber.txt"),
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            size: 0,
            ftype: Some(String::from("txt")), // File type
            symlink: None,                    // UNIX only
            user: Some(0),                    // UNIX only
            group: Some(0),                   // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        assert!(client
            .rename(&dummy, PathBuf::from("/a/b/c").as_path())
            .is_err());
        // Remove
        assert!(client.remove(list.get(1).unwrap()).is_ok());
        assert!(client.remove(list.get(1).unwrap()).is_err());
        // Receive file
        let mut writable = client
            .send_file(&entry, PathBuf::from("/tmp/uploads/README.txt").as_path())
            .ok()
            .unwrap();
        write_file(&file, &mut writable);
        assert!(client.on_sent(writable).is_ok());
        let file: FsFile = client
            .list_dir(PathBuf::from("/tmp/uploads").as_path())
            .ok()
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
            .unwrap_file();
        let mut readable = client.recv_file(&file).ok().unwrap();
        let mut data: Vec<u8> = vec![0; 1024];
        assert!(readable.read(&mut data).is_ok());
        assert!(client.on_recv(readable).is_ok());
        // Receive file (err)
        assert!(client.recv_file(&entry).is_err());
        // Cleanup
        assert!(client.change_dir(PathBuf::from("/").as_path()).is_ok());
        assert!(client
            .remove(&make_fsentry(PathBuf::from("/tmp/omar"), true))
            .is_ok());
        assert!(client
            .remove(&make_fsentry(PathBuf::from("/tmp/uploads"), true))
            .is_ok());
        // Disconnect
        assert!(client.disconnect().is_ok());
        assert_eq!(client.is_connected(), false);
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_sftp_ssh_storage() {
        let mut storage: SshKeyStorage = SshKeyStorage::empty();
        let key_file: tempfile::NamedTempFile = write_ssh_key();
        storage.add_key("127.0.0.1", "sftp", key_file.path().to_path_buf());
        let mut client: SftpFileTransfer = SftpFileTransfer::new(storage);
        // Connect
        assert!(client
            .connect(
                String::from("127.0.0.1"),
                10022,
                Some(String::from("sftp")),
                None,
            )
            .is_ok());
        assert_eq!(client.is_connected(), true);
        assert!(client.disconnect().is_ok());
    }

    #[test]
    fn test_filetransfer_sftp_bad_auth() {
        let mut client: SftpFileTransfer = SftpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("127.0.0.1"),
                10022,
                Some(String::from("demo")),
                Some(String::from("badpassword"))
            )
            .is_err());
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_sftp_no_credentials() {
        let mut client: SftpFileTransfer = SftpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(String::from("127.0.0.1"), 10022, None, None)
            .is_err());
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_sftp_get_remote_path() {
        let mut client: SftpFileTransfer = SftpFileTransfer::new(SshKeyStorage::empty());
        // Connect
        assert!(client
            .connect(
                String::from("127.0.0.1"),
                10022,
                Some(String::from("sftp")),
                Some(String::from("password"))
            )
            .is_ok());
        // get realpath
        assert!(client
            .change_dir(PathBuf::from("/config").as_path())
            .is_ok());
        assert_eq!(
            client
                .get_remote_path(PathBuf::from("sshd.pid").as_path())
                .ok()
                .unwrap(),
            PathBuf::from("/config/sshd.pid")
        );
        // No such file
        assert!(client
            .get_remote_path(PathBuf::from("omarone.txt").as_path())
            .is_err());
        // Ok abs path
        assert_eq!(
            client
                .get_remote_path(PathBuf::from("/config/sshd.pid").as_path())
                .ok()
                .unwrap(),
            PathBuf::from("/config/sshd.pid")
        );
    }

    #[test]
    fn test_filetransfer_sftp_bad_server() {
        let mut client: SftpFileTransfer = SftpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("mybadserver.veryverybad.awful"),
                22,
                None,
                None
            )
            .is_err());
    }

    #[test]
    fn test_filetransfer_sftp_uninitialized() {
        let file: FsFile = FsFile {
            name: String::from("omar.txt"),
            abs_path: PathBuf::from("/omar.txt"),
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            size: 0,
            ftype: Some(String::from("txt")), // File type
            symlink: None,                    // UNIX only
            user: Some(0),                    // UNIX only
            group: Some(0),                   // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        };
        let mut sftp: SftpFileTransfer = SftpFileTransfer::new(SshKeyStorage::empty());
        assert!(sftp.change_dir(Path::new("/tmp")).is_err());
        assert!(sftp
            .copy(
                &make_fsentry(PathBuf::from("/nowhere"), false),
                PathBuf::from("/culonia").as_path()
            )
            .is_err());
        assert!(sftp.exec("echo 5").is_err());
        assert!(sftp.disconnect().is_err());
        assert!(sftp.list_dir(Path::new("/tmp")).is_err());
        assert!(sftp.mkdir(Path::new("/tmp")).is_err());
        assert!(sftp.pwd().is_err());
        assert!(sftp
            .remove(&make_fsentry(PathBuf::from("/nowhere"), false))
            .is_err());
        assert!(sftp
            .rename(
                &make_fsentry(PathBuf::from("/nowhere"), false),
                PathBuf::from("/culonia").as_path()
            )
            .is_err());
        assert!(sftp.stat(Path::new("/tmp")).is_err());
        assert!(sftp.recv_file(&file).is_err());
        assert!(sftp.send_file(&file, Path::new("/tmp/omar.txt")).is_err());
    }
}
