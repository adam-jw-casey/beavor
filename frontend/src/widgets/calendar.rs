use iced::widget::{
    Column,
    Row,
    column,
    text,
};

use iced::{
    Element,
    Length,
};

use chrono::{
    Weekday,
    NaiveDate,
    naive::Days,
};

use backend::{
    utils::today_date,
    Schedule,
};

use crate::Message;

#[allow(non_snake_case)]
pub fn Calendar(schedule: &Schedule) -> Element<'static, Message>{
    // TODO this is a pretty grungy implementation, but it should do for now
    // Get the days of the week that contains the passed day
    fn week_of(d: NaiveDate) -> Vec<NaiveDate>{
        let w = d.week(Weekday::Mon);

        vec![
            w.first_day(),
            w.first_day() + Days::new(1),
            w.first_day() + Days::new(2),
            w.first_day() + Days::new(3),
            w.first_day() + Days::new(4),
        ]
    }

    let today = today_date();

    // TODO this is a pretty grungy implementation, but it should do for now
    // Show this week and the following ones
    Column::with_children(
        [
        today,
        today + Days::new(7),
        today + Days::new(14),
        today + Days::new(21),
        ]
            .iter()
            .map(|d| Row::with_children(
                week_of(*d).iter().map(
                    |d| Element::from(CalDay(*d, schedule.workload_on_day(*d)).padding(4))
                ).collect()
            ).into())
            .collect()
    )
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
}

#[allow(non_snake_case)]
fn CalDay(day: NaiveDate, load: u32) -> Column<'static, Message>{
    column![
        text(
            day.format("%b %d")
        ),
        text(
            format!("{:.1}", load as f32/60.0)
        )
    ]
}
