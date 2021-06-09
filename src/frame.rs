use bytes::{Bytes, Buf};
use std::io::Cursor;
use std::fmt;
use std::fmt::Formatter;
use std::convert::TryInto;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

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
                Ok(())
            }
            b'$' => { // 多行字符串
                // 首先判断是不是Null -1\r\n (4个u8)
                if b'-' == peek_u8(src)? {
                    skip(src, 4)
                } else {
                    // 获取长度
                    let len: usize = get_decimal(src)?.try_into()?;
                    skip(src, len + 2)
                }
            }
            b':' => { // 数字
                get_decimal(src)?;
                Ok(())
            }
            b'-' => { // 错误消息
                get_line(src)?;
                Ok(())
            }
            b'*' => { // 数组
                // 获取数组长度
                let len: usize = get_decimal(src)?.try_into()?;

                for _i in 0..len {
                    Frame::check(src)?;
                }
                Ok(())
            }
            actual => {
                return Err(format!("非法的redis协议，非法字符 {}", actual).into());
            }
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            b'+' => { // 单行字符串
                let line = get_line(src)?.to_vec();
                let str = String::from_utf8(line)?;
                Ok(Frame::Simple(str))
            }
            b'$' => { // 多行字符串
                let len = get_decimal(src)?.try_into()?;
                let n = len + 2; // \r\n

                if src.remaining() < n {
                    return Err(Error::Incomplete);
                }

                let bytes = Bytes::copy_from_slice(&src.chunk()[..len]);
                skip(src, n)?;
                Ok(Frame::Bulk(bytes))
            }
            b':' => {
                let d = get_decimal(src)?;
                Ok(Frame::Integer(d))
            }
            b'-' => {
                let data = get_line(src)?.to_vec();
                let err_msg = String::from_utf8(data)?;
                Ok(Frame::Error(err_msg))
            }
            b'*' => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;
                    if line != b"-1" {
                        return Err("protocol error; invalid frame format".into());
                    }
                    Ok(Frame::Null)
                } else {
                    let len = get_decimal(src)?;

                    let mut arr = vec![];
                    for _i in 0..len {
                        let c = Frame::parse(src)?;
                        arr.push(c);
                    }
                    Ok(Frame::Array(arr))
                }
            }
            actual => {
                Err(format!("非法的redis协议，非法字符 {}", actual).into())
            }
        }
    }
}

// 这个函数返回一个[u8]的引用，需要确定其的生命周期
pub fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
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

pub fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        // 读不出内容了
        return Err(Error::Incomplete);
    }

    // 这个函数读出u8后会丢弃buff中读出那个u8
    Ok(src.get_u8())
}

pub fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        // 读不出内容了
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

pub fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;
    let line = get_line(src)?;
    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

pub fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }
    src.advance(n);
    Ok(())
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

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Self {
        "protocol error; invalid frame format".into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        "protocol error; invalid frame format".into()
    }
}
