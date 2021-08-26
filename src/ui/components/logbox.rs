//! ## LogBox
//!
//! `LogBox` component renders a log box view

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
// ext
use tui_realm_stdlib::utils::{get_block, wrap_spans};
use tuirealm::event::{Event, KeyCode};
use tuirealm::props::{
    Alignment, BlockTitle, BordersProps, Props, PropsBuilder, Table as TextTable,
};
use tuirealm::tui::{
    layout::{Corner, Rect},
    style::{Color, Style},
    widgets::{BorderType, Borders, List, ListItem, ListState},
};
use tuirealm::{Component, Frame, Msg, Payload, PropPayload, PropValue, Value};

// -- props

const PROP_TABLE: &str = "table";

pub struct LogboxPropsBuilder {
    props: Option<Props>,
}

impl Default for LogboxPropsBuilder {
    fn default() -> Self {
        LogboxPropsBuilder {
            props: Some(Props::default()),
        }
    }
}

impl PropsBuilder for LogboxPropsBuilder {
    fn build(&mut self) -> Props {
        self.props.take().unwrap()
    }

    fn hidden(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.visible = false;
        }
        self
    }

    fn visible(&mut self) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.visible = true;
        }
        self
    }
}

impl From<Props> for LogboxPropsBuilder {
    fn from(props: Props) -> Self {
        LogboxPropsBuilder { props: Some(props) }
    }
}

impl LogboxPropsBuilder {
    /// ### with_borders
    ///
    /// Set component borders style
    pub fn with_borders(
        &mut self,
        borders: Borders,
        variant: BorderType,
        color: Color,
    ) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.borders = BordersProps {
                borders,
                variant,
                color,
            }
        }
        self
    }

    /// ### with_background
    ///
    /// Set background color for area
    pub fn with_background(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.background = color;
        }
        self
    }

    pub fn with_title<S: AsRef<str>>(&mut self, text: S, alignment: Alignment) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.title = Some(BlockTitle::new(text, alignment));
        }
        self
    }

    pub fn with_log(&mut self, table: TextTable) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props
                .own
                .insert(PROP_TABLE, PropPayload::One(PropValue::Table(table)));
        }
        self
    }
}

// -- states

/// ## OwnStates
///
/// OwnStates contains states for this component
#[derive(Clone)]
struct OwnStates {
    list_index: usize, // Index of selected element in list
    list_len: usize,   // Length of file list
    focus: bool,       // Has focus?
}

impl Default for OwnStates {
    fn default() -> Self {
        OwnStates {
            list_index: 0,
            list_len: 0,
            focus: false,
        }
    }
}

impl OwnStates {
    /// ### set_list_len
    ///
    /// Set list length
    pub fn set_list_len(&mut self, len: usize) {
        self.list_len = len;
    }

    /// ### get_list_index
    ///
    /// Return current value for list index
    pub fn get_list_index(&self) -> usize {
        self.list_index
    }

    /// ### incr_list_index
    ///
    /// Incremenet list index
    pub fn incr_list_index(&mut self) {
        // Check if index is at last element
        if self.list_index + 1 < self.list_len {
            self.list_index += 1;
        }
    }

    /// ### decr_list_index
    ///
    /// Decrement list index
    pub fn decr_list_index(&mut self) {
        // Check if index is bigger than 0
        if self.list_index > 0 {
            self.list_index -= 1;
        }
    }

    /// ### reset_list_index
    ///
    /// Reset list index to last element
    pub fn reset_list_index(&mut self) {
        self.list_index = 0; // Last element is always 0
    }
}

// -- Component

/// ## LogBox
///
/// LogBox list component
pub struct LogBox {
    props: Props,
    states: OwnStates,
}

impl LogBox {
    /// ### new
    ///
    /// Instantiates a new FileList starting from Props
    /// The method also initializes the component states.
    pub fn new(props: Props) -> Self {
        // Initialize states
        let mut states: OwnStates = OwnStates::default();
        // Set list length
        states.set_list_len(Self::table_len(&props));
        // Reset list index
        states.reset_list_index();
        LogBox { props, states }
    }

    fn table_len(props: &Props) -> usize {
        match props.own.get(PROP_TABLE) {
            Some(PropPayload::One(PropValue::Table(table))) => table.len(),
            _ => 0,
        }
    }
}

impl Component for LogBox {
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Frame, area: Rect) {
        if self.props.visible {
            let width: usize = area.width as usize - 4;
            // Make list
            let list_items: Vec<ListItem> = match self.props.own.get(PROP_TABLE) {
                Some(PropPayload::One(PropValue::Table(table))) => table
                    .iter()
                    .map(|row| ListItem::new(wrap_spans(row, width, &self.props)))
                    .collect(), // Make List item from TextSpan
                _ => Vec::new(),
            };
            let w = List::new(list_items)
                .block(get_block(
                    &self.props.borders,
                    self.props.title.as_ref(),
                    self.states.focus,
                ))
                .start_corner(Corner::BottomLeft)
                .highlight_symbol(">> ")
                .style(Style::default().bg(self.props.background))
                .highlight_style(Style::default().add_modifier(self.props.modifiers));
            let mut state: ListState = ListState::default();
            state.select(Some(self.states.list_index));
            render.render_stateful_widget(w, area, &mut state);
        }
    }

    fn update(&mut self, props: Props) -> Msg {
        self.props = props;
        // re-Set list length
        self.states.set_list_len(Self::table_len(&self.props));
        // Reset list index
        self.states.reset_list_index();
        Msg::None
    }

    fn get_props(&self) -> Props {
        self.props.clone()
    }

    fn on(&mut self, ev: Event) -> Msg {
        // Match event
        if let Event::Key(key) = ev {
            match key.code {
                KeyCode::Up => {
                    // Update states
                    self.states.incr_list_index();
                    Msg::None
                }
                KeyCode::Down => {
                    // Update states
                    self.states.decr_list_index();
                    Msg::None
                }
                KeyCode::PageUp => {
                    // Update states
                    for _ in 0..8 {
                        self.states.incr_list_index();
                    }
                    Msg::None
                }
                KeyCode::PageDown => {
                    // Update states
                    for _ in 0..8 {
                        self.states.decr_list_index();
                    }
                    Msg::None
                }
                _ => {
                    // Return key event to activity
                    Msg::OnKey(key)
                }
            }
        } else {
            // Unhandled event
            Msg::None
        }
    }

    fn get_state(&self) -> Payload {
        Payload::One(Value::Usize(self.states.get_list_index()))
    }

    fn blur(&mut self) {
        self.states.focus = false;
    }

    fn active(&mut self) {
        self.states.focus = true;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;
    use tuirealm::event::{KeyCode, KeyEvent};
    use tuirealm::props::{TableBuilder, TextSpan};
    use tuirealm::tui::style::Color;

    #[test]
    fn test_ui_components_logbox() {
        let mut component: LogBox = LogBox::new(
            LogboxPropsBuilder::default()
                .hidden()
                .visible()
                .with_borders(Borders::ALL, BorderType::Double, Color::Red)
                .with_background(Color::Blue)
                .with_title("Log", Alignment::Left)
                .with_log(
                    TableBuilder::default()
                        .add_col(TextSpan::from("12:29"))
                        .add_col(TextSpan::from("system crashed"))
                        .add_row()
                        .add_col(TextSpan::from("12:38"))
                        .add_col(TextSpan::from("system alive"))
                        .build(),
                )
                .build(),
        );
        assert_eq!(component.props.visible, true);
        assert_eq!(component.props.background, Color::Blue);
        assert_eq!(component.props.title.as_ref().unwrap().text(), "Log");
        // Verify states
        assert_eq!(component.states.list_index, 0);
        assert_eq!(component.states.list_len, 2);
        assert_eq!(component.states.focus, false);
        // Focus
        component.active();
        assert_eq!(component.states.focus, true);
        component.blur();
        assert_eq!(component.states.focus, false);
        // Update
        let props = LogboxPropsBuilder::from(component.get_props())
            .hidden()
            .build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.props.visible, false);
        // Increment list index
        component.states.list_index += 1;
        assert_eq!(component.states.list_index, 1);
        // Update
        component.update(
            LogboxPropsBuilder::from(component.get_props())
                .with_log(
                    TableBuilder::default()
                        .add_col(TextSpan::from("12:29"))
                        .add_col(TextSpan::from("system crashed"))
                        .add_row()
                        .add_col(TextSpan::from("12:38"))
                        .add_col(TextSpan::from("system alive"))
                        .add_row()
                        .add_col(TextSpan::from("12:41"))
                        .add_col(TextSpan::from("system is going down for REBOOT"))
                        .build(),
                )
                .build(),
        );
        // Verify states
        assert_eq!(component.states.list_index, 0); // Last item
        assert_eq!(component.states.list_len, 3);
        // get value
        assert_eq!(component.get_state(), Payload::One(Value::Usize(0)));
        // RenderData
        assert_eq!(component.states.list_index, 0);
        // Set cursor to 0
        component.states.list_index = 0;
        // Handle inputs
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Up))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 1);
        // Index should be decremented
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Down))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 0);
        // Index should be 2
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageUp))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be 0
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageDown))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 0);
        // On key
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Backspace))),
            Msg::OnKey(KeyEvent::from(KeyCode::Backspace))
        );
    }
}
