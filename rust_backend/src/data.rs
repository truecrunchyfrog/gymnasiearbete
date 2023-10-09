use crate::task_queue::get_task;
use crate::task_queue::is_empty;
use crate::tasks::Queue;
use crate::tasks::Task;
use once_cell::sync::Lazy;
/// Use Mutex for thread-safe access to a variable e.g. our DATA data.
use std::sync::Mutex;

pub static QUEUE: Lazy<Mutex<Queue<Task>>> = Lazy::new(|| Mutex::new(Queue::new()));

pub async fn queue_thread() {
    loop {
        if !is_empty().await {
            let task = get_task().await;
            task.run_task().await;
        }
    }
}
