use iced::{widget::Button, Element};

use theming::Theme;

pub fn primary_button<'a, Message>(
    content: impl Into<Element<'a, Message, Theme>>,
) -> Button<'a, Message, Theme> {
    Button::new(content).style(theming::iced::button::primary)
}

pub fn secondary_button<'a, Message>(
    content: impl Into<Element<'a, Message, Theme>>,
) -> Button<'a, Message, Theme> {
    Button::new(content).style(theming::iced::button::secondary)
}

pub fn text_button<'a, Message>(
    content: impl Into<Element<'a, Message, Theme>>,
) -> Button<'a, Message, Theme> {
    Button::new(content).style(theming::iced::button::text)
}

pub mod a {
    use iced::{
        advanced::{
            graphics::geometry::{self, Frame},
            layout, mouse, renderer,
            widget::{tree, Operation, Tree},
            Clipboard, Layout, Shell, Widget,
        },
        event, touch,
        widget::canvas::{self, path::Builder, Fill, Stroke},
        Element, Event, Length, Point, Rectangle, Size, Vector,
    };
    use theming::{Border, Color, Font, Margin, Theme};

    #[derive(Debug, Clone)]
    pub struct Style {
        pub background: Color,
        pub margin: Margin,
        pub border: Border,
        pub font: Font,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct State {
        is_pressed: bool,
    }

    pub struct Button<'a, Message, Renderer> {
        content: Element<'a, Message, Theme, Renderer>,
        selected: bool,
        height: f32,
        min_width: f32,
        on_press: Option<Message>,
    }

    impl<'a, Message, Renderer> Button<'a, Message, Renderer> {
        pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
            Self {
                content: content.into(),
                selected: false,
                height: 40.0,
                min_width: 200.0,
                on_press: None,
            }
        }

        pub fn min_width(mut self, value: f32) -> Self {
            self.min_width = value;
            self
        }

        pub fn height(mut self, value: f32) -> Self {
            self.height = value;
            self
        }

        pub fn selected(mut self, selected: bool) -> Self {
            self.selected = selected;
            self
        }

        pub fn on_press(mut self, message: Message) -> Self {
            self.on_press = Some(message);
            self
        }

        pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
            self.on_press = message;
            self
        }
    }

    impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for Button<'a, Message, Renderer>
    where
        Message: Clone + 'a,
        Renderer: geometry::Renderer,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<State>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(State::default())
        }

        fn size(&self) -> Size<Length> {
            Size::new(Length::Shrink, Length::Shrink)
        }

        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(&self.content)]
        }

        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(std::slice::from_ref(&self.content));
        }

        fn layout(
            &self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::contained(limits, self.min_width, self.height, |limits| {
                self.content
                    .as_widget()
                    .layout(&mut tree.children[0], renderer, limits)
            })
        }

        fn operate(
            &self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation,
        ) {
            operation.container(None, layout.bounds(), &mut |operation| {
                self.content.as_widget().operate(
                    &mut tree.children[0],
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            });
        }

        fn on_event(
            &mut self,
            tree: &mut Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            if let event::Status::Captured = self.content.as_widget_mut().on_event(
                &mut tree.children[0],
                event.clone(),
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            ) {
                return event::Status::Captured;
            }

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        let state = tree.state.downcast_mut::<State>();

                        state.is_pressed = true;

                        return event::Status::Captured;
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. }) => {
                    let state = tree.state.downcast_mut::<State>();

                    if state.is_pressed {
                        state.is_pressed = false;

                        if let Some(on_press) = self.on_press.clone() {
                            if cursor.is_over(layout.bounds()) {
                                shell.publish(on_press);
                            }
                        }

                        return event::Status::Captured;
                    }
                }
                Event::Touch(touch::Event::FingerLost { .. }) => {
                    let state = tree.state.downcast_mut::<State>();

                    state.is_pressed = false;
                }
                _ => {}
            }

            event::Status::Ignored
        }

        fn mouse_interaction(
            &self,
            _tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            let is_mouse_over = cursor.is_over(layout.bounds());

            if is_mouse_over && self.on_press.is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style_: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let is_mouse_over = cursor.is_over(layout.bounds());

            let status = if is_mouse_over {
                let state = tree.state.downcast_ref::<State>();
                if state.is_pressed {
                    Status::Pressed
                } else {
                    Status::Hovered
                }
            } else {
                Status::Active
            };

            let style = if self.selected {
                Style {
                    background: theme.tab.selected.background,
                    border: theme.tab.selected.border.clone(),
                    margin: theme.tab.selected.margin,
                    font: theme.tab.selected.font.clone(),
                }
            } else {
                match status {
                    Status::Active => Style {
                        background: theme.tab.active.background,
                        border: theme.tab.active.border.clone(),
                        margin: theme.tab.active.margin,
                        font: theme.tab.active.font.clone(),
                    },
                    Status::Hovered => Style {
                        background: theme.tab.hover.background,
                        border: theme.tab.hover.border.clone(),
                        margin: theme.tab.hover.margin,
                        font: theme.tab.hover.font.clone(),
                    },
                    Status::Pressed => Style {
                        background: theme.tab.hover.background,
                        border: theme.tab.hover.border.clone(),
                        margin: theme.tab.hover.margin,
                        font: theme.tab.hover.font.clone(),
                    },
                }
            };

            let radius = style.border.radius.clone();
            let draw_bound = Size::new(
                layout.bounds().width
                    + style.border.width * 2.0
                    + if radius.bottom_left < 0.0 {
                        radius.bottom_left.abs()
                    } else {
                        0.0
                    }
                    + if radius.bottom_right < 0.0 {
                        radius.bottom_left.abs()
                    } else {
                        0.0
                    },
                layout.bounds().height + style.border.width * 2.0,
            );

            // Offset of inner content due to possible outer border radius
            let content_x = if radius.bottom_left < 0.0 {
                radius.bottom_left.abs()
            } else {
                0.0
            } + style.margin.left;
            let content_y = style.margin.top;

            // Size of content
            let content_width = self.min_width - style.margin.left - style.margin.right;
            let content_height = self.height - style.margin.top - style.margin.bottom;

            let mut frame = Frame::new(renderer, draw_bound);
            let mut builder = Builder::new();

            // Top line
            builder.move_to(Point::new(content_x + radius.top_left, content_y));
            builder.line_to(Point::new(
                content_x + content_width - radius.top_right,
                content_y,
            ));

            // Top right arc
            builder.arc_to(
                Point::new(content_x + content_width, content_y),
                Point::new(
                    content_x + content_width,
                    content_y + content_height - radius.bottom_right.abs(),
                ),
                radius.top_right.abs(),
            );

            // Bottom right arc
            builder.arc_to(
                Point::new(content_x + content_width, content_y + content_height),
                Point::new(
                    content_x + content_width - radius.bottom_right,
                    content_y + content_height,
                ),
                radius.bottom_right.abs(),
            );

            // Bottom line
            builder.line_to(Point::new(
                content_x + radius.bottom_left,
                content_y + content_height,
            ));

            // Bottom left arc
            builder.arc_to(
                Point::new(content_x, content_y + content_height),
                Point::new(
                    content_x,
                    content_y + content_height - radius.bottom_left.abs(),
                ),
                radius.bottom_left.abs(),
            );

            // Top left arc
            builder.arc_to(
                Point::new(content_x, content_y),
                Point::new(content_x + radius.top_left, content_y),
                radius.top_left.abs(),
            );

            let path = builder.build();

            // Background
            frame.fill(
                &path,
                Fill {
                    style: canvas::Style::Solid(style.background.into()),
                    ..Default::default()
                },
            );

            // Border
            frame.stroke(
                &path,
                Stroke {
                    style: canvas::Style::Solid(style.border.color.into()),
                    width: style.border.width,
                    ..Default::default()
                },
            );

            let geometry = frame.into_geometry();
            renderer.with_translation(
                Vector::new(layout.bounds().x - content_x, layout.bounds().y - content_y),
                |renderer| {
                    renderer.draw_geometry(geometry);
                },
            );

            let content_layout = layout.children().next().unwrap();
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &renderer::Style {
                    text_color: style_.text_color,
                },
                content_layout,
                cursor,
                viewport,
            );
        }
    }

    impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: geometry::Renderer + 'a,
    {
        fn from(button: Button<'a, Message, Renderer>) -> Self {
            Self::new(button)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Status {
        Active,
        Hovered,
        Pressed,
    }
}
