use crate::frame::Frame;
use std::vec::IntoIter;
use std::fmt::{Display, Formatter};
use bytes::Bytes;

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



