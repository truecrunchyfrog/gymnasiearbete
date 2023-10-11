use crate::data::QUEUE;
use crate::docker;
use std::path::Path;

pub struct ContanerBuildInfo {
    pub file_path: String,
    pub user_id: String,
}

pub enum TaskTypes {
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
        match &self.task_type {
            TaskTypes::CreateImage(s) => self.create_image(s).await,
            TaskTypes::StartContainer(s) => self.start_container(s).await,
            _ => todo!("Unimplemented task"),
        }
    }

    async fn create_image(&self, build_info: &ContanerBuildInfo) {
        #[cfg(unix)]
        {
            let file_path: &Path = Path::new(build_info.file_path);
            docker::create_image(&file_path, &build_info.user_id)
                .await
                .expect("Failed to create image");
        }
        #[cfg(not(unix))]
        {
            error!("Cant create images on windows!");
        }
    }

    async fn start_container(&self, tag: &str) {
        #[cfg(unix)]
        {
            docker::start_container(tag)
                .await
                .expect("failed to start container");
        }
        #[cfg(not(unix))]
        {
            error!("Cant start container on windows!");
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
        let de_task = self.queue.remove(0);
        return de_task;
    }
    pub fn length(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
