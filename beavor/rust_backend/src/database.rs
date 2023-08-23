use std::str::FromStr;

use tokio::runtime::Runtime;

use pyo3::prelude::{
    pyclass,
    pymethods,
    PyResult,
};

use pyo3::types::PyType;

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
    today_date,
    DueDate
};

impl TryFrom<SqliteRow> for Task{
    type Error = ParseDateError;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(Task{
            category:                     row.get::<String, &str>("Category"),
            finished:                     row.get::<String, &str>("O"),
            task_name:                    row.get::<String, &str>("Task"),
            _time_budgeted:               row.get::<i32,    &str>("Budget"),
            time_needed:                  row.get::<i32,    &str>("Time"),
            time_used:                    row.get::<i32,    &str>("Used"),
            next_action_date: parse_date(&row.get::<String, &str>("NextAction"))?,
            due_date:                     row.get::<String, &str>("DueDate").try_into()?,
            notes:                        row.get::<String, &str>("Notes"),
            id:                           row.get::<Option<i32>, &str>("rowid"),
            date_added:       parse_date(&row.get::<String, &str>("DateAdded"))?,
        })
    }
}

#[pyclass]
pub struct DatabaseManager{
    pool: SqlitePool,
    rt: Runtime,
}

// TODO should make all these pass the asyncness through to Python to deal with
#[pymethods]
impl DatabaseManager{
    #[new]
    fn new(database_path: String) -> PyResult<Self>{
        let rt = Runtime::new().unwrap();
        Ok(Self{
            pool: rt.block_on(SqlitePool::connect(database_path.as_str()))
                .expect("Should be able to connect to database"),
            rt,
        })
    }

    #[classmethod]
    fn create_new_database(_cls: &PyType, database_path: String){
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
    }

    fn create_task(&self, task: Task) -> Task{
        let mut new_task = self.default_task();

        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let due_date_str = task.due_date.to_string();
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let date_added_str = DueDate::Date(task.date_added).to_string();

        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO worklist
                    (
                        Category,
                        O,
                        Task,
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
                task.task_name,
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
            new_task = sqlx::query("
                SELECT *, rowid
                FROM worklist
                WHERE rowid == ?
            ")
                .bind(new_rowid)
                .fetch_one(&self.pool)
                .await
                .expect("Should have inserted and retrieved a task")
                .try_into()
                .expect("Database should contain valid Tasks only");
        });

        new_task
    }

    fn update_task(&self, task: Task){
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let due_date_str = task.due_date.to_string();

        self.rt.block_on(async{
            sqlx::query!("
                UPDATE worklist
                SET
                    Category =    ?,
                    O =           ?,
                    Task =        ?,
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
                task.task_name,
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

    fn delete_task(&self, task: Task){
        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM worklist
                WHERE rowid == ?
            ",
                task.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able do delete task");
        });
    }

    fn get_open_tasks(&self) -> Vec<Task>{
        let mut tasks: Vec<Task> = Vec::new();

        self.rt.block_on(async{
            // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
            // return type of query! to write an impl From<T> for Task
            tasks = sqlx::query("
                SELECT *, rowid
                FROM worklist
                WHERE O == 'O'
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

        tasks
    }

    fn get_categories(&self) -> Vec<String>{
        let mut categories: Vec<String> = Vec::new();

        self.rt.block_on(async{
            categories = sqlx::query!("
                SELECT DISTINCT Category
                FROM worklist
                ORDER BY Category
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get categories")
                .into_iter()
                .map(|r| r.Category.expect("Each category should be a string"))
                .collect()

        });

        categories
    }

    fn default_task(&self) -> Task{
        Task{
            category:         "Work".into(),
            finished:         "O".into(),
            task_name:        "".into(),
            _time_budgeted:   0,
            time_needed:      0,
            time_used:        0,
            next_action_date: today_date(),
            due_date:         DueDate::Date(today_date()),
            notes:            "".into(),
            id:               None,
            date_added:       today_date(),
        }
    }
}
