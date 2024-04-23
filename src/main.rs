// Uncomment this block to pass the first stage
use std::{
    env, fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = Arc::new(Mutex::new(String::new()));

    for (i, arg) in args.iter().enumerate() {
        if arg == "--directory" && args.get(i + 1).is_some() {
            let mut d = dir.lock().unwrap();
            *d = args[i + 1].to_string();
            break;
        }
    }

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let d = Arc::clone(&dir);
                thread::spawn(move || {
                    handle_stream(&mut stream, d).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_stream(stream: &mut TcpStream, dir: Arc<Mutex<String>>) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer)?;
    let buffer_str = String::from_utf8_lossy(&buffer);
    let buffer_lines: Vec<&str> = buffer_str.split("\r\n").collect();
    let start_lines: Vec<&str> = buffer_lines[0].split_whitespace().collect();

    let response = match start_lines[1] {
        "/" => response_ok("", ""),
        path if path.starts_with("/echo") => response_echo(path),
        path if path.starts_with("/user-agent") => response_user_agent(buffer_lines[2]),
        path if path.starts_with("/files") => response_files(path, dir),
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

        response_ok("text/plain", &content)
    } else {
        response_not_found()
    }
}

fn response_user_agent(user_agent: &str) -> String {
    response_ok("text/plain", user_agent.split_whitespace().last().unwrap())
}

fn response_files(path: &str, directory: Arc<Mutex<String>>) -> String {
    let paths: Vec<&str> = path.split('/').collect();
    let dir = directory.lock().unwrap();
    let file_path = format!("{}{}", dir, paths.last().unwrap());
    let file = fs::read_to_string(file_path);

    if let Ok(file_content) = file {
        return response_ok("application/octet-stream", &file_content);
    }

    response_not_found()
}

fn response_ok(content_type: &str, content: &str) -> String {
    if content_type.is_empty() || content.is_empty() {
        "HTTP/1.1 200 OK\r\n\r\n".to_string()
    } else {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type,
            content.len(),
            content
        )
    }
}

fn response_not_found() -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}
