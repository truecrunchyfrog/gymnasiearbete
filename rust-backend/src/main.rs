use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{exit, Command};

const BUFFER_SIZE: usize = 1024;

pub enum Language {
    C,
    Python,
    Bin,
}

pub fn get_language() -> Result<Language, env::VarError> {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        match String::from(args[1].clone()).as_str() {
            "python" => {
                return Ok(Language::Python);
            }
            "bin" => {
                return Ok(Language::Bin);
            }
            _ => {
                return Err(env::VarError::NotPresent);
            }
        };
    }
    return Err(env::VarError::NotPresent);
}

pub fn write_stdin_to_file(size: &mut usize) -> io::Result<()> {
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut fp = File::create("./binary.exe")?;

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

    // Flush the file buffer
    fp.flush()?;

    // Explicitly drop the File to close it
    drop(fp);

    Ok(())
}

fn run_bin_program() {
    let mut size: usize = 0;
    match write_stdin_to_file(&mut size) {
        Ok(_) => {}
        Err(e) => eprintln!("Error writing stdin to file: {}", e),
    };
    if size == 0 {
        println!("Empty binary file, discarding");
        exit(0);
    }
    let output = Command::new("./binary.exe")
        .output()
        .expect("Failed to execute the binary");
    println!("Executing binary inside the sandbox");
    print!("{}", String::from_utf8_lossy(&output.stdout));
}

fn main() {
    let language = match get_language() {
        Ok(lang) => lang,
        Err(err) => {
            println!("Failed {:?}", err);
            exit(1)
        }
    };
    match language {
        Language::Bin => run_bin_program(),
        _ => return,
    }
}
