use std::str::FromStr;
use std::sync::Arc;

use pyo3::prelude::{
    pyclass,
    pymethods,
    PyResult,
    PyErr,
    Python,
    PyAny
};

use pyo3::exceptions::{
    PyTypeError,
    PyConnectionError
};

use pyo3::types::PyType;

use pyo3_asyncio::async_std::future_into_py;
use pyo3_asyncio::tokio::re_exports::runtime::Runtime;

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
    ParseDateError,
    parse_date,
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
            id:                           row.get::<Option<i32>, &str>("rowid"),
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
#[allow(non_snake_case)]
struct Holiday{
    provinces: Vec<Province>,
    observedDate: String
}
#[pyclass]
pub struct PyDatabaseManager{
    db: Arc<DatabaseManager>,
}

#[allow(non_snake_case)]
#[pymethods]
impl PyDatabaseManager{
    #[new]
    fn new(database_path: String) -> PyResult<Self>{
        let rt = Runtime::new()?;
        Ok(Self{
            db: rt.block_on(DatabaseManager::new(database_path)).into()
        })
    }

    #[classmethod]
    fn create_new_database<'a>(_cls: &PyType, py: Python<'a>, database_path: String) -> PyResult<&'a PyAny>{
        future_into_py(py, async move{
            DatabaseManager::create_new_database(database_path).await;
            Ok(())
        })
    }

    fn create_task<'a>(&self, py: Python<'a>, task: Task) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            Ok(me.create_task(task).await)
        })
    }

    fn update_task<'a>(&self, py: Python<'a>, task: Task) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            me.update_task(task).await;
            Ok(())
        })
    }

    fn delete_task<'a>(&self, py: Python<'a>, task: Task) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            me.delete_task(task).await;
            Ok(())
        })
    }

    fn get_open_tasks<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            Ok(me.get_open_tasks().await)
        })
    }

    fn get_categories<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            Ok(me.get_categories().await)
        })
    }

    fn try_update_holidays<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            me.try_update_holidays().await
        })
    }

    fn add_vacation_day<'a>(&self, py: Python<'a>, date: NaiveDate) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            me.add_vacation_day(date).await;
            Ok(())
        })
    }

    fn delete_vacation_day<'a>(&self, py: Python<'a>, date: NaiveDate)-> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            me.delete_vacation_day(date).await;
            Ok(())
        })
    }

    fn get_vacation_days<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            Ok(me.get_vacation_days().await)
        })
    }

    fn get_holidays<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            Ok(me.get_holidays().await)
        })
    }

    fn get_days_off<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            Ok(me.get_days_off().await)
        })
    }

    fn get_schedule<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny>{
        let me = Arc::clone(&self.db);
        future_into_py(py, async move{
            Ok(me.get_schedule().await)
        })
    }
}

pub struct DatabaseManager{
    pool: SqlitePool,
}

impl DatabaseManager{
    async fn new(database_path: String) -> Self{
        Self{pool: SqlitePool::connect(database_path.as_str())
           .await
           .expect("Should be able to connect to database")
        }
    }

    async fn create_new_database(database_path: String){
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
    }

    async fn create_task(&self, task: Task) -> Task{
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
    }

    async fn update_task(&self, task: Task){
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let due_date_str = task.due_date.to_string();

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
    }

    async fn delete_task(&self, task: Task){
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
    }

    async fn get_open_tasks(&self) -> Vec<Task>{
        // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
        // return type of query! to write an impl From<T> for Task
      let mut tasks: Vec<Task> = sqlx::query("
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
            ).collect();

        tasks.sort_by(|a,b| a.due_date.cmp(&b.due_date));

        tasks
    }

    #[allow(non_snake_case)]
    async fn get_categories(&self) -> Vec<String>{
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

    async fn try_update_holidays(&self) -> Result<(), PyErr>{
        // If database already has holidays from the current year, exit
        if self.get_holidays()
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
            .await
            .map_err(|e: reqwest::Error| PyErr::new::<PyConnectionError, _>(e.to_string()))?
            .text()
            .await
            .map_err(|e: reqwest::Error| PyErr::new::<PyConnectionError, _>(e.to_string()))?;

        let holiday_dates: Vec<NaiveDate> = serde_json::from_str::<Holidays>(&response)
            .map_err(|e| PyErr::new::<PyTypeError, _>(e.to_string()))?
            .holidays
            .iter()
            .filter(|h| h.provinces.contains(&Province{id: "BC".to_string()}))
            .map(|h| h.observedDate.parse::<NaiveDate>())
            .collect::<Result<Vec<NaiveDate>, _>>()
            .map_err(|e| PyErr::new::<PyTypeError, _>(e.to_string()))?;

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

    async fn add_vacation_day(&self, date: NaiveDate){
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

    async fn delete_vacation_day(&self, date: NaiveDate){
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

    #[allow(non_snake_case)]
    async fn get_vacation_days(&self) -> Vec<NaiveDate>{
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

    #[allow(non_snake_case)]
    async fn get_holidays(&self) -> Vec<NaiveDate>{
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

    async fn get_days_off(&self) -> Vec<NaiveDate> {
        let mut days_off = Vec::new();

        self.try_update_holidays().await.unwrap();
        days_off.append(&mut self.get_holidays().await);
        days_off.append(&mut self.get_vacation_days().await);

        days_off
    }

    async fn get_schedule(&self) -> Schedule{
        Schedule::new(
            self.get_days_off().await,
            self.get_open_tasks().await,
        )
    }
}
