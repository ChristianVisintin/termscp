//! ## Formatter
//!
//! `formatter` is the module which provides formatting utilities for `FileExplorer`

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
use super::FsEntry;
use crate::utils::fmt::{fmt_path_elide, fmt_pex, fmt_time};
// Ext
use bytesize::ByteSize;
use regex::Regex;
#[cfg(target_family = "unix")]
use users::{get_group_by_gid, get_user_by_uid};
// Types
// FmtCallback: Formatter, fsentry: &FsEntry, cur_str, prefix, length, extra
type FmtCallback = fn(&Formatter, &FsEntry, &str, &str, Option<&usize>, Option<&String>) -> String;

// Keys
const FMT_KEY_ATIME: &str = "ATIME";
const FMT_KEY_CTIME: &str = "CTIME";
const FMT_KEY_GROUP: &str = "GROUP";
const FMT_KEY_MTIME: &str = "MTIME";
const FMT_KEY_NAME: &str = "NAME";
const FMT_KEY_PEX: &str = "PEX";
const FMT_KEY_SIZE: &str = "SIZE";
const FMT_KEY_SYMLINK: &str = "SYMLINK";
const FMT_KEY_USER: &str = "USER";
// Default
const FMT_DEFAULT_STX: &str = "{NAME} {PEX} {USER} {SIZE} {MTIME}";
// Regex
lazy_static! {
    /**
     * Regex matches:
     *  - group 0: KEY NAME
     *  - group 1?: LENGTH
     *  - group 2?: EXTRA
     */
    static ref FMT_KEY_REGEX: Regex = Regex::new(r"\{(.*?)\}").ok().unwrap();
    static ref FMT_ATTR_REGEX: Regex = Regex::new(r"(?:([A-Z]+))(:?([0-9]+))?(:?(.+))?").ok().unwrap();
}

/// ## CallChainBlock
///
/// Call Chain block is a block in a chain of functions which are called in order to format the FsEntry.
/// A callChain is instantiated starting from the Formatter syntax and the regex, once the groups are found
/// a chain of function is made using the Formatters method.
/// This method provides an extremely fast way to format fs entries
struct CallChainBlock {
    func: FmtCallback,
    prefix: String,
    fmt_len: Option<usize>,
    fmt_extra: Option<String>,
    next_block: Option<Box<CallChainBlock>>,
}

impl CallChainBlock {
    /// ### new
    ///
    /// Create a new `CallChainBlock`
    pub fn new(
        func: FmtCallback,
        prefix: String,
        fmt_len: Option<usize>,
        fmt_extra: Option<String>,
    ) -> Self {
        CallChainBlock {
            func,
            prefix,
            fmt_len,
            fmt_extra,
            next_block: None,
        }
    }

    /// ### next
    ///
    /// Call next callback in the CallChain
    pub fn next(&self, fmt: &Formatter, fsentry: &FsEntry, cur_str: &str) -> String {
        // Call func
        let new_str: String = (self.func)(
            fmt,
            fsentry,
            cur_str,
            self.prefix.as_str(),
            self.fmt_len.as_ref(),
            self.fmt_extra.as_ref(),
        );
        // If next is some, call next, otherwise (END OF CHAIN) return new_str
        match &self.next_block {
            Some(block) => block.next(fmt, fsentry, new_str.as_str()),
            None => new_str,
        }
    }

    /// ### push
    ///
    /// Push func to the last element in the Call chain
    pub fn push(
        &mut self,
        func: FmtCallback,
        prefix: String,
        fmt_len: Option<usize>,
        fmt_extra: Option<String>,
    ) {
        // Call recursively until an element with next_block equal to None is found
        match &mut self.next_block {
            None => {
                self.next_block = Some(Box::new(CallChainBlock::new(
                    func, prefix, fmt_len, fmt_extra,
                )))
            }
            Some(block) => block.push(func, prefix, fmt_len, fmt_extra),
        }
    }
}

/// ## Formatter
///
/// Formatter takes care of formatting FsEntries according to the provided keys.
/// Formatting is performed using the `CallChainBlock`, which composed makes a Call Chain. This method is extremely fast compared to match the format groups
/// at each fmt call.
pub struct Formatter {
    call_chain: CallChainBlock,
}

impl Default for Formatter {
    /// ### default
    ///
    /// Instantiates a Formatter with the default fmt syntax
    fn default() -> Self {
        Formatter {
            call_chain: Self::make_callchain(FMT_DEFAULT_STX),
        }
    }
}

impl Formatter {
    /// ### new
    ///
    /// Instantiates a new `Formatter` with the provided format string
    pub fn new(fmt_str: &str) -> Self {
        Formatter {
            call_chain: Self::make_callchain(fmt_str),
        }
    }

    /// ### fmt
    ///
    /// Format fsentry
    pub fn fmt(&self, fsentry: &FsEntry) -> String {
        // Execute callchain blocks
        self.call_chain.next(self, fsentry, "")
    }

    // Fmt methods

    /// ### fmt_atime
    ///
    /// Format last access time
    fn fmt_atime(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        fmt_len: Option<&usize>,
        fmt_extra: Option<&String>,
    ) -> String {
        // Get date (use extra args as format or default "%b %d %Y %H:%M")
        let datetime: String = fmt_time(
            fsentry.get_last_access_time(),
            match fmt_extra {
                Some(fmt) => fmt.as_ref(),
                None => "%b %d %Y %H:%M",
            },
        );
        // Add to cur str, prefix and the key value
        format!(
            "{}{}{:0width$}",
            cur_str,
            prefix,
            datetime,
            width = fmt_len.unwrap_or(&17)
        )
    }

    /// ### fmt_ctime
    ///
    /// Format creation time
    fn fmt_ctime(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        fmt_len: Option<&usize>,
        fmt_extra: Option<&String>,
    ) -> String {
        // Get date
        let datetime: String = fmt_time(
            fsentry.get_creation_time(),
            match fmt_extra {
                Some(fmt) => fmt.as_ref(),
                None => "%b %d %Y %H:%M",
            },
        );
        // Add to cur str, prefix and the key value
        format!(
            "{}{}{:0width$}",
            cur_str,
            prefix,
            datetime,
            width = fmt_len.unwrap_or(&17)
        )
    }

    /// ### fmt_group
    ///
    /// Format owner group
    fn fmt_group(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        // Get username
        #[cfg(target_family = "unix")]
        let group: String = match fsentry.get_group() {
            Some(gid) => match get_group_by_gid(gid) {
                Some(user) => user.name().to_string_lossy().to_string(),
                None => gid.to_string(),
            },
            None => 0.to_string(),
        };
        #[cfg(target_os = "windows")]
        let group: String = match fsentry.get_group() {
            Some(gid) => gid.to_string(),
            None => 0.to_string(),
        };
        // Add to cur str, prefix and the key value
        format!(
            "{}{}{:0width$}",
            cur_str,
            prefix,
            group,
            width = fmt_len.unwrap_or(&12)
        )
    }

    /// ### fmt_mtime
    ///
    /// Format last change time
    fn fmt_mtime(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        fmt_len: Option<&usize>,
        fmt_extra: Option<&String>,
    ) -> String {
        // Get date
        let datetime: String = fmt_time(
            fsentry.get_last_change_time(),
            match fmt_extra {
                Some(fmt) => fmt.as_ref(),
                None => "%b %d %Y %H:%M",
            },
        );
        // Add to cur str, prefix and the key value
        format!(
            "{}{}{:0width$}",
            cur_str,
            prefix,
            datetime,
            width = fmt_len.unwrap_or(&17)
        )
    }

    /// ### fmt_name
    ///
    /// Format file name
    fn fmt_name(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        // Get file name (or elide if too long)
        let file_len: usize = match fmt_len {
            Some(l) => *l,
            None => 24,
        };
        let name: &str = fsentry.get_name();
        let last_idx: usize = match fsentry.is_dir() {
            // NOTE: For directories is l - 2, since we push '/' to name
            true => file_len - 2,
            false => file_len - 1,
        };
        let mut name: String = match name.len() >= file_len {
            false => name.to_string(),
            true => format!("{}…", &name[0..last_idx]),
        };
        if fsentry.is_dir() {
            name.push('/');
        }
        // Add to cur str, prefix and the key value
        format!("{}{}{:0width$}", cur_str, prefix, name, width = file_len)
    }

    /// ### fmt_pex
    ///
    /// Format file permissions
    fn fmt_pex(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        _fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        // Create mode string
        let mut pex: String = String::with_capacity(10);
        let file_type: char = match fsentry.is_symlink() {
            true => 'l',
            false => match fsentry.is_dir() {
                true => 'd',
                false => '-',
            },
        };
        pex.push(file_type);
        match fsentry.get_unix_pex() {
            None => pex.push_str("?????????"),
            Some((owner, group, others)) => pex.push_str(
                format!("{}{}{}", fmt_pex(owner), fmt_pex(group), fmt_pex(others)).as_str(),
            ),
        }
        // Add to cur str, prefix and the key value
        format!("{}{}{:10}", cur_str, prefix, pex)
    }

    /// ### fmt_size
    ///
    /// Format file size
    fn fmt_size(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        _fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        if fsentry.is_file() {
            // Get byte size
            let size: ByteSize = ByteSize(fsentry.get_size() as u64);
            // Add to cur str, prefix and the key value
            format!("{}{}{:10}", cur_str, prefix, size.to_string())
        } else {
            // Add to cur str, prefix and the key value
            format!("{}{}          ", cur_str, prefix)
        }
    }

    /// ### fmt_symlink
    ///
    /// Format file symlink (if any)
    fn fmt_symlink(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        // Get file name (or elide if too long)
        let file_len: usize = match fmt_len {
            Some(l) => *l,
            None => 21,
        };
        // Replace `FMT_KEY_NAME` with name
        match fsentry.is_symlink() {
            false => format!("{}{}                        ", cur_str, prefix),
            true => format!(
                "{}{}-> {:0width$}",
                cur_str,
                prefix,
                fmt_path_elide(
                    fsentry.get_realfile().get_abs_path().as_path(),
                    file_len - 1
                ),
                width = file_len
            ),
        }
    }

    /// ### fmt_user
    ///
    /// Format owner user
    fn fmt_user(
        &self,
        fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        _fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        // Get username
        #[cfg(target_family = "unix")]
        let username: String = match fsentry.get_user() {
            Some(uid) => match get_user_by_uid(uid) {
                Some(user) => user.name().to_string_lossy().to_string(),
                None => uid.to_string(),
            },
            None => 0.to_string(),
        };
        #[cfg(target_os = "windows")]
        let username: String = match fsentry.get_user() {
            Some(uid) => uid.to_string(),
            None => 0.to_string(),
        };
        // Add to cur str, prefix and the key value
        format!("{}{}{:12}", cur_str, prefix, username)
    }

    /// ### fmt_fallback
    ///
    /// Fallback function in case the format key is unknown
    /// It does nothing, just returns cur_str
    fn fmt_fallback(
        &self,
        _fsentry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        _fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        // Add to cur str and prefix
        format!("{}{}", cur_str, prefix)
    }

    // Static

    /// ### make_callchain
    ///
    /// Make a callchain starting from the fmt str
    fn make_callchain(fmt_str: &str) -> CallChainBlock {
        // Init chain block
        let mut callchain: Option<CallChainBlock> = None;
        // Track index of the last match found, to get the prefix for each token
        let mut last_index: usize = 0;
        // Match fmt str against regex
        for regex_match in FMT_KEY_REGEX.captures_iter(fmt_str) {
            // Get match index (unwrap is safe, since always exists)
            let index: usize = fmt_str.find(&regex_match[0]).unwrap();
            // Get prefix
            let prefix: String = String::from(&fmt_str[last_index..index]);
            // Increment last index (sum prefix lenght and the length of the key)
            last_index += prefix.len() + regex_match[0].len();
            // Match attributes
            match FMT_ATTR_REGEX.captures(&regex_match[1]) {
                Some(regex_match) => {
                    // Match group 0 (which is name)
                    let callback: FmtCallback = match &regex_match.get(1) {
                        Some(key) => match key.as_str() {
                            FMT_KEY_ATIME => Self::fmt_atime,
                            FMT_KEY_CTIME => Self::fmt_ctime,
                            FMT_KEY_GROUP => Self::fmt_group,
                            FMT_KEY_MTIME => Self::fmt_mtime,
                            FMT_KEY_NAME => Self::fmt_name,
                            FMT_KEY_PEX => Self::fmt_pex,
                            FMT_KEY_SIZE => Self::fmt_size,
                            FMT_KEY_SYMLINK => Self::fmt_symlink,
                            FMT_KEY_USER => Self::fmt_user,
                            _ => Self::fmt_fallback,
                        },
                        None => Self::fmt_fallback,
                    };
                    // Match format length: group 3
                    let fmt_len: Option<usize> = match &regex_match.get(3) {
                        Some(len) => match len.as_str().parse::<usize>() {
                            Ok(len) => Some(len),
                            Err(_) => None,
                        },
                        None => None,
                    };
                    // Match format extra: group 2 + 1
                    let fmt_extra: Option<String> = regex_match
                        .get(5)
                        .as_ref()
                        .map(|extra| extra.as_str().to_string());
                    // Create a callchain or push new element to its back
                    match callchain.as_mut() {
                        None => {
                            callchain =
                                Some(CallChainBlock::new(callback, prefix, fmt_len, fmt_extra))
                        }
                        Some(chain_block) => chain_block.push(callback, prefix, fmt_len, fmt_extra),
                    }
                }
                None => continue,
            }
        }
        // Finalize and return
        match callchain {
            Some(callchain) => callchain,
            None => CallChainBlock::new(Self::fmt_fallback, String::new(), None, None),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::fs::{FsDirectory, FsFile, UnixPex};

    use pretty_assertions::assert_eq;
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_fs_explorer_formatter_callchain() {
        // Make a dummy formatter
        let dummy_formatter: Formatter = Formatter::new("");
        // Make a dummy entry
        let t_now: SystemTime = SystemTime::now();
        let dummy_entry: FsEntry = FsEntry::File(FsFile {
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
        let prefix: String = String::from("h");
        let mut callchain: CallChainBlock = CallChainBlock::new(dummy_fmt, prefix, None, None);
        assert!(callchain.next_block.is_none());
        assert_eq!(callchain.prefix, String::from("h"));
        // Execute
        assert_eq!(
            callchain.next(&dummy_formatter, &dummy_entry, ""),
            String::from("hA")
        );
        // Push 4 new blocks
        callchain.push(dummy_fmt, String::from("h"), None, None);
        callchain.push(dummy_fmt, String::from("h"), None, None);
        callchain.push(dummy_fmt, String::from("h"), None, None);
        callchain.push(dummy_fmt, String::from("h"), None, None);
        // Verify
        assert_eq!(
            callchain.next(&dummy_formatter, &dummy_entry, ""),
            String::from("hAhAhAhAhA")
        );
    }

    #[test]
    fn test_fs_explorer_formatter_format_files() {
        // Make default
        let formatter: Formatter = Formatter::default();
        // Experiments :D
        let t: SystemTime = SystemTime::now();
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        #[cfg(target_family = "unix")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "bar.txt                  -rw-r--r-- root         8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
        #[cfg(target_os = "windows")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "bar.txt                  -rw-r--r-- 0            8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
        // Elide name
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("piroparoporoperoperupupu.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        #[cfg(target_family = "unix")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "piroparoporoperoperupup… -rw-r--r-- root         8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
        #[cfg(target_os = "windows")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "piroparoporoperoperupup… -rw-r--r-- 0            8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
        // No pex
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: None, // UNIX only
        });
        #[cfg(target_family = "unix")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "bar.txt                  -????????? root         8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
        #[cfg(target_os = "windows")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "bar.txt                  -????????? 0            8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
        // No user
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: None,     // UNIX only
            group: Some(0), // UNIX only
            unix_pex: None, // UNIX only
        });
        #[cfg(target_family = "unix")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "bar.txt                  -????????? 0            8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
        #[cfg(target_os = "windows")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "bar.txt                  -????????? 0            8.2 KB     {}",
                fmt_time(t, "%b %d %Y %H:%M")
            )
        );
    }

    #[test]
    fn test_fs_explorer_formatter_format_dirs() {
        // Make default
        let formatter: Formatter = Formatter::default();
        // Experiments :D
        let t_now: SystemTime = SystemTime::now();
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("projects"),
            abs_path: PathBuf::from("/home/cvisintin/projects"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))), // UNIX only
        });
        #[cfg(target_family = "unix")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "projects/                drwxr-xr-x root                    {}",
                fmt_time(t_now, "%b %d %Y %H:%M")
            )
        );
        #[cfg(target_os = "windows")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "projects/                drwxr-xr-x 0                       {}",
                fmt_time(t_now, "%b %d %Y %H:%M")
            )
        );
        // No pex, no user
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("projects"),
            abs_path: PathBuf::from("/home/cvisintin/projects"),
            last_change_time: t_now,
            last_access_time: t_now,
            creation_time: t_now,
            symlink: None,  // UNIX only
            user: None,     // UNIX only
            group: Some(0), // UNIX only
            unix_pex: None, // UNIX only
        });
        #[cfg(target_family = "unix")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "projects/                d????????? 0                       {}",
                fmt_time(t_now, "%b %d %Y %H:%M")
            )
        );
        #[cfg(target_os = "windows")]
        assert_eq!(
            formatter.fmt(&entry),
            format!(
                "projects/                d????????? 0                       {}",
                fmt_time(t_now, "%b %d %Y %H:%M")
            )
        );
    }

    #[test]
    fn test_fs_explorer_formatter_all_together_now() {
        let formatter: Formatter =
            Formatter::new("{NAME:16} {SYMLINK:12} {GROUP} {USER} {PEX} {SIZE} {ATIME:20:%a %b %d %Y %H:%M} {CTIME:20:%a %b %d %Y %H:%M} {MTIME:20:%a %b %d %Y %H:%M}");
        // Directory (with symlink)
        let t: SystemTime = SystemTime::now();
        let pointer: FsEntry = FsEntry::File(FsFile {
            name: String::from("project.info"),
            abs_path: PathBuf::from("/project.info"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: None,     // UNIX only
            group: None,    // UNIX only
            unix_pex: None, // UNIX only
        });
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("projects"),
            abs_path: PathBuf::from("/home/cvisintin/project"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            symlink: Some(Box::new(pointer)), // UNIX only
            user: None,                       // UNIX only
            group: None,                      // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))), // UNIX only
        });
        assert_eq!(formatter.fmt(&entry), format!(
            "projects/        -> project.info 0            0            lrwxr-xr-x            {} {} {}",
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
        ));
        // Directory without symlink
        let entry: FsEntry = FsEntry::Directory(FsDirectory {
            name: String::from("projects"),
            abs_path: PathBuf::from("/home/cvisintin/project"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            symlink: None, // UNIX only
            user: None,    // UNIX only
            group: None,   // UNIX only
            unix_pex: Some((UnixPex::from(7), UnixPex::from(5), UnixPex::from(5))), // UNIX only
        });
        assert_eq!(formatter.fmt(&entry), format!(
            "projects/                                 0            0            drwxr-xr-x            {} {} {}",
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
        ));
        // File with symlink
        let pointer: FsEntry = FsEntry::File(FsFile {
            name: String::from("project.info"),
            abs_path: PathBuf::from("/project.info"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None,  // UNIX only
            user: None,     // UNIX only
            group: None,    // UNIX only
            unix_pex: None, // UNIX only
        });
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: Some(Box::new(pointer)), // UNIX only
            user: None,                       // UNIX only
            group: None,                      // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        assert_eq!(formatter.fmt(&entry), format!(
            "bar.txt          -> project.info 0            0            lrw-r--r-- 8.2 KB     {} {} {}",
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
        ));
        // File without symlink
        let entry: FsEntry = FsEntry::File(FsFile {
            name: String::from("bar.txt"),
            abs_path: PathBuf::from("/bar.txt"),
            last_change_time: t,
            last_access_time: t,
            creation_time: t,
            size: 8192,
            ftype: Some(String::from("txt")),
            symlink: None, // UNIX only
            user: None,    // UNIX only
            group: None,   // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        });
        assert_eq!(formatter.fmt(&entry), format!(
            "bar.txt                                   0            0            -rw-r--r-- 8.2 KB     {} {} {}",
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
            fmt_time(t, "%a %b %d %Y %H:%M"), 
        ));
    }

    /// ### dummy_fmt
    ///
    /// Dummy formatter, just yelds an 'A' at the end of the current string
    fn dummy_fmt(
        _fmt: &Formatter,
        _entry: &FsEntry,
        cur_str: &str,
        prefix: &str,
        _fmt_len: Option<&usize>,
        _fmt_extra: Option<&String>,
    ) -> String {
        format!("{}{}A", cur_str, prefix)
    }
}
