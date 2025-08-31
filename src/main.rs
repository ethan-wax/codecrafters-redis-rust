#![allow(unused_imports)]
use once_cell::sync::Lazy;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::i32::MIN;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{result, thread};
use tokio::stream;

pub mod parse;
use parse::{parse_command, Command};

type StoreType = HashMap<String, (String, Option<std::time::SystemTime>)>;
type ListStoreType = HashMap<String, Vec<String>>;

static STORE: Lazy<Arc<Mutex<StoreType>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static LIST_STORE: Lazy<Arc<Mutex<ListStoreType>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

fn main() {
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
                    Command::ECHO(text) => handle_echo(stream, &text),
                    Command::SET(key, value, exp) => handle_set(stream, &key, &value, &exp),
                    Command::GET(key) => handle_get(stream, &key),
                    Command::RPUSH(key, val_vec) => handle_rpush(stream, &key, &val_vec),
                    Command::LRANGE(key, start, end) => handle_lrange(stream, &key, &start, &end),
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

fn handle_echo(mut stream: &TcpStream, text: &String) {
    stream
        .write_all(format!("${}\r\n{}\r\n", text.len(), text).as_bytes())
        .unwrap();
}

fn handle_set(mut stream: &TcpStream, key: &String, value: &String, exp: &Option<SystemTime>) {
    {
        let mut store_guard = STORE.lock().unwrap();
        store_guard.insert(key.clone(), (value.clone(), exp.clone()));
    }
    stream.write_all("+OK\r\n".as_bytes()).unwrap();
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
    stream.write_all("$-1\r\n".as_bytes()).unwrap();
}

fn handle_rpush(mut stream: &TcpStream, key: &String, val_vec: &Vec<String>) {
    let pos = {
        let mut list_store_guard = LIST_STORE.lock().unwrap();
        let list = list_store_guard.entry(key.clone()).or_insert_with(Vec::new);
        list.extend(val_vec.iter().cloned());
        list.len() as i32
    };
    stream
        .write_all(format!(":{}\r\n", pos).as_bytes())
        .unwrap();
}

fn write_empty_array(mut stream: &TcpStream) {
    stream.write_all("*0\r\n".as_bytes()).unwrap();
}

fn handle_lrange(mut stream: &TcpStream, key: &String, start: &i32, end: &i32) {
    let result = {
        let list_store_guard = LIST_STORE.lock().unwrap();
        list_store_guard.get(key).cloned()
    };

    match result {
        Some(list) => {
            let len = list.len() as i32;

            let start_pos = if start >= &0 {
                *start
            } else if *start > -1 * len {
                len as i32 + start
            } else {
                0
            } as usize;

            let end_pos = if end >= &0 {
                min(*end + 1, len)
            } else if *end > -1 * len {
                len as i32 + end + 1
            } else {
                0
            } as usize;

            if start_pos >= len as usize || start_pos > end_pos {
                write_empty_array(stream);
            } else {
                let slice = &list[start_pos..end_pos];
                let mut output = format!("*{}\r\n", end_pos - start_pos);
                for value in slice {
                    output.push_str(&format!("${}\r\n{}\r\n", value.len(), value));
                }
                stream.write_all(output.as_bytes()).unwrap();
            }
        }
        None => write_empty_array(stream),
    }
}
