//! ## ProgressBar
//!
//! `ProgressBar` component renders a progress bar

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
use super::{Canvas, Component, InputEvent, Msg, Payload, PropValue, Props, PropsBuilder};
// ext
use tui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Gauge},
};

// -- component

pub struct ProgressBar {
    props: Props,
}

impl ProgressBar {
    /// ### new
    ///
    /// Instantiate a new Progress Bar
    pub fn new(props: Props) -> Self {
        ProgressBar { props }
    }
}

impl Component for ProgressBar {
    /// ### render
    ///
    /// Based on the current properties and states, renders a widget using the provided render engine in the provided Area
    /// If focused, cursor is also set (if supported by widget)
    #[cfg(not(tarpaulin_include))]
    fn render(&self, render: &mut Canvas, area: Rect) {
        // Make a Span
        if self.props.visible {
            let title: String = match self.props.texts.title.as_ref() {
                Some(t) => t.clone(),
                None => String::new(),
            };
            // Text
            let label: String = match self.props.texts.rows.as_ref() {
                Some(rows) => match rows.get(0) {
                    Some(label) => label.content.clone(),
                    None => String::new(),
                },
                None => String::new(),
            };
            // Get percentage
            let percentage: f64 = match self.props.value {
                PropValue::Float(ratio) => ratio,
                _ => 0.0,
            };
            // Make progress bar
            render.render_widget(
                Gauge::default()
                    .block(Block::default().borders(self.props.borders).title(title))
                    .gauge_style(
                        Style::default()
                            .fg(self.props.foreground)
                            .bg(self.props.background)
                            .add_modifier(self.props.get_modifiers()),
                    )
                    .label(label)
                    .ratio(percentage),
                area,
            );
        }
    }

    /// ### update
    ///
    /// Update component properties
    /// Properties should first be retrieved through `get_props` which creates a builder from
    /// existing properties and then edited before calling update.
    /// Returns a Msg to the view
    fn update(&mut self, props: Props) -> Msg {
        self.props = props;
        // Return None
        Msg::None
    }

    /// ### get_props
    ///
    /// Returns a props builder starting from component properties.
    /// This returns a prop builder in order to make easier to create
    /// new properties for the element.
    fn get_props(&self) -> PropsBuilder {
        PropsBuilder::from(self.props.clone())
    }

    /// ### on
    ///
    /// Handle input event and update internal states.
    /// Returns a Msg to the view.
    /// Returns always None, since cannot have any focus
    fn on(&mut self, ev: InputEvent) -> Msg {
        // Return key
        if let InputEvent::Key(key) = ev {
            Msg::OnKey(key)
        } else {
            Msg::None
        }
    }

    /// ### get_value
    ///
    /// Get current value from component
    /// For this component returns always None
    fn get_value(&self) -> Payload {
        Payload::None
    }

    // -- events

    /// ### blur
    ///
    /// Blur component
    fn blur(&mut self) {}

    /// ### active
    ///
    /// Active component
    fn active(&mut self) {}
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ui::layout::props::{TextParts, TextSpan};

    use crossterm::event::{KeyCode, KeyEvent};
    use tui::style::Color;

    #[test]
    fn test_ui_layout_components_progress_bar() {
        let mut component: ProgressBar = ProgressBar::new(
            PropsBuilder::default()
                .with_texts(TextParts::new(
                    Some(String::from("Uploading file...")),
                    Some(vec![TextSpan::from("96.5% - ETA 00:02 (4.2MB/s)")]),
                ))
                .build(),
        );
        // Get value
        assert_eq!(component.get_value(), Payload::None);
        component.active();
        component.blur();
        // Update
        let props = component.get_props().with_foreground(Color::Red).build();
        assert_eq!(component.update(props), Msg::None);
        assert_eq!(component.props.foreground, Color::Red);
        // Event
        assert_eq!(
            component.on(InputEvent::Key(KeyEvent::from(KeyCode::Delete))),
            Msg::OnKey(KeyEvent::from(KeyCode::Delete))
        );
    }
}