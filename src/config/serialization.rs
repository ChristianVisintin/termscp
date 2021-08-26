//! ## Serialization
//!
//! `serialization` provides serialization and deserialization for configurations

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
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
use thiserror::Error;

/// ## SerializerError
///
/// Contains the error for serializer/deserializer
#[derive(std::fmt::Debug)]
pub struct SerializerError {
    kind: SerializerErrorKind,
    msg: Option<String>,
}

/// ## SerializerErrorKind
///
/// Describes the kind of error for the serializer/deserializer
#[derive(Error, Debug)]
pub enum SerializerErrorKind {
    #[error("Operation failed")]
    Generic,
    #[error("IO error")]
    Io,
    #[error("Serialization error")]
    Serialization,
    #[error("Syntax error")]
    Syntax,
}

impl SerializerError {
    /// ### new
    ///
    /// Instantiate a new `SerializerError`
    pub fn new(kind: SerializerErrorKind) -> SerializerError {
        SerializerError { kind, msg: None }
    }

    /// ### new_ex
    ///
    /// Instantiates a new `SerializerError` with description message
    pub fn new_ex(kind: SerializerErrorKind, msg: String) -> SerializerError {
        let mut err: SerializerError = SerializerError::new(kind);
        err.msg = Some(msg);
        err
    }
}

impl std::fmt::Display for SerializerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.msg {
            Some(msg) => write!(f, "{} ({})", self.kind, msg),
            None => write!(f, "{}", self.kind),
        }
    }
}

/// ### serialize
///
/// Serialize `UserHosts` into TOML and write content to writable
pub fn serialize<S>(serializable: &S, mut writable: Box<dyn Write>) -> Result<(), SerializerError>
where
    S: Serialize + Sized,
{
    // Serialize content
    let data: String = match toml::ser::to_string(serializable) {
        Ok(dt) => dt,
        Err(err) => {
            return Err(SerializerError::new_ex(
                SerializerErrorKind::Serialization,
                err.to_string(),
            ))
        }
    };
    trace!("Serialized new bookmarks data: {}", data);
    // Write file
    match writable.write_all(data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(err) => Err(SerializerError::new_ex(
            SerializerErrorKind::Io,
            err.to_string(),
        )),
    }
}

/// ### deserialize
///
/// Read data from readable and deserialize its content as TOML
pub fn deserialize<S>(mut readable: Box<dyn Read>) -> Result<S, SerializerError>
where
    S: DeserializeOwned + Sized + std::fmt::Debug,
{
    // Read file content
    let mut data: String = String::new();
    if let Err(err) = readable.read_to_string(&mut data) {
        return Err(SerializerError::new_ex(
            SerializerErrorKind::Io,
            err.to_string(),
        ));
    }
    trace!("Read bookmarks from file: {}", data);
    // Deserialize
    match toml::de::from_str(data.as_str()) {
        Ok(deserialized) => {
            debug!("Read bookmarks from file {:?}", deserialized);
            Ok(deserialized)
        }
        Err(err) => Err(SerializerError::new_ex(
            SerializerErrorKind::Syntax,
            err.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::io::{Seek, SeekFrom};
    use std::path::PathBuf;
    use tuirealm::tui::style::Color;

    use crate::config::bookmarks::{Bookmark, UserHosts};
    use crate::config::params::UserConfig;
    use crate::config::themes::Theme;
    use crate::utils::test_helpers::create_file_ioers;

    #[test]
    fn test_config_serialization_errors() {
        let error: SerializerError = SerializerError::new(SerializerErrorKind::Syntax);
        assert!(error.msg.is_none());
        assert_eq!(format!("{}", error), String::from("Syntax error"));
        let error: SerializerError =
            SerializerError::new_ex(SerializerErrorKind::Syntax, String::from("bad syntax"));
        assert!(error.msg.is_some());
        assert_eq!(
            format!("{}", error),
            String::from("Syntax error (bad syntax)")
        );
        // Fmt
        assert_eq!(
            format!("{}", SerializerError::new(SerializerErrorKind::Generic)),
            String::from("Operation failed")
        );
        assert_eq!(
            format!("{}", SerializerError::new(SerializerErrorKind::Io)),
            String::from("IO error")
        );
        assert_eq!(
            format!(
                "{}",
                SerializerError::new(SerializerErrorKind::Serialization)
            ),
            String::from("Serialization error")
        );
    }

    // -- Serialization of params

    #[test]
    fn test_config_serialization_params_deserialize_ok() {
        let toml_file: tempfile::NamedTempFile = create_good_toml_bookmarks_params();
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        // Parse
        let cfg = deserialize(Box::new(toml_file));
        assert!(cfg.is_ok());
        let cfg: UserConfig = cfg.ok().unwrap();
        // Verify configuration
        // Verify ui
        assert_eq!(cfg.user_interface.default_protocol, String::from("SCP"));
        assert_eq!(cfg.user_interface.text_editor, PathBuf::from("vim"));
        assert_eq!(cfg.user_interface.show_hidden_files, true);
        assert_eq!(cfg.user_interface.check_for_updates.unwrap(), true);
        assert_eq!(cfg.user_interface.group_dirs, Some(String::from("last")));
        assert_eq!(
            cfg.user_interface.file_fmt,
            Some(String::from("{NAME} {PEX}"))
        );
        assert_eq!(
            cfg.user_interface.remote_file_fmt,
            Some(String::from("{NAME} {USER}")),
        );
        // Verify keys
        assert_eq!(
            *cfg.remote
                .ssh_keys
                .get(&String::from("192.168.1.31"))
                .unwrap(),
            PathBuf::from("/home/omar/.ssh/raspberry.key")
        );
        assert_eq!(
            *cfg.remote
                .ssh_keys
                .get(&String::from("192.168.1.32"))
                .unwrap(),
            PathBuf::from("/home/omar/.ssh/beaglebone.key")
        );
        assert!(cfg.remote.ssh_keys.get(&String::from("1.1.1.1")).is_none());
    }

    #[test]
    fn test_config_serialization_params_deserialize_ok_no_opts() {
        let toml_file: tempfile::NamedTempFile = create_good_toml_bookmarks_params_no_opts();
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        // Parse
        let cfg = deserialize(Box::new(toml_file));
        assert!(cfg.is_ok());
        let cfg: UserConfig = cfg.ok().unwrap();
        // Verify configuration
        // Verify ui
        assert_eq!(cfg.user_interface.default_protocol, String::from("SCP"));
        assert_eq!(cfg.user_interface.text_editor, PathBuf::from("vim"));
        assert_eq!(cfg.user_interface.show_hidden_files, true);
        assert_eq!(cfg.user_interface.group_dirs, None);
        assert!(cfg.user_interface.check_for_updates.is_none());
        assert!(cfg.user_interface.file_fmt.is_none());
        assert!(cfg.user_interface.remote_file_fmt.is_none());
        // Verify keys
        assert_eq!(
            *cfg.remote
                .ssh_keys
                .get(&String::from("192.168.1.31"))
                .unwrap(),
            PathBuf::from("/home/omar/.ssh/raspberry.key")
        );
        assert_eq!(
            *cfg.remote
                .ssh_keys
                .get(&String::from("192.168.1.32"))
                .unwrap(),
            PathBuf::from("/home/omar/.ssh/beaglebone.key")
        );
        assert!(cfg.remote.ssh_keys.get(&String::from("1.1.1.1")).is_none());
    }

    #[test]
    fn test_config_serialization_params_deserialize_nok() {
        let toml_file: tempfile::NamedTempFile = create_bad_toml_bookmarks_params();
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        // Parse
        assert!(deserialize::<UserConfig>(Box::new(toml_file)).is_err());
    }

    #[test]
    fn test_config_serialization_params_serialize() {
        let mut cfg: UserConfig = UserConfig::default();
        let toml_file: tempfile::NamedTempFile = tempfile::NamedTempFile::new().ok().unwrap();
        // Insert key
        cfg.remote.ssh_keys.insert(
            String::from("192.168.1.31"),
            PathBuf::from("/home/omar/.ssh/id_rsa"),
        );
        // Serialize
        let writer: Box<dyn Write> = Box::new(std::fs::File::create(toml_file.path()).unwrap());
        assert!(serialize(&cfg, writer).is_ok());
        // Reload configuration and check if it's ok
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        assert!(deserialize::<UserConfig>(Box::new(toml_file)).is_ok());
    }

    #[test]
    fn test_config_serialization_params_fail_write() {
        let toml_file: tempfile::NamedTempFile = tempfile::NamedTempFile::new().ok().unwrap();
        let writer: Box<dyn Write> = Box::new(std::fs::File::open(toml_file.path()).unwrap());
        // Try to write unexisting file
        let cfg: UserConfig = UserConfig::default();
        assert!(serialize(&cfg, writer).is_err());
    }

    #[test]
    fn test_config_serialization_params_fail_read() {
        let toml_file: tempfile::NamedTempFile = tempfile::NamedTempFile::new().ok().unwrap();
        let reader: Box<dyn Read> = Box::new(std::fs::File::open(toml_file.path()).unwrap());
        // Try to write unexisting file
        assert!(deserialize::<UserConfig>(reader).is_err());
    }

    fn create_good_toml_bookmarks_params() -> tempfile::NamedTempFile {
        // Write
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_content: &str = r#"
        [user_interface]
        default_protocol = "SCP"
        text_editor = "vim"
        show_hidden_files = true
        check_for_updates = true
        group_dirs = "last"
        file_fmt = "{NAME} {PEX}"
        remote_file_fmt = "{NAME} {USER}"

        [remote.ssh_keys]
        "192.168.1.31" = "/home/omar/.ssh/raspberry.key"
        "192.168.1.32" = "/home/omar/.ssh/beaglebone.key"
        "#;
        tmpfile.write_all(file_content.as_bytes()).unwrap();
        tmpfile
    }

    fn create_good_toml_bookmarks_params_no_opts() -> tempfile::NamedTempFile {
        // Write
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_content: &str = r#"
        [user_interface]
        default_protocol = "SCP"
        text_editor = "vim"
        show_hidden_files = true

        [remote.ssh_keys]
        "192.168.1.31" = "/home/omar/.ssh/raspberry.key"
        "192.168.1.32" = "/home/omar/.ssh/beaglebone.key"
        "#;
        tmpfile.write_all(file_content.as_bytes()).unwrap();
        tmpfile
    }

    fn create_bad_toml_bookmarks_params() -> tempfile::NamedTempFile {
        // Write
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_content: &str = r#"
        [user_interface]
        default_protocol = "SFTP"

        [remote.ssh_keys]
        "192.168.1.31" = "/home/omar/.ssh/raspberry.key"
        "#;
        tmpfile.write_all(file_content.as_bytes()).unwrap();
        tmpfile
    }

    // -- bookmarks

    #[test]
    fn test_config_serializer_bookmarks_serializer_deserialize_ok() {
        let toml_file: tempfile::NamedTempFile = create_good_toml_bookmarks();
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        // Parse
        let hosts = deserialize(Box::new(toml_file));
        assert!(hosts.is_ok());
        let hosts: UserHosts = hosts.ok().unwrap();
        // Verify hosts
        // Verify recents
        assert_eq!(hosts.recents.len(), 1);
        let host: &Bookmark = hosts.recents.get("ISO20201215T094000Z").unwrap();
        assert_eq!(host.address, String::from("172.16.104.10"));
        assert_eq!(host.port, 22);
        assert_eq!(host.protocol, String::from("SCP"));
        assert_eq!(host.username, String::from("root"));
        assert_eq!(host.password, None);
        // Verify bookmarks
        assert_eq!(hosts.bookmarks.len(), 3);
        let host: &Bookmark = hosts.bookmarks.get("raspberrypi2").unwrap();
        assert_eq!(host.address, String::from("192.168.1.31"));
        assert_eq!(host.port, 22);
        assert_eq!(host.protocol, String::from("SFTP"));
        assert_eq!(host.username, String::from("root"));
        assert_eq!(*host.password.as_ref().unwrap(), String::from("mypassword"));
        let host: &Bookmark = hosts.bookmarks.get("msi-estrem").unwrap();
        assert_eq!(host.address, String::from("192.168.1.30"));
        assert_eq!(host.port, 22);
        assert_eq!(host.protocol, String::from("SFTP"));
        assert_eq!(host.username, String::from("cvisintin"));
        assert_eq!(*host.password.as_ref().unwrap(), String::from("mysecret"));
        let host: &Bookmark = hosts.bookmarks.get("aws-server-prod1").unwrap();
        assert_eq!(host.address, String::from("51.23.67.12"));
        assert_eq!(host.port, 21);
        assert_eq!(host.protocol, String::from("FTPS"));
        assert_eq!(host.username, String::from("aws001"));
        assert_eq!(host.password, None);
    }

    #[test]
    fn test_config_serializer_bookmarks_serializer_deserialize_nok() {
        let toml_file: tempfile::NamedTempFile = create_bad_toml_bookmarks();
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        // Parse
        assert!(deserialize::<UserHosts>(Box::new(toml_file)).is_err());
    }

    #[test]
    fn test_config_serializer_bookmarks_serializer_serialize() {
        let mut bookmarks: HashMap<String, Bookmark> = HashMap::with_capacity(2);
        // Push two samples
        bookmarks.insert(
            String::from("raspberrypi2"),
            Bookmark {
                address: String::from("192.168.1.31"),
                port: 22,
                protocol: String::from("SFTP"),
                username: String::from("root"),
                password: None,
            },
        );
        bookmarks.insert(
            String::from("msi-estrem"),
            Bookmark {
                address: String::from("192.168.1.30"),
                port: 4022,
                protocol: String::from("SFTP"),
                username: String::from("cvisintin"),
                password: Some(String::from("password")),
            },
        );
        let mut recents: HashMap<String, Bookmark> = HashMap::with_capacity(1);
        recents.insert(
            String::from("ISO20201215T094000Z"),
            Bookmark {
                address: String::from("192.168.1.254"),
                port: 3022,
                protocol: String::from("SCP"),
                username: String::from("omar"),
                password: Some(String::from("aaa")),
            },
        );
        let tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        // Serialize
        let hosts: UserHosts = UserHosts { bookmarks, recents };
        assert!(serialize(&hosts, Box::new(tmpfile)).is_ok());
    }

    #[test]
    fn test_config_serialization_theme_serialize() {
        let mut theme: Theme = Theme::default();
        theme.auth_address = Color::Rgb(240, 240, 240);
        let tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let (reader, writer) = create_file_ioers(tmpfile.path());
        assert!(serialize(&theme, Box::new(writer)).is_ok());
        // Try to deserialize
        let deserialized_theme: Theme = deserialize(Box::new(reader)).ok().unwrap();
        assert_eq!(theme, deserialized_theme);
    }

    #[test]
    fn test_config_serialization_theme_deserialize() {
        let toml_file = create_good_toml_theme();
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        assert!(deserialize::<Theme>(Box::new(toml_file)).is_ok());
        let toml_file = create_bad_toml_theme();
        toml_file.as_file().sync_all().unwrap();
        toml_file.as_file().seek(SeekFrom::Start(0)).unwrap();
        assert!(deserialize::<Theme>(Box::new(toml_file)).is_err());
    }

    fn create_good_toml_bookmarks() -> tempfile::NamedTempFile {
        // Write
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_content: &str = r#"
        [bookmarks]
        raspberrypi2 = { address = "192.168.1.31", port = 22, protocol = "SFTP", username = "root", password = "mypassword" }
        msi-estrem = { address = "192.168.1.30", port = 22, protocol = "SFTP", username = "cvisintin", password = "mysecret" }
        aws-server-prod1 = { address = "51.23.67.12", port = 21, protocol = "FTPS", username = "aws001" }

        [recents]
        ISO20201215T094000Z = { address = "172.16.104.10", port = 22, protocol = "SCP", username = "root" }
        "#;
        tmpfile.write_all(file_content.as_bytes()).unwrap();
        //write!(tmpfile, "[bookmarks]\nraspberrypi2 = {{ address = \"192.168.1.31\", port = 22, protocol = \"SFTP\", username = \"root\" }}\nmsi-estrem = {{ address = \"192.168.1.30\", port = 22, protocol = \"SFTP\", username = \"cvisintin\" }}\naws-server-prod1 = {{ address = \"51.23.67.12\", port = 21, protocol = \"FTPS\", username = \"aws001\" }}\n\n[recents]\nISO20201215T094000Z = {{ address = \"172.16.104.10\", port = 22, protocol = \"SCP\", username = \"root\" }}\n");
        tmpfile
    }

    fn create_bad_toml_bookmarks() -> tempfile::NamedTempFile {
        // Write
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_content: &str = r#"
        [bookmarks]
        raspberrypi2 = { address = "192.168.1.31", port = 22, protocol = "SFTP", username = "root"}
        msi-estrem = { address = "192.168.1.30", port = 22, protocol = "SFTP" }
        aws-server-prod1 = { address = "51.23.67.12", port = 21, protocol = "FTPS", username = "aws001" }

        [recents]
        ISO20201215T094000Z = { address = "172.16.104.10", protocol = "SCP", username = "root", port = 22 }
        "#;
        tmpfile.write_all(file_content.as_bytes()).unwrap();
        tmpfile
    }

    fn create_good_toml_theme() -> tempfile::NamedTempFile {
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_content: &str = r##"auth_address = "Yellow"
        auth_bookmarks = "LightGreen"
        auth_password = "LightBlue"
        auth_port = "LightCyan"
        auth_protocol = "LightGreen"
        auth_recents = "LightBlue"
        auth_username = "LightMagenta"
        misc_error_dialog = "Red"
        misc_input_dialog = "240,240,240"
        misc_keys = "Cyan"
        misc_quit_dialog = "Yellow"
        misc_save_dialog = "Cyan"
        misc_warn_dialog = "LightRed"
        transfer_local_explorer_background = "rgb(240, 240, 240)"
        transfer_local_explorer_foreground = "rgb(60, 60, 60)"
        transfer_local_explorer_highlighted = "Yellow"
        transfer_log_background = "255, 255, 255"
        transfer_log_window = "LightGreen"
        transfer_progress_bar_full = "forestgreen"
        transfer_progress_bar_partial = "Green"
        transfer_remote_explorer_background = "#f0f0f0"
        transfer_remote_explorer_foreground = "rgb(40, 40, 40)"
        transfer_remote_explorer_highlighted = "LightBlue"
        transfer_status_hidden = "LightBlue"
        transfer_status_sorting = "LightYellow"
        transfer_status_sync_browsing = "LightGreen"
        "##;
        tmpfile.write_all(file_content.as_bytes()).unwrap();
        tmpfile
    }

    fn create_bad_toml_theme() -> tempfile::NamedTempFile {
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        let file_content: &str = r#"
        auth_address = "Yellow"
        auth_bookmarks = "LightGreen"
        auth_password = "LightBlue"
        auth_port = "LightCyan"
        auth_protocol = "LightGreen"
        auth_recents = "LightBlue"
        auth_username = "LightMagenta"
        misc_error_dialog = "Red"
        misc_input_dialog = "240,240,240"
        misc_keys = "Cyan"
        misc_quit_dialog = "Yellow"
        misc_warn_dialog = "LightRed"
        transfer_local_explorer_text = "rgb(240, 240, 240)"
        transfer_local_explorer_window = "Yellow"
        transfer_log_text = "255, 255, 255"
        transfer_log_window = "LightGreen"
        transfer_progress_bar = "Green"
        transfer_remote_explorer_text = "verdazzurro"
        transfer_remote_explorer_window = "LightBlue"
        transfer_status_hidden = "LightBlue"
        transfer_status_sorting = "LightYellow"
        transfer_status_sync_browsing = "LightGreen"
        "#;
        tmpfile.write_all(file_content.as_bytes()).unwrap();
        tmpfile
    }
}
