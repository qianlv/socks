use super::next_token;
use bytes::{Buf, BufMut, BytesMut};
use mio::net::TcpStream;
use mio::Token;
use std::io;
use std::io::prelude::*;

const BUF_SIZE: usize = 65536;

struct Stream {
    conn: TcpStream,
    token: Token,
    buf: BytesMut,
    wbuf: BytesMut,
}

impl Stream {
    fn new(conn: TcpStream) -> Self {
        Stream {
            conn,
            token: unsafe { next_token() },
            buf: BytesMut::new(),
            wbuf: BytesMut::new(),
        }
    }

    fn read(&mut self) -> bool {
        let mut context = [0u8; BUF_SIZE];
        match self.conn.read(&mut context) {
            Ok(n) if n > 0 => {
                self.buf.put_slice(&context[0..n]);
            }
            Ok(n) if n == 0 => {
                return false;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => {
                eprintln!(
                    "In Stream::read {} {} {}",
                    self.conn.local_addr().unwrap(),
                    self.conn.peer_addr().unwrap(),
                    e
                );
                return false;
            }
            _ => {}
        }
        true
    }

    fn write(&mut self, msg: &[u8]) -> bool {
        self.wbuf.put_slice(msg);
        match self.conn.write(self.wbuf.as_ref()) {
            Ok(n) => true,
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock
                    || e.kind() == io::ErrorKind::Interrupted =>
            {
                true
            }
            Err(e) => {
                eprintln!(
                    "In Stream::write {} {} {}",
                    self.conn.local_addr().unwrap(),
                    self.conn.peer_addr().unwrap(),
                    e
                );
                false
            }
        }
    }
}

pub struct Connect {
    client: Stream,
    remote: Option<Stream>,
}

impl Connect {
    pub fn new(conn: TcpStream) -> Self {
        Connect {
            client: Stream::new(conn),
            remote: None,
        }
    }

    pub fn handle_read(&mut self) {}
}
