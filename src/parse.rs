use std::fmt::format;
use std::result::Result;
use std::time::{Duration, SystemTime};

pub enum Command {
    PING,
    ECHO(String),
    SET(String, String, Option<SystemTime>),
    GET(String),
    RPUSH(String, Vec<String>),
    LRANGE(String, i32, i32),
    LPUSH(String, Vec<String>)
}

fn extract_text(chunks: &Vec<&str>, pos: usize) -> Result<String, String> {
    let length_str = chunks
        .get(pos)
        .ok_or("Missing length".to_string())?
        .trim_start_matches("$");
    let text_length = length_str
        .parse::<i32>()
        .map_err(|_| format!("Length must be an integer, got: '{}'", length_str))?;
    let text = chunks.get(pos+1).ok_or("Missing text".to_string())?;

    if text.len() as i32 != text_length {
        return Err(format!("Actual text ({}:{}) doesn't match expected length ({})", text.len() as i32, text, text_length).to_string());
    }

    Ok(text.to_string())
}

fn extract_int(chunks: &Vec<&str>, pos: usize) -> Result<i32, String> {
    let num_str = extract_text(chunks, pos)?;
    let num = num_str
        .parse::<i32>()
        .map_err(|_| format!("Expected an integer but got: {}", num_str))?;
    
    Ok(num)
}

pub fn parse_command(command: &str) -> Result<Command, String> {
    let mut chunks: Vec<&str> = command.split("\r\n").collect();
    chunks.pop(); // The last item is always empty
    let args = chunks.get(0).ok_or("Missing argument length".to_string())?;
    let com = extract_text(&chunks, 1)?;


    match (*args, com.as_str()) {
        ("*1", "PING") => Ok(Command::PING),
        ("*2", "ECHO") => parse_echo(&chunks),
        ("*3", "SET") => parse_set(&chunks),
        ("*5", "SET") => parse_set_expiry(&chunks),
        ("*2", "GET") => parse_get(&chunks),
        (_, "RPUSH") => parse_rpush(&chunks),
        ("*4", "LRANGE") => parse_lrange(&chunks),
        (_, "LPUSH") => parse_lpush(&chunks),
        _ => Err(format!("Command not recognized: {}", args)),
    }
}


fn parse_echo(chunks: &Vec<&str>) -> Result<Command, String> {
    let text = extract_text(chunks, 3)?;
    Ok(Command::ECHO(text))
}

fn parse_set(chunks: &Vec<&str>) -> Result<Command, String> {
    let key = extract_text(chunks, 3)?;
    let val = extract_text(chunks, 5)?;
    Ok(Command::SET(key, val, None))
}

fn parse_set_expiry(chunks: &Vec<&str>) -> Result<Command, String> {
    let key = extract_text(chunks, 3)?;
    let val = extract_text(chunks, 5)?;
    let expiry = extract_text(chunks, 7)?;

    if expiry.to_uppercase() != "PX" {
        return Err(format!("Unrecognized flag: {}", expiry));
    }

    let expire_str = chunks
        .get(10)
        .ok_or("Missing expire time")?;
    let millis = expire_str
        .parse::<u64>()
        .map_err(|_| format!("Expire time must be an integer, got: '{}'", expire_str))?;
    let exp_time = SystemTime::now() + Duration::from_millis(millis);

    Ok(Command::SET(
        key,
        val,
        Some(exp_time),
    ))
}

fn parse_get(chunks: &Vec<&str>) -> Result<Command, String> {
    let key = extract_text(chunks, 3)?;
    Ok(Command::GET(key))
}

fn parse_rpush(chunks: &Vec<&str>) -> Result<Command, String> {
    let key = extract_text(chunks, 3)?;
    let mut val_vec = Vec::<String>::new();
    for i in (5..chunks.len()).step_by(2){
        val_vec.push(extract_text(chunks, i)?);
    }
    Ok(Command::RPUSH(key, val_vec))
}

fn parse_lrange(chunks: &Vec<&str>) -> Result<Command, String> {
    let key = extract_text(chunks, 3)?;
    let start = extract_int(chunks, 5)?;
    let end = extract_int(chunks, 7)?;
    Ok(Command::LRANGE(key, start, end))
}

fn parse_lpush(chunks: &Vec<&str>) -> Result<Command, String> {
    let key = extract_text(chunks, 3)?;
    let mut val_vec = Vec::<String>::new();
    for i in (5..chunks.len()).step_by(2){
        val_vec.push(extract_text(chunks, i)?);
    }
    Ok(Command::LPUSH(key, val_vec))
}