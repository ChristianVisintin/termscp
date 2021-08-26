//! ## AuthActivity
//!
//! `auth_activity` is the module which implements the authentication activity

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
use super::{AuthActivity, FileTransferProtocol};
use crate::system::bookmarks_client::BookmarksClient;
use crate::system::environment;

// Ext
use std::path::PathBuf;
use tui_realm_stdlib::{input::InputPropsBuilder, radio::RadioPropsBuilder};
use tuirealm::{Payload, PropsBuilder, Value};

impl AuthActivity {
    /// ### del_bookmark
    ///
    /// Delete bookmark
    pub(super) fn del_bookmark(&mut self, idx: usize) {
        if let Some(bookmarks_cli) = self.bookmarks_client.as_mut() {
            // Iterate over kyes
            let name: Option<&String> = self.bookmarks_list.get(idx);
            if let Some(name) = name {
                bookmarks_cli.del_bookmark(name);
                // Write bookmarks
                self.write_bookmarks();
            }
            // Delete element from vec
            self.bookmarks_list.remove(idx);
        }
    }

    /// ### load_bookmark
    ///
    /// Load selected bookmark (at index) to input fields
    pub(super) fn load_bookmark(&mut self, idx: usize) {
        if let Some(bookmarks_cli) = self.bookmarks_client.as_ref() {
            // Iterate over bookmarks
            if let Some(key) = self.bookmarks_list.get(idx) {
                if let Some(bookmark) = bookmarks_cli.get_bookmark(key) {
                    // Load parameters into components
                    self.load_bookmark_into_gui(
                        bookmark.0, bookmark.1, bookmark.2, bookmark.3, bookmark.4,
                    );
                }
            }
        }
    }

    /// ### save_bookmark
    ///
    /// Save current input fields as a bookmark
    pub(super) fn save_bookmark(&mut self, name: String, save_password: bool) {
        let (address, port, protocol, username, password) = self.get_input();
        if let Some(bookmarks_cli) = self.bookmarks_client.as_mut() {
            // Check if password must be saved
            let password: Option<String> = match save_password {
                true => match self
                    .view
                    .get_state(super::COMPONENT_RADIO_BOOKMARK_SAVE_PWD)
                {
                    Some(Payload::One(Value::Usize(0))) => Some(password), // Yes
                    _ => None,                                             // No such component / No
                },
                false => None,
            };
            bookmarks_cli.add_bookmark(name.clone(), address, port, protocol, username, password);
            // Save bookmarks
            self.write_bookmarks();
            // Remove `name` from bookmarks if exists
            self.bookmarks_list.retain(|b| b.as_str() != name.as_str());
            // Push bookmark to list and sort
            self.bookmarks_list.push(name);
            self.sort_bookmarks();
        }
    }
    /// ### del_recent
    ///
    /// Delete recent
    pub(super) fn del_recent(&mut self, idx: usize) {
        if let Some(client) = self.bookmarks_client.as_mut() {
            let name: Option<&String> = self.recents_list.get(idx);
            if let Some(name) = name {
                client.del_recent(name);
                // Write bookmarks
                self.write_bookmarks();
            }
            // Delete element from vec
            self.recents_list.remove(idx);
        }
    }

    /// ### load_recent
    ///
    /// Load selected recent (at index) to input fields
    pub(super) fn load_recent(&mut self, idx: usize) {
        if let Some(client) = self.bookmarks_client.as_ref() {
            // Iterate over bookmarks
            if let Some(key) = self.recents_list.get(idx) {
                if let Some(bookmark) = client.get_recent(key) {
                    // Load parameters
                    self.load_bookmark_into_gui(
                        bookmark.0, bookmark.1, bookmark.2, bookmark.3, None,
                    );
                }
            }
        }
    }

    /// ### save_recent
    ///
    /// Save current input fields as a "recent"
    pub(super) fn save_recent(&mut self) {
        let (address, port, protocol, username, _password) = self.get_input();
        if let Some(bookmarks_cli) = self.bookmarks_client.as_mut() {
            bookmarks_cli.add_recent(address, port, protocol, username);
            // Save bookmarks
            self.write_bookmarks();
        }
    }

    /// ### write_bookmarks
    ///
    /// Write bookmarks to file
    fn write_bookmarks(&mut self) {
        if let Some(bookmarks_cli) = self.bookmarks_client.as_ref() {
            if let Err(err) = bookmarks_cli.write_bookmarks() {
                self.mount_error(format!("Could not write bookmarks: {}", err).as_str());
            }
        }
    }

    /// ### init_bookmarks_client
    ///
    /// Initialize bookmarks client
    pub(super) fn init_bookmarks_client(&mut self) {
        // Get config dir
        match environment::init_config_dir() {
            Ok(path) => {
                // If some configure client, otherwise do nothing; don't bother users telling them that bookmarks are not supported on their system.
                if let Some(config_dir_path) = path {
                    let bookmarks_file: PathBuf =
                        environment::get_bookmarks_paths(config_dir_path.as_path());
                    // Initialize client
                    match BookmarksClient::new(
                        bookmarks_file.as_path(),
                        config_dir_path.as_path(),
                        16,
                    ) {
                        Ok(cli) => {
                            // Load bookmarks into list
                            let mut bookmarks_list: Vec<String> =
                                Vec::with_capacity(cli.iter_bookmarks().count());
                            for bookmark in cli.iter_bookmarks() {
                                bookmarks_list.push(bookmark.clone());
                            }
                            // Load recents into list
                            let mut recents_list: Vec<String> =
                                Vec::with_capacity(cli.iter_recents().count());
                            for recent in cli.iter_recents() {
                                recents_list.push(recent.clone());
                            }
                            self.bookmarks_client = Some(cli);
                            self.bookmarks_list = bookmarks_list;
                            self.recents_list = recents_list;
                            // Sort bookmark list
                            self.sort_bookmarks();
                            self.sort_recents();
                        }
                        Err(err) => {
                            self.mount_error(
                                format!(
                                    "Could not initialize bookmarks (at \"{}\", \"{}\"): {}",
                                    bookmarks_file.display(),
                                    config_dir_path.display(),
                                    err
                                )
                                .as_str(),
                            );
                        }
                    }
                }
            }
            Err(err) => {
                self.mount_error(
                    format!("Could not initialize configuration directory: {}", err).as_str(),
                );
            }
        }
    }

    // -- privates

    /// ### sort_bookmarks
    ///
    /// Sort bookmarks in list
    fn sort_bookmarks(&mut self) {
        // Conver to lowercase when sorting
        self.bookmarks_list
            .sort_by(|a, b| a.to_lowercase().as_str().cmp(b.to_lowercase().as_str()));
    }

    /// ### sort_recents
    ///
    /// Sort recents in list
    fn sort_recents(&mut self) {
        // Reverse order
        self.recents_list.sort_by(|a, b| b.cmp(a));
    }

    /// ### load_bookmark_into_gui
    ///
    /// Load bookmark data into the gui components
    fn load_bookmark_into_gui(
        &mut self,
        addr: String,
        port: u16,
        protocol: FileTransferProtocol,
        username: String,
        password: Option<String>,
    ) {
        // Load parameters into components
        if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_ADDR) {
            let props = InputPropsBuilder::from(props).with_value(addr).build();
            self.view.update(super::COMPONENT_INPUT_ADDR, props);
        }
        if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_PORT) {
            let props = InputPropsBuilder::from(props)
                .with_value(port.to_string())
                .build();
            self.view.update(super::COMPONENT_INPUT_PORT, props);
        }
        if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_PROTOCOL) {
            let props = RadioPropsBuilder::from(props)
                .with_value(Self::protocol_enum_to_opt(protocol))
                .build();
            self.view.update(super::COMPONENT_RADIO_PROTOCOL, props);
        }
        if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_USERNAME) {
            let props = InputPropsBuilder::from(props).with_value(username).build();
            self.view.update(super::COMPONENT_INPUT_USERNAME, props);
        }
        if let Some(password) = password {
            if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_PASSWORD) {
                let props = InputPropsBuilder::from(props).with_value(password).build();
                self.view.update(super::COMPONENT_INPUT_PASSWORD, props);
            }
        }
    }
}
