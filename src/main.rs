#![allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::io::Write;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    // Writing a comment so I can push again

    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");
    let mut buf = [0; 512];
    let _read_bytes = stream.read(buf).unwrap();
    if std::str::from_utf8(&buf).unwrap().contains("PING") {
        stream.write_all(b"+PONG\r\n").unwrap();
    }
}
