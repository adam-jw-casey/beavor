use iced::widget::{
    Column,
    container,
    row,
    scrollable,
    text,
};

use iced::{
    Sandbox,
    Element,
    Settings,
    Length,
};

use backend::{DatabaseManager, Task};

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
enum Message{
}

struct Beavor{
    db: DatabaseManager,
    selected_task: Option<usize>,
}

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        Self{
            db: DatabaseManager::new("worklist.db".into()),
            selected_task: None,
        }
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) {
        match message{
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let task_scroller: Element<Message> = TaskScroller::new(self.db.get_open_tasks());

        let content: Element<Message> = row![task_scroller].into();

        container(content).into()
    }
}

struct TaskScroller;

impl TaskScroller{
    #[allow(clippy::new_ret_no_self)]
    fn new(tasks: Vec<Task>) -> Element<'static, Message>{
        scrollable(
            Column::with_children(
                tasks
                    .iter()
                    .map(TaskRow::new)
                    .collect()
            )
                .width(Length::Fill)
                .padding([40, 0, 40, 0])
        ).into()
    }
}

struct TaskRow;

impl TaskRow{
    #[allow(clippy::new_ret_no_self)]
    fn new(task: &Task) -> Element<'static, Message>{
        text(&task.name).into()
    }
}
