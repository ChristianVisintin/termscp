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
use crate::ui::components::bookmark_list::{BookmarkList, BookmarkListPropsBuilder};
use crate::utils::ui::draw_area_in;
// Ext
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
    View,
};

impl SetupActivity {
    // -- view

    /// ### init_ssh_keys
    ///
    /// Initialize ssh keys view
    pub(super) fn init_ssh_keys(&mut self) {
        // Init view
        self.view = View::init();
        // Common stuff
        // Radio tab
        // Radio tab
        self.mount_header_tab(1);
        // Footer
        self.mount_footer();
        self.view.mount(
            super::COMPONENT_LIST_SSH_KEYS,
            Box::new(BookmarkList::new(
                BookmarkListPropsBuilder::default()
                    .with_title("SSH keys", Alignment::Left)
                    .with_borders(Borders::ALL, BorderType::Plain, Color::LightGreen)
                    .with_background(Color::LightGreen)
                    .with_foreground(Color::Black)
                    .build(),
            )),
        );
        // Give focus
        self.view.active(super::COMPONENT_LIST_SSH_KEYS);
        // Load keys
        self.reload_ssh_keys();
    }

    pub(crate) fn view_ssh_keys(&mut self) {
        let mut ctx: Context = self.context.take().unwrap();
        let _ = ctx.terminal().draw(|f| {
            // Prepare main chunks
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),      // Current tab
                        Constraint::Percentage(90), // Main body
                        Constraint::Length(3),      // Help footer
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // Render common widget
            self.view.render(super::COMPONENT_RADIO_TAB, f, chunks[0]);
            self.view.render(super::COMPONENT_TEXT_FOOTER, f, chunks[2]);
            self.view
                .render(super::COMPONENT_LIST_SSH_KEYS, f, chunks[1]);
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
            if let Some(props) = self.view.get_props(super::COMPONENT_RADIO_DEL_SSH_KEY) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 30, 10);
                    f.render_widget(Clear, popup);
                    self.view
                        .render(super::COMPONENT_RADIO_DEL_SSH_KEY, f, popup);
                }
            }
            if let Some(props) = self.view.get_props(super::COMPONENT_INPUT_SSH_HOST) {
                if props.visible {
                    // make popup
                    let popup = draw_area_in(f.size(), 50, 20);
                    f.render_widget(Clear, popup);
                    let popup_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Length(3), // Host
                                Constraint::Length(3), // Username
                            ]
                            .as_ref(),
                        )
                        .split(popup);
                    self.view
                        .render(super::COMPONENT_INPUT_SSH_HOST, f, popup_chunks[0]);
                    self.view
                        .render(super::COMPONENT_INPUT_SSH_USERNAME, f, popup_chunks[1]);
                }
            }
        });
        // Put context back to context
        self.context = Some(ctx);
    }

    // -- mount

    /// ### mount_del_ssh_key
    ///
    /// Mount delete ssh key component
    pub(crate) fn mount_del_ssh_key(&mut self) {
        self.view.mount(
            super::COMPONENT_RADIO_DEL_SSH_KEY,
            Box::new(Radio::new(
                RadioPropsBuilder::default()
                    .with_color(Color::LightRed)
                    .with_inverted_color(Color::Black)
                    .with_borders(Borders::ALL, BorderType::Rounded, Color::LightRed)
                    .with_title("Delete key?", Alignment::Center)
                    .with_options(&[String::from("Yes"), String::from("No")])
                    .with_value(1) // Default: No
                    .rewind(true)
                    .build(),
            )),
        );
        // Active
        self.view.active(super::COMPONENT_RADIO_DEL_SSH_KEY);
    }

    /// ### umount_del_ssh_key
    ///
    /// Umount delete ssh key
    pub(crate) fn umount_del_ssh_key(&mut self) {
        self.view.umount(super::COMPONENT_RADIO_DEL_SSH_KEY);
    }

    /// ### mount_new_ssh_key
    ///
    /// Mount new ssh key prompt
    pub(crate) fn mount_new_ssh_key(&mut self) {
        self.view.mount(
            super::COMPONENT_INPUT_SSH_HOST,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_label("Hostname or address", Alignment::Center)
                    .with_borders(
                        Borders::TOP | Borders::RIGHT | Borders::LEFT,
                        BorderType::Plain,
                        Color::Reset,
                    )
                    .build(),
            )),
        );
        self.view.mount(
            super::COMPONENT_INPUT_SSH_USERNAME,
            Box::new(Input::new(
                InputPropsBuilder::default()
                    .with_label("Username", Alignment::Center)
                    .with_borders(
                        Borders::BOTTOM | Borders::RIGHT | Borders::LEFT,
                        BorderType::Plain,
                        Color::Reset,
                    )
                    .build(),
            )),
        );
        self.view.active(super::COMPONENT_INPUT_SSH_HOST);
    }

    /// ### umount_new_ssh_key
    ///
    /// Umount new ssh key prompt
    pub(crate) fn umount_new_ssh_key(&mut self) {
        self.view.umount(super::COMPONENT_INPUT_SSH_HOST);
        self.view.umount(super::COMPONENT_INPUT_SSH_USERNAME);
    }

    /// ### reload_ssh_keys
    ///
    /// Reload ssh keys
    pub(crate) fn reload_ssh_keys(&mut self) {
        // get props
        if let Some(props) = self.view.get_props(super::COMPONENT_LIST_SSH_KEYS) {
            // Create texts
            let keys: Vec<String> = self
                .config()
                .iter_ssh_keys()
                .map(|x| {
                    let (addr, username, _) = self.config().get_ssh_key(x).ok().unwrap().unwrap();
                    format!("{} at {}", addr, username)
                })
                .collect();
            let props = BookmarkListPropsBuilder::from(props)
                .with_bookmarks(keys)
                .build();
            self.view.update(super::COMPONENT_LIST_SSH_KEYS, props);
        }
    }
}
