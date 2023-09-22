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

use backend::DatabaseManager;

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
enum Message{
}

struct Beavor{
    db: DatabaseManager,
}

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        Self{
            db: DatabaseManager::new("worklist.db".into()),
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
        let task_scroller: Element<Message> =
            scrollable(
                Column::with_children(
                    self.db.get_open_tasks()
                        .iter()
                        .map(|t| text(&t.name).into())
                        .collect()
                )
                    .width(Length::Fill)
                    .padding([40, 0, 40, 0])
            ).into();

        let content: Element<Message> = row![task_scroller].into();

        container(content).into()
    }
}
