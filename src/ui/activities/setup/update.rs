//! ## SetupActivity
//!
//! `setup_activity` is the module which implements the Setup activity, which is the activity to
//! work on termscp configuration

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
    SetupActivity, ViewLayout, COMPONENT_COLOR_AUTH_ADDR, COMPONENT_COLOR_AUTH_BOOKMARKS,
    COMPONENT_COLOR_AUTH_PASSWORD, COMPONENT_COLOR_AUTH_PORT, COMPONENT_COLOR_AUTH_PROTOCOL,
    COMPONENT_COLOR_AUTH_RECENTS, COMPONENT_COLOR_AUTH_USERNAME, COMPONENT_COLOR_MISC_ERROR,
    COMPONENT_COLOR_MISC_INPUT, COMPONENT_COLOR_MISC_KEYS, COMPONENT_COLOR_MISC_QUIT,
    COMPONENT_COLOR_MISC_SAVE, COMPONENT_COLOR_MISC_WARN,
    COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG, COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG,
    COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG, COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG,
    COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG, COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG,
    COMPONENT_COLOR_TRANSFER_LOG_BG, COMPONENT_COLOR_TRANSFER_LOG_WIN,
    COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL, COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL,
    COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN, COMPONENT_COLOR_TRANSFER_STATUS_SORTING,
    COMPONENT_COLOR_TRANSFER_STATUS_SYNC, COMPONENT_INPUT_LOCAL_FILE_FMT,
    COMPONENT_INPUT_REMOTE_FILE_FMT, COMPONENT_INPUT_SSH_HOST, COMPONENT_INPUT_SSH_USERNAME,
    COMPONENT_INPUT_TEXT_EDITOR, COMPONENT_LIST_SSH_KEYS, COMPONENT_RADIO_DEFAULT_PROTOCOL,
    COMPONENT_RADIO_DEL_SSH_KEY, COMPONENT_RADIO_GROUP_DIRS, COMPONENT_RADIO_HIDDEN_FILES,
    COMPONENT_RADIO_QUIT, COMPONENT_RADIO_SAVE, COMPONENT_RADIO_UPDATES, COMPONENT_TEXT_ERROR,
    COMPONENT_TEXT_HELP,
};
use crate::ui::keymap::*;
use crate::utils::parser::parse_color;

// ext
use tuirealm::{Msg, Payload, Update, Value};

impl Update for SetupActivity {
    /// ### update
    ///
    /// Update auth activity model based on msg
    /// The function exits when returns None
    fn update(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        match self.layout {
            ViewLayout::SetupForm => self.update_setup(msg),
            ViewLayout::SshKeys => self.update_ssh_keys(msg),
            ViewLayout::Theme => self.update_theme(msg),
        }
    }
}

impl SetupActivity {
    fn update_setup(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        // Match msg
        match ref_msg {
            None => None,
            Some(msg) => match msg {
                // Input field <DOWN>
                (COMPONENT_INPUT_TEXT_EDITOR, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_RADIO_DEFAULT_PROTOCOL);
                    None
                }
                (COMPONENT_RADIO_DEFAULT_PROTOCOL, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_RADIO_HIDDEN_FILES);
                    None
                }
                (COMPONENT_RADIO_HIDDEN_FILES, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_RADIO_UPDATES);
                    None
                }
                (COMPONENT_RADIO_UPDATES, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_RADIO_GROUP_DIRS);
                    None
                }
                (COMPONENT_RADIO_GROUP_DIRS, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_INPUT_LOCAL_FILE_FMT);
                    None
                }
                (COMPONENT_INPUT_LOCAL_FILE_FMT, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_INPUT_REMOTE_FILE_FMT);
                    None
                }
                (COMPONENT_INPUT_REMOTE_FILE_FMT, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_INPUT_TEXT_EDITOR);
                    None
                }
                // Input field <UP>
                (COMPONENT_INPUT_REMOTE_FILE_FMT, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_INPUT_LOCAL_FILE_FMT);
                    None
                }
                (COMPONENT_INPUT_LOCAL_FILE_FMT, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_RADIO_GROUP_DIRS);
                    None
                }
                (COMPONENT_RADIO_GROUP_DIRS, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_RADIO_UPDATES);
                    None
                }
                (COMPONENT_RADIO_UPDATES, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_RADIO_HIDDEN_FILES);
                    None
                }
                (COMPONENT_RADIO_HIDDEN_FILES, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_RADIO_DEFAULT_PROTOCOL);
                    None
                }
                (COMPONENT_RADIO_DEFAULT_PROTOCOL, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_INPUT_TEXT_EDITOR);
                    None
                }
                (COMPONENT_INPUT_TEXT_EDITOR, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_INPUT_REMOTE_FILE_FMT);
                    None
                }
                // Error <ENTER> or <ESC>
                (COMPONENT_TEXT_ERROR, key) | (COMPONENT_TEXT_ERROR, key)
                    if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER =>
                {
                    // Umount text error
                    self.umount_error();
                    None
                }
                (COMPONENT_TEXT_ERROR, _) => None,
                // Exit
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::One(Value::Usize(0)))) => {
                    // Save changes
                    if let Err(err) = self.action_save_all() {
                        self.mount_error(err.as_str());
                    }
                    // Exit
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::One(Value::Usize(1)))) => {
                    // Quit
                    self.exit_reason = Some(super::ExitReason::Quit);
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(_)) => {
                    // Umount popup
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, _) => None,
                // Close help
                (COMPONENT_TEXT_HELP, key) | (COMPONENT_TEXT_HELP, key)
                    if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER =>
                {
                    // Umount help
                    self.umount_help();
                    None
                }
                (COMPONENT_TEXT_HELP, _) => None,
                // Save popup
                (COMPONENT_RADIO_SAVE, Msg::OnSubmit(Payload::One(Value::Usize(0)))) => {
                    // Save config
                    if let Err(err) = self.action_save_all() {
                        self.mount_error(err.as_str());
                    }
                    self.umount_save_popup();
                    None
                }
                (COMPONENT_RADIO_SAVE, Msg::OnSubmit(_)) => {
                    // Umount radio save
                    self.umount_save_popup();
                    None
                }
                (COMPONENT_RADIO_SAVE, _) => None,
                // Detect config changed
                (_, Msg::OnChange(_)) => {
                    // An input field has changed value; report config changed
                    self.set_config_changed(true);
                    None
                }
                // <CTRL+H> Show help
                (_, key) if key == &MSG_KEY_CTRL_H => {
                    // Show help
                    self.mount_help();
                    None
                }
                (_, key) if key == &MSG_KEY_TAB => {
                    // Change view
                    if let Err(err) = self.action_change_tab(ViewLayout::SshKeys) {
                        self.mount_error(err.as_str());
                    }
                    None
                }
                // <CTRL+R> Revert changes
                (_, key) if key == &MSG_KEY_CTRL_R => {
                    // Revert changes
                    if let Err(err) = self.action_reset_config() {
                        self.mount_error(err.as_str());
                    }
                    None
                }
                // <CTRL+S> Save
                (_, key) if key == &MSG_KEY_CTRL_S => {
                    // Show save
                    self.mount_save_popup();
                    None
                }
                // <ESC>
                (_, key) if key == &MSG_KEY_ESC => {
                    self.action_on_esc();
                    None
                }
                (_, _) => None, // Nothing to do
            },
        }
    }

    fn update_ssh_keys(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        // Match msg
        match ref_msg {
            None => None,
            Some(msg) => match msg {
                // Error <ENTER> or <ESC>
                (COMPONENT_TEXT_ERROR, key) | (COMPONENT_TEXT_ERROR, key)
                    if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER =>
                {
                    // Umount text error
                    self.umount_error();
                    None
                }
                (COMPONENT_TEXT_ERROR, _) => None,
                // Exit
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::One(Value::Usize(0)))) => {
                    // Save changes
                    if let Err(err) = self.action_save_all() {
                        self.mount_error(err.as_str());
                    }
                    // Exit
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::One(Value::Usize(1)))) => {
                    // Quit
                    self.exit_reason = Some(super::ExitReason::Quit);
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(_)) => {
                    // Umount popup
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, _) => None,
                // Close help
                (COMPONENT_TEXT_HELP, key) | (COMPONENT_TEXT_HELP, key)
                    if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER =>
                {
                    // Umount help
                    self.umount_help();
                    None
                }
                (COMPONENT_TEXT_HELP, _) => None,
                // Delete key
                (COMPONENT_RADIO_DEL_SSH_KEY, Msg::OnSubmit(Payload::One(Value::Usize(0)))) => {
                    // Delete key
                    self.action_delete_ssh_key();
                    // Reload ssh keys
                    self.reload_ssh_keys();
                    // Delete popup
                    self.umount_del_ssh_key();
                    None
                }
                (COMPONENT_RADIO_DEL_SSH_KEY, Msg::OnSubmit(_)) => {
                    // Umount
                    self.umount_del_ssh_key();
                    None
                }
                (COMPONENT_RADIO_DEL_SSH_KEY, _) => None,
                // Save popup
                (COMPONENT_RADIO_SAVE, Msg::OnSubmit(Payload::One(Value::Usize(0)))) => {
                    // Save config
                    if let Err(err) = self.action_save_all() {
                        self.mount_error(err.as_str());
                    }
                    self.umount_save_popup();
                    None
                }
                (COMPONENT_RADIO_SAVE, Msg::OnSubmit(_)) => {
                    // Umount radio save
                    self.umount_save_popup();
                    None
                }
                (COMPONENT_RADIO_SAVE, _) => None,
                // Edit SSH Key
                // <CTRL+H> Show help
                (_, key) if key == &MSG_KEY_CTRL_H => {
                    // Show help
                    self.mount_help();
                    None
                }
                // New key <DOWN>
                (COMPONENT_INPUT_SSH_HOST, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_INPUT_SSH_USERNAME);
                    None
                }
                (COMPONENT_INPUT_SSH_USERNAME, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_INPUT_SSH_HOST);
                    None
                }
                // New key <UP>
                (COMPONENT_INPUT_SSH_USERNAME, key) | (COMPONENT_INPUT_SSH_USERNAME, key)
                    if key == &MSG_KEY_UP || key == &MSG_KEY_TAB =>
                {
                    self.view.active(COMPONENT_INPUT_SSH_HOST);
                    None
                }
                (COMPONENT_INPUT_SSH_HOST, key) | (COMPONENT_INPUT_SSH_HOST, key)
                    if key == &MSG_KEY_UP || key == &MSG_KEY_TAB =>
                {
                    self.view.active(COMPONENT_INPUT_SSH_USERNAME);
                    None
                }
                // New key <ENTER>
                (COMPONENT_INPUT_SSH_HOST, Msg::OnSubmit(_))
                | (COMPONENT_INPUT_SSH_USERNAME, Msg::OnSubmit(_)) => {
                    // Save ssh key
                    self.action_new_ssh_key();
                    self.umount_new_ssh_key();
                    self.reload_ssh_keys();
                    None
                }
                // New key <ESC>
                (COMPONENT_INPUT_SSH_HOST, key) | (COMPONENT_INPUT_SSH_USERNAME, key)
                    if key == &MSG_KEY_ESC =>
                {
                    // Umount new ssh key
                    self.umount_new_ssh_key();
                    None
                }
                // <CTRL+N> New key
                (COMPONENT_LIST_SSH_KEYS, key) if key == &MSG_KEY_CTRL_N => {
                    // Show new key popup
                    self.mount_new_ssh_key();
                    None
                }
                // <ENTER> Edit key
                (COMPONENT_LIST_SSH_KEYS, Msg::OnSubmit(Payload::One(Value::Usize(idx)))) => {
                    // Edit ssh key
                    if let Err(err) = self.edit_ssh_key(*idx) {
                        self.mount_error(err.as_str());
                    }
                    None
                }
                // <DEL | CTRL+E> Show delete
                (COMPONENT_LIST_SSH_KEYS, key) | (COMPONENT_LIST_SSH_KEYS, key)
                    if key == &MSG_KEY_CTRL_E || key == &MSG_KEY_DEL =>
                {
                    // Show delete key
                    self.mount_del_ssh_key();
                    None
                }
                (_, key) if key == &MSG_KEY_TAB => {
                    // Change view
                    if let Err(err) = self.action_change_tab(ViewLayout::Theme) {
                        self.mount_error(err.as_str());
                    }
                    None
                }
                // <CTRL+R> Revert changes
                (_, key) if key == &MSG_KEY_CTRL_R => {
                    // Revert changes
                    if let Err(err) = self.action_reset_config() {
                        self.mount_error(err.as_str());
                    }
                    None
                }
                // <CTRL+S> Save
                (_, key) if key == &MSG_KEY_CTRL_S => {
                    // Show save
                    self.mount_save_popup();
                    None
                }
                // <ESC>
                (_, key) if key == &MSG_KEY_ESC => {
                    self.action_on_esc();
                    None
                }
                (_, _) => None, // Nothing to do
            },
        }
    }

    fn update_theme(&mut self, msg: Option<(String, Msg)>) -> Option<(String, Msg)> {
        let ref_msg: Option<(&str, &Msg)> = msg.as_ref().map(|(s, msg)| (s.as_str(), msg));
        // Match msg
        match ref_msg {
            None => None,
            Some(msg) => match msg {
                // Input fields
                (COMPONENT_COLOR_AUTH_PROTOCOL, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_AUTH_ADDR);
                    None
                }
                (COMPONENT_COLOR_AUTH_ADDR, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_AUTH_PORT);
                    None
                }
                (COMPONENT_COLOR_AUTH_PORT, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_AUTH_USERNAME);
                    None
                }
                (COMPONENT_COLOR_AUTH_USERNAME, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_AUTH_PASSWORD);
                    None
                }
                (COMPONENT_COLOR_AUTH_PASSWORD, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_AUTH_BOOKMARKS);
                    None
                }
                (COMPONENT_COLOR_AUTH_BOOKMARKS, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_AUTH_RECENTS);
                    None
                }
                (COMPONENT_COLOR_AUTH_RECENTS, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_MISC_ERROR);
                    None
                }
                (COMPONENT_COLOR_MISC_ERROR, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_MISC_INPUT);
                    None
                }
                (COMPONENT_COLOR_MISC_INPUT, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_MISC_KEYS);
                    None
                }
                (COMPONENT_COLOR_MISC_KEYS, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_MISC_QUIT);
                    None
                }
                (COMPONENT_COLOR_MISC_QUIT, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_MISC_SAVE);
                    None
                }
                (COMPONENT_COLOR_MISC_SAVE, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_MISC_WARN);
                    None
                }
                (COMPONENT_COLOR_MISC_WARN, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG, key) if key == &MSG_KEY_DOWN => {
                    self.view
                        .active(COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG, key) if key == &MSG_KEY_DOWN => {
                    self.view
                        .active(COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG, key) if key == &MSG_KEY_DOWN => {
                    self.view
                        .active(COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_LOG_BG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_LOG_BG, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_LOG_WIN);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_LOG_WIN, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_STATUS_SORTING);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_STATUS_SORTING, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_STATUS_SYNC);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_STATUS_SYNC, key) if key == &MSG_KEY_DOWN => {
                    self.view.active(COMPONENT_COLOR_AUTH_PROTOCOL);
                    None
                }
                (COMPONENT_COLOR_AUTH_PROTOCOL, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_STATUS_SYNC);
                    None
                }
                (COMPONENT_COLOR_AUTH_ADDR, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_AUTH_PROTOCOL);
                    None
                }
                (COMPONENT_COLOR_AUTH_PORT, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_AUTH_ADDR);
                    None
                }
                (COMPONENT_COLOR_AUTH_USERNAME, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_AUTH_PORT);
                    None
                }
                (COMPONENT_COLOR_AUTH_PASSWORD, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_AUTH_USERNAME);
                    None
                }
                (COMPONENT_COLOR_AUTH_BOOKMARKS, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_AUTH_PASSWORD);
                    None
                }
                (COMPONENT_COLOR_AUTH_RECENTS, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_AUTH_BOOKMARKS);
                    None
                }
                (COMPONENT_COLOR_MISC_ERROR, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_AUTH_RECENTS);
                    None
                }
                (COMPONENT_COLOR_MISC_INPUT, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_MISC_ERROR);
                    None
                }
                (COMPONENT_COLOR_MISC_KEYS, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_MISC_INPUT);
                    None
                }
                (COMPONENT_COLOR_MISC_QUIT, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_MISC_KEYS);
                    None
                }
                (COMPONENT_COLOR_MISC_SAVE, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_MISC_QUIT);
                    None
                }
                (COMPONENT_COLOR_MISC_WARN, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_MISC_SAVE);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_MISC_WARN);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG, key) if key == &MSG_KEY_UP => {
                    self.view
                        .active(COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG, key) if key == &MSG_KEY_UP => {
                    self.view
                        .active(COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL, key) if key == &MSG_KEY_UP => {
                    self.view
                        .active(COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_LOG_BG, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_LOG_WIN, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_LOG_BG);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_STATUS_SORTING, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_LOG_WIN);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_STATUS_SORTING);
                    None
                }
                (COMPONENT_COLOR_TRANSFER_STATUS_SYNC, key) if key == &MSG_KEY_UP => {
                    self.view.active(COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN);
                    None
                }
                // On color change
                (component, Msg::OnChange(Payload::One(Value::Str(color)))) => {
                    if let Some(color) = parse_color(color) {
                        self.action_save_color(component, color);
                        // Set unsaved changes to true
                        self.set_config_changed(true);
                    }
                    None
                }
                // Error <ENTER> or <ESC>
                (COMPONENT_TEXT_ERROR, key) | (COMPONENT_TEXT_ERROR, key)
                    if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER =>
                {
                    // Umount text error
                    self.umount_error();
                    None
                }
                (COMPONENT_TEXT_ERROR, _) => None,
                // Exit
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::One(Value::Usize(0)))) => {
                    // Save changes
                    if let Err(err) = self.action_save_all() {
                        self.mount_error(err.as_str());
                    }
                    // Exit
                    self.exit_reason = Some(super::ExitReason::Quit);
                    None
                }
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(Payload::One(Value::Usize(1)))) => {
                    // Quit
                    self.exit_reason = Some(super::ExitReason::Quit);
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, Msg::OnSubmit(_)) => {
                    // Umount popup
                    self.umount_quit();
                    None
                }
                (COMPONENT_RADIO_QUIT, _) => None,
                // Close help
                (COMPONENT_TEXT_HELP, key) | (COMPONENT_TEXT_HELP, key)
                    if key == &MSG_KEY_ESC || key == &MSG_KEY_ENTER =>
                {
                    // Umount help
                    self.umount_help();
                    None
                }
                (COMPONENT_TEXT_HELP, _) => None,
                // Save popup
                (COMPONENT_RADIO_SAVE, Msg::OnSubmit(Payload::One(Value::Usize(0)))) => {
                    // Save config
                    if let Err(err) = self.action_save_all() {
                        self.mount_error(err.as_str());
                    }
                    self.umount_save_popup();
                    None
                }
                (COMPONENT_RADIO_SAVE, Msg::OnSubmit(_)) => {
                    // Umount radio save
                    self.umount_save_popup();
                    None
                }
                (COMPONENT_RADIO_SAVE, _) => None,
                // Edit SSH Key
                // <CTRL+H> Show help
                (_, key) if key == &MSG_KEY_CTRL_H => {
                    // Show help
                    self.mount_help();
                    None
                }
                (_, key) if key == &MSG_KEY_TAB => {
                    // Change view
                    if let Err(err) = self.action_change_tab(ViewLayout::SetupForm) {
                        self.mount_error(err.as_str());
                    }
                    None
                }
                // <CTRL+R> Revert changes
                (_, key) if key == &MSG_KEY_CTRL_R => {
                    // Revert changes
                    if let Err(err) = self.action_reset_theme() {
                        self.mount_error(err.as_str());
                    }
                    None
                }
                // <CTRL+S> Save
                (_, key) if key == &MSG_KEY_CTRL_S => {
                    // Show save
                    self.mount_save_popup();
                    None
                }
                // <ESC>
                (_, key) if key == &MSG_KEY_ESC => {
                    self.action_on_esc();
                    None
                }
                (_, _) => None, // Nothing to do
            },
        }
    }
}
