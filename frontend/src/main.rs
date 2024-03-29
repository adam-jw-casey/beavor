use std::fs;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use tokio::sync::oneshot;

use chrono::NaiveDate;

use iced::widget::{
    container,
    row,
    column,
    text,
    rule::Rule,
    combo_box::State as ComboBoxState,
};

use iced::{
    Application,
    Element,
    Settings,
    Subscription,
    Command,
    executor,
    Theme,
    time::Instant,
    Length,
    Alignment,
    font,
    window,
};

use backend::{
    DatabaseManager,
    Task,
    Schedule,
    schedule::WorkWeek,
    TimeSheet,
};

mod widgets;
use widgets::{
    calendar::{
        calendar,
        Message as CalendarMessage,
        State as CalendarState,
    },
    task_scroller,
    task_editor::{
        task_editor,
        DisplayedTask,
        TimerMessage,
    },
    confirm_modal,
    error_bar,
    error_bar::State as ErrorBarState,
};

use widgets::task_editor::UpdateDraftTask;

const CONFIG_FILE_PATH: &str = "./resources/config.json";
const WORKLIST_PATH:    &str = "worklist.db";
const TIMESHEET_PATH:   &str = "timesheet.csv";

fn main() {
    let default = Settings::<Flags>::default();

    let flags: Flags = serde_json::from_str(
        &fs::read_to_string(CONFIG_FILE_PATH)
            .unwrap_or_else(|_| {
                let default = serde_json::to_string_pretty(&Flags::default())
                    .expect("Flags::default() serializes correctly by definition");

                fs::write(CONFIG_FILE_PATH, &default)
                    .expect("Panics if cannot write to filesystem");

                default
            })
    ).expect("Panics if config file incorrectly formatted");

    let settings: Settings<Flags> = Settings {
        flags,
        id: default.id,
        window: default.window,
        default_font: default.default_font,
        default_text_size: default.default_text_size,
        antialiasing: default.antialiasing,
        exit_on_close_request: default.exit_on_close_request,
    };

    Beavor::run(settings)
        .expect("Application failed");
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Flags {
    work_week: WorkWeek,
}

#[derive(Debug, Clone)]
pub struct ConfirmationRequest {
    message: String,
    run_on_confirm: Box<Message>,
}

#[derive(Debug, Clone)]
pub enum ModalMessage {
    Show(ModalType),
    Ok,
}

#[derive(Debug, Clone)]
pub enum ModalType {
    None,
    NextAction,
    DueDate,
    Confirm(ConfirmationRequest),
}

#[derive(Debug, Clone)]
pub enum MutateMessage {
    SaveDraftTask,
    ForceDeleteTask,
    VacationStatus(NaiveDate, bool),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    Refresh(Cache),
    Tick(Instant),
    ForceSelectTask(Option<Task>),
    TrySelectTask(Option<Task>),
    TryDeleteTask,
    UpdateDraftTask(UpdateDraftTask),
    Modal(ModalMessage),
    TryNewTask,
    Mutate(MutateMessage),
    Loaded(State),
    SetEditingLinkID(Option<usize>),
    Open(String),
    None,
    Calendar(CalendarMessage),
    Timer(TimerMessage),
    UpdateFlags(Flags),
    Error(Option<String>),
}

#[derive(Debug, Clone)]
pub struct State {
    db:             DatabaseManager,
    timesheet:      Arc<Mutex<TimeSheet>>,
    cache:          Cache,
    displayed_task: DisplayedTask,
    modal_state:    ModalType,
    error_bar:      ErrorBarState,
    calendar_state: CalendarState,
    flags:          Flags,
}

#[derive(Debug, Clone)]
pub struct Cache {
    loaded_tasks: Vec<Task>,
    loaded_schedule: Schedule,
    categories: ComboBoxState<String>,
}

enum Beavor {
    Loading,
    Loaded(State),
}

impl Application for Beavor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Beavor, iced::Command<Message>) {
        (
            Self::Loading,
            Command::batch(vec![
                Command::perform(async {
                    let db = match DatabaseManager::new(WORKLIST_PATH).await {
                        Ok(db) => db,
                        Err(_) => DatabaseManager::with_new_database(WORKLIST_PATH).await.expect("Should be able to create database"),
                    };

                    let tasks = db.open_tasks().await;

                    let timesheet: Arc<Mutex<TimeSheet>> = Arc::new(Mutex::new(match TimeSheet::new_timesheet(TIMESHEET_PATH) {
                        Ok(logger) => logger,
                        Err(_) => TimeSheet::open(TIMESHEET_PATH).expect("Should be able to open timesheet"),
                    }));

                    State {
                        cache: Cache {
                            loaded_schedule: db.schedule(flags.work_week.clone(), &tasks).await,
                            categories: ComboBoxState::new(Self::unique_categories(&tasks)),
                            loaded_tasks: tasks,
                        }, // TODO this should call the same code that refreshes the cache
                        db,
                        timesheet,
                        displayed_task: DisplayedTask::default(),
                        modal_state:    ModalType::None,
                        error_bar:      ErrorBarState::default(),
                        calendar_state: CalendarState::default(),
                        flags,
                    }
                }, Message::Loaded),
                font::load(iced_aw::graphics::icons::ICON_FONT_BYTES).map(|_| Message::None),
                window::change_icon(window::icon::from_file_data(
                    include_bytes!("../../resources/logo.png"),
                    None
                ).expect("Icon should load"))
            ])
        )
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            Beavor::Loading => self.update_loading(message),
            Beavor::Loaded(_) => self.update_loaded(message),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = match self {
            Beavor::Loading => text("Loading...").into(),
            Beavor::Loaded(state) =>
                column![
                    row![
                        task_scroller(
                            &state.cache.loaded_tasks,
                            state.calendar_state.filter_date.as_ref(),
                            &state.cache.loaded_schedule
                        )
                            .width(Length::FillPortion(2))
                            .height(Length::FillPortion(1)),
                        Rule::vertical(4),
                        task_editor(
                            &state.displayed_task,
                            &state.modal_state,
                            &state.cache.categories
                        )
                            .padding(8)
                            .width(Length::FillPortion(3))
                            .height(Length::FillPortion(1)),
                        Rule::vertical(4),
                        calendar(&state.cache.loaded_schedule, &state.calendar_state),
                        confirm_modal(&state.modal_state),
                    ]
                        .align_items(Alignment::End)
                        .height(Length::Fill)
                        .width(Length::Fill),
                    Rule::horizontal(4),
                    error_bar(&state.error_bar),
                ]
                    .spacing(8)
                    .into()
        };

        container(content)
            .center_x()
            .center_y()
            .padding(8)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::time::every(iced::time::Duration::from_secs(1)).map(Message::Tick)
    }
}

impl Beavor {
    fn update_loading(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(state) => {
                *self = Self::Loaded(state);
            },
            Message::Tick(_) | Message::None => (),
            m => panic!("Should never happen: {m:#?}")
        }
        Command::none()
    }

    fn update_loaded(&mut self, message: Message) -> Command<Message> {
        let state = match self {
            Beavor::Loaded(state) => state,
            Beavor::Loading => panic!("Should never happen"),
        };

        match message {
            Message::Mutate(mutate_message) => Beavor::mutate(&state.db, &mut state.displayed_task, &mutate_message, &state.flags.work_week, state.timesheet.clone()),
            other => {match other {
                Message::Modal(modal_message) => {
                    match modal_message {
                        ModalMessage::Show(modal_type) => {
                            Self::update_modal_state(&mut state.modal_state, modal_type);
                            Command::none()
                        },
                        ModalMessage::Ok => {
                            let m = Self::complete_modal(&mut state.modal_state);
                            self.update(m)
                        }
                    }
                },
                Message::Open(url) => {
                    if open::that(url.clone()).is_err() {
                        self.update(Message::Error(Some(format!("Error opening ' {url}'"))))
                    }else {
                        Command::none()
                    }
                },
                other => {
                    match other {
                        Message::TryNewTask => Self::try_select_task(state, None),
                        Message::TryDeleteTask => {
                            // Confirm before deleting
                            let name = state.displayed_task.draft.name.clone();
                            Self::update_modal_state(&mut state.modal_state, ModalType::Confirm(ConfirmationRequest {
                                message: format!("Are you sure you want to delete ' {name}'?"),
                                run_on_confirm: Box::new(Message::Mutate(MutateMessage::ForceDeleteTask))
                            }));
                        },
                        Message::UpdateDraftTask(task_field_update) => {
                            if let Some(m) = state.displayed_task.update_draft(task_field_update) {
                                Self::update_modal_state(&mut state.modal_state, m);
                            }
                        },
                        Message::TrySelectTask(maybe_task) => Self::try_select_task(state, maybe_task),
                        Message::ForceSelectTask(maybe_task) => state.displayed_task.select(maybe_task),
                        Message::Timer(message) => state.displayed_task.update_timer(message),
                        Message::Refresh(cache) => {
                            state.cache = cache;
                            // This is called after mutating state, e.g., saving a task
                            // If the task was finished, need to also clear the displayed task
                            if state.displayed_task.draft.finished {
                                state.displayed_task.select(None);
                            }
                        },
                        Message::SetEditingLinkID(h_id) => Self::set_editing_link_id(state, h_id),
                        Message::Calendar(calendar_message) => state.calendar_state.update(calendar_message),
                        Message::UpdateFlags(new_flags) => {
                            fs::write(
                                CONFIG_FILE_PATH, serde_json::to_string_pretty(&new_flags)
                                  .expect("I don't know how this could fail")
                            ).expect("Panics if cannot write to filesystem");

                            state.flags = new_flags;
                        },
                        Message::Error(maybe_error) => state.error_bar.error = maybe_error,
                        Message::Tick(_) | Message::None => (),
                        Message::Modal(_) => panic!("Can never happen"),
                        Message::Loaded(_) | Message::Mutate(_) |  Message::Open(_)=> panic!("Should never happen"),
                    }
                    Command::none()
                }
            }}
        }
    }

    fn set_editing_link_id(state: &mut State, id: Option<usize>) {
        state.displayed_task.editing_link_idx = id;
    }

    fn try_select_task(state: &mut State, maybe_task: Option<Task>) {
        // Stop timer and save if timer is running
        state.displayed_task.stop_timer();
        // Close the links list
        Self::set_editing_link_id(state, None);

        // Don't overwrite a modified task
        if state.displayed_task.is_unmodified() {
            state.displayed_task.select(maybe_task);
        }else {
            Self::update_modal_state(&mut state.modal_state, ModalType::Confirm(ConfirmationRequest {
                message: "Unsaved changes will be lost. Continue without saving?".to_string(),
                run_on_confirm: Box::new(Message::ForceSelectTask(maybe_task))
            }));
        }
    }

    fn update_modal_state(modal_state: &mut ModalType, modal_type: ModalType) {
        *modal_state = modal_type;
    }

    #[must_use] fn complete_modal(modal_state: &mut ModalType) -> Message {
        match &modal_state {
            ModalType::Confirm(confirmation_request) => {
                let m = confirmation_request.run_on_confirm.clone();
                *modal_state = ModalType::None; // this bypasses the update function
                *m
            },
            _ => panic!("Should never happen"),
        }
    }

    fn mutate(db: &DatabaseManager, displayed_task: &mut DisplayedTask, message: &MutateMessage, work_week: &WorkWeek, timesheet: Arc<Mutex<TimeSheet>>) -> Command<Message> {
        displayed_task.stop_timer();
        // TODO this is so stupid but it works and I got tired of hacking at Arc<>
        let db_clone1 = db.clone();
        let db_clone2 = db.clone();
        let t1 = displayed_task.clone();
        let t2 = displayed_task.draft.clone();
        let work_week_clone = work_week.clone();

        let (tx, rx) = oneshot::channel::<()>(); // Synchronize the writes to the database with the reads that update the cache

        Command::batch(
            [
                match message {
                    MutateMessage::SaveDraftTask => match t1.draft.id {
                        Some(_) => Command::perform(async move {
                            // Log time worked to the timesheet
                            timesheet
                                .lock().
                                expect("Fails if mutex is poisoned")
                                .log_time(t1.added_time().expect("We've checked that the id is Some, so the task already exists and this will also be Some"), &t1.draft)
                                .expect("Fails if cannot write to timesheet csv file");

                            // Update the database with the new task
                            db_clone1.update_task(&t1.draft).await
                                .expect("The task should already exist");

                            // Done - let the cache refresh
                            tx.send(()).unwrap();
                        }, |()| Message::ForceSelectTask(Some(t2))),
                        None => Command::perform(async move {
                            let t: Task = db_clone1.create_task(&t1.draft).await;
                            tx.send(()).unwrap();
                            t
                        }, |t: Task| Message::ForceSelectTask(Some(t))),
                    },
                    MutateMessage::ForceDeleteTask => {
                        let t = std::mem::take(&mut displayed_task.draft);
                        displayed_task.select(None);
                        Command::perform(async move {
                            db_clone1.delete_task(t).await;

                            // Done - let the cache refresh
                            tx.send(()).unwrap();
                        }, |()| Message::TryNewTask)
                    },
                    MutateMessage::VacationStatus(date, is_vacation) => {
                        let date = *date; // These copies make lifetimes happy
                        let is_vacation = *is_vacation;
                        Command::perform(async move {
                            if is_vacation {
                                db_clone1.add_vacation_day(&date).await;
                            }else {
                                db_clone1.delete_vacation_day(&date).await;
                            }
                            tx.send(()).unwrap();
                        }, |()| Message::None)
                    },
                },
                Command::perform(async move {
                    rx.await.unwrap();
                    let tasks = db_clone2.open_tasks().await;

                    Cache {
                        loaded_schedule: db_clone2.schedule(work_week_clone, &tasks).await,
                        categories: ComboBoxState::new(Self::unique_categories(&tasks)),
                        loaded_tasks: tasks,
                    }
                }, Message::Refresh)
            ]
        )
    }

    fn unique_categories(tasks: &[Task]) -> Vec<String> {
        tasks
            .iter()
            .map(|t| t.category.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}
