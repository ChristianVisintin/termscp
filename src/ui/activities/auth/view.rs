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
use super::{AuthActivity, Context, FileTransferProtocol};
use crate::ui::components::bookmark_list::{BookmarkList, BookmarkListPropsBuilder};
use crate::utils::ui::draw_area_in;
// Ext
use tui_realm_stdlib::{
    input::{Input, InputPropsBuilder},
    label::{Label, LabelPropsBuilder},
    list::{List, ListPropsBuilder},
    paragraph::{Paragraph, ParagraphPropsBuilder},
    radio::{Radio, RadioPropsBuilder},
    span::{Span, SpanPropsBuilder},
    textarea::{Textarea, TextareaPropsBuilder},
};
use tuirealm::tui::{
    layout::{Constraint, Direction, Layout},
    style::Color,
    widgets::{BorderType, Borders, Clear},
};
use tuirealm::{
    props::{Alignment, InputType, PropsBuilder, TableBuilder, TextSpan},
    Msg, Payload, Value,
};

impl AuthActivity {
    /// ### init
    ///
    /// Initialize view, mounting all startup components inside the view
    pub(super) fn init(&mut self) {
        let key_color = self.theme().misc_keys;
        let addr_color = self.theme().auth_address;
        let protocol_color = self.theme().auth_protocol;
        let port_color = self.theme().auth_port;
        let username_color = self.theme().auth_username;
        let password_color = self.theme().auth_password;
        let bookmarks_color = self.theme().auth_bookmarks;
        let recents_color = self.theme().auth_recents;
        // Headers
        self.view.mount(
            super::COMPONENT_TEXT_H1,
            Box::new(Label::new(
                LabelPropsBuilder::default()
                    .bold()
                    .italic()
                    .with_text(String::from("$ termscp"))
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_TEXT_H2,
            Box::new(Label::new(
                LabelPropsBuilder::default()
                    .bold()
                    .italic()
                    .with_text(format!("$ version {}", env!("CARGO_PKG_VERSION")))
                    .build(),
            )),
        );
        // Footer
        self.view.mount(
            super::COMPONENT_TEXT_FOOTER,
            Box::new(Span::new(
                SpanPropsBuilder::default()
                    .with_spans(vec![
                        TextSpan::new("Press ").bold(),
                        TextSpan::new("<CTRL+H>").bold().fg(key_color),
                        TextSpan::new(" to show keybindings; ").bold(),
                        TextSpan::new("<CTRL+C>").bold().fg(key_color),
                        TextSpan::new(" to enter setup").bold(),
                    ])
                    .build(),
            )),
        );
        // Get default protocol
        let default_protocol: FileTransferProtocol = self.context().config().get_default_protocol();
        // Protocol
        self.view.mount(
            super::COMPONENT_RADIO_PROTOCOL,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(protocol_color)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, protocol_color)
                    .with_title("Protocol", Alignment::Left)
                    .with_options(&["SFTP", "SCP", "FTP", "FTPS"])
                    .with_value(Self::protocol_enum_to_opt(default_protocol))
                    .rewind(true)
                    .build(),
            )),
        );
        // Address
        self.view.mount(
            super::COMPONENT_INPUT_ADDR,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(addr_color)
                    .with_borders(Borders::ALL, BorderType::Rounded, addr_color)
                    .with_label("Remote host", Alignment::Left)
                    .build(),
            )),
        );
        // Port
        self.view.mount(
            super::COMPONENT_INPUT_PORT,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(port_color)
                    .with_borders(Borders::ALL, BorderType::Rounded, port_color)
                    .with_label("Port number", Alignment::Left)
                    .with_input(InputType::Number)
                    .with_input_len(5)
                    .with_value(Self::get_default_port_for_protocol(default_protocol).to_string())
                    .build(),
            )),
        );
        // Username
        self.view.mount(
            super::COMPONENT_INPUT_USERNAME,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(username_color)
                    .with_borders(Borders::ALL, BorderType::Rounded, username_color)
                    .with_label("Username", Alignment::Left)
                    .build(),
            )),
        );
        // Password
        self.view.mount(
            super::COMPONENT_INPUT_PASSWORD,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(password_color)
                    .with_borders(Borders::ALL, BorderType::Rounded, password_color)
                    .with_label("Password", Alignment::Left)
                    .with_input(InputType::Password)
                    .build(),
            )),
        );
        // Version notice
        if let Some(version) = self
            .context()
            .store()
            .get_string(super::STORE_KEY_LATEST_VERSION)
        {
            let version: String = version.to_string();
            self.view.mount(
                super::COMPONENT_TEXT_NEW_VERSION,
                Box::new(Span::new(
                    SpanPropsBuilder::default()
                        .with_foreground(Color::Yellow)
                        .with_spans(vec![
                            TextSpan::from("termscp "),
                            TextSpan::new(version.as_str()).underlined().bold(),
                            TextSpan::from(" is NOW available! Get it from <https://veeso.github.io/termscp/>; view release notes with <CTRL+R>"),
                        ])
                        .build(),
                )),
            );
        }
        // Bookmarks
        self.view.mount(
            super::COMPONENT_BOOKMARKS_LIST,
            Box::new(BookmarkList::new(
                BookmarkListPropsBuilder::default()
                    .with_background(bookmarks_color)
                    .with_foreground(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Plain, bookmarks_color)
                    .with_title("Bookmarks", Alignment::Left)
                    .build(),
            )),
        );
        // Recents
        self.view.mount(
            super::COMPONENT_RECENTS_LIST,
            Box::new(BookmarkList::new(
                BookmarkListPropsBuilder::default()
                    .with_background(recents_color)
                    .with_foreground(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Plain, recents_color)
                    .with_title("Recent connections", Alignment::Left)
                    .build(),
            )),
        );
        // Load bookmarks
        let _ = self.view_bookmarks();
        let _ = self.view_recent_connections();
        // Active protocol
        self.view.active(super::COMPONENT_RADIO_PROTOCOL);
    }

    /// ### view
    ///
    /// Display view on canvas
    pub(super) fn view(&mut self) {
        let mut ctx: Context = self.context.take().unwrap();
        let _ = ctx.terminal().draw(|f| {
            // Check window size
            let height: u16 = f.size().height;
            self.check_minimum_window_size(height);
            // Prepare chunks
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(21), // Auth Form
                        Constraint::Min(3),     // Bookmarks
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // Create explorer chunks
            let auth_chunks = Layout::default()
                .constraints(
                    [
                        Constraint::Length(1), // h1
                        Constraint::Length(1), // h2
                        Constraint::Length(1), // Version
                        Constraint::Length(3), // protocol
                        Constraint::Length(3), // host
                        Constraint::Length(3), // port
                        Constraint::Length(3), // username
                        Constraint::Length(3), // password
                        Constraint::Length(3), // footer
                    ]
                    .as_ref(),
                )
                .direction(Direction::Vertical)
                .split(chunks[0]);
            // Create bookmark chunks
            let bookmark_chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[1]);
            // Render
            // Auth chunks
            self.view
                .render(super::COMPONENT_TEXT_H1, f, auth_chunks[0]);
            self.view
                .render(super::COMPONENT_TEXT_H2, f, auth_chunks[1]);
            self.view
                .render(super::COMPONENT_TEXT_NEW_VERSION, f, auth_chunks[2]);
            self.view
                .render(super::COMPONENT_RADIO_PROTOCOL, f, auth_chunks[3]);
            self.view
                .render(super::COMPONENT_INPUT_ADDR, f, auth_chunks[4]);
            self.view
                .render(super::COMPONENT_INPUT_PORT, f, auth_chunks[5]);
            self.view
                .render(super::COMPONENT_INPUT_USERNAME, f, auth_chunks[6]);
            self.view
                .render(super::COMPONENT_INPUT_PASSWORD, f, auth_chunks[7]);
            self.view
                .render(super::COMPONENT_TEXT_FOOTER, f, auth_chunks[8]);
            // Bookmark chunks
            self.view
                .render(super::COMPONENT_BOOKMARKS_LIST, f, bookmark_chunks[0]);
            self.view
                .render(super::COMPONENT_RECENTS_LIST, f, bookmark_chunks[1]);
            // Popups
            if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_ERROR) {
                if props.visible {
                    let popup = draw_area_in(f.size(), 50, 10);
                    f.render_widget(Clear, popup);
                    // make popup
                    self.view.render(super::COMPONENT_TEXT_ERROR, f, popup);
                }
            }
            if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_SIZE_ERR) {
                if props.visible {
                    let popup = draw_area_in(f.size(), 80, 20);
                    f.render_widget(Clear, popup);
                    // make popup
                    self.view.render(super::COMPONENT_TEXT_SIZE_ERR, f, popup);
                }
            }
            if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_QUIT) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 30, 10);
                    f.render_widget(Clear, popup);
                    self.view.render(super::COMPONENT_RADIO_QUIT, f, popup);
                }
            }
            if let Some(props) = self
                .view
                .get_props(super::COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK)
            {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 30, 10);
                    f.render_widget(Clear, popup);
                    self.view
                        .render(super::COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK, f, popup);
                }
            }
            if let Some(props) = self
                .view
                .get_props(super::COMPONENT_RADIO_BOOKMARK_DEL_RECENT)
            {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 30, 10);
                    f.render_widget(Clear, popup);
                    self.view
                        .render(super::COMPONENT_RADIO_BOOKMARK_DEL_RECENT, f, popup);
                }
            }
            if let Some(props) = self.view.get_props(super::COMPONENT_TEXT_NEW_VERSION_NOTES) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 90, 90);
                    f.render_widget(Clear, popup);
                    self.view
                        .render(super::COMPONENT_TEXT_NEW_VERSION_NOTES, f, popup);
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
            if let Some(props) = self
                .view
                .get_props(super::COMPONENT_RADIO_BOOKMARK_SAVE_PWD)
            {
                if props.visible {
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
                    self.view
                        .render(super::COMPONENT_INPUT_BOOKMARK_NAME, f, popup_chunks[0]);
                    self.view
                        .render(super::COMPONENT_RADIO_BOOKMARK_SAVE_PWD, f, popup_chunks[1]);
                }
            }
        });
        self.context = Some(ctx);
    }

    // -- partials

    /// ### view_bookmarks
    ///
    /// Make text span from bookmarks
    pub(super) fn view_bookmarks(&mut self) -> Option<(String, Msg)> {
        let bookmarks: Vec<String> = self
            .bookmarks_list
            .iter()
            .map(|x| {
                let entry: (String, u16, FileTransferProtocol, String, _) = self
                    .bookmarks_client
                    .as_ref()
                    .unwrap()
                    .get_bookmark(x)
                    .unwrap();
                format!(
                    "{} ({}://{}@{}:{})",
                    x,
                    entry.2.to_string().to_lowercase(),
                    entry.3,
                    entry.0,
                    entry.1
                )
            })
            .collect();
        match self.view.get_props(super::COMPONENT_BOOKMARKS_LIST) {
            None => None,
            Some(props) => {
                let msg = self.view.update(
                    super::COMPONENT_BOOKMARKS_LIST,
                    BookmarkListPropsBuilder::from(props)
                        .with_bookmarks(bookmarks)
                        .build(),
                );
                msg
            }
        }
    }

    /// ### view_recent_connections
    ///
    /// View recent connections
    pub(super) fn view_recent_connections(&mut self) -> Option<(String, Msg)> {
        let bookmarks: Vec<String> = self
            .recents_list
            .iter()
            .map(|x| {
                let entry: (String, u16, FileTransferProtocol, String) = self
                    .bookmarks_client
                    .as_ref()
                    .unwrap()
                    .get_recent(x)
                    .unwrap();

                format!(
                    "{}://{}@{}:{}",
                    entry.2.to_string().to_lowercase(),
                    entry.3,
                    entry.0,
                    entry.1
                )
            })
            .collect();
        match self.view.get_props(super::COMPONENT_RECENTS_LIST) {
            None => None,
            Some(props) => {
                let msg = self.view.update(
                    super::COMPONENT_RECENTS_LIST,
                    BookmarkListPropsBuilder::from(props)
                        .with_bookmarks(bookmarks)
                        .build(),
                );
                msg
            }
        }
    }

    // -- mount

    /// ### mount_error
    ///
    /// Mount error box
    pub(super) fn mount_error(&mut self, text: &str) {
        // Mount
        let err_color = self.theme().misc_error_dialog;
        self.view.mount(
            super::COMPONENT_TEXT_ERROR,
            Box::new(Paragraph::new(
                ParagraphPropsBuilder::default()
                    .with_foreground(err_color)
                    .with_borders(Borders::ALL, BorderType::Thick, err_color)
                    .bold()
                    .with_text_alignment(Alignment::Center)
                    .with_texts(vec![TextSpan::from(text)])
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

    /// ### mount_size_err
    ///
    /// Mount size error
    pub(super) fn mount_size_err(&mut self) {
        // Mount
        let err_color = self.theme().misc_error_dialog;
        self.view.mount(
            super::COMPONENT_TEXT_SIZE_ERR,
            Box::new(Paragraph::new(
                ParagraphPropsBuilder::default()
                    .with_foreground(err_color)
                    .with_borders(Borders::ALL, BorderType::Thick, err_color)
                    .bold()
                    .with_texts(vec![TextSpan::from(
                        "termscp requires at least 24 lines of height to run",
                    )])
                    .with_text_alignment(Alignment::Center)
                    .build(),
            )),
        );
        // Give focus to error
        self.view.active(super::COMPONENT_TEXT_SIZE_ERR);
    }

    /// ### umount_size_err
    ///
    /// Umount error size error
    pub(super) fn umount_size_err(&mut self) {
        self.view.umount(super::COMPONENT_TEXT_SIZE_ERR);
    }

    /// ### mount_quit
    ///
    /// Mount quit popup
    pub(super) fn mount_quit(&mut self) {
        // Protocol
        let quit_color = self.theme().misc_quit_dialog;
        self.view.mount(
            super::COMPONENT_RADIO_QUIT,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(quit_color)
                    .with_borders(Borders::ALL, BorderType::Rounded, quit_color)
                    .with_inverted_color(Color::Black)
                    .with_title("Quit termscp?", Alignment::Center)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .rewind(true)
                    .build(),
            )),
        );
        self.view.active(super::COMPONENT_RADIO_QUIT);
    }

    /// ### umount_quit
    ///
    /// Umount quit popup
    pub(super) fn umount_quit(&mut self) {
        self.view.umount(super::COMPONENT_RADIO_QUIT);
    }

    /// ### mount_bookmark_del_dialog
    ///
    /// Mount bookmark delete dialog
    pub(super) fn mount_bookmark_del_dialog(&mut self) {
        let warn_color = self.theme().misc_warn_dialog;
        self.view.mount(
            super::COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(warn_color)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, warn_color)
                    .with_title("Delete bookmark?", Alignment::Center)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .with_value(1)
                    .rewind(true)
                    .build(),
            )),
        );
        // Active
        self.view
            .active(super::COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK);
    }

    /// ### umount_bookmark_del_dialog
    ///
    /// umount delete bookmark dialog
    pub(super) fn umount_bookmark_del_dialog(&mut self) {
        self.view
            .umount(super::COMPONENT_RADIO_BOOKMARK_DEL_BOOKMARK);
    }

    /// ### mount_bookmark_del_dialog
    ///
    /// Mount recent delete dialog
    pub(super) fn mount_recent_del_dialog(&mut self) {
        let warn_color = self.theme().misc_warn_dialog;
        self.view.mount(
            super::COMPONENT_RADIO_BOOKMARK_DEL_RECENT,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(warn_color)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, warn_color)
                    .with_title("Delete bookmark?", Alignment::Center)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .with_value(1)
                    .rewind(true)
                    .build(),
            )),
        );
        // Active
        self.view.active(super::COMPONENT_RADIO_BOOKMARK_DEL_RECENT);
    }

    /// ### umount_recent_del_dialog
    ///
    /// umount delete recent dialog
    pub(super) fn umount_recent_del_dialog(&mut self) {
        self.view.umount(super::COMPONENT_RADIO_BOOKMARK_DEL_RECENT);
    }

    /// ### mount_bookmark_save_dialog
    ///
    /// Mount bookmark save dialog
    pub(super) fn mount_bookmark_save_dialog(&mut self) {
        let save_color = self.theme().misc_save_dialog;
        let warn_color = self.theme().misc_warn_dialog;
        self.view.mount(
            super::COMPONENT_INPUT_BOOKMARK_NAME,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_foreground(save_color)
                    .with_label("Save bookmark as…", Alignment::Center)
                    .with_borders(
                        Borders::TOP | Borders::RIGHT | Borders::LEFT,
                        BorderType::Rounded,
                        Color::Reset,
                    )
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_RADIO_BOOKMARK_SAVE_PWD,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(warn_color)
                    .with_borders(
                        Borders::BOTTOM | Borders::RIGHT | Borders::LEFT,
                        BorderType::Rounded,
                        Color::Reset,
                    )
                    .with_title("Save password?", Alignment::Center)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .rewind(true)
                    .build(),
            )),
        );
        // Give focus to input bookmark name
        self.view.active(super::COMPONENT_INPUT_BOOKMARK_NAME);
    }

    /// ### umount_bookmark_save_dialog
    ///
    /// Umount bookmark save dialog
    pub(super) fn umount_bookmark_save_dialog(&mut self) {
        self.view.umount(super::COMPONENT_RADIO_BOOKMARK_SAVE_PWD);
        self.view.umount(super::COMPONENT_INPUT_BOOKMARK_NAME);
    }

    /// ### mount_help
    ///
    /// Mount help
    pub(super) fn mount_help(&mut self) {
        let key_color = self.theme().misc_keys;
        self.view.mount(
            super::COMPONENT_TEXT_HELP,
            Box::new(List::new(
                ListPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::White)
                    .with_highlighted_str(Some("?"))
                    .with_max_scroll_step(8)
                    .scrollable(true)
                    .bold()
                    .with_title("Help", Alignment::Center)
                    .with_rows(
                        TableBuilder::default()
                            .add_col(TextSpan::new("<ESC>").bold().fg(key_color))
                            .add_col(TextSpan::from("           Quit termscp"))
                            .add_row()
                            .add_col(TextSpan::new("<TAB>").bold().fg(key_color))
                            .add_col(TextSpan::from("           Switch from form and bookmarks"))
                            .add_row()
                            .add_col(TextSpan::new("<RIGHT/LEFT>").bold().fg(key_color))
                            .add_col(TextSpan::from("    Switch bookmark tab"))
                            .add_row()
                            .add_col(TextSpan::new("<UP/DOWN>").bold().fg(key_color))
                            .add_col(TextSpan::from("       Move up/down in current tab"))
                            .add_row()
                            .add_col(TextSpan::new("<ENTER>").bold().fg(key_color))
                            .add_col(TextSpan::from("         Connect/Load bookmark"))
                            .add_row()
                            .add_col(TextSpan::new("<DEL|E>").bold().fg(key_color))
                            .add_col(TextSpan::from("         Delete selected bookmark"))
                            .add_row()
                            .add_col(TextSpan::new("<CTRL+C>").bold().fg(key_color))
                            .add_col(TextSpan::from("        Enter setup"))
                            .add_row()
                            .add_col(TextSpan::new("<CTRL+S>").bold().fg(key_color))
                            .add_col(TextSpan::from("        Save bookmark"))
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

    /// ### mount_release_notes
    ///
    /// mount release notes text area
    pub(super) fn mount_release_notes(&mut self) {
        if let Some(ctx) = self.context.as_ref() {
            if let Some(release_notes) = ctx.store().get_string(super::STORE_KEY_RELEASE_NOTES) {
                // make spans
                let spans: Vec<TextSpan> = release_notes.lines().map(TextSpan::from).collect();
                self.view.mount(
                    super::COMPONENT_TEXT_NEW_VERSION_NOTES,
                    Box::new(Textarea::new(
                        TextareaPropsBuilder::default()
                            .with_borders(Borders::ALL, BorderType::Rounded, Color::LightYellow)
                            .with_title("Release notes", Alignment::Center)
                            .with_texts(spans)
                            .build(),
                    )),
                );
                self.view.active(super::COMPONENT_TEXT_NEW_VERSION_NOTES);
            }
        }
    }

    /// ### umount_release_notes
    ///
    /// Umount release notes text area
    pub(super) fn umount_release_notes(&mut self) {
        self.view.umount(super::COMPONENT_TEXT_NEW_VERSION_NOTES);
    }

    /// ### get_input
    ///
    /// Collect input values from view
    pub(super) fn get_input(&self) -> (String, u16, FileTransferProtocol, String, String) {
        let addr: String = self.get_input_addr();
        let port: u16 = self.get_input_port();
        let protocol: FileTransferProtocol = self.get_input_protocol();
        let username: String = self.get_input_username();
        let password: String = self.get_input_password();
        (addr, port, protocol, username, password)
    }

    pub(super) fn get_input_addr(&self) -> String {
        match self.view.get_state(super::COMPONENT_INPUT_ADDR) {
            Some(Payload::One(Value::Str(x))) => x,
            _ => String::new(),
        }
    }

    pub(super) fn get_input_port(&self) -> u16 {
        match self.view.get_state(super::COMPONENT_INPUT_PORT) {
            Some(Payload::One(Value::Usize(x))) => match x > 65535 {
                true => 0,
                false => x as u16,
            },
            _ => 0,
        }
    }

    pub(super) fn get_input_protocol(&self) -> FileTransferProtocol {
        match self.view.get_state(super::COMPONENT_RADIO_PROTOCOL) {
            Some(Payload::One(Value::Usize(x))) => Self::protocol_opt_to_enum(x),
            _ => FileTransferProtocol::Sftp,
        }
    }

    pub(super) fn get_input_username(&self) -> String {
        match self.view.get_state(super::COMPONENT_INPUT_USERNAME) {
            Some(Payload::One(Value::Str(x))) => x,
            _ => String::new(),
        }
    }

    pub(super) fn get_input_password(&self) -> String {
        match self.view.get_state(super::COMPONENT_INPUT_PASSWORD) {
            Some(Payload::One(Value::Str(x))) => x,
            _ => String::new(),
        }
    }
}
