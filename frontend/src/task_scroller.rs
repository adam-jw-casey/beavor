use iced::widget::{
    Column,
    scrollable,
    text,
    Button,
};

use iced::{
    Element,
    Length,
};

use backend::Task;

use crate::Message;

#[allow(non_snake_case)]
pub fn TaskScroller(tasks: &[Task]) -> Element<'static, Message>{
    scrollable(
        Column::with_children(
            tasks
                .iter()
                .map(TaskRow)
                .collect()
        )
            .width(Length::Shrink)
            .padding([40, 0, 40, 0])
    ).into()
}

#[allow(non_snake_case)]
fn TaskRow(task: &Task) -> Element<'static, Message>{
    Button::new(
        text(&task.name)
    )
        .on_press(Message::SelectTask(task.clone()))
        .into()
}
