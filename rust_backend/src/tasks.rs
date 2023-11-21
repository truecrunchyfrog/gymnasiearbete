use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use tokio::time::sleep;
use async_trait::async_trait;
use crate::database::connection::get_connection;

pub struct TaskManager {
    pub tasks: Vec<Box<dyn Task + Send>>
}

pub enum TaskResult {
    Completed,
    InvalidArguments(String),
    Failed(String)
}


impl TaskManager {
    async fn run_next(&mut self) {
        if let Some(real_task) = self.tasks.pop() {
            info!("Started running task");
            let task_result = real_task.run();
            info!("Done running task");
            match task_result.await {
                Ok(k) => match k {
                    TaskResult::Completed => info!("Task completed!"),
                    TaskResult::Failed(e) => warn!("Task failed: {}",e),
                    TaskResult::InvalidArguments(e) => error!("Task is missing argument! {}" ,e)
                },
                Err(e) => error!("{}", e)
            }
        }
    }
    fn add_task(&mut self, task: Box<dyn Task + Send>) {
        self.tasks.push(task);
    }
}

pub struct ExampleTask {
    db: Pool<ConnectionManager<PgConnection>>
}

#[async_trait]
pub trait Task: Send {
    async fn run(&self) -> Result<TaskResult, Box<dyn Error>>;
}


#[async_trait]
impl Task for ExampleTask {
    async fn run(&self) -> Result<TaskResult, Box<dyn Error>> {

        Ok(TaskResult::Completed)
    }
}

impl ExampleTask {
    pub fn new(tm: &Arc<Mutex<TaskManager>>, db: Pool<ConnectionManager<PgConnection>>) {
        let t = Box::new(ExampleTask {db});
        let mut tm = tm.lock().unwrap();
        tm.tasks.push(t);
    }
}

pub fn start_task_thread(tm: Arc<Mutex<TaskManager>>) {
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            loop {
                sleep(Duration::from_secs(1)).await;
                {
                    let tm = Arc::clone(&tm);
                    let tm_guard = tm.lock().unwrap();
                    if !tm_guard.tasks.is_empty() {
                        let tm_ref = Arc::clone(&tm);
                        tokio::task::spawn_blocking(move || {
                            let mut tm = tm_ref.lock().unwrap();
                            tokio::runtime::Runtime::new().unwrap().block_on(async {
                                tm.run_next().await;
                            });
                        });
                    }
                    // tm_guard is automatically dropped here, releasing the lock
                }
            }
        });
    });
}