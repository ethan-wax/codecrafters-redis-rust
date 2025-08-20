use std::result::Result;

pub enum Command {
    PING,
    ECHO(i32, String),
}

pub fn parse_command(command: &str) -> Result<Command, &str> {
    let chunks: Vec<&str> = command.split("\r\n").collect();
    let args = chunks.get(0).ok_or("Missing argument length")?;
    let length = chunks
        .get(1)
        .ok_or("Missing command length")?
        .trim_start_matches("$")
        .parse::<i32>()
        .map_err(|_| "Length must be an integer")?;
    let command_name = chunks.get(2).ok_or("Missing command")?.to_uppercase();

    if command_name.len() as i32 != length {
        return Err("Actual command length doesn't match expected length");
    }

    match (*args, command_name.as_str()) {
        ("*1", "PING") => Ok(Command::PING),
        ("*2", "ECHO") => {
            let text_length = chunks
                .get(3)
                .ok_or("Missing text length")?
                .trim_start_matches("$")
                .parse::<i32>()
                .map_err(|_| "Length must be an integer")?;
            let text = chunks.get(4).ok_or("Missing text")?;

            if text.len() as i32 != text_length {
                return Err("Actual text length doesn't match expected length");
            }

            return Ok(Command::ECHO(text_length, text.to_string()));
        }
        _ => Err("Command not recognized"),
    }
}
