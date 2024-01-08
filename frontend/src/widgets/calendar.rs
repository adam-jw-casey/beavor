use iced::widget::{
    Column,
    row,
    Row,
    column,
    button,
    text,
    MouseArea,
};

use iced::{
    Element,
    Length,
    Alignment,
};

use chrono::{
    Weekday,
    NaiveDate,
    naive::Days,
    Duration,
    Datelike,
};

use backend::{
    utils::today_date,
    Schedule,
};

use iced_aw::{
    Icon,
    graphics::icons::{
        icon_to_char,
        ICON_FONT,
    },
};

use crate::Message as MessageWrapper;
use crate::MutateMessage;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ScrollDown,
    ScrollUp,
    ScrollUpMax,
    FilterToDate(Option<NaiveDate>), //TODO I have a feeling I'll want more filters at some point
    ClickDate(Option<NaiveDate>),
}

#[derive(Debug, Clone, Default)]
pub struct State {
    weeks_scrolled: u16,
    pub clicked_date: Option<NaiveDate>,
    pub filter_date: Option<NaiveDate>,
}

impl State {
    fn scroll_down(&mut self) {
        self.weeks_scrolled += 1;
    }

    fn scroll_up(&mut self) {
        self.weeks_scrolled = self.weeks_scrolled.saturating_sub(1);
    }

    fn scroll_up_max(&mut self) {
        self.weeks_scrolled = 0;
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ScrollDown         => self.scroll_down(),
            Message::ScrollUp           => self.scroll_up(),
            Message::ScrollUpMax        => self.scroll_up_max(),
            Message::FilterToDate(date) => self.filter_date = date,
            Message::ClickDate(d)       => self.clicked_date = d,
        }
    }
}

pub fn calendar(schedule: &Schedule, state: &State) -> Element<'static, MessageWrapper> {
    // Get the days of the week that contains the passed day
    fn week_of(d: NaiveDate) -> Vec<NaiveDate> {
        let w = d.week(Weekday::Mon);

        // TODO this should respect the actual schedule that's loaded in
        // Monday to Friday
        (0..=4)
            .map(|n| w.first_day() + Days::new(n))
            .collect()
    }

    let today = today_date();

    let num_weeks = 4;
    row![
        Row::with_children(
            week_of(today + Days::new((7*state.weeks_scrolled).into()))
                .iter()
                .map(|d| column![
                     text(d.weekday()),
                     // TODO would be nice to throw a Rule::horizontal(4) in here, but it wants to
                     // be wide and I haven't taken the time to figure out how to fix that
                     Column::with_children(
                        (0..num_weeks)
                            .map(|n| *d + Days::new(7*n))
                            .map(|d| Element::from(cal_day(
                                d,
                                schedule.get_time_assigned_on_day(d),
                                Some(d) == state.filter_date,
                                state.clicked_date.as_ref(),
                                state.filter_date.as_ref(),
                            )))
                            .collect()
                        )
                    ]
                        .align_items(Alignment::Center)
                        .into()
                ).collect()
        )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(8),
            column![
                button(text(icon_to_char(Icon::ChevronDoubleUp)).font(ICON_FONT))
                    .on_press_maybe(if state.weeks_scrolled > 0 {Some(MessageWrapper::Calendar(Message::ScrollUpMax))}else {None}),
                button(text(icon_to_char(Icon::ChevronUp)).font(ICON_FONT))
                    .on_press_maybe(if state.weeks_scrolled > 0 {Some(MessageWrapper::Calendar(Message::ScrollUp))}else {None}),
                button(text(icon_to_char(Icon::ChevronDown)).font(ICON_FONT))
                    .on_press(MessageWrapper::Calendar(Message::ScrollDown)),
            ].height(Length::Shrink)
    ]
        .align_items(Alignment::Center)
        .into()
}

fn cal_day(day: NaiveDate, load: Option<Duration>, is_selected: bool, clicked_date: Option<&NaiveDate>, filter_date: Option<&NaiveDate>) -> Element<'static, MessageWrapper> {
    MouseArea::new(
        column![
            text(
                if is_selected {
                    day.format("[%b %d]")
                }else {
                    day.format("%b %d")
                }
            ),
            text(
                // Casting to f64 from i64 reduces precision from 64 to 52 bits.
                // This is ok because the number of minutes to work on a day will never occupy 52 bits
                #[allow(clippy::cast_precision_loss)]
                if let Some(load) = load {format!(" {:.1}", load.num_minutes() as f64/60.0)} else {"-".to_string()}
            )
        ]
            .padding(4)
            .align_items(Alignment::Center)
    )
        .on_press(MessageWrapper::Calendar(Message::ClickDate(Some(day))))
        .on_release(
            if clicked_date.is_some_and(|d| *d == day) {
                // TODO what I MEAN here is "toggle" but there's no way to express that in a message at the moment
                if filter_date.is_some_and(|d| *d == day) {
                    MessageWrapper::Calendar(Message::FilterToDate(None))
                }else {
                    MessageWrapper::Calendar(Message::FilterToDate(Some(day)))
                }
            }else {
                MessageWrapper::Calendar(Message::ClickDate(None))
            }
        )
        .on_right_press(
             MessageWrapper::Mutate(MutateMessage::VacationStatus(day, load.is_some()))
        )
        .into()
}
