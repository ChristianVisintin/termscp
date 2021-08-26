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
// Locals
use super::{Context, SetupActivity};
use crate::filetransfer::FileTransferProtocol;
use crate::fs::explorer::GroupDirs;
use crate::utils::ui::draw_area_in;
// Ext
use std::path::PathBuf;
use tui_realm_stdlib::{
    input::{Input, InputPropsBuilder},
    radio::{Radio, RadioPropsBuilder},
};
use tuirealm::tui::{
    layout::{Constraint, Direction, Layout},
    style::Color,
    widgets::{BorderType, Borders, Clear},
};
use tuirealm::{
    props::{Alignment, PropsBuilder},
    Payload, Value, View,
};

impl SetupActivity {
    // -- view

    /// ### init_setup
    ///
    /// Initialize setup view
    pub(super) fn init_setup(&mut self) {
        // Init view
        self.view = View::init();
        // Common stuff
        // Radio tab
        self.mount_header_tab(0);
        // Footer
        self.mount_footer();
        // Input fields
        self.view.mount(
            super::COMPONENT_INPUT_TEXT_EDITOR,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(Color::LightGreen)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightGreen)
                    .with_label("Text editor", Alignment::Left)
                    .build(),
            )),
        );
        self.view.active(super::COMPONENT_INPUT_TEXT_EDITOR); // <-- Focus
        self.view.mount(
            super::COMPONENT_RADIO_DEFAULT_PROTOCOL,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightCyan)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightCyan)
                    .with_title("Default file transfer protocol", Alignment::Left)
                    .with_options(&[
                        String::from("SFTP"),
                        String::from("SCP"),
                        String::from("FTP"),
                        String::from("FTPS"),
                    ])
                    .rewind(true)
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_RADIO_HIDDEN_FILES,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightRed)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightRed)
                    .with_title("Show hidden files (by default)?", Alignment::Left)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .rewind(true)
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_RADIO_UPDATES,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightYellow)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                    .with_title("Check for updates?", Alignment::Left)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .rewind(true)
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_RADIO_GROUP_DIRS,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightMagenta)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightMagenta)
                    .with_title("Group directories", Alignment::Left)
                    .with_options(&[
                        String::from("Display first"),
                        String::from("Display Last"),
                        String::from("No"),
                    ])
                    .rewind(true)
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_INPUT_LOCAL_FILE_FMT,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(Color::LightBlue)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightBlue)
                    .with_label("File formatter syntax (local)", Alignment::Left)
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_INPUT_REMOTE_FILE_FMT,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(Color::LightGreen)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightGreen)
                    .with_label("File formatter syntax (remote)", Alignment::Left)
                    .build(),
            )),
        );
        // Load values
        self.load_input_values();
    }

    pub(super) fn view_setup(&mut self) {
        let mut ctx: Context = self.context.take().unwrap();
        let _ = ctx.terminal().draw(|f| {
            // Prepare main chunks
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),  // Current tab
                        Constraint::Length(21), // Main body
                        Constraint::Length(3),  // Help footer
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // Render common widget
            self.view.render(super::COMPONENT_RADIO_TAB, f, chunks[0]);
            self.view.render(super::COMPONENT_TEXT_FOOTER, f, chunks[2]);
            // Make chunks
            let ui_cfg_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Text editor
                        Constraint::Length(3), // Protocol tab
                        Constraint::Length(3), // Hidden files
                        Constraint::Length(3), // Updates tab
                        Constraint::Length(3), // Group dirs
                        Constraint::Length(3), // Local Format input
                        Constraint::Length(3), // Remote Format input
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);
            self.view
                .render(super::COMPONENT_INPUT_TEXT_EDITOR, f, ui_cfg_chunks[0]);
            self.view
                .render(super::COMPONENT_RADIO_DEFAULT_PROTOCOL, f, ui_cfg_chunks[1]);
            self.view
                .render(super::COMPONENT_RADIO_HIDDEN_FILES, f, ui_cfg_chunks[2]);
            self.view
                .render(super::COMPONENT_RADIO_UPDATES, f, ui_cfg_chunks[3]);
            self.view
                .render(super::COMPONENT_RADIO_GROUP_DIRS, f, ui_cfg_chunks[4]);
            self.view
                .render(super::COMPONENT_INPUT_LOCAL_FILE_FMT, f, ui_cfg_chunks[5]);
            self.view
                .render(super::COMPONENT_INPUT_REMOTE_FILE_FMT, f, ui_cfg_chunks[6]);
            // Popups
            if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_ERROR) {
                if props.visible {
                    let popup = draw_area_in(f.size(), 50, 10);
                    f.render_widget(Clear, popup);
                    // make popup
                    self.view.render(super::COMPONENT_TEXT_ERROR, f, popup);
                }
            }
            if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_QUIT) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 40, 10);
                    f.render_widget(Clear, popup);
                    self.view.render(super::COMPONENT_RADIO_QUIT, f, popup);
                }
            }
            if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_HELP) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 50, 70);
                    f.render_widget(Clear, popup);
                    self.view.render(super::COMPONENT_TEXT_HELP, f, popup);
                }
            }
            if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_SAVE) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 30, 10);
                    f.render_widget(Clear, popup);
                    self.view.render(super::COMPONENT_RADIO_SAVE, f, popup);
                }
            }
        });
        // Put context back to context
        self.context = Some(ctx);
    }

    /// ### load_input_values
    ///
    /// Load values from configuration into input fields
    pub(crate) fn load_input_values(&mut self) {
        // Text editor
        if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_TEXT_EDITOR) {
            let text_editor: String =
                String::from(self.config().get_text_editor().as_path().to_string_lossy());
            let props = InputPropsBuilder::from(props)
                .with_value(text_editor)
                .build();
            let _ = self.view.update(super::COMPONENT_INPUT_TEXT_EDITOR, props);
        }
        // Protocol
        if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_DEFAULT_PROTOCOL) {
            let protocol: usize = match self.config().get_default_protocol() {
                FileTransferProtocol::Sftp => 0,
                FileTransferProtocol::Scp => 1,
                FileTransferProtocol::Ftp(false) => 2,
                FileTransferProtocol::Ftp(true) => 3,
            };
            let props = RadioPropsBuilder::from(props).with_value(protocol).build();
            let _ = self
                .view
                .update(super::COMPONENT_RADIO_DEFAULT_PROTOCOL, props);
        }
        // Hidden files
        if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_HIDDEN_FILES) {
            let hidden: usize = match self.config().get_show_hidden_files() {
                true => 0,
                false => 1,
            };
            let props = RadioPropsBuilder::from(props).with_value(hidden).build();
            let _ = self.view.update(super::COMPONENT_RADIO_HIDDEN_FILES, props);
        }
        // Updates
        if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_UPDATES) {
            let updates: usize = match self.config().get_check_for_updates() {
                true => 0,
                false => 1,
            };
            let props = RadioPropsBuilder::from(props).with_value(updates).build();
            let _ = self.view.update(super::COMPONENT_RADIO_UPDATES, props);
        }
        // Group dirs
        if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_GROUP_DIRS) {
            let dirs: usize = match self.config().get_group_dirs() {
                Some(GroupDirs::First) => 0,
                Some(GroupDirs::Last) => 1,
                None => 2,
            };
            let props = RadioPropsBuilder::from(props).with_value(dirs).build();
            let _ = self.view.update(super::COMPONENT_RADIO_GROUP_DIRS, props);
        }
        // Local File Fmt
        if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_LOCAL_FILE_FMT) {
            let file_fmt: String = self.config().get_local_file_fmt().unwrap_or_default();
            let props = InputPropsBuilder::from(props).with_value(file_fmt).build();
            let _ = self
                .view
                .update(super::COMPONENT_INPUT_LOCAL_FILE_FMT, props);
        }
        // Remote File Fmt
        if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_REMOTE_FILE_FMT) {
            let file_fmt: String = self.config().get_remote_file_fmt().unwrap_or_default();
            let props = InputPropsBuilder::from(props).with_value(file_fmt).build();
            let _ = self
                .view
                .update(super::COMPONENT_INPUT_REMOTE_FILE_FMT, props);
        }
    }

    /// ### collect_input_values
    ///
    /// Collect values from input and put them into the configuration
    pub(crate) fn collect_input_values(&mut self) {
        if let Some(Payload::One(Value::Str(editor))) =
            self.view.get_state(super::COMPONENT_INPUT_TEXT_EDITOR)
        {
            self.config_mut()
                .set_text_editor(PathBuf::from(editor.as_str()));
        }
        if let Some(Payload::One(Value::Usize(protocol))) =
            self.view.get_state(super::COMPONENT_RADIO_DEFAULT_PROTOCOL)
        {
            let protocol: FileTransferProtocol = match protocol {
                1 => FileTransferProtocol::Scp,
                2 => FileTransferProtocol::Ftp(false),
                3 => FileTransferProtocol::Ftp(true),
                _ => FileTransferProtocol::Sftp,
            };
            self.config_mut().set_default_protocol(protocol);
        }
        if let Some(Payload::One(Value::Usize(opt))) =
            self.view.get_state(super::COMPONENT_RADIO_HIDDEN_FILES)
        {
            let show: bool = matches!(opt, 0);
            self.config_mut().set_show_hidden_files(show);
        }
        if let Some(Payload::One(Value::Usize(opt))) =
            self.view.get_state(super::COMPONENT_RADIO_UPDATES)
        {
            let check: bool = matches!(opt, 0);
            self.config_mut().set_check_for_updates(check);
        }
        if let Some(Payload::One(Value::Str(fmt))) =
            self.view.get_state(super::COMPONENT_INPUT_LOCAL_FILE_FMT)
        {
            self.config_mut().set_local_file_fmt(fmt);
        }
        if let Some(Payload::One(Value::Str(fmt))) =
            self.view.get_state(super::COMPONENT_INPUT_REMOTE_FILE_FMT)
        {
            self.config_mut().set_remote_file_fmt(fmt);
        }
        if let Some(Payload::One(Value::Usize(opt))) =
            self.view.get_state(super::COMPONENT_RADIO_GROUP_DIRS)
        {
            let dirs: Option<GroupDirs> = match opt {
                0 => Some(GroupDirs::First),
                1 => Some(GroupDirs::Last),
                _ => None,
            };
            self.config_mut().set_group_dirs(dirs);
        }
    }
}
