use iced::widget::{
    container,
    row,
};

use iced::{
    Sandbox,
    Element,
    Settings,
};

use chrono::NaiveDate;

use backend::{
    DatabaseManager,
    Task,
};

mod widgets;
use widgets::{
    Calendar,
    TaskScroller,
    TaskEditor,
};

use widgets::task_editor::UpdateDraftTask;

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
pub enum Message{
    SelectTask(Option<Task>),
    SelectDate(Option<NaiveDate>),
    UpdateDraftTask(UpdateDraftTask),
    SaveDraftTask,
}

struct Beavor{
    db:            DatabaseManager,
    selected_task: Option<Task>, // TODO should this maybe just store task ID to minimize the state
                                 // lying around?
    selected_date: Option<NaiveDate>,
    draft_task: Option<Task>,
    
}

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        let db = DatabaseManager::new("worklist.db".into());
        Self{
            db,
            selected_task: None,
            selected_date: None,
            draft_task:    None,
        }
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) {
        match message{
            Message::SelectTask(maybe_task) => {
                self.selected_task = maybe_task.clone();
                // Don't overwrite an existing draft task
                match self.draft_task{
                    Some(_) => println!("Refusing to overwrite draft task"), // TODO allow case
                                                                             // where the draft
                                                                             // task is unmodified
                                                                             // -> maybe a
                                                                             // DraftTask type with
                                                                             // that metadata?
                    None => self.draft_task = maybe_task.clone(),
                }
                 
            },
            Message::SelectDate(maybe_date) => self.selected_date = maybe_date,
            Message::UpdateDraftTask(task_field_update) => if let Some(t) = self.draft_task.as_mut(){
                use UpdateDraftTask as UDT;
                match task_field_update{
                    UDT::Category(category) => t.category = category,
                    UDT::Name(name) => t.name = name,
                    UDT::TimeNeeded(time_needed) => t.time_needed = time_needed,
                    UDT::TimeUsed(time_used) => t.time_used = time_used,
                    UDT::NextActionDate(next_action_date) => t.next_action_date = next_action_date,
                    UDT::DueDate(due_date) => t.due_date = due_date,
                    UDT::Notes(notes) => t.notes = notes,
                }
            },
            Message::SaveDraftTask => todo!(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = row![
            TaskScroller(&self.db.get_open_tasks()),
            TaskEditor(self.draft_task.as_ref()),
            Calendar(&self.db.get_schedule()),
        ].into();

        container(content).into()
    }
}
