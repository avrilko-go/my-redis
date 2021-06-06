use crate::frame::Frame;
use crate::db::Db;
use crate::connection::Connection;
use crate::shutdown::Shutdown;

#[derive(Debug)]
pub enum Command {
    get
}

impl Command {
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        Ok(Command::get)
    }

    pub async fn apply(self, db: &Db, dst: &mut Connection, shutdown: &mut Shutdown) -> crate::Result<()> {
        println!("{}", 1);
        Ok(())
    }
}