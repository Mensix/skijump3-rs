use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use super::protocol::*;

enum HostCmd {
    Broadcast(ServerMsg),
    Stop,
}

pub struct HostHandle {
    cmd_tx: mpsc::Sender<HostCmd>,
    pub event_rx: mpsc::Receiver<NetEvent>,
    _threads: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl HostHandle {
    pub fn start() -> std::io::Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, PORT))?;
        listener.set_nonblocking(true)?;

        let (evt_tx, event_rx) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel::<HostCmd>();
        let stop = Arc::new(AtomicBool::new(false));

        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        let disc_tx = evt_tx.clone();
        let disc_stop = Arc::clone(&stop);
        threads.push(std::thread::spawn(move || {
            let _ = super::discovery::announce(PORT, &disc_stop);
            let _ = disc_tx.send(NetEvent::Error("discovery thread exited".into()));
        }));

        let accept_tx = evt_tx;
        let accept_stop = Arc::clone(&stop);
        threads.push(std::thread::spawn(move || {
            host_loop(listener, accept_tx, cmd_rx, &accept_stop);
        }));

        Ok(Self {
            cmd_tx,
            event_rx,
            _threads: threads,
            stop,
        })
    }

    pub fn broadcast(&self, msg: ServerMsg) {
        let _ = self.cmd_tx.send(HostCmd::Broadcast(msg));
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.cmd_tx.send(HostCmd::Stop);
    }
}

struct ClientWriters {
    _id: usize,
    writer: TcpStream,
    _reader: JoinHandle<()>,
}

fn host_loop(
    listener: TcpListener,
    evt_tx: mpsc::Sender<NetEvent>,
    cmd_rx: mpsc::Receiver<HostCmd>,
    stop: &AtomicBool,
) {
    let mut next_id: usize = 0;
    let mut clients: HashMap<usize, ClientWriters> = HashMap::new();

    loop {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        if let Ok((stream, _)) = listener.accept() {
            let _ = stream.set_nonblocking(false);
            let id = next_id;
            next_id += 1;
            let reader_stream = stream.try_clone().unwrap();
            let msg_tx = evt_tx.clone();
            let reader = std::thread::spawn(move || {
                client_reader(id, reader_stream, msg_tx);
            });
            clients.insert(
                id,
                ClientWriters {
                    _id: id,
                    writer: stream,
                    _reader: reader,
                },
            );
        }

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                HostCmd::Broadcast(msg) => {
                    let line = serialize_msg(&msg);
                    clients.retain(|id, cw| {
                        let mut w = cw.writer.try_clone().unwrap();
                        if w.write_all(line.as_bytes()).is_err() {
                            let _ = evt_tx.send(NetEvent::ClientLeft(*id));
                            false
                        } else {
                            true
                        }
                    });
                }
                HostCmd::Stop => return,
            }
        }

        let dead: Vec<usize> = clients
            .iter()
            .filter(|(_, cw)| cw._reader.is_finished())
            .map(|(id, _)| *id)
            .collect();
        for id in dead {
            let _ = evt_tx.send(NetEvent::ClientLeft(id));
            clients.remove(&id);
        }

        std::thread::sleep(Duration::from_millis(20));
    }
}

fn client_reader(id: usize, stream: TcpStream, evt_tx: mpsc::Sender<NetEvent>) {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let msg: ClientMsg = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let evt = match msg {
            ClientMsg::Hello(h) => NetEvent::ClientHello(id, h),
            ClientMsg::Ready(r) => NetEvent::ClientReady(id, r),
            ClientMsg::Chat(t) => NetEvent::ClientChat(id, t),
            ClientMsg::JumpComplete {
                distance,
                score,
                style_points,
                landing_style,
                fall_type,
            } => NetEvent::ClientJumpComplete {
                id,
                distance,
                score,
                style_points,
                landing_style,
                fall_type,
            },
            ClientMsg::Leave => NetEvent::ClientLeft(id),
        };
        if evt_tx.send(evt).is_err() {
            break;
        }
    }
}

fn serialize_msg(msg: &ServerMsg) -> String {
    let json = serde_json::to_string(msg).unwrap_or_default();
    format!("{json}\n")
}
