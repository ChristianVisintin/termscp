//! ## BookmarksClient
//!
//! `bookmarks_client` is the module which provides an API between the Bookmarks module and the system

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
// Crate
#[cfg(feature = "with-keyring")]
use super::keys::keyringstorage::KeyringStorage;
use super::keys::{filestorage::FileStorage, KeyStorage, KeyStorageError};
// Local
use crate::config::{
    bookmarks::{Bookmark, UserHosts},
    serialization::{deserialize, serialize, SerializerError, SerializerErrorKind},
};
use crate::filetransfer::FileTransferProtocol;
use crate::utils::crypto;
use crate::utils::fmt::fmt_time;
use crate::utils::random::random_alphanumeric_with_len;
// Ext
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::time::SystemTime;

/// ## BookmarksClient
///
/// BookmarksClient provides a layer between the host system and the bookmarks module
pub struct BookmarksClient {
    hosts: UserHosts,
    bookmarks_file: PathBuf,
    key: String,
    recents_size: usize,
}

impl BookmarksClient {
    /// ### BookmarksClient
    ///
    /// Instantiates a new BookmarksClient
    /// Bookmarks file path must be provided
    /// Storage path for file provider must be provided
    pub fn new(
        bookmarks_file: &Path,
        storage_path: &Path,
        recents_size: usize,
    ) -> Result<BookmarksClient, SerializerError> {
        // Create default hosts
        let default_hosts: UserHosts = UserHosts::default();
        debug!("Setting up bookmarks client...");
        // Make a key storage (with-keyring)
        #[cfg(feature = "with-keyring")]
        let (key_storage, service_id): (Box<dyn KeyStorage>, &str) = {
            debug!("Setting up KeyStorage");
            let username: String = whoami::username();
            let storage: KeyringStorage = KeyringStorage::new(username.as_str());
            // Check if keyring storage is supported
            #[cfg(not(test))]
            let app_name: &str = "termscp";
            #[cfg(test)] // NOTE: when running test, add -test
            let app_name: &str = "termscp-test";
            match storage.is_supported() {
                true => {
                    debug!("Using KeyringStorage");
                    (Box::new(storage), app_name)
                }
                false => {
                    warn!("KeyringStorage is not supported; using FileStorage");
                    (Box::new(FileStorage::new(storage_path)), "bookmarks")
                }
            }
        };
        // Make a key storage (wno-keyring)
        #[cfg(not(feature = "with-keyring"))]
        let (key_storage, service_id): (Box<dyn KeyStorage>, &str) = {
            #[cfg(not(test))]
            let app_name: &str = "bookmarks";
            #[cfg(test)] // NOTE: when running test, add -test
            let app_name: &str = "bookmarks-test";
            debug!("Using FileStorage");
            (Box::new(FileStorage::new(storage_path)), app_name)
        };
        // Load key
        let key: String = match key_storage.get_key(service_id) {
            Ok(k) => {
                debug!("Key loaded with success");
                k
            }
            Err(e) => match e {
                KeyStorageError::NoSuchKey => {
                    // If no such key, generate key and set it into the storage
                    let key: String = Self::generate_key();
                    debug!("Key doesn't exist yet or could not be loaded; generated a new key");
                    if let Err(e) = key_storage.set_key(service_id, key.as_str()) {
                        error!("Failed to set new key into storage: {}", e);
                        return Err(SerializerError::new_ex(
                            SerializerErrorKind::Io,
                            format!("Could not write key to storage: {}", e),
                        ));
                    }
                    // Return key
                    key
                }
                _ => {
                    error!("Failed to get key from storage: {}", e);
                    return Err(SerializerError::new_ex(
                        SerializerErrorKind::Io,
                        format!("Could not get key from storage: {}", e),
                    ));
                }
            },
        };
        let mut client: BookmarksClient = BookmarksClient {
            hosts: default_hosts,
            bookmarks_file: PathBuf::from(bookmarks_file),
            key,
            recents_size,
        };
        // If bookmark file doesn't exist, initialize it
        if !bookmarks_file.exists() {
            info!("Bookmarks file doesn't exist yet; creating it...");
            if let Err(err) = client.write_bookmarks() {
                error!("Failed to create bookmarks file: {}", err);
                return Err(err);
            }
        } else {
            // Load bookmarks from file
            if let Err(err) = client.read_bookmarks() {
                error!("Failed to load bookmarks: {}", err);
                return Err(err);
            }
        }
        info!("Bookmarks client initialized");
        // Load key
        Ok(client)
    }

    /// ### iter_bookmarks
    ///
    /// Iterate over bookmarks keys
    pub fn iter_bookmarks(&self) -> impl Iterator<Item = &String> + '_ {
        Box::new(self.hosts.bookmarks.keys())
    }

    /// ### get_bookmark
    ///
    /// Get bookmark associated to key
    pub fn get_bookmark(
        &self,
        key: &str,
    ) -> Option<(String, u16, FileTransferProtocol, String, Option<String>)> {
        let entry: &Bookmark = self.hosts.bookmarks.get(key)?;
        debug!("Getting bookmark {}", key);
        Some((
            entry.address.clone(),
            entry.port,
            match FileTransferProtocol::from_str(entry.protocol.as_str()) {
                Ok(proto) => proto,
                Err(err) => {
                    error!(
                        "Found invalid protocol in bookmarks: {}; defaulting to SFTP",
                        err
                    );
                    FileTransferProtocol::Sftp // Default
                }
            },
            entry.username.clone(),
            match &entry.password {
                // Decrypted password if Some; if decryption fails return None
                Some(pwd) => match self.decrypt_str(pwd.as_str()) {
                    Ok(decrypted_pwd) => Some(decrypted_pwd),
                    Err(err) => {
                        error!("Failed to decrypt password for bookmark: {}", err);
                        None
                    }
                },
                None => None,
            },
        ))
    }

    /// ### add_recent
    ///
    /// Add a new recent to bookmarks
    pub fn add_bookmark(
        &mut self,
        name: String,
        addr: String,
        port: u16,
        protocol: FileTransferProtocol,
        username: String,
        password: Option<String>,
    ) {
        if name.is_empty() {
            error!("Fatal error; bookmark name is empty");
            panic!("Bookmark name can't be empty");
        }
        // Make bookmark
        info!("Added bookmark {} with address {}", name, addr);
        let host: Bookmark = self.make_bookmark(addr, port, protocol, username, password);
        self.hosts.bookmarks.insert(name, host);
    }

    /// ### del_bookmark
    ///
    /// Delete entry from bookmarks
    pub fn del_bookmark(&mut self, name: &str) {
        let _ = self.hosts.bookmarks.remove(name);
        info!("Removed bookmark {}", name);
    }
    /// ### iter_recents
    ///
    /// Iterate over recents keys
    pub fn iter_recents(&self) -> impl Iterator<Item = &String> + '_ {
        Box::new(self.hosts.recents.keys())
    }

    /// ### get_recent
    ///
    /// Get recent associated to key
    pub fn get_recent(&self, key: &str) -> Option<(String, u16, FileTransferProtocol, String)> {
        // NOTE: password is not decrypted; recents will never have password
        info!("Getting bookmark {}", key);
        let entry: &Bookmark = self.hosts.recents.get(key)?;
        Some((
            entry.address.clone(),
            entry.port,
            match FileTransferProtocol::from_str(entry.protocol.as_str()) {
                Ok(proto) => proto,
                Err(err) => {
                    error!(
                        "Found invalid protocol in bookmarks: {}; defaulting to SFTP",
                        err
                    );
                    FileTransferProtocol::Sftp // Default
                }
            },
            entry.username.clone(),
        ))
    }

    /// ### add_recent
    ///
    /// Add a new recent to bookmarks
    pub fn add_recent(
        &mut self,
        addr: String,
        port: u16,
        protocol: FileTransferProtocol,
        username: String,
    ) {
        // Make bookmark
        let host: Bookmark = self.make_bookmark(addr, port, protocol, username, None);
        // Check if duplicated
        for recent_host in self.hosts.recents.values() {
            if *recent_host == host {
                debug!("Discarding recent since duplicated ({})", host.address);
                // Don't save duplicates
                return;
            }
        }
        // If hosts size is bigger than self.recents_size; pop last
        if self.hosts.recents.len() >= self.recents_size {
            // Get keys
            let mut keys: Vec<String> = Vec::with_capacity(self.hosts.recents.len());
            for key in self.hosts.recents.keys() {
                keys.push(key.clone());
            }
            // Sort keys; NOTE: most recent is the last element
            keys.sort();
            // Delete keys starting from the last one
            for key in keys.iter() {
                let _ = self.hosts.recents.remove(key);
                debug!("Removed recent bookmark {}", key);
                // If length is < self.recents_size; break
                if self.hosts.recents.len() < self.recents_size {
                    break;
                }
            }
        }
        let name: String = fmt_time(SystemTime::now(), "ISO%Y%m%dT%H%M%S");
        info!("Saved recent host {} ({})", name, host.address);
        self.hosts.recents.insert(name, host);
    }

    /// ### del_recent
    ///
    /// Delete entry from recents
    pub fn del_recent(&mut self, name: &str) {
        let _ = self.hosts.recents.remove(name);
        info!("Removed recent host {}", name);
    }

    /// ### write_bookmarks
    ///
    /// Write bookmarks to file
    pub fn write_bookmarks(&self) -> Result<(), SerializerError> {
        // Open file
        debug!("Writing bookmarks");
        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.bookmarks_file.as_path())
        {
            Ok(writer) => serialize(&self.hosts, Box::new(writer)),
            Err(err) => {
                error!("Failed to write bookmarks: {}", err);
                Err(SerializerError::new_ex(
                    SerializerErrorKind::Io,
                    err.to_string(),
                ))
            }
        }
    }

    /// ### read_bookmarks
    ///
    /// Read bookmarks from file
    fn read_bookmarks(&mut self) -> Result<(), SerializerError> {
        // Open bookmarks file for read
        debug!("Reading bookmarks");
        match OpenOptions::new()
            .read(true)
            .open(self.bookmarks_file.as_path())
        {
            Ok(reader) => {
                // Deserialize
                match deserialize(Box::new(reader)) {
                    Ok(hosts) => {
                        self.hosts = hosts;
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            Err(err) => {
                error!("Failed to read bookmarks: {}", err);
                Err(SerializerError::new_ex(
                    SerializerErrorKind::Io,
                    err.to_string(),
                ))
            }
        }
    }

    /// ### generate_key
    ///
    /// Generate a new AES key
    fn generate_key() -> String {
        // Generate 256 bytes (2048 bits) key
        random_alphanumeric_with_len(256)
    }

    /// ### make_bookmark
    ///
    /// Make bookmark from credentials
    fn make_bookmark(
        &self,
        addr: String,
        port: u16,
        protocol: FileTransferProtocol,
        username: String,
        password: Option<String>,
    ) -> Bookmark {
        Bookmark {
            address: addr,
            port,
            username,
            protocol: protocol.to_string(),
            password: password.map(|p| self.encrypt_str(p.as_str())),
        }
    }

    /// ### encrypt_str
    ///
    /// Encrypt provided string using AES-128. Encrypted buffer is then converted to BASE64
    fn encrypt_str(&self, txt: &str) -> String {
        crypto::aes128_b64_crypt(self.key.as_str(), txt)
    }

    /// ### decrypt_str
    ///
    /// Decrypt provided string using AES-128
    fn decrypt_str(&self, secret: &str) -> Result<String, SerializerError> {
        match crypto::aes128_b64_decrypt(self.key.as_str(), secret) {
            Ok(txt) => Ok(txt),
            Err(err) => Err(SerializerError::new_ex(
                SerializerErrorKind::Syntax,
                err.to_string(),
            )),
        }
    }
}

#[cfg(test)]
#[cfg(not(target_os = "macos"))] // CI/CD blocks
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]

    fn test_system_bookmarks_new() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Verify client
        assert_eq!(client.hosts.bookmarks.len(), 0);
        assert_eq!(client.hosts.recents.len(), 0);
        assert_eq!(client.key.len(), 256);
        assert_eq!(client.bookmarks_file, cfg_path);
        assert_eq!(client.recents_size, 16);
    }

    #[test]
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    fn test_system_bookmarks_new_err() {
        assert!(BookmarksClient::new(
            Path::new("/tmp/oifoif/omar"),
            Path::new("/tmp/efnnu/omar"),
            16
        )
        .is_err());

        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, _): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        assert!(
            BookmarksClient::new(cfg_path.as_path(), Path::new("/tmp/efnnu/omar"), 16).is_err()
        );
    }

    #[test]

    fn test_system_bookmarks_new_from_existing() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Add some bookmarks
        client.add_bookmark(
            String::from("raspberry"),
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
            Some(String::from("mypassword")),
        );
        client.add_recent(
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
        );
        let recent_key: String = String::from(client.iter_recents().next().unwrap());
        assert!(client.write_bookmarks().is_ok());
        let key: String = client.key.clone();
        // Re-initialize a client
        let client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Verify it loaded parameters correctly
        assert_eq!(client.key, key);
        let bookmark: (String, u16, FileTransferProtocol, String, Option<String>) =
            client.get_bookmark(&String::from("raspberry")).unwrap();
        assert_eq!(bookmark.0, String::from("192.168.1.31"));
        assert_eq!(bookmark.1, 22);
        assert_eq!(bookmark.2, FileTransferProtocol::Sftp);
        assert_eq!(bookmark.3, String::from("pi"));
        assert_eq!(*bookmark.4.as_ref().unwrap(), String::from("mypassword"));
        let bookmark: (String, u16, FileTransferProtocol, String) =
            client.get_recent(&recent_key).unwrap();
        assert_eq!(bookmark.0, String::from("192.168.1.31"));
        assert_eq!(bookmark.1, 22);
        assert_eq!(bookmark.2, FileTransferProtocol::Sftp);
        assert_eq!(bookmark.3, String::from("pi"));
    }

    #[test]

    fn test_system_bookmarks_manipulate_bookmarks() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Add bookmark
        client.add_bookmark(
            String::from("raspberry"),
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
            Some(String::from("mypassword")),
        );
        client.add_bookmark(
            String::from("raspberry2"),
            String::from("192.168.1.32"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
            Some(String::from("mypassword2")),
        );
        // Iter
        assert_eq!(client.iter_bookmarks().count(), 2);
        // Get bookmark
        let bookmark: (String, u16, FileTransferProtocol, String, Option<String>) =
            client.get_bookmark(&String::from("raspberry")).unwrap();
        assert_eq!(bookmark.0, String::from("192.168.1.31"));
        assert_eq!(bookmark.1, 22);
        assert_eq!(bookmark.2, FileTransferProtocol::Sftp);
        assert_eq!(bookmark.3, String::from("pi"));
        assert_eq!(*bookmark.4.as_ref().unwrap(), String::from("mypassword"));
        // Write bookmarks
        assert!(client.write_bookmarks().is_ok());
        // Delete bookmark
        client.del_bookmark(&String::from("raspberry"));
        // Get unexisting bookmark
        assert!(client.get_bookmark(&String::from("raspberry")).is_none());
        // Write bookmarks
        assert!(client.write_bookmarks().is_ok());
    }

    #[test]
    #[should_panic]

    fn test_system_bookmarks_bad_bookmark_name() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Add bookmark
        client.add_bookmark(
            String::from(""),
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
            Some(String::from("mypassword")),
        );
    }

    #[test]

    fn test_system_bookmarks_manipulate_recents() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Add bookmark
        client.add_recent(
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
        );
        // Iter
        assert_eq!(client.iter_recents().count(), 1);
        let key: String = String::from(client.iter_recents().next().unwrap());
        // Get bookmark
        let bookmark: (String, u16, FileTransferProtocol, String) =
            client.get_recent(&key).unwrap();
        assert_eq!(bookmark.0, String::from("192.168.1.31"));
        assert_eq!(bookmark.1, 22);
        assert_eq!(bookmark.2, FileTransferProtocol::Sftp);
        assert_eq!(bookmark.3, String::from("pi"));
        // Write bookmarks
        assert!(client.write_bookmarks().is_ok());
        // Delete bookmark
        client.del_recent(&key);
        // Get unexisting bookmark
        assert!(client.get_bookmark(&key).is_none());
        // Write bookmarks
        assert!(client.write_bookmarks().is_ok());
    }

    #[test]

    fn test_system_bookmarks_dup_recent() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Add bookmark
        client.add_recent(
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
        );
        client.add_recent(
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
        );
        // There should be only one recent
        assert_eq!(client.iter_recents().count(), 1);
    }

    #[test]

    fn test_system_bookmarks_recents_more_than_limit() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 2).unwrap();
        // Add recent, wait 1 second for each one (cause the name depends on time)
        // 1
        client.add_recent(
            String::from("192.168.1.1"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
        );
        sleep(Duration::from_secs(1));
        // 2
        client.add_recent(
            String::from("192.168.1.2"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
        );
        sleep(Duration::from_secs(1));
        // 3
        client.add_recent(
            String::from("192.168.1.3"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
        );
        // Limit is 2
        assert_eq!(client.iter_recents().count(), 2);
        // Check that 192.168.1.1 has been removed
        let key: String = client.iter_recents().nth(0).unwrap().to_string();
        assert!(matches!(
            client.hosts.recents.get(&key).unwrap().address.as_str(),
            "192.168.1.2" | "192.168.1.3"
        ));
        let key: String = client.iter_recents().nth(1).unwrap().to_string();
        assert!(matches!(
            client.hosts.recents.get(&key).unwrap().address.as_str(),
            "192.168.1.2" | "192.168.1.3"
        ));
    }

    #[test]
    #[should_panic]
    fn test_system_bookmarks_add_bookmark_empty() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        // Add bookmark
        client.add_bookmark(
            String::from(""),
            String::from("192.168.1.31"),
            22,
            FileTransferProtocol::Sftp,
            String::from("pi"),
            Some(String::from("mypassword")),
        );
    }

    #[test]
    fn test_system_bookmarks_decrypt_str() {
        let tmp_dir: tempfile::TempDir = TempDir::new().ok().unwrap();
        let (cfg_path, key_path): (PathBuf, PathBuf) = get_paths(tmp_dir.path());
        // Initialize a new bookmarks client
        let mut client: BookmarksClient =
            BookmarksClient::new(cfg_path.as_path(), key_path.as_path(), 16).unwrap();
        client.key = "MYSUPERSECRETKEY".to_string();
        assert_eq!(
            client.decrypt_str("z4Z6LpcpYqBW4+bkIok+5A==").ok().unwrap(),
            "Hello world!"
        );
        assert!(client.decrypt_str("bidoof").is_err());
    }

    /// ### get_paths
    ///
    /// Get paths for configuration and key for bookmarks
    fn get_paths(dir: &Path) -> (PathBuf, PathBuf) {
        let k: PathBuf = PathBuf::from(dir);
        let mut c: PathBuf = k.clone();
        c.push("bookmarks.toml");
        (c, k)
    }
}
