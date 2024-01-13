use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use crate::due_date::DueDate;
use crate::task::Id;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Milestone {
    name:     String,
    category: String,
    due_date: DueDate,
    id:       Id,
    finished: bool,
}

impl TryFrom<SqliteRow> for Milestone {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(
            Self {
                name:     row.get::<String,      &str>("Name"),
                category: row.get::<String,      &str>("Category"),
                due_date: row.get::<&str,        &str>("DueDate").try_into()?,
                id:       row.get::<Option<u32>, &str>("Id"),
                finished: row.get::<bool,        &str>("Finished"),
            }
        )
    }
}

impl Milestone {
    pub fn get_id(&self) -> Id {
        self.id
    }
}
