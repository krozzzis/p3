#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;

use config::{
    workdir::{create_config_dir, create_workdir},
    Config,
};
use iced::{
    keyboard::{on_key_press, Key},
    widget::{
        row,
        text_editor::{self, Content},
        Container,
    },
    Element, Settings, Subscription, Task,
};
use state::State;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::util::{get_file_name, open_file, pick_file, save_file};

use plugin::{ExamplePlugin, Plugin, PluginHost, PluginId, PluginInfo};

use theming::{
    catalog::{get_themes, Catalog, ThemeID},
    metadata::ThemeMetadata,
    Theme,
};
use widget::pane::{self, pane_stack};

use core::{
    action::{Action, DocumentAction, FileAction, GenericAction, PaneAction},
    document::{DocumentHandler, DocumentId, DocumentStore},
    pane::{Pane, PaneModel},
    smol_str::SmolStr,
    value::Value,
    HotKey, Modifiers,
};

type HotKeyHandler = dyn Fn(&State) -> AppMessage;

static DEFAULT_THEME: &str = "core.light";

pub struct App {
    state: State,
    plugin_host: PluginHost,
    hotkeys: HashMap<HotKey, Box<HotKeyHandler>>,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    LoadPlugin(PluginId, bool),
    LoadTheme(ThemeID),
    AddTheme(ThemeID, Box<Theme>, ThemeMetadata<'static>),
    OpenedFile(Result<(PathBuf, String), ()>),
    GenericAction(GenericAction),
    Action(Action),
    SavedFile(DocumentId),
    OpenDirectory(PathBuf),
    TextEditorAction(text_editor::Action, DocumentId),
    OnKeyPress(Key, iced::keyboard::Modifiers),
    None,
}

impl App {
    fn new(config: Config) -> (Self, Task<AppMessage>) {
        let mut plugin_host = PluginHost::new();
        plugin_host.register_plugin(
            PluginInfo::new()
                .name("ExamplePlugin")
                .id("core.example")
                .author("krozzzis")
                .version("1.0")
                .description("An example plugin that do nothing useful)"),
            Box::new(ExamplePlugin {}) as Box<dyn Plugin>,
        );

        let mut panes = PaneModel::new();
        {
            let id = panes.add(Pane::NewDocument);
            panes.open(&id);
        }

        let state = State {
            documents: DocumentStore::new(),
            panes,
            themes: Catalog::new(),
            config,
        };

        let mut app = Self {
            state,
            plugin_host,
            hotkeys: HashMap::new(),
        };

        // Ctrl-o open file
        app.add_hotkey(
            HotKey {
                modifiers: Modifiers::Ctrl,
                key: 'o',
            },
            |_: &State| AppMessage::Action(Action::new(FileAction::PickFile)),
        );

        // Ctrl-d enable dark mode
        app.add_hotkey(
            HotKey {
                modifiers: Modifiers::Ctrl,
                key: 'd',
            },
            |_: &State| AppMessage::LoadTheme("core.dark".into()),
        );

        // Ctrl-t open new document tab
        app.add_hotkey(
            HotKey {
                modifiers: Modifiers::Ctrl,
                key: 't',
            },
            |_: &State| AppMessage::Action(Action::new(PaneAction::Add(Pane::NewDocument))),
        );

        // Ctrl-w close open tab
        app.add_hotkey(
            HotKey {
                modifiers: Modifiers::Ctrl,
                key: 'w',
            },
            |state: &State| {
                if let Some(id) = state.panes.get_open_id() {
                    AppMessage::Action(Action::new(PaneAction::Close(*id)))
                } else {
                    AppMessage::None
                }
            },
        );

        // Ctrl-b open experimental buffer pane
        app.add_hotkey(
            HotKey {
                modifiers: Modifiers::Ctrl,
                key: 'b',
            },
            |_state: &State| AppMessage::Action(Action::new(PaneAction::Add(Pane::Buffer))),
        );

        // Ctrl-, open config viewer pane
        app.add_hotkey(
            HotKey {
                modifiers: Modifiers::Ctrl,
                key: ',',
            },
            |_state: &State| AppMessage::Action(Action::new(PaneAction::Add(Pane::Config))),
        );

        let mut tasks = Vec::new();

        for id in app.plugin_host.get_plugin_ids() {
            let task = Task::done(AppMessage::LoadPlugin(id.clone(), true));
            tasks.push(task);
        }

        // Read themes from directory to stream
        let read_themes = Task::future(get_themes("./themes")).then(|stream| {
            // Add each theme from stream
            Task::run(stream, |(theme, metadata)| {
                AppMessage::AddTheme(metadata.id.to_string().into(), Box::new(theme), metadata)
            })
        });

        // Apply theme
        let theme = if let Some(Value::String(id)) = app.state.config.get("system", "theme") {
            id
        } else {
            SmolStr::new(DEFAULT_THEME)
        };
        let apply_theme = Task::perform(async move { theme }, AppMessage::LoadTheme);
        tasks.push(read_themes.chain(apply_theme));

        (app, Task::batch(tasks))
    }

    fn add_hotkey<F>(&mut self, hotkey: HotKey, func: F)
    where
        F: Fn(&State) -> AppMessage + 'static,
    {
        log::info!("Added hotkey {hotkey:?}");
        self.hotkeys.insert(hotkey, Box::new(func));
    }

    fn title(&self) -> String {
        String::from("Strelka")
    }

    fn perform_action(&mut self, action: GenericAction) -> Task<AppMessage> {
        match action {
            GenericAction::File(action) => match action {
                FileAction::PickFile => {
                    return Task::perform(pick_file(None), AppMessage::OpenedFile)
                }
                FileAction::OpenFileCurrentTab(path) => {
                    return Task::perform(open_file(path), AppMessage::OpenedFile)
                }
                FileAction::OpenFileForceCurrentTab(path) => {
                    return Task::perform(open_file(path), AppMessage::OpenedFile)
                }
                FileAction::OpenFileNewTab(path) => {
                    return Task::perform(open_file(path), AppMessage::OpenedFile)
                }
            },
            GenericAction::Pane(action) => match action {
                PaneAction::Close(id) => {
                    let pane = self.state.panes.remove(&id);

                    // Close document if Editor pane was closed
                    if let Some(Pane::Editor(doc_id)) = pane {
                        self.state.documents.remove(&doc_id);
                    }

                    // If there no panes left, create a NewDocument one
                    if self.state.panes.count() == 0 {
                        let id = self.state.panes.add(Pane::NewDocument);
                        self.state.panes.open(&id);
                    }
                }
                PaneAction::Open(id) => self.state.panes.open(&id),
                PaneAction::Add(pane) => {
                    let id = self.state.panes.add(pane);
                    self.state.panes.open(&id);
                }
                PaneAction::Replace(id, pane) => {
                    self.state.panes.replace(&id, pane);
                }
            },
            GenericAction::Document(action) => match action {
                DocumentAction::Add(handler) => {
                    let content = Content::with_text(&handler.text_content);
                    let handler = DocumentHandler {
                        text_content: content,
                        path: handler.path.clone(),
                        filename: handler.filename.clone(),
                        changed: handler.changed,
                    };
                    self.state.documents.add(handler);
                }
                DocumentAction::Open(id) => {
                    let pane = Pane::Editor(id);
                    return Task::done(AppMessage::Action(Action::new(PaneAction::Add(pane))));
                }
                DocumentAction::Save(id) => {
                    if let Some(handler) = self.state.documents.get(&id) {
                        let message = AppMessage::SavedFile(id);
                        return Task::perform(
                            save_file(handler.path.clone(), Arc::new(handler.text_content.text())),
                            move |_| message.clone(),
                        );
                    }
                }
                DocumentAction::Remove(id) => {
                    self.state.documents.remove(&id);
                }
            },
        }
        Task::none()
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        log::info!("Handling message: {message:?}");

        match message {
            AppMessage::None => {}

            AppMessage::Action(action) => {
                let action = self.plugin_host.process_action(&self.state, action);
                let mut tasks = Vec::new();
                for generic in action.iter() {
                    tasks.push(self.perform_action(generic.clone()));
                }

                return Task::batch(tasks);
            }

            AppMessage::GenericAction(action) => return self.perform_action(action),

            AppMessage::AddTheme(id, theme, metadata) => {
                self.state.themes.insert(id, *theme, metadata);
            }

            AppMessage::LoadTheme(id) => self.state.set_theme(id),

            AppMessage::SavedFile(id) => {
                if let Some(handler) = self.state.documents.get_mut(&id) {
                    handler.changed = false;
                }
            }

            AppMessage::OnKeyPress(key, modifiers) => {
                if let Some(message) = self.on_key_press(key, modifiers) {
                    return Task::done(message);
                }
            }

            AppMessage::LoadPlugin(id, load) => {
                if load {
                    self.plugin_host.load_plugin(&id);
                } else {
                    self.plugin_host.unload_plugin(&id);
                }
            }

            AppMessage::TextEditorAction(action, document) => {
                if let Some(handler) = self.state.documents.get_mut(&document) {
                    if action.is_edit() {
                        handler.changed = true;
                    }
                    handler.text_content.perform(action);
                }
            }

            // TODO: Should accept an document id and fill it's handler with content
            AppMessage::OpenedFile(result) => {
                if let Ok((path, content)) = result {
                    let handler = DocumentHandler {
                        text_content: Content::with_text(&content),
                        path: path.clone(),
                        filename: get_file_name(&path),
                        changed: false,
                    };

                    let doc_id = self.state.documents.add(handler);
                    let pane = Pane::Editor(doc_id);

                    // If opened pane is NewDocument, replace it with Editor pane
                    // otherwise add new one with Editor
                    if let Some(&Pane::NewDocument) = self.state.panes.get_open() {
                        self.state.panes.replace(
                            &self.state.panes.get_open_id().cloned().unwrap_or(0usize),
                            pane,
                        );
                    } else {
                        let pane_id = self.state.panes.add(pane);
                        self.state.panes.open(&pane_id);
                    }
                }
            }

            AppMessage::OpenDirectory(path) => {
                if path.is_dir() {
                    let path: PathBuf = path.canonicalize().unwrap_or_default();

                    self.state.config.insert(
                        "system",
                        "workdir",
                        Value::String(SmolStr::new(path.to_str().unwrap_or_default())),
                    );
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<AppMessage, Theme> {
        let mut grid_elements = Vec::new();
        grid_elements.push(
            pane_stack::pane_stack(&self.state).map(|msg| -> AppMessage {
                match msg {
                    pane_stack::Message::NewDocument(pane::new_document::Message::PickFile) => {
                        AppMessage::Action(Action::new(FileAction::PickFile))
                    }

                    pane_stack::Message::NewPane(pane) => {
                        AppMessage::Action(Action::new(PaneAction::Add(pane)))
                    }

                    pane_stack::Message::OpenPane(id) => {
                        AppMessage::Action(Action::new(PaneAction::Open(id)))
                    }

                    pane_stack::Message::ClosePane(id) => {
                        AppMessage::Action(Action::new(PaneAction::Close(id)))
                    }

                    pane_stack::Message::TextEditor(
                        id,
                        pane::text_editor::Message::EditorAction(action),
                    ) => AppMessage::TextEditorAction(action, id),

                    pane_stack::Message::None => AppMessage::None,
                }
            }),
        );
        let grid = row(grid_elements);

        let primary_screen = Container::new(grid);

        primary_screen.into()
    }

    fn theme(&self) -> Theme {
        self.state.get_theme()
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        on_key_press(|key, modifiers| Some(AppMessage::OnKeyPress(key, modifiers)))
    }

    fn on_key_press(
        &mut self,
        key: Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<AppMessage> {
        if let Key::Character(c) = key {
            let modifier = if modifiers.control() && modifiers.alt() {
                Modifiers::CtrlAlt
            } else if modifiers.control() {
                Modifiers::Ctrl
            } else if modifiers.alt() {
                Modifiers::Alt
            } else {
                Modifiers::None
            };

            let hotkey = HotKey {
                key: c.chars().next().unwrap_or_default(),
                modifiers: modifier,
            };

            if let Some(func) = self.hotkeys.get(&hotkey) {
                return Some(func(&self.state));
            }
        }
        None
    }
}

fn main() -> iced::Result {
    env_logger::init();

    let mut config = Config::new();

    // Initializing workdir. Default is ~/strelka
    let workdir_path = if let Ok(path) = create_workdir() {
        path
    } else {
        panic!("Can't create workdir")
    };

    config.insert(
        "system",
        "workdir",
        Value::String(SmolStr::new(workdir_path.to_str().unwrap())),
    );

    // Initializing config directory. Default is ~/strelka/.config
    let config_dir_path = if let Ok(path) = create_config_dir(&workdir_path) {
        path
    } else {
        panic!("Can't create config directory")
    };

    config.insert(
        "system",
        "config_dir",
        Value::String(SmolStr::new(config_dir_path.to_str().unwrap())),
    );

    // Path to system config file
    let system_config_path = {
        let mut a = config_dir_path.clone();
        a.push("system.toml");
        a
    };

    // Default config which used when config from file doesn't loaded
    let mut default_config = Config::new();
    default_config.insert(
        "system",
        "theme",
        Value::String(SmolStr::new(DEFAULT_THEME)),
    );

    // Loading system config from file or initializing it with default one
    let system_config =
        Config::load_or_create_default(&system_config_path, default_config).unwrap();
    config.merge(system_config);

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .settings(Settings {
            antialiasing: true,
            ..Settings::default()
        })
        .centered()
        .run_with(move || App::new(config))
}
