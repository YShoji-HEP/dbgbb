//! # dbgbb!
//!
//! A framework for analyzing debugging data in a Mathematica/Jupyter notebook.
mod external;
mod reader;
mod rename;
mod sender;

#[doc(hidden)]
pub use array_object::{ArrayObject, Pack, TryConcat};

#[doc(hidden)]
pub use bulletin_board_common::*;

#[doc(hidden)]
pub use reader::read_bulletin;
pub use rename::Rename;
pub use sender::Buffer;

#[doc(hidden)]
pub use sender::SENDER;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[doc(hidden)]
pub static COUNTER: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[doc(hidden)]
pub static DATA_ACC: LazyLock<Mutex<HashMap<(String, String, String), Vec<ArrayObject>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Send the debug data to the server.
///
/// Usage:
/// ```
/// use dbgbb::dbgbb;
/// for a in 0..3 {
///     for b in 0..3 {
///         dbgbb!(a, b);
///         dbgbb!(every => 3, a, b);
///         dbgbb!(oneshot => 5, a, b);
///     }
/// }
/// ```
#[macro_export]
macro_rules! dbgbb {
    ($($x:expr),*) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Rename, Pack};
        let mut objs = vec![];
        $(
            let title = match $x.get_name() {
                Some(name) => name,
                None => stringify!($x).to_string(),
            };
            let tag = format!("{}:{}:{}", file!(), line!(), column!());
            let obj: dbgbb::ArrayObject = $x.clone().try_into().unwrap();
            objs.push((title, tag, obj));
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
///
/// Usage:
/// ```
/// use dbgbb::dbgbb_acc;
/// for a in 0..3 {
///     for b in 0..3 {
///         dbgbb_acc!(label => "i", a, b);
///         dbgbb_acc!(label => "j", every => 2, a, b);
///     }
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
            let title = match $x.get_name() {
                Some(name) => name,
                None => stringify!($x).to_string(),
            };
            let tag = format!("{}:{}:{}", file!(), line!(), column!());
            let entry = map.entry(($label.to_string(), title, tag)).or_insert(vec![]);
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
///
/// Usage:
/// ```
/// use dbgbb::dbgbb_read;
/// let vv: Vec<i64> = dbgbb_read!("vv");
/// let vv2: Vec<i64> = dbgbb_read!("vv", rev => 0);
/// let a: Vec<u64> = dbgbb_read!("a", "src/lib.rs:9:9");
/// let b: i64 = dbgbb_read!("b", "src/lib.rs:8:9", rev => 0);
/// ```
#[macro_export]
macro_rules! dbgbb_read {
    ($title:literal, $tag:literal, rev => $revision:literal) => {{
        let obj = dbgbb::read_bulletin(
            $title.to_string(),
            Some($tag.to_string()),
            Some($revision as u64),
        );
        obj.try_into().unwrap()
    }};
    ($title:literal, $tag:literal) => {{
        let obj = dbgbb::read_bulletin($title.to_string(), Some($tag.to_string()), None);
        obj.try_into().unwrap()
    }};
    ($title:literal, rev => $revision:literal) => {{
        let obj = dbgbb::read_bulletin($title.to_string(), None, Some($revision as u64));
        obj.try_into().unwrap()
    }};
    ($title:literal) => {{
        let obj = dbgbb::read_bulletin($title.to_string(), None, None);
        obj.try_into().unwrap()
    }};
}

/// Send each element to the server.
///
/// Usage:
/// ```
/// use dbgbb::dbgbb_flatten;
/// let a = vec![vec![1u32, 2], vec![3, 4]];
/// dbgbb_flatten!(a, depth => 1);
/// dbgbb_flatten!(a, depth => 2);
/// ```
#[macro_export]
macro_rules! dbgbb_flatten {
    ($x:expr, depth => 1) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for inner in $x.clone().iter() {
            let obj: dbgbb::ArrayObject = inner.clone().try_into().unwrap();
            objs.push((title.clone(), tag.clone(), obj));
        }
        sender.post(objs).unwrap();
    }};
    ($x:expr, depth => 2) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for inner0 in $x.clone().iter() {
            for inner1 in inner0.iter() {
                let obj: dbgbb::ArrayObject = inner1.clone().try_into().unwrap();
                objs.push((title.clone(), tag.clone(), obj));
            }
        }
        sender.post(objs).unwrap();
    }};
    ($x:expr, depth => 3) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for inner0 in $x.clone().iter() {
            for inner1 in inner0.iter() {
                for inner2 in inner1.iter() {
                    let obj: dbgbb::ArrayObject = inner2.clone().try_into().unwrap();
                    objs.push((title.clone(), tag.clone(), obj));
                }
            }
        }
        sender.post(objs).unwrap();
    }};
    ($x:expr, depth => 4) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for inner0 in $x.clone().iter() {
            for inner1 in inner0.iter() {
                for inner2 in inner1.iter() {
                    for inner3 in inner2.iter() {
                        let obj: dbgbb::ArrayObject = inner3.clone().try_into().unwrap();
                        objs.push((title.clone(), tag.clone(), obj));
                    }
                }
            }
        }
        sender.post(objs).unwrap();
    }};
}

/// Create a single array and send it to the server. The lengths of the elements should be the same.
///
/// Usage:
/// ```
/// use dbgbb::dbgbb_concat;
/// let a = vec![vec![1u32, 2], vec![3, 4]];
/// dbgbb_concat!(a, depth => 1);
/// ```
#[macro_export]
macro_rules! dbgbb_concat {
    ($x:expr, depth => 1) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename, TryConcat};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for inner in $x.clone().iter() {
            let obj: dbgbb::ArrayObject = inner.clone().try_into().unwrap();
            objs.push(obj);
        }
        let cat = objs.try_concat().unwrap();
        sender.post(vec![(title, tag, cat)]).unwrap();
    }};
    ($x:expr, depth => 2) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename, TryConcat};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs0 = vec![];
        for inner0 in $x.clone().iter() {
            let mut objs1 = vec![];
            for inner1 in inner0.iter() {
                let obj: dbgbb::ArrayObject = inner1.clone().try_into().unwrap();
                objs1.push(obj);
            }
            let cat = objs1.try_concat().unwrap();
            objs0.push(cat);
        }
        let cat = objs0.try_concat().unwrap();
        sender.post(vec![(title, tag, cat)]).unwrap();
    }};
    ($x:expr, depth => 3) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename, TryConcat};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs0 = vec![];
        for inner0 in $x.clone().iter() {
            let mut objs1 = vec![];
            for inner1 in inner0.iter() {
                let mut objs2 = vec![];
                for inner2 in inner1.iter() {
                    let obj: dbgbb::ArrayObject = inner2.clone().try_into().unwrap();
                    objs2.push(obj);
                }
                let cat = objs2.try_concat().unwrap();
                objs1.push(cat);
            }
            let cat = objs1.try_concat().unwrap();
            objs0.push(cat);
        }
        let cat = objs0.try_concat().unwrap();
        sender.post(vec![(title, tag, cat)]).unwrap();
    }};
    ($x:expr, depth => 4) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename, TryConcat};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs0 = vec![];
        for inner0 in $x.clone().iter() {
            let mut objs1 = vec![];
            for inner1 in inner0.iter() {
                let mut objs2 = vec![];
                for inner2 in inner1.iter() {
                    let mut objs3 = vec![];
                    for inner3 in inner2.iter() {
                        let obj: dbgbb::ArrayObject = inner3.clone().try_into().unwrap();
                        objs3.push(obj);
                    }
                    let cat = objs3.try_concat().unwrap();
                    objs2.push(cat);
                }
                let cat = objs2.try_concat().unwrap();
                objs1.push(cat);
            }
            let cat = objs1.try_concat().unwrap();
            objs0.push(cat);
        }
        let cat = objs0.try_concat().unwrap();
        sender.post(vec![(title, tag, cat)]).unwrap();
    }};
}

/// Send each element to the server adding the index to the tag.
///
/// Usage:
/// ```
/// use dbgbb::dbgbb_index;
/// let a = vec![vec![1u32, 2], vec![3, 4]];
/// dbgbb_index!(a, depth => 1);
/// dbgbb_index!(a, depth => 2);
/// ```
#[macro_export]
macro_rules! dbgbb_index {
    ($x:expr, depth => 1) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for (i, inner) in $x.clone().iter().enumerate() {
            let obj: dbgbb::ArrayObject = inner.clone().try_into().unwrap();
            objs.push((title.clone(), format!("{tag}:[{i}]"), obj));
        }
        sender.post(objs).unwrap();
    }};
    ($x:expr, depth => 2) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for (i, inner0) in $x.clone().iter().enumerate() {
            for (j, inner1) in inner0.iter().enumerate() {
                let obj: dbgbb::ArrayObject = inner1.clone().try_into().unwrap();
                objs.push((title.clone(), format!("{tag}:[{i},{j}]"), obj));
            }
        }
        sender.post(objs).unwrap();
    }};
    ($x:expr, depth => 3) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for (i, inner0) in $x.clone().iter().enumerate() {
            for (j, inner1) in inner0.iter().enumerate() {
                for (k, inner2) in inner1.iter().enumerate() {
                    let obj: dbgbb::ArrayObject = inner2.clone().try_into().unwrap();
                    objs.push((title.clone(), format!("{tag}:[{i},{j},{k}]"), obj));
                }
            }
        }
        sender.post(objs).unwrap();
    }};
    ($x:expr, depth => 4) => {{
        let sender = dbgbb::SENDER.lock().unwrap();
        use dbgbb::{Pack, Rename};
        let title = match $x.get_name() {
            Some(name) => name,
            None => stringify!($x).to_string(),
        };
        let tag = format!("{}:{}:{}", file!(), line!(), column!());
        let mut objs = vec![];
        for (i, inner0) in $x.clone().iter().enumerate() {
            for (j, inner1) in inner0.iter().enumerate() {
                for (k, inner2) in inner1.iter().enumerate() {
                    for (l, inner3) in inner2.iter().enumerate() {
                        let obj: dbgbb::ArrayObject = inner3.clone().try_into().unwrap();
                        objs.push((title.clone(), format!("{tag}:[{i},{j},{k},{l}]"), obj));
                    }
                }
            }
        }
        sender.post(objs).unwrap();
    }};
}
