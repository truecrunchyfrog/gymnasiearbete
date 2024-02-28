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
    async fn start(self) -> Result<(), anyhow::Error>;
    async fn setup(&self);
    async fn verify(&self) -> bool;
    async fn run(&mut self) -> Result<Option<String>, anyhow::Error>;
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
    async fn start(mut self) -> Result<(), anyhow::Error> {
        info!("Starting game");
        self.setup().await;
        self.result = self.run().await?;
        let won = self.verify().await;
        info!("Game won: {}", won);
        Ok(())
    }
    async fn setup(&self) {
        info!("Setting up game")
    }
    async fn verify(&self) -> bool {
        info!("Verifying game");
        self.result
            .as_ref()
            .map_or(false, |result| result == &self.correct_answer)
    }
    async fn run(&mut self) -> Result<Option<String>, anyhow::Error> {
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

        let artifact = build_file(code_file).await?;

        let output = run_file(artifact).await?;

        let logs: Option<String> = match output.logs.last() {
            Some(ref logs) => Some(logs.to_string()),
            None => return Ok(None),
        };
        self.result = logs.clone();
        info!("Player output: {:?}", self.result);

        Ok(logs)
    }
}

pub async fn start_game<T: GameLogic + Send>(game: T) {
    info!("Starting game");
    T::start(game);
}
