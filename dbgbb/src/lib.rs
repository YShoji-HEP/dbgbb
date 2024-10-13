//! # dbgbb!
//! 
mod external;
mod rename;
mod sender;

#[cfg(not(feature = "unix"))]
pub use std::net::TcpStream as TcpOrUnixStream;
#[cfg(feature = "unix")]
pub use std::os::unix::net::UnixStream as TcpOrUnixStream;

pub use array_object::{ArrayObject, Pack, TryConcat, Unpack};
pub use bulletin_board_common::*;
pub use ciborium;
pub use rename::Rename;
pub use sender::Buffer;
pub use sender::SENDER;
pub use serde_bytes;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Used to count the numbers.
pub static COUNTER: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
/// Used to accumulate data before sending.
pub static DATA_ACC: LazyLock<Mutex<HashMap<(String, String, String), Vec<ArrayObject>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Send the debug data to the server.
#[macro_export]
macro_rules! dbgbb {
    ($($x:expr),*) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Rename, Pack};
        let mut buffer = std::io::Cursor::new(vec![]);
        $(
            dbgbb::ciborium::into_writer(&dbgbb::Operation::Post, &mut buffer).unwrap();
            let var_name = match $x.get_name() {
                Some(name) => name,
                None => stringify!($x).to_string(),
            };
            let var_tag = format!("{}:{}:{}", file!(), line!(), column!());
            let obj: dbgbb::ArrayObject = $x.clone().try_into().unwrap();
            let val = dbgbb::serde_bytes::ByteBuf::from(obj.pack());
            dbgbb::ciborium::into_writer(&(var_name, var_tag, val), &mut buffer).unwrap();
        )*
        sender.send(Some(buffer)).unwrap();
    }};
    (every => $n:literal, $($x:expr),*) => {{
        let mut map = dbgbb::COUNTER.lock().unwrap();
        let count = map.entry(format!("{}:{}:{}", file!(), line!(), column!())).or_insert(0);
        if *count % $n == 0 {
            dbgbb!($($x),*);
        }
        *count += 1;
    }};
    (oneshot => $n:literal, $($x:expr),*) => {{
        let mut map = dbgbb::COUNTER.lock().unwrap();
        let count = map.entry(format!("{}:{}:{}", file!(), line!(), column!())).or_insert(0);
        if *count == $n {
            dbgbb!($($x),*);
        }
        *count += 1;
    }};
}

/// Accumulate and send the debug data to the server.
#[macro_export]
macro_rules! dbgbb_acc {
    (label => $label:literal, $($x:expr),*) => {{
        use dbgbb::Rename;
        let mut map = dbgbb::DATA_ACC.lock().unwrap();
        $(
            let var_name = match $x.get_name() {
                Some(name) => name,
                None => stringify!($x).to_string(),
            };
            let var_tag = format!("{}:{}:{}", file!(), line!(), column!());
            let entry = map.entry(($label.to_string(), var_name, var_tag)).or_insert(vec![]);
            let obj: dbgbb::ArrayObject = $x.clone().try_into().unwrap();
            entry.push(obj);
        )*
    }};
    (label => $label:literal, every => $n:literal, $($x:expr),*) => {{
        let mut map = dbgbb::COUNTER.lock().unwrap();
        let count = map.entry(format!("{}:{}:{}", file!(), line!(), column!())).or_insert(0);
        if *count % $n == 0 {
            dbgbb_acc!(label => $label, $($x),*);
        }
        *count += 1;
    }};
    ($label:literal => post) => {{
        use dbgbb::{Pack, TryConcat};
        let mut buffer = std::io::Cursor::new(vec![]);
        let sender = dbgbb::SENDER.lock().unwrap();
        let mut map = dbgbb::DATA_ACC.lock().unwrap();
        let keys: Vec<_> = map.keys()
            .filter(|key| key.0 == $label)
            .map(|key|key.clone())
            .collect();
        for key in keys {
            let obj = map.remove(&key).unwrap().try_concat().unwrap();
            dbgbb::ciborium::into_writer(&dbgbb::Operation::Post, &mut buffer).unwrap();
            let val = dbgbb::serde_bytes::ByteBuf::from(obj.pack());
            dbgbb::ciborium::into_writer(&(key.1, key.2, val), &mut buffer).unwrap();
        }
        sender.send(Some(buffer)).unwrap();
    }};
}

/// Read data from the server.
#[macro_export]
macro_rules! dbgbb_read {
    ($var_name:literal, $var_tag:literal, $revision:literal) => {{
        use dbgbb::Unpack;
        let sender = dbgbb::SENDER.lock().unwrap();
        let addr = sender.get_addr();
        let mut stream = dbgbb::TcpOrUnixStream::connect(&addr).unwrap();
        let mut buffer = std::io::Cursor::new(vec![]);
        dbgbb::ciborium::into_writer(&dbgbb::Operation::Read, &mut buffer).unwrap();
        dbgbb::ciborium::into_writer(
            &(
                $var_name.to_string(),
                Some($var_tag.to_string()),
                $revision as u64,
            ),
            &mut buffer,
        )
        .unwrap();
        buffer.set_position(0);
        std::io::copy(&mut buffer, &mut stream).unwrap();
        let val: serde_bytes::ByteBuf = dbgbb::ciborium::from_reader(&mut stream).unwrap();
        let obj = dbgbb::ArrayObject::unpack(val.to_vec());
        obj.try_into().unwrap()
    }};
    ($var_name:literal, $var_tag:literal) => {{
        use dbgbb::Unpack;
        let sender = dbgbb::SENDER.lock().unwrap();
        let addr = sender.get_addr();
        let mut stream = dbgbb::TcpOrUnixStream::connect(&addr).unwrap();
        let mut buffer = std::io::Cursor::new(vec![]);
        dbgbb::ciborium::into_writer(&dbgbb::Operation::Read, &mut buffer).unwrap();
        dbgbb::ciborium::into_writer::<(_, _, Option<u64>), _>(
            &($var_name.to_string(), Some($var_tag.to_string()), None),
            &mut buffer,
        )
        .unwrap();
        buffer.set_position(0);
        std::io::copy(&mut buffer, &mut stream).unwrap();
        let val: serde_bytes::ByteBuf = dbgbb::ciborium::from_reader(&mut stream).unwrap();
        let obj = dbgbb::ArrayObject::unpack(val.to_vec());
        obj.try_into().unwrap()
    }};
    ($var_name:literal) => {{
        use dbgbb::Unpack;
        let sender = dbgbb::SENDER.lock().unwrap();
        let addr = sender.get_addr();
        let mut stream = dbgbb::TcpOrUnixStream::connect(&addr).unwrap();
        let mut buffer = std::io::Cursor::new(vec![]);
        dbgbb::ciborium::into_writer(&dbgbb::Operation::Read, &mut buffer).unwrap();
        dbgbb::ciborium::into_writer::<(_, Option<String>, Option<u64>), _>(
            &($var_name.to_string(), None, None),
            &mut buffer,
        )
        .unwrap();
        buffer.set_position(0);
        std::io::copy(&mut buffer, &mut stream).unwrap();
        let val: serde_bytes::ByteBuf = dbgbb::ciborium::from_reader(&mut stream).unwrap();
        let obj = dbgbb::ArrayObject::unpack(val.to_vec()).unwrap();
        obj.try_into().unwrap()
    }};
}
