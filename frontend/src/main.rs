use std::fs;

use serde::{Deserialize, Serialize};

use tokio::sync::oneshot;

use chrono::NaiveDate;

use iced::widget::{
    container,
    row,
    text,
    rule::Rule,
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
};

mod widgets;
use widgets::{
    calendar::{
        calendar,
        CalendarState,
    },
    task_scroller,
    task_editor::{
        task_editor,
        DisplayedTask,
    },
    confirm_modal,
};

use widgets::task_editor::UpdateDraftTask;

const CONFIG_FILE_PATH: &str = "./resources/config.json";

fn main() {
    // TODO eventially the setting call should be async so that a window with "loading" shows,
    // rather than nothing
    // TODO why on earth does iced::Settings<> not #[Derive(Serialize, Deserialize)]?
    let default = Settings::<Flags>::default();

    let flags: Flags = serde_json::from_str(
        &fs::read_to_string(CONFIG_FILE_PATH)
            .unwrap_or_else(|_|{
                let default = serde_json::to_string_pretty(&Flags::default())
                    .expect("Flags::default() serializes correctly by definition");

                fs::write(CONFIG_FILE_PATH, &default)
                    .expect("Panics if cannot write to filesystem");

                default
            })
    ).expect("Panics if config file incorrectly formatted");

    let settings: Settings<Flags> = Settings{
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
pub struct Flags{
    work_week: WorkWeek,
}

#[derive(Debug, Clone)]
pub struct ConfirmationRequest{
    message: String,
    run_on_confirm: Box<Message>,
}

#[derive(Debug, Clone)]
pub enum ModalMessage{
    Show(ModalType),
    Ok,
}

#[derive(Debug, Clone)]
pub enum ModalType{
    None,
    NextAction,
    DueDate,
    Confirm(ConfirmationRequest),
}

#[derive(Debug, Clone)]
pub enum MutateMessage{
    SaveDraftTask,
    ForceDeleteTask,
}

#[derive(Debug, Clone, Copy)]
pub enum CalendarMessage{
    ScrollDown,
    ScrollUp,
    ScrollUpMax,
    FilterToDate(Option<NaiveDate>), //TODO I have a feeling I'll want more filters at some point
    ClickDate(Option<NaiveDate>),
}

#[derive(Debug, Clone, Copy)]
pub enum TimerMessage{
    Start,
    Stop,
    Toggle,
}

#[derive(Debug, Clone)]
pub enum Message{
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
}

#[derive(Debug, Clone)]
pub struct State{
    db:             DatabaseManager,
    cache:          Cache,
    displayed_task: DisplayedTask,
    modal_state:    ModalType,
    calendar_state: CalendarState,
    flags:          Flags,
}

#[derive(Debug, Clone)]
pub struct Cache{
    loaded_tasks: Box<[Task]>,
    loaded_schedule: Schedule,
}

enum Beavor{
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
                Command::perform(async{
                    let db = match DatabaseManager::new("worklist.db").await{
                        Ok(db) => db,
                        Err(_) => DatabaseManager::with_new_database("worklist.db").await.expect("Should be able to create database"),
                    };
                    State{
                        cache: Cache{
                            loaded_tasks: db.open_tasks().await.into(),
                            loaded_schedule: db.schedule(flags.work_week.clone()).await,
                        },
                        db,
                        displayed_task: DisplayedTask::default(),
                        modal_state: ModalType::None,
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
        match self{
            Beavor::Loading => self.update_loading(message),
            Beavor::Loaded(_) => self.update_loaded(message),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = match self{
            Beavor::Loading => text("Loading...").into(),
            Beavor::Loaded(state) =>
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
                    .width(Length::Fill)
                    .into()
        };

        container(content)
            .center_x()
            .center_y()
            .padding(8)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message>{
        iced::time::every(iced::time::Duration::from_secs(1)).map(Message::Tick)
    }
}

impl Beavor{
    fn update_loading(&mut self, message: Message) -> Command<Message>{
        match message{
            Message::Loaded(state) => {
                *self = Self::Loaded(state);
            },
            Message::Tick(_) | Message::None => (),
            m => panic!("Should never happen: {m:#?}")
        }
        Command::none()
    }

    fn update_loaded(&mut self, message: Message) -> Command<Message>{
        let state = match self{
            Beavor::Loaded(state) => state,
            Beavor::Loading => panic!("Should never happen"),
        };

        match message{
            Message::Mutate(mutate_message) => Beavor::mutate(&state.db, &mut state.displayed_task, &mutate_message, &state.flags.work_week),
            other => {match other{
                Message::Modal(modal_message) => {
                    match modal_message{
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
                other => {
                    match other{
                        Message::TryNewTask => Self::try_select_task(state, None),
                        Message::TryDeleteTask => {
                            // Confirm before deleting
                            let name = state.displayed_task.draft.name.clone();
                            Self::update_modal_state(&mut state.modal_state, ModalType::Confirm(ConfirmationRequest{
                                message: format!("Are you sure you want to delete '{name}'?"),
                                run_on_confirm: Box::new(Message::Mutate(MutateMessage::ForceDeleteTask))
                            }));
                        },
                        Message::UpdateDraftTask(task_field_update) => {
                            if let Some(m) = state.displayed_task.update_draft(task_field_update){
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
                            if state.displayed_task.draft.finished{
                                state.displayed_task.select(None);
                            }
                        },
                        Message::Open(url) => {
                            if open::that(url.clone()).is_err(){
                                println!("Error opening '{url}'"); // TODO this should be visible in the GUI, not just the terminal
                            };
                        },
                        Message::SetEditingLinkID(h_id) => state.displayed_task.editing_link_idx = h_id,
                        Message::Calendar(calendar_message) => state.calendar_state.update(calendar_message),
                        Message::UpdateFlags(new_flags) => {
                            fs::write(
                                CONFIG_FILE_PATH, serde_json::to_string_pretty(&new_flags)
                                  .expect("I don't know how this could fail")
                            ).expect("Panics if cannot write to filesystem");

                            state.flags = new_flags;
                        },
                        Message::Tick(_) | Message::None => (),
                        Message::Modal(_) => panic!("Can never happen"),
                        Message::Loaded(_) | Message::Mutate(_) => panic!("Should never happen"),
                    }
                    Command::none()
                }
            }}
        }
    }


    fn try_select_task(state: &mut State, maybe_task: Option<Task>){
        // Stop timer and save if timer is running
        state.displayed_task.stop_timer();

        // Don't overwrite a modified task
        if state.displayed_task.is_unmodified(){
            state.displayed_task.select(maybe_task);
        }else{
            Self::update_modal_state(&mut state.modal_state, ModalType::Confirm(ConfirmationRequest{
                message: "Unsaved changes will be lost. Continue without saving?".to_string(),
                run_on_confirm: Box::new(Message::ForceSelectTask(maybe_task))
            }));
        }
    }

    fn update_modal_state(modal_state: &mut ModalType, modal_type: ModalType){
        *modal_state = modal_type;
    }

    #[must_use] fn complete_modal(modal_state: &mut ModalType) -> Message{
        match &modal_state{
            ModalType::Confirm(confirmation_request) => {
                let m = confirmation_request.run_on_confirm.clone();
                *modal_state = ModalType::None; // this bypasses the update function
                *m
            },
            _ => panic!("Should never happen"),
        }
    }

    fn mutate(db: &DatabaseManager, displayed_task: &mut DisplayedTask, message: &MutateMessage, work_week: &WorkWeek) -> Command<Message>{
        displayed_task.stop_timer();
        // TODO this is so stupid but it works and I got tired of hacking at Arc<>
        let db_clone1 = db.clone();
        let db_clone2 = db.clone();
        let t1 = displayed_task.draft.clone();
        let t2 = displayed_task.draft.clone();
        let work_week_clone = work_week.clone();

        let (tx, rx) = oneshot::channel::<()>(); // Synchronize the writes to the database with the reads that update the cache

        Command::batch(
            [
                match message{
                    MutateMessage::SaveDraftTask => match t1.id{
                        Some(_) => Command::perform(async move {
                            db_clone1.update_task(&t1).await
                                .expect("The task should already exist");
                            tx.send(()).unwrap();
                        }, |()| Message::ForceSelectTask(Some(t2))),
                        None => Command::perform(async move {
                            let t: Task = db_clone1.create_task(&t1).await;
                            tx.send(()).unwrap();
                            t
                        }, |t: Task| Message::ForceSelectTask(Some(t))),
                    },
                    MutateMessage::ForceDeleteTask => {
                        let t = std::mem::take(&mut displayed_task.draft);
                        displayed_task.select(None);
                        Command::perform(async move {
                            db_clone1.delete_task(&t).await;
                            tx.send(()).unwrap();
                        }, |()| Message::TryNewTask)
                    }
                },
                Command::perform(async move {
                    rx.await.unwrap();

                    Cache{
                        loaded_tasks:    db_clone2.open_tasks().await.into(),
                        loaded_schedule: db_clone2.schedule(work_week_clone).await,
                    }
                }, Message::Refresh)
            ]
        )
    }
}
