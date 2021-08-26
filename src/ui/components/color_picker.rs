//! ## ColorPicker
//!
//! `ColorPicker` component extends an `Input` component in order to provide some extra features
//! for the color picker.

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
// locals
use crate::utils::fmt::fmt_color;
use crate::utils::parser::parse_color;
// ext
use tui_realm_stdlib::input::{Input, InputPropsBuilder};
use tuirealm::event::Event;
use tuirealm::props::{Alignment, Props, PropsBuilder};
use tuirealm::tui::{
    layout::Rect,
    style::Color,
    widgets::{BorderType, Borders},
};
use tuirealm::{Component, Frame, Msg, Payload, Value};

// -- props

/// ## ColorPickerPropsBuilder
///
/// A wrapper around an `InputPropsBuilder`
pub struct ColorPickerPropsBuilder {
    puppet: InputPropsBuilder,
}

impl Default for ColorPickerPropsBuilder {
    fn default() -> Self {
        Self {
            puppet: InputPropsBuilder::default(),
        }
    }
}

impl PropsBuilder for ColorPickerPropsBuilder {
    fn build(&mut self) -> Props {
        self.puppet.build()
    }

    fn hidden(&mut self) -> &mut Self {
        self.puppet.hidden();
        self
    }

    fn visible(&mut self) -> &mut Self {
        self.puppet.visible();
        self
    }
}

impl From<Props> for ColorPickerPropsBuilder {
    fn from(props: Props) -> Self {
        ColorPickerPropsBuilder {
            puppet: InputPropsBuilder::from(props),
        }
    }
}

impl ColorPickerPropsBuilder {
    /// ### with_borders
    ///
    /// Set component borders style
    pub fn with_borders(
        &mut self,
        borders: Borders,
        variant: BorderType,
        color: Color,
    ) -> &mut Self {
        self.puppet.with_borders(borders, variant, color);
        self
    }

    /// ### with_label
    ///
    /// Set input label
    pub fn with_label<S: AsRef<str>>(&mut self, label: S, alignment: Alignment) -> &mut Self {
        self.puppet.with_label(label, alignment);
        self
    }

    /// ### with_color
    ///
    /// Set initial value for component
    pub fn with_color(&mut self, color: &Color) -> &mut Self {
        self.puppet.with_value(fmt_color(color));
        self
    }
}

// -- component

/// ## ColorPicker
///
/// a wrapper component of `Input` which adds a superset of rules to behave as a color picker
pub struct ColorPicker {
    input: Input,
}

impl ColorPicker {
    /// ### new
    ///
    /// Instantiate a new `ColorPicker`
    pub fn new(props: Props) -> Self {
        // Instantiate a new color picker using input
        Self {
            input: Input::new(props),
        }
    }

    /// ### update_colors
    ///
    /// Update colors to match selected color, with provided one
    fn update_colors(&mut self, color: Color) {
        let mut props = self.get_props();
        props.foreground = color;
        props.borders.color = color;
        let _ = self.input.update(props);
    }
}

impl Component for ColorPicker {
    /// ### render
    ///
    /// Based on the current properties and states, renders a widget using the provided render engine in the provided Area
    /// If focused, cursor is also set (if supported by widget)
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Frame, area: Rect) {
        self.input.render(render, area);
    }

    /// ### update
    ///
    /// Update component properties
    /// Properties should first be retrieved through `get_props` which creates a builder from
    /// existing properties and then edited before calling update.
    /// Returns a Msg to the view
    fn update(&mut self, props: Props) -> Msg {
        let msg: Msg = self.input.update(props);
        match msg {
            Msg::OnChange(Payload::One(Value::Str(input))) => match parse_color(input.as_str()) {
                Some(color) => {
                    // Update color and return OK
                    self.update_colors(color);
                    Msg::OnChange(Payload::One(Value::Str(input)))
                }
                None => {
                    // Invalid color
                    self.update_colors(Color::Red);
                    Msg::None
                }
            },
            msg => msg,
        }
    }

    /// ### get_props
    ///
    /// Returns a props builder starting from component properties.
    /// This returns a prop builder in order to make easier to create
    /// new properties for the element.
    fn get_props(&self) -> Props {
        self.input.get_props()
    }

    /// ### on
    ///
    /// Handle input event and update internal states.
    /// Returns a Msg to the view
    fn on(&mut self, ev: Event) -> Msg {
        // Capture message from input
        match self.input.on(ev) {
            Msg::OnChange(Payload::One(Value::Str(input))) => {
                // Capture color and validate
                match parse_color(input.as_str()) {
                    Some(color) => {
                        // Update color and return OK
                        self.update_colors(color);
                        Msg::OnChange(Payload::One(Value::Str(input)))
                    }
                    None => {
                        // Invalid color
                        self.update_colors(Color::Red);
                        Msg::None
                    }
                }
            }
            Msg::OnSubmit(_) => Msg::None,
            msg => msg,
        }
    }

    /// ### get_state
    ///
    /// Get current state from component
    /// For this component returns Unsigned if the input type is a number, otherwise a text
    /// The value is always the current input.
    fn get_state(&self) -> Payload {
        match self.input.get_state() {
            Payload::One(Value::Str(color)) => match parse_color(color.as_str()) {
                None => Payload::None,
                Some(_) => Payload::One(Value::Str(color)),
            },
            _ => Payload::None,
        }
    }

    // -- events

    /// ### blur
    ///
    /// Blur component; basically remove focus
    fn blur(&mut self) {
        self.input.blur();
    }

    /// ### active
    ///
    /// Active component; basically give focus
    fn active(&mut self) {
        self.input.active();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crossterm::event::{KeyCode, KeyEvent};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_ui_components_color_picker() {
        let mut component: ColorPicker = ColorPicker::new(
            ColorPickerPropsBuilder::default()
                .visible()
                .with_color(&Color::Rgb(204, 170, 0))
                .with_borders(Borders::ALL, BorderType::Double, Color::Rgb(204, 170, 0))
                .with_label("omar", Alignment::Left)
                .build(),
        );
        // Focus
        component.blur();
        component.active();
        // Get value
        assert_eq!(
            component.get_state(),
            Payload::One(Value::Str(String::from("#ccaa00")))
        );
        // Set an invalid color
        let props = InputPropsBuilder::from(component.get_props())
            .with_value(String::from("#pippo1"))
            .hidden()
            .build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.get_state(), Payload::None);
        // Reset color
        let props = ColorPickerPropsBuilder::from(component.get_props())
            .with_color(&Color::Rgb(204, 170, 0))
            .hidden()
            .build();
        assert_eq!(
            component.update(props),
            Msg::OnChange(Payload::One(Value::Str("#ccaa00".to_string())))
        );
        // Backspace (invalid)
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Backspace))),
            Msg::None
        );
        // Press '1'
        assert_eq!(
            component.on(Event::Key(KeyEvent::from(KeyCode::Char('1')))),
            Msg::OnChange(Payload::One(Value::Str(String::from("#ccaa01"))))
        );
    }
}
