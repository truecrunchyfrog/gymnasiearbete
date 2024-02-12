use std::{io::Write, path::Path};

use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::api::run_code::{self, build_file, run_file};

pub struct PingPong {
    pub submited_code: File,
    pub result: String,
    pub answer: String,
}

pub trait GameLogic {
    async fn start(self);
    async fn setup(&self);
    async fn verify(&self) -> bool;
    async fn run(&mut self) -> String;
}

impl PingPong {
    pub async fn new(player: i32) -> Self {
        Self {
            // Load example.c from disk
            submited_code: File::open("example.c").await.unwrap(),

            answer: "Correct".to_string(),
            result: "".to_string(),
        }
    }
}

impl GameLogic for PingPong {
    async fn start(mut self) {
        self.setup().await;
        self.result = self.run().await;
        let won = self.verify().await;
        info!("Game won: {}", won);
    }
    async fn setup(&self) {}
    async fn verify(&self) -> bool {
        self.answer == self.result
    }
    async fn run(&mut self) -> String {
        let mut code_file: File =
            File::from_std(tempfile().expect("Failed to create a temporary file"));
        // Write the content of the file to the temporary file
        let mut content = Vec::new();
        self.submited_code
            .read_to_end(&mut content)
            .await
            .expect("Failed to read file");
        code_file
            .write_all(&content)
            .await
            .expect("Failed to write to file");

        let artifact = build_file(code_file).await.unwrap();
        let mut file_content =
            crate::docker::common::extract_file_from_tar_archive(artifact, "program.o")
                .await
                .expect("Failed to extract file from archive");
        let mut file: File = File::from_std(tempfile().expect("Failed to create a temporary file"));
        file.write_all(&file_content)
            .await
            .expect("Failed to write to file");
        let output = run_file(file).await.unwrap();

        info!("Metrics: {:?}", output.metrics);

        return output.logs.last().unwrap().to_string();
    }
}

pub async fn start_game<T: GameLogic>(game: T) {
    T::start(game);
}
