use dlna_config::AppConfig as Config;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
const SSDP_PORT: u16 = 1900;

fn bind_reusable_udp(port: u16) -> std::io::Result<UdpSocket> {
    #[cfg(unix)]
    {
        use std::os::fd::FromRawFd;
        unsafe {
            let fd = libc::socket(libc::AF_INET, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0);
            if fd < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let optval: libc::c_int = 1;
            libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, &optval as *const _ as *const libc::c_void, std::mem::size_of::<libc::c_int>() as libc::socklen_t);
            #[cfg(target_os = "linux")]
            libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEPORT, &optval as *const _ as *const libc::c_void, std::mem::size_of::<libc::c_int>() as libc::socklen_t);
            let addr = libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: port.to_be(),
                sin_addr: libc::in_addr { s_addr: 0 },
                sin_zero: [0; 8],
            };
            if libc::bind(fd, &addr as *const _ as *const libc::sockaddr, std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
                let err = std::io::Error::last_os_error();
                libc::close(fd);
                return Err(err);
            }
            return Ok(UdpSocket::from_raw_fd(fd));
        }
    }
    #[cfg(not(unix))]
    {
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))
    }
}

pub fn spawn(config: Arc<Config>, running: Arc<AtomicBool>, ready: Arc<AtomicBool>, ip: Ipv4Addr) -> Result<thread::JoinHandle<()>, String> {
    let socket = bind_reusable_udp(SSDP_PORT).map_err(|e| format!("cannot bind SSDP UDP/1900: {e}"))?;
    socket.join_multicast_v4(&GROUP, &ip).map_err(|e| format!("cannot join SSDP multicast on {ip}: {e}"))?;
    socket.set_read_timeout(Some(Duration::from_secs(1))).ok();
    socket.set_multicast_ttl_v4(2).ok();
    Ok(thread::spawn(move || {
        let mut last_alive = Instant::now() - Duration::from_secs(120);
        let mut buffer = [0u8; 8192];
        while running.load(Ordering::Relaxed) {
            if ready.load(Ordering::Relaxed) && last_alive.elapsed() >= Duration::from_secs(50) {
                notify(&socket, &config, ip, true);
                last_alive = Instant::now();
            }
            match socket.recv_from(&mut buffer) {
                Ok((n, peer)) => {
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    if request.to_ascii_uppercase().starts_with("M-SEARCH") {
                        let st = header(&request, "ST").unwrap_or_else(|| "ssdp:all".into());
                        if ready.load(Ordering::Relaxed) && valid_search(&st) { respond_search(&socket, peer, &config, ip, &st); }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {},
                Err(_) => thread::sleep(Duration::from_millis(100)),
            }
        }
        if ready.load(Ordering::Relaxed) { notify(&socket, &config, ip, false); }
    }))
}

fn header(message: &str, wanted: &str) -> Option<String> {
    message.lines().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        if k.trim().eq_ignore_ascii_case(wanted) { Some(v.trim().to_string()) } else { None }
    })
}
fn valid_search(st: &str) -> bool {
    st.eq_ignore_ascii_case("ssdp:all") || st.eq_ignore_ascii_case("upnp:rootdevice") ||
    st.eq_ignore_ascii_case("urn:schemas-upnp-org:device:MediaServer:1") ||
    st.eq_ignore_ascii_case("urn:schemas-upnp-org:service:ContentDirectory:1") ||
    st.eq_ignore_ascii_case("urn:schemas-upnp-org:service:ConnectionManager:1") || st.starts_with("uuid:")
}
fn local_location(config: &Config, ip: Ipv4Addr) -> String { format!("http://{}:{}/device-v2.xml", ip, config.port) }
fn targets(config: &Config) -> Vec<(String, String)> {
    let uuid = format!("uuid:{}", config.uuid);
    vec![
        ("upnp:rootdevice".into(), format!("{}::upnp:rootdevice", uuid)),
        (uuid.clone(), uuid.clone()),
        ("urn:schemas-upnp-org:device:MediaServer:1".into(), format!("{}::urn:schemas-upnp-org:device:MediaServer:1", uuid)),
        ("urn:schemas-upnp-org:service:ContentDirectory:1".into(), format!("{}::urn:schemas-upnp-org:service:ContentDirectory:1", uuid)),
        ("urn:schemas-upnp-org:service:ConnectionManager:1".into(), format!("{}::urn:schemas-upnp-org:service:ConnectionManager:1", uuid)),
    ]
}
fn respond_search(socket: &UdpSocket, peer: SocketAddr, config: &Config, ip: Ipv4Addr, requested: &str) {
    let all = targets(config);
    for (st, usn) in all {
        if !requested.eq_ignore_ascii_case("ssdp:all") && !requested.eq_ignore_ascii_case(&st) { continue; }
        let response = format!("HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=1800\r\nEXT:\r\nLOCATION: {}\r\nSERVER: Linux UPnP/1.1 DLNA-SERVER-CORE/2.0\r\nST: {}\r\nUSN: {}\r\nBOOTID.UPNP.ORG: 1\r\nCONFIGID.UPNP.ORG: 3\r\n\r\n", local_location(config, ip), st, usn);
        let _ = socket.send_to(response.as_bytes(), peer);
    }
}
fn notify(socket: &UdpSocket, config: &Config, ip: Ipv4Addr, alive: bool) {
    let nts = if alive { "ssdp:alive" } else { "ssdp:byebye" };
    for (nt, usn) in targets(config) {
        let mut message = format!("NOTIFY * HTTP/1.1\r\nHOST: {}:{}\r\nNT: {}\r\nNTS: {}\r\nUSN: {}\r\n", GROUP, SSDP_PORT, nt, nts, usn);
        if alive {
            message.push_str(&format!("CACHE-CONTROL: max-age=1800\r\nLOCATION: {}\r\nSERVER: Linux UPnP/1.1 DLNA-SERVER-CORE/2.0\r\nBOOTID.UPNP.ORG: 1\r\nCONFIGID.UPNP.ORG: 3\r\n", local_location(config, ip)));
        }
        message.push_str("\r\n");
        let _ = socket.send_to(message.as_bytes(), (GROUP, SSDP_PORT));
    }
}
