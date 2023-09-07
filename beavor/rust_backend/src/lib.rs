use pyo3::prelude::{
    pyfunction,
    pymodule,
    pyclass,
    pymethods,
    PyResult,
    PyModule,
    Python
};
use pyo3::types::PyType;
use pyo3::wrap_pyfunction;

use tokio::runtime::Runtime;

use sqlx::sqlite::{
    SqlitePool,
    SqlitePoolOptions,
    SqliteRow,
    SqliteConnectOptions,
};
use sqlx::{
    Executor,
    ConnectOptions,
};

use std::str::FromStr;

mod date;
use date::{
    DueDate,
    PyDueDate,
    PyDueDateType,
    today_date,
    parse_date,
    format_date,
    today_str,
};

mod task;
use task::Task;

#[pyfunction]
fn green_red_scale(low: f32, high: f32, val: f32) -> String {
    let frac = f32::max(0.0,f32::min(1.0,(val-low)/(high-low)));

    let red: u8;
    let green: u8;

    if frac > 0.5{
        red = 255;
        green = ((2.0-2.0*frac) * 255.0) as u8;
    }else{
        red = ((2.0*frac) * 255.0) as u8;
        green = 255
    }

    format!("#{red:02X}{green:02X}00")
}

#[pyclass]
struct DatabaseManager{
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
            pool: rt.block_on(
                      SqlitePoolOptions::new()
                      .after_connect(|conn, _meta| Box::pin( async {
                          conn.execute("PRAGMA foreign_keys=ON").await?;
                        Ok(())
                      }))
                      .connect(database_path.as_str())
                    ).expect("Should be able to connect to database"),
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
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let date_added_str = DueDate::Date(task.date_added).to_string();

        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO tasks
                    (
                        Name,
                        Finished,
                        TimeBudgeted,
                        TimeNeeded,
                        TimeUsed,
                        Available,
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
                        ?
                    )
            ",
                task.task_name,
                task.finished,
                task.time_needed, // When creating a new task, save the initial time_needed estimate as time_budgeted
                task.time_needed,
                task.time_used,
                next_action_str,
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
                FROM tasks
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

        self.rt.block_on(async{
            sqlx::query!("
                UPDATE tasks
                SET
                    Finished =           ?,
                    Name =        ?,
                    TimeNeeded =        ?,
                    TimeUsed =        ?,
                    Available =  ?,
                    Notes =       ?
                WHERE
                    rowid == ?
            ",
                task.finished,
                task.task_name,
                task.time_needed,
                task.time_used,
                next_action_str,
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

    fn get_open_tasks(&self) -> Vec<Task>{
        let mut tasks: Vec<Task> = Vec::new();

        self.rt.block_on(async{
            // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
            // return type of query! to write an impl From<T> for Task
            tasks = sqlx::query("
                SELECT *, rowid
                FROM tasks
                WHERE Finished == 0
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
                SELECT Name
                FROM categories
                ORDER BY Name
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get categories")
                .into_iter()
                .map(|r| r.Name.expect("Each category should be a string"))
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

#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_date, m)?)?;
    m.add_function(wrap_pyfunction!(green_red_scale, m)?)?;
    m.add_function(wrap_pyfunction!(parse_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_str, m)?)?;
    m.add_class::<Task>().unwrap();
    m.add_class::<PyDueDate>().unwrap();
    m.add_class::<PyDueDateType>().unwrap();
    m.add_class::<DatabaseManager>().unwrap();
    Ok(())
}
