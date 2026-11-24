use std::net::{Ipv4Addr, UdpSocket};

pub fn default_route_ip() -> Result<Ipv4Addr, String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Ошибка создания UDP-сокета: {e}"))?;
    socket
        .connect("192.0.2.1:9")
        .map_err(|e| format!("Нет активного сетевого маршрута: {e}"))?;
    match socket.local_addr().map_err(|e| e.to_string())?.ip() {
        std::net::IpAddr::V4(ip) if !ip.is_loopback() => Ok(ip),
        _ => Err("Не найден активный локальный IPv4-адрес".into()),
    }
}
