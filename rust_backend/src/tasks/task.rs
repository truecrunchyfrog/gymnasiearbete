use async_trait::async_trait;
use std::error::Error;
use tokio::time::sleep;
use tokio::time::Duration;

#[derive(Debug)]
pub enum TaskType {
    Maintenance,
}

#[derive(Debug)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[async_trait]
pub trait Task {
    fn task_type(&self) -> TaskType;
    fn run(&self) -> Result<TaskStatus, Box<dyn Error>>;
}

pub struct TaskManager {
    tasks: Vec<Box<dyn Task + Send>>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: Box<dyn Task + Send>) {
        self.tasks.push(task);
    }

    pub async fn start_runner(self) {
        tokio::spawn(async move { runner(self).await });
    }

    fn run_next_task(&self) {
        if let Some(task) = self.tasks.first() {
            match task.run() {
                Ok(TaskStatus::Completed) => {
                    println!("Task completed");
                }
                Ok(TaskStatus::Failed) => {
                    println!("Task failed");
                }
                _ => {
                    println!("Task is still running");
                }
            }
        }
    }
}

struct ExampleTask;

#[async_trait]
impl Task for ExampleTask {
    fn task_type(&self) -> TaskType {
        TaskType::Maintenance
    }

    fn run(&self) -> Result<TaskStatus, Box<dyn Error>> {
        println!("Running example task");
        Ok(TaskStatus::Completed)
    }
}

async fn runner(tm: TaskManager) {
    let mut example_task = ExampleTask;

    loop {
        tm.run_next_task();
        sleep(Duration::from_secs(1)).await;
    }
}
