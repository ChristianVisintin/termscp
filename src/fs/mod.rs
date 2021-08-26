//! ## Fs
//!
//! `fs` is the module which provides file system entities

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
// Mod
pub mod explorer;
// Ext
use std::path::PathBuf;
use std::time::SystemTime;

/// ## FsEntry
///
/// FsEntry represents a generic entry in a directory

#[derive(Clone, std::fmt::Debug)]
pub enum FsEntry {
    Directory(FsDirectory),
    File(FsFile),
}

/// ## FsDirectory
///
/// Directory provides an interface to file system directories

#[derive(Clone, std::fmt::Debug)]
pub struct FsDirectory {
    pub name: String,
    pub abs_path: PathBuf,
    pub last_change_time: SystemTime,
    pub last_access_time: SystemTime,
    pub creation_time: SystemTime,
    pub symlink: Option<Box<FsEntry>>,                 // UNIX only
    pub user: Option<u32>,                             // UNIX only
    pub group: Option<u32>,                            // UNIX only
    pub unix_pex: Option<(UnixPex, UnixPex, UnixPex)>, // UNIX only
}

/// ### FsFile
///
/// FsFile provides an interface to file system files

#[derive(Clone, std::fmt::Debug)]
pub struct FsFile {
    pub name: String,
    pub abs_path: PathBuf,
    pub last_change_time: SystemTime,
    pub last_access_time: SystemTime,
    pub creation_time: SystemTime,
    pub size: usize,
    pub ftype: Option<String>,                         // File type
    pub symlink: Option<Box<FsEntry>>,                 // UNIX only
    pub user: Option<u32>,                             // UNIX only
    pub group: Option<u32>,                            // UNIX only
    pub unix_pex: Option<(UnixPex, UnixPex, UnixPex)>, // UNIX only
}

/// ## UnixPex
///
/// Describes the permissions on POSIX system.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UnixPex {
    read: bool,
    write: bool,
    execute: bool,
}

impl UnixPex {
    /// ### new
    ///
    /// Instantiates a new `UnixPex`
    pub fn new(read: bool, write: bool, execute: bool) -> Self {
        Self {
            read,
            write,
            execute,
        }
    }

    /// ### can_read
    ///
    /// Returns whether user can read
    pub fn can_read(&self) -> bool {
        self.read
    }

    /// ### can_write
    ///
    /// Returns whether user can write
    pub fn can_write(&self) -> bool {
        self.write
    }

    /// ### can_execute
    ///
    /// Returns whether user can execute
    pub fn can_execute(&self) -> bool {
        self.execute
    }

    /// ### as_byte
    ///
    /// Convert permission to byte as on POSIX systems
    pub fn as_byte(&self) -> u8 {
        ((self.read as u8) << 2) + ((self.write as u8) << 1) + (self.execute as u8)
    }
}

impl From<u8> for UnixPex {
    fn from(bits: u8) -> Self {
        Self {
            read: ((bits >> 2) & 0x01) != 0,
            write: ((bits >> 1) & 0x01) != 0,
            execute: (bits & 0x01) != 0,
        }
    }
}

impl FsEntry {
    /// ### get_abs_path
    ///
    /// Get absolute path from `FsEntry`
    pub fn get_abs_path(&self) -> PathBuf {
        match self {
            FsEntry::Directory(dir) => dir.abs_path.clone(),
            FsEntry::File(file) => file.abs_path.clone(),
        }
    }

    /// ### get_name
    ///
    /// Get file name from `FsEntry`
    pub fn get_name(&self) -> &'_ str {
        match self {
            FsEntry::Directory(dir) => dir.name.as_ref(),
            FsEntry::File(file) => file.name.as_ref(),
        }
    }

    /// ### get_last_change_time
    ///
    /// Get last change time from `FsEntry`
    pub fn get_last_change_time(&self) -> SystemTime {
        match self {
            FsEntry::Directory(dir) => dir.last_change_time,
            FsEntry::File(file) => file.last_change_time,
        }
    }

    /// ### get_last_access_time
    ///
    /// Get access time from `FsEntry`
    pub fn get_last_access_time(&self) -> SystemTime {
        match self {
            FsEntry::Directory(dir) => dir.last_access_time,
            FsEntry::File(file) => file.last_access_time,
        }
    }

    /// ### get_creation_time
    ///
    /// Get creation time from `FsEntry`
    pub fn get_creation_time(&self) -> SystemTime {
        match self {
            FsEntry::Directory(dir) => dir.creation_time,
            FsEntry::File(file) => file.creation_time,
        }
    }

    /// ### get_size
    ///
    /// Get size from `FsEntry`. For directories is always 4096
    pub fn get_size(&self) -> usize {
        match self {
            FsEntry::Directory(_) => 4096,
            FsEntry::File(file) => file.size,
        }
    }

    /// ### get_ftype
    ///
    /// Get file type from `FsEntry`. For directories is always None
    pub fn get_ftype(&self) -> Option<String> {
        match self {
            FsEntry::Directory(_) => None,
            FsEntry::File(file) => file.ftype.clone(),
        }
    }

    /// ### get_user
    ///
    /// Get uid from `FsEntry`
    pub fn get_user(&self) -> Option<u32> {
        match self {
            FsEntry::Directory(dir) => dir.user,
            FsEntry::File(file) => file.user,
        }
    }

    /// ### get_group
    ///
    /// Get gid from `FsEntry`
    pub fn get_group(&self) -> Option<u32> {
        match self {
            FsEntry::Directory(dir) => dir.group,
            FsEntry::File(file) => file.group,
        }
    }

    /// ### get_unix_pex
    ///
    /// Get unix pex from `FsEntry`
    pub fn get_unix_pex(&self) -> Option<(UnixPex, UnixPex, UnixPex)> {
        match self {
            FsEntry::Directory(dir) => dir.unix_pex,
            FsEntry::File(file) => file.unix_pex,
        }
    }

    /// ### is_symlink
    ///
    /// Returns whether the `FsEntry` is a symlink
    pub fn is_symlink(&self) -> bool {
        match self {
            FsEntry::Directory(dir) => dir.symlink.is_some(),
            FsEntry::File(file) => file.symlink.is_some(),
        }
    }

    /// ### is_dir
    ///
    /// Returns whether a FsEntry is a directory
    pub fn is_dir(&self) -> bool {
        matches!(self, FsEntry::Directory(_))
    }

    /// ### is_file
    ///
    /// Returns whether a FsEntry is a File
    pub fn is_file(&self) -> bool {
        matches!(self, FsEntry::File(_))
    }

    /// ### is_hidden
    ///
    /// Returns whether FsEntry is hidden
    pub fn is_hidden(&self) -> bool {
        self.get_name().starts_with('.')
    }

    /// ### get_realfile
    ///
    /// Return the real file pointed by a `FsEntry`
    pub fn get_realfile(&self) -> FsEntry {
        match self {
            FsEntry::Directory(dir) => match &dir.symlink {
                Some(symlink) => symlink.get_realfile(),
                None => self.clone(),
            },
            FsEntry::File(file) => match &file.symlink {
                Some(symlink) => symlink.get_realfile(),
                None => self.clone(),
            },
        }
    }

    /// ### unwrap_file
    ///
    /// Unwrap FsEntry as FsFile
    pub fn unwrap_file(self) -> FsFile {
        match self {
            FsEntry::File(file) => file,
            _ => panic!("unwrap_file: not a file"),
        }
    }

    #[cfg(test)]
    /// ### unwrap_dir
    ///
    /// Unwrap FsEntry as FsDirectory
    pub fn unwrap_dir(self) -> FsDirectory {
        match self {
            FsEntry::Directory(dir) => dir,
            _ => panic!("unwrap_dir: not a directory"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_fs_fsentry_dir() {
        let t_now: SystemTime = SystemTime::now();
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("foo"),
            abs_path: PathBuf::from("/foo"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))), // UNIX only
        });
        assert_eq!(entry.get_abs_path(), PathBuf::from("/foo"));
        assert_eq!(entry.get_name(), String::from("foo"));
        assert_eq!(entry.get_last_access_time(), t_now);
        assert_eq!(entry.get_last_change_time(), t_now);
        assert_eq!(entry.get_creation_time(), t_now);
        assert_eq!(entry.get_size(), 4096);
        assert_eq!(entry.get_ftype(), None);
        assert_eq!(entry.get_user(), Some(0));
        assert_eq!(entry.get_group(), Some(0));
        assert_eq!(entry.is_symlink(), false);
        assert_eq!(entry.is_dir(), true);
        assert_eq!(entry.is_file(), false);
        assert_eq!(
            entry.get_unix_pex(),
            Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5)))
        );
        assert_eq!(entry.unwrap_dir().abs_path, PathBuf::from("/foo"));
    }

    #[test]
    fn test_fs_fsentry_file() {
        let t_now: SystemTime = SystemTime::now();
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        assert_eq!(entry.get_abs_path(), PathBuf::from("/bar.txt"));
        assert_eq!(entry.get_name(), String::from("bar.txt"));
        assert_eq!(entry.get_last_access_time(), t_now);
        assert_eq!(entry.get_last_change_time(), t_now);
        assert_eq!(entry.get_creation_time(), t_now);
        assert_eq!(entry.get_size(), 8192);
        assert_eq!(entry.get_ftype(), Some(String::from("txt")));
        assert_eq!(entry.get_user(), Some(0));
        assert_eq!(entry.get_group(), Some(0));
        assert_eq!(
            entry.get_unix_pex(),
            Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4)))
        );
        assert_eq!(entry.is_symlink(), false);
        assert_eq!(entry.is_dir(), false);
        assert_eq!(entry.is_file(), true);
        assert_eq!(entry.unwrap_file().abs_path, PathBuf::from("/bar.txt"));
    }

    #[test]
    #[should_panic]
    fn test_fs_fsentry_file_unwrap_bad() {
        let t_now: SystemTime = SystemTime::now();
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        entry.unwrap_dir();
    }

    #[test]
    #[should_panic]
    fn test_fs_fsentry_dir_unwrap_bad() {
        let t_now: SystemTime = SystemTime::now();
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("foo"),
            abs_path: PathBuf::from("/foo"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))), // UNIX only
        });
        entry.unwrap_file();
    }

    #[test]
    fn test_fs_fsentry_hidden_files() {
        let t_now: SystemTime = SystemTime::now();
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        assert_eq!(entry.is_hidden(), false);
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from(".gitignore"),
            abs_path: PathBuf::from("/.gitignore"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        assert_eq!(entry.is_hidden(), true);
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from(".git"),
            abs_path: PathBuf::from("/.git"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))), // UNIX only
        });
        assert_eq!(entry.is_hidden(), true);
    }

    #[test]
    fn test_fs_fsentry_realfile_none() {
        let t_now: SystemTime = SystemTime::now();
        // With file...
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        // Symlink is None...
        assert_eq!(
            entry.get_realfile().get_abs_path(),
            PathBuf::from("/bar.txt")
        );
        // With directory...
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("foo"),
            abs_path: PathBuf::from("/foo"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))), // UNIX only
        });
        assert_eq!(entry.get_realfile().get_abs_path(), PathBuf::from("/foo"));
    }

    #[test]
    fn test_fs_fsentry_realfile_some() {
        let t_now: SystemTime = SystemTime::now();
        // Prepare entries
        // root -> child -> target
        let entry_target: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("projects"),
            abs_path: PathBuf::from("/home/cvisintin/projects"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(7), UnixPex::from(7))), // UNIX only
        });
        let entry_child: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("projects"),
            abs_path: PathBuf::from("/develop/projects"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: Some(Box::new(entry_target)),
            user: Some(0),
            group: Some(0),
            unix_pex: Some((UnixPex::from(7), UnixPex::from(7), UnixPex::from(7))),
        });
        let entry_root: FsEntry = FsEntry::File(FsFile {
            name: String::from("projects"),
            abs_path: PathBuf::from("/projects"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            size: 8,
            ftype: None,
            symlink: Some(Box::new(entry_child)),
            user: Some(0),
            group: Some(0),
            unix_pex: Some((UnixPex::from(7), UnixPex::from(7), UnixPex::from(7))),
        });
        assert_eq!(entry_root.is_symlink(), true);
        // get real file
        let real_file: FsEntry = entry_root.get_realfile();
        // real file must be projects in /home/cvisintin
        assert_eq!(
            real_file.get_abs_path(),
            PathBuf::from("/home/cvisintin/projects")
        );
    }

    #[test]
    fn unix_pex() {
        let pex: UnixPex = UnixPex::from(4);
        assert_eq!(pex.can_read(), true);
        assert_eq!(pex.can_write(), false);
        assert_eq!(pex.can_execute(), false);
        let pex: UnixPex = UnixPex::from(0);
        assert_eq!(pex.can_read(), false);
        assert_eq!(pex.can_write(), false);
        assert_eq!(pex.can_execute(), false);
        let pex: UnixPex = UnixPex::from(3);
        assert_eq!(pex.can_read(), false);
        assert_eq!(pex.can_write(), true);
        assert_eq!(pex.can_execute(), true);
        let pex: UnixPex = UnixPex::from(7);
        assert_eq!(pex.can_read(), true);
        assert_eq!(pex.can_write(), true);
        assert_eq!(pex.can_execute(), true);
        let pex: UnixPex = UnixPex::from(3);
        assert_eq!(pex.as_byte(), 3);
        let pex: UnixPex = UnixPex::from(7);
        assert_eq!(pex.as_byte(), 7);
    }
}
