//! ## CLI opts
//!
//! defines the types for main.rs types

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
use argh::FromArgs;

use crate::activity_manager::NextActivity;
use crate::filetransfer::FileTransferParams;
use crate::system::logging::LogLevel;

use std::path::PathBuf;
use std::time::Duration;

pub enum Task {
    Activity(NextActivity),
    ImportTheme(PathBuf),
    InstallUpdate,
}

#[derive(FromArgs)]
#[argh(description = "
where positional can be: 
        - [address]         [local-wrkdir]
    OR
        - [bookmark-Name]   [local-wrkdir]

Address syntax can be:

    - `protocol://user@address:port:wrkdir` for protocols such as Sftp, Scp, Ftp
    - `s3://bucket-name@region:profile:/wrkdir` for Aws S3 protocol

Please, report issues to <https://github.com/veeso/termscp>
Please, consider supporting the author <https://ko-fi.com/veeso>")]
pub struct Args {
    #[argh(
        switch,
        short = 'b',
        description = "resolve address argument as a bookmark name"
    )]
    pub address_as_bookmark: bool,
    #[argh(switch, short = 'c', description = "open termscp configuration")]
    pub config: bool,
    #[argh(switch, short = 'D', description = "enable TRACE log level")]
    pub debug: bool,
    #[argh(option, short = 'P', description = "provide password from CLI")]
    pub password: Option<String>,
    #[argh(switch, short = 'q', description = "disable logging")]
    pub quiet: bool,
    #[argh(option, short = 't', description = "import specified theme")]
    pub theme: Option<String>,
    #[argh(
        switch,
        short = 'u',
        description = "update termscp to the latest version"
    )]
    pub update: bool,
    #[argh(
        option,
        short = 'T',
        default = "10",
        description = "set UI ticks; default 10ms"
    )]
    pub ticks: u64,
    #[argh(switch, short = 'v', description = "print version")]
    pub version: bool,
    // -- positional
    #[argh(
        positional,
        description = "protocol://user@address:port:wrkdir local-wrkdir"
    )]
    pub positional: Vec<String>,
}

pub struct RunOpts {
    pub remote: Remote,
    pub ticks: Duration,
    pub log_level: LogLevel,
    pub task: Task,
}

impl Default for RunOpts {
    fn default() -> Self {
        Self {
            remote: Remote::None,
            ticks: Duration::from_millis(10),
            log_level: LogLevel::Info,
            task: Task::Activity(NextActivity::Authentication),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Remote {
    Bookmark(BookmarkParams),
    Host(HostParams),
    None,
}

pub struct BookmarkParams {
    pub name: String,
    pub password: Option<String>,
}

pub struct HostParams {
    pub params: FileTransferParams,
    pub password: Option<String>,
}

impl BookmarkParams {
    pub fn new<S: AsRef<str>>(name: S, password: Option<S>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            password: password.map(|x| x.as_ref().to_string()),
        }
    }
}

impl HostParams {
    pub fn new<S: AsRef<str>>(params: FileTransferParams, password: Option<S>) -> Self {
        Self {
            params,
            password: password.map(|x| x.as_ref().to_string()),
        }
    }
}
