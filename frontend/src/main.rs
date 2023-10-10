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
use widgets::Calendar;
use widgets::TaskScroller;

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
pub enum Message{
    SelectTask(Task),
    DeselectTask,
    SelectDate(NaiveDate),
    DeselectDate,
}

struct Beavor{
    db: DatabaseManager,
    selected_task: Option<Task>,
    selected_date: Option<NaiveDate>,
}

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        let db = DatabaseManager::new("worklist.db".into());
        Self{
            db,
            selected_task: None,
            selected_date: None,
        }
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) {
        match message{
            Message::SelectTask(task) => self.selected_task = Some(task),
            Message::DeselectTask     => self.selected_task = None,
            Message::SelectDate(date) => self.selected_date = Some(date),
            Message::DeselectDate     => self.selected_date = None,
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = row![
            TaskScroller(&self.db.get_open_tasks()),
            Calendar(&self.db.get_schedule()),
        ].into();

        container(content).into()
    }
}
