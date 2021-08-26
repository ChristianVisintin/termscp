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
use crate::config::themes::Theme;
use crate::ui::components::color_picker::{ColorPicker, ColorPickerPropsBuilder};
use crate::utils::parser::parse_color;
use crate::utils::ui::draw_area_in;
// Ext
use tui_realm_stdlib::label::{Label, LabelPropsBuilder};
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

    /// ### init_theme
    ///
    /// Initialize thene view
    pub(super) fn init_theme(&mut self) {
        // Init view
        self.view = View::init();
        // Common stuff
        // Radio tab
        self.mount_header_tab(2);
        // Footer
        self.mount_footer();
        // auth colors
        self.mount_title(super::COMPONENT_COLOR_AUTH_TITLE, "Authentication styles");
        self.mount_color_picker(super::COMPONENT_COLOR_AUTH_PROTOCOL, "Protocol");
        self.mount_color_picker(super::COMPONENT_COLOR_AUTH_ADDR, "Ip address");
        self.mount_color_picker(super::COMPONENT_COLOR_AUTH_PORT, "Port");
        self.mount_color_picker(super::COMPONENT_COLOR_AUTH_USERNAME, "Username");
        self.mount_color_picker(super::COMPONENT_COLOR_AUTH_PASSWORD, "Password");
        self.mount_color_picker(super::COMPONENT_COLOR_AUTH_BOOKMARKS, "Bookmarks");
        self.mount_color_picker(super::COMPONENT_COLOR_AUTH_RECENTS, "Recent connections");
        // Misc
        self.mount_title(super::COMPONENT_COLOR_MISC_TITLE, "Misc styles");
        self.mount_color_picker(super::COMPONENT_COLOR_MISC_ERROR, "Error");
        self.mount_color_picker(super::COMPONENT_COLOR_MISC_INPUT, "Input fields");
        self.mount_color_picker(super::COMPONENT_COLOR_MISC_KEYS, "Key strokes");
        self.mount_color_picker(super::COMPONENT_COLOR_MISC_QUIT, "Quit dialogs");
        self.mount_color_picker(super::COMPONENT_COLOR_MISC_SAVE, "Save confirmations");
        self.mount_color_picker(super::COMPONENT_COLOR_MISC_WARN, "Warnings");
        // Transfer (1)
        self.mount_title(super::COMPONENT_COLOR_TRANSFER_TITLE, "Transfer styles");
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG,
            "Local explorer background",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG,
            "Local explorer foreground",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG,
            "Local explorer highlighted",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG,
            "Remote explorer background",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG,
            "Remote explorer foreground",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG,
            "Remote explorer highlighted",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL,
            "'Full transfer' Progress bar",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL,
            "'Partial transfer' Progress bar",
        );
        // Transfer (2)
        self.mount_title(
            super::COMPONENT_COLOR_TRANSFER_TITLE_2,
            "Transfer styles (2)",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_LOG_BG,
            "Log window background",
        );
        self.mount_color_picker(super::COMPONENT_COLOR_TRANSFER_LOG_WIN, "Log window");
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_STATUS_SORTING,
            "File sorting",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN,
            "Hidden files",
        );
        self.mount_color_picker(
            super::COMPONENT_COLOR_TRANSFER_STATUS_SYNC,
            "Synchronized browsing",
        );
        // Load styles
        self.load_styles();
        // Active first field
        self.view.active(super::COMPONENT_COLOR_AUTH_PROTOCOL);
    }

    pub(super) fn view_theme(&mut self) {
        let mut ctx: Context = self.context.take().unwrap();
        let _ = ctx.terminal().draw(|f| {
            // Prepare main chunks
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),  // Current tab
                        Constraint::Length(22), // Main body
                        Constraint::Length(3),  // Help footer
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // Render common widget
            self.view.render(super::COMPONENT_RADIO_TAB, f, chunks[0]);
            self.view.render(super::COMPONENT_TEXT_FOOTER, f, chunks[2]);
            // Make chunks
            let colors_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);
            let auth_colors_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1), // Title
                        Constraint::Length(3), // Protocol
                        Constraint::Length(3), // Addr
                        Constraint::Length(3), // Port
                        Constraint::Length(3), // Username
                        Constraint::Length(3), // Password
                        Constraint::Length(3), // Bookmarks
                        Constraint::Length(3), // Recents
                    ]
                    .as_ref(),
                )
                .split(colors_layout[0]);
            self.view
                .render(super::COMPONENT_COLOR_AUTH_TITLE, f, auth_colors_layout[0]);
            self.view.render(
                super::COMPONENT_COLOR_AUTH_PROTOCOL,
                f,
                auth_colors_layout[1],
            );
            self.view
                .render(super::COMPONENT_COLOR_AUTH_ADDR, f, auth_colors_layout[2]);
            self.view
                .render(super::COMPONENT_COLOR_AUTH_PORT, f, auth_colors_layout[3]);
            self.view.render(
                super::COMPONENT_COLOR_AUTH_USERNAME,
                f,
                auth_colors_layout[4],
            );
            self.view.render(
                super::COMPONENT_COLOR_AUTH_PASSWORD,
                f,
                auth_colors_layout[5],
            );
            self.view.render(
                super::COMPONENT_COLOR_AUTH_BOOKMARKS,
                f,
                auth_colors_layout[6],
            );
            self.view.render(
                super::COMPONENT_COLOR_AUTH_RECENTS,
                f,
                auth_colors_layout[7],
            );
            let misc_colors_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1), // Title
                        Constraint::Length(3), // Error
                        Constraint::Length(3), // Input
                        Constraint::Length(3), // Keys
                        Constraint::Length(3), // Quit
                        Constraint::Length(3), // Save
                        Constraint::Length(3), // Warn
                        Constraint::Length(3), // Empty
                    ]
                    .as_ref(),
                )
                .split(colors_layout[1]);
            self.view
                .render(super::COMPONENT_COLOR_MISC_TITLE, f, misc_colors_layout[0]);
            self.view
                .render(super::COMPONENT_COLOR_MISC_ERROR, f, misc_colors_layout[1]);
            self.view
                .render(super::COMPONENT_COLOR_MISC_INPUT, f, misc_colors_layout[2]);
            self.view
                .render(super::COMPONENT_COLOR_MISC_KEYS, f, misc_colors_layout[3]);
            self.view
                .render(super::COMPONENT_COLOR_MISC_QUIT, f, misc_colors_layout[4]);
            self.view
                .render(super::COMPONENT_COLOR_MISC_SAVE, f, misc_colors_layout[5]);
            self.view
                .render(super::COMPONENT_COLOR_MISC_WARN, f, misc_colors_layout[6]);

            let transfer_colors_layout_col1 = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1), // Title
                        Constraint::Length(3), // local explorer bg
                        Constraint::Length(3), // local explorer fg
                        Constraint::Length(3), // local explorer hg
                        Constraint::Length(3), // remote explorer bg
                        Constraint::Length(3), // remote explorer fg
                        Constraint::Length(3), // remote explorer hg
                        Constraint::Length(3), // empty
                    ]
                    .as_ref(),
                )
                .split(colors_layout[2]);
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_TITLE,
                f,
                transfer_colors_layout_col1[0],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG,
                f,
                transfer_colors_layout_col1[1],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG,
                f,
                transfer_colors_layout_col1[2],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG,
                f,
                transfer_colors_layout_col1[3],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG,
                f,
                transfer_colors_layout_col1[4],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG,
                f,
                transfer_colors_layout_col1[5],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG,
                f,
                transfer_colors_layout_col1[6],
            );
            let transfer_colors_layout_col2 = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1), // Title
                        Constraint::Length(3), // Full prog bar
                        Constraint::Length(3), // Partial prog bar
                        Constraint::Length(3), // log bg
                        Constraint::Length(3), // log window
                        Constraint::Length(3), // status sorting
                        Constraint::Length(3), // status hidden
                        Constraint::Length(3), // sync browsing
                    ]
                    .as_ref(),
                )
                .split(colors_layout[3]);
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_TITLE_2,
                f,
                transfer_colors_layout_col2[0],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL,
                f,
                transfer_colors_layout_col2[1],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL,
                f,
                transfer_colors_layout_col2[2],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_LOG_BG,
                f,
                transfer_colors_layout_col2[3],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_LOG_WIN,
                f,
                transfer_colors_layout_col2[4],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_STATUS_SORTING,
                f,
                transfer_colors_layout_col2[5],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN,
                f,
                transfer_colors_layout_col2[6],
            );
            self.view.render(
                super::COMPONENT_COLOR_TRANSFER_STATUS_SYNC,
                f,
                transfer_colors_layout_col2[7],
            );
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

    /// ### load_styles
    ///
    /// Load values from theme into input fields
    pub(crate) fn load_styles(&mut self) {
        let theme: Theme = self.theme().clone();
        self.update_color(super::COMPONENT_COLOR_AUTH_ADDR, theme.auth_address);
        self.update_color(super::COMPONENT_COLOR_AUTH_BOOKMARKS, theme.auth_bookmarks);
        self.update_color(super::COMPONENT_COLOR_AUTH_PASSWORD, theme.auth_password);
        self.update_color(super::COMPONENT_COLOR_AUTH_PORT, theme.auth_port);
        self.update_color(super::COMPONENT_COLOR_AUTH_PROTOCOL, theme.auth_protocol);
        self.update_color(super::COMPONENT_COLOR_AUTH_RECENTS, theme.auth_recents);
        self.update_color(super::COMPONENT_COLOR_AUTH_USERNAME, theme.auth_username);
        self.update_color(super::COMPONENT_COLOR_MISC_ERROR, theme.misc_error_dialog);
        self.update_color(super::COMPONENT_COLOR_MISC_INPUT, theme.misc_input_dialog);
        self.update_color(super::COMPONENT_COLOR_MISC_KEYS, theme.misc_keys);
        self.update_color(super::COMPONENT_COLOR_MISC_QUIT, theme.misc_quit_dialog);
        self.update_color(super::COMPONENT_COLOR_MISC_SAVE, theme.misc_save_dialog);
        self.update_color(super::COMPONENT_COLOR_MISC_WARN, theme.misc_warn_dialog);
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG,
            theme.transfer_local_explorer_background,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG,
            theme.transfer_local_explorer_foreground,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG,
            theme.transfer_local_explorer_highlighted,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG,
            theme.transfer_remote_explorer_background,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG,
            theme.transfer_remote_explorer_foreground,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG,
            theme.transfer_remote_explorer_highlighted,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL,
            theme.transfer_progress_bar_full,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL,
            theme.transfer_progress_bar_partial,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_LOG_BG,
            theme.transfer_log_background,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_LOG_WIN,
            theme.transfer_log_window,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_STATUS_SORTING,
            theme.transfer_status_sorting,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN,
            theme.transfer_status_hidden,
        );
        self.update_color(
            super::COMPONENT_COLOR_TRANSFER_STATUS_SYNC,
            theme.transfer_status_sync_browsing,
        );
    }

    /// ### collect_styles
    ///
    /// Collect values from input and put them into the theme.
    /// If a component has an invalid color, returns Err(component_id)
    pub(crate) fn collect_styles(&mut self) -> Result<(), &'static str> {
        // auth
        let auth_address: Color = self
            .get_color(super::COMPONENT_COLOR_AUTH_ADDR)
            .map_err(|_| super::COMPONENT_COLOR_AUTH_ADDR)?;
        let auth_bookmarks: Color = self
            .get_color(super::COMPONENT_COLOR_AUTH_BOOKMARKS)
            .map_err(|_| super::COMPONENT_COLOR_AUTH_BOOKMARKS)?;
        let auth_password: Color = self
            .get_color(super::COMPONENT_COLOR_AUTH_PASSWORD)
            .map_err(|_| super::COMPONENT_COLOR_AUTH_PASSWORD)?;
        let auth_port: Color = self
            .get_color(super::COMPONENT_COLOR_AUTH_PORT)
            .map_err(|_| super::COMPONENT_COLOR_AUTH_PORT)?;
        let auth_protocol: Color = self
            .get_color(super::COMPONENT_COLOR_AUTH_PROTOCOL)
            .map_err(|_| super::COMPONENT_COLOR_AUTH_PROTOCOL)?;
        let auth_recents: Color = self
            .get_color(super::COMPONENT_COLOR_AUTH_RECENTS)
            .map_err(|_| super::COMPONENT_COLOR_AUTH_RECENTS)?;
        let auth_username: Color = self
            .get_color(super::COMPONENT_COLOR_AUTH_USERNAME)
            .map_err(|_| super::COMPONENT_COLOR_AUTH_USERNAME)?;
        // misc
        let misc_error_dialog: Color = self
            .get_color(super::COMPONENT_COLOR_MISC_ERROR)
            .map_err(|_| super::COMPONENT_COLOR_MISC_ERROR)?;
        let misc_input_dialog: Color = self
            .get_color(super::COMPONENT_COLOR_MISC_INPUT)
            .map_err(|_| super::COMPONENT_COLOR_MISC_INPUT)?;
        let misc_keys: Color = self
            .get_color(super::COMPONENT_COLOR_MISC_KEYS)
            .map_err(|_| super::COMPONENT_COLOR_MISC_KEYS)?;
        let misc_quit_dialog: Color = self
            .get_color(super::COMPONENT_COLOR_MISC_QUIT)
            .map_err(|_| super::COMPONENT_COLOR_MISC_QUIT)?;
        let misc_save_dialog: Color = self
            .get_color(super::COMPONENT_COLOR_MISC_SAVE)
            .map_err(|_| super::COMPONENT_COLOR_MISC_SAVE)?;
        let misc_warn_dialog: Color = self
            .get_color(super::COMPONENT_COLOR_MISC_WARN)
            .map_err(|_| super::COMPONENT_COLOR_MISC_WARN)?;
        // transfer
        let transfer_local_explorer_background: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_BG)?;
        let transfer_local_explorer_foreground: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_FG)?;
        let transfer_local_explorer_highlighted: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_EXPLORER_LOCAL_HG)?;
        let transfer_remote_explorer_background: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_BG)?;
        let transfer_remote_explorer_foreground: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_FG)?;
        let transfer_remote_explorer_highlighted: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_EXPLORER_REMOTE_HG)?;
        let transfer_log_background: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_LOG_BG)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_LOG_BG)?;
        let transfer_log_window: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_LOG_WIN)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_LOG_WIN)?;
        let transfer_progress_bar_full: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_PROG_BAR_FULL)?;
        let transfer_progress_bar_partial: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_PROG_BAR_PARTIAL)?;
        let transfer_status_hidden: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_STATUS_HIDDEN)?;
        let transfer_status_sorting: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_STATUS_SORTING)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_STATUS_SORTING)?;
        let transfer_status_sync_browsing: Color = self
            .get_color(super::COMPONENT_COLOR_TRANSFER_STATUS_SYNC)
            .map_err(|_| super::COMPONENT_COLOR_TRANSFER_STATUS_SYNC)?;
        // Update theme
        let mut theme: &mut Theme = self.theme_mut();
        theme.auth_address = auth_address;
        theme.auth_bookmarks = auth_bookmarks;
        theme.auth_password = auth_password;
        theme.auth_port = auth_port;
        theme.auth_protocol = auth_protocol;
        theme.auth_recents = auth_recents;
        theme.auth_username = auth_username;
        theme.misc_error_dialog = misc_error_dialog;
        theme.misc_input_dialog = misc_input_dialog;
        theme.misc_keys = misc_keys;
        theme.misc_quit_dialog = misc_quit_dialog;
        theme.misc_save_dialog = misc_save_dialog;
        theme.misc_warn_dialog = misc_warn_dialog;
        theme.transfer_local_explorer_background = transfer_local_explorer_background;
        theme.transfer_local_explorer_foreground = transfer_local_explorer_foreground;
        theme.transfer_local_explorer_highlighted = transfer_local_explorer_highlighted;
        theme.transfer_remote_explorer_background = transfer_remote_explorer_background;
        theme.transfer_remote_explorer_foreground = transfer_remote_explorer_foreground;
        theme.transfer_remote_explorer_highlighted = transfer_remote_explorer_highlighted;
        theme.transfer_log_background = transfer_log_background;
        theme.transfer_log_window = transfer_log_window;
        theme.transfer_progress_bar_full = transfer_progress_bar_full;
        theme.transfer_progress_bar_partial = transfer_progress_bar_partial;
        theme.transfer_status_hidden = transfer_status_hidden;
        theme.transfer_status_sorting = transfer_status_sorting;
        theme.transfer_status_sync_browsing = transfer_status_sync_browsing;
        Ok(())
    }

    /// ### update_color
    ///
    /// Update color for provided component
    fn update_color(&mut self, component: &str, color: Color) {
        if let Some(props) = self.view.get_props(component) {
            self.view.update(
                component,
                ColorPickerPropsBuilder::from(props)
                    .with_color(&color)
                    .build(),
            );
        }
    }

    /// ### get_color
    ///
    /// Get color from component
    fn get_color(&self, component: &str) -> Result<Color, ()> {
        match self.view.get_state(component) {
            Some(Payload::One(Value::Str(color))) => match parse_color(color.as_str()) {
                Some(c) => Ok(c),
                None => Err(()),
            },
            _ => Err(()),
        }
    }

    /// ### mount_color_picker
    ///
    /// Mount color picker with provided data
    fn mount_color_picker(&mut self, id: &str, label: &str) {
        self.view.mount(
            id,
            Box::new(ColorPicker::new(
                ColorPickerPropsBuilder::default()
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::Reset)
                    .with_label(label.to_string(), Alignment::Left)
                    .build(),
            )),
        );
    }

    /// ### mount_title
    ///
    /// Mount title
    fn mount_title(&mut self, id: &str, text: &str) {
        self.view.mount(
            id,
            Box::new(Label::new(
                LabelPropsBuilder::default()
                    .bold()
                    .with_text(text.to_string())
                    .build(),
            )),
        );
    }
}
