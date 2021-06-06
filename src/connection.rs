use crate::frame::Frame;
use tokio::io::{BufWriter, AsyncReadExt};
use tokio::net::TcpStream;
use bytes::BytesMut;
use std::io::Cursor;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // 当buff了里面没有内容的时候，将stream里面的内容拷贝进buff中

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    // 读不出内容了
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                };
            }
        }


        Ok(Some(Frame::Null))
    }

    pub fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        use crate::frame::Error;

        // 新建一个游标
        let mut buff = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buff) {
            Ok(_) => {}
            Err(Error::Incomplete) => {
                // 代表本次读取不完整，需要等到下次
                return Ok(None);
            }
            Err(err) => {
                // 读取真正出现了错误
                return Err(err.into());
            }
        }


        Ok(Some(Frame::Null))
    }
}