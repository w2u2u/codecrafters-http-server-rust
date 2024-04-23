// Uncomment this block to pass the first stage
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                thread::spawn(move || {
                    handle_stream(&mut stream).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_stream(stream: &mut TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer)?;
    let buffer_str = String::from_utf8_lossy(&buffer);
    let buffer_lines: Vec<&str> = buffer_str.split("\r\n").collect();
    let start_lines: Vec<&str> = buffer_lines[0].split_whitespace().collect();

    let response = match start_lines[1] {
        "/" => response_ok(""),
        path if path.starts_with("/echo") => response_echo(path),
        path if path.starts_with("/user-agent") => response_user_agent(buffer_lines[2]),
        _ => response_not_found(),
    };

    let _ = stream.write(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn response_echo(path: &str) -> String {
    let paths: Vec<&str> = path.split('/').collect();

    if paths.len() > 2 {
        let content = paths[2..].join("/");

        response_ok(&content)
    } else {
        response_not_found()
    }
}

fn response_user_agent(user_agent: &str) -> String {
    response_ok(user_agent.split_whitespace().last().unwrap())
}

fn response_ok(content: &str) -> String {
    if content.is_empty() {
        "HTTP/1.1 200 OK\r\n\r\n".to_string()
    } else {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        )
    }
}

fn response_not_found() -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}
