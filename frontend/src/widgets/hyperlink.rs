use iced::widget::{
    button,
    text,
    row,
    text_input,
};

use iced::{
    Element,
    Length,
};

use crate::Message;
use crate::widgets::task_editor::{UpdateDraftTask, LinkMessage};

use backend::Hyperlink;

// TODO some sort of URI scheme handling would be nice here, so the user doesn't have to type it in
// manually, e.g., file://
pub fn hyperlink <'a>(link: &Hyperlink, link_idx: usize, editing_idx: Option<usize>) -> Element<'a, Message>{

    // This is janky but it works?
    let display_link = link.clone();
    let url_link = link.clone();

    if editing_idx.is_some_and(|e_idx| e_idx == link_idx){
        row![
            text_input("name...", &link.display).on_input(move |s: String| {
                let mut new_link = display_link.clone();
                new_link.display = s;
                Message::UpdateDraftTask(UpdateDraftTask::Link(LinkMessage::Update((new_link, link_idx)))
            )}).width(Length::FillPortion(2)),
            text_input("url...", &link.url).on_input(move |s: String| {
                let mut new_link = url_link.clone();
                new_link.url = s;
                Message::UpdateDraftTask(UpdateDraftTask::Link(LinkMessage::Update((new_link, link_idx)))
            )}).width(Length::FillPortion(2)),
            button(text("Done"))
                .on_press(Message::SetEditingLinkID(None))
                .width(Length::Shrink),
        ]
            .spacing(4)
            .into()
    }else{
        row![
            button(text(&link.display))
                .on_press(Message::Open(link.url.to_string()))
                .width(Length::FillPortion(4)),
            button(text("Edit"))
                .on_press(Message::SetEditingLinkID(Some(link_idx)))
                .width(Length::FillPortion(1)),
            button(text("Delete"))
                .on_press(Message::UpdateDraftTask(UpdateDraftTask::Link(LinkMessage::Delete(link_idx))))
                .width(Length::FillPortion(1)),
        ]
            .spacing(4)
            .into()
    }
}
