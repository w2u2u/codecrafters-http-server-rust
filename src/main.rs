use std::{
    env, fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = parse_directory_arg(&args);

    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let dir = Arc::clone(&directory);

                thread::spawn(move || {
                    if let Err(e) = handle_stream(&mut stream, dir) {
                        eprintln!("error handling stream: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("error accepting connection: {}", e);
            }
        }
    }
}

fn parse_directory_arg(args: &[String]) -> Arc<Mutex<String>> {
    let mut dir = String::new();

    for (i, arg) in args.iter().enumerate() {
        if arg == "--directory" && args.get(i + 1).is_some() {
            dir = args[i + 1].clone();
            break;
        }
    }

    Arc::new(Mutex::new(dir))
}

fn handle_stream(stream: &mut TcpStream, dir: Arc<Mutex<String>>) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let buffer_size = stream.read(&mut buffer)?;
    let buffer_str = String::from_utf8_lossy(&buffer[..buffer_size]);
    let buffer_lines: Vec<&str> = buffer_str.trim().split("\r\n").collect();

    let response = match parse_request(&buffer_lines) {
        Ok(request) => handle_request(request, &dir),
        Err(_) => response_not_found(),
    };

    let _ = stream.write(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

struct Request<'a> {
    method: &'a str,
    path: &'a str,
    user_agent: Option<&'a str>,
    body: &'a str,
}

fn parse_request<'a>(lines: &'a [&'a str]) -> Result<Request<'a>, ()> {
    let start_line = lines.first().ok_or(())?;
    let start_line_parts: Vec<&str> = start_line.split_whitespace().collect();

    if start_line_parts.len() < 2 {
        return Err(());
    }

    let method = start_line_parts[0];
    let path = start_line_parts[1];
    let user_agent = lines.get(2).copied();
    let body = lines.last().unwrap_or(&"");

    Ok(Request {
        method,
        path,
        user_agent,
        body,
    })
}

fn handle_request(request: Request, dir: &Arc<Mutex<String>>) -> String {
    match (request.method, request.path) {
        ("GET", "/") => response_ok("", ""),
        ("GET", path) if path.starts_with("/echo") => handle_echo(path),
        ("GET", path) if path.starts_with("/user-agent") => handle_user_agent(request.user_agent),
        ("GET", path) if path.starts_with("/files") => handle_read_file(path, dir),
        ("POST", path) if path.starts_with("/files") => handle_write_file(path, dir, request.body),
        _ => response_not_found(),
    }
}

fn handle_echo(path: &str) -> String {
    let content = path.trim_start_matches("/echo/");

    response_ok("text/plain", content)
}

fn handle_user_agent(user_agent: Option<&str>) -> String {
    if let Some(user_agent) = user_agent {
        response_ok(
            "text/plain",
            user_agent.split_whitespace().last().unwrap_or(""),
        )
    } else {
        response_not_found()
    }
}

fn handle_read_file(path: &str, directory: &Arc<Mutex<String>>) -> String {
    let file_name = path.split('/').last().unwrap_or("");
    let file_path = format!("{}{}", directory.lock().unwrap(), file_name);

    if let Ok(file_content) = fs::read_to_string(file_path) {
        response_ok("application/octet-stream", &file_content)
    } else {
        response_not_found()
    }
}

fn handle_write_file(path: &str, directory: &Arc<Mutex<String>>, content: &str) -> String {
    let file_name = path.split('/').last().unwrap_or("");
    let file_path = format!("{}{}", directory.lock().unwrap(), file_name);

    if fs::write(file_path, content).is_ok() {
        response_created()
    } else {
        response_not_found()
    }
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

fn response_created() -> String {
    "HTTP/1.1 201 CREATED\r\n\r\n".to_string()
}

fn response_not_found() -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}
