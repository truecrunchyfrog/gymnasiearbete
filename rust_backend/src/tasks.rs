use crate::docker;
use async_trait::async_trait;
use core::time;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

pub enum TaskResult {
    Done,
    Failed(Box<dyn Error>),
    Skipped,
}
#[async_trait]
pub trait Task: Send + Sync {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>>;
    fn dependencies(&self) -> Vec<Box<dyn Task>>;
}

async fn create_worker(queue: Arc<TaskQueue>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if let Some(task) = queue.dequeue() {
                let _task_result = task.execute();
                match _task_result.await {
                    _ => info!("Task Completed?"),
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

    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for ClearCache
    }
}

struct BuildImage(String, Path);
#[async_trait]
impl Task for BuildImage {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        info!("Started Building Image with ID: {}", self.0);
        let build_id = &self.0;
        let code_path = &self.1;

        let image = docker::create_image(code_path, &build_id).await;

        match image {
            Ok(_) => return Ok(Some(TaskResult::Done)),
            Err(e) => return Ok(Some(TaskResult::Failed(Box::new(e)))),
        }
    }

    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for StopContainer
    }
}

struct StopContainer(String);
#[async_trait]
impl Task for StopContainer {
    async fn execute(&self) -> Result<Option<TaskResult>, Box<dyn Error>> {
        println!("Stopping container with ID: {}", self.0);
        Ok(Some(TaskResult::Done))
    }

    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for StopContainer
    }
}
#[derive(Clone)]
pub struct JobSystem {
    queue: Arc<TaskQueue>,
}

impl JobSystem {
    pub fn new(num_workers: usize) -> Self {
        let queue = Arc::new(TaskQueue::new());
        let mut workers = Vec::new();
        for _ in 0..num_workers {
            workers.push(create_worker(queue.clone()));
        }
        JobSystem { queue }
    }

    pub fn submit_task(&self, task: Box<dyn Task>) {
        // Enqueue the task with its dependencies
        let mut tasks_to_execute = vec![task];
        while !tasks_to_execute.is_empty() {
            let current_task = tasks_to_execute.pop().unwrap();
            for dependency in current_task.dependencies() {
                tasks_to_execute.push(dependency);
            }
            self.queue.enqueue(current_task);
        }
    }
}
