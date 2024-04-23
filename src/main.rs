// Uncomment this block to pass the first stage
use std::{
    io::{self, Write},
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
    let _ = stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes())?;
    stream.flush()?;

    Ok(())
}
