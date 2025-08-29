use std::result::Result;
use std::time::{Duration, SystemTime};

pub enum Command {
    PING,
    ECHO(i32, String),
    SET(String, String, Option<SystemTime>),
    GET(String),
    RPUSH(String, String),
}

fn extract_text(chunks: &Vec<&str>, pos: usize) -> Result<String, String> {
    let text_length = chunks
        .get(pos)
        .ok_or("Missing length".to_string())?
        .trim_start_matches("$")
        .parse::<i32>()
        .map_err(|_| "Length must be an integer".to_string())?;
    let text = chunks.get(pos+1).ok_or("Missing text".to_string())?;

    if text.len() as i32 != text_length {
        return Err(format!("Actual text ({}:{}) doesn't match expected length ({})", text.len() as i32, text, text_length).to_string());
    }

    Ok(text.to_string())
}

pub fn parse_command(command: &str) -> Result<Command, String> {
    let chunks: Vec<&str> = command.split("\r\n").collect();
    let args = chunks.get(0).ok_or("Missing argument length".to_string())?;
    let com = extract_text(&chunks, 1)?;

    match (*args, com.as_str()) {
        ("*1", "PING") => Ok(Command::PING),
        ("*2", "ECHO") => parse_echo(&chunks),
        ("*3", "SET") => parse_set(&chunks),
        ("*5", "SET") => parse_set_expiry(&chunks),
        ("*2", "GET") => parse_get(&chunks),
        ("*3", "RPUSH") => parse_rpush(&chunks),
        _ => Err(format!("Command not recognized: {}", args)),
    }
}


fn parse_echo(chunks: &Vec<&str>) -> Result<Command, String> {
    let text = extract_text(chunks, 3)?;
    Ok(Command::ECHO(text.len() as i32, text))
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

    let millis = chunks
        .get(10)
        .ok_or("Missing expire time")?
        .parse::<u64>()
        .map_err(|e| format!("Expire time {e} must be an integer"))?;
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
    let val = extract_text(chunks, 5)?;
    Ok(Command::RPUSH(key, val))
}