//! ## FileList
//!
//! `FileList` component renders a file list tab

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
use tuirealm::event::{Event, KeyCode, KeyModifiers};
use tuirealm::props::{
    Alignment, BlockTitle, BordersProps, PropPayload, PropValue, Props, PropsBuilder,
};
use tuirealm::tui::{
    layout::{Corner, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{BorderType, Borders, List, ListItem, ListState},
};
use tuirealm::{Component, Frame, Msg, Payload, Value};

// -- props

const PROP_FILES: &str = "files";
const PALETTE_HIGHLIGHT_COLOR: &str = "props-highlight-color";

pub struct FileListPropsBuilder {
    props: Option<Props>,
}

impl Default for FileListPropsBuilder {
    fn default() -> Self {
        FileListPropsBuilder {
            props: Some(Props::default()),
        }
    }
}

impl PropsBuilder for FileListPropsBuilder {
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

impl From<Props> for FileListPropsBuilder {
    fn from(props: Props) -> Self {
        FileListPropsBuilder { props: Some(props) }
    }
}

impl FileListPropsBuilder {
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

    /// ### with_highlight_color
    ///
    /// Set highlighted color
    pub fn with_highlight_color(&mut self, color: Color) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            props.palette.insert(PALETTE_HIGHLIGHT_COLOR, color);
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

    pub fn with_files(&mut self, files: Vec<String>) -> &mut Self {
        if let Some(props) = self.props.as_mut() {
            let files: Vec<PropValue> = files.into_iter().map(PropValue::Str).collect();
            props.own.insert(PROP_FILES, PropPayload::Vec(files));
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
    list_index: usize,    // Index of selected element in list
    selected: Vec<usize>, // Selected files
    focus: bool,          // Has focus?
}

impl Default for OwnStates {
    fn default() -> Self {
        OwnStates {
            list_index: 0,
            selected: Vec::new(),
            focus: false,
        }
    }
}

impl OwnStates {
    /// ### init_list_states
    ///
    /// Initialize list states
    pub fn init_list_states(&mut self, len: usize) {
        self.selected = Vec::with_capacity(len);
        self.fix_list_index();
    }

    /// ### list_index
    ///
    /// Return current value for list index
    pub fn list_index(&self) -> usize {
        self.list_index
    }

    /// ### incr_list_index
    ///
    /// Incremenet list index
    pub fn incr_list_index(&mut self) {
        // Check if index is at last element
        if self.list_index + 1 < self.list_len() {
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

    /// ### list_len
    ///
    /// Returns the length of the file list, which is actually the capacity of the selection vector
    pub fn list_len(&self) -> usize {
        self.selected.capacity()
    }

    /// ### is_selected
    ///
    /// Returns whether the file with index `entry` is selected
    pub fn is_selected(&self, entry: usize) -> bool {
        self.selected.contains(&entry)
    }

    /// ### is_selection_empty
    ///
    /// Returns whether the selection is currently empty
    pub fn is_selection_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// ### get_selection
    ///
    /// Returns current file selection
    pub fn get_selection(&self) -> Vec<usize> {
        self.selected.clone()
    }

    /// ### fix_list_index
    ///
    /// Keep index if possible, otherwise set to lenght - 1
    fn fix_list_index(&mut self) {
        if self.list_index >= self.list_len() && self.list_len() > 0 {
            self.list_index = self.list_len() - 1;
        } else if self.list_len() == 0 {
            self.list_index = 0;
        }
    }

    // -- select manipulation

    /// ### toggle_file
    ///
    /// Select or deselect file with provided entry index
    pub fn toggle_file(&mut self, entry: usize) {
        match self.is_selected(entry) {
            true => self.deselect(entry),
            false => self.select(entry),
        }
    }

    /// ### select_all
    ///
    /// Select all files
    pub fn select_all(&mut self) {
        for i in 0..self.list_len() {
            self.select(i);
        }
    }

    /// ### select
    ///
    /// Select provided index if not selected yet
    fn select(&mut self, entry: usize) {
        if !self.is_selected(entry) {
            self.selected.push(entry);
        }
    }

    /// ### deselect
    ///
    /// Remove element file with associated index
    fn deselect(&mut self, entry: usize) {
        if self.is_selected(entry) {
            self.selected.retain(|&x| x != entry);
        }
    }
}

// -- Component

/// ## FileList
///
/// File list component
pub struct FileList {
    props: Props,
    states: OwnStates,
}

impl FileList {
    /// ### new
    ///
    /// Instantiates a new FileList starting from Props
    /// The method also initializes the component states.
    pub fn new(props: Props) -> Self {
        // Initialize states
        let mut states: OwnStates = OwnStates::default();
        // Init list states
        states.init_list_states(Self::files_len(&props));
        FileList { props, states }
    }

    fn files_len(props: &Props) -> usize {
        match props.own.get(PROP_FILES) {
            None => 0,
            Some(files) => files.unwrap_vec().len(),
        }
    }
}

impl Component for FileList {
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Frame, area: Rect) {
        if self.props.visible {
            // Make list
            let list_item: Vec<ListItem> = match self.props.own.get(PROP_FILES) {
                Some(PropPayload::Vec(lines)) => lines
                    .iter()
                    .enumerate()
                    .map(|(num, line)| {
                        let to_display: String = match self.states.is_selected(num) {
                            true => format!("*{}", line.unwrap_str()),
                            false => line.unwrap_str().to_string(),
                        };
                        ListItem::new(Span::from(to_display))
                    })
                    .collect(),
                _ => vec![],
            };
            let highlighted_color: Color = match self.props.palette.get(PALETTE_HIGHLIGHT_COLOR) {
                Some(c) => *c,
                _ => Color::Reset,
            };
            let (h_fg, h_bg): (Color, Color) = match self.states.focus {
                true => (Color::Black, highlighted_color),
                false => (highlighted_color, self.props.background),
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
                    .style(
                        Style::default()
                            .fg(self.props.foreground)
                            .bg(self.props.background),
                    )
                    .highlight_style(
                        Style::default()
                            .bg(h_bg)
                            .fg(h_fg)
                            .add_modifier(self.props.modifiers),
                    ),
                area,
                &mut state,
            );
        }
    }

    fn update(&mut self, props: Props) -> Msg {
        self.props = props;
        // re-Set list states
        self.states.init_list_states(Self::files_len(&self.props));
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
                KeyCode::Char('a') => match key.modifiers.intersects(KeyModifiers::CONTROL) {
                    // CTRL+A
                    true => {
                        // Select all
                        self.states.select_all();
                        Msg::None
                    }
                    false => Msg::OnKey(key),
                },
                KeyCode::Char('m') => {
                    // Toggle current file in selection
                    self.states.toggle_file(self.states.list_index());
                    Msg::None
                }
                KeyCode::Enter => Msg::OnSubmit(self.get_state()),
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

    /// ### get_state
    ///
    /// Get state returns for this component two different payloads based on the states:
    /// - if the file selection is empty, returns the highlighted item as `One` of `Usize`
    /// - if at least one item is selected, return the selected as a `Vec` of `Usize`
    fn get_state(&self) -> Payload {
        match self.states.is_selection_empty() {
            true => Payload::One(Value::Usize(self.states.list_index())),
            false => Payload::Vec(
                self.states
                    .get_selection()
                    .into_iter()
                    .map(Value::Usize)
                    .collect(),
            ),
        }
    }

    // -- events

    /// ### blur
    ///
    /// Blur component; basically remove focus
    fn blur(&mut self) {
        self.states.focus = false;
    }

    /// ### active
    ///
    /// Active component; basically give focus
    fn active(&mut self) {
        self.states.focus = true;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use pretty_assertions::assert_eq;
    use tuirealm::event::{KeyEvent, KeyModifiers};

    #[test]
    fn test_ui_components_file_list_states() {
        let mut states: OwnStates = OwnStates::default();
        assert_eq!(states.list_len(), 0);
        assert_eq!(states.selected.len(), 0);
        assert_eq!(states.focus, false);
        // Init states
        states.init_list_states(4);
        assert_eq!(states.list_len(), 4);
        assert_eq!(states.selected.len(), 0);
        assert!(states.is_selection_empty());
        // Select all files
        states.select_all();
        assert_eq!(states.list_len(), 4);
        assert_eq!(states.selected.len(), 4);
        assert_eq!(states.is_selection_empty(), false);
        assert_eq!(states.get_selection(), vec![0, 1, 2, 3]);
        // Verify reset
        states.init_list_states(5);
        assert_eq!(states.list_len(), 5);
        assert_eq!(states.selected.len(), 0);
        // Toggle file
        states.toggle_file(2);
        assert_eq!(states.list_len(), 5);
        assert_eq!(states.selected.len(), 1);
        assert_eq!(states.selected[0], 2);
        states.toggle_file(4);
        assert_eq!(states.list_len(), 5);
        assert_eq!(states.selected.len(), 2);
        assert_eq!(states.selected[1], 4);
        states.toggle_file(2);
        assert_eq!(states.list_len(), 5);
        assert_eq!(states.selected.len(), 1);
        assert_eq!(states.selected[0], 4);
        // Select twice (nothing should change)
        states.select(4);
        assert_eq!(states.list_len(), 5);
        assert_eq!(states.selected.len(), 1);
        assert_eq!(states.selected[0], 4);
        // Deselect not-selectd item
        states.deselect(2);
        assert_eq!(states.list_len(), 5);
        assert_eq!(states.selected.len(), 1);
        assert_eq!(states.selected[0], 4);
        // Index
        states.init_list_states(2);
        states.incr_list_index();
        assert_eq!(states.list_index(), 1);
        states.incr_list_index();
        assert_eq!(states.list_index(), 1);
        states.decr_list_index();
        assert_eq!(states.list_index(), 0);
        states.decr_list_index();
        assert_eq!(states.list_index(), 0);
        // Try fixing index
        states.init_list_states(5);
        states.list_index = 4;
        states.init_list_states(3);
        assert_eq!(states.list_index(), 2);
        states.init_list_states(6);
        assert_eq!(states.list_index(), 2);
        // Focus
        states.focus = true;
        assert_eq!(states.focus, true);
    }

    #[test]
    fn test_ui_components_file_list() {
        // Make component
        let mut component: FileList = FileList::new(
            FileListPropsBuilder::default()
                .hidden()
                .visible()
                .with_foreground(Color::Red)
                .with_background(Color::Blue)
                .with_highlight_color(Color::LightRed)
                .with_borders(Borders::ALL, BorderType::Double, Color::Red)
                .with_title("files", Alignment::Left)
                .with_files(vec![String::from("file1"), String::from("file2")])
                .build(),
        );
        assert_eq!(
            *component
                .props
                .palette
                .get(PALETTE_HIGHLIGHT_COLOR)
                .unwrap(),
            Color::LightRed
        );
        assert_eq!(component.props.foreground, Color::Red);
        assert_eq!(component.props.background, Color::Blue);
        assert_eq!(component.props.visible, true);
        assert_eq!(component.props.title.as_ref().unwrap().text(), "files");
        assert_eq!(
            component
                .props
                .own
                .get(PROP_FILES)
                .as_ref()
                .unwrap()
                .unwrap_vec()
                .len(),
            2
        );
        // Verify states
        assert_eq!(component.states.list_index, 0);
        assert_eq!(component.states.selected.len(), 0);
        assert_eq!(component.states.list_len(), 2);
        assert_eq!(component.states.selected.capacity(), 2);
        assert_eq!(component.states.focus, false);
        // Focus
        component.active();
        assert_eq!(component.states.focus, true);
        component.blur();
        assert_eq!(component.states.focus, false);
        // Update
        let props = FileListPropsBuilder::from(component.get_props())
            .with_foreground(Color::Yellow)
            .hidden()
            .build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.props.visible, false);
        assert_eq!(component.props.foreground, Color::Yellow);
        // Increment list index
        component.states.list_index += 1;
        assert_eq!(component.states.list_index, 1);
        // Update
        component.update(
            FileListPropsBuilder::from(component.get_props())
                .with_files(vec![
                    String::from("file1"),
                    String::from("file2"),
                    String::from("file3"),
                ])
                .build(),
        );
        // Verify states
        assert_eq!(component.states.list_index, 1); // Kept
        assert_eq!(component.states.list_len(), 3);
        // get value
        assert_eq!(component.get_state(), Payload::One(Value::Usize(1)));
        // Render
        assert_eq!(component.states.list_index, 1);
        // Handle inputs
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Down))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 2);
        // Index should be decremented
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Up))),
            Msg::None
        );
        // Index should be incremented
        assert_eq!(component.states.list_index, 1);
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
        // Verify 'A' still works
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Char('a')))),
            Msg::OnKey(KeyEvent::from(KeyCode::Char('a')))
        );
        // Ctrl + a
        assert_eq!(
            component.on(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL
            ))),
            Msg::None
        );
        assert_eq!(component.states.selected.len(), component.states.list_len());
    }

    #[test]
    fn test_ui_components_file_list_selection() {
        // Make component
        let mut component: FileList = FileList::new(
            FileListPropsBuilder::default()
                .with_files(vec![
                    String::from("file1"),
                    String::from("file2"),
                    String::from("file3"),
                ])
                .build(),
        );
        // Get state
        assert_eq!(component.get_state(), Payload::One(Value::Usize(0)));
        // Select one
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Char('m')))),
            Msg::None
        );
        // Now should be a vec
        assert_eq!(component.get_state(), Payload::Vec(vec![Value::Usize(0)]));
        // De-select
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Char('m')))),
            Msg::None
        );
        assert_eq!(component.get_state(), Payload::One(Value::Usize(0)));
        // Go down
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Down))),
            Msg::None
        );
        // Select
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Char('m')))),
            Msg::None
        );
        assert_eq!(component.get_state(), Payload::Vec(vec![Value::Usize(1)]));
        // Go down and select
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Down))),
            Msg::None
        );
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Char('m')))),
            Msg::None
        );
        assert_eq!(
            component.get_state(),
            Payload::Vec(vec![Value::Usize(1), Value::Usize(2)])
        );
        // Select all
        assert_eq!(
            component.on(Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
            })),
            Msg::None
        );
        // All selected
        assert_eq!(
            component.get_state(),
            Payload::Vec(vec![Value::Usize(1), Value::Usize(2), Value::Usize(0)])
        );
        // Update files
        component.update(
            FileListPropsBuilder::from(component.get_props())
                .with_files(vec![String::from("file1"), String::from("file2")])
                .build(),
        );
        // Selection should now be empty
        assert_eq!(component.get_state(), Payload::One(Value::Usize(1)));
    }
}
