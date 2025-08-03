#[cfg(not(feature = "unix"))]
pub use std::net::TcpStream as TcpOrUnixStream;
#[cfg(feature = "unix")]
pub use std::os::unix::net::UnixStream as TcpOrUnixStream;

use crate::{Operation, Response, SENDER};
use array_object::{ArrayObject, Unpack};
use serde_bytes::ByteBuf;

/// [Only for internal use] Helper function for `dbgbb_read!(...)`.
pub fn read_bulletin(title: String, tag: Option<String>, revision: Option<u64>) -> ArrayObject {
    let revisions = match revision {
        Some(rev) => vec![rev],
        None => vec![],
    };
    let sender = SENDER.lock().unwrap();
    let addr = sender.get_addr();
    let mut stream = TcpOrUnixStream::connect(&addr).unwrap();
    let mut buffer = std::io::Cursor::new(vec![]);
    ciborium::into_writer(&Operation::Read, &mut buffer).unwrap();
    ciborium::into_writer(&(title, tag, revisions), &mut buffer).unwrap();
    buffer.set_position(0);
    std::io::copy(&mut buffer, &mut stream).unwrap();
    let res = ciborium::from_reader(&mut stream).unwrap();
    match res {
        Response::Ok => {
            let val: ByteBuf = ciborium::from_reader(&mut stream).unwrap();
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            ArrayObject::unpack(val.to_vec()).unwrap()
        }
        Response::NotFound => {
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            panic!("Not found.");
        }
        Response::NotUnique(list) => {
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            panic!("Multiple entries found: {:?}.", list);
        }
    }
}
