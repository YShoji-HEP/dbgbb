//! # dbgbb!
//!
mod external;
mod reader;
mod rename;
mod sender;

pub use array_object::{ArrayObject, Pack, TryConcat};
pub use bulletin_board_common::*;
pub use reader::read_bulletin;
pub use rename::Rename;
pub use sender::Buffer;
pub use sender::SENDER;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Used to count the numbers.
pub static COUNTER: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
/// Used to accumulate data before sending.
pub static DATA_ACC: LazyLock<Mutex<HashMap<(String, String, String), Vec<ArrayObject>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Send the debug data to the server.
/// Usage:
/// ```
/// dbgbb!(a, b, ...);
/// dbgbb!(every => 3, a, b, ...);
/// dbgbb!(oneshot => 5, a, b, ...);
/// ```
#[macro_export]
macro_rules! dbgbb {
    ($($x:expr),*) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Rename, Pack};
        let mut objs = vec![];
        $(
            let var_name = match $x.get_name() {
                Some(name) => name,
                None => stringify!($x).to_string(),
            };
            let var_tag = format!("{}:{}:{}", file!(), line!(), column!());
            let obj: dbgbb::ArrayObject = $x.clone().try_into().unwrap();
            objs.push((var_name, var_tag, obj));
        )*
        sender.post(objs).unwrap();
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
/// Usage:
/// ```
/// for _ in 0..10 {
///     dbgbb_acc!(label => "i", a, b, ...);
///     dbgbb_acc!(label => "j", every => 3, a, b, ...);
/// }
/// dbgbb_acc!("i" => post);
/// dbgbb_acc!("j" => post);
/// ```
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
        let mut objs = vec![];
        let sender = dbgbb::SENDER.lock().unwrap();
        let mut map = dbgbb::DATA_ACC.lock().unwrap();
        let keys: Vec<_> = map.keys()
            .filter(|key| key.0 == $label)
            .map(|key|key.clone())
            .collect();
        for key in keys {
            let obj = map.remove(&key).unwrap().try_concat().unwrap();
            objs.push((key.1, key.2, obj));
        }
        sender.post(objs).unwrap();
    }};
}

/// Read data from the server.
/// Usage:
/// ```
/// let a: Vec<u32> = dbgbb_read!("a");
/// let b: Vec<f64> = dbgbb_read!("b", "tag1");
/// let c: i64 = dbgbb_read!("c", "tag2", 0);
/// ```
#[macro_export]
macro_rules! dbgbb_read {
    ($var_name:literal, $var_tag:literal, $revision:literal) => {{
        let obj = dbgbb::read_bulletin(
            $var_name.to_string(),
            Some($var_tag.to_string()),
            $revision as u64,
        );
        obj.try_into().unwrap()
    }};
    ($var_name:literal, $var_tag:literal) => {{
        let obj = dbgbb::read_bulletin($var_name.to_string(), Some($var_tag.to_string()), None);
        obj.try_into().unwrap()
    }};
    ($var_name:literal) => {{
        let obj = dbgbb::read_bulletin($var_name.to_string(), None, None);
        obj.try_into().unwrap()
    }};
}
