#[cfg(not(feature = "unix"))]
use std::net::TcpStream as TcpOrUnixStream;
#[cfg(feature = "unix")]
use std::os::unix::net::UnixStream as TcpOrUnixStream;

use crate::Operation;
use array_object::{ArrayObject, Pack};
use serde_bytes::ByteBuf;
use std::io::{self, Cursor};
use std::sync::mpsc::{self, Sender};
use std::sync::{LazyLock, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Utility for sending data.
pub static SENDER: LazyLock<Mutex<BufferedSender>> =
    LazyLock::new(|| Mutex::new(BufferedSender::new()));

/// Buffer for sending the data over TCP.
pub struct Buffer {}

impl Buffer {
    /// Enable the buffer.
    pub fn on() -> Self {
        let mut sender = SENDER.lock().unwrap();

        let timeout = std::env::var("BB_TIMEOUT")
            .unwrap_or("3000".to_string())
            .parse()
            .unwrap_or(500);
        let interval = std::env::var("BB_INTERVAL")
            .unwrap_or("1000".to_string())
            .parse()
            .unwrap_or(1000);
        sender.start(timeout, interval);
        Self {}
    }
    /// Disable the buffer.
    pub fn off(&self) {
        let mut sender = SENDER.lock().unwrap();
        sender.join();
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.off();
    }
}

pub struct BufferedSender {
    addr: String,
    handle: Option<(JoinHandle<()>, Sender<SenderControl>)>,
}

pub enum SenderControl {
    Post((String, String, ArrayObject)),
    Shutdown,
}

impl BufferedSender {
    fn new() -> Self {
        #[cfg(not(feature = "unix"))]
        let addr = std::env::var("BB_ADDR").unwrap_or("127.0.0.1:7578".to_string());
        #[cfg(feature = "unix")]
        let addr = std::env::var("BB_ADDR").unwrap_or("/tmp/bb.sock".to_string());
        Self { addr, handle: None }
    }
    pub fn get_addr(&self) -> &String {
        &self.addr
    }
    fn start(&mut self, timeout: u64, interval: u64) {
        let (tx, rx) = mpsc::channel::<SenderControl>();
        let addr = self.addr.clone();
        let handle = std::thread::spawn(move || {
            let mut buffer = Cursor::new(vec![]);
            let mut time = Instant::now();
            loop {
                if let Ok(ctl) = rx.recv_timeout(Duration::from_millis(timeout)) {
                    match ctl {
                        SenderControl::Post((title, tag, obj)) => {
                            let data = ByteBuf::from(obj.pack());
                            ciborium::into_writer(&Operation::Post, &mut buffer).unwrap();
                            ciborium::into_writer(&(title, tag, data), &mut buffer).unwrap();
                        }
                        SenderControl::Shutdown => {
                            if buffer.position() > 0 {
                                let mut stream = TcpOrUnixStream::connect(&addr).unwrap();
                                buffer.set_position(0);
                                io::copy(&mut buffer, &mut stream).unwrap();
                            }
                            break;
                        }
                    }
                }
                let now = Instant::now();
                if buffer.position() > 0 && now - time > Duration::from_millis(interval) {
                    time = now;
                    let mut stream = TcpOrUnixStream::connect(&addr).unwrap();
                    buffer.set_position(0);
                    io::copy(&mut buffer, &mut stream).unwrap();
                    buffer = Cursor::new(vec![]);
                }
            }
        });
        self.handle = Some((handle, tx));
    }
    pub fn post(
        &self,
        objs: Vec<(String, String, ArrayObject)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &self.handle {
            Some((_, tx)) => {
                for obj in objs {
                    tx.send(SenderControl::Post(obj))?;
                }
            }
            None => {
                if !objs.is_empty() {
                    let mut buffer = Cursor::new(vec![]);
                    for (title, tag, obj) in objs {
                        let data = ByteBuf::from(obj.pack());
                        ciborium::into_writer(&Operation::Post, &mut buffer).unwrap();
                        ciborium::into_writer(&(title, tag, data), &mut buffer).unwrap();
                    }
                    buffer.set_position(0);
                    let mut stream = TcpOrUnixStream::connect(&self.addr)?;
                    io::copy(&mut buffer, &mut stream)?;
                }
            }
        }
        Ok(())
    }
    fn join(&mut self) {
        if let Some((handle, tx)) = self.handle.take() {
            tx.send(SenderControl::Shutdown).unwrap();
            handle.join().unwrap();
        }
    }
}
