//! ## FileTransferActivity
//!
//! `filetransfer_activiy` is the module which implements the Filetransfer activity, which is the main activity afterall

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
// This module is split into files, cause it's just too big
pub(self) mod actions;
pub(self) mod lib;
pub(self) mod misc;
pub(self) mod session;
pub(self) mod update;
pub(self) mod view;

// locals
use super::{Activity, Context, ExitReason};
use crate::config::themes::Theme;
use crate::filetransfer::ftp_transfer::FtpFileTransfer;
use crate::filetransfer::scp_transfer::ScpFileTransfer;
use crate::filetransfer::sftp_transfer::SftpFileTransfer;
use crate::filetransfer::{FileTransfer, FileTransferProtocol};
use crate::fs::explorer::FileExplorer;
use crate::fs::FsEntry;
use crate::host::Localhost;
use crate::system::config_client::ConfigClient;
pub(self) use lib::browser;
use lib::browser::Browser;
use lib::transfer::TransferStates;
pub(self) use session::TransferPayload;

// Includes
use chrono::{DateTime, Local};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::collections::VecDeque;
use tempfile::TempDir;
use tuirealm::View;

// -- Storage keys

const STORAGE_EXPLORER_WIDTH: &str = "FILETRANSFER_EXPLORER_WIDTH";

// -- components

const COMPONENT_EXPLORER_LOCAL: &str = "EXPLORER_LOCAL";
const COMPONENT_EXPLORER_REMOTE: &str = "EXPLORER_REMOTE";
const COMPONENT_EXPLORER_FIND: &str = "EXPLORER_FIND";
const COMPONENT_LOG_BOX: &str = "LOG_BOX";
const COMPONENT_PROGRESS_BAR_FULL: &str = "PROGRESS_BAR_FULL";
const COMPONENT_PROGRESS_BAR_PARTIAL: &str = "PROGRESS_BAR_PARTIAL";
const COMPONENT_TEXT_ERROR: &str = "TEXT_ERROR";
const COMPONENT_TEXT_FATAL: &str = "TEXT_FATAL";
const COMPONENT_TEXT_HELP: &str = "TEXT_HELP";
const COMPONENT_TEXT_WAIT: &str = "TEXT_WAIT";
const COMPONENT_INPUT_COPY: &str = "INPUT_COPY";
const COMPONENT_INPUT_EXEC: &str = "INPUT_EXEC";
const COMPONENT_INPUT_FIND: &str = "INPUT_FIND";
const COMPONENT_INPUT_GOTO: &str = "INPUT_GOTO";
const COMPONENT_INPUT_MKDIR: &str = "INPUT_MKDIR";
const COMPONENT_INPUT_NEWFILE: &str = "INPUT_NEWFILE";
const COMPONENT_INPUT_OPEN_WITH: &str = "INPUT_OPEN_WITH";
const COMPONENT_INPUT_RENAME: &str = "INPUT_RENAME";
const COMPONENT_INPUT_SAVEAS: &str = "INPUT_SAVEAS";
const COMPONENT_RADIO_DELETE: &str = "RADIO_DELETE";
const COMPONENT_RADIO_DISCONNECT: &str = "RADIO_DISCONNECT";
const COMPONENT_RADIO_QUIT: &str = "RADIO_QUIT";
const COMPONENT_RADIO_SORTING: &str = "RADIO_SORTING";
const COMPONENT_SPAN_STATUS_BAR_LOCAL: &str = "STATUS_BAR_LOCAL";
const COMPONENT_SPAN_STATUS_BAR_REMOTE: &str = "STATUS_BAR_REMOTE";
const COMPONENT_LIST_FILEINFO: &str = "LIST_FILEINFO";

/// ## LogLevel
///
/// Log level type
enum LogLevel {
    Error,
    Warn,
    Info,
}

/// ## LogRecord
///
/// Log record entry
struct LogRecord {
    pub time: DateTime<Local>,
    pub level: LogLevel,
    pub msg: String,
}

impl LogRecord {
    /// ### new
    ///
    /// Instantiates a new LogRecord
    pub fn new(level: LogLevel, msg: String) -> LogRecord {
        LogRecord {
            time: Local::now(),
            level,
            msg,
        }
    }
}

/// ## FileTransferActivity
///
/// FileTransferActivity is the data holder for the file transfer activity
pub struct FileTransferActivity {
    exit_reason: Option<ExitReason>,  // Exit reason
    context: Option<Context>,         // Context holder
    view: View,                       // View
    host: Localhost,                  // Localhost
    client: Box<dyn FileTransfer>,    // File transfer client
    browser: Browser,                 // Browser
    log_records: VecDeque<LogRecord>, // Log records
    transfer: TransferStates,         // Transfer states
    cache: Option<TempDir>,           // Temporary directory where to store stuff
}

impl FileTransferActivity {
    /// ### new
    ///
    /// Instantiates a new FileTransferActivity
    pub fn new(host: Localhost, protocol: FileTransferProtocol) -> FileTransferActivity {
        // Get config client
        let config_client: ConfigClient = Self::init_config_client();
        FileTransferActivity {
            exit_reason: None,
            context: None,
            view: View::init(),
            host,
            client: match protocol {
                FileTransferProtocol::Sftp => Box::new(SftpFileTransfer::new(
                    Self::make_ssh_storage(&config_client),
                )),
                FileTransferProtocol::Ftp(ftps) => Box::new(FtpFileTransfer::new(ftps)),
                FileTransferProtocol::Scp => {
                    Box::new(ScpFileTransfer::new(Self::make_ssh_storage(&config_client)))
                }
            },
            browser: Browser::new(&config_client),
            log_records: VecDeque::with_capacity(256), // 256 events is enough I guess
            transfer: TransferStates::default(),
            cache: match TempDir::new() {
                Ok(d) => Some(d),
                Err(_) => None,
            },
        }
    }

    fn local(&self) -> &FileExplorer {
        self.browser.local()
    }

    fn local_mut(&mut self) -> &mut FileExplorer {
        self.browser.local_mut()
    }

    fn remote(&self) -> &FileExplorer {
        self.browser.remote()
    }

    fn remote_mut(&mut self) -> &mut FileExplorer {
        self.browser.remote_mut()
    }

    fn found(&self) -> Option<&FileExplorer> {
        self.browser.found()
    }

    fn found_mut(&mut self) -> Option<&mut FileExplorer> {
        self.browser.found_mut()
    }

    /// ### get_cache_tmp_name
    ///
    /// Get file name for a file in cache
    fn get_cache_tmp_name(&self, name: &str, file_type: Option<&str>) -> Option<String> {
        self.cache.as_ref().map(|_| {
            let base: String = format!(
                "{}-{}",
                name,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            match file_type {
                None => base,
                Some(file_type) => format!("{}.{}", base, file_type),
            }
        })
    }

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

    /// ### config
    ///
    /// Returns config client reference
    fn config(&self) -> &ConfigClient {
        self.context().config()
    }

    /// ### theme
    ///
    /// Get a reference to `Theme`
    fn theme(&self) -> &Theme {
        self.context().theme_provider().theme()
    }
}

/**
 * Activity Trait
 * Keep it clean :)
 * Use methods instead!
 */

impl Activity for FileTransferActivity {
    /// ### on_create
    ///
    /// `on_create` is the function which must be called to initialize the activity.
    /// `on_create` must initialize all the data structures used by the activity
    fn on_create(&mut self, context: Context) {
        debug!("Initializing activity...");
        // Set context
        self.context = Some(context);
        // Clear terminal
        self.context_mut().clear_screen();
        // Put raw mode on enabled
        if let Err(err) = enable_raw_mode() {
            error!("Failed to enter raw mode: {}", err);
        }
        // Get files at current pwd
        self.reload_local_dir();
        debug!("Read working directory");
        // Configure text editor
        self.setup_text_editor();
        debug!("Setup text editor");
        // init view
        self.init();
        debug!("Initialized view");
        // Verify error state from context
        if let Some(err) = self.context.as_mut().unwrap().error() {
            error!("Fatal error on create: {}", err);
            self.mount_fatal(&err);
        }
        info!("Created FileTransferActivity");
    }

    /// ### on_draw
    ///
    /// `on_draw` is the function which draws the graphical interface.
    /// This function must be called at each tick to refresh the interface
    fn on_draw(&mut self) {
        // Should ui actually be redrawned?
        let mut redraw: bool = false;
        // Context must be something
        if self.context.is_none() {
            return;
        }
        // Check if connected (popup must be None, otherwise would try reconnecting in loop in case of error)
        if !self.client.is_connected() && self.view.get_props(COMPONENT_TEXT_FATAL).is_none() {
            let params = self.context().ft_params().unwrap();
            info!(
                "Client is not connected to remote; connecting to {}:{}",
                params.address, params.port
            );
            let msg: String = format!("Connecting to {}:{}…", params.address, params.port);
            // Set init state to connecting popup
            self.mount_wait(msg.as_str());
            // Force ui draw
            self.view();
            // Connect to remote
            self.connect();
            // Redraw
            redraw = true;
        }
        // Handle input events (if false, becomes true; otherwise remains true)
        redraw |= self.read_input_event();
        // @! draw interface
        if redraw {
            self.view();
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
    fn on_destroy(&mut self) -> Option<Context> {
        // Destroy cache
        if let Some(cache) = self.cache.take() {
            if let Err(err) = cache.close() {
                error!("Failed to delete cache: {}", err);
            }
        }
        // Disable raw mode
        if let Err(err) = disable_raw_mode() {
            error!("Failed to disable raw mode: {}", err);
        }
        // Disconnect client
        if self.client.is_connected() {
            let _ = self.client.disconnect();
        }
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
