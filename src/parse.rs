use std::result::Result;

pub enum Command {
    PING,
    ECHO(i32, String),
    SET(String, String),
    GET(String)
}

pub fn parse_command(command: &str) -> Result<Command, String> {
    let chunks: Vec<&str> = command.split("\r\n").collect();
    let args = chunks.get(0).ok_or("Missing argument length".to_string())?;
    let length = chunks
        .get(1)
        .ok_or("Missing command length".to_string())?
        .trim_start_matches("$")
        .parse::<i32>()
        .map_err(|_| "Length must be an integer".to_string())?;
    let command_name = chunks.get(2).ok_or("Missing command".to_string())?.to_uppercase();

    if command_name.len() as i32 != length {
        return Err("Actual command length doesn't match expected length".to_string());
    }

    match (*args, command_name.as_str()) {
        ("*1", "PING") => Ok(Command::PING),
        ("*2", "ECHO") => {
            let text_length = chunks
                .get(3)
                .ok_or("Missing text length".to_string())?
                .trim_start_matches("$")
                .parse::<i32>()
                .map_err(|_| "Length must be an integer".to_string())?;
            let text = chunks.get(4).ok_or("Missing text".to_string())?;

            if text.len() as i32 != text_length {
                return Err("Actual text length doesn't match expected length".to_string());
            }

            Ok(Command::ECHO(text_length, text.to_string()))
        }
        ("*3", "SET") => {
            let key_length = chunks
                .get(3)
                .ok_or("Missing key length".to_string())?
                .trim_start_matches("$")
                .parse::<i32>()
                .map_err(|_| "Key length must be an integer".to_string())?;
            let key = chunks.get(4).ok_or("Missing key".to_string())?;

            if key.len() as i32 != key_length {
                return Err("Actual key length doesn't match expected length".to_string());
            }

            let val_length = chunks
                .get(5)
                .ok_or("Missing val length".to_string())?
                .trim_start_matches("$")
                .parse::<i32>()
                .map_err(|_| "Val length must be an integer".to_string())?;
            let val = chunks.get(6).ok_or("Missing val".to_string())?;
            
            if val.len() as i32 != val_length {
                return Err("Actual val length doesn't match expected length".to_string());
            }

            Ok(Command::SET(key.to_string(), val.to_string()))
        }
        ("*2", "GET") => {
            let key_length = chunks
                .get(3)
                .ok_or("Missing key length".to_string())?
                .trim_start_matches("$")
                .parse::<i32>()
                .map_err(|_| "Length must be an integer".to_string())?;
            let key = chunks.get(4).ok_or("Missing key".to_string())?;

            if key.len() as i32 != key_length {
                return Err("Actual key length doesn't match expected length".to_string());
            }

            Ok(Command::GET(key.to_string()))
        }
        _ => Err(format!("Command not recognized: {}", args)),
    }
}
