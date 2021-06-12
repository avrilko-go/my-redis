use bytes::Bytes;
use std::time::{Duration};
use crate::parse::{Parse, ParseError::EndOfStream};

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
    expire: Option<Duration>,
}

impl Set {
    pub fn new(str: impl ToString, value: Bytes, expire: Option<Duration>) -> Set {
        Set {
            key: str.to_string(),
            value,
            expire,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Bytes {
        &self.value
    }

    pub fn expire(&self) -> Option<Duration> {
        self.expire
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Set> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;

        // 继续到期时间
        let mut expire = None;
        match parse.next_string() {
            Ok(s) if s.to_uppercase() == "EX" => {
                let number = parse.next_int()?;
                expire = Some(Duration::from_secs(number));
            }
            Ok(s) if s.to_uppercase() == "PX" => {
                let number = parse.next_int()?;
                expire = Some(Duration::from_millis(number));
            }
            Ok(_) => return Err("currently `SET` only supports the expiration option".into()),
            Err(EndOfStream) => {}
            Err(err) => return Err(err.into()),
        }

        Ok(Set {
            key,
            value,
            expire,
        })
    }
}