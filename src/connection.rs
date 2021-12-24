use super::frame::Frame;
use crate::frame::Error;
use bytes::Buf;
use std::io::{self, Cursor};
use std::result::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
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
    // buffer: BytesMut,
    buffer: Vec<u8>,
    cursor: usize,
}

impl Reader {
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, Error> {
        loop {
            // Read into the buffer, tracking the number of bytes read
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;
            if 0 == n {
                return if self.cursor == 0 {
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                };
            } else {
                // Update our cursor
                self.cursor += n;
            }

            // Bytes.from(body.length) ++ body
            // total = 4 + body.length ++ body
            let mut buf = Cursor::new(&self.buffer[..]);
            let size = buf.get_u32() as usize;
            // info!("--size = {}", size);
            // info!("--remaining = {}", buf.remaining());
            let remain = buf.remaining();
            if size <= remain {
                let (left, right) = self.buffer.split_at(size + 4);
                let data = left.into();
                self.buffer = self.buffer.split_off(size + 4);
                self.cursor = self.cursor - size - 4;
                // Return the parsed frame to the caller.
                return Ok(Some(Frame::Bulk(data)));
            }
            // Ensure the buffer has capacity
            if self.buffer.len() == self.cursor {
                // Grow the buffer
                self.buffer.resize(self.cursor * 2, 0);
            }
        }
    }
}

pub fn pair(stream: TcpStream) -> (Reader, Writer) {
    let (r, w) = stream.into_split();
    (
        Reader {
            stream: r,
            // buffer: BytesMut::with_capacity(4 * 1024),
            // Allocate the buffer with 4kb of capacity.
            buffer: vec![0; 4096],
            cursor: 0,
        },
        Writer {
            stream: BufWriter::new(w),
        },
    )
}
