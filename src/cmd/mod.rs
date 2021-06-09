use crate::frame::Frame;
use crate::db::Db;
use crate::connection::Connection;
use crate::shutdown::Shutdown;
use crate::parse::{Parse, ParseError};
use tracing::{debug};

#[derive(Debug)]
pub enum Command {
    get
}

impl Command {
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        let mut parse = Parse::new(frame)?;
        let command = parse.next_string()?;
        println!("{}", command);

        Ok(Command::get)
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection, shutdown: &mut Shutdown) -> crate::Result<()> {
        Ok(())
    }
}

