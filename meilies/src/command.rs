use std::{fmt, str, string};
use crate::codec::RespValue;

pub fn arguments_from_resp_value(value: RespValue) -> Result<Vec<Vec<u8>>, ()> {
    match value {
        RespValue::Array(Some(array)) => {
            let mut args = Vec::new();

            for value in array {
                match value {
                    RespValue::BulkString(Some(buffer)) => {
                        args.push(buffer);
                    },
                    _ => return Err(()),
                }
            }

            Ok(args)
        },
        _ => Err(())
    }
}

pub enum Command {
    Publish { stream: String, event: Vec<u8> },
    Subscribe { stream: String, from: i64 },
}

impl fmt::Debug for Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Publish { stream, event } => {
                let mut dbg = fmt.debug_struct("Publish");
                dbg.field("stream", &stream);
                match str::from_utf8(&event) {
                    Ok(event) => dbg.field("event", &event),
                    Err(_) => dbg.field("event", &event),
                };
                dbg.finish()
            },
            Command::Subscribe { stream, from } => {
                fmt.debug_struct("Subscribe")
                    .field("stream", &stream)
                    .field("from", &from)
                    .finish()
            }
        }
    }
}

#[derive(Debug)]
pub enum CommandError {
    CommandNotFound,
    MissingCommandName,
    InvalidNumberOfArguments { expected: usize },
    InvalidUtf8String(str::Utf8Error),
}

impl fmt::Display for CommandError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::CommandNotFound => {
                write!(fmt, "command not found")
            },
            CommandError::MissingCommandName => {
                write!(fmt, "missing command name")
            },
            CommandError::InvalidNumberOfArguments { expected } => {
                write!(fmt, "invalid number of arguments (expected {})", expected)
            },
            CommandError::InvalidUtf8String(error) => {
                write!(fmt, "invalid utf8 string: {}", error)
            },
        }
    }
}

impl From<str::Utf8Error> for CommandError {
    fn from(error: str::Utf8Error) -> CommandError {
        CommandError::InvalidUtf8String(error)
    }
}

impl From<string::FromUtf8Error> for CommandError {
    fn from(error: string::FromUtf8Error) -> CommandError {
        CommandError::InvalidUtf8String(error.utf8_error())
    }
}

impl Command {
    pub fn from_args(mut args: Vec<Vec<u8>>) -> Result<Command, CommandError> {
        let mut args = args.drain(..);

        let command = match args.next() {
            Some(command) => str::from_utf8(&command)?.to_lowercase(),
            None => return Err(CommandError::MissingCommandName),
        };

        match command.as_str() {
            "publish" => {
                match (args.next(), args.next(), args.next()) {
                    (Some(stream), Some(event), None) => {
                        let stream = String::from_utf8(stream)?;
                        Ok(Command::Publish { stream, event })
                    },
                    _ => Err(CommandError::InvalidNumberOfArguments { expected: 2 })
                }
            },
            "subscribe" => {
                match (args.next(), args.next()) {
                    (Some(mut stream), None) => {
                        match stream.iter().position(|c| *c == b':') {
                            Some(colon_offset) => {
                                let from = stream.split_off(colon_offset + 1);
                                stream.pop(); // remove the colon itself

                                let stream = String::from_utf8(stream)?;

                                let from = str::from_utf8(&from)?;
                                let from = i64::from_str_radix(from, 10).unwrap();

                                Ok(Command::Subscribe { stream, from })
                            },
                            None => {
                                let stream = String::from_utf8(stream)?;
                                Ok(Command::Subscribe { stream, from: -1 })
                            }
                        }
                    },
                    _ => Err(CommandError::InvalidNumberOfArguments { expected: 2 })
                }
            },
            _ => Err(CommandError::CommandNotFound),
        }
    }
}
