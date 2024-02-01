use std::{io::Write, path::Path};

use tempfile::NamedTempFile;

use crate::api::run_code::{run_user_bin, setup_game_container};
pub struct PingPong {
    pub game_bin_path: String,
    pub player_bin: i32,
}

pub trait GameLogic {
    async fn start(&self) {}
}

impl PingPong {
    pub fn new(player: i32) -> Self {
        PingPong {
            game_bin_path: "./rust_server/demo_code/program.o".to_string(),
            player_bin: player,
        }
    }
}

fn path_to_tmp_file(path: &str) -> Result<NamedTempFile, anyhow::Error> {
    let mut tmp_file = match NamedTempFile::new() {
        Ok(o) => o,
        Err(e) => {
            println!("Failed to create tempfile: {}", e);
            return Err(anyhow::anyhow!("Failed to create tempfile"));
        }
    };
    let test_file_content = match std::fs::read(path) {
        Ok(o) => o,
        Err(e) => {
            println!("Failed to read file: {}", e);
            return Err(anyhow::anyhow!("Failed to read file"));
        }
    };
    // Write into tmpfile
    match tmp_file.write_all(&test_file_content) {
        Ok(o) => o,
        Err(e) => {
            println!("Failed to write to tmpfile: {}", e);
            return Err(anyhow::anyhow!("Failed to write to tmpfile"));
        }
    };
    Ok(tmp_file)
}

impl GameLogic for PingPong {
    async fn start(&self) {
        println!("Game started");
        // Steps
        // 1. Start the game bin in a container
        // 2. Get the first output from the container,
        // 3. Start the player container,
        // 4. Send the output to the player container,
        // 5. Get the output from the player container,
        // 6. Send the output to the game container,
        // 7. Repeat steps 2-6 until the game is over.

        // load file into named temp file
        let tmp_file = match path_to_tmp_file(self.game_bin_path.as_str()) {
            Ok(o) => o,
            Err(e) => {
                println!("Failed to create tempfile: {}", e);
                return;
            }
        };

        let output = match setup_game_container(tmp_file).await {
            Ok(o) => o,
            Err(e) => {
                println!("Failed to start game container: {}", e);
                return;
            }
        };
        println!("Output: {}", output);

        let example_player = "./rust_server/demo_code/program.o";
        // Start player container
        let user_file = match path_to_tmp_file(example_player) {
            Ok(o) => o,
            Err(e) => {
                println!("Failed to create tempfile: {}", e);
                return;
            }
        };
        let output = match run_user_bin(user_file, output).await {
            Ok(o) => o,
            Err(e) => {
                println!("Failed to start user container: {}", e);
                return;
            }
        };
        println!("Output: {}", output);
    }
}

pub fn start_game<T: GameLogic>(game: &mut T) {
    T::start(game);
}
