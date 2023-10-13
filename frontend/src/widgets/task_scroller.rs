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

#[allow(non_snake_case)]
pub fn TaskScroller(tasks: &[Task]) -> Scrollable<'static, Message>{
    scrollable(
        Column::with_children(
            tasks
                .iter()
                .map(TaskRow)
                .collect()
        )
            .width(Length::Shrink) // TODO make each row take a consistent width
            .padding([40, 0, 40, 0])
    )
}

#[allow(non_snake_case)]
fn TaskRow(task: &Task) -> Element<'static, Message>{
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
