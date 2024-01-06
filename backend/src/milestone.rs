use chrono::NaiveDate;

use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use crate::due_date::DueDate;
use crate::task::Id;

use crate::utils::format_date_borrowed;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Milestone {
    // TODO I don't like that this duplicates the data of task.next_action_date and task.due_date
    AnonymousStart(NaiveDate),
    AnonymousEnd(DueDate),
    Concrete {
        name:     String,
        category: String,
        due_date: DueDate,
        id:       Id,
        finished: bool,
    },
}

impl Milestone {
    #[must_use] fn get_name(&self) -> String {
        match self {
            Milestone::AnonymousStart(date) => format_date_borrowed(date),
            Milestone::AnonymousEnd(due_date) => due_date.into(),
            Milestone::Concrete { name, category: _, due_date: _, id: _, finished: _ } => name.into(),
        }
    }
}

impl TryFrom<SqliteRow> for Milestone {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(
            Milestone::Concrete {
                name:     row.get::<String,      &str>("Name"),
                category: row.get::<String,      &str>("Category"),
                due_date: row.get::<String,      &str>("DueDate").try_into()?,
                id:       row.get::<Option<u32>, &str>("Id"),
                finished: row.get::<bool,        &str>("Finished"),
            }
        )
    }
}
