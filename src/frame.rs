use bytes::{Bytes, Buf};
use std::io::Cursor;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            b'+' => { // 单行字符串
                get_line(src)?;
                return Ok(());
            }
            actual => {
                return Err(format!("非法的redis协议，非法字符 {}", actual).into());
            }
        }
        Ok(())
    }
}

// 这个函数返回一个[u8]的引用，需要确定其的生命周期
pub fn get_line<'a>(mut src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    // redis 一行是以\r\n结束的
    let start = src.position() as usize;
    // 只用循环到倒数第二个
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::Incomplete)
}

pub fn get_u8(mut src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        // 读不出内容了
        return Err(Error::Incomplete);
    }

    // 这个函数读出u8后会丢弃buff中读出那个u8
    Ok(src.get_u8())
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Self {
        Error::Other(src.to_string().into())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Incomplete => {
                "stream end early".fmt(fmt)
            }
            Error::Other(err) => {
                err.fmt(fmt)
            }
        }
    }
}
