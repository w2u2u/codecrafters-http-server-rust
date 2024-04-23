// Uncomment this block to pass the first stage
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
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
                handle_stream(&mut stream).unwrap();
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
    let start_line: Vec<&str> = buffer_lines[0].split_whitespace().collect();

    let response = match start_line[1] {
        "/" => "HTTP/1.1 200 OK\r\n\r\n",
        _ => "HTTP/1.1 404 Not Found\r\n\r\n",
    };

    println!("{}", response);

    let _ = stream.write(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
