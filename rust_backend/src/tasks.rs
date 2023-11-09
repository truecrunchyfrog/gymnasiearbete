use crate::docker;
use async_trait::async_trait;
use core::time;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use uuid::Uuid;
pub enum TaskResult {
    Done(Option<String>),
    Failed(Box<dyn Error>),
    Skipped,
}

#[derive(Clone)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

#[async_trait]
pub trait TaskTrait: Send + Sync {
    async fn execute(&self) -> TaskResult;
    fn dependencies(&self) -> Vec<Uuid>;
}

pub struct Task {
    pub status: TaskStatus,
    pub id: Uuid,
    pub dependencies: Vec<Uuid>,
    pub task_trait: Box<dyn TaskTrait>,
}

impl Task {
    pub fn new(task_trait: Box<dyn TaskTrait>) -> Self {
        let task = Task {
            status: TaskStatus::NotStarted,
            id: Uuid::new_v4(),
            dependencies: vec![],
            task_trait,
        };
        info!("Created task with id: {}", task.id);
        return task;
    }
    pub fn new_with_dependencies(task_trait: Box<dyn TaskTrait>, deps: Vec<&Task>) -> Self {
        let mut ids: Vec<Uuid> = vec![];
        for d in deps {
            info!("Added dep {}", d.id);
            ids.push(d.id);
        }
        let task = Task {
            status: TaskStatus::NotStarted,
            id: Uuid::new_v4(),
            dependencies: ids,
            task_trait,
        };
        info!("Created task with id: {}", task.id);
        return task;
    }
}

pub struct TaskQueue {
    queue: Mutex<Vec<Arc<Mutex<Box<Task>>>>>,
    condvar: Condvar,
}

impl TaskQueue {
    pub fn new() -> Self {
        TaskQueue {
            queue: Mutex::new(vec![]),
            condvar: Condvar::new(),
        }
    }

    pub fn enqueue(&self, task: Arc<Mutex<Box<Task>>>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(task);
        self.condvar.notify_one();
    }

    pub fn dequeue(&self) -> Option<Arc<Mutex<Box<Task>>>> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop()
    }
}

pub struct ClearCache {}

#[async_trait]
impl TaskTrait for ClearCache {
    async fn execute(&self) -> TaskResult {
        warn!("Started Clearing Cache!");
        let tens = time::Duration::from_secs(10);
        thread::sleep(tens);
        info!("Cache Cleaning Done!");
        TaskResult::Done(None)
    }

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }
}

struct BuildImage {
    image_id: String,
    code_path: String,
}
#[async_trait]
impl TaskTrait for BuildImage {
    async fn execute(&self) -> TaskResult {
        info!("Started Building Image with ID: {}", self.image_id);
        let code_path = Path::new(&self.code_path);

        let image = docker::create_image(code_path, &self.image_id).await;

        match image {
            Ok(_) => return TaskResult::Done(None),
            Err(e) => TaskResult::Failed(Box::new(e)),
        }
    }

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }
}

pub struct RunCode {
    pub code_path: String,
}
#[async_trait]
impl TaskTrait for RunCode {
    async fn execute(&self) -> TaskResult {
        info!("Started running runcode!");
        let tens = time::Duration::from_secs(10);
        thread::sleep(tens);
        info!("Code run done");
        TaskResult::Done(None)
    }

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }
}

struct StopContainer {
    container_id: String,
}
#[async_trait]
impl TaskTrait for StopContainer {
    async fn execute(&self) -> TaskResult {
        info!("Stopping container with ID: {}", self.container_id);
        match docker::stop_container(&self.container_id).await {
            Ok(_) => TaskResult::Done(None),
            Err(e) => TaskResult::Failed(Box::new(e)),
        }
    }

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }
}

#[derive(Clone)]
pub struct JobSystem {
    task_queue: Arc<TaskQueue>,
    pub tasks: HashMap<Uuid, Arc<Mutex<Box<Task>>>>,
}

impl JobSystem {
    pub async fn new(num_workers: usize) -> Self {
        let queue = Arc::new(TaskQueue::new());

        let job_system = JobSystem {
            task_queue: Arc::clone(&queue),
            tasks: HashMap::new(),
        };

        // Create a new thread for worker creation
        let job_system_clone = job_system.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for _ in 0..num_workers {
                    job_system_clone.create_worker().await;
                }
            });
        });

        job_system
    }

    async fn create_worker(&self) {
        loop {
            if let Some(task) = &self.task_queue.dequeue() {
                let task = Arc::clone(&task);
                let task_id = task.lock().unwrap().id;
                if self.should_wait(&task.lock().unwrap()) {
                    self.task_queue.enqueue(task);
                    continue;
                }
                info!("Started with task: {}", &task_id);

                let task_result = &task.lock().unwrap().task_trait.execute().await;
                match task_result {
                    TaskResult::Done(Some(d)) => info!("{} completed {}", &task_id, d),
                    TaskResult::Done(None) => info!("{} completed", &task_id),
                    TaskResult::Failed(e) => error!("{} failed: {}", &task_id, e),
                    TaskResult::Skipped => debug!("{} was skipped", &task_id),
                }
                info!("Started looking for a new task!");
                continue;
            } else {
                let _ = &self
                    .task_queue
                    .condvar
                    .wait(self.task_queue.queue.lock().unwrap());
                info!("Started looking for a new task!");
            }
        }
    }
    pub fn add_and_submit_task(&mut self, task: Task) {
        let task_id = task.id;
        self.tasks
            .insert(task_id, Arc::new(Mutex::new(Box::new(task))));
        self.submit_task(task_id)
    }
    pub fn submit_task(&self, task_id: Uuid) {
        if let Some(task) = self.tasks.get(&task_id) {
            info!("Added task with id: {}", &task_id);
            self.task_queue.enqueue(task.clone());
        }
    }
    pub fn should_wait(&self, task: &Task) -> bool {
        if task.dependencies.is_empty() {
            return false;
        }
        for d in &task.dependencies {
            info!("I have dep: {}", d);
            let task = self.tasks.get(d);
            if !task.is_some() {
                info!("I cant find task with id: {}", d);
                return false;
            }
            let status = &task.unwrap().lock().unwrap().status;
            match status {
                TaskStatus::NotStarted => return true,
                TaskStatus::InProgress => return true,
                _ => continue,
            }
        }
        return false;
    }
}
