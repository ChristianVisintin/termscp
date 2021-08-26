#![doc(html_playground_url = "https://play.rust-lang.org")]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/veeso/termscp/main/assets/images/termscp-128.png"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/veeso/termscp/main/assets/images/termscp-512.png"
)]

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

#[macro_use]
extern crate bitflags;
extern crate bytesize;
extern crate chrono;
extern crate content_inspector;
extern crate crossterm;
extern crate dirs;
extern crate edit;
extern crate hostname;
#[cfg(feature = "with-keyring")]
extern crate keyring;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate magic_crypt;
extern crate open;
#[cfg(target_os = "windows")]
extern crate path_slash;
extern crate rand;
extern crate regex;
extern crate ssh2;
extern crate suppaftp;
extern crate tempfile;
extern crate textwrap;
extern crate tui_realm_stdlib;
extern crate tuirealm;
extern crate ureq;
#[cfg(target_family = "unix")]
extern crate users;
extern crate whoami;
extern crate wildmatch;

pub mod activity_manager;
pub mod config;
pub mod filetransfer;
pub mod fs;
pub mod host;
pub mod support;
pub mod system;
pub mod ui;
pub mod utils;
