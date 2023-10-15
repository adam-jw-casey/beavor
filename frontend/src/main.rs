#![warn(clippy::pedantic)]
use std::sync::Arc;

use iced::widget::{
    container,
    row,
    text,
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
};

use chrono::{
    NaiveDate,
    offset::Utc,
};

use backend::{
    DatabaseManager,
    Task,
    Schedule,
};

mod widgets;
use widgets::{
    calendar,
    task_scroller,
    task_editor::{
        task_editor,
        TimerState,
        DatePickerState,
    },
};

use widgets::task_editor::UpdateDraftTask;

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
pub enum Mutate{
    SaveDraftTask,
    DeleteTask,
}

#[derive(Debug, Clone)]
pub enum Message{
    Refresh(Cache),
    Tick(Instant),
    SelectTask(Option<Task>),
    TrySelectTask(Option<Task>),
    SelectDate(Option<NaiveDate>),
    UpdateDraftTask(UpdateDraftTask),
    ToggleTimer, // Consider having seperate start/stop/toggle messages
    PickNextActionDate,
    CancelPickNextActionDate,
    PickDueDate,
    CancelPickDueDate,
    NewTask,
    Mutate(Mutate),
    Loaded(State),
    None(()),
}

// TODO need a better way of keeping track of whether the shown task:
//          a. Already exists in the database, as shown,
//          b. Exists in the database, but is shown with unsaved user modifications,
//          c. Is the default, placeholder task, or
//          d. Is a new task with some or all information already entered
//  TODO Whatever type draft_task ends of having, it should have a take/replace/swap like in
//       std::mem to get an owned copy and overwrite the original in an "atomic" operation, to
//       help ease state management
#[derive(Debug, Clone)]
pub struct State{
    db:            Arc<DatabaseManager>,
    selected_task: Option<Task>,
    selected_date: Option<NaiveDate>,
    draft_task: Task,
    timer_state: TimerState,
    date_picker_state: DatePickerState,
    cache: Cache,
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

    // TODO database path should be a flag
    fn new(_flags: Self::Flags) -> (Beavor, iced::Command<Message>) {
        (
            Self::Loading,
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
                    date_picker_state: DatePickerState::None,
                    cache: Cache{
                        loaded_tasks: db.open_tasks().await.into(),
                        loaded_schedule: db.schedule().await,
                    },
                    db: db.into(),
                }
            }, Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    // TODO break each match case out into seperate functions (or at least into groups). This is getting ridiculous.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self{
            Beavor::Loading => {
                match message{
                    Message::Loaded(state) => {
                        *self = Self::Loaded(state);
                        Command::none()
                    },
                    _ => panic!("Should never happen")
                }
            },
            Beavor::Loaded(state) => match message{
                Message::Mutate(mutate) => {
                    // TODO this is so stupid but it works and I got tired of hacking at Arc<>
                    let db_clone1 = state.db.clone();
                    let db_clone2 = state.db.clone();
                    let t1 = state.draft_task.clone();
                    let t2 = state.draft_task.clone();
                    Command::batch(
                    [
                        match mutate{
                            Mutate::SaveDraftTask => match t1.id{
                                Some(_) => Command::perform(async move {
                                    db_clone1.update_task(&t1).await;
                                }, |()| Message::SelectTask(Some(t2))),
                                None => Command::perform(async move {
                                    db_clone1.create_task(&t1).await;
                                }, |()| Message::SelectTask(Some(t2))),
                            },
                            Mutate::DeleteTask => {
                                let t = std::mem::take(&mut state.draft_task);
                                state.selected_task = None;
                                Command::perform(async move {
                                    db_clone1.delete_task(&t).await;
                                }, |()| Message::NewTask)
                            }
                        },
                        Command::perform(async move {Cache{
                            loaded_tasks:    db_clone2.open_tasks().await.into(),
                            loaded_schedule: db_clone2.schedule().await,
                        }}, Message::Refresh)
                    ]
                )},
                other => {match other{
                    Message::Mutate(_) => panic!("Unreachable"),
                    Message::NewTask => {let _ = self.update(Message::SelectTask(None));},
                    Message::SelectTask(maybe_task) => {
                        state.selected_task = maybe_task.clone();
                        state.draft_task = match maybe_task{
                            Some(t) =>  t.clone(),
                            None => Task::default(),
                        }
                    },
                    Message::TrySelectTask(maybe_task) => {
                        // Don't overwrite an existing draft task
                        if (state.selected_task.is_none() && state.draft_task == Task::default()) || state.selected_task.as_ref().is_some_and(|t| *t == state.draft_task){
                            let _ = self.update(Message::SelectTask(maybe_task));
                        }else{
                            println!("Refusing to overwrite draft task"); // TODO handle this case elegantly
                        }
                    },
                    Message::SelectDate(maybe_date) => state.selected_date = maybe_date,
                    Message::UpdateDraftTask(task_field_update) => {
                        use UpdateDraftTask as UDT;

                        let t = &mut state.draft_task;
                        match task_field_update{
                            UDT::Category(category) => t.category = category,
                            UDT::Name(name) => t.name = name,
                            UDT::TimeNeeded(time_needed) => if let Ok(time_needed) = time_needed {t.time_needed = time_needed},
                            UDT::TimeUsed(time_used) => if let Ok(time_used) = time_used {t.time_used = time_used},
                            UDT::NextActionDate(next_action_date) => {
                                t.next_action_date = next_action_date;
                                let _ = self.update(Message::CancelPickNextActionDate);
                            },
                            UDT::DueDate(due_date) => {
                                t.due_date = due_date;
                                let _ = self.update(Message::CancelPickDueDate);
                            },
                            UDT::Notes(notes) => t.notes = notes,
                            UDT::Finished(finished) => t.finished = finished,
                        }
                    },
                    Message::ToggleTimer => match state.timer_state.time_running(){
                        Some(duration) => {
                            state.draft_task.time_used += u32::try_from(duration.num_minutes()).expect("This will be positive and small enough to fit");
                            state.timer_state = TimerState::Stopped;
                            let _ = self.update(Message::Mutate(Mutate::SaveDraftTask));
                        },
                        None => state.timer_state = TimerState::Timing{start_time: Utc::now()},
                    },
                    Message::Tick(_) | Message::None(()) => {},
                    Message::Refresh(cache) => state.cache = cache,
                    // TODO These three should be grouped together somehow maybe a Modal type
                    // message? so only one modal at a time can ever be showing?
                    Message::PickNextActionDate => state.date_picker_state = DatePickerState::NextAction,
                    Message::CancelPickNextActionDate | Message::CancelPickDueDate => state.date_picker_state = DatePickerState::None,
                    Message::PickDueDate => state.date_picker_state = DatePickerState::DueDate,
                    Message::Loaded(_) => panic!("Should never happen"),
                }Command::none()}
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = match self{
            Beavor::Loading => text("Loading...").into(),
            Beavor::Loaded(state) => 
                row![
                    task_scroller(&state.cache.loaded_tasks)
                        .width(Length::FillPortion(2))
                        .height(Length::FillPortion(1)),
                    task_editor(
                        &state.draft_task,
                        &state.timer_state,
                        &state.date_picker_state,
                    )
                        .padding(8)
                        .width(Length::FillPortion(3))
                        .height(Length::FillPortion(1)),
                    calendar(&state.cache.loaded_schedule),
                ]
                    .align_items(Alignment::End)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
        };

        container(content).into()
    }

    fn subscription(&self) -> Subscription<Self::Message>{
        iced::time::every(iced::time::Duration::from_secs(1)).map(Message::Tick)
    }
}
