use crate::frame::Frame;
use std::vec::IntoIter;
use std::fmt::{Display, Formatter};
use bytes::{Bytes, Buf};

pub(crate) struct Parse {
    parts: IntoIter<Frame>,
}

#[derive(Debug)]
pub(crate) enum ParseError {
    EndOfStream,
    Other(crate::Error),
}


impl Parse {
    pub fn new(frame: Frame) -> Result<Parse, ParseError> {
        match frame {
            Frame::Array(arr) => {
                Ok(Parse {
                    parts: arr.into_iter()
                })
            }
            _ => {
                Err(ParseError::Other(format!("protocol error; expected array, got {:?}", frame).into()))
            }
        }
    }

    pub fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or_else(|| ParseError::EndOfStream)
    }

    pub fn next_string(&mut self) -> Result<String, ParseError> {
        let frame = self.next()?;
        match frame {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(b) => std::str::from_utf8(&b[..]).map(|s| s.to_string()).map_err(|_| "protocol error; invalid string".into()),
            frame => Err(format!(
                "protocol error; expected simple frame or bulk frame, got {:?}",
                frame
            )
                .into()),
        }
    }

    pub fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
        let frame = self.next()?;

        match frame {
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Frame::Bulk(b) => Ok(b),
            frame => Err(format!(
                "protocol error; expected simple frame or bulk frame, got {:?}",
                frame
            )
                .into()),
        }
    }

    pub fn next_int(&mut self) -> Result<u64, ParseError> {
        use atoi::atoi;
        const MSG: &str = "protocol error; invalid number";

        match self.next()? {
            Frame::Simple(s) => atoi::<u64>(s.as_bytes()).ok_or_else(|| MSG.into()),
            Frame::Integer(i) => Ok(i),
            Frame::Bulk(b) => {
                atoi::<u64>(&b).ok_or_else(|| MSG.into())
            }
            frame => Err(format!("protocol error; expected int frame but got {:?}", frame).into()),
        }
    }

    pub fn finish(&mut self) -> Result<(), ParseError> {
        if self.parts.next().is_none() {
            Ok(())
        } else {
            Err("protocol error; expected end of frame, but there was more".into())
        }
    }
}

impl std::error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            ParseError::Other(err) => err.fmt(f),
        }
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> Self {
        ParseError::Other(src.to_string().into())
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> Self {
        ParseError::Other(src.into())
    }
}



