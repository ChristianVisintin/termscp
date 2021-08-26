//! ## Bookmark list
//!
//! `BookmarkList` component renders a bookmark list tab

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
use tui_realm_stdlib::utils::get_block;
use tuirealm::event::{Event, KeyCode};
use tuirealm::props::{Alignment, BlockTitle, BordersProps, Props, PropsBuilder};
use tuirealm::tui::{
    layout::{Corner, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{BorderType, Borders, List, ListItem, ListState},
};
use tuirealm::{Component, Frame, Msg, Payload, PropPayload, PropValue, Value};

// -- props
const PROP_BOOKMARKS: &str = "bookmarks";

pub struct BookmarkListPropsBuilder {
    props: Option<Props>,
}

impl Default for BookmarkListPropsBuilder {
    fn default() -> Self {
        BookmarkListPropsBuilder {
            props: Some(Props::default()),
        }
    }
}

impl PropsBuilder for BookmarkListPropsBuilder {
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

impl From<Props> for BookmarkListPropsBuilder {
    fn from(props: Props) -> Self {
        BookmarkListPropsBuilder { props: Some(props) }
    }
}

impl BookmarkListPropsBuilder {
    /// ### with_foreground
    ///
    /// Set foreground color for area
    pub fn with_foreground(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.foreground = color;
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

    pub fn with_title<S: AsRef<str>>(&mut self, text: S, alignment: Alignment) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.title = Some(BlockTitle::new(text, alignment));
        }
        self
    }

    pub fn with_bookmarks(&mut self, bookmarks: Vec<String>) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            let bookmarks: Vec<PropValue> = bookmarks.into_iter().map(PropValue::Str).collect();
            props
                .own
                .insert(PROP_BOOKMARKS, PropPayload::Vec(bookmarks));
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
    /// Reset list index to 0
    pub fn reset_list_index(&mut self) {
        self.list_index = 0;
    }
}

// -- Component

/// ## BookmarkList
///
/// Bookmark list component
pub struct BookmarkList {
    props: Props,
    states: OwnStates,
}

impl BookmarkList {
    /// ### new
    ///
    /// Instantiates a new FileList starting from Props
    /// The method also initializes the component states.
    pub fn new(props: Props) -> Self {
        // Initialize states
        let mut states: OwnStates = OwnStates::default();
        // Set list length
        states.set_list_len(Self::bookmarks_len(&props));
        BookmarkList { props, states }
    }

    fn bookmarks_len(props: &Props) -> usize {
        match props.own.get(PROP_BOOKMARKS) {
            None => 0,
            Some(bookmarks) => bookmarks.unwrap_vec().len(),
        }
    }
}

impl Component for BookmarkList {
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Frame, area: Rect) {
        if self.props.visible {
            // Make list
            let list_item: Vec<ListItem> = match self.props.own.get(PROP_BOOKMARKS) {
                Some(PropPayload::Vec(lines)) => lines
                    .iter()
                    .map(|x| x.unwrap_str())
                    .map(|x| ListItem::new(Span::from(x.to_string())))
                    .collect(),
                _ => vec![],
            };
            let (fg, bg): (Color, Color) = match self.states.focus {
                true => (self.props.foreground, self.props.background),
                false => (Color::Reset, Color::Reset),
            };
            // Render
            let mut state: ListState = ListState::default();
            state.select(Some(self.states.list_index));
            render.render_stateful_widget(
                List::new(list_item)
                    .block(get_block(
                        &self.props.borders,
                        self.props.title.as_ref(),
                        self.states.focus,
                    ))
                    .start_corner(Corner::TopLeft)
                    .highlight_style(
                        Style::default()
                            .bg(bg)
                            .fg(fg)
                            .add_modifier(self.props.modifiers),
                    ),
                area,
                &mut state,
            );
        }
    }

    fn update(&mut self, props: Props) -> Msg {
        self.props = props;
        // re-Set list length
        self.states.set_list_len(Self::bookmarks_len(&self.props));
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
                KeyCode::Down => {
                    // Update states
                    self.states.incr_list_index();
                    Msg::None
                }
                KeyCode::Up => {
                    // Update states
                    self.states.decr_list_index();
                    Msg::None
                }
                KeyCode::PageDown => {
                    // Update states
                    for _ in 0..8 {
                        self.states.incr_list_index();
                    }
                    Msg::None
                }
                KeyCode::PageUp => {
                    // Update states
                    for _ in 0..8 {
                        self.states.decr_list_index();
                    }
                    Msg::None
                }
                KeyCode::Enter => {
                    // Report event
                    Msg::OnSubmit(self.get_state())
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
    use tuirealm::event::KeyEvent;

    #[test]
    fn test_ui_components_bookmarks_list() {
        // Make component
        let mut component: BookmarkList = BookmarkList::new(
            BookmarkListPropsBuilder::default()
                .hidden()
                .visible()
                .with_foreground(Color::Red)
                .with_background(Color::Blue)
                .with_borders(Borders::ALL, BorderType::Double, Color::Red)
                .with_title("filelist", Alignment::Left)
                .with_bookmarks(vec![String::from("file1"), String::from("file2")])
                .build(),
        );
        assert_eq!(component.props.foreground, Color::Red);
        assert_eq!(component.props.background, Color::Blue);
        assert_eq!(component.props.visible, true);
        assert_eq!(component.props.title.as_ref().unwrap().text(), "filelist");
        assert_eq!(
            component
                .props
                .own
                .get(PROP_BOOKMARKS)
                .unwrap()
                .unwrap_vec()
                .len(),
            2
        );
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
        let props = BookmarkListPropsBuilder::from(component.get_props())
            .with_foreground(Color::Yellow)
            .hidden()
            .build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.props.foreground, Color::Yellow);
        assert_eq!(component.props.visible, false);
        // Increment list index
        component.states.list_index += 1;
        assert_eq!(component.states.list_index, 1);
        // Update
        component.update(
            BookmarkListPropsBuilder::from(component.get_props())
                .with_bookmarks(vec![
                    String::from("file1"),
                    String::from("file2"),
                    String::from("file3"),
                ])
                .build(),
        );
        // Verify states
        assert_eq!(component.states.list_index, 0);
        assert_eq!(component.states.list_len, 3);
        // get value
        assert_eq!(component.get_state(), Payload::One(Value::Usize(0)));
        // Render
        assert_eq!(component.states.list_index, 0);
        // Handle inputs
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Down))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 1);
        // Index should be decremented
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Up))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 0);
        // Index should be 2
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageDown))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be 0
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::PageUp))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 0);
        // Enter
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Enter))),
            Msg::OnSubmit(Payload::One(Value::Usize(0)))
        );
        // On key
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Backspace))),
            Msg::OnKey(KeyEvent::from(KeyCode::Backspace))
        );
    }
}
