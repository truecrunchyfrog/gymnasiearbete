use crate::data::QUEUE;
use crate::docker;
use crate::task_queue::add_task;
use std::path::Path;

pub struct ContanerBuildInfo {
    pub file_path: String,
    pub user_id: String,
}

pub enum TaskTypes {
    RunCode(String),
    CreateImage(ContanerBuildInfo),
    StartContainer(String),
    RemoveContainer(String),
    CreateContainer(String),
}

pub struct Task {
    pub(crate) task_type: TaskTypes,
    pub dependencies: Vec<Task>,
}

impl Task {
    pub async fn run_task(&self) {
        info!("Looking for tasks to run!");
        match &self.task_type {
            TaskTypes::CreateImage(s) => {
                self.create_image(s).await.expect("Failed to create image");
            }
            TaskTypes::StartContainer(s) => self.start_container(s).await,
            _ => todo!("Unimplemented task"),
        }
        info!("Task completed!");
    }
    pub async fn add_to_queue(self) {
        add_task(self).await;
    }
    async fn create_image(
        &self,
        build_info: &ContanerBuildInfo,
    ) -> Result<String, shiplift::Error> {
        info!("Started create_image task!");
        #[cfg(unix)]
        {
            let file_path: &Path = Path::new(&build_info.file_path);
            return docker::create_image(&file_path, &build_info.user_id).await;
        }
        info!("Image was created!");
        #[cfg(not(unix))]
        {
            error!("Cannot create images on Windows!");
        }
    }

    async fn start_container(&self, tag: &str) {
        info!("Started start_container task!");
        #[cfg(unix)]
        {
            docker::start_container(tag)
                .await
                .expect("Failed to start container");
        }
        #[cfg(not(unix))]
        {
            error!("Cannot start container on Windows!");
        }
    }
}

pub struct Queue<Task> {
    queue: Vec<Task>,
}

impl Queue<Task> {
    pub fn new() -> Self {
        Queue { queue: Vec::new() }
    }

    pub fn enqueue(&mut self, item: Task) {
        self.queue.push(item)
    }

    pub fn dequeue(&mut self) -> Task {
        self.queue.remove(0)
    }

    pub fn length(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
