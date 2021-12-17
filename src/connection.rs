use super::frame::Error::Incomplete;
use super::frame::Frame;
use crate::frame::Error;
use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use std::result::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Writer {
    // The `TcpStream`. It is decorated with a `BufWriter`, which provides write
    // level buffering. The `BufWriter` implementation provided by Tokio is
    // sufficient for our needs.
    stream: BufWriter<OwnedWriteHalf>,
}

impl Writer {
    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        self.write_value(frame).await?;
        self.stream.flush().await
    }
    /// Write a frame literal to the stream
    async fn write_value(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Error(val) => {
                let len = val.len() as u32;
                self.stream.write_u32(len).await?;
                self.stream.write_all(val.as_bytes()).await?;
            }
            Frame::Bulk(val) => {
                // let len = val.len() as u32;
                // self.stream.write_u32(len).await?;
                self.stream.write_all(val).await?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Reader {
    stream: OwnedReadHalf,
    // The buffer for reading frames.
    buffer: BytesMut,
}

impl Reader {
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, Error> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, Error> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);
                // Return the parsed frame to the caller.
                Ok(Some(frame))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

pub fn pair(stream: TcpStream) -> (Reader, Writer) {
    let (r, w) = stream.into_split();
    (
        Reader {
            stream: r,
            buffer: BytesMut::with_capacity(4 * 1024),
        },
        Writer {
            stream: BufWriter::new(w),
        },
    )
}
