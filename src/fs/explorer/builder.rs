//! ## Builder
//!
//! `builder` is the module which provides a builder for FileExplorer

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
use super::formatter::Formatter;
use super::{ExplorerOpts, FileExplorer, FileSorting, GroupDirs};
// Ext
use std::collections::VecDeque;

/// ## FileExplorerBuilder
///
/// Struct used to create a `FileExplorer`
pub struct FileExplorerBuilder {
    explorer: Option<FileExplorer>,
}

impl FileExplorerBuilder {
    /// ### new
    ///
    /// Build a new `FileExplorerBuilder`
    pub fn new() -> Self {
        FileExplorerBuilder {
            explorer: Some(FileExplorer::default()),
        }
    }

    /// ### build
    ///
    /// Take FileExplorer out of builder
    pub fn build(&mut self) -> FileExplorer {
        self.explorer.take().unwrap()
    }

    /// ### with_hidden_files
    ///
    /// Enable HIDDEN_FILES option
    pub fn with_hidden_files(&mut self, val: bool) -> &mut FileExplorerBuilder {
        if let Some(e) = self.explorer.as_mut() {
            match val {
                true => e.opts.insert(ExplorerOpts::SHOW_HIDDEN_FILES),
                false => e.opts.remove(ExplorerOpts::SHOW_HIDDEN_FILES),
            }
        }
        self
    }

    /// ### with_file_sorting
    ///
    /// Set sorting method
    pub fn with_file_sorting(&mut self, sorting: FileSorting) -> &mut FileExplorerBuilder {
        if let Some(e) = self.explorer.as_mut() {
            e.sort_by(sorting);
        }
        self
    }

    /// ### with_dirs_first
    ///
    /// Enable DIRS_FIRST option
    pub fn with_group_dirs(&mut self, group_dirs: Option<GroupDirs>) -> &mut FileExplorerBuilder {
        if let Some(e) = self.explorer.as_mut() {
            e.group_dirs_by(group_dirs);
        }
        self
    }

    /// ### with_stack_size
    ///
    /// Set stack size for FileExplorer
    pub fn with_stack_size(&mut self, sz: usize) -> &mut FileExplorerBuilder {
        if let Some(e) = self.explorer.as_mut() {
            e.stack_size = sz;
            e.dirstack = VecDeque::with_capacity(sz);
        }
        self
    }

    /// ### with_formatter
    ///
    /// Set formatter for FileExplorer
    pub fn with_formatter(&mut self, fmt_str: Option<&str>) -> &mut FileExplorerBuilder {
        if let Some(e) = self.explorer.as_mut() {
            if let Some(fmt_str) = fmt_str {
                e.fmt = Formatter::new(fmt_str);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_fs_explorer_builder_new_default() {
        let explorer: FileExplorer = FileExplorerBuilder::new().build();
        // Verify
        assert!(!explorer.opts.intersects(ExplorerOpts::SHOW_HIDDEN_FILES));
        assert_eq!(explorer.file_sorting, FileSorting::Name); // Default
        assert_eq!(explorer.group_dirs, None);
        assert_eq!(explorer.stack_size, 16);
    }

    #[test]
    fn test_fs_explorer_builder_new_all() {
        let explorer: FileExplorer = FileExplorerBuilder::new()
            .with_file_sorting(FileSorting::ModifyTime)
            .with_group_dirs(Some(GroupDirs::First))
            .with_hidden_files(true)
            .with_stack_size(24)
            .with_formatter(Some("{NAME}"))
            .build();
        // Verify
        assert!(explorer.opts.intersects(ExplorerOpts::SHOW_HIDDEN_FILES));
        assert_eq!(explorer.file_sorting, FileSorting::ModifyTime); // Default
        assert_eq!(explorer.group_dirs, Some(GroupDirs::First));
        assert_eq!(explorer.stack_size, 24);
    }
}
