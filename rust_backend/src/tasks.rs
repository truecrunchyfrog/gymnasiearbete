use crate::docker;
use async_trait::async_trait;
use core::time;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use tokio::task;
use uuid::Uuid;
pub enum TaskResult {
    Done,
    Failed(Box<dyn Error>),
    Skipped,
}
// Define a TaskStatus enum
#[derive(Clone)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

#[async_trait]
pub trait Task: Send + Sync {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>>;
    fn dependencies(&self) -> Vec<Box<dyn Task>>;
    fn status(&self) -> TaskStatus;
}

async fn create_worker(queue: Arc<TaskQueue>) -> task::JoinHandle<()> {
    task::spawn(async move {
        loop {
            if let Some(task) = queue.dequeue() {
                info!("Started On a task!");
                let _task_result = task.execute().await;
                if let Ok(Some(result)) = _task_result {
                    // no need to unwrap
                    match result {
                        TaskResult::Done => info!("Task completed!"),
                        TaskResult::Failed(e) => warn!("Task failed: {}", e),
                        TaskResult::Skipped => info!("Task skipped"),
                    }
                } else if let Err(e) = _task_result {
                    error!("{}", e);
                }
            } else {
                // No tasks available, wait for a signal.
                let _ = queue.condvar.wait(queue.queue.lock().unwrap());
            }
        }
    })
}
struct TaskQueue {
    queue: Mutex<Vec<Box<dyn Task>>>,
    condvar: Condvar,
}

impl TaskQueue {
    fn new() -> Self {
        TaskQueue {
            queue: Mutex::new(vec![]),
            condvar: Condvar::new(),
        }
    }

    fn enqueue(&self, task: Box<dyn Task>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(task);
        self.condvar.notify_one();
    }

    fn dequeue(&self) -> Option<Box<dyn Task>> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop()
    }
}

pub struct ClearCache;

#[async_trait]
impl Task for ClearCache {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        warn!("Started Clearing Cache!");
        let tens = time::Duration::from_secs(10);
        thread::sleep(tens);
        info!("Cache Cleaning Done!");
        Ok(Some(TaskResult::Done))
    }
    fn status(&self) -> TaskStatus {
        TaskStatus::InProgress
    }
    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for ClearCache
    }
}

struct BuildImage(String, String);
#[async_trait]
impl Task for BuildImage {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        info!("Started Building Image with ID: {}", self.0);
        let build_id = &self.0;
        let code_path = Path::new(&self.1);

        let image = docker::create_image(code_path, &build_id).await;

        match image {
            Ok(_) => return Ok(Some(TaskResult::Done)),
            Err(e) => return Ok(Some(TaskResult::Failed(Box::new(e)))),
        }
    }

    fn status(&self) -> TaskStatus {
        TaskStatus::InProgress
    }

    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for StopContainer
    }
}

pub struct RunCode(pub String);
#[async_trait]
impl Task for RunCode {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        info!("Code run done");

        Ok(Some(TaskResult::Done))
    }
    fn status(&self) -> TaskStatus {
        TaskStatus::InProgress
    }
    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        let code_path = &self.0;
        let build_id = Uuid::new_v4().simple().to_string();
        // Create image
        let create_image_task = BuildImage(build_id.clone(), code_path.to_string());
        // Start Container
        let start_container_task = StartContainer(build_id.clone());
        // Stop Container
        let stop_container_task = StopContainer(build_id.clone());
        // Remove Container
        let remove_container_task = RemoveContainer(build_id.clone());
        vec![
            Box::new(create_image_task),
            Box::new(start_container_task),
            Box::new(stop_container_task),
            Box::new(remove_container_task),
        ]
    }
}

struct RemoveContainer(String);
#[async_trait]
impl Task for RemoveContainer {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        info!("Removing container with ID: {}", self.0);
        let build_id = &self.0;

        let container_status = docker::remove_container(&build_id).await;

        match container_status {
            Ok(_) => return Ok(Some(TaskResult::Done)),
            Err(e) => return Ok(Some(TaskResult::Failed(Box::new(e)))),
        }
    }
    fn status(&self) -> TaskStatus {
        TaskStatus::InProgress
    }
    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for StopContainer
    }
}

struct StartContainer(String);
#[async_trait]
impl Task for StartContainer {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        info!("Started container with ID: {}", self.0);
        let build_id = &self.0;

        let container_status = docker::start_container(&build_id).await;

        match container_status {
            Ok(_) => return Ok(Some(TaskResult::Done)),
            Err(e) => return Ok(Some(TaskResult::Failed(Box::new(e)))),
        }
    }
    fn status(&self) -> TaskStatus {
        TaskStatus::InProgress
    }
    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for StopContainer
    }
}

struct StopContainer(String);
#[async_trait]
impl Task for StopContainer {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        info!("Stopping container with ID: {}", self.0);
        Ok(Some(TaskResult::Done))
    }
    fn status(&self) -> TaskStatus {
        TaskStatus::InProgress
    }
    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for StopContainer
    }
}
#[derive(Clone)]
pub struct JobSystem {
    queue: Arc<TaskQueue>,
    task_statuses: HashMap<Uuid, TaskStatus>, // to keep track of task statuses
}

impl JobSystem {
    pub fn new(num_workers: usize) -> Self {
        let queue = Arc::new(TaskQueue::new());
        let mut workers = Vec::new();
        for _ in 0..num_workers {
            workers.push(create_worker(queue.clone()));
        }
        JobSystem {
            queue,
            task_statuses: HashMap::new(),
        }
    }

    pub fn submit_task(&self, task: Box<dyn Task>) {
        // Before enqueueing a task, check its dependencies
        if task
            .dependencies()
            .iter()
            .all(|dep| match self.task_statuses.get(&dep.id()) {
                Some(TaskStatus::Completed) => true,
                _ => false,
                None => todo!(),
            })
        {
            self.queue.enqueue(task);
        } else {
            // If all dependencies are not completed, re-enqueue the task
            // You might want to add some delay or backoff here to avoid busy-waiting
            self.submit_task(task);
        }
    }
}
