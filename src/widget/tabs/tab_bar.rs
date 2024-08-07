use std::borrow::Cow;
use std::sync::Arc;

use iced::{
    widget::{container, row, Container},
    Element, Length, Padding,
};

use crate::{theming::Theme, widget::tabs::Tab};

pub fn tab_bar<'a, Message: 'a + Clone>(
    labels: Vec<(Arc<String>, Message, Option<Message>)>,
    selected: Option<usize>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let tabs: Vec<Element<'_, Message>> = labels
        .into_iter()
        .enumerate()
        .map(|(id, (label, click, close))| {
            Tab::new(Cow::Owned(label.as_ref().clone()))
                .theme(theme)
                .on_click(click)
                .on_close_maybe(close)
                .selected(Some(id) == selected)
                .into()
        })
        .collect();

    Container::new(row(tabs).spacing(4.0))
        .width(Length::Fill)
        .height(Length::Fixed(36.0))
        .padding(Padding::new(4.0).bottom(0.0))
        .style(move |_| container::Style {
            background: Some(theme.tab_bar.background.into()),
            ..Default::default()
        })
        .into()
}
