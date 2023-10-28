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
pub enum ModalMessage{
    PickNextActionDate,
    PickDueDate,
    Confirm((String, Box<Message>)),
    Close,
    Ok,
}

#[derive(Debug, Clone)]
pub enum ModalShowing{
    None,
    NextAction,
    DueDate,
    Confirm(String, Box<Message>),
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
    SelectDate(Option<NaiveDate>),
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
pub struct State{
    db:            DatabaseManager,
    selected_task: Option<Task>,
    selected_date: Option<NaiveDate>,
    draft_task:    Task,
    timer_state:   TimerState,
    modal_state:   ModalShowing,
    calendar_state: CalendarState,
    cache:         Cache,
    editing_link: Option<usize>,
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
                        selected_task: None,
                        selected_date: None,
                        draft_task:    Task::default(),
                        timer_state: TimerState::Stopped,
                        modal_state: ModalShowing::None,
                        cache: Cache{
                            loaded_tasks: db.open_tasks().await.into(),
                            loaded_schedule: db.schedule().await,
                        },
                        db,
                        calendar_state: CalendarState::default(),
                        editing_link: None,
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
                        Command::none()
                    },
                    Message::Tick(_) | Message::None => {Command::none()},
                    m => panic!("Should never happen: {m:#?}")
                }
            },
            Beavor::Loaded(state) => match message{ // TODO most of these branches end in
                                                    // Command::none() - how can this be cleaner?
                // Mutate messages modify the database
                Message::Mutate(mutate_message) => Beavor::mutate(state, &mutate_message),
                other => {match other{
                    Message::ForceSelectTask(maybe_task) => {Self::select_task(state, maybe_task); Command::none()}, // TODO this message needs to go
                    Message::TrySelectTask(maybe_task) => {
                        let m = Self::try_select_task(state, maybe_task);
                        self.update(m)
                    },
                    Message::NewTask => {
                        let m = Self::try_select_task(state, None);
                        self.update(m)
                    },
                    Message::TryDeleteTask => {
                        // Confirm before deleting
                        let name = state.draft_task.name.clone();
                        self.update(Message::Modal(ModalMessage::Confirm((
                            format!("Are you sure you want to delete '{name}'?"),
                            Box::new(Message::Mutate(MutateMessage::DeleteTask))
                        ))))
                    },
                    Message::Modal(modal_message) => {
                        let m = Self::handle_modal_message(&mut state.modal_state, modal_message);
                        self.update(m)
                    },
                    Message::UpdateDraftTask(task_field_update) => {
                        let m = Beavor::update_draft_task(&mut state.draft_task, task_field_update);
                        self.update(m)
                    },
                    Message::SelectDate(maybe_date) => {state.selected_date = maybe_date; Command::none()},
                    Message::StartTimer => {Self::start_timer(&mut state.timer_state); Command::none()},
                    Message::StopTimer => {Self::stop_timer(state); Command::none()},
                    #[allow(clippy::single_match_else)]
                    Message::ToggleTimer => {
                        match state.timer_state.num_minutes_running(){
                            Some(_) => Self::stop_timer(state),
                            None => Self::start_timer(&mut state.timer_state),
                        }
                        Command::none()
                    },
                    Message::Refresh(cache) => {
                        state.cache = cache;
                        if state.draft_task.finished{
                            Self::select_task(state, None);
                        }
                        Command::none()
                    },
                    Message::Tick(_) | Message::None => Command::none(),
                    Message::Loaded(_) | Message::Mutate(_) => panic!("Should never happen"),
                    Message::ScrollDownCalendar => {state.calendar_state.scroll_down(); Command::none()},
                    Message::ScrollUpCalendar => {state.calendar_state.scroll_up(); Command::none()},
                    Message::ScrollUpMaxCalendar => {state.calendar_state.scroll_up_max(); Command::none()},
                    Message::Open(url) => {
                        if open::that(url.clone()).is_err(){
                            println!("Error opening '{url}'"); // TODO this should be visible in
                                                               // the GUI, not just the terminal
                        };
                        Command::none()
                    },
                    Message::SetEditingLinkID(h_id) => {state.editing_link = h_id; Command::none()},
                    Message::FilterToDate(date) => {state.calendar_state.filter_date = date; Command::none()},
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
                        &state.draft_task,
                        &state.timer_state,
                        &state.modal_state,
                        state.editing_link,
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
    #[must_use] fn try_select_task(state: &mut State, maybe_task: Option<Task>) -> Message{
        // Stop timer and save if timer is running
        Self::stop_timer(state);

        // Don't overwrite a modified task
        if match &state.selected_task{
            Some(t) => *t == state.draft_task,
            None => state.draft_task == Task::default(),
        }{
            Self::select_task(state, maybe_task);
            Message::None
        }else{
            Message::Modal(ModalMessage::Confirm((
                "Unsaved changes will be lost. Continue without saving?".to_string(),
                Box::new(Message::ForceSelectTask(maybe_task))
            )))
        }
    }

    fn select_task(state: &mut State, maybe_task: Option<Task>){
        state.selected_task = maybe_task.clone();
        state.draft_task = match maybe_task{
            Some(t) =>  t.clone(),
            None => Task::default(),
        };
    }

    fn start_timer(timer_state: &mut TimerState){
        if timer_state.num_minutes_running().is_none(){
            *timer_state = TimerState::Timing{start_time: Utc::now()};
        }
    }

    fn stop_timer(state: &mut State){
        if let Some(minutes) = state.timer_state.num_minutes_running(){
            state.draft_task.time_used += minutes;
            state.timer_state = TimerState::Stopped;
        }
    }

    fn handle_modal_message(modal_state: &mut ModalShowing, modal_message: ModalMessage) -> Message{
        match modal_message{
            ModalMessage::Ok => match &modal_state{
                ModalShowing::Confirm(_, confirmed_message) => {
                    let m = confirmed_message.clone();
                    *modal_state = ModalShowing::None; // this bypasses the update function
                    *m
                },
                _ => panic!("Should never happen"),
            },
            other => {
                *modal_state = match other{
                    ModalMessage::Confirm((string, message)) =>  ModalShowing::Confirm(string, message),
                    ModalMessage::PickNextActionDate =>          ModalShowing::NextAction,
                    ModalMessage::PickDueDate =>                 ModalShowing::DueDate,
                    ModalMessage::Close =>                       ModalShowing::None,
                    ModalMessage::Ok => panic!("will never happen"),
                };
                Message::None
            }
        }
    }

    #[must_use] fn update_draft_task(draft_task: &mut Task, message: UpdateDraftTask) -> Message{
        use UpdateDraftTask as UDT;

        match message{
            UDT::NextActionDate(next_action_date) => {
                draft_task.next_action_date = next_action_date;
                Message::Modal(ModalMessage::Close)
            },
            UDT::DueDate(due_date) => {
                draft_task.due_date = due_date;
                Message::Modal(ModalMessage::Close)
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
                        },
                        LinkMessage::Delete(idx) => {
                            draft_task.links.remove(idx);
                        },
                        LinkMessage::Update((link, idx)) => {
                            draft_task.links[idx] = link;
                        },
                    }
                }
                Message::None
            }
        }
    }

    fn mutate(state: &mut State, message: &MutateMessage) -> Command<Message>{
        Self::stop_timer(state);
        // TODO this is so stupid but it works and I got tired of hacking at Arc<>
        let db_clone1 = state.db.clone();
        let db_clone2 = state.db.clone();
        let t1 = state.draft_task.clone();
        let t2 = state.draft_task.clone();

        let (tx, rx) = oneshot::channel::<()>(); // Synchronize the writes to the database with the
                                                 // reads that update the cache
        
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
                        let t = std::mem::take(&mut state.draft_task);
                        state.selected_task = None;
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
