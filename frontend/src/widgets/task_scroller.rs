use iced::widget::{
    Column,
    column,
    scrollable,
    Scrollable,
    text,
    Button,
};

use iced::{
    Element,
    Length,
};

use backend::Task;

use crate::Message;

pub fn task_scroller(tasks: &[Task]) -> Scrollable<'static, Message>{
    scrollable(
        Column::with_children(
            tasks
                .iter()
                .map(task_row)
                .collect()
        )
            .width(Length::Shrink)
            .spacing(2)
            .padding(4)
    )
        .height(Length::Fill)
}

fn task_row(task: &Task) -> Element<'static, Message>{
    Button::new(
        column![
            text(&task.name),
            text(&task.category),
        ]
    )
        .on_press(Message::SelectTask(Some(task.clone())))
        .width(Length::Fill)
        .into()
}
