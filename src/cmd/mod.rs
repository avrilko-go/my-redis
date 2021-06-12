use crate::frame::Frame;
use crate::db::Db;
use crate::connection::Connection;
use crate::shutdown::Shutdown;
use crate::parse::{Parse, ParseError};
use tracing::{debug};

mod get;

pub use get::Get;

mod set;

pub use set::Set;

mod unknown;

pub use unknown::Unknown;

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    UnKnown(Unknown),
}

impl Command {
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        let mut parse = Parse::new(frame)?;
        let command_name = parse.next_string()?;
        let command = match &command_name.to_lowercase()[..] {
            "get" => {
                Command::Get(Get::parse_frames(&mut parse)?)
            }
            "set" => {
                Command::Set(Set::parse_frames(&mut parse)?)
            }
            _ => {
                Command::UnKnown(Unknown::new(command_name))
            }
        };

        // 检查是否还有剩余的
        parse.finish()?;
        Ok(command)
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection, shutdown: &mut Shutdown) -> crate::Result<()> {
        match self {
            Command::Get(cmd) => {}
            Command::Set(cmd) => {}
            Command::UnKnown(cmd) => {}
        }


        Ok(())
    }
}

