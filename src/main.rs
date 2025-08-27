#![allow(unused_imports)]
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

pub mod parse;
use parse::{parse_command, Command};

static STORE: Lazy<Arc<Mutex<HashMap<String, String>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

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
    let mut buf = [0; 1024];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("client disconnected");
                break;
            }
            Ok(n) => {
                let s = String::from_utf8_lossy(&buf[..n]);

                match parse_command(&s).unwrap() {
                    Command::PING => stream.write_all(b"+PONG\r\n").unwrap(),
                    Command::ECHO(length, text) => {
                        stream
                            .write_all(format!("${}\r\n{}\r\n", length, text).as_bytes())
                            .unwrap();
                    }
                    Command::SET(key, value) => {
                        {
                            let mut store_guard = STORE.lock().unwrap();
                            store_guard.insert(key, value);
                        }
                        stream
                            .write_all("+OK\r\n".as_bytes())
                            .unwrap();
                    }
                    Command::GET(key) => {
                        let value = {
                            let store_guard = STORE.lock().unwrap();
                            store_guard.get(&key).cloned().unwrap()
                        };
                        stream
                            .write_all(format!("${}\r\n{}\r\n", value.len() as i32, value).as_bytes())
                            .unwrap();
                    }
                }
            }
            Err(_) => {
                println!("error reading into buffer");
                break;
            }
        }
    }
}
