use iced::{
    Element,
    Length,
};

use iced::widget::{
    text,
    row,
    Row,
    button,
};

use iced_aw::{
    Card,
    modal,
};

use crate::{
    Message,
    ModalMessage,
    ModalType,
};

pub fn confirm_modal<'a>(state: &ModalType) -> Element<'a, Message>{
    let underlay = row![];

    let overlay = match state {
        ModalType::Confirm(confirmation_request) => {
            Some(
                Card::new(
                    text("Confirm:"),
                    text(confirmation_request.message.clone()),
                )
                .foot(
                    Row::new()
                    .spacing(10)
                    .padding(5)
                    .width(Length::Fill)
                    .push(
                        button(text("Cancel"))
                            .width(Length::Fill)
                            .on_press(Message::Modal(ModalMessage::Show(ModalType::None))),
                    )
                    .push(
                        button("Ok")
                            .width(Length::Fill)
                            .on_press(Message::Modal(ModalMessage::Ok)),
                    ),
                )
                    .max_width(300.0)
                    .on_close(Message::Modal(ModalMessage::Show(ModalType::None))),
            )
        },
        _ => None,
    };

    modal(underlay, overlay)
        .backdrop(Message::Modal(ModalMessage::Show(ModalType::None)))
        .on_esc(Message::Modal(ModalMessage::Show(ModalType::None)))
        .into()
}
