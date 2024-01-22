use async_trait::async_trait;

use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::time::sleep;
use tokio::time::Duration;
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
pub struct TaskManager {
    pub tasks: Vec<Box<dyn Task + Send>>,
}

#[allow(clippy::module_name_repetitions)]
pub enum TaskResult {
    Completed,
    InvalidArguments(String),
    Failed(String),
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
                    TaskResult::Failed(e) => warn!("Task failed: {}", e),
                    TaskResult::InvalidArguments(e) => error!("Task is missing argument! {}", e),
                },
                Err(e) => error!("{}", e),
            }
        }
    }
    fn add_task(&mut self, task: Box<dyn Task + Send>) {
        self.tasks.push(task);
    }
}

pub struct ExampleTask {}

pub struct BuildImageTask {
    file_id: Uuid,
}

impl dyn Task {
    fn add_dependency(&self, deps: Box<dyn Task>) -> Result<(), Box<dyn Error>> {
        todo!()
    }
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
    pub fn new(tm: &Arc<Mutex<TaskManager>>) {
        let t = Box::new(Self {});
        let mut tm = match tm.lock() {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to lock task manager: {}", e);
                return;
            }
        };
        tm.add_task(t);
    }
}

#[async_trait]
impl Task for BuildImageTask {
    async fn run(&self) -> Result<TaskResult, Box<dyn Error>> {
        // Update database of file to mark that build has started
        // Download the file from the database
        // Send path to docker component
        // await result
        // return it

        Ok(TaskResult::Completed)
    }
}

impl BuildImageTask {
    pub fn new(tm: &Arc<Mutex<TaskManager>>, file_id: Uuid) {
        let t = Box::new(Self { file_id });
        let mut tm = match tm.lock() {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to lock task manager: {}", e);
                return;
            }
        };
        tm.add_task(t);
    }
}

pub fn start_task_thread(tm: Arc<Mutex<TaskManager>>) {
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(async {
                loop {
                    sleep(Duration::from_secs(1)).await;
                    {
                        let tm = Arc::clone(&tm);
                        let tm_guard = tm.lock().expect("Failed to acquire lock");
                        if !tm_guard.tasks.is_empty() {
                            let tm_ref = Arc::clone(&tm);
                            tokio::task::spawn_blocking(move || {
                                let mut tm = tm_ref.lock().expect("Failed to acquire lock");
                                tokio::runtime::Runtime::new()
                                    .expect("Failed to create Tokio runtime")
                                    .block_on(async {
                                        tm.run_next().await;
                                    });
                            });
                        }
                    }
                }
            });
    });
}
