use std::error::Error;
use std::str::FromStr;

use tokio::runtime::Runtime;

use sqlx::sqlite::{
    SqlitePool,
    SqliteRow,
    SqliteConnectOptions,
};
use sqlx::{
    Row,
    ConnectOptions,
};

use crate::{
    Task,
    due_date::ParseDateError,
    utils::parse_date,
    DueDate,
    Schedule,
};

use chrono::{
    NaiveDate,
    Datelike,
    Local
};

use serde::{
    Serialize,
    Deserialize,
};

impl TryFrom<SqliteRow> for Task{
    type Error = ParseDateError;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(Task{
            category:                     row.get::<String, &str>("Category"),
            finished:                     row.get::<bool,   &str>("Finished"),
            name:                         row.get::<String, &str>("Name"),
            _time_budgeted:               row.get::<u32,    &str>("Budget"),
            time_needed:                  row.get::<u32,    &str>("Time"),
            time_used:                    row.get::<u32,    &str>("Used"),
            next_action_date: parse_date(&row.get::<String, &str>("NextAction"))?,
            due_date:                     row.get::<String, &str>("DueDate").try_into()?,
            notes:                        row.get::<String, &str>("Notes"),
            id:                           row.get::<Option<u32>, &str>("rowid"),
            date_added:       parse_date(&row.get::<String, &str>("DateAdded"))?,
        })
    }
}

#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
struct Province{
    id: String,
}

#[derive(Serialize, Deserialize)]
struct Holidays{
    holidays: Vec<Holiday>,
}

#[derive(Serialize, Deserialize)]
struct Holiday{
    provinces: Vec<Province>,
    observed_date: String
}

pub struct DatabaseManager{
    pool: SqlitePool,
    rt: Runtime,
}

impl DatabaseManager{
    // TODO all these methods that take Strings should take &str instead
    pub fn new(database_path: String) -> Result<Self, sqlx::Error>{
        let rt = Runtime::new().unwrap();
        Ok(Self{
            pool: rt.block_on(SqlitePool::connect(database_path.as_str()))?,
            rt,
        })
    }

    pub fn create_new_database(database_path: String) -> Result<Self, sqlx::Error>{
        let rt = Runtime::new().unwrap();
        rt.block_on(async{
            let mut conn = SqliteConnectOptions::from_str(&database_path)
                .expect("This should work")
                .create_if_missing(true)
                .connect()
                .await
                .expect("Should be able to connect to database");

            // This doesn't use query! because when creating a database, it doesn't make sense to
            // check against an existing database
            sqlx::query_file!("resources/schema.sql")
                .execute(&mut conn)
                .await
                .expect("Should be able to create the schema");
        });

        Self::new(database_path)
    }

    pub fn create_task(&self, task: &Task) -> Task{
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let due_date_str = task.due_date.to_string();
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let date_added_str = DueDate::Date(task.date_added).to_string();

        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO tasks
                    (
                        Category,
                        Finished,
                        Name,
                        Budget,
                        Time,
                        Used,
                        NextAction,
                        DueDate,
                        Notes,
                        DateAdded
                    )
                VALUES
                    (
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?
                    )
            ",
                task.category,
                task.finished,
                task.name,
                task.time_needed, // When creating a new task, save the initial time_needed estimate as time_budgeted
                task.time_needed,
                task.time_used,
                next_action_str,
                due_date_str,
                task.notes,
                date_added_str,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert Task into database")
                .last_insert_rowid();

            // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
            // return type of query! to write an impl From<T> for Task
            sqlx::query("
                SELECT *, rowid
                FROM tasks
                WHERE rowid == ?
            ")
                .bind(new_rowid)
                .fetch_one(&self.pool)
                .await
                .expect("Should have inserted and retrieved a task")
                .try_into()
                .expect("Database should contain valid Tasks only")
        })
    }

    // TODO I think this will silently fail if passed a task that is not already present in the
    // database, e.g., if someone erroneously used this where create_task() was required. Perhaps
    // this should return a Result<(), _>
    pub fn update_task(&self, task: &Task){
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let due_date_str = task.due_date.to_string();

        self.rt.block_on(async{
            sqlx::query!("
                UPDATE tasks
                SET
                    Category =    ?,
                    Finished =    ?,
                    Name =        ?,
                    Time =        ?,
                    Used =        ?,
                    NextAction =  ?,
                    DueDate =     ?,
                    Notes =       ?
                WHERE
                    rowid == ?
            ",
                task.category,
                task.finished,
                task.name,
                task.time_needed,
                task.time_used,
                next_action_str,
                due_date_str,
                task.notes,
                task.id,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to update task");
        })
    }

    // Note: this deliberately takes ownership of task, because it will be deleted afterward and
    // this prevents references to it from surviving
    pub fn delete_task(&self, task: Task){
        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM tasks
                WHERE rowid == ?
            ",
                task.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able do delete task");
        });
    }

    pub fn get_open_tasks(&self) -> Vec<Task>{
        let mut tasks: Vec<Task> = self.rt.block_on(async{
            // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
            // return type of query! to write an impl From<T> for Task
            sqlx::query("
                SELECT *, rowid
                FROM tasks
                WHERE Finished == false
                ORDER BY DueDate
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get tasks")
                .into_iter()
                .map(|r: SqliteRow| Task::try_from(r)
                     .expect("Database should hold valid Tasks")
                ).collect()
        });

        tasks.sort_by(|a,b| a.due_date.cmp(&b.due_date));

        tasks
    }

    pub fn get_categories(&self) -> Vec<String>{
        self.rt.block_on(async{
            sqlx::query!("
                SELECT DISTINCT Category
                FROM tasks
                ORDER BY Category
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get categories")
                .into_iter()
                .map(|r| r.Category.expect("Each category should be a string"))
                .collect()
        })
    }

    pub fn try_update_holidays(&self) -> Result<(), Box<dyn Error>>{
        // If database already has holidays from the current year, exit
        if self.get_holidays()
                .iter()
                .filter(|h| h.year() == Local::now().year())
                .peekable()
                .peek()
                .is_some()
        {
            return Ok(())
        }


        // If database doesn't have the holidays for this year, get them
        // and store them in the database
        let response: String = self.rt.block_on(async{
            reqwest::get("https://canada-holidays.ca/api/v1/holidays")
                .await?
                .text()
                .await
        })?;

        let holiday_dates: Vec<NaiveDate> = serde_json::from_str::<Holidays>(&response)?
            .holidays
            .iter()
            .filter(|h| h.provinces.contains(&Province{id: "BC".to_string()}))
            .map(|h| h.observed_date.parse::<NaiveDate>())
            .collect::<Result<Vec<NaiveDate>, _>>()?;

        self.rt.block_on(async{
            for d in holiday_dates{
                let date_string = d.to_string();

                sqlx::query!("
                    INSERT INTO days_off
                        (
                            Day,
                            Reason
                        )
                    VALUES
                        (
                            ?,
                            'stat_holiday'
                        )
                    ",
                    date_string
                )
                    .execute(&self.pool)
                    .await
                    .expect("Should be able to add the stat holiday");
            }
        });

        Ok(())
    }

    pub fn add_vacation_day(&self, date: &NaiveDate){
        let date_string = date.to_string();
        self.rt.block_on(async{
            sqlx::query!("
                INSERT INTO days_off
                    (
                        Day,
                        Reason
                    )
                VALUES
                    (
                        ?,
                        'vacation'
                    )
                ",
                date_string
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to add vacation day");
        });
    }

    pub fn delete_vacation_day(&self, date: &NaiveDate){
        let date_string = date.to_string();

        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM days_off
                WHERE Day == ? AND Reason=='vacation'
            ",
                date_string
            )
                .execute(&self.pool)
                .await
                .expect("Should be able do delete task");
        });
    }

    pub fn get_vacation_days(&self) -> Vec<NaiveDate>{
        self.rt.block_on(async{
            sqlx::query!("
                SELECT Day
                FROM days_off
                WHERE Reason == 'vacation'
                ORDER BY Day
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get vacation days")
                .into_iter()
                .map(|record|
                     record.Day.expect("Day is a field in days_off")
                     .parse::<NaiveDate>().expect("days_off should contain valid dates")
                )
                .collect()
        })
    }

    pub fn get_holidays(&self) -> Vec<NaiveDate>{
        self.rt.block_on(async{
            sqlx::query!("
                SELECT Day
                FROM days_off
                WHERE Reason == 'stat_holiday'
                ORDER BY Day
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get stat holidays")
                .into_iter()
                .map(|record|
                     record.Day.expect("Day is a field in days_off")
                     .parse::<NaiveDate>().expect("days_off should contain valid dates")
                )
                .collect()
        })
    }

    pub fn get_days_off(&self) -> Vec<NaiveDate> {
        let mut days_off = Vec::new();

        self.try_update_holidays().unwrap();
        days_off.append(&mut self.get_holidays());
        days_off.append(&mut self.get_vacation_days());

        days_off
    }

    pub fn get_schedule(&self) -> Schedule{
        Schedule::new(
            self.get_days_off(),
            self.get_open_tasks(),
        )
    }
}
