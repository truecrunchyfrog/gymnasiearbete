use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::os::unix::net::UnixListener;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let listener = UnixListener::bind("/tmp/my_socket.sock")?;

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
    let mut file = File::create("temp")?;

    while let Ok(size) = stream.read(&mut buffer) {
        if size == 0 {
            break;
        }
        file.write_all(&buffer[..size])?;
    }

    let output = Command::new("./temp").output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}
