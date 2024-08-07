use std::path::{Path, PathBuf};

use iced::{border::Radius, Border, Color, Element, Length, Shadow, Vector};
use iced::{
    widget::{component, container, stack, Component, Container, Space},
    Size,
};
use iced_aw::widgets::ContextMenu;

use crate::{
    theming::{self, Theme},
    widget::list::{list, ListItem},
};

pub struct FileExplorer<'a, Message> {
    pub files: Vec<&'a PathBuf>,
    pub dirs: Vec<&'a PathBuf>,
    pub selected_file: Option<PathBuf>,
    pub on_click: Option<Box<dyn Fn(PathBuf) -> Message>>,
    pub theme: Option<&'a Theme<'a>>,
}

impl<'a, Message> FileExplorer<'a, Message> {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            dirs: Vec::new(),
            selected_file: None,
            on_click: None,
            theme: None,
        }
    }

    pub fn with_content(content: &'a [PathBuf]) -> Self {
        let files = content.iter().filter(|x| x.is_file()).collect();
        let dirs = content.iter().filter(|x| x.is_dir()).collect();
        Self {
            files,
            dirs,
            ..Self::new()
        }
    }

    pub fn with_content_maybe(content: Option<&'a [PathBuf]>) -> Self {
        if let Some(content) = content {
            Self::with_content(content)
        } else {
            Self::new()
        }
    }

    pub fn file_click<F>(mut self, func: F) -> Self
    where
        F: 'static + Fn(PathBuf) -> Message,
    {
        self.on_click = Some(Box::new(func));
        self
    }

    pub fn select_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.selected_file = Some(path.into());
        self
    }

    pub fn select_file_maybe(mut self, path: Option<impl Into<PathBuf>>) -> Self {
        if let Some(path) = path {
            self.selected_file = Some(path.into());
        }
        self
    }

    pub fn theme(mut self, theme: &'a Theme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<'a, Message> Default for FileExplorer<'a, Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Msg> Component<Msg> for FileExplorer<'a, Msg> {
    type State = ();

    type Event = Message;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Msg> {
        match event {
            Message::OpenDir(_path) => {}

            Message::OpenFile(path) => {
                if let Some(func) = &self.on_click {
                    return Some(func(path));
                }
            }

            Message::NewFile => {}
        }
        None
    }

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event> {
        let dirs = self.dirs.iter().map(|path| {
            ListItem::new(get_directory_name(path).unwrap_or(String::from("NaN")))
                .click(Message::OpenDir((*path).clone()))
                .theme(self.theme)
        });

        let files = self.files.iter().map(|path| {
            ListItem::new(get_file_name(path).unwrap_or(String::from("NaN")))
                .click(Message::OpenFile((*path).clone()))
                .selected(self.selected_file == Some((*path).clone()))
                .theme(self.theme)
        });

        let theme = self.theme.unwrap_or(&theming::FALLBACK);

        let items = container(list(
            dirs.chain(files)
                .map(|x| x.into())
                .collect::<Vec<Element<_>>>(),
            theme,
        ))
        .padding(theme.file_explorer.padding);

        let underlay = Container::new(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                text_color: Some(theme.file_explorer.text.into()),
                background: Some(theme.file_explorer.background.into()),
                ..Default::default()
            });

        let menu = ContextMenu::new(underlay, move || {
            container(list(
                vec![ListItem::new("New file")
                    .theme(self.theme)
                    .click(Message::NewFile)
                    .into()],
                theme,
            ))
            .padding(theme.context_menu.padding + theme.context_menu.border_width)
            .width(Length::Fixed(theme.context_menu.width))
            .style(move |_| container::Style {
                background: Some(theme.context_menu.background.into()),
                border: Border {
                    color: theme.context_menu.border_color.into(),
                    width: theme.context_menu.border_width,
                    radius: Radius::new(theme.context_menu.radius),
                },
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::new(theme.context_menu.shadow_x, theme.context_menu.shadow_y),
                    blur_radius: theme.context_menu.shadow_blur,
                },
                ..Default::default()
            })
            .into()
        });

        stack![menu, items].into()
    }

    fn size_hint(&self) -> iced::Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
}

impl<'a, Message> From<FileExplorer<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(widget: FileExplorer<'a, Message>) -> Self {
        component(widget)
    }
}

#[derive(Clone)]
pub enum Message {
    OpenFile(PathBuf),
    OpenDir(PathBuf),
    NewFile,
}

fn get_directory_name(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|os_str| os_str.to_str())
        .map(String::from)
}

fn get_file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(String::from)
}
