use diesel::PgConnection;
use std::error::Error;


pub struct TaskManager {
    tasks: Vec<Box<dyn Task>>
}

pub enum TaskResult {
    Completed,
    InvalidArguments(String),
    Failed
}

impl TaskManager {
    fn run_next(&mut self) {
        if let Some(real_task) = self.tasks.pop() {
            let task_result = real_task.run();
            match task_result {
                Ok(_k) => info!("Task Completed"),
                Err(e) => error!("{}", e)
            }
        }
    }
}


pub struct ExampleTask {
    connection: Option<PgConnection>
}

pub trait Task {
    fn run(&self) -> Result<TaskResult, Box<dyn Error>>;
}

impl Task for ExampleTask {
    fn run(&self) -> Result<TaskResult, Box<dyn Error>> {
        if self.connection.is_none() {
            return Ok(TaskResult::InvalidArguments("No connection to database given!".to_string()));
        }
        Ok(TaskResult::Completed)
    }
}

impl ExampleTask {
    fn new(tm: &mut TaskManager, connection: Option<PgConnection>) -> &Box<dyn Task> {
        let t = Box::new(ExampleTask { connection });
        tm.tasks.push(t);
        tm.tasks.last().unwrap()
    }
}

fn test_task() {
    let mut tm = TaskManager { tasks: Vec::new() };
    ExampleTask::new(&mut tm, None);
    tm.run_next();
}