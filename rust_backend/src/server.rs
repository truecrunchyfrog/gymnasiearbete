use crate::tasks::{Task, TaskTypes};
use crate::{docker, task_queue};
use axum::extract::Multipart;
use std::fs;
use std::io::Write;
use std::path::Path;

pub async fn upload(mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let path_str = format!("./upload/{}", name);
        let upload_dir: &Path = Path::new(&path_str);
        let mut file = fs::OpenOptions::new()
            .create(true)
            // .create(true) // To create a new file
            .write(true)
            // either use the ? operator or unwrap since it returns a Result
            .open(upload_dir)
            .expect(format!("Failed to find path: {}", path_str).as_str());

        file.write_all(&data).expect("Failed to write file");
        println!("Length of `{}` is {} bytes", name, data.len());
        let task: Task = Task {
            task_type: TaskTypes::CreateImage(path_str),
        };

        task_queue::add_task(task).await;

        let task2: Task = Task {
            task_type: TaskTypes::StartContainer(String::from("shiplift")),
        };
        task_queue::add_task(task2).await;
    }
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}
