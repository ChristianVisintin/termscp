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
pub mod setup;
pub mod ssh_keys;
pub mod theme;

use super::*;
pub use setup::*;
pub use ssh_keys::*;
pub use theme::*;
// Ext
use tui_realm_stdlib::{
    list::{List, ListPropsBuilder},
    paragraph::{Paragraph, ParagraphPropsBuilder},
    radio::{Radio, RadioPropsBuilder},
    span::{Span, SpanPropsBuilder},
};
use tuirealm::props::{Alignment, PropsBuilder, TableBuilder, TextSpan};
use tuirealm::tui::{
    style::Color,
    widgets::{BorderType, Borders},
};

impl SetupActivity {
    // -- view

    pub(super) fn init(&mut self, layout: ViewLayout) {
        self.layout = layout;
        match self.layout {
            ViewLayout::SetupForm => self.init_setup(),
            ViewLayout::SshKeys => self.init_ssh_keys(),
            ViewLayout::Theme => self.init_theme(),
        }
    }

    /// ### view
    ///
    /// View gui
    pub(super) fn view(&mut self) {
        match self.layout {
            ViewLayout::SetupForm => self.view_setup(),
            ViewLayout::SshKeys => self.view_ssh_keys(),
            ViewLayout::Theme => self.view_theme(),
        }
    }

    // -- mount

    /// ### mount_error
    ///
    /// Mount error box
    pub(super) fn mount_error(&mut self, text: &str) {
        // Mount
        self.view.mount(
            super::COMPONENT_TEXT_ERROR,
            Box::new(Paragraph::new(
                ParagraphPropsBuilder::default()
                    .with_foreground(Color::Red)
                    .bold()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Red)
                    .with_texts(vec![TextSpan::from(text)])
                    .with_text_alignment(Alignment::Center)
                    .build(),
            )),
        );
        // Give focus to error
        self.view.active(super::COMPONENT_TEXT_ERROR);
    }

    /// ### umount_error
    ///
    /// Umount error message
    pub(super) fn umount_error(&mut self) {
        self.view.umount(super::COMPONENT_TEXT_ERROR);
    }

    /// ### mount_quit
    ///
    /// Mount quit popup
    pub(super) fn mount_quit(&mut self) {
        self.view.mount(
            super::COMPONENT_RADIO_QUIT,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightRed)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightRed)
                    .with_title(
                        "There are unsaved changes! Save changes before leaving?",
                        Alignment::Center,
                    )
                    .with_options(&[
                        String::from("Save"),
                        String::from("Don't save"),
                        String::from("Cancel"),
                    ])
                    .rewind(true)
                    .build(),
            )),
        );
        // Active
        self.view.active(super::COMPONENT_RADIO_QUIT);
    }

    /// ### umount_quit
    ///
    /// Umount quit
    pub(super) fn umount_quit(&mut self) {
        self.view.umount(super::COMPONENT_RADIO_QUIT);
    }

    /// ### mount_save_popup
    ///
    /// Mount save popup
    pub(super) fn mount_save_popup(&mut self) {
        self.view.mount(
            super::COMPONENT_RADIO_SAVE,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightYellow)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                    .with_title("Save changes?", Alignment::Center)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .rewind(true)
                    .build(),
            )),
        );
        // Active
        self.view.active(super::COMPONENT_RADIO_SAVE);
    }

    /// ### umount_quit
    ///
    /// Umount quit
    pub(super) fn umount_save_popup(&mut self) {
        self.view.umount(super::COMPONENT_RADIO_SAVE);
    }

    pub(self) fn mount_header_tab(&mut self, idx: usize) {
        self.view.mount(
            super::COMPONENT_RADIO_TAB,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightYellow)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::BOTTOM, BorderType::Thick, Color::White)
                    .with_options(&[
                        String::from("User Interface"),
                        String::from("SSH Keys"),
                        String::from("Theme"),
                    ])
                    .with_value(idx)
                    .rewind(true)
                    .build(),
            )),
        );
    }

    pub(self) fn mount_footer(&mut self) {
        self.view.mount(
            super::COMPONENT_TEXT_FOOTER,
            Box::new(Span::new(
                SpanPropsBuilder::default()
                    .with_spans(vec![
                        TextSpan::new("Press ").bold(),
                        TextSpan::new("<CTRL+H>").bold().fg(Color::Cyan),
                        TextSpan::new(" to show keybindings").bold(),
                    ])
                    .build(),
            )),
        );
    }

    /// ### mount_help
    ///
    /// Mount help
    pub(super) fn mount_help(&mut self) {
        self.view.mount(
            super::COMPONENT_TEXT_HELP,
            Box::new(List::new(
                ListPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::White)
                    .with_highlighted_str(Some("?"))
                    .with_max_scroll_step(8)
                    .bold()
                    .with_title("Help", Alignment::Center)
                    .scrollable(true)
                    .with_rows(
                        TableBuilder::default()
                            .add_col(TextSpan::new("<ESC>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("           Exit setup"))
                            .add_row()
                            .add_col(TextSpan::new("<TAB>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("           Change setup page"))
                            .add_row()
                            .add_col(TextSpan::new("<RIGHT/LEFT>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("    Change cursor"))
                            .add_row()
                            .add_col(TextSpan::new("<UP/DOWN>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("       Change input field"))
                            .add_row()
                            .add_col(TextSpan::new("<ENTER>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("         Select / Dismiss popup"))
                            .add_row()
                            .add_col(TextSpan::new("<DEL|E>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("         Delete SSH key"))
                            .add_row()
                            .add_col(TextSpan::new("<CTRL+N>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("        New SSH key"))
                            .add_row()
                            .add_col(TextSpan::new("<CTRL+R>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("        Revert changes"))
                            .add_row()
                            .add_col(TextSpan::new("<CTRL+S>").bold().fg(Color::Cyan))
                            .add_col(TextSpan::from("        Save configuration"))
                            .build(),
                    )
                    .build(),
            )),
        );
        // Active help
        self.view.active(super::COMPONENT_TEXT_HELP);
    }

    /// ### umount_help
    ///
    /// Umount help
    pub(super) fn umount_help(&mut self) {
        self.view.umount(super::COMPONENT_TEXT_HELP);
    }
}
