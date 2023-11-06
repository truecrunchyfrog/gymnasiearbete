use std::sync::{Arc, Condvar, Mutex};
use std::thread;

trait Task: Send + Sync {
    fn execute(&self);
    fn dependencies(&self) -> Vec<Box<dyn Task>>;
}

fn create_worker(queue: Arc<TaskQueue>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if let Some(task) = queue.dequeue() {
                task.execute();
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

impl Task for ClearCache {
    fn execute(&self) {
        println!("Clearing cache...");
    }

    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for ClearCache
    }
}

struct StopContainer(String);

impl Task for StopContainer {
    fn execute(&self) {
        println!("Stopping container with ID: {}", self.0);
    }

    fn dependencies(&self) -> Vec<Box<dyn Task>> {
        vec![] // No dependencies for StopContainer
    }
}

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
            self.queue.enqueue(current_task);
            for dependency in current_task.dependencies() {
                tasks_to_execute.push(dependency);
            }
        }
    }
}
