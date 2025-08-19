use std::result::Result;

pub enum Command {
    PING,
    ECHO(i32, String),
}

pub fn parse_command(command: &str) -> Result<Command, &str> {
    println!("started parse");
    let chunks: Vec<&str> = command.split("\r\n").collect();
    println!("{:?}", chunks);
    if chunks.is_empty() {
        return Err("String is empty");
    }

    match chunks.get(0).map(|s| s.to_uppercase()).as_deref() {
        Some("+PING") => Ok(Command::PING),
        Some("*2") => {
            let length = chunks
                .get(1)
                .ok_or("Missing length")?
                .trim_start_matches("$")
                .parse::<i32>()
                .expect("Length must be a number");
            let com = chunks
                .get(2)
                .ok_or("Missing command")?
                .to_uppercase();
            if com.chars().count() as i32 != length {
                return  Err("Incorrect length");
            }
            match com.as_str() {
                "ECHO" => {
                    let text_length = chunks
                        .get(3)
                        .ok_or("Missing payload length")?
                        .trim_start_matches("$")
                        .parse::<i32>()
                        .expect("Text length must be a number");
                    let text = chunks
                        .get(4)
                        .ok_or("Missing text")?;
                    if text.len() as i32 != text_length {
                        return Err("Incorrect test length");
                    }
                    return Ok(Command::ECHO(text_length, text.to_string()));
                }
                _ => return  Err("Command not recognized")
            }
        }
        _ => Err("Command not recognized"),
    }
}
