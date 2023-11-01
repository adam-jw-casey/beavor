#![warn(clippy::pedantic)]

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

use tokio::sync::oneshot;

use chrono::{
    NaiveDate,
    offset::Utc,
};

use backend::{
    DatabaseManager,
    Task,
    Schedule,
    Hyperlink,
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
        TimerState,
        LinkMessage,
    },
    confirm_modal,
};

use widgets::task_editor::UpdateDraftTask;

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
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
    DeleteTask,
}

#[derive(Debug, Clone)]
pub enum Message{
    Refresh(Cache),
    Tick(Instant),
    ForceSelectTask(Option<Task>),
    TrySelectTask(Option<Task>),
    TryDeleteTask,
    UpdateDraftTask(UpdateDraftTask),
    StartTimer,
    StopTimer,
    ToggleTimer, // Consider having separate start/stop/toggle messages
    Modal(ModalMessage),
    NewTask,
    Mutate(MutateMessage),
    Loaded(State),
    ScrollDownCalendar,
    ScrollUpCalendar,
    ScrollUpMaxCalendar, // TODO merge these calendar messages
    SetEditingLinkID(Option<usize>),
    Open(String),
    None,
    FilterToDate(Option<NaiveDate>), //TODO I have a feeling I'll want more filters at some point
}

#[derive(Debug, Clone)]
pub struct DisplayedTask{
    selected:               Option<Task>,
    pub draft:              Task,
    pub editing_link_idx:   Option<usize>,
    pub timer:              TimerState,
}

impl DisplayedTask{
    fn is_unmodified(&self) -> bool{
        match &self.selected{
            Some(t) => *t == self.draft,
            None => self.draft == Task::default(),
        }
    }

    fn select(&mut self, maybe_task: Option<Task>){
        self.selected = maybe_task.clone();
        self.draft = match maybe_task{
            Some(t) =>  t.clone(),
            None => Task::default(),
        };
    }

    fn start_timer(&mut self){
        if matches!(self.timer, TimerState::Timing{..}){
            self.timer = TimerState::Timing{start_time: Utc::now()};
        }
    }

    fn stop_timer(&mut self){
        if let Some(minutes) = self.timer.num_minutes_running(){
            self.draft.time_used += minutes;
            self.timer = TimerState::Stopped;
        }
    }

    fn toggle_timer(&mut self){
        match self.timer{
            TimerState::Timing{..} => self.stop_timer(),
            TimerState::Stopped => self.start_timer(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct State{
    db:             DatabaseManager,
    cache:          Cache,
    displayed_task: DisplayedTask,
    modal_state:    ModalType,
    calendar_state: CalendarState,
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
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Beavor, iced::Command<Message>) {
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
                            loaded_schedule: db.schedule().await,
                        },
                        db,
                        displayed_task: DisplayedTask{
                            selected: None,
                            draft: Task::default(),
                            timer: TimerState::Stopped,
                            editing_link_idx: None,
                        },
                        modal_state: ModalType::None,
                        calendar_state: CalendarState::default(),
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
            Beavor::Loading => {
                match message{
                    Message::Loaded(state) => {
                        *self = Self::Loaded(state);
                    },
                    Message::Tick(_) | Message::None => (),
                    m => panic!("Should never happen: {m:#?}")
                }
                Command::none()
            },
            Beavor::Loaded(state) => match message{
                Message::Mutate(mutate_message) => Beavor::mutate(state, &mutate_message),
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
                            Message::NewTask => Self::try_select_task(state, None),
                            Message::TryDeleteTask => {
                                // Confirm before deleting
                                let name = state.displayed_task.draft.name.clone();
                                Self::update_modal_state(&mut state.modal_state, ModalType::Confirm(ConfirmationRequest{
                                    message: format!("Are you sure you want to delete '{name}'?"),
                                    run_on_confirm: Box::new(Message::Mutate(MutateMessage::DeleteTask))
                                }));
                            },
                            Message::UpdateDraftTask(task_field_update) => {
                                if let Some(m) = Beavor::update_draft_task(&mut state.displayed_task.draft, &mut state.displayed_task.editing_link_idx, task_field_update){
                                    Self::update_modal_state(&mut state.modal_state, m); //TODO this whole thing could be a method on DisplayedTask
                                }
                            },
                            Message::TrySelectTask(maybe_task) => Self::try_select_task(state, maybe_task),
                            Message::ForceSelectTask(maybe_task) => state.displayed_task.select(maybe_task), // TODO this message needs to go
                            Message::StartTimer => state.displayed_task.start_timer(),
                            Message::StopTimer => state.displayed_task.stop_timer(),
                            #[allow(clippy::single_match_else)]
                            Message::ToggleTimer => state.displayed_task.toggle_timer(),
                            Message::Refresh(cache) => {
                                state.cache = cache;
                                // This is called after mutating state, e.g., saving a task
                                // If the task was finished, need to also clear the displayed task
                                if state.displayed_task.draft.finished{
                                    state.displayed_task.select(None);
                                }
                            },
                            Message::Tick(_) | Message::None => (),
                            Message::Loaded(_) | Message::Mutate(_) => panic!("Should never happen"),
                            Message::ScrollDownCalendar => state.calendar_state.scroll_down(),
                            Message::ScrollUpCalendar => state.calendar_state.scroll_up(),
                            Message::ScrollUpMaxCalendar => state.calendar_state.scroll_up_max(),
                            Message::Open(url) => {
                                if open::that(url.clone()).is_err(){
                                    println!("Error opening '{url}'"); // TODO this should be visible in the GUI, not just the terminal
                                };
                            },
                            Message::SetEditingLinkID(h_id) => state.displayed_task.editing_link_idx = h_id,
                            Message::FilterToDate(date) => state.calendar_state.filter_date = date,
                            Message::Modal(_) => panic!("Can never happen"),
                        }
                        Command::none()
                    }
                }}
            },
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
                        &state.displayed_task.draft,
                        &state.displayed_task.timer,
                        &state.modal_state,
                        state.displayed_task.editing_link_idx, // TODO just pass in displayed_task itself
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

    #[must_use] fn update_draft_task(draft_task: &mut Task, editing_link: &mut Option<usize>,message: UpdateDraftTask) -> Option<ModalType>{
        use UpdateDraftTask as UDT;

        match message{
            UDT::NextActionDate(next_action_date) => {
                draft_task.next_action_date = next_action_date;
                Some(ModalType::None)
            },
            UDT::DueDate(due_date) => {
                draft_task.due_date = due_date;
                Some(ModalType::None)
            },
            other => {
                match other{
                    UDT::NextActionDate(_) | UDT::DueDate(_) => panic!("This will never happen"),
                    UDT::Category(category) => draft_task.category = category,
                    UDT::Name(name) => draft_task.name = name,
                    UDT::TimeNeeded(time_needed) => if let Ok(time_needed) = time_needed {draft_task.time_needed = time_needed},
                    UDT::TimeUsed(time_used) => if let Ok(time_used) = time_used {draft_task.time_used = time_used},
                    UDT::Notes(notes) => draft_task.notes = notes,
                    UDT::Finished(finished) => draft_task.finished = finished,
                    UDT::Link(link_message) => match link_message{
                        LinkMessage::New => if !draft_task.links.contains(&Hyperlink::default()){
                            draft_task.links.push(Hyperlink::default());
                            *editing_link = Some(draft_task.links.len()-1);

                        },
                        LinkMessage::Delete(idx) => {
                            draft_task.links.remove(idx);
                        },
                        LinkMessage::Update((link, idx)) => {
                            draft_task.links[idx] = link;
                        },
                    }
                }
                None
            }
        }
    }

    fn mutate(state: &mut State, message: &MutateMessage) -> Command<Message>{
        state.displayed_task.stop_timer();
        // TODO this is so stupid but it works and I got tired of hacking at Arc<>
        let db_clone1 = state.db.clone();
        let db_clone2 = state.db.clone();
        let t1 = state.displayed_task.draft.clone();
        let t2 = state.displayed_task.draft.clone();

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
                    MutateMessage::DeleteTask => {
                        let t = std::mem::take(&mut state.displayed_task.draft);
                        state.displayed_task.selected = None;
                        Command::perform(async move {
                            db_clone1.delete_task(&t).await;
                            tx.send(()).unwrap();
                        }, |()| Message::NewTask)
                    }
                },
                Command::perform(async move {
                    rx.await.unwrap();

                    Cache{
                        loaded_tasks:    db_clone2.open_tasks().await.into(),
                        loaded_schedule: db_clone2.schedule().await,
                    }
                }, Message::Refresh)
            ]
        )
    }
}
