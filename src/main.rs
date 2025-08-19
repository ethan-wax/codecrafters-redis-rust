#![allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

pub mod parse;
use parse::{parse_command, Command};

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
                let _handle = thread::spawn(move || {
                    handle_connection(&mut _stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: &TcpStream) {
    println!("accepted new connection");
    loop {
        println!("entered loop");
        let mut buf = String::new();
        let _read_bytes = stream.read_to_string(&mut buf);
        println!(&buf);
        let com = parse_command(&buf).unwrap();
        match com {
            Command::PING => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
            Command::ECHO(length, text) => {
                let s = format!("${}\r\n{}\r\n", length, text);
                stream.write_all(s.as_bytes()).unwrap();
            }
        }
    }
}
