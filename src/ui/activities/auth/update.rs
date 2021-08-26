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
// locals
use super::{
    AuthActivity, FileTransferProtocol, COMPONENT_BOOKMARKS_LIST, COMPONENT_INPUT_ADDR,
    COMPONENT_INPUT_BOOKMARK_NAME, COMPONENT_INPUT_PASSWORD, COMPONENT_INPUT_PORT,
    COMPONENT_INPUT_USERNAME, COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK,
    COMPONENT_RADIO_BOOKMARK_DEL_RECENT, COMPONENT_RADIO_BOOKMARK_SAVE_PWD,
    COMPONENT_RADIO_PROTOCOL, COMPONENT_RADIO_QUIT, COMPONENT_RECENTS_LIST, COMPONENT_TEXT_ERROR,
    COMPONENT_TEXT_HELP, COMPONENT_TEXT_NEW_VERSION_NOTES, COMPONENT_TEXT_SIZE_ERR,
};
use crate::ui::keymap::*;
use tui_realm_stdlib::InputPropsBuilder;
use tuirealm::{Msg, Payload, PropsBuilder, Update, Value};

// -- update

impl Update for AuthActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        // Match msg
        match ref_msg {
            None => None, // Exit after None
            Some(msg) => match msg {
                // Focus ( DOWN )
                (COMPONENT_RADIO_PROTOCOL, key) if key == &MSG_KEY_DOWN => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_ADDR);
                    None
                }
                (COMPONENT_INPUT_ADDR, key) if key == &MSG_KEY_DOWN => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PORT);
                    None
                }
                (COMPONENT_INPUT_PORT, key) if key == &MSG_KEY_DOWN => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_USERNAME);
                    None
                }
                (COMPONENT_INPUT_USERNAME, key) if key == &MSG_KEY_DOWN => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                (COMPONENT_INPUT_PASSWORD, key) if key == &MSG_KEY_DOWN => {
                    // Give focus to port
                    self.view.active(COMPONENT_RADIO_PROTOCOL);
                    None
                }
                // Focus ( UP )
                (COMPONENT_INPUT_PASSWORD, key) if key == &MSG_KEY_UP => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_USERNAME);
                    None
                }
                (COMPONENT_INPUT_USERNAME, key) if key == &MSG_KEY_UP => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PORT);
                    None
                }
                (COMPONENT_INPUT_PORT, key) if key == &MSG_KEY_UP => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_ADDR);
                    None
                }
                (COMPONENT_INPUT_ADDR, key) if key == &MSG_KEY_UP => {
                    // Give focus to port
                    self.view.active(COMPONENT_RADIO_PROTOCOL);
                    None
                }
                (COMPONENT_RADIO_PROTOCOL, key) if key == &MSG_KEY_UP => {
                    // Give focus to port
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                // Protocol - On Change
                (COMPONENT_RADIO_PROTOCOL, Msg::OnChange(Payload::One(Value::Usize(protocol)))) => {
                    // If port is standard, update the current port with default for selected protocol
                    let protocol: FileTransferProtocol = Self::protocol_opt_to_enum(*protocol);
                    // Get port
                    let port: u16 = self.get_input_port();
                    match Self::is_port_standard(port) {
                        false => None, // Return None
                        true => {
                            self.update_input_port(Self::get_default_port_for_protocol(protocol))
                        }
                    }
                }
                // Bookmarks commands
                // <RIGHT> / <LEFT>
                (COMPONENT_BOOKMARKS_LIST, key) if key == &MSG_KEY_RIGHT => {
                    // Give focus to recents
                    self.view.active(COMPONENT_RECENTS_LIST);
                    None
                }
                (COMPONENT_RECENTS_LIST, key) if key == &MSG_KEY_LEFT => {
                    // Give focus to bookmarks
                    self.view.active(COMPONENT_BOOKMARKS_LIST);
                    None
                }
                // <DEL | 'E'>
                (COMPONENT_BOOKMARKS_LIST, key)
                    if key == &MSG_KEY_DEL || key == &MSG_KEY_CHAR_E =>
                {
                    // Show delete popup
                    self.mount_bookmark_del_dialog();
                    None
                }
                (COMPONENT_RECENTS_LIST, key) if key == &MSG_KEY_DEL || key == &MSG_KEY_CHAR_E => {
                    // Show delete popup
                    self.mount_recent_del_dialog();
                    None
                }
                // Enter
                (COMPONENT_BOOKMARKS_LIST, Msg::OnSubmit(Payload::One(Value::Usize(idx)))) => {
                    self.load_bookmark(*idx);
                    // Give focus to input password
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                (COMPONENT_RECENTS_LIST, Msg::OnSubmit(Payload::One(Value::Usize(idx)))) => {
                    self.load_recent(*idx);
                    // Give focus to input password
                    self.view.active(COMPONENT_INPUT_PASSWORD);
                    None
                }
                // Bookmark radio
                // Del bookmarks
                (
                    COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK,
                    Msg::OnSubmit(Payload::One(Value::Usize(index))),
                ) => {
                    // hide bookmark delete
                    self.umount_bookmark_del_dialog();
                    // Index must be 0 => YES
                    match *index {
                        0 => {
                            // Get selected bookmark
                            match self.view.get_state(COMPONENT_BOOKMARKS_LIST) {
                                Some(Payload::One(Value::Usize(index))) => {
                                    // Delete bookmark
                                    self.del_bookmark(index);
                                    // Update bookmarks
                                    self.view_bookmarks()
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
                (
                    COMPONENT_RADIO_BOOKMARK_DEL_RECENT,
                    Msg::OnSubmit(Payload::One(Value::Usize(index))),
                ) => {
                    // hide bookmark delete
                    self.umount_recent_del_dialog();
                    // Index must be 0 => YES
                    match *index {
                        0 => {
                            // Get selected bookmark
                            match self.view.get_state(COMPONENT_RECENTS_LIST) {
                                Some(Payload::One(Value::Usize(index))) => {
                                    // Delete recent
                                    self.del_recent(index);
                                    // Update bookmarks
                                    self.view_recent_connections()
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
                // <ESC> hide tab
                (COMPONENT_RADIO_BOOKMARK_DEL_RECENT, key) if key == &MSG_KEY_ESC => {
                    self.umount_recent_del_dialog();
                    None
                }
                (COMPONENT_RADIO_BOOKMARK_DEL_RECENT, _) => None,
                (COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK, key) if key == &MSG_KEY_ESC => {
                    self.umount_bookmark_del_dialog();
                    None
                }
                (COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK, _) => None,
                // Error message
                (COMPONENT_TEXT_ERROR, key) if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER => {
                    // Umount text error
                    self.umount_error();
                    None
                }
                (COMPONENT_TEXT_ERROR, _) => None,
                (COMPONENT_TEXT_NEW_VERSION_NOTES, key)
                    if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER =>
                {
                    // Umount release notes
                    self.umount_release_notes();
                    None
                }
                (COMPONENT_TEXT_NEW_VERSION_NOTES, _) => None,
                // Help
                (_, key) if key == &MSG_KEY_CTRL_H => {
                    // Show help
                    self.mount_help();
                    None
                }
                // Release notes
                (_, key) if key == &MSG_KEY_CTRL_R => {
                    // Show release notes
                    self.mount_release_notes();
                    None
                }
                (COMPONENT_TEXT_HELP, key) if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER => {
                    // Hide text help
                    self.umount_help();
                    None
                }
                (COMPONENT_TEXT_HELP, _) => None,
                // Enter setup
                (_, key) if key == &MSG_KEY_CTRL_C => {
                    self.exit_reason = Some(super::ExitReason::EnterSetup);
                    None
                }
                // Save bookmark; show popup
                (_, key) if key == &MSG_KEY_CTRL_S => {
                    // Show popup
                    self.mount_bookmark_save_dialog();
                    // Give focus to bookmark name
                    self.view.active(COMPONENT_INPUT_BOOKMARK_NAME);
                    None
                }
                (COMPONENT_INPUT_BOOKMARK_NAME, key) if key == &MSG_KEY_DOWN => {
                    // Give focus to pwd
                    self.view.active(COMPONENT_RADIO_BOOKMARK_SAVE_PWD);
                    None
                }
                (COMPONENT_RADIO_BOOKMARK_SAVE_PWD, key) if key == &MSG_KEY_UP => {
                    // Give focus to pwd
                    self.view.active(COMPONENT_INPUT_BOOKMARK_NAME);
                    None
                }
                // Save bookmark
                (COMPONENT_INPUT_BOOKMARK_NAME, Msg::OnSubmit(_))
                | (COMPONENT_RADIO_BOOKMARK_SAVE_PWD, Msg::OnSubmit(_)) => {
                    // Get values
                    let bookmark_name: String =
                        match self.view.get_state(COMPONENT_INPUT_BOOKMARK_NAME) {
                            Some(Payload::One(Value::Str(s))) => s,
                            _ => String::new(),
                        };
                    let save_pwd: bool = matches!(
                        self.view.get_state(COMPONENT_RADIO_BOOKMARK_SAVE_PWD),
                        Some(Payload::One(Value::Usize(0)))
                    );
                    // Save bookmark
                    if !bookmark_name.is_empty() {
                        self.save_bookmark(bookmark_name, save_pwd);
                    }
                    // Umount popup
                    self.umount_bookmark_save_dialog();
                    // Reload bookmarks
                    self.view_bookmarks()
                }
                // Hide save bookmark
                (COMPONENT_INPUT_BOOKMARK_NAME, key) | (COMPONENT_RADIO_BOOKMARK_SAVE_PWD, key)
                    if key == &MSG_KEY_ESC =>
                {
                    // Umount popup
                    self.umount_bookmark_save_dialog();
                    None
                }
                (COMPONENT_INPUT_BOOKMARK_NAME, _) | (COMPONENT_RADIO_BOOKMARK_SAVE_PWD, _) => None,
                // Quit dialog
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::One(Value::Usize(choice)))) => {
                    // If choice is 0, quit termscp
                    if *choice == 0 {
                        self.exit_reason = Some(super::ExitReason::Quit);
                    }
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, key) if key == &MSG_KEY_ESC => {
                    self.umount_quit();
                    None
                }
                // -- text size error; block everything
                (COMPONENT_TEXT_SIZE_ERR, _) => None,
                // <TAB> bookmarks
                (COMPONENT_BOOKMARKS_LIST, key) | (COMPONENT_RECENTS_LIST, key)
                    if key == &MSG_KEY_TAB =>
                {
                    // Give focus to address
                    self.view.active(COMPONENT_INPUT_ADDR);
                    None
                }
                // Any <TAB>, go to bookmarks
                (_, key) if key == &MSG_KEY_TAB => {
                    self.view.active(COMPONENT_BOOKMARKS_LIST);
                    None
                }
                // On submit on any unhandled (connect)
                (_, Msg::OnSubmit(_)) => self.on_unhandled_submit(),
                (_, key) if key == &MSG_KEY_ENTER => self.on_unhandled_submit(),
                // <ESC> => Quit
                (_, key) if key == &MSG_KEY_ESC => {
                    self.mount_quit();
                    None
                }
                (_, _) => None, // Ignore other events
            },
        }
    }
}

impl AuthActivity {
    fn update_input_port(&mut self, port: u16) -> Option<(String, Msg)> {
        match self.view.get_props(COMPONENT_INPUT_PORT) {
            None => None,
            Some(props) => {
                let props = InputPropsBuilder::from(props)
                    .with_value(port.to_string())
                    .build();
                self.view.update(COMPONENT_INPUT_PORT, props)
            }
        }
    }

    fn on_unhandled_submit(&mut self) -> Option<(String, Msg)> {
        // Validate fields
        match self.collect_host_params() {
            Err(err) => {
                // mount error
                self.mount_error(err);
            }
            Ok(params) => {
                self.save_recent();
                // Set file transfer params to context
                self.context_mut().set_ftparams(params);
                // Set exit reason
                self.exit_reason = Some(super::ExitReason::Connect);
            }
        }
        // Return None
        None
    }
}
