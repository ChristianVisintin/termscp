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
use super::{components, AuthActivity, Context, FileTransferProtocol, Id, InputMask};
use crate::filetransfer::params::ProtocolParams;
use crate::filetransfer::FileTransferParams;
use crate::utils::ui::draw_area_in;

use std::str::FromStr;
use tuirealm::tui::layout::{Constraint, Direction, Layout};
use tuirealm::tui::widgets::Clear;
use tuirealm::{State, StateValue, Sub, SubClause, SubEventClause};

impl AuthActivity {
    /// Initialize view, mounting all startup components inside the view
    pub(super) fn init(&mut self) {
        let key_color = self.theme().misc_keys;
        let info_color = self.theme().misc_info_dialog;
        // Headers
        assert!(self
            .app
            .mount(Id::Title, Box::new(components::Title::default()), vec![])
            .is_ok());
        assert!(self
            .app
            .mount(
                Id::Subtitle,
                Box::new(components::Subtitle::default()),
                vec![]
            )
            .is_ok());
        // Footer
        assert!(self
            .app
            .mount(
                Id::HelpFooter,
                Box::new(components::HelpFooter::new(key_color)),
                vec![]
            )
            .is_ok());
        // Get default protocol
        let default_protocol: FileTransferProtocol = self.context().config().get_default_protocol();
        // Auth form
        self.mount_protocol(default_protocol);
        self.mount_address("");
        self.mount_port(Self::get_default_port_for_protocol(default_protocol));
        self.mount_username("");
        self.mount_password("");
        self.mount_s3_bucket("");
        self.mount_s3_profile("");
        self.mount_s3_region("");
        // Version notice
        if let Some(version) = self
            .context()
            .store()
            .get_string(super::STORE_KEY_LATEST_VERSION)
        {
            let version: String = version.to_string();
            assert!(self
                .app
                .mount(
                    Id::NewVersionDisclaimer,
                    Box::new(components::NewVersionDisclaimer::new(
                        version.as_str(),
                        info_color
                    )),
                    vec![]
                )
                .is_ok());
        }
        // Load bookmarks
        self.view_bookmarks();
        self.view_recent_connections();
        // Global listener
        self.init_global_listener();
        // Active protocol
        assert!(self.app.active(&Id::Protocol).is_ok());
    }

    /// Display view on canvas
    pub(super) fn view(&mut self) {
        self.redraw = false;
        let mut ctx: Context = self.context.take().unwrap();
        let _ = ctx.terminal().raw_mut().draw(|f| {
            // Check window size
            let height: u16 = f.size().height;
            self.check_minimum_window_size(height);
            // Prepare chunks
            let body = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(24),   // Body
                        Constraint::Length(1), // Footer
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // Footer
            self.app.view(&Id::HelpFooter, f, body[1]);
            let auth_form_len = 7 + self.input_mask_size();
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(auth_form_len), // Auth Form
                        Constraint::Min(3),                // Bookmarks
                    ]
                    .as_ref(),
                )
                .split(body[0]);
            // Create explorer chunks
            let auth_chunks = Layout::default()
                .constraints(
                    [
                        Constraint::Length(1),                      // h1
                        Constraint::Length(1),                      // h2
                        Constraint::Length(1),                      // Version
                        Constraint::Length(3),                      // protocol
                        Constraint::Length(self.input_mask_size()), // Input mask
                        Constraint::Length(1), // Prevents last field to overflow
                    ]
                    .as_ref(),
                )
                .direction(Direction::Vertical)
                .split(main_chunks[0]);
            // Input mask chunks
            let input_mask = match self.input_mask() {
                InputMask::AwsS3 => Layout::default()
                    .constraints(
                        [
                            Constraint::Length(3), // bucket
                            Constraint::Length(3), // region
                            Constraint::Length(3), // profile
                        ]
                        .as_ref(),
                    )
                    .direction(Direction::Vertical)
                    .split(auth_chunks[4]),
                InputMask::Generic => Layout::default()
                    .constraints(
                        [
                            Constraint::Length(3), // host
                            Constraint::Length(3), // port
                            Constraint::Length(3), // username
                            Constraint::Length(3), // password
                        ]
                        .as_ref(),
                    )
                    .direction(Direction::Vertical)
                    .split(auth_chunks[4]),
            };
            // Create bookmark chunks
            let bookmark_chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(main_chunks[1]);
            // Render
            // Auth chunks
            self.app.view(&Id::Title, f, auth_chunks[0]);
            self.app.view(&Id::Subtitle, f, auth_chunks[1]);
            self.app.view(&Id::NewVersionDisclaimer, f, auth_chunks[2]);
            self.app.view(&Id::Protocol, f, auth_chunks[3]);
            // Render input mask
            match self.input_mask() {
                InputMask::AwsS3 => {
                    self.app.view(&Id::S3Bucket, f, input_mask[0]);
                    self.app.view(&Id::S3Region, f, input_mask[1]);
                    self.app.view(&Id::S3Profile, f, input_mask[2]);
                }
                InputMask::Generic => {
                    self.app.view(&Id::Address, f, input_mask[0]);
                    self.app.view(&Id::Port, f, input_mask[1]);
                    self.app.view(&Id::Username, f, input_mask[2]);
                    self.app.view(&Id::Password, f, input_mask[3]);
                }
            }
            // Bookmark chunks
            self.app.view(&Id::BookmarksList, f, bookmark_chunks[0]);
            self.app.view(&Id::RecentsList, f, bookmark_chunks[1]);
            // Popups
            if self.app.mounted(&Id::ErrorPopup) {
                let popup = draw_area_in(f.size(), 50, 10);
                f.render_widget(Clear, popup);
                // make popup
                self.app.view(&Id::ErrorPopup, f, popup);
            } else if self.app.mounted(&Id::InfoPopup) {
                let popup = draw_area_in(f.size(), 50, 10);
                f.render_widget(Clear, popup);
                // make popup
                self.app.view(&Id::InfoPopup, f, popup);
            } else if self.app.mounted(&Id::WaitPopup) {
                let popup = draw_area_in(f.size(), 50, 10);
                f.render_widget(Clear, popup);
                // make popup
                self.app.view(&Id::WaitPopup, f, popup);
            } else if self.app.mounted(&Id::WindowSizeError) {
                let popup = draw_area_in(f.size(), 80, 20);
                f.render_widget(Clear, popup);
                // make popup
                self.app.view(&Id::WindowSizeError, f, popup);
            } else if self.app.mounted(&Id::QuitPopup) {
                // make popup
                let popup = draw_area_in(f.size(), 30, 10);
                f.render_widget(Clear, popup);
                self.app.view(&Id::QuitPopup, f, popup);
            } else if self.app.mounted(&Id::DeleteBookmarkPopup) {
                // make popup
                let popup = draw_area_in(f.size(), 30, 10);
                f.render_widget(Clear, popup);
                self.app.view(&Id::DeleteBookmarkPopup, f, popup);
            } else if self.app.mounted(&Id::DeleteRecentPopup) {
                // make popup
                let popup = draw_area_in(f.size(), 30, 10);
                f.render_widget(Clear, popup);
                self.app.view(&Id::DeleteRecentPopup, f, popup);
            } else if self.app.mounted(&Id::NewVersionChangelog) {
                // make popup
                let popup = draw_area_in(f.size(), 90, 85);
                f.render_widget(Clear, popup);
                let popup_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(90), // Notes
                            Constraint::Length(3),      // Install radio
                        ]
                        .as_ref(),
                    )
                    .split(popup);
                self.app.view(&Id::NewVersionChangelog, f, popup_chunks[0]);
                self.app.view(&Id::InstallUpdatePopup, f, popup_chunks[1]);
            } else if self.app.mounted(&Id::Keybindings) {
                // make popup
                let popup = draw_area_in(f.size(), 50, 70);
                f.render_widget(Clear, popup);
                self.app.view(&Id::Keybindings, f, popup);
            } else if self.app.mounted(&Id::BookmarkSavePassword) {
                // make popup
                let popup = draw_area_in(f.size(), 20, 20);
                f.render_widget(Clear, popup);
                let popup_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(3), // Input form
                            Constraint::Length(2), // Yes/No
                        ]
                        .as_ref(),
                    )
                    .split(popup);
                self.app.view(&Id::BookmarkName, f, popup_chunks[0]);
                self.app.view(&Id::BookmarkSavePassword, f, popup_chunks[1]);
            }
        });
        self.context = Some(ctx);
    }

    // -- partials

    /// Make text span from bookmarks
    pub(super) fn view_bookmarks(&mut self) {
        let bookmarks: Vec<String> = self
            .bookmarks_list
            .iter()
            .map(|x| {
                Self::fmt_bookmark(
                    x,
                    self.bookmarks_client
                        .as_ref()
                        .unwrap()
                        .get_bookmark(x)
                        .unwrap(),
                )
            })
            .collect();
        let bookmarks_color = self.theme().auth_bookmarks;
        assert!(self
            .app
            .remount(
                Id::BookmarksList,
                Box::new(components::BookmarksList::new(&bookmarks, bookmarks_color)),
                vec![]
            )
            .is_ok());
    }

    /// View recent connections
    pub(super) fn view_recent_connections(&mut self) {
        let bookmarks: Vec<String> = self
            .recents_list
            .iter()
            .map(|x| {
                Self::fmt_recent(
                    self.bookmarks_client
                        .as_ref()
                        .unwrap()
                        .get_recent(x)
                        .unwrap(),
                )
            })
            .collect();
        let recents_color = self.theme().auth_recents;
        assert!(self
            .app
            .remount(
                Id::RecentsList,
                Box::new(components::RecentsList::new(&bookmarks, recents_color)),
                vec![]
            )
            .is_ok());
    }

    // -- mount

    /// Mount error box
    pub(super) fn mount_error<S: AsRef<str>>(&mut self, text: S) {
        let err_color = self.theme().misc_error_dialog;
        assert!(self
            .app
            .remount(
                Id::ErrorPopup,
                Box::new(components::ErrorPopup::new(text, err_color)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::ErrorPopup).is_ok());
    }

    /// Umount error message
    pub(super) fn umount_error(&mut self) {
        let _ = self.app.umount(&Id::ErrorPopup);
    }

    /// Mount info box
    pub(super) fn mount_info<S: AsRef<str>>(&mut self, text: S) {
        let color = self.theme().misc_info_dialog;
        assert!(self
            .app
            .remount(
                Id::InfoPopup,
                Box::new(components::InfoPopup::new(text, color)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::InfoPopup).is_ok());
    }

    /// Umount info message
    pub(super) fn umount_info(&mut self) {
        let _ = self.app.umount(&Id::InfoPopup);
    }

    /// Mount wait box
    pub(super) fn mount_wait(&mut self, text: &str) {
        let wait_color = self.theme().misc_info_dialog;
        assert!(self
            .app
            .remount(
                Id::WaitPopup,
                Box::new(components::WaitPopup::new(text, wait_color)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::WaitPopup).is_ok());
    }

    /// Umount wait message
    pub(super) fn umount_wait(&mut self) {
        let _ = self.app.umount(&Id::WaitPopup);
    }

    /// Mount size error
    pub(super) fn mount_size_err(&mut self) {
        // Mount
        let err_color = self.theme().misc_error_dialog;
        assert!(self
            .app
            .remount(
                Id::WindowSizeError,
                Box::new(components::WindowSizeError::new(err_color)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::WindowSizeError).is_ok());
    }

    /// Umount error size error
    pub(super) fn umount_size_err(&mut self) {
        let _ = self.app.umount(&Id::WindowSizeError);
    }

    /// Mount quit popup
    pub(super) fn mount_quit(&mut self) {
        // Protocol
        let quit_color = self.theme().misc_quit_dialog;
        assert!(self
            .app
            .remount(
                Id::QuitPopup,
                Box::new(components::QuitPopup::new(quit_color)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::QuitPopup).is_ok());
    }

    /// Umount quit popup
    pub(super) fn umount_quit(&mut self) {
        let _ = self.app.umount(&Id::QuitPopup);
    }

    /// Mount bookmark delete dialog
    pub(super) fn mount_bookmark_del_dialog(&mut self) {
        let warn_color = self.theme().misc_warn_dialog;
        assert!(self
            .app
            .remount(
                Id::DeleteBookmarkPopup,
                Box::new(components::DeleteBookmarkPopup::new(warn_color)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteBookmarkPopup).is_ok());
    }

    /// umount delete bookmark dialog
    pub(super) fn umount_bookmark_del_dialog(&mut self) {
        let _ = self.app.umount(&Id::DeleteBookmarkPopup);
    }

    /// Mount recent delete dialog
    pub(super) fn mount_recent_del_dialog(&mut self) {
        let warn_color = self.theme().misc_warn_dialog;
        assert!(self
            .app
            .remount(
                Id::DeleteRecentPopup,
                Box::new(components::DeleteRecentPopup::new(warn_color)),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::DeleteRecentPopup).is_ok());
    }

    /// umount delete recent dialog
    pub(super) fn umount_recent_del_dialog(&mut self) {
        let _ = self.app.umount(&Id::DeleteRecentPopup);
    }

    /// Mount bookmark save dialog
    pub(super) fn mount_bookmark_save_dialog(&mut self) {
        let save_color = self.theme().misc_save_dialog;
        let warn_color = self.theme().misc_warn_dialog;
        assert!(self
            .app
            .remount(
                Id::BookmarkName,
                Box::new(components::BookmarkName::new(save_color)),
                vec![]
            )
            .is_ok());
        assert!(self
            .app
            .remount(
                Id::BookmarkSavePassword,
                Box::new(components::BookmarkSavePassword::new(warn_color)),
                vec![]
            )
            .is_ok());
        // Give focus to input bookmark name
        assert!(self.app.active(&Id::BookmarkName).is_ok());
    }

    /// Umount bookmark save dialog
    pub(super) fn umount_bookmark_save_dialog(&mut self) {
        let _ = self.app.umount(&Id::BookmarkName);
        let _ = self.app.umount(&Id::BookmarkSavePassword);
    }

    /// Mount keybindings
    pub(super) fn mount_keybindings(&mut self) {
        let key_color = self.theme().misc_keys;
        assert!(self
            .app
            .remount(
                Id::Keybindings,
                Box::new(components::Keybindings::new(key_color)),
                vec![]
            )
            .is_ok());
        // Active help
        assert!(self.app.active(&Id::Keybindings).is_ok());
    }

    /// Umount help
    pub(super) fn umount_help(&mut self) {
        let _ = self.app.umount(&Id::Keybindings);
    }

    /// mount release notes text area
    pub(super) fn mount_release_notes(&mut self) {
        if let Some(ctx) = self.context.as_ref() {
            if let Some(release_notes) = ctx.store().get_string(super::STORE_KEY_RELEASE_NOTES) {
                // make spans
                let info_color = self.theme().misc_info_dialog;
                assert!(self
                    .app
                    .remount(
                        Id::NewVersionChangelog,
                        Box::new(components::ReleaseNotes::new(release_notes, info_color)),
                        vec![]
                    )
                    .is_ok());
                assert!(self
                    .app
                    .remount(
                        Id::InstallUpdatePopup,
                        Box::new(components::InstallUpdatePopup::new(info_color)),
                        vec![]
                    )
                    .is_ok());
                assert!(self.app.active(&Id::InstallUpdatePopup).is_ok());
            }
        }
    }

    /// Umount release notes text area
    pub(super) fn umount_release_notes(&mut self) {
        let _ = self.app.umount(&Id::NewVersionChangelog);
        let _ = self.app.umount(&Id::InstallUpdatePopup);
    }

    pub(super) fn mount_protocol(&mut self, protocol: FileTransferProtocol) {
        let protocol_color = self.theme().auth_protocol;
        assert!(self
            .app
            .remount(
                Id::Protocol,
                Box::new(components::ProtocolRadio::new(protocol, protocol_color)),
                vec![]
            )
            .is_ok());
    }

    pub(super) fn mount_address(&mut self, address: &str) {
        let addr_color = self.theme().auth_address;
        assert!(self
            .app
            .remount(
                Id::Address,
                Box::new(components::InputAddress::new(address, addr_color)),
                vec![]
            )
            .is_ok());
    }

    pub(super) fn mount_port(&mut self, port: u16) {
        let port_color = self.theme().auth_port;
        assert!(self
            .app
            .remount(
                Id::Port,
                Box::new(components::InputPort::new(port, port_color)),
                vec![]
            )
            .is_ok());
    }

    pub(crate) fn mount_username(&mut self, username: &str) {
        let username_color = self.theme().auth_username;
        assert!(self
            .app
            .remount(
                Id::Username,
                Box::new(components::InputUsername::new(username, username_color)),
                vec![]
            )
            .is_ok());
    }

    pub(crate) fn mount_password(&mut self, password: &str) {
        let password_color = self.theme().auth_password;
        assert!(self
            .app
            .remount(
                Id::Password,
                Box::new(components::InputPassword::new(password, password_color)),
                vec![]
            )
            .is_ok());
    }

    pub(super) fn mount_s3_bucket(&mut self, bucket: &str) {
        let addr_color = self.theme().auth_address;
        assert!(self
            .app
            .remount(
                Id::S3Bucket,
                Box::new(components::InputS3Bucket::new(bucket, addr_color)),
                vec![]
            )
            .is_ok());
    }

    pub(super) fn mount_s3_region(&mut self, region: &str) {
        let port_color = self.theme().auth_port;
        assert!(self
            .app
            .remount(
                Id::S3Region,
                Box::new(components::InputS3Region::new(region, port_color)),
                vec![]
            )
            .is_ok());
    }

    pub(crate) fn mount_s3_profile(&mut self, profile: &str) {
        let username_color = self.theme().auth_username;
        assert!(self
            .app
            .remount(
                Id::S3Profile,
                Box::new(components::InputS3Profile::new(profile, username_color)),
                vec![]
            )
            .is_ok());
    }

    // -- query

    /// Collect input values from view
    pub(super) fn get_generic_params_input(&self) -> (String, u16, String, String) {
        let addr: String = self.get_input_addr();
        let port: u16 = self.get_input_port();
        let username: String = self.get_input_username();
        let password: String = self.get_input_password();
        (addr, port, username, password)
    }

    /// Collect s3 input values from view
    pub(super) fn get_s3_params_input(&self) -> (String, String, Option<String>) {
        let bucket: String = self.get_input_s3_bucket();
        let region: String = self.get_input_s3_region();
        let profile: Option<String> = self.get_input_s3_profile();
        (bucket, region, profile)
    }

    pub(super) fn get_input_addr(&self) -> String {
        match self.app.state(&Id::Address) {
            Ok(State::One(StateValue::String(x))) => x,
            _ => String::new(),
        }
    }

    pub(super) fn get_input_port(&self) -> u16 {
        match self.app.state(&Id::Port) {
            Ok(State::One(StateValue::String(x))) => match u16::from_str(x.as_str()) {
                Ok(v) => v,
                _ => 0,
            },
            _ => 0,
        }
    }

    pub(super) fn get_input_username(&self) -> String {
        match self.app.state(&Id::Username) {
            Ok(State::One(StateValue::String(x))) => x,
            _ => String::new(),
        }
    }

    pub(super) fn get_input_password(&self) -> String {
        match self.app.state(&Id::Password) {
            Ok(State::One(StateValue::String(x))) => x,
            _ => String::new(),
        }
    }

    pub(super) fn get_input_s3_bucket(&self) -> String {
        match self.app.state(&Id::S3Bucket) {
            Ok(State::One(StateValue::String(x))) => x,
            _ => String::new(),
        }
    }

    pub(super) fn get_input_s3_region(&self) -> String {
        match self.app.state(&Id::S3Region) {
            Ok(State::One(StateValue::String(x))) => x,
            _ => String::new(),
        }
    }

    pub(super) fn get_input_s3_profile(&self) -> Option<String> {
        match self.app.state(&Id::S3Profile) {
            Ok(State::One(StateValue::String(x))) if !x.is_empty() => Some(x),
            _ => None,
        }
    }

    /// Get new bookmark params
    pub(super) fn get_new_bookmark(&self) -> (String, bool) {
        let name = match self.app.state(&Id::BookmarkName) {
            Ok(State::One(StateValue::String(name))) => name,
            _ => String::default(),
        };
        if matches!(
            self.app.state(&Id::BookmarkSavePassword),
            Ok(State::One(StateValue::Usize(0)))
        ) {
            (name, true)
        } else {
            (name, false)
        }
    }

    // -- len

    /// Returns the input mask size based on current input mask
    pub(super) fn input_mask_size(&self) -> u16 {
        match self.input_mask() {
            InputMask::AwsS3 => 9,
            InputMask::Generic => 12,
        }
    }

    // -- fmt

    /// Format bookmark to display on ui
    fn fmt_bookmark(name: &str, b: FileTransferParams) -> String {
        let addr: String = Self::fmt_recent(b);
        format!("{} ({})", name, addr)
    }

    /// Format recent connection to display on ui
    fn fmt_recent(b: FileTransferParams) -> String {
        let protocol: String = b.protocol.to_string().to_lowercase();
        match b.params {
            ProtocolParams::AwsS3(s3) => {
                let profile: String = match s3.profile {
                    Some(p) => format!("[{}]", p),
                    None => String::default(),
                };
                format!(
                    "{}://{} ({}) {}",
                    protocol, s3.bucket_name, s3.region, profile
                )
            }
            ProtocolParams::Generic(params) => {
                let username: String = match params.username {
                    None => String::default(),
                    Some(u) => format!("{}@", u),
                };
                format!(
                    "{}://{}{}:{}",
                    protocol, username, params.address, params.port
                )
            }
        }
    }

    fn init_global_listener(&mut self) {
        use tuirealm::event::{Key, KeyEvent, KeyModifiers};
        assert!(self
            .app
            .mount(
                Id::GlobalListener,
                Box::new(components::GlobalListener::default()),
                vec![
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Esc,
                            modifiers: KeyModifiers::NONE,
                        }),
                        Self::no_popup_mounted_clause(),
                    ),
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Function(10),
                            modifiers: KeyModifiers::NONE,
                        }),
                        Self::no_popup_mounted_clause(),
                    ),
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Char('c'),
                            modifiers: KeyModifiers::CONTROL,
                        }),
                        Self::no_popup_mounted_clause(),
                    ),
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Char('h'),
                            modifiers: KeyModifiers::CONTROL,
                        }),
                        Self::no_popup_mounted_clause(),
                    ),
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Function(1),
                            modifiers: KeyModifiers::NONE,
                        }),
                        Self::no_popup_mounted_clause(),
                    ),
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Char('r'),
                            modifiers: KeyModifiers::CONTROL,
                        }),
                        Self::no_popup_mounted_clause(),
                    ),
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Char('s'),
                            modifiers: KeyModifiers::CONTROL,
                        }),
                        Self::no_popup_mounted_clause(),
                    ),
                ]
            )
            .is_ok());
    }

    /// Returns a sub clause which requires that no popup is mounted in order to be satisfied
    fn no_popup_mounted_clause() -> SubClause<Id> {
        SubClause::And(
            Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                Id::ErrorPopup,
            )))),
            Box::new(SubClause::And(
                Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                    Id::InfoPopup,
                )))),
                Box::new(SubClause::And(
                    Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                        Id::Keybindings,
                    )))),
                    Box::new(SubClause::And(
                        Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                            Id::DeleteBookmarkPopup,
                        )))),
                        Box::new(SubClause::And(
                            Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                                Id::DeleteRecentPopup,
                            )))),
                            Box::new(SubClause::And(
                                Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                                    Id::InstallUpdatePopup,
                                )))),
                                Box::new(SubClause::And(
                                    Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                                        Id::BookmarkSavePassword,
                                    )))),
                                    Box::new(SubClause::Not(Box::new(SubClause::IsMounted(
                                        Id::WaitPopup,
                                    )))),
                                )),
                            )),
                        )),
                    )),
                )),
            )),
        )
    }
}
