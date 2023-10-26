use std::error::Error;
use std::str::FromStr;

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
    Hyperlink,
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
            id:                           row.get::<Option<u32>, &str>("TaskID"),
            date_added:       parse_date(&row.get::<String, &str>("DateAdded"))?,
            links:                        Vec::new(),
        })
    }
}

impl TryFrom<SqliteRow> for Hyperlink{
    type Error = std::convert::Infallible;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(Hyperlink{
            url:     row.get::<String, &str>("Url"),
            display: row.get::<String, &str>("Display"),
            id:      row.get::<u32, &str>("rowid") as usize,
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

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Holiday{
    provinces: Vec<Province>,
    observedDate: String
}

#[derive(Debug, Clone)]
pub struct Connection{
    pool: SqlitePool,
}

// TODO these methods should selectively take a &mut self when they modify the database, with some
// sort of "dirty" flag. This would both allow caching (for a slight performance boost) AND let the
// borrow checker monitor the state of the database itself, not just the connection to it
impl Connection{
    /// # Errors
    /// Will fail a connection to the database cannot be established. This is generally if the
    /// database file does not exist or is corrupted
    pub async fn new(database_path: &str) -> Result<Self, sqlx::Error>{
        let pool = SqlitePool::connect(database_path).await?;

        sqlx::query!("PRAGMA foreign_keys=ON").execute(&pool).await?;

        Ok(Self{
            pool,
        })
    }

    /// # Errors
    /// Will fail a connection to the database cannot be established. This is generally if the
    /// database file does not exist or is corrupted
    pub async fn with_new_database(database_path: &str) -> Result<Self, sqlx::Error>{
        let mut conn = SqliteConnectOptions::from_str(database_path)?
            .create_if_missing(true)
            .connect()
            .await?;

        // This doesn't use query! because when creating a database, it doesn't make sense to
        // check against an existing database
        sqlx::query_file!("resources/schema.sql")
            .execute(&mut conn)
            .await?;

        Self::new(database_path).await
    }

    /// # Panics
    /// Panics if any database queries fail, or if the database contains invalid tasks
    pub async fn create_task(&self, task: &Task) -> Task{
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let due_date_str = task.due_date.to_string();
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let date_added_str = DueDate::Date(task.date_added).to_string();

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

        for h in &task.links{
            sqlx::query!("
                INSERT INTO hyperlinks (Url, Display, Task)
                VALUES(?,?,?)
            ",
                h.url,
                h.display,
                new_rowid,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert new link");
        }

        // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
        // return type of query! to write an impl From<T> for Task
        sqlx::query("
            SELECT *
            FROM tasks
            WHERE TaskID == ?
        ")
            .bind(new_rowid)
            .fetch_one(&self.pool)
            .await
            .expect("Should have inserted and retrieved a task")
            .try_into()
            .expect("Database should contain valid Tasks only")
    }

    /// # Panics
    /// Panics if any database queries fail
    /// # Errors
    /// This returns a `RowNotFound` error if the update step fails to update any rows.
    /// This indicated that no task with an id matching the passed task exists in the database
    pub async fn update_task(&self, task: &Task) -> Result<(), sqlx::Error>{
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let due_date_str = task.due_date.to_string();

        if sqlx::query!("
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
                TaskID == ?
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
            .expect("Should be able to update task")
            .rows_affected() != 1{
                return Err(sqlx::Error::RowNotFound)
            }

        sqlx::query!("
            DELETE FROM hyperlinks
            WHERE Task == ?
        ",
            task.id,
        )
            .execute(&self.pool)
            .await
            .expect("Should be able to delete links");

        // TODO this is duplicated in create_task()
        for h in &task.links{
            sqlx::query!("
                INSERT INTO hyperlinks (Url, Display, Task)
                VALUES(?,?,?)
            ",
                h.url,
                h.display,
                task.id,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert new link");
        }
        Ok(())
    }

    /// Note: this deliberately takes ownership of task, because it will be deleted afterward and
    /// this prevents references to it from surviving
    /// # Panics
    /// Panics if any database queries fail
    pub async fn delete_task(&self, task: &Task){
        // Note that hyperlinks are ON DELETE CASCADE, so do not need to be deleted manually
        sqlx::query!("
            DELETE
            FROM tasks
            WHERE TaskID == ?
        ",
            task.id
        )
            .execute(&self.pool)
            .await
            .expect("Should be able do delete task");
    }

    /// # Panics
    /// Panics if any database queries fail, or if the database contains invalid tasks or invalid
    /// links
    pub async fn open_tasks(&self) -> Vec<Task>{
        
        // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
        // return type of query! to write an impl From<T> for Task
        let mut tasks: Vec<Task> = sqlx::query("
            SELECT *
            FROM tasks
            WHERE Finished == false
            ORDER BY DueDate
        ")
            .fetch_all(&self.pool)
            .await
            .expect("Should be able to get tasks")
            .into_iter()
            .map(|r: SqliteRow| Task::try_from(r).expect("Database should hold valid Tasks"))
            .collect();

        #[allow(clippy::explicit_iter_loop)]
        for task in tasks.iter_mut(){
            task.links = sqlx::query("
                SELECT *, rowid
                FROM hyperlinks
                WHERE Task == ?
             ")
                .bind(task.id)
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get hyperlinks for task")
                .into_iter()
                .map(|r: SqliteRow| Hyperlink::try_from(r).expect("Database should hold valid links"))
                .collect();
        }

        tasks.sort_by(|a,b| a.due_date.cmp(&b.due_date));

        tasks
    }

    /// # Panics
    /// Panics if any database queries fail, or if any row has a non-string category
    #[allow(non_snake_case)]
    pub async fn categories(&self) -> Vec<String>{
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
    }

    /// # Panics
    /// Panics if any database queries fail
    ///
    /// # Errors
    /// Returns an error if unable to connect to the holiday server or unable to parse the response
    pub async fn try_update_holidays(&self) -> Result<(), Box<dyn Error>>{
        // If database already has holidays from the current year, exit
        if self.holidays()
            .await
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
        let response: String = reqwest::get("https://canada-holidays.ca/api/v1/holidays")
            .await?
            .text()
            .await?;

        let holiday_dates: Vec<NaiveDate> = serde_json::from_str::<Holidays>(&response)?
            .holidays
            .iter()
            .filter(|h| h.provinces.contains(&Province{id: "BC".to_string()}))
            .map(|h| h.observedDate.parse::<NaiveDate>())
            .collect::<Result<Vec<NaiveDate>, _>>()?;

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

        Ok(())
    }

    /// # Panics
    /// Panics if any database queries fail
    pub async fn add_vacation_day(&self, date: &NaiveDate){
        let date_string = date.to_string();
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
    }

    /// # Panics
    /// Panics if any database queries fail
    pub async fn delete_vacation_day(&self, date: &NaiveDate){
        let date_string = date.to_string();

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
    }

    /// # Panics
    /// Panics if any database queries fail, or `days_off` contains invalid dates
    #[allow(non_snake_case)]
    pub async fn vacation_days(&self) -> Vec<NaiveDate>{
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
    }

    /// # Panics
    /// Panics if any database queries fail, or `days_off` contains invalid dates
    #[allow(non_snake_case)]
    pub async fn holidays(&self) -> Vec<NaiveDate>{
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
    }

    // TODO This will likely cause a crash if missing internet connection (but only first time app
    // is launched in a given calendar year?)
    /// # Panics
    /// Panics if unable to update holidays 
    pub async fn days_off(&self) -> Vec<NaiveDate> {
        let mut days_off = Vec::new();

        self.try_update_holidays().await.unwrap();
        days_off.append(&mut self.holidays().await);
        days_off.append(&mut self.vacation_days().await);

        days_off
    }

    pub async fn schedule(&self) -> Schedule{
        Schedule::new(
            self.days_off().await,
            self.open_tasks().await,
        )
    }
}
