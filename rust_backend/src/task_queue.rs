use crate::data::QUEUE;
use crate::tasks::Task;
use std::thread;

pub async fn add_task(task: Task) {
    thread::spawn(move || {
        let mut data = QUEUE.lock().unwrap();
        data.enqueue(task);
    })
    .join()
    .unwrap()
}
pub async fn get_task() -> Task {
    thread::spawn(move || {
        let mut data = QUEUE.lock().unwrap();
        return data.dequeue();
    })
    .join()
    .unwrap()
}

pub async fn is_empty() -> bool {
    thread::spawn(move || {
        let data = QUEUE.lock().unwrap();
        return data.is_empty();
    })
    .join()
    .unwrap()
}
