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
// Submodules
mod actions;
mod config;
mod update;
mod view;

// Locals
use super::{Activity, Context, ExitReason};
use crate::config::themes::Theme;
use crate::system::config_client::ConfigClient;
use crate::system::theme_provider::ThemeProvider;
// Ext
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tuirealm::{Update, View};

// -- components
// -- common
const COMPONENT_TEXT_HELP: &str = "TEXT_HELP";
const COMPONENT_TEXT_FOOTER: &str = "TEXT_FOOTER";
const COMPONENT_TEXT_ERROR: &str = "TEXT_ERROR";
const COMPONENT_RADIO_QUIT: &str = "RADIO_QUIT";
const COMPONENT_RADIO_SAVE: &str = "RADIO_SAVE";
const COMPONENT_RADIO_TAB: &str = "RADIO_TAB";
// -- config
const COMPONENT_INPUT_TEXT_EDITOR: &str = "INPUT_TEXT_EDITOR";
const COMPONENT_RADIO_DEFAULT_PROTOCOL: &str = "RADIO_DEFAULT_PROTOCOL";
const COMPONENT_RADIO_HIDDEN_FILES: &str = "RADIO_HIDDEN_FILES";
const COMPONENT_RADIO_UPDATES: &str = "RADIO_CHECK_UPDATES";
const COMPONENT_RADIO_GROUP_DIRS: &str = "RADIO_GROUP_DIRS";
const COMPONENT_INPUT_LOCAL_FILE_FMT: &str = "INPUT_LOCAL_FILE_FMT";
const COMPONENT_INPUT_REMOTE_FILE_FMT: &str = "INPUT_REMOTE_FILE_FMT";
// -- ssh keys
const COMPONENT_LIST_SSH_KEYS: &str = "LIST_SSH_KEYS";
const COMPONENT_INPUT_SSH_HOST: &str = "INPUT_SSH_HOST";
const COMPONENT_INPUT_SSH_USERNAME: &str = "INPUT_SSH_USERNAME";
const COMPONENT_RADIO_DEL_SSH_KEY: &str = "RADIO_DEL_SSH_KEY";
// -- theme
const COMPONENT_COLOR_AUTH_TITLE: &str = "COMPONENT_COLOR_AUTH_TITLE";
const COMPONENT_COLOR_MISC_TITLE: &str = "COMPONENT_COLOR_MISC_TITLE";
const COMPONENT_COLOR_TRANSFER_TITLE: &str = "COMPONENT_COLOR_TRANSFER_TITLE";
const COMPONENT_COLOR_TRANSFER_TITLE_2: &str = "COMPONENT_COLOR_TRANSFER_TITLE_2";
const COMPONENT_COLOR_AUTH_ADDR: &str = "COMPONENT_COLOR_AUTH_ADDR";
const COMPONENT_COLOR_AUTH_BOOKMARKS: &str = "COMPONENT_COLOR_AUTH_BOOKMARKS";
const COMPONENT_COLOR_AUTH_PASSWORD: &str = "COMPONENT_COLOR_AUTH_PASSWORD";
const COMPONENT_COLOR_AUTH_PORT: &str = "COMPONENT_COLOR_AUTH_PORT";
const COMPONENT_COLOR_AUTH_PROTOCOL: &str = "COMPONENT_COLOR_AUTH_PROTOCOL";
const COMPONENT_COLOR_AUTH_RECENTS: &str = "COMPONENT_COLOR_AUTH_RECENTS";
const COMPONENT_COLOR_AUTH_USERNAME: &str = "COMPONENT_COLOR_AUTH_USERNAME";
const COMPONENT_COLOR_MISC_ERROR: &str = "COMPONENT_COLOR_MISC_ERROR";
const COMPONENT_COLOR_MISC_INPUT: &str = "COMPONENT_COLOR_MISC_INPUT";
const COMPONENT_COLOR_MISC_KEYS: &str = "COMPONENT_COLOR_MISC_KEYS";
const COMPONENT_COLOR_MISC_QUIT: &str = "COMPONENT_COLOR_MISC_QUIT";
const COMPONENT_COLOR_MISC_SAVE: &str = "COMPONENT_COLOR_MISC_SAVE";
const COMPONENT_COLOR_MISC_WARN: &str = "COMPONENT_COLOR_MISC_WARN";
const COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG: &str =
    "COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG";
const COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG: &str =
    "COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG";
const COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG: &str =
    "COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG";
const COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG: &str =
    "COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG";
const COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG: &str =
    "COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG";
const COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG: &str =
    "COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG";
const COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL: &str = "COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL";
const COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL: &str = "COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL";
const COMPONENT_COLOR_TRANSFER_LOG_BG: &str = "COMPONENT_COLOR_TRANSFER_LOG_BG";
const COMPONENT_COLOR_TRANSFER_LOG_WIN: &str = "COMPONENT_COLOR_TRANSFER_LOG_WIN";
const COMPONENT_COLOR_TRANSFER_STATUS_SORTING: &str = "COMPONENT_COLOR_TRANSFER_STATUS_SORTING";
const COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN: &str = "COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN";
const COMPONENT_COLOR_TRANSFER_STATUS_SYNC: &str = "COMPONENT_COLOR_TRANSFER_STATUS_SYNC";

// -- store
const STORE_CONFIG_CHANGED: &str = "SETUP_CONFIG_CHANGED";

/// ### ViewLayout
///
/// Current view layout
#[derive(std::cmp::PartialEq)]
enum ViewLayout {
    SetupForm,
    SshKeys,
    Theme,
}

/// ## SetupActivity
///
/// Setup activity states holder
pub struct SetupActivity {
    exit_reason: Option<ExitReason>,
    context: Option<Context>, // Context holder
    view: View,               // View
    layout: ViewLayout,       // View layout
    redraw: bool,
}

impl Default for SetupActivity {
    fn default() -> Self {
        SetupActivity {
            exit_reason: None,
            context: None,
            view: View::init(),
            layout: ViewLayout::SetupForm,
            redraw: true, // Draw at first `on_draw`
        }
    }
}

impl SetupActivity {
    /// ### context
    ///
    /// Returns a reference to context
    fn context(&self) -> &Context {
        self.context.as_ref().unwrap()
    }

    /// ### context_mut
    ///
    /// Returns a mutable reference to context
    fn context_mut(&mut self) -> &mut Context {
        self.context.as_mut().unwrap()
    }

    fn config(&self) -> &ConfigClient {
        self.context().config()
    }

    fn config_mut(&mut self) -> &mut ConfigClient {
        self.context_mut().config_mut()
    }

    fn theme(&self) -> &Theme {
        self.context().theme_provider().theme()
    }

    fn theme_mut(&mut self) -> &mut Theme {
        self.context_mut().theme_provider_mut().theme_mut()
    }

    fn theme_provider(&mut self) -> &mut ThemeProvider {
        self.context_mut().theme_provider_mut()
    }

    /// ### config_changed
    ///
    /// Returns whether config has changed
    fn config_changed(&self) -> bool {
        self.context()
            .store()
            .get_boolean(STORE_CONFIG_CHANGED)
            .unwrap_or(false)
    }

    /// ### set_config_changed
    ///
    /// Set value for config changed key into the store
    fn set_config_changed(&mut self, changed: bool) {
        self.context_mut()
            .store_mut()
            .set_boolean(STORE_CONFIG_CHANGED, changed);
    }
}

impl Activity for SetupActivity {
    /// ### on_create
    ///
    /// `on_create` is the function which must be called to initialize the activity.
    /// `on_create` must initialize all the data structures used by the activity
    /// Context is taken from activity manager and will be released only when activity is destroyed
    fn on_create(&mut self, context: Context) {
        // Set context
        self.context = Some(context);
        // Clear terminal
        self.context.as_mut().unwrap().clear_screen();
        // Set config changed to false
        self.set_config_changed(false);
        // Put raw mode on enabled
        if let Err(err) = enable_raw_mode() {
            error!("Failed to enter raw mode: {}", err);
        }
        // Init view
        self.init(ViewLayout::SetupForm);
        // Verify error state from context
        if let Some(err) = self.context.as_mut().unwrap().error() {
            self.mount_error(err.as_str());
        }
    }

    /// ### on_draw
    ///
    /// `on_draw` is the function which draws the graphical interface.
    /// This function must be called at each tick to refresh the interface
    fn on_draw(&mut self) {
        // Context must be something
        if self.context.is_none() {
            return;
        }
        // Read one event
        if let Ok(Some(event)) = self.context().input_hnd().read_event() {
            // Set redraw to true
            self.redraw = true;
            // Handle event
            let msg = self.view.on(event);
            self.update(msg);
        }
        // Redraw if necessary
        if self.redraw {
            // View
            self.view();
            // Redraw back to false
            self.redraw = false;
        }
    }

    /// ### will_umount
    ///
    /// `will_umount` is the method which must be able to report to the activity manager, whether
    /// the activity should be terminated or not.
    /// If not, the call will return `None`, otherwise return`Some(ExitReason)`
    fn will_umount(&self) -> Option<&ExitReason> {
        self.exit_reason.as_ref()
    }

    /// ### on_destroy
    ///
    /// `on_destroy` is the function which cleans up runtime variables and data before terminating the activity.
    /// This function must be called once before terminating the activity.
    /// This function finally releases the context
    fn on_destroy(&mut self) -> Option<Context> {
        // Disable raw mode
        if let Err(err) = disable_raw_mode() {
            error!("Failed to disable raw mode: {}", err);
        }
        self.context.as_ref()?;
        // Clear terminal and return
        match self.context.take() {
            Some(mut ctx) => {
                ctx.clear_screen();
                Some(ctx)
            }
            None => None,
        }
    }
}
