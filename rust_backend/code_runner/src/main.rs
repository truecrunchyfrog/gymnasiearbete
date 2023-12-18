use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::process::Command;
use uuid::Uuid;

fn main() -> std::io::Result<()> {
    let socket_path = env::var("SOCKET_PATH").unwrap_or_else(|_| "/tmp/my_socket.sock".to_string());
    let listener = UnixListener::bind(socket_path)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream)?,
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: UnixStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];

    let temp_file_name = format!("temp_{}.c", uuid::Uuid::new_v4());

    // Create or truncate the temporary file
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&temp_file_name)?;

    while let Ok(size) = stream.read(&mut buffer) {
        if size == 0 {
            break;
        }
        file.write_all(&buffer[..size])?;
    }

    let output = Command::new("gcc").arg(&temp_file_name).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            stream.write_all(stdout.as_bytes())?;
            stream.write_all(stderr.as_bytes())?;
        }
        Err(e) => {
            let error_message = format!("Failed to execute gcc: {}", e);
            stream.write_all(error_message.as_bytes())?;
        }
    }

    // Remove the input file
    let _ = fs::remove_file(&temp_file_name);

    Ok(())
}
