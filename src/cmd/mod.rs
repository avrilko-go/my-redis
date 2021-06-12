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


#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
}

impl Command {
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        let mut parse = Parse::new(frame)?;
        let command_name = parse.next_string()?;

        let command = match &command_name[..] {
            "get" => {
                Command::Get(Get::parse_frames(&mut parse)?)
            }
            "set" => {
                Command::Set(Set::parse_frames(&mut parse)?)
            }
            _ => {
                Command::Get(Get::new("1".to_string()))
            }
        };
        println!("{:?}", command);

        Ok(Command::Get(Get::new("syr".to_string())))
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection, shutdown: &mut Shutdown) -> crate::Result<()> {
        Ok(())
    }
}

