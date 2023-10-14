#![warn(clippy::pedantic)]
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
    DateTime,
};

use backend::{
    DatabaseManager,
    Task,
};

mod widgets;
use widgets::{
    calendar,
    task_scroller,
    task_editor::task_editor,
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
    Refresh(()),
    Tick(Instant),
    SelectTask(Option<Task>),
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
struct State{
    db:            DatabaseManager,
    selected_task: Option<Task>, // TODO should this maybe just store task ID to minimize the state
                                 // lying around?
    selected_date: Option<NaiveDate>,
    draft_task: Task,
    timer_start_utc: Option<DateTime<Utc>>, // This should really be a custom enum like TimerState
                                            // or somesuch
                                            // It should also have a method to compute the time
                                            // since started, since this is done in several places
    next_action_date_picker_showing: bool,
    due_date_picker_showing: bool,
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
            Command::perform(async{State{
                db: match DatabaseManager::new("worklist.db").await{
                    Ok(db) => db,
                    Err(_) => DatabaseManager::with_new_database("worklist.db").await.expect("Should be able to create database"),
                },
                selected_task: None,
                selected_date: None,
                draft_task:    Task::default(),
                timer_start_utc: None,
                next_action_date_picker_showing: false,
                due_date_picker_showing: false,
            }}, Message::Loaded),
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
                    Message::Loaded(state) => {*self = Self::Loaded(state); Command::none()},
                    _ => panic!("Should never happen")
                }
            },
            Beavor::Loaded(state) => match message{
                Message::Mutate(mutate) => Command::perform(async {
                    match mutate{
                        Mutate::SaveDraftTask => {
                            let t = &state.draft_task;
                            match t.id{
                                Some(_) => state.db.update_task(t).await,
                                None => state.draft_task = state.db.create_task(t).await,
                            }
                            state.selected_task = Some(state.draft_task.clone());
                        },
                        Mutate::DeleteTask => {
                            let t = std::mem::take(&mut state.draft_task);
                            state.db.delete_task(&t).await;
                            state.selected_task = None;
                            let _ = self.update(Message::NewTask);
                        },
                    }
                }, Message::Refresh),
                other => {match other{
                    Message::Mutate(_) => panic!("Unreachable"),
                    Message::NewTask => {let _ = self.update(Message::SelectTask(None));},
                    Message::SelectTask(maybe_task) => {
                        // Don't overwrite an existing draft task
                        if (state.selected_task.is_none() && state.draft_task == Task::default()) || state.draft_task == *state.selected_task.as_ref().unwrap(){
                            state.selected_task = maybe_task.clone();
                            state.draft_task = match maybe_task{
                                Some(t) =>  t.clone(),
                                None => Task::default(),
                            }
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
                    Message::ToggleTimer => match state.timer_start_utc{
                        Some(timer_start_utc) => {
                            state.draft_task.time_used += u32::try_from((Utc::now() - timer_start_utc).num_minutes()).expect("This will be positive and small enough to fit");
                            state.timer_start_utc = None;
                            let _ = self.update(Message::Mutate(Mutate::SaveDraftTask));
                        },
                        None => state.timer_start_utc = Some(Utc::now()),
                    },
                    Message::Tick(_) => {},
                    Message::Refresh(_) => {},
                    Message::PickNextActionDate => state.next_action_date_picker_showing = true,
                    Message::CancelPickNextActionDate => state.next_action_date_picker_showing = false,
                    Message::PickDueDate => state.due_date_picker_showing = true,
                    Message::CancelPickDueDate => state.due_date_picker_showing = false,
                    Message::Loaded(state) => panic!("Should never happen"),
                }Command::none()}
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        
        let content: Element<Message> = match self{
            Beavor::Loading => text("Loading...").into(),
            Beavor::Loaded(state) => 
                row![
                    task_scroller(&state.db.open_tasks().await)
                        .width(Length::FillPortion(2))
                        .height(Length::FillPortion(1)),
                    task_editor(&state.draft_task, state.timer_start_utc.as_ref(), state.next_action_date_picker_showing, state.due_date_picker_showing)
                        .padding(8)
                        .width(Length::FillPortion(3))
                        .height(Length::FillPortion(1)),
                    calendar(&state.db.schedule().await),
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
