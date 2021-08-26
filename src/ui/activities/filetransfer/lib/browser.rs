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
use crate::fs::explorer::{builder::FileExplorerBuilder, FileExplorer, FileSorting, GroupDirs};
use crate::fs::FsEntry;
use crate::system::config_client::ConfigClient;

/// ## FileExplorerTab
///
/// File explorer tab
#[derive(Clone, Copy)]
pub enum FileExplorerTab {
    Local,
    Remote,
    FindLocal,  // Find result tab
    FindRemote, // Find result tab
}

/// ## Browser
///
/// Browser contains the browser options
pub struct Browser {
    local: FileExplorer,         // Local File explorer state
    remote: FileExplorer,        // Remote File explorer state
    found: Option<FileExplorer>, // File explorer for find result
    tab: FileExplorerTab,        // Current selected tab
    pub sync_browsing: bool,
}

impl Browser {
    /// ### new
    ///
    /// Build a new `Browser` struct
    pub fn new(cli: &ConfigClient) -> Self {
        Self {
            local: Self::build_local_explorer(cli),
            remote: Self::build_remote_explorer(cli),
            found: None,
            tab: FileExplorerTab::Local,
            sync_browsing: false,
        }
    }

    pub fn local(&self) -> &FileExplorer {
        &self.local
    }

    pub fn local_mut(&mut self) -> &mut FileExplorer {
        &mut self.local
    }

    pub fn remote(&self) -> &FileExplorer {
        &self.remote
    }

    pub fn remote_mut(&mut self) -> &mut FileExplorer {
        &mut self.remote
    }

    pub fn found(&self) -> Option<&FileExplorer> {
        self.found.as_ref()
    }

    pub fn found_mut(&mut self) -> Option<&mut FileExplorer> {
        self.found.as_mut()
    }

    pub fn set_found(&mut self, files: Vec<FsEntry>) {
        let mut explorer = Self::build_found_explorer();
        explorer.set_files(files);
        self.found = Some(explorer);
    }

    pub fn del_found(&mut self) {
        self.found = None;
    }

    pub fn tab(&self) -> FileExplorerTab {
        self.tab
    }

    /// ### change_tab
    ///
    /// Update tab value
    pub fn change_tab(&mut self, tab: FileExplorerTab) {
        self.tab = tab;
    }

    /// ### toggle_sync_browsing
    ///
    /// Invert the current state for the sync browsing
    pub fn toggle_sync_browsing(&mut self) {
        self.sync_browsing = !self.sync_browsing;
    }

    /// ### build_local_explorer
    ///
    /// Build a file explorer with local host setup
    pub fn build_local_explorer(cli: &ConfigClient) -> FileExplorer {
        let mut builder = Self::build_explorer(cli);
        builder.with_formatter(cli.get_local_file_fmt().as_deref());
        builder.build()
    }

    /// ### build_remote_explorer
    ///
    /// Build a file explorer with remote host setup
    pub fn build_remote_explorer(cli: &ConfigClient) -> FileExplorer {
        let mut builder = Self::build_explorer(cli);
        builder.with_formatter(cli.get_remote_file_fmt().as_deref());
        builder.build()
    }

    /// ### build_explorer
    ///
    /// Build explorer reading configuration from `ConfigClient`
    fn build_explorer(cli: &ConfigClient) -> FileExplorerBuilder {
        let mut builder: FileExplorerBuilder = FileExplorerBuilder::new();
        // Set common keys
        builder
            .with_file_sorting(FileSorting::Name)
            .with_stack_size(16)
            .with_group_dirs(cli.get_group_dirs())
            .with_hidden_files(cli.get_show_hidden_files());
        builder
    }

    /// ### build_found_explorer
    ///
    /// Build explorer reading from `ConfigClient`, for found result (has some differences)
    fn build_found_explorer() -> FileExplorer {
        FileExplorerBuilder::new()
            .with_file_sorting(FileSorting::Name)
            .with_group_dirs(Some(GroupDirs::First))
            .with_hidden_files(true)
            .with_stack_size(0)
            .with_formatter(Some("{NAME:32} {SYMLINK}"))
            .build()
    }
}
