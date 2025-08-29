#![allow(unused_imports)]
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{result, thread};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use tokio::stream;
use std::time::SystemTime;

pub mod parse;
use parse::{parse_command, Command};

type StoreType = HashMap<String, (String, Option<std::time::SystemTime>)>;
type ListStoreType = HashMap<String, Vec<String>>;

static STORE: Lazy<Arc<Mutex<StoreType>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

static LIST_STORE: Lazy<Arc<Mutex<ListStoreType>>> = Lazy::new(|| {
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
                    Command::PING => handle_ping(stream),
                    Command::ECHO(length, text) => handle_echo(stream, &length, &text),
                    Command::SET(key, value, exp) => handle_set(stream, &key, &value, &exp),
                    Command::GET(key) => handle_get(stream, &key),
                    Command::RPUSH(key, value) => handle_rpush(stream, &key, &value),
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                bulk_null(stream)
            }
        }
    }
}

fn handle_ping(mut stream: &TcpStream) {
    stream.write_all(b"+PONG\r\n").unwrap()
}

fn handle_echo(mut stream: &TcpStream, length: &i32, text: &String) {
    stream
        .write_all(format!("${}\r\n{}\r\n", length, text).as_bytes())
        .unwrap();
}

fn handle_set(mut stream: &TcpStream, key: &String, value: &String, exp: &Option<SystemTime>) {
    {
        let mut store_guard = STORE.lock().unwrap();
        store_guard.insert(key.clone(), (value.clone(), exp.clone()));
    }
    stream
        .write_all("+OK\r\n".as_bytes())
        .unwrap();
}

fn handle_get(mut stream: &TcpStream, key: &String) {
    let result = {
        let store_guard = STORE.lock().unwrap();
        store_guard.get(key).cloned()
    };

    match result {
        Some((value, None)) => {
            stream
                .write_all(format!("${}\r\n{}\r\n", value.len() as i32, value).as_bytes())
                .unwrap();
        }
        Some((value, Some(time))) => {
            if SystemTime::now() > time {
                {
                    let mut store_guard = STORE.lock().unwrap();
                    store_guard.remove(key);
                }
                bulk_null(stream);
            } else {
                stream
                    .write_all(format!("${}\r\n{}\r\n", value.len() as i32, value).as_bytes())
                    .unwrap();
            }
        }
        None => bulk_null(stream),
    }
}

fn bulk_null(mut stream: &TcpStream) {
    stream
        .write_all("$-1\r\n".as_bytes())
        .unwrap();
}

fn handle_rpush(mut stream: &TcpStream, key: &String, value: &String) {
    let pos = {
        let mut list_store_guard = LIST_STORE.lock().unwrap();
        let list = list_store_guard.entry(key.clone()).or_insert_with(Vec::new);
        list.push(value.clone());
        list.len() as i32
    };
    stream
        .write_all(format!(":{}\r\n", pos).as_bytes())
        .unwrap();
}