use std::{io::Write, path::Path};

use tempfile::tempfile;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::api::run_code::{self, build_file, run_file};

pub struct PingPong {
    pub submitted_code: File,
    pub result: Option<String>,
    pub correct_answer: String,
}

pub trait GameLogic {
    async fn start(self);
    async fn setup(&self);
    async fn verify(&self) -> bool;
    async fn run(&mut self) -> Option<String>;
}

impl PingPong {
    pub async fn new(file: File) -> Self {
        Self {
            // Load example.c from disk
            submitted_code: file,
            correct_answer: "pong".to_string(),
            result: None,
        }
    }
}

impl GameLogic for PingPong {
    async fn start(mut self) {
        info!("Starting game");
        self.setup().await;
        self.result = self.run().await;
        let won = self.verify().await;
        info!("Game won: {}", won);
    }
    async fn setup(&self) {
        info!("Setting up game")
    }
    async fn verify(&self) -> bool {
        info!("Verifying game");
        match self.result {
            Some(ref result) => result == &self.correct_answer,
            None => false,
        }
    }
    async fn run(&mut self) -> Option<String> {
        info!("Running game");
        let mut code_file: File =
            File::from_std(tempfile().expect("Failed to create a temporary file"));
        // Write the content of the file to the temporary file
        let mut content = Vec::new();
        self.submitted_code
            .read_to_end(&mut content)
            .await
            .expect("Failed to read file");
        code_file
            .write_all(&content)
            .await
            .expect("Failed to write to file");

        let artifact = build_file(code_file).await.unwrap();

        let output = run_file(artifact).await.unwrap();
        self.result = Some(output.logs.last().unwrap().to_string());
        info!("Player output: {:?}", self.result);

        return Some(output.logs.last().unwrap().to_string());
    }
}

pub async fn start_game<T: GameLogic>(game: T) {
    info!("Starting game");
    T::start(game);
}
