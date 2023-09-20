use std::fs::File;
use std::io::Cursor;
use std::io::Write;
use std::io::{self, Read};
use std::process::{exit, Command};

const BUFFER_SIZE: usize = 1024;

pub fn write_stdin_to_file(size: &mut usize) -> io::Result<()> {
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut fp = File::create("./binary")?;

    loop {
        match io::stdin().read(&mut buffer) {
            Ok(read_bytes) => {
                if read_bytes == 0 {
                    // EOF
                    break;
                }

                // Write data to the file
                *size += read_bytes;
                fp.write_all(&buffer[..read_bytes])?;
            }
            Err(_) => {
                eprintln!("Failed to read binary data, exiting");
                exit(1);
            }
        }
    }

    Ok(())
}

fn main() {
    let mut size: usize = 0;
    let output_buffer_size = 1024; // replace with actual buffer size
    let mut output_buffer = vec![0u8; output_buffer_size];
    match write_stdin_to_file(&mut size) {
        Ok(_) => {}
        Err(e) => eprintln!("Error writing stdin to file: {}", e),
    };
    if size == 0 {
        println!("Empty binary file, discarding");
        exit(0);
    }
    let process = Command::new("./binary").output();

    match process {
        Ok(output) => {
            println!("Executing binary inside the sandbox");
            let mut cursor = Cursor::new(output.stdout);
            // Read the data as buffers and stream it to stdout
            loop {
                let read_bytes = cursor
                    .read(&mut output_buffer)
                    .expect("Failed to read the output");

                if read_bytes == 0 {
                    // EOF
                    break;
                }
                output_buffer[read_bytes] = 0;
                print!("{}", String::from_utf8_lossy(&output_buffer[..read_bytes]));
            }
        }
        Err(_) => {
            println!("Failed to execute the binary");
            exit(-1);
        }
    }
}
