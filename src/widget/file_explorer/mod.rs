use std::path::{Path, PathBuf};

use iced::{
    border::Radius,
    widget::{column, component, container, stack, Component, Container, Space},
    Border, Color, Element, Length, Renderer, Shadow, Theme, Vector,
};
use iced_aw::widgets::ContextMenu;

use crate::widget::list::{list, ListItem};

pub struct FileExplorer<'a, Message> {
    pub files: Vec<&'a PathBuf>,
    pub dirs: Vec<&'a PathBuf>,
    pub opened_file: Option<&'a Path>,
    pub on_click: Option<Box<dyn Fn(PathBuf) -> Message>>,
}

impl<'a, Message> FileExplorer<'a, Message> {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            dirs: Vec::new(),
            opened_file: None,
            on_click: None,
        }
    }

    pub fn with_content(content: &'a [PathBuf]) -> Self {
        let files = content.iter().filter(|x| x.is_file()).collect();
        let dirs = content.iter().filter(|x| x.is_dir()).collect();
        Self {
            files,
            dirs,
            opened_file: None,
            on_click: None,
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

    pub fn opened_file(mut self, path: &'a Path) -> Self {
        self.opened_file = Some(path);
        self
    }

    pub fn opened_file_maybe(mut self, path: Option<&'a Path>) -> Self {
        if let Some(path) = path {
            self.opened_file = Some(path);
        }
        self
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

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let dirs = self.dirs.iter().map(|path| {
            ListItem::new(get_directory_name(path).unwrap_or(String::from("NaN")))
                .click(Message::OpenDir((*path).clone()))
        });

        let files = self.files.iter().map(|path| {
            ListItem::new(get_file_name(path).unwrap_or(String::from("NaN")))
                .click(Message::OpenFile((*path).clone()))
        });

        let items = list(
            dirs.chain(files)
                .map(|x| x.into())
                .collect::<Vec<Element<_>>>(),
        );

        let underlay = Container::new(Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill);

        let menu = ContextMenu::new(underlay, || {
            container(list(vec![ListItem::new("New file")
                .click(Message::NewFile)
                .into()]))
            .padding(4.0)
            .width(Length::Fixed(200.0))
            .style(|_| container::Style {
                background: Some(Color::new(0.95, 0.95, 0.95, 1.0).into()),
                border: Border {
                    color: Color::new(0.7, 0.7, 0.7, 1.0),
                    width: 1.0,
                    radius: Radius::new(4.0),
                },
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::new(4.0, 4.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            })
            .into()
        });

        stack![menu, items].into()
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
