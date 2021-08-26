//! ## SCP_Transfer
//!
//! `scps_transfer` is the module which provides the implementation for the SCP file transfer

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
use crate::utils::parser::parse_lstime;

// Includes
use regex::Regex;
use ssh2::{Channel, Session};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// ## ScpFileTransfer
///
/// SCP file transfer structure
pub struct ScpFileTransfer {
    session: Option<Session>,
    wrkdir: PathBuf,
    key_storage: SshKeyStorage,
}

impl ScpFileTransfer {
    /// ### new
    ///
    /// Instantiates a new ScpFileTransfer
    pub fn new(key_storage: SshKeyStorage) -> ScpFileTransfer {
        ScpFileTransfer {
            session: None,
            wrkdir: PathBuf::from("~"),
            key_storage,
        }
    }

    /// ### resolve
    ///
    /// Fix provided path; on Windows fixes the backslashes, converting them to slashes
    /// While on POSIX does nothing
    #[cfg(target_os = "windows")]
    fn resolve(p: &Path) -> PathBuf {
        PathBuf::from(path_slash::PathExt::to_slash_lossy(p).as_str())
    }

    #[cfg(target_family = "unix")]
    fn resolve(p: &Path) -> PathBuf {
        p.to_path_buf()
    }

    /// ### absolutize
    ///
    /// Absolutize target path if relative.
    /// This also converts backslashes to slashes if relative
    fn absolutize(wrkdir: &Path, target: &Path) -> PathBuf {
        match target.is_absolute() {
            true => target.to_path_buf(),
            false => {
                let mut p: PathBuf = wrkdir.to_path_buf();
                p.push(target);
                Self::resolve(p.as_path())
            }
        }
    }

    /// ### parse_ls_output
    ///
    /// Parse a line of `ls -l` output and tokenize the output into a `FsEntry`
    fn parse_ls_output(&mut self, path: &Path, line: &str) -> Result<FsEntry, ()> {
        // Prepare list regex
        // NOTE: about this damn regex <https://stackoverflow.com/questions/32480890/is-there-a-regex-to-parse-the-values-from-an-ftp-directory-listing>
        lazy_static! {
            static ref LS_RE: Regex = Regex::new(r#"^([\-ld])([\-rwxs]{9})\s+(\d+)\s+(\w+)\s+(\w+)\s+(\d+)\s+(\w{3}\s+\d{1,2}\s+(?:\d{1,2}:\d{1,2}|\d{4}))\s+(.+)$"#).unwrap();
        }
        debug!("Parsing LS line: '{}'", line);
        // Apply regex to result
        match LS_RE.captures(line) {
            // String matches regex
            Some(metadata) => {
                // NOTE: metadata fmt: (regex, file_type, permissions, link_count, uid, gid, filesize, mtime, filename)
                // Expected 7 + 1 (8) values: + 1 cause regex is repeated at 0
                if metadata.len() < 8 {
                    return Err(());
                }
                // Collect metadata
                // Get if is directory and if is symlink
                let (mut is_dir, is_symlink): (bool, bool) = match metadata.get(1).unwrap().as_str()
                {
                    "-" => (false, false),
                    "l" => (false, true),
                    "d" => (true, false),
                    _ => return Err(()), // Ignore special files
                };
                // Check string length (unix pex)
                if metadata.get(2).unwrap().as_str().len() < 9 {
                    return Err(());
                }

                let pex = |range: Range<usize>| {
                    let mut count: u8 = 0;
                    for (i, c) in metadata.get(2).unwrap().as_str()[range].chars().enumerate() {
                        match c {
                            '-' => {}
                            _ => {
                                count += match i {
                                    0 => 4,
                                    1 => 2,
                                    2 => 1,
                                    _ => 0,
                                }
                            }
                        }
                    }
                    count
                };

                // Get unix pex
                let unix_pex = (
                    UnixPex::from(pex(0..3)),
                    UnixPex::from(pex(3..6)),
                    UnixPex::from(pex(6..9)),
                );

                // Parse mtime and convert to SystemTime
                let mtime: SystemTime = match parse_lstime(
                    metadata.get(7).unwrap().as_str(),
                    "%b %d %Y",
                    "%b %d %H:%M",
                ) {
                    Ok(t) => t,
                    Err(_) => SystemTime::UNIX_EPOCH,
                };
                // Get uid
                let uid: Option<u32> = match metadata.get(4).unwrap().as_str().parse::<u32>() {
                    Ok(uid) => Some(uid),
                    Err(_) => None,
                };
                // Get gid
                let gid: Option<u32> = match metadata.get(5).unwrap().as_str().parse::<u32>() {
                    Ok(gid) => Some(gid),
                    Err(_) => None,
                };
                // Get filesize
                let filesize: usize = metadata
                    .get(6)
                    .unwrap()
                    .as_str()
                    .parse::<usize>()
                    .unwrap_or(0);
                // Get link and name
                let (file_name, symlink_path): (String, Option<PathBuf>) = match is_symlink {
                    true => self.get_name_and_link(metadata.get(8).unwrap().as_str()),
                    false => (String::from(metadata.get(8).unwrap().as_str()), None),
                };
                // Check if file_name is '.' or '..'
                if file_name.as_str() == "." || file_name.as_str() == ".." {
                    debug!("File name is {}; ignoring entry", file_name);
                    return Err(());
                }
                // Get symlink; PATH mustn't be equal to filename
                let symlink: Option<Box<FsEntry>> = match symlink_path {
                    None => None,
                    Some(p) => match p.file_name().unwrap_or_else(|| std::ffi::OsStr::new(""))
                        == file_name.as_str()
                    {
                        // If name is equal, don't stat path; otherwise it would get stuck
                        true => None,
                        false => match self.stat(p.as_path()) {
                            // If path match filename
                            Ok(e) => {
                                // If e is a directory, set is_dir to true
                                if e.is_dir() {
                                    is_dir = true;
                                }
                                Some(Box::new(e))
                            }
                            Err(_) => None, // Ignore errors
                        },
                    },
                };
                // Re-check if is directory
                let mut abs_path: PathBuf = PathBuf::from(path);
                abs_path.push(file_name.as_str());
                let abs_path: PathBuf = Self::resolve(abs_path.as_path());
                // Get extension
                let extension: Option<String> = abs_path
                    .as_path()
                    .extension()
                    .map(|s| String::from(s.to_string_lossy()));
                // Return
                debug!("Follows LS line '{}' attributes", line);
                debug!("Is directory? {}", is_dir);
                debug!("Is symlink? {}", is_symlink);
                debug!("name: {}", file_name);
                debug!("abs_path: {}", abs_path.display());
                debug!("last_change_time: {}", fmt_time(mtime, "%Y-%m-%dT%H:%M:%S"));
                debug!("last_access_time: {}", fmt_time(mtime, "%Y-%m-%dT%H:%M:%S"));
                debug!("creation_time: {}", fmt_time(mtime, "%Y-%m-%dT%H:%M:%S"));
                debug!("symlink: {:?}", symlink);
                debug!("user: {:?}", uid);
                debug!("group: {:?}", gid);
                debug!("unix_pex: {:?}", unix_pex);
                debug!("---------------------------------------");
                // Push to entries
                Ok(match is_dir {
                    true => FsEntry::Directory(FsDirectory {
                        name: file_name,
                        abs_path,
                        last_change_time: mtime,
                        last_access_time: mtime,
                        creation_time: mtime,
                        symlink,
                        user: uid,
                        group: gid,
                        unix_pex: Some(unix_pex),
                    }),
                    false => FsEntry::File(FsFile {
                        name: file_name,
                        abs_path,
                        last_change_time: mtime,
                        last_access_time: mtime,
                        creation_time: mtime,
                        size: filesize,
                        ftype: extension,
                        symlink,
                        user: uid,
                        group: gid,
                        unix_pex: Some(unix_pex),
                    }),
                })
            }
            None => Err(()),
        }
    }

    /// ### get_name_and_link
    ///
    /// Returns from a `ls -l` command output file name token, the name of the file and the symbolic link (if there is any)
    fn get_name_and_link(&self, token: &str) -> (String, Option<PathBuf>) {
        let tokens: Vec<&str> = token.split(" -> ").collect();
        let filename: String = String::from(*tokens.get(0).unwrap());
        let symlink: Option<PathBuf> = tokens.get(1).map(PathBuf::from);
        (filename, symlink)
    }

    /// ### perform_shell_cmd_with
    ///
    /// Perform a shell command, but change directory to specified path first
    fn perform_shell_cmd_with_path(
        &mut self,
        path: &Path,
        cmd: &str,
    ) -> Result<String, FileTransferError> {
        self.perform_shell_cmd(format!("cd \"{}\"; {}", path.display(), cmd).as_str())
    }

    /// ### perform_shell_cmd
    ///
    /// Perform a shell command and read the output from shell
    /// This operation is, obviously, blocking.
    fn perform_shell_cmd(&mut self, cmd: &str) -> Result<String, FileTransferError> {
        match self.session.as_mut() {
            Some(session) => {
                debug!("Running command: {}", cmd);
                // Create channel
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

impl FileTransfer for ScpFileTransfer {
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
                    debug!("{} succeded", socket_addr);
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
        // Get banner
        let banner: Option<String> = session.banner().map(String::from);
        debug!(
            "Connection established: {}",
            banner.as_deref().unwrap_or("")
        );
        // Set session
        self.session = Some(session);
        // Get working directory
        debug!("Getting working directory...");
        self.wrkdir = self
            .perform_shell_cmd("pwd")
            .map(|x| PathBuf::from(x.as_str().trim()))?;
        info!(
            "Connection established; working directory: {}",
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
                        // Set session to none
                        self.session = None;
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
        match self.is_connected() {
            true => Ok(self.wrkdir.clone()),
            false => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### change_dir
    ///
    /// Change working directory

    fn change_dir(&mut self, dir: &Path) -> Result<PathBuf, FileTransferError> {
        match self.is_connected() {
            true => {
                let p: PathBuf = self.wrkdir.clone();
                let remote_path: PathBuf = Self::absolutize(Path::new("."), dir);
                info!("Changing working directory to {}", remote_path.display());
                // Change directory
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("cd \"{}\"; echo $?; pwd", remote_path.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Trim
                        let output: String = String::from(output.as_str().trim());
                        // Check if output starts with 0; should be 0{PWD}
                        match output.as_str().starts_with('0') {
                            true => {
                                // Set working directory
                                self.wrkdir = PathBuf::from(&output.as_str()[1..].trim());
                                info!("Changed working directory to {}", self.wrkdir.display());
                                Ok(self.wrkdir.clone())
                            }
                            false => Err(FileTransferError::new_ex(
                                // No such file or directory
                                FileTransferErrorType::NoSuchFileOrDirectory,
                                format!("\"{}\"", dir.display()),
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

    /// ### copy
    ///
    /// Copy file to destination
    fn copy(&mut self, src: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        match self.is_connected() {
            true => {
                let dst: PathBuf = Self::resolve(dst);
                info!(
                    "Copying {} to {}",
                    src.get_abs_path().display(),
                    dst.display()
                );
                // Run `cp -rf`
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
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
        match self.is_connected() {
            true => {
                // Send ls -l to path
                info!("Getting file entries in {}", path.display());
                let path: PathBuf = Self::resolve(path);
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("unset LANG; ls -la \"{}/\"", path.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Split output by (\r)\n
                        let lines: Vec<&str> = output.as_str().lines().collect();
                        let mut entries: Vec<FsEntry> = Vec::with_capacity(lines.len());
                        for line in lines.iter() {
                            // First line must always be ignored
                            // Parse row, if ok push to entries
                            if let Ok(entry) = self.parse_ls_output(path.as_path(), line) {
                                entries.push(entry);
                            }
                        }
                        info!(
                            "Found {} out of {} valid file entries",
                            entries.len(),
                            lines.len()
                        );
                        Ok(entries)
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

    /// ### mkdir
    ///
    /// Make directory
    /// In case the directory already exists, it must return an Error of kind `FileTransferErrorType::DirectoryAlreadyExists`
    fn mkdir(&mut self, dir: &Path) -> Result<(), FileTransferError> {
        match self.is_connected() {
            true => {
                let dir: PathBuf = Self::resolve(dir);
                info!("Making directory {}", dir.display());
                let p: PathBuf = self.wrkdir.clone();
                // If directory already exists, return Err
                let mut dir_stat_path: PathBuf = dir.clone();
                dir_stat_path.push("./");
                if self.stat(dir_stat_path.as_path()).is_ok() {
                    error!("Directory {} already exists", dir.display());
                    return Err(FileTransferError::new(
                        FileTransferErrorType::DirectoryAlreadyExists,
                    ));
                }
                // Mkdir dir && echo 0
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("mkdir \"{}\"; echo $?", dir.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Check if output is 0
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // Directory created
                            false => Err(FileTransferError::new_ex(
                                // Could not create directory
                                FileTransferErrorType::FileCreateDenied,
                                format!("\"{}\"", dir.display()),
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

    /// ### remove
    ///
    /// Remove a file or a directory
    fn remove(&mut self, file: &FsEntry) -> Result<(), FileTransferError> {
        // Yay, we have rm -rf here :D
        match self.is_connected() {
            true => {
                // Get path
                let path: PathBuf = file.get_abs_path();
                info!("Removing file {}", path.display());
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!("rm -rf \"{}\"; echo $?", path.display()).as_str(),
                ) {
                    Ok(output) => {
                        // Check if output is 0
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // Directory created
                            false => Err(FileTransferError::new_ex(
                                // Could not create directory
                                FileTransferErrorType::PexError,
                                format!("\"{}\"", path.display()),
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

    /// ### rename
    ///
    /// Rename file or a directory
    fn rename(&mut self, file: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        match self.is_connected() {
            true => {
                // Get path
                let dst: PathBuf = Self::resolve(dst);
                let path: PathBuf = file.get_abs_path();
                info!("Renaming {} to {}", path.display(), dst.display());
                let p: PathBuf = self.wrkdir.clone();
                match self.perform_shell_cmd_with_path(
                    p.as_path(),
                    format!(
                        "mv -f \"{}\" \"{}\"; echo $?",
                        path.display(),
                        dst.display()
                    )
                    .as_str(),
                ) {
                    Ok(output) => {
                        // Check if output is 0
                        match output.as_str().trim() == "0" {
                            true => Ok(()), // File renamed
                            false => Err(FileTransferError::new_ex(
                                // Could not move file
                                FileTransferErrorType::PexError,
                                format!("\"{}\"", path.display()),
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

    /// ### stat
    ///
    /// Stat file and return FsEntry
    fn stat(&mut self, path: &Path) -> Result<FsEntry, FileTransferError> {
        let path: PathBuf = Self::absolutize(self.wrkdir.as_path(), path);
        match self.is_connected() {
            true => {
                let p: PathBuf = self.wrkdir.clone();
                info!("Stat {}", path.display());
                // make command; Directories require `-d` option
                let cmd: String = match path.to_string_lossy().ends_with('/') {
                    true => format!("ls -ld \"{}\"", path.display()),
                    false => format!("ls -l \"{}\"", path.display()),
                };
                match self.perform_shell_cmd_with_path(p.as_path(), cmd.as_str()) {
                    Ok(line) => {
                        // Parse ls line
                        let parent: PathBuf = match path.as_path().parent() {
                            Some(p) => PathBuf::from(p),
                            None => {
                                return Err(FileTransferError::new_ex(
                                    FileTransferErrorType::DirStatFailed,
                                    String::from("Path has no parent"),
                                ))
                            }
                        };
                        match self.parse_ls_output(parent.as_path(), line.as_str().trim()) {
                            Ok(entry) => Ok(entry),
                            Err(_) => Err(FileTransferError::new(
                                FileTransferErrorType::NoSuchFileOrDirectory,
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

    /// ### exec
    ///
    /// Execute a command on remote host
    fn exec(&mut self, cmd: &str) -> Result<String, FileTransferError> {
        match self.is_connected() {
            true => {
                let p: PathBuf = self.wrkdir.clone();
                info!("Executing command {}", cmd);
                match self.perform_shell_cmd_with_path(p.as_path(), cmd) {
                    Ok(output) => Ok(output),
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

    /// ### send_file
    ///
    /// Send file to remote
    /// File name is referred to the name of the file as it will be saved
    /// Data contains the file data
    /// Returns file and its size
    fn send_file(
        &mut self,
        local: &FsFile,
        file_name: &Path,
    ) -> Result<Box<dyn Write>, FileTransferError> {
        match self.session.as_ref() {
            Some(session) => {
                let file_name: PathBuf = Self::absolutize(self.wrkdir.as_path(), file_name);
                info!(
                    "Sending file {} to {}",
                    local.abs_path.display(),
                    file_name.display()
                );
                // Set blocking to true
                debug!("blocking channel...");
                session.set_blocking(true);
                // Calculate file mode
                let mode: i32 = match local.unix_pex {
                    None => 0o644,
                    Some((u, g, o)) => {
                        ((u.as_byte() as i32) << 6)
                            + ((g.as_byte() as i32) << 3)
                            + (o.as_byte() as i32)
                    }
                };
                // Calculate mtime, atime
                let times: (u64, u64) = {
                    let mtime: u64 = match local
                        .last_change_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                    {
                        Ok(durr) => durr.as_secs() as u64,
                        Err(_) => 0,
                    };
                    let atime: u64 = match local
                        .last_access_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                    {
                        Ok(durr) => durr.as_secs() as u64,
                        Err(_) => 0,
                    };
                    (mtime, atime)
                };
                // We need to get the size of local; NOTE: don't use the `size` attribute, since might be out of sync
                let file_size: u64 = match std::fs::metadata(local.abs_path.as_path()) {
                    Ok(metadata) => metadata.len(),
                    Err(_) => local.size as u64, // NOTE: fallback to fsentry size
                };
                debug!(
                    "File mode {:?}; mtime: {}, atime: {}; file size: {}",
                    mode, times.0, times.1, file_size
                );
                // Send file
                match session.scp_send(file_name.as_path(), mode, file_size, Some(times)) {
                    Ok(channel) => Ok(Box::new(BufWriter::with_capacity(65536, channel))),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        err.to_string(),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### recv_file
    ///
    /// Receive file from remote with provided name
    /// Returns file and its size
    fn recv_file(&mut self, file: &FsFile) -> Result<Box<dyn Read>, FileTransferError> {
        match self.session.as_ref() {
            Some(session) => {
                info!("Receiving file {}", file.abs_path.display());
                // Set blocking to true
                debug!("Set blocking...");
                session.set_blocking(true);
                match session.scp_recv(file.abs_path.as_path()) {
                    Ok(reader) => Ok(Box::new(BufReader::with_capacity(65536, reader.0))),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::ProtocolError,
                        err.to_string(),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### on_sent
    ///
    /// Finalize send method.
    /// This method must be implemented only if necessary; in case you don't need it, just return `Ok(())`
    /// The purpose of this method is to finalize the connection with the peer when writing data.
    /// This is necessary for some protocols such as FTP.
    /// You must call this method each time you want to finalize the write of the remote file.
    fn on_sent(&mut self, _writable: Box<dyn Write>) -> Result<(), FileTransferError> {
        // Nothing to do
        Ok(())
    }

    /// ### on_recv
    ///
    /// Finalize recv method.
    /// This method must be implemented only if necessary; in case you don't need it, just return `Ok(())`
    /// The purpose of this method is to finalize the connection with the peer when reading data.
    /// This mighe be necessary for some protocols.
    /// You must call this method each time you want to finalize the read of the remote file.
    fn on_recv(&mut self, _readable: Box<dyn Read>) -> Result<(), FileTransferError> {
        // Nothing to do
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::test_helpers::make_fsentry;
    use pretty_assertions::assert_eq;

    #[cfg(feature = "with-containers")]
    use crate::utils::test_helpers::{create_sample_file_entry, write_file, write_ssh_key};

    #[test]
    fn test_filetransfer_scp_new() {
        let client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client.session.is_none());
        assert_eq!(client.is_connected(), false);
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_scp_server() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        // Sample file
        let (entry, file): (FsFile, tempfile::NamedTempFile) = create_sample_file_entry();
        // Connect
        assert!(client
            .connect(
                String::from("127.0.0.1"),
                10222,
                Some(String::from("sftp")),
                Some(String::from("password"))
            )
            .is_ok());
        // Check session and sftp
        assert!(client.session.is_some());
        assert_eq!(client.wrkdir, PathBuf::from("/config"));
        assert_eq!(client.is_connected(), true);
        // Pwd
        assert_eq!(client.wrkdir.clone(), client.pwd().ok().unwrap());
        // Stat
        let stat: FsFile = client
            .stat(PathBuf::from("sshd.pid").as_path())
            .ok()
            .unwrap()
            .unwrap_file();
        assert_eq!(stat.abs_path, PathBuf::from("/config/sshd.pid"));
        let stat: FsDirectory = client
            .stat(PathBuf::from("/config/").as_path())
            .ok()
            .unwrap()
            .unwrap_dir();
        assert_eq!(stat.abs_path, PathBuf::from("/config/"));
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
        // Copy file
        assert!(client
            .copy(
                &make_fsentry(PathBuf::from("/config/sshd.pid"), false),
                PathBuf::from("/tmp/sshd.pid").as_path()
            )
            .is_ok());
        // Copy dir
        assert!(client
            .copy(
                &make_fsentry(PathBuf::from("/tmp/omar"), true),
                PathBuf::from("/tmp/ommlar").as_path()
            )
            .is_ok());
        // Copy (err)
        assert!(client
            .copy(
                &make_fsentry(PathBuf::from("/tmp/zattera"), false),
                PathBuf::from("/").as_path()
            )
            .is_err());
        // Exec
        assert_eq!(client.exec("echo 5").ok().unwrap().as_str(), "5\n");
        // Change dir to ommlar
        assert!(client
            .change_dir(PathBuf::from("/tmp/ommlar/").as_path())
            .is_ok());
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
            .list_dir(PathBuf::from("/tmp/ommlar").as_path())
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
            .remove(&make_fsentry(PathBuf::from("/tmp/ommlar"), true))
            .is_ok());
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
    fn test_filetransfer_scp_ssh_storage() {
        let mut storage: SshKeyStorage = SshKeyStorage::empty();
        let key_file: tempfile::NamedTempFile = write_ssh_key();
        storage.add_key("127.0.0.1", "sftp", key_file.path().to_path_buf());
        let mut client: ScpFileTransfer = ScpFileTransfer::new(storage);
        // Connect
        assert!(client
            .connect(
                String::from("127.0.0.1"),
                10222,
                Some(String::from("sftp")),
                None,
            )
            .is_ok());
        assert_eq!(client.is_connected(), true);
        assert!(client.disconnect().is_ok());
    }

    #[test]
    fn test_filetransfer_scp_bad_auth() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(
                String::from("127.0.0.1"),
                10222,
                Some(String::from("demo")),
                Some(String::from("badpassword"))
            )
            .is_err());
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_scp_no_credentials() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(client
            .connect(String::from("127.0.0.1"), 10222, None, None)
            .is_err());
    }

    #[test]
    fn test_filetransfer_scp_bad_server() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
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
    fn test_filetransfer_scp_parse_ls() {
        let mut client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        // File
        let entry: FsFile = client
            .parse_ls_output(
                PathBuf::from("/tmp").as_path(),
                "-rw-r--r-- 1 root root  2056 giu 13 21:11 Cargo.toml",
            )
            .ok()
            .unwrap()
            .unwrap_file();
        assert_eq!(entry.name.as_str(), "Cargo.toml");
        assert_eq!(entry.abs_path, PathBuf::from("/tmp/Cargo.toml"));
        assert_eq!(
            entry.unix_pex.unwrap(),
            (UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))
        );
        assert_eq!(entry.size, 2056);
        assert_eq!(entry.ftype.unwrap().as_str(), "toml");
        assert!(entry.symlink.is_none());
        // File (year)
        let entry: FsFile = client
            .parse_ls_output(
                PathBuf::from("/tmp").as_path(),
                "-rw-rw-rw- 1 root root  3368 nov  7  2020 CODE_OF_CONDUCT.md",
            )
            .ok()
            .unwrap()
            .unwrap_file();
        assert_eq!(entry.name.as_str(), "CODE_OF_CONDUCT.md");
        assert_eq!(entry.abs_path, PathBuf::from("/tmp/CODE_OF_CONDUCT.md"));
        assert_eq!(
            entry.unix_pex.unwrap(),
            (UnixPex::from(6), UnixPex::from(6), UnixPex::from(6))
        );
        assert_eq!(entry.size, 3368);
        assert_eq!(entry.ftype.unwrap().as_str(), "md");
        assert!(entry.symlink.is_none());
        // Directory
        let entry: FsDirectory = client
            .parse_ls_output(
                PathBuf::from("/tmp").as_path(),
                "drwxr-xr-x 1 root root   512 giu 13 21:11 docs",
            )
            .ok()
            .unwrap()
            .unwrap_dir();
        assert_eq!(entry.name.as_str(), "docs");
        assert_eq!(entry.abs_path, PathBuf::from("/tmp/docs"));
        assert_eq!(
            entry.unix_pex.unwrap(),
            (UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))
        );
        assert!(entry.symlink.is_none());
        // Short metadata
        assert!(client
            .parse_ls_output(
                PathBuf::from("/tmp").as_path(),
                "drwxr-xr-x 1 root root   512 giu 13 21:11",
            )
            .is_err());
        // Special file
        assert!(client
            .parse_ls_output(
                PathBuf::from("/tmp").as_path(),
                "crwxr-xr-x 1 root root   512 giu 13 21:11 ttyS1",
            )
            .is_err());
        // Bad pex
        assert!(client
            .parse_ls_output(
                PathBuf::from("/tmp").as_path(),
                "-rwxr-xr 1 root root   512 giu 13 21:11 ttyS1",
            )
            .is_err());
    }

    #[test]
    fn test_filetransfer_scp_get_name_and_link() {
        let client: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert_eq!(
            client.get_name_and_link("Cargo.toml"),
            (String::from("Cargo.toml"), None)
        );
        assert_eq!(
            client.get_name_and_link("Cargo -> Cargo.toml"),
            (String::from("Cargo"), Some(PathBuf::from("Cargo.toml")))
        );
    }

    #[test]
    fn test_filetransfer_scp_uninitialized() {
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
        let mut scp: ScpFileTransfer = ScpFileTransfer::new(SshKeyStorage::empty());
        assert!(scp.change_dir(Path::new("/tmp")).is_err());
        assert!(scp.disconnect().is_err());
        assert!(scp.exec("echo 5").is_err());
        assert!(scp.list_dir(Path::new("/tmp")).is_err());
        assert!(scp.mkdir(Path::new("/tmp")).is_err());
        assert!(scp.pwd().is_err());
        assert!(scp
            .remove(&make_fsentry(PathBuf::from("/nowhere"), false))
            .is_err());
        assert!(scp
            .rename(
                &make_fsentry(PathBuf::from("/nowhere"), false),
                PathBuf::from("/culonia").as_path()
            )
            .is_err());
        assert!(scp.stat(Path::new("/tmp")).is_err());
        assert!(scp.recv_file(&file).is_err());
        assert!(scp.send_file(&file, Path::new("/tmp/omar.txt")).is_err());
    }
}
