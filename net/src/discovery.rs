use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use super::protocol::{DISCOVERY_MAGIC, DISCOVERY_RESPONSE, DISCOVERY_PORT};

pub fn announce(host_port: u16) -> std::io::Result<UdpSocket> {
    let sock = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT)))?;
    sock.set_broadcast(true)?;
    sock.set_read_timeout(Some(Duration::from_millis(500)))?;
    let msg = format!("{DISCOVERY_RESPONSE}:{host_port}");
    let buf = msg.as_bytes();
    loop {
        let mut recv = [0u8; 128];
        if let Ok((_, src)) = sock.recv_from(&mut recv) {
            if recv.starts_with(DISCOVERY_MAGIC.as_bytes()) {
                sock.send_to(buf, src)?;
            }
        }
    }
}

pub fn discover(timeout: Duration) -> std::io::Result<Option<SocketAddr>> {
    let sock = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
    sock.set_broadcast(true)?;
    sock.set_read_timeout(Some(timeout))?;
    sock.send_to(
        DISCOVERY_MAGIC.as_bytes(),
        SocketAddr::from((Ipv4Addr::BROADCAST, DISCOVERY_PORT)),
    )?;
    let mut buf = [0u8; 128];
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        match sock.recv_from(&mut buf) {
            Ok((n, src)) => {
                let text = String::from_utf8_lossy(&buf[..n]);
                if let Some(port_str) = text.strip_prefix(DISCOVERY_RESPONSE).and_then(|s| s.strip_prefix(':')) {
                    if let Ok(port) = port_str.trim().parse::<u16>() {
                        let addr = SocketAddr::new(src.ip(), port);
                        return Ok(Some(addr));
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }
    Ok(None)
}
