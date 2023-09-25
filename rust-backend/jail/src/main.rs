use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
fn main() {
    // Spawn the firejail process with the necessary arguments
    let mut child = Command::new("firejail")
        .arg("--net=none")
        .arg("--private")
        .arg("--nodbus")
        .arg("--seccomp")
        .arg("--nonewprivs")
        .arg("--noroot")
        .arg("--nogroups")
        .arg("--shell=none")
        .arg("--noprofile")
        .arg("--machine-id")
        .arg("--disable-mnt")
        .arg("--quiet")
        .arg("--blacklist=/tmp")
        .arg("--whitelist=/tmp/app/jail")
        .arg("/tmp/app/jail")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn firejail");

    // Get a handle to the child process's stdin and stdout
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    // Write to the child process's stdin
    child_stdin
        .write_all(b"Your input data")
        .expect("Failed to write to stdin");

    // Connect to port 8000 on the local machine

    // Read from the child process's stdout and send it to the connected stream
    let mut output = Vec::new();
    child_stdout
        .read_to_end(&mut output)
        .expect("Failed to read from stdout");
    let mut stream = TcpStream::connect("127.0.0.1:8000").expect("Failed to connect to port 8000");
    stream
        .write_all(&output)
        .expect("Failed to send data to port 8000");

    // Wait for the child process to finish
    let status = child.wait().expect("Failed to wait on child");
    println!(
        "Child process exited with status: {:?}",
        status.code().unwrap()
    );
}
