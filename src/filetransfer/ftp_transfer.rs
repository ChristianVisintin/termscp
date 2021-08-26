//! ## FTP transfer
//!
//! `ftp_transfer` is the module which provides the implementation for the FTP/FTPS file transfer

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
use super::{FileTransfer, FileTransferError, FileTransferErrorType, ProtocolParams};
use crate::fs::{FsDirectory, FsEntry, FsFile, UnixPex};
use crate::utils::fmt::shadow_password;
use crate::utils::path;

// Includes
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use suppaftp::native_tls::TlsConnector;
use suppaftp::{
    list::{File, PosixPexQuery},
    status::FILE_UNAVAILABLE,
    types::{FileType, Response},
    FtpError, FtpStream,
};

/// ## FtpFileTransfer
///
/// Ftp file transfer struct
pub struct FtpFileTransfer {
    stream: Option<FtpStream>,
    ftps: bool,
}

impl FtpFileTransfer {
    /// ### new
    ///
    /// Instantiates a new `FtpFileTransfer`
    pub fn new(ftps: bool) -> FtpFileTransfer {
        FtpFileTransfer { stream: None, ftps }
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

    /// ### parse_list_lines
    ///
    /// Parse all lines of LIST command output and instantiates a vector of FsEntry from it.
    /// This function also converts from `suppaftp::list::File` to `FsEntry`
    fn parse_list_lines(&mut self, path: &Path, lines: Vec<String>) -> Vec<FsEntry> {
        // Iter and collect
        lines
            .into_iter()
            .map(File::try_from) // Try to convert to file
            .flatten() // Remove errors
            .map(|x| {
                let mut abs_path: PathBuf = path.to_path_buf();
                abs_path.push(x.name());
                match x.is_directory() {
                    true => FsEntry::Directory(FsDirectory {
                        name: x.name().to_string(),
                        abs_path,
                        last_access_time: x.modified(),
                        last_change_time: x.modified(),
                        creation_time: x.modified(),
                        symlink: None,
                        user: x.uid(),
                        group: x.gid(),
                        unix_pex: Some(Self::query_unix_pex(&x)),
                    }),
                    false => FsEntry::File(FsFile {
                        name: x.name().to_string(),
                        size: x.size(),
                        ftype: abs_path
                            .extension()
                            .map(|ext| String::from(ext.to_str().unwrap_or(""))),
                        last_access_time: x.modified(),
                        last_change_time: x.modified(),
                        creation_time: x.modified(),
                        user: x.uid(),
                        group: x.gid(),
                        symlink: Self::get_symlink_entry(path, x.symlink()),
                        abs_path,
                        unix_pex: Some(Self::query_unix_pex(&x)),
                    }),
                }
            })
            .collect()
    }

    /// ### get_symlink_entry
    ///
    /// Get FsEntry from symlink
    fn get_symlink_entry(wrkdir: &Path, link: Option<&Path>) -> Option<Box<FsEntry>> {
        match link {
            None => None,
            Some(p) => {
                // Make abs path
                let abs_path: PathBuf = path::absolutize(wrkdir, p);
                Some(Box::new(FsEntry::File(FsFile {
                    name: p
                        .file_name()
                        .map(|x| x.to_str().unwrap_or("").to_string())
                        .unwrap_or_default(),
                    ftype: abs_path
                        .extension()
                        .map(|ext| String::from(ext.to_str().unwrap_or(""))),
                    size: 0,
                    last_access_time: UNIX_EPOCH,
                    last_change_time: UNIX_EPOCH,
                    creation_time: UNIX_EPOCH,
                    user: None,
                    group: None,
                    symlink: None,
                    unix_pex: None,
                    abs_path,
                })))
            }
        }
    }

    /// ### query_unix_pex
    ///
    /// Returns unix pex in tuple of values
    fn query_unix_pex(f: &File) -> (UnixPex, UnixPex, UnixPex) {
        (
            UnixPex::new(
                f.can_read(PosixPexQuery::Owner),
                f.can_write(PosixPexQuery::Owner),
                f.can_execute(PosixPexQuery::Owner),
            ),
            UnixPex::new(
                f.can_read(PosixPexQuery::Group),
                f.can_write(PosixPexQuery::Group),
                f.can_execute(PosixPexQuery::Group),
            ),
            UnixPex::new(
                f.can_read(PosixPexQuery::Others),
                f.can_write(PosixPexQuery::Others),
                f.can_execute(PosixPexQuery::Others),
            ),
        )
    }
}

impl FileTransfer for FtpFileTransfer {
    /// ### connect
    ///
    /// Connect to the remote server

    fn connect(&mut self, params: &ProtocolParams) -> Result<Option<String>, FileTransferError> {
        let params = match params.generic_params() {
            Some(params) => params,
            None => return Err(FileTransferError::new(FileTransferErrorType::BadAddress)),
        };
        // Get stream
        info!("Connecting to {}:{}", params.address, params.port);
        let mut stream: FtpStream =
            match FtpStream::connect(format!("{}:{}", params.address, params.port)) {
                Ok(stream) => stream,
                Err(err) => {
                    error!("Failed to connect: {}", err);
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::ConnectionError,
                        err.to_string(),
                    ));
                }
            };
        // If SSL, open secure session
        if self.ftps {
            info!("Setting up TLS stream...");
            let ctx = match TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
            {
                Ok(tls) => tls,
                Err(err) => {
                    error!("Failed to setup TLS stream: {}", err);
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::SslError,
                        err.to_string(),
                    ));
                }
            };
            stream = match stream.into_secure(ctx, params.address.as_str()) {
                Ok(s) => s,
                Err(err) => {
                    error!("Failed to setup TLS stream: {}", err);
                    return Err(FileTransferError::new_ex(
                        FileTransferErrorType::SslError,
                        err.to_string(),
                    ));
                }
            };
        }
        // Login (use anonymous if credentials are unspecified)
        let username: String = match &params.username {
            Some(u) => u.to_string(),
            None => String::from("anonymous"),
        };
        let password: String = match &params.password {
            Some(pwd) => pwd.to_string(),
            None => String::new(),
        };
        info!(
            "Signin in with username: {}, password: {}",
            username,
            shadow_password(password.as_str())
        );
        if let Err(err) = stream.login(username.as_str(), password.as_str()) {
            error!("Login failed: {}", err);
            return Err(FileTransferError::new_ex(
                FileTransferErrorType::AuthenticationFailed,
                err.to_string(),
            ));
        }
        debug!("Setting transfer type to Binary");
        // Initialize file type
        if let Err(err) = stream.transfer_type(FileType::Binary) {
            error!("Failed to set transfer type to binary: {}", err);
            return Err(FileTransferError::new_ex(
                FileTransferErrorType::ProtocolError,
                err.to_string(),
            ));
        }
        // Set stream
        self.stream = Some(stream);
        info!("Connection successfully established");
        // Return OK
        Ok(self
            .stream
            .as_ref()
            .unwrap()
            .get_welcome_msg()
            .map(|x| x.to_string()))
    }

    /// ### disconnect
    ///
    /// Disconnect from the remote server

    fn disconnect(&mut self) -> Result<(), FileTransferError> {
        info!("Disconnecting from FTP server...");
        match &mut self.stream {
            Some(stream) => match stream.quit() {
                Ok(_) => {
                    self.stream = None;
                    Ok(())
                }
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    err.to_string(),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### is_connected
    ///
    /// Indicates whether the client is connected to remote
    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// ### pwd
    ///
    /// Print working directory

    fn pwd(&mut self) -> Result<PathBuf, FileTransferError> {
        info!("PWD");
        match &mut self.stream {
            Some(stream) => match stream.pwd() {
                Ok(path) => Ok(PathBuf::from(path.as_str())),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    err.to_string(),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### change_dir
    ///
    /// Change working directory

    fn change_dir(&mut self, dir: &Path) -> Result<PathBuf, FileTransferError> {
        let dir: PathBuf = Self::resolve(dir);
        info!("Changing directory to {}", dir.display());
        match &mut self.stream {
            Some(stream) => match stream.cwd(&dir.as_path().to_string_lossy()) {
                Ok(_) => Ok(dir),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    err.to_string(),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### copy
    ///
    /// Copy file to destination
    fn copy(&mut self, _src: &FsEntry, _dst: &Path) -> Result<(), FileTransferError> {
        // FTP doesn't support file copy
        debug!("COPY issues (will fail, since unsupported)");
        Err(FileTransferError::new(
            FileTransferErrorType::UnsupportedFeature,
        ))
    }

    /// ### list_dir
    ///
    /// List directory entries

    fn list_dir(&mut self, path: &Path) -> Result<Vec<FsEntry>, FileTransferError> {
        let dir: PathBuf = Self::resolve(path);
        info!("LIST dir {}", dir.display());
        match &mut self.stream {
            Some(stream) => match stream.list(Some(&dir.as_path().to_string_lossy())) {
                Ok(lines) => {
                    debug!("Got {} lines in LIST result", lines.len());
                    // Iterate over entries
                    Ok(self.parse_list_lines(path, lines))
                }
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::DirStatFailed,
                    err.to_string(),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### mkdir
    ///
    /// In case the directory already exists, it must return an Error of kind `FileTransferErrorType::DirectoryAlreadyExists`
    fn mkdir(&mut self, dir: &Path) -> Result<(), FileTransferError> {
        let dir: PathBuf = Self::resolve(dir);
        info!("MKDIR {}", dir.display());
        match &mut self.stream {
            Some(stream) => match stream.mkdir(&dir.as_path().to_string_lossy()) {
                Ok(_) => Ok(()),
                Err(FtpError::UnexpectedResponse(Response {
                    // Directory already exists
                    code: FILE_UNAVAILABLE,
                    body: _,
                })) => {
                    error!("Directory {} already exists", dir.display());
                    Err(FileTransferError::new(
                        FileTransferErrorType::DirectoryAlreadyExists,
                    ))
                }
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::FileCreateDenied,
                    err.to_string(),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### remove
    ///
    /// Remove a file or a directory
    fn remove(&mut self, fsentry: &FsEntry) -> Result<(), FileTransferError> {
        if self.stream.is_none() {
            return Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            ));
        }
        info!("Removing entry {}", fsentry.get_abs_path().display());
        let wrkdir: PathBuf = self.pwd()?;
        match fsentry {
            // Match fs entry...
            FsEntry::File(file) => {
                // Go to parent directory
                if let Some(parent_dir) = file.abs_path.parent() {
                    debug!("Changing wrkdir to {}", parent_dir.display());
                    self.change_dir(parent_dir)?;
                }
                debug!("entry is a file; removing file {}", file.abs_path.display());
                // Remove file directly
                let result = self
                    .stream
                    .as_mut()
                    .unwrap()
                    .rm(file.name.as_ref())
                    .map(|_| ())
                    .map_err(|e| {
                        FileTransferError::new_ex(FileTransferErrorType::PexError, e.to_string())
                    });
                // Go to source directory
                match self.change_dir(wrkdir.as_path()) {
                    Err(err) => Err(err),
                    Ok(_) => result,
                }
            }
            FsEntry::Directory(dir) => {
                // Get directory files
                debug!("Entry is a directory; iterating directory entries");
                let result = match self.list_dir(dir.abs_path.as_path()) {
                    Ok(files) => {
                        // Remove recursively files
                        debug!("Removing {} entries from directory...", files.len());
                        for file in files.iter() {
                            if let Err(err) = self.remove(file) {
                                return Err(FileTransferError::new_ex(
                                    FileTransferErrorType::PexError,
                                    err.to_string(),
                                ));
                            }
                        }
                        // Once all files in directory have been deleted, remove directory
                        debug!("Finally removing directory {}...", dir.name);
                        // Enter parent directory
                        if let Some(parent_dir) = dir.abs_path.parent() {
                            debug!(
                                "Changing wrkdir to {} to delete directory {}",
                                parent_dir.display(),
                                dir.name
                            );
                            self.change_dir(parent_dir)?;
                        }
                        match self.stream.as_mut().unwrap().rmdir(dir.name.as_str()) {
                            Ok(_) => {
                                debug!("Removed {}", dir.abs_path.display());
                                Ok(())
                            }
                            Err(err) => Err(FileTransferError::new_ex(
                                FileTransferErrorType::PexError,
                                err.to_string(),
                            )),
                        }
                    }
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::DirStatFailed,
                        err.to_string(),
                    )),
                };
                // Restore directory
                match self.change_dir(wrkdir.as_path()) {
                    Err(err) => Err(err),
                    Ok(_) => result,
                }
            }
        }
    }

    /// ### rename
    ///
    /// Rename file or a directory
    fn rename(&mut self, file: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        let dst: PathBuf = Self::resolve(dst);
        info!(
            "Renaming {} to {}",
            file.get_abs_path().display(),
            dst.display()
        );
        match &mut self.stream {
            Some(stream) => {
                // Get name
                let src_name: String = match file {
                    FsEntry::Directory(dir) => dir.name.clone(),
                    FsEntry::File(file) => file.name.clone(),
                };
                // Only names are supported
                match stream.rename(src_name.as_str(), &dst.as_path().to_string_lossy()) {
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

    /// ### stat
    ///
    /// Stat file and return FsEntry
    fn stat(&mut self, _path: &Path) -> Result<FsEntry, FileTransferError> {
        match &mut self.stream {
            Some(_) => Err(FileTransferError::new(
                FileTransferErrorType::UnsupportedFeature,
            )),
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### exec
    ///
    /// Execute a command on remote host
    fn exec(&mut self, _cmd: &str) -> Result<String, FileTransferError> {
        Err(FileTransferError::new(
            FileTransferErrorType::UnsupportedFeature,
        ))
    }

    /// ### send_file
    ///
    /// Send file to remote
    /// File name is referred to the name of the file as it will be saved
    /// Data contains the file data
    /// Returns file and its size
    fn send_file(
        &mut self,
        _local: &FsFile,
        file_name: &Path,
    ) -> Result<Box<dyn Write>, FileTransferError> {
        let file_name: PathBuf = Self::resolve(file_name);
        info!("Sending file {}", file_name.display());
        match &mut self.stream {
            Some(stream) => match stream.put_with_stream(&file_name.as_path().to_string_lossy()) {
                Ok(writer) => Ok(Box::new(writer)), // NOTE: don't use BufWriter here, since already returned by the library
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::FileCreateDenied,
                    err.to_string(),
                )),
            },
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
        info!("Receiving file {}", file.abs_path.display());
        match &mut self.stream {
            Some(stream) => match stream.retr_as_stream(&file.abs_path.as_path().to_string_lossy())
            {
                Ok(reader) => Ok(Box::new(reader)), // NOTE: don't use BufReader here, since already returned by the library
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::NoSuchFileOrDirectory,
                    err.to_string(),
                )),
            },
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
    fn on_sent(&mut self, writable: Box<dyn Write>) -> Result<(), FileTransferError> {
        info!("Finalizing put stream");
        match &mut self.stream {
            Some(stream) => match stream.finalize_put_stream(writable) {
                Ok(_) => Ok(()),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ProtocolError,
                    err.to_string(),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### on_recv
    ///
    /// Finalize recv method.
    /// This method must be implemented only if necessary; in case you don't need it, just return `Ok(())`
    /// The purpose of this method is to finalize the connection with the peer when reading data.
    /// This mighe be necessary for some protocols.
    /// You must call this method each time you want to finalize the read of the remote file.
    fn on_recv(&mut self, readable: Box<dyn Read>) -> Result<(), FileTransferError> {
        info!("Finalizing get");
        match &mut self.stream {
            Some(stream) => match stream.finalize_retr_stream(readable) {
                Ok(_) => Ok(()),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ProtocolError,
                    err.to_string(),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::filetransfer::params::GenericProtocolParams;
    use crate::utils::file::open_file;
    #[cfg(feature = "with-containers")]
    use crate::utils::test_helpers::write_file;
    use crate::utils::test_helpers::{create_sample_file_entry, make_fsentry};

    use pretty_assertions::assert_eq;
    use std::io::{Read, Write};
    use std::time::Duration;

    #[test]
    fn test_filetransfer_ftp_new() {
        let ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        assert_eq!(ftp.ftps, false);
        assert!(ftp.stream.is_none());
        // FTPS
        let ftp: FtpFileTransfer = FtpFileTransfer::new(true);
        assert_eq!(ftp.ftps, true);
        assert!(ftp.stream.is_none());
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_ftp_server() {
        let mut ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        // Sample file
        let (entry, file): (FsFile, tempfile::NamedTempFile) = create_sample_file_entry();
        // Connect
        let hostname: String = String::from("127.0.0.1");
        assert!(ftp
            .connect(&ProtocolParams::Generic(
                GenericProtocolParams::default()
                    .address(hostname)
                    .port(10021)
                    .username(Some("test"))
                    .password(Some("test"))
            ))
            .is_ok());
        assert_eq!(ftp.is_connected(), true);
        // Get pwd
        assert_eq!(ftp.pwd().unwrap(), PathBuf::from("/"));
        // List dir (dir is empty)
        assert_eq!(ftp.list_dir(&Path::new("/")).unwrap().len(), 0);
        // Make directory
        assert!(ftp.mkdir(PathBuf::from("/home").as_path()).is_ok());
        // Remake directory (should report already exists)
        assert_eq!(
            ftp.mkdir(PathBuf::from("/home").as_path())
                .err()
                .unwrap()
                .kind(),
            FileTransferErrorType::DirectoryAlreadyExists
        );
        // Make directory (err)
        assert!(ftp.mkdir(PathBuf::from("/root/pommlar").as_path()).is_err());
        // Change directory
        assert!(ftp.change_dir(PathBuf::from("/home").as_path()).is_ok());
        // Change directory (err)
        assert!(ftp
            .change_dir(PathBuf::from("/tmp/oooo/aaaa/eee").as_path())
            .is_err());
        // Copy (not supported)
        assert!(ftp
            .copy(&FsEntry::File(entry.clone()), PathBuf::from("/").as_path())
            .is_err());
        // Exec (not supported)
        assert!(ftp.exec("echo 1;").is_err());
        // Upload 2 files
        let mut writable = ftp
            .send_file(&entry, PathBuf::from("omar.txt").as_path())
            .ok()
            .unwrap();
        write_file(&file, &mut writable);
        assert!(ftp.on_sent(writable).is_ok());
        let mut writable = ftp
            .send_file(&entry, PathBuf::from("README.md").as_path())
            .ok()
            .unwrap();
        write_file(&file, &mut writable);
        assert!(ftp.on_sent(writable).is_ok());
        // Upload file (err)
        assert!(ftp
            .send_file(&entry, PathBuf::from("/ommlar/omarone").as_path())
            .is_err());
        // List dir
        let list: Vec<FsEntry> = ftp.list_dir(PathBuf::from("/home").as_path()).ok().unwrap();
        assert_eq!(list.len(), 2);
        // Find
        assert!(ftp.change_dir(PathBuf::from("/").as_path()).is_ok());
        assert_eq!(ftp.find("*.txt").ok().unwrap().len(), 1);
        assert_eq!(ftp.find("*.md").ok().unwrap().len(), 1);
        assert_eq!(ftp.find("*.jpeg").ok().unwrap().len(), 0);
        assert!(ftp.change_dir(PathBuf::from("/home").as_path()).is_ok());
        // Rename
        assert!(ftp.mkdir(PathBuf::from("/uploads").as_path()).is_ok());
        assert!(ftp
            .rename(
                list.get(0).unwrap(),
                PathBuf::from("/uploads/README.txt").as_path()
            )
            .is_ok());
        // Rename (err)
        assert!(ftp
            .rename(list.get(0).unwrap(), PathBuf::from("OMARONE").as_path())
            .is_err());
        let dummy: FsEntry = FsEntry::File(FsFile {
            name: String::from("cucumber.txt"),
            abs_path: PathBuf::from("/cucumber.txt"),
            last_change_time: UNIX_EPOCH,
            last_access_time: UNIX_EPOCH,
            creation_time: UNIX_EPOCH,
            size: 0,
            ftype: Some(String::from("txt")), // File type
            symlink: None,                    // UNIX only
            user: Some(0),                    // UNIX only
            group: Some(0),                   // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        assert!(ftp
            .rename(&dummy, PathBuf::from("/a/b/c").as_path())
            .is_err());
        // Remove
        assert!(ftp.remove(list.get(1).unwrap()).is_ok());
        assert!(ftp.remove(list.get(1).unwrap()).is_err());
        // Receive file
        let mut writable = ftp
            .send_file(&entry, PathBuf::from("/uploads/README.txt").as_path())
            .ok()
            .unwrap();
        write_file(&file, &mut writable);
        assert!(ftp.on_sent(writable).is_ok());
        let file: FsFile = ftp
            .list_dir(PathBuf::from("/uploads").as_path())
            .ok()
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
            .unwrap_file();
        let mut readable = ftp.recv_file(&file).ok().unwrap();
        let mut data: Vec<u8> = vec![0; 1024];
        assert!(readable.read(&mut data).is_ok());
        assert!(ftp.on_recv(readable).is_ok());
        // Receive file (err)
        assert!(ftp.recv_file(&entry).is_err());
        // Cleanup
        assert!(ftp.change_dir(PathBuf::from("/").as_path()).is_ok());
        assert!(ftp
            .remove(&make_fsentry(PathBuf::from("/home"), true))
            .is_ok());
        assert!(ftp
            .remove(&make_fsentry(PathBuf::from("/uploads"), true))
            .is_ok());
        // Disconnect
        assert!(ftp.disconnect().is_ok());
        assert_eq!(ftp.is_connected(), false);
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_ftp_server_bad_auth() {
        let mut ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        // Connect
        assert!(ftp
            .connect(&ProtocolParams::Generic(
                GenericProtocolParams::default()
                    .address("127.0.0.1")
                    .port(10021)
                    .username(Some("omar"))
                    .password(Some("ommlar"))
            ))
            .is_err());
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_filetransfer_ftp_no_credentials() {
        let mut ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        assert!(ftp
            .connect(&ProtocolParams::Generic(
                GenericProtocolParams::default()
                    .address("127.0.0.1")
                    .port(10021)
                    .username::<&str>(None)
                    .password::<&str>(None)
            ))
            .is_err());
    }

    #[test]
    fn test_filetransfer_ftp_server_bad_server() {
        let mut ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        // Connect
        assert!(ftp
            .connect(&ProtocolParams::Generic(
                GenericProtocolParams::default()
                    .address("mybad.veribad.server")
                    .port(21)
                    .username::<&str>(None)
                    .password::<&str>(None)
            ))
            .is_err());
    }

    #[test]
    fn test_filetransfer_ftp_parse_list_line_unix() {
        let mut ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        // Simple file
        let file: FsFile = ftp
            .parse_list_lines(
                PathBuf::from("/tmp").as_path(),
                vec!["-rw-rw-r-- 1 root  dialout  8192 Nov 5 2018 omar.txt".to_string()],
            )
            .get(0)
            .unwrap()
            .clone()
            .unwrap_file();
        assert_eq!(file.abs_path, PathBuf::from("/tmp/omar.txt"));
        assert_eq!(file.name, String::from("omar.txt"));
        assert_eq!(file.size, 8192);
        assert!(file.symlink.is_none());
        assert_eq!(file.user, None);
        assert_eq!(file.group, None);
        assert_eq!(
            file.unix_pex.unwrap(),
            (UnixPex::from(6), UnixPex::from(6), UnixPex::from(4))
        );
        assert_eq!(
            file.last_access_time
                .duration_since(UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1541376000)
        );
        assert_eq!(
            file.last_change_time
                .duration_since(UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1541376000)
        );
        assert_eq!(
            file.creation_time.duration_since(UNIX_EPOCH).ok().unwrap(),
            Duration::from_secs(1541376000)
        );
    }

    #[test]
    fn test_filetransfer_ftp_list_dir_dos_syntax() {
        let mut ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        // Connect
        assert!(ftp
            .connect(&ProtocolParams::Generic(
                GenericProtocolParams::default()
                    .address("test.rebex.net")
                    .port(21)
                    .username(Some("demo"))
                    .password(Some("password"))
            ))
            .is_ok());
        // Pwd
        assert_eq!(ftp.pwd().ok().unwrap(), PathBuf::from("/"));
        // List dir
        let files: Vec<FsEntry> = ftp.list_dir(PathBuf::from("/").as_path()).ok().unwrap();
        // There should be at least 1 file
        assert!(files.len() > 0);
        // Disconnect
        assert!(ftp.disconnect().is_ok());
    }

    #[test]
    fn test_filetransfer_ftp_uninitialized() {
        let file: FsFile = FsFile {
            name: String::from("omar.txt"),
            abs_path: PathBuf::from("/omar.txt"),
            last_change_time: UNIX_EPOCH,
            last_access_time: UNIX_EPOCH,
            creation_time: UNIX_EPOCH,
            size: 0,
            ftype: Some(String::from("txt")), // File type
            symlink: None,                    // UNIX only
            user: Some(0),                    // UNIX only
            group: Some(0),                   // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        };
        let mut ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        assert!(ftp.change_dir(Path::new("/tmp")).is_err());
        assert!(ftp.disconnect().is_err());
        assert!(ftp.list_dir(Path::new("/tmp")).is_err());
        assert!(ftp.mkdir(Path::new("/tmp")).is_err());
        assert!(ftp
            .remove(&make_fsentry(PathBuf::from("/nowhere"), false))
            .is_err());
        assert!(ftp
            .rename(
                &make_fsentry(PathBuf::from("/nowhere"), false),
                PathBuf::from("/culonia").as_path()
            )
            .is_err());
        assert!(ftp.pwd().is_err());
        assert!(ftp.stat(Path::new("/tmp")).is_err());
        assert!(ftp.recv_file(&file).is_err());
        assert!(ftp.send_file(&file, Path::new("/tmp/omar.txt")).is_err());
        let (_, temp): (FsFile, tempfile::NamedTempFile) = create_sample_file_entry();
        let readable: Box<dyn Read> = Box::new(std::fs::File::open(temp.path()).unwrap());
        assert!(ftp.on_recv(readable).is_err());
        let (_, temp): (FsFile, tempfile::NamedTempFile) = create_sample_file_entry();
        let writable: Box<dyn Write> =
            Box::new(open_file(temp.path(), true, true, true).ok().unwrap());
        assert!(ftp.on_sent(writable).is_err());
    }
}
