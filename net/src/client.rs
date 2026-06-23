use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use super::protocol::*;

enum ClientCmd {
    Send(ClientMsg),
    Disconnect,
}

pub struct ClientHandle {
    cmd_tx: mpsc::Sender<ClientCmd>,
    pub event_rx: mpsc::Receiver<NetEvent>,
    _thread: JoinHandle<()>,
}

impl ClientHandle {
    pub fn connect(addr: std::net::SocketAddr, hello: Hello) -> std::io::Result<Self> {
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
        stream.set_nonblocking(true)?;

        let (evt_tx, event_rx) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel::<ClientCmd>();

        let reader_stream = stream.try_clone()?;
        let writer_stream = stream.try_clone()?;
        let _ = evt_tx.send(NetEvent::Connected);

        let hello_json = serde_json::to_string(&ClientMsg::Hello(hello)).unwrap_or_default();
        let mut w = writer_stream.try_clone().unwrap();
        let _ = w.write_all(format!("{hello_json}\n").as_bytes());

        let r_evt_tx = evt_tx.clone();
        let reader = std::thread::spawn(move || {
            client_reader(reader_stream, r_evt_tx);
        });

        let w_evt_tx = evt_tx;
        let writer = std::thread::spawn(move || {
            client_writer(writer_stream, cmd_rx, w_evt_tx);
        });

        let _thread = std::thread::spawn(move || {
            reader.join().ok();
            writer.join().ok();
        });

        Ok(Self { cmd_tx, event_rx, _thread })
    }

    pub fn send(&self, msg: ClientMsg) {
        let _ = self.cmd_tx.send(ClientCmd::Send(msg));
    }

    pub fn disconnect(self) {
        let _ = self.cmd_tx.send(ClientCmd::Disconnect);
    }
}

fn client_reader(stream: TcpStream, evt_tx: mpsc::Sender<NetEvent>) {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let msg: ServerMsg = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if evt_tx.send(NetEvent::ServerMsg(msg)).is_err() {
            break;
        }
    }
    let _ = evt_tx.send(NetEvent::Disconnected);
}

fn client_writer(
    mut stream: TcpStream,
    cmd_rx: mpsc::Receiver<ClientCmd>,
    evt_tx: mpsc::Sender<NetEvent>,
) {
    for cmd in cmd_rx {
        match cmd {
            ClientCmd::Send(msg) => {
                let line = serde_json::to_string(&msg).unwrap_or_default();
                let line = format!("{line}\n");
                if stream.write_all(line.as_bytes()).is_err() {
                    let _ = evt_tx.send(NetEvent::Error("write error".into()));
                    break;
                }
            }
            ClientCmd::Disconnect => break,
        }
    }
}
