use crate::parse::Parse;
use crate::db::Db;
use crate::connection::Connection;

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn new(str: impl ToString) -> Get {
        Get {
            key: str.to_string()
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Get> {
        let key = parse.next_string()?;
        Ok(Get::new(key))
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {}
}