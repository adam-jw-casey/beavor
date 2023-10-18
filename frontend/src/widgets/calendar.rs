use iced::widget::{
    Column,
    Row,
    column,
    text,
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
};

use backend::{
    utils::today_date,
    Schedule,
};

use crate::Message;

// TODO this needs day-of-week headers
pub fn calendar(schedule: &Schedule) -> Element<'static, Message>{
    // Get the days of the week that contains the passed day
    fn week_of(d: NaiveDate) -> Vec<NaiveDate>{
        let w = d.week(Weekday::Mon);

        // Monday to Friday
        (0..=4)
            .map(|n| w.first_day() + Days::new(n))
            .collect()
    }

    let today = today_date();

    let num_weeks = 4;
    Row::with_children(
        week_of(today)
            .iter()
            .map(|d| Column::with_children(
                (0..num_weeks)
                    .map(|n| *d + Days::new(7*n))
                    .map(|d| Element::from(cal_day(d, schedule.workload_on_day(d)).padding(4)))
                    .collect()
                ).into()
            ).collect()
    )
        .width(Length::Shrink)
        .height(Length::Shrink)
        .padding(8)
        .into()
}

fn cal_day(day: NaiveDate, load: Option<u32>) -> Column<'static, Message>{
    column![
        text(
            day.format("%b %d")
        ),
        text(
            if let Some(load) = load {format!("{:.1}", f64::from(load)/60.0)} else{"-".to_string()}
        )
    ]
        .padding(4)
        .align_items(Alignment::Center)
}
