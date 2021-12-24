// use iced::alignment::{self, Alignment};
use chrono::Local;
use iced::button::{self, Button};
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{
    Align, Application, Checkbox, Clipboard, Column, Command, Container, Element, Font, Length,
    Row, Settings, Subscription, Text, Image, Space , Radio,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;


pub fn main() -> iced::Result {
    //    Todos::run(Settings::default())
    Todos::run(Settings {
        default_font: Some(include_bytes!("../fonts/NotoSansJP-Regular.otf")),
        ..Settings::default()
    })
}

#[derive(Debug,Copy,Clone,Serialize, Deserialize, Eq, PartialEq)]
pub enum Importance {
    Low,
    Normal,
    High,
}

impl Importance {
    fn all() -> [Importance; 3] {
        [
            Importance::Low,
            Importance::Normal,
            Importance::High,
        ]
    }
}


impl From<Importance> for String {
    fn from(importance: Importance) -> String {
        String::from(match importance {
            Importance::Low => "Low",
            Importance::Normal => "Normal",
            Importance::High => "High",
        })
    }
}

#[derive(Debug)]
enum Todos {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default)]
struct State {
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
    controls: Controls,
    dirty: bool,
    saving: bool,
    file_path: PathBuf,
    datetime: String,
    filter_input_value: String,
    filter_input: text_input::State,
    selected_importance: Option<Importance>
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
    CreateTask,
    FilterChanged(Filter),
    TaskMessage(usize, TaskMessage),
    Dropped(iced_native::Event),
    FilterTextChanged(String),
    ImportanceChanged(Importance),
}

impl Application for Todos {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Todos, Command<Message>) {
        (
            Todos::Loading,
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        let dirty = match self {
            Todos::Loading => false,
            Todos::Loaded(state) => state.dirty,
        };

        format!("Todos{} - Iced", if dirty { "*" } else { "" })
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events().map(Message::Dropped)
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match self {
            Todos::Loading => {
                match message {
                    Message::Loaded(Ok(state)) => {
                        *self = Todos::Loaded(State {
                            input_value: state.input_value,
                            filter: state.filter,
                            tasks: state.tasks,
                            ..State::default()
                        });
                    }
                    Message::Loaded(Err(_)) => {
                        *self = Todos::Loaded(State::default());
                    }
                    _ => {}
                }

                Command::none()
            }
            Todos::Loaded(state) => {
                let mut saved = false;

                match message {


                    Message::CreateTask => {
                        if !state.input_value.is_empty() {
                            state.file_path = PathBuf::new();
                            state
                                .tasks
                                .push(Task::new(
                                    state.input_value.clone(),
                                    state.file_path.clone(),
                                    state.datetime.clone(),
                                    Importance::from(state.selected_importance.unwrap_or(Importance::Normal)),
                                    ));
                            state.input_value.clear();
                        }
                    }

                    Message::FilterTextChanged(value) => {
                        state.filter_input_value = value
                    }

                    Message::ImportanceChanged(importance) => {
                        state.selected_importance = Some(importance);
                    }

                    
                    Message::InputChanged(value) => {
                        state.input_value = value;
                    }

                    Message::FilterChanged(filter) => {
                        state.filter = filter;
                    }
                    Message::TaskMessage(i, TaskMessage::Delete) => {
                        state.tasks.remove(i);
                    }
                    Message::TaskMessage(i, task_message) => {
                        if let Some(task) = state.tasks.get_mut(i) {
                            task.update(task_message);
                        }
                    }
                    Message::Saved(_) => {
                        state.saving = false;
                        saved = true;
                    }
                    Message::Dropped(event) => {
                        if let iced_native::event::Event::Window(we) = event {
                            if let iced_native::window::Event::FileDropped(path) = we {
                                state.datetime = Local::now().format(" Added %Y/%m/%d %H:%M").to_string();
                                state.file_path = path;
                                state.tasks.push(Task::new(
                                    state.input_value.clone(),
                                    state.file_path.clone(),
                                        state.datetime.clone(),
                                        Importance::from(state.selected_importance.unwrap_or(Importance::Normal)),
                                ));
                                state.input_value.clear();
                            }
                        }
                    }
                    _ => {}
                }

                if !saved {
                    state.dirty = true;
                }

                if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;

                    Command::perform(
                        SavedState {
                            input_value: state.input_value.clone(),
                            filter: state.filter,
                            tasks: state.tasks.clone(),
                        }
                        .save(),
                        Message::Saved,
                    )
                } else {
                    Command::none()
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Todos::Loading => loading_message(),
            Todos::Loaded(State {
                scroll,
                input,
                input_value,
                filter,
                tasks,
                controls,
                filter_input_value,
                filter_input,
                selected_importance,
                ..
            }) => {

                let _title = Text::new("todos")
                    .width(Length::Fill)
                    .size(100)
                    .color([0.5, 0.5, 0.5])
                    .horizontal_alignment(iced::HorizontalAlignment::Center);

                let filter_textbox =  TextInput::new(
                    filter_input,
                           "",
                           filter_input_value,
                            Message::FilterTextChanged,
                        );


                let input = TextInput::new(
                    input,
                    "何をする？",
                    input_value,
                    Message::InputChanged,
                )
                .padding(15)
                .size(30)
                .on_submit(Message::CreateTask);

                let importance_selector = Column::new()
                .push(Importance::all().iter().cloned().fold(
                    Row::new().spacing(5),
                    |choices, importance| {
                        choices.push(Radio::new(
                            importance,
                            importance,
                            *selected_importance,
                            Message::ImportanceChanged,
                        ).text_size(20).size(20).spacing(5))
                    },
                ));

                let controls = controls.view(&tasks, *filter);
                let filtered_tasks = tasks.iter().filter(|task| filter.matches(task) & filter.word_matches(task, filter_input_value));


                let tasks: Element<_> = if filtered_tasks.count() > 0 {
                    tasks
                        .iter_mut()
                        .enumerate()
                        .filter(|(_, task)| filter.matches(task) & filter.word_matches(task, filter_input_value))
                        .fold(Column::new().spacing(20), |column, (i, task)| {
                            column.push(
                                task.view()
                                    .map(move |message| Message::TaskMessage(i, message)),
                            )
                        })
                        .into()
                } else {
                    empty_message(match filter {
                        Filter::All => "まだ何のタスクもありません...",
                        Filter::Active => "タスクを全て完了しました :D",
                        Filter::Completed => "まだ何のタスクも完了していません...",
                    })
                };

                let content = Column::new()
                    .max_width(800)
                    .spacing(20)
                    .push(input)
                    .push(importance_selector)
                    .push(filter_textbox)
                    .push(controls)
                    .push(tasks);

                Scrollable::new(scroll)
                    .padding(40)
                    .push(Container::new(content).width(Length::Fill).center_x())
                    .into()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    description: String,
    file_path: PathBuf,
    completed: bool,
    date: String,
    importance: Importance,
    #[serde(skip)]
    state: TaskState,
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Idle {
        edit_button: button::State,
        start_process_button: button::State,
    },
    Editing {
        text_input: text_input::State,
        delete_button: button::State,
    },
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Idle {
            edit_button: button::State::new(),
            start_process_button: button::State::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Completed(bool),
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
    StartProcess(PathBuf),
}

impl Task {
    fn new(description: String, file_path: PathBuf,date: String , importance: Importance) -> Self {
        Task {
            description,
            completed: false,
            file_path,
            date,
            importance,
            state: TaskState::Idle {
                edit_button: button::State::new(),
                start_process_button: button::State::new(),
            },
        }
    }

    fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Completed(completed) => {
                self.completed = completed;
            }
            TaskMessage::Edit => {
                let text_input = text_input::State::focused();
                self.state = TaskState::Editing {
                    text_input,
                    delete_button: button::State::new(),
                };
            }
            TaskMessage::DescriptionEdited(new_description) => {
                self.description = new_description;
            }
            TaskMessage::FinishEdition => {
                if !self.description.is_empty() {
                    self.state = TaskState::Idle {
                        edit_button: button::State::new(),
                        start_process_button: button::State::new(),
                    }
                }
            }

            TaskMessage::StartProcess(process) => {
                open::that(process).unwrap();
            }
            TaskMessage::Delete => {}
        }
    }

    fn view(&mut self) -> Element<TaskMessage> {
        match &mut self.state {
            TaskState::Idle {
                edit_button,
                start_process_button,
            } => {
                let checkbox =
                    Checkbox::new(self.completed, &self.description, TaskMessage::Completed)
                        .width(Length::Fill);

                let important = Text::new(self.importance).width(Length::Fill);

                let filename = match self.file_path.file_name(){
                    Some(result) => result.to_str().unwrap().to_string(),
                    None => String::new(),
                    };
                
                let file_extention = match self.file_path.extension() {
                    Some(result) => result.to_str().unwrap().to_string(),
                    None => String::new(),
                    };

                let image = match file_extention.as_str() {
                    "txt" => Image::new("icons/icons8-txt-48.png")
                    .width(Length::Units(30))
                    .height(Length::Units(30)),

                    "xlsx" => Image::new("icons/icons8-xls-48.png")
                    .width(Length::Units(30))
                    .height(Length::Units(30)),

                    "jpg" => Image::new("icons/icons8-jpg-48.png")
                    .width(Length::Units(30))
                    .height(Length::Units(30)),

                    "exe" => Image::new("icons/icons8-exe-48.png")
                    .width(Length::Units(30))
                    .height(Length::Units(30)),

                    "zip" => Image::new("icons/icons8-zip-48.png")
                    .width(Length::Units(30))
                    .height(Length::Units(30)),

                    _ => Image::new("")
                    .width(Length::Units(30))
                    .height(Length::Units(30)),
                };

                
                let datetime_text = Text::new(&self.date);


                Column::new()
                    .push(
                        Row::new()
                            .spacing(20)
                            .align_items(Align::Center)
                            .push(checkbox)
                            .push(
                                Button::new(edit_button, edit_icon())
                                    .on_press(TaskMessage::Edit)
                                    .padding(10)
                                    .style(style::Button::Icon),
                            ),
                    )
                    .push(
                        Row::new()
                        .push(Button::new(start_process_button, image).on_press(
                                TaskMessage::StartProcess(PathBuf::from(&self.file_path))))
                        .push(Space::new(Length::Units(5),Length::Units(5)))
                        .push(Text::new(filename)).align_items(Align::End)
                        .push(Space::new(Length::Fill,Length::Units(5)))
                    )
                    .push(important)
                    .push(Space::new(Length::Fill,Length::Units(5)))
                    .push(datetime_text).align_items(Align::End)
                    .into()
            }
            TaskState::Editing {
                text_input,
                delete_button,
            } => {
                let text_input = TextInput::new(
                    text_input,
                    "Describe your task...",
                    &self.description,
                    TaskMessage::DescriptionEdited,
                )
                .on_submit(TaskMessage::FinishEdition)
                .padding(10);

 
                Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(text_input)
                    .push(
                        Button::new(
                            delete_button,
                            Row::new()
                                .spacing(10)
                                .push(delete_icon())
                                .push(Text::new("Delete")),
                        )
                        .on_press(TaskMessage::Delete)
                        .padding(10)
                        .style(style::Button::Destructive),
                    )
                    .into()
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Controls {
    all_button: button::State,
    active_button: button::State,
    completed_button: button::State,
}

impl Controls {
    fn view(&mut self, tasks: &[Task], current_filter: Filter) -> Row<Message> {
        let Controls {
            all_button,
            active_button,
            completed_button,
        } = self;

        let tasks_left = tasks.iter().filter(|task| !task.completed).count();

        let filter_button = |state, label, filter, current_filter| {
            let label = Text::new(label).size(16);
            let button = Button::new(state, label).style(if filter == current_filter {
                style::Button::FilterSelected
            } else {
                style::Button::FilterActive
            });

            button.on_press(Message::FilterChanged(filter)).padding(8)
        };


        Row::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(
                Text::new(format!(
                    "{} {} left",
                    tasks_left,
                    if tasks_left == 1 { "task" } else { "tasks" }
                ))
                .width(Length::Fill)
                .size(16),
            )
            .push(
                Row::new()
                    .width(Length::Shrink)
                    .spacing(10)
                    .push(filter_button(
                        all_button,
                        "All",
                        Filter::All,
                        current_filter,
                    ))
                    .push(filter_button(
                        active_button,
                        "Active",
                        Filter::Active,
                        current_filter,
                    ))
                    .push(filter_button(
                        completed_button,
                        "Completed",
                        Filter::Completed,
                        current_filter,
                    )),
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}

impl Filter {
    fn matches(&self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !task.completed,
            Filter::Completed => task.completed,
        }
    }

    fn word_matches(&self, task: &Task ,filter_input_value: &String) -> bool {
        task.description.contains(filter_input_value)
    }

//    fn importance_matches(&self, task: &Task ,importance: &i8) -> bool {
//
//    }
}

fn loading_message<'a>() -> Element<'a, Message> {
    Container::new(
        Text::new("Loading...")
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}

fn empty_message<'a>(message: &str) -> Element<'a, Message> {
    Container::new(
        Text::new(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .color([0.7, 0.7, 0.7]),
    )
    .width(Length::Fill)
    .height(Length::Units(200))
    .center_y()
    .into()
}

// Fonts
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/NotoSansJP-Regular.otf"),
};

fn icon(unicode: char) -> Text {
    Text::new(unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(iced::HorizontalAlignment::Center)
        .size(20)
}

fn edit_icon() -> Text {
    icon('\u{F303}')
}

fn delete_icon() -> Text {
    icon('\u{F1F8}')
}

// Persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedState {
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
enum SaveError {
    FileError,
    WriteError,
    FormatError,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    fn path() -> std::path::PathBuf {
        let mut path = if let Some(project_dirs) =
            directories_next::ProjectDirs::from("rs", "Iced", "Todos")
        {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or(std::path::PathBuf::new())
        };

        path.push("todos.json");

        path
    }

    async fn load() -> Result<SavedState, LoadError> {
        use async_std::prelude::*;

        let mut contents = String::new();

        let mut file = async_std::fs::File::open(Self::path())
            .await
            .map_err(|_| LoadError::FileError)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::FileError)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::FormatError)
    }

    async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::FormatError)?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::FileError)?;
        }

        {
            let mut file = async_std::fs::File::create(path)
                .await
                .map_err(|_| SaveError::FileError)?;

            file.write_all(json.as_bytes())
                .await
                .map_err(|_| SaveError::WriteError)?;
        }

        // This is a simple way to save at most once every couple seconds
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl SavedState {
    fn storage() -> Option<web_sys::Storage> {
        let window = web_sys::window()?;

        window.local_storage().ok()?
    }

    async fn load() -> Result<SavedState, LoadError> {
        let storage = Self::storage().ok_or(LoadError::FileError)?;

        let contents = storage
            .get_item("state")
            .map_err(|_| LoadError::FileError)?
            .ok_or(LoadError::FileError)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::FormatError)
    }

    async fn save(self) -> Result<(), SaveError> {
        let storage = Self::storage().ok_or(SaveError::FileError)?;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::FormatError)?;

        storage
            .set_item("state", &json)
            .map_err(|_| SaveError::WriteError)?;

        let _ = wasm_timer::Delay::new(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        FilterActive,
        FilterSelected,
        Icon,
        Destructive,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            match self {
                Button::FilterActive => button::Style::default(),
                Button::FilterSelected => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.7))),
                    border_radius: 10.0,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                },
                Button::Icon => button::Style {
                    text_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..button::Style::default()
                },
                Button::Destructive => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                    border_radius: 5.0,
                    text_color: Color::WHITE,
                    shadow_offset: Vector::new(1.0, 1.0),
                    ..button::Style::default()
                },
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            button::Style {
                text_color: match self {
                    Button::Icon => Color::from_rgb(0.2, 0.2, 0.7),
                    Button::FilterActive => Color::from_rgb(0.2, 0.2, 0.7),
                    _ => active.text_color,
                },
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            }
        }
    }
}
