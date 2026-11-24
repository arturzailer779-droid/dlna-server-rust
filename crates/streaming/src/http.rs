use dlna_config::AppConfig as Config;
use dlna_media_library::{mime_for, MediaIndex};
use dlna_protocol::xml;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

const MAX_HEADER: usize = 16 * 1024;
const MAX_BODY: usize = 1024 * 1024;
const MAX_CONNECTIONS: usize = 48;

#[derive(Clone)]
struct Subscription { callback_host: String, callback_port: u16, callback_path: String, timeout: u64, expires: std::time::Instant, seq: u32 }
type SubscriptionMap = Arc<Mutex<HashMap<String, Subscription>>>;
static NEXT_SID: AtomicU64 = AtomicU64::new(1);

struct Request { method: String, path: String, headers: Vec<(String, String)>, body: Vec<u8> }
impl Request {
    fn header(&self, key: &str) -> Option<&str> { self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str()) }
}

pub fn run(listener: TcpListener, config: Arc<Config>, index: Arc<RwLock<MediaIndex>>, running: Arc<AtomicBool>) {
    let active = Arc::new(AtomicUsize::new(0));
    let subscriptions: SubscriptionMap = Arc::new(Mutex::new(HashMap::new()));
    {
        let subs = Arc::clone(&subscriptions);
        let idx = Arc::clone(&index);
        let alive = Arc::clone(&running);
        thread::spawn(move || {
            let mut last_update = idx.read().map(|i| i.update_id).unwrap_or(0);
            while alive.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
                let current = idx.read().map(|i| i.update_id).unwrap_or(last_update);
                let mut deliveries = Vec::new();
                if let Ok(mut map) = subs.lock() {
                    map.retain(|_, sub| sub.expires > std::time::Instant::now());
                    if current != last_update {
                        for (sid, sub) in map.iter_mut() {
                            let seq = sub.seq;
                            sub.seq = sub.seq.wrapping_add(1);
                            deliveries.push((sid.clone(), sub.clone(), seq));
                        }
                    }
                }
                if current != last_update {
                    last_update = current;
                    let body = format!("<?xml version=\"1.0\"?><e:propertyset xmlns:e=\"urn:schemas-upnp-org:event-1-0\"><e:property><SystemUpdateID>{}</SystemUpdateID></e:property></e:propertyset>", current);
                    for (sid, sub, seq) in deliveries {
                        if send_notify(&sub, &sid, seq, &body).is_err() {
                            if let Ok(mut map) = subs.lock() { map.remove(&sid); }
                            eprintln!("[GENA] NOTIFY failed; removed SID={}", sid);
                        }
                    }
                }
            }
        });
    }
    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                if active.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
                    active.fetch_sub(1, Ordering::SeqCst);
                    let mut s = stream;
                    let _ = respond(&mut s, 503, "Service Unavailable", "text/plain", b"Busy", &[]);
                    continue;
                }
                let cfg = Arc::clone(&config);
                let idx = Arc::clone(&index);
                let count = Arc::clone(&active);
                let subs = Arc::clone(&subscriptions);
                thread::spawn(move || {
                    let _guard = ActiveGuard(count);
                    handle(stream, cfg, idx, subs);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(25)),
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }
}
struct ActiveGuard(Arc<AtomicUsize>);
impl Drop for ActiveGuard { fn drop(&mut self) { self.0.fetch_sub(1, Ordering::SeqCst); } }

fn parse_request(stream: &mut TcpStream) -> Result<Request, String> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let mut data = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    let header_end;
    loop {
        let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 { return Err("empty request".into()); }
        data.extend_from_slice(&chunk[..n]);
        if data.len() > MAX_HEADER + MAX_BODY { return Err("request too large".into()); }
        if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") { header_end = pos + 4; break; }
        if data.len() > MAX_HEADER { return Err("HTTP headers too large".into()); }
    }
    let head = String::from_utf8_lossy(&data[..header_end]);
    let mut lines = head.split("\r\n");
    let first = lines.next().ok_or("missing request line")?;
    let mut parts = first.split_whitespace();
    let method = parts.next().ok_or("missing method")?.to_string();
    let target = parts.next().ok_or("missing path")?;
    let path = target.split('?').next().unwrap_or("/").to_string();
    let mut headers = Vec::new();
    let mut content_length = 0usize;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            let value = v.trim().to_string();
            if key == "content-length" { content_length = value.parse().map_err(|_| "invalid Content-Length")?; }
            headers.push((key, value));
        }
    }
    if content_length > MAX_BODY { return Err("SOAP body too large".into()); }
    let already = data.len().saturating_sub(header_end);
    while data.len() - header_end < content_length {
        let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 { return Err("truncated request body".into()); }
        data.extend_from_slice(&chunk[..n]);
    }
    let body = data[header_end..header_end + content_length].to_vec();
    let _ = already;
    Ok(Request { method, path, headers, body })
}

fn handle(mut stream: TcpStream, config: Arc<Config>, index: Arc<RwLock<MediaIndex>>, subscriptions: SubscriptionMap) {
    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".into());
    let request = match parse_request(&mut stream) {
        Ok(r) => r,
        Err(error) => {
            eprintln!("[DLNA-DIAG] peer={} request_parse_error={:?}", peer, error);
            let _ = respond(&mut stream, 400, "Bad Request", "text/plain", b"Bad request", &[]);
            return;
        }
    };
    log_incoming_request(&peer, &request);
    let head_only = request.method == "HEAD";
    if request.method != "GET" && request.method != "HEAD" && request.method != "POST" && request.method != "SUBSCRIBE" && request.method != "UNSUBSCRIBE" {
        eprintln!("[DLNA-DIAG] peer={} rejected_method={} path={} status=405", peer, request.method, request.path);
        let _ = respond(&mut stream, 405, "Method Not Allowed", "text/plain", b"Method not allowed", &[("Allow", "GET, HEAD, POST, SUBSCRIBE, UNSUBSCRIBE")]); return;
    }
    let host = local_ip_for_peer(&stream).unwrap_or_else(|| "127.0.0.1".into());
    if request.path == "/ContentDirectory/event" && request.method == "SUBSCRIBE" {
        handle_subscribe(&mut stream, &request, &index, &subscriptions);
        return;
    }
    if request.path == "/ContentDirectory/event" && request.method == "UNSUBSCRIBE" {
        handle_unsubscribe(&mut stream, &request, &subscriptions);
        return;
    }
    match request.path.as_str() {
        "/device.xml" | "/device-v2.xml" if request.method == "GET" || head_only => {
            let body = xml::device_xml(&config.name, &config.uuid, &host, config.port);
            if head_only { let _ = respond_head(&mut stream, 200, "OK", "text/xml; charset=utf-8", body.len(), &[]); }
            else { let _ = respond(&mut stream, 200, "OK", "text/xml; charset=utf-8", body.as_bytes(), &[]); }
        }
        "/ContentDirectory.xml" => {
            let body = xml::content_directory_scpd().as_bytes();
            if head_only { let _ = respond_head(&mut stream, 200, "OK", "text/xml; charset=utf-8", body.len(), &[]); }
            else { let _ = respond(&mut stream, 200, "OK", "text/xml; charset=utf-8", body, &[]); }
        }
        "/ConnectionManager.xml" => {
            let body = xml::connection_manager_scpd().as_bytes();
            if head_only { let _ = respond_head(&mut stream, 200, "OK", "text/xml; charset=utf-8", body.len(), &[]); }
            else { let _ = respond(&mut stream, 200, "OK", "text/xml; charset=utf-8", body, &[]); }
        }
        "/ContentDirectory/control" if request.method == "POST" => {
            let action = request.header("SOAPAction").unwrap_or("");
            let action = action.rsplit('#').next().unwrap_or("").trim_matches('"');
            let body = String::from_utf8_lossy(&request.body);
            let response = content_directory_action(action, &body, &index, &host, config.port, &config.transcode_mode);
            eprintln!("[DLNA-DIAG] ContentDirectory response action={:?} status=200 response_bytes={}", action, response.len());
            let _ = respond(&mut stream, 200, "OK", "text/xml; charset=utf-8", response.as_bytes(), &[("EXT", "")]);
        }
        "/ConnectionManager/control" if request.method == "POST" => {
            let action = request.header("SOAPAction").unwrap_or("");
            let action = action.rsplit('#').next().unwrap_or("").trim_matches('"');
            let response = connection_manager_action(action);
            let _ = respond(&mut stream, 200, "OK", "text/xml; charset=utf-8", response.as_bytes(), &[("EXT", "")]);
        }
        "/icon.png" | "/icon-48.png" | "/icon-128.png" | "/icon-256.png" | "/icon-v2-48.png" | "/icon-v2-128.png" | "/icon-v2-256.png" | "/brand-v3-48.png" | "/brand-v3-128.png" | "/brand-v3-256.png" => {
            let size = match request.path.as_str() { "/icon-48.png" | "/icon-v2-48.png" | "/brand-v3-48.png" => "48", "/icon-128.png" | "/icon-v2-128.png" | "/brand-v3-128.png" => "128", "/icon-256.png" | "/icon-v2-256.png" | "/brand-v3-256.png" => "256", _ => "256" };
            // Serve the exact size advertised in device.xml. The TV receives
            // the supplied brand artwork, never a generic desktop folder icon.
            let tailored = config.icon.parent().unwrap_or(Path::new("/")).join(format!("brand_{}.png", size));
            let icon_path = if tailored.is_file() { tailored } else { config.icon.clone() };
            match fs::read(icon_path) {
                Ok(bytes) => {
                    if head_only { let _ = respond_head(&mut stream, 200, "OK", "image/png", bytes.len(), &[("Cache-Control", "public, max-age=86400")]); }
                    else { let _ = respond(&mut stream, 200, "OK", "image/png", &bytes, &[("Cache-Control", "public, max-age=86400")]); }
                },
                Err(_) => { let _ = respond(&mut stream, 404, "Not Found", "text/plain", b"Icon unavailable", &[]); }
            }
        }
        _ if request.path.starts_with("/thumb/") && (request.method == "GET" || head_only) => {
            let raw_id = request.path.trim_start_matches("/thumb/").trim_end_matches(".jpg");
            serve_thumbnail(&mut stream, raw_id, head_only, &config, &index);
        }
        _ if request.path.starts_with("/media/") && (request.method == "GET" || head_only) => {
            let id = request.path.trim_start_matches("/media/");
            serve_media(&mut stream, id, request.header("Range"), head_only, &index, &config.transcode_mode);
        }
        "/" => { let _ = respond(&mut stream, 200, "OK", "text/plain; charset=utf-8", b"DLNA SERVER CORE is running", &[]); }
        _ => {
            eprintln!("[DLNA-DIAG] peer={} unmatched_request method={} path={} status=404", peer, request.method, request.path);
            let _ = respond(&mut stream, 404, "Not Found", "text/plain", b"Not found", &[]);
        }
    }
}

fn handle_subscribe(stream: &mut TcpStream, request: &Request, index: &Arc<RwLock<MediaIndex>>, subscriptions: &SubscriptionMap) {
    let timeout_header = request.header("timeout").unwrap_or("Second-1800");
    let timeout = timeout_header.split(',').next().unwrap_or("Second-1800").trim()
        .strip_prefix("Second-").and_then(|v| v.parse::<u64>().ok()).unwrap_or(1800).clamp(30, 1800);
    if let Some(sid) = request.header("sid") {
        let mut map = match subscriptions.lock() { Ok(m) => m, Err(_) => { let _ = respond(stream, 500, "Internal Server Error", "text/plain", b"Internal error", &[]); return; } };
        if let Some(sub) = map.get_mut(sid) {
            sub.expires = std::time::Instant::now() + Duration::from_secs(timeout);
            sub.timeout = timeout;
            let timeout_value = format!("Second-{}", timeout);
            let _ = respond(stream, 200, "OK", "text/plain", b"", &[("SID", sid), ("TIMEOUT", timeout_value.as_str())]);
        } else { let _ = respond(stream, 412, "Precondition Failed", "text/plain", b"Unknown SID", &[]); }
        return;
    }
    let callback = match request.header("callback") {
        Some(v) if request.header("nt").map(|x| x.eq_ignore_ascii_case("upnp:event")).unwrap_or(false) => v.trim().trim_start_matches('<').trim_end_matches('>'),
        _ => { let _ = respond(stream, 400, "Bad Request", "text/plain", b"CALLBACK and NT: upnp:event required", &[]); return; }
    };
    let (cb_host, cb_port, cb_path) = match parse_callback(callback) {
        Some(v) => v,
        None => { let _ = respond(stream, 412, "Precondition Failed", "text/plain", b"Unsupported CALLBACK URL", &[]); return; }
    };
    let n = NEXT_SID.fetch_add(1, Ordering::Relaxed);
    let sid = format!("uuid:dlna-core-{}-{}", std::process::id(), n);
    let sub = Subscription { callback_host: cb_host, callback_port: cb_port, callback_path: cb_path, timeout, expires: std::time::Instant::now() + Duration::from_secs(timeout), seq: 1 };
    if let Ok(mut map) = subscriptions.lock() { map.insert(sid.clone(), sub.clone()); }
    let timeout_value = format!("Second-{}", timeout);
    let _ = respond(stream, 200, "OK", "text/plain", b"", &[("SID", &sid), ("TIMEOUT", &timeout_value)]);
    let update_id = index.read().map(|i| i.update_id).unwrap_or(0);
    let body = format!("<?xml version=\"1.0\"?><e:propertyset xmlns:e=\"urn:schemas-upnp-org:event-1-0\"><e:property><SystemUpdateID>{}</SystemUpdateID></e:property></e:propertyset>", update_id);
    if send_notify(&sub, &sid, 0, &body).is_err() {
        if let Ok(mut map) = subscriptions.lock() { map.remove(&sid); }
        eprintln!("[GENA] initial NOTIFY failed for {}", sid);
    } else { eprintln!("[GENA] SUBSCRIBE accepted SID={} timeout={}", sid, timeout); }
}

fn handle_unsubscribe(stream: &mut TcpStream, request: &Request, subscriptions: &SubscriptionMap) {
    let sid = match request.header("sid") { Some(v) => v, None => { let _ = respond(stream, 400, "Bad Request", "text/plain", b"SID required", &[]); return; } };
    let removed = subscriptions.lock().map(|mut m| m.remove(sid).is_some()).unwrap_or(false);
    if removed { let _ = respond(stream, 200, "OK", "text/plain", b"", &[]); eprintln!("[GENA] UNSUBSCRIBE SID={}", sid); }
    else { let _ = respond(stream, 412, "Precondition Failed", "text/plain", b"Unknown SID", &[]); }
}

fn parse_callback(url: &str) -> Option<(String, u16, String)> {
    let rest = url.strip_prefix("http://")?;
    let (authority, path) = match rest.split_once('/') { Some((a, p)) => (a, format!("/{}", p)), None => (rest, "/".to_string()) };
    let (host, port) = authority.rsplit_once(':')?;
    if host.is_empty() || host.contains('@') { return None; }
    Some((host.trim_matches(['[', ']']).to_string(), port.parse().ok()?, path))
}

fn send_notify(sub: &Subscription, sid: &str, seq: u32, body: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect((sub.callback_host.as_str(), sub.callback_port))?;
    stream.set_write_timeout(Some(Duration::from_secs(3))).ok();
    write!(stream, "NOTIFY {} HTTP/1.1\r\nHOST: {}:{}\r\nCONTENT-TYPE: text/xml; charset=\"utf-8\"\r\nCONTENT-LENGTH: {}\r\nNT: upnp:event\r\nNTS: upnp:propertychange\r\nSID: {}\r\nSEQ: {}\r\nCONNECTION: close\r\n\r\n{}", sub.callback_path, sub.callback_host, sub.callback_port, body.len(), sid, seq, body)?;
    stream.flush()?;
    Ok(())
}

fn log_incoming_request(peer: &str, request: &Request) {
    // Diagnostic-only logging: no request handling or response behavior is changed.
    let fields = ["user-agent", "soapaction", "content-type", "content-length", "nt", "callback", "timeout", "sid", "seq", "man", "mx", "st"];
    let mut details = String::new();
    for key in fields {
        if let Some(value) = request.header(key) {
            if !details.is_empty() { details.push(' '); }
            details.push_str(key);
            details.push('=');
            details.push_str(&format!("{:?}", value));
        }
    }
    eprintln!("[DLNA-DIAG] peer={} request={} {}{}", peer, request.method, request.path,
        if details.is_empty() { String::new() } else { format!(" {}", details) });
}

fn local_ip_for_peer(stream: &TcpStream) -> Option<String> {
    let peer = stream.peer_addr().ok()?;
    let route = UdpSocket::bind("0.0.0.0:0").ok()?;
    route.connect(peer).ok()?;
    Some(route.local_addr().ok()?.ip().to_string())
}

fn respond_head(stream: &mut TcpStream, status: u16, reason: &str, content_type: &str, content_length: usize, extra: &[(&str, &str)]) -> std::io::Result<()> {
    write!(stream, "HTTP/1.1 {} {}\r\nServer: DLNA-SERVER-CORE/2.0 UPnP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n", status, reason, content_type, content_length)?;
    for (k, v) in extra { if !v.is_empty() { write!(stream, "{}: {}\r\n", k, v)?; } else { write!(stream, "{}:\r\n", k)?; } }
    write!(stream, "\r\n")?;
    stream.flush()
}

fn respond(stream: &mut TcpStream, status: u16, reason: &str, content_type: &str, body: &[u8], extra: &[(&str, &str)]) -> std::io::Result<()> {
    respond_head(stream, status, reason, content_type, body.len(), extra)?;
    stream.write_all(body)?;
    stream.flush()
}

fn content_directory_action(action: &str, body: &str, index: &Arc<RwLock<MediaIndex>>, host: &str, port: u16, transcode_mode: &str) -> String {
    let idx = match index.read() { Ok(i) => i, Err(_) => return soap_fault(501, "Internal error") };
    let service = "urn:schemas-upnp-org:service:ContentDirectory:1";
    match action {
        "GetSystemUpdateID" => soap(service, "GetSystemUpdateID", &format!("<Id>{}</Id>", idx.update_id)),
        "GetSearchCapabilities" => soap(service, "GetSearchCapabilities", "<SearchCaps></SearchCaps>"),
        "GetSortCapabilities" => soap(service, "GetSortCapabilities", "<SortCaps>dc:title</SortCaps>"),
        "Browse" => {
            let object_id = xml::tag_value(body, "ObjectID").unwrap_or_else(|| "0".into());
            let flag = xml::tag_value(body, "BrowseFlag").unwrap_or_else(|| "BrowseDirectChildren".into());
            let start: usize = xml::tag_value(body, "StartingIndex").and_then(|s| s.parse().ok()).unwrap_or(0);
            let requested: usize = xml::tag_value(body, "RequestedCount").and_then(|s| s.parse().ok()).unwrap_or(0);
            let filter = xml::tag_value(body, "Filter").unwrap_or_else(|| "<missing>".into());
            let sort = xml::tag_value(body, "SortCriteria").unwrap_or_else(|| "<missing>".into());
            eprintln!("[DLNA-DIAG] Browse object_id={:?} flag={:?} filter={:?} sort={:?} starting_index={} requested_count={}",
                object_id, flag, filter, sort, start, requested);
            let mut selected: Vec<&dlna_media_library::Entry> = Vec::new();
            if flag == "BrowseMetadata" {
                if object_id == "0" { selected = Vec::new(); }
                else if let Some(entry) = idx.get(&object_id) { selected.push(entry); }
            } else if object_id == "0" || idx.get(&object_id).map(|e| e.is_dir).unwrap_or(false) {
                selected = idx.children(&object_id);
            }
            let root_entry = dlna_media_library::Entry { id: "0".into(), parent: "-1".into(), title: "Video".into(), path: std::path::PathBuf::new(), is_dir: true, size: 0 };
            if flag == "BrowseMetadata" && object_id == "0" { selected.push(&root_entry); }
            let total = selected.len();
            let end = if requested == 0 { total } else { start.saturating_add(requested).min(total) };
            let page: Vec<_> = if start < total { selected[start..end].to_vec() } else { Vec::new() };
            let didl = xml::didl_document(&page, &idx, host, port, transcode_mode);
            eprintln!("[DLNA-DIAG] Browse result total_matches={} number_returned={} didl_bytes={} update_id={} item_ids={:?}",
                total, page.len(), didl.len(), idx.update_id, page.iter().map(|entry| entry.id.as_str()).collect::<Vec<_>>());
            let output = format!("<Result>{}</Result><NumberReturned>{}</NumberReturned><TotalMatches>{}</TotalMatches><UpdateID>{}</UpdateID>", xml::escape(&didl), page.len(), total, idx.update_id);
            soap(service, "Browse", &output)
        }
        _ => soap_fault(401, "Invalid Action"),
    }
}

fn connection_manager_action(action: &str) -> String {
    let service = "urn:schemas-upnp-org:service:ConnectionManager:1";
    match action {
        "GetProtocolInfo" => soap(service, "GetProtocolInfo", "<Source>http-get:*:video/mp4:*,http-get:*:video/x-matroska:*,http-get:*:video/x-msvideo:*,http-get:*:video/quicktime:*,http-get:*:video/mpeg:*,http-get:*:video/mp2t:*,http-get:*:video/x-ms-wmv:*,http-get:*:video/webm:*,http-get:*:video/dvd:*,http-get:*:video/3gpp:*,http-get:*:video/x-flv:*,http-get:*:video/ogg:*</Source><Sink></Sink>"),
        "GetCurrentConnectionIDs" => soap(service, "GetCurrentConnectionIDs", "<ConnectionIDs></ConnectionIDs>"),
        "GetCurrentConnectionInfo" => soap(service, "GetCurrentConnectionInfo", "<RcsID>-1</RcsID><AVTransportID>-1</AVTransportID><ProtocolInfo></ProtocolInfo><PeerConnectionManager></PeerConnectionManager><PeerConnectionID>-1</PeerConnectionID><Direction>Output</Direction><Status>OK</Status>"),
        _ => soap_fault(401, "Invalid Action"),
    }
}

fn soap(service: &str, action: &str, body: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="utf-8"?><s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/"><s:Body><u:{action}Response xmlns:u="{service}">{body}</u:{action}Response></s:Body></s:Envelope>"#)
}
fn soap_fault(code: u16, description: &str) -> String {
    format!(r#"<?xml version="1.0"?><s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"><s:Body><s:Fault><faultcode>s:Client</faultcode><faultstring>UPnPError</faultstring><detail><UPnPError xmlns="urn:schemas-upnp-org:control-1-0"><errorCode>{}</errorCode><errorDescription>{}</errorDescription></UPnPError></detail></s:Fault></s:Body></s:Envelope>"#, code, xml::escape(description))
}

fn serve_media(stream: &mut TcpStream, id: &str, range: Option<&str>, head: bool, index: &Arc<RwLock<MediaIndex>>, mode: &str) {
    if mode != "off"
        && let Ok(guard) = index.read()
            && let Some(entry) = guard.get(id) {
                let should = match mode { "on" | "1080p" | "720p" => !xml::is_4k_or_higher(&entry.path), "auto" => xml::needs_transcode(&entry.path), _ => false };
                if !entry.is_dir && should {
                    let path = entry.path.clone();
                    drop(guard);
                    serve_transcoded(stream, &path, head, mode);
                    return;
                }
            }
    let entry = match index.read().ok().and_then(|idx| idx.get(id).cloned()) { Some(e) if !e.is_dir => e, _ => { let _ = respond(stream, 404, "Not Found", "text/plain", b"Media not found", &[]); return; } };
    let canonical = match std::fs::canonicalize(&entry.path) { Ok(p) => p, Err(_) => { let _ = respond(stream, 404, "Not Found", "text/plain", b"Media unavailable", &[]); return; } };
    if canonical != entry.path || !canonical.is_file() { let _ = respond(stream, 403, "Forbidden", "text/plain", b"Forbidden", &[]); return; }
    let mut file = match File::open(&canonical) { Ok(f) => f, Err(_) => { let _ = respond(stream, 404, "Not Found", "text/plain", b"Media unavailable", &[]); return; } };
    let size = match file.metadata() { Ok(m) => m.len(), Err(_) => 0 };
    let mut start = 0u64;
    let mut end = size.saturating_sub(1);
    let mut partial = false;
    if let Some(value) = range
        && let Some(spec) = value.strip_prefix("bytes=")
            && let Some((a, b)) = spec.split_once('-') {
                if a.is_empty() {
                    if let Ok(suffix) = b.parse::<u64>() { start = size.saturating_sub(suffix); partial = true; }
                } else if let Ok(s) = a.parse::<u64>() {
                    start = s;
                    if !b.is_empty() && let Ok(e) = b.parse::<u64>() { end = e.min(size.saturating_sub(1)); }
                    partial = true;
                }
            }
    if size == 0 { start = 0; end = 0; }
    if start >= size && size > 0 || end < start {
        let _ = respond(stream, 416, "Range Not Satisfiable", "text/plain", b"Invalid range", &[("Content-Range", &format!("bytes */{}", size))]); return;
    }
    let length = if size == 0 { 0 } else { end - start + 1 };
    if file.seek(SeekFrom::Start(start)).is_err() { let _ = respond(stream, 500, "Internal Server Error", "text/plain", b"Seek failed", &[]); return; }
    let status = if partial { 206 } else { 200 };
    let reason = if partial { "Partial Content" } else { "OK" };
    let mime = mime_for(&canonical);
    let _ = write!(stream, "HTTP/1.1 {} {}\r\nServer: DLNA-SERVER-CORE/2.0 UPnP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\ntransferMode.dlna.org: Streaming\r\ncontentFeatures.dlna.org: DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000\r\nConnection: close\r\n", status, reason, mime, length);
    if partial { let _ = write!(stream, "Content-Range: bytes {}-{}/{}\r\n", start, end, size); }
    let _ = stream.write_all(b"\r\n");
    if head { return; }
    let mut remaining = length;
    let mut buffer = [0u8; 64 * 1024];
    while remaining > 0 {
        let wanted = buffer.len().min(remaining as usize);
        let n = match file.read(&mut buffer[..wanted]) { Ok(0) | Err(_) => break, Ok(n) => n };
        if stream.write_all(&buffer[..n]).is_err() { break; }
        remaining -= n as u64;
    }
    let _ = stream.flush();
}

fn serve_thumbnail(stream: &mut TcpStream, id: &str, head: bool, config: &Config, index: &Arc<RwLock<MediaIndex>>) {
    let video_path = {
        let Ok(guard) = index.read() else {
            let _ = respond(stream, 503, "Service Unavailable", "text/plain", b"Index busy", &[]);
            return;
        };
        match guard.get(id) {
            Some(entry) if !entry.is_dir => entry.path.clone(),
            _ => {
                let _ = respond(stream, 404, "Not Found", "text/plain", b"Thumbnail not found", &[]);
                return;
            }
        }
    };

    let cache_dir = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(".cache/dlna-video-server-2/thumbs");
    let _ = fs::create_dir_all(&cache_dir);
    let thumb_file = cache_dir.join(format!("{}.jpg", id));

    if !thumb_file.is_file() {
        let tmp_file = cache_dir.join(format!("{}.tmp-{}.jpg", id, std::process::id()));
        for seek in ["00:00:05", "00:00:00"] {
            let ok = std::process::Command::new("ffmpeg")
                .args(["-y", "-loglevel", "error", "-ss", seek, "-i"])
                .arg(&video_path)
                .args(["-vframes", "1", "-vf", "scale=320:-1", "-q:v", "4"])
                .arg(&tmp_file)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok && tmp_file.is_file() {
                let _ = fs::rename(&tmp_file, &thumb_file);
                break;
            }
            let _ = fs::remove_file(&tmp_file);
        }
    }

    if let Ok(bytes) = fs::read(&thumb_file) {
        if head {
            let _ = respond_head(stream, 200, "OK", "image/jpeg", bytes.len(), &[("Cache-Control", "public, max-age=86400")]);
        } else {
            let _ = respond(stream, 200, "OK", "image/jpeg", &bytes, &[("Cache-Control", "public, max-age=86400")]);
        }
        return;
    }

    if let Ok(bytes) = fs::read(&config.icon) {
        if head {
            let _ = respond_head(stream, 200, "OK", "image/png", bytes.len(), &[("Cache-Control", "public, max-age=3600")]);
        } else {
            let _ = respond(stream, 200, "OK", "image/png", &bytes, &[("Cache-Control", "public, max-age=3600")]);
        }
    } else {
        let _ = respond(stream, 404, "Not Found", "text/plain", b"No thumbnail", &[]);
    }
}

fn detect_hw_encoder() -> (&'static str, Vec<&'static str>) {
    use std::sync::OnceLock;
    static ENC: OnceLock<u8> = OnceLock::new();
    let kind = *ENC.get_or_init(|| {
        let nv = std::process::Command::new("ffmpeg")
            .args(["-hide_banner", "-f", "lavfi", "-i", "nullsrc=s=256x256:d=0.1", "-c:v", "h264_nvenc", "-f", "null", "-"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if nv { return 1; }
        let va = std::process::Command::new("ffmpeg")
            .args(["-hide_banner", "-vaapi_device", "/dev/dri/renderD128", "-f", "lavfi", "-i", "nullsrc=s=256x256:d=0.1", "-vf", "format=nv12,hwupload", "-c:v", "h264_vaapi", "-f", "null", "-"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if va { return 2; }
        0
    });
    match kind {
        1 => ("nvenc", vec![]),
        2 => ("vaapi", vec!["-vaapi_device", "/dev/dri/renderD128"]),
        _ => ("cpu", vec![]),
    }
}

fn spawn_ffmpeg_transcode(path: &Path, kind: &str, _max_w: &str, _maxrate: &str, _bufsize: &str) -> Option<std::process::Child> {
    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(["-loglevel", "error", "-thread_queue_size", "1024"]);
    match kind {
        "nvenc" => {
            cmd.args(["-hwaccel", "cuda"]);
            cmd.arg("-i").arg(path);
            cmd.args(["-vf", "format=yuv420p", "-c:v", "h264_nvenc", "-profile:v", "high", "-level", "4.1", "-preset", "p5", "-rc", "constqp", "-qp", "22", "-bf", "2", "-spatial_aq", "1", "-g", "60"]);
        }
        "vaapi" => {
            cmd.args(["-hwaccel", "vaapi", "-hwaccel_device", "/dev/dri/renderD128", "-hwaccel_output_format", "vaapi"]);
            cmd.arg("-i").arg(path);
            cmd.args(["-vf", "scale_vaapi=format=nv12", "-c:v", "h264_vaapi", "-profile:v", "high", "-level", "4.1", "-qp", "22", "-bf", "2", "-g", "60"]);
        }
        _ => {
            cmd.arg("-i").arg(path);
            cmd.args(["-threads", "2", "-vf", "format=yuv420p", "-c:v", "libx264", "-profile:v", "high", "-level", "4.1", "-preset", "superfast", "-crf", "23", "-maxrate", "6M", "-bufsize", "12M", "-g", "60"]);
        }
    }
    cmd.args(["-max_muxing_queue_size", "1024", "-c:a", "aac", "-ac", "2", "-b:a", "192k", "-mpegts_flags", "+resend_headers", "-f", "mpegts", "pipe:1"]);
    cmd.stdout(std::process::Stdio::piped()).spawn().ok()
}

fn serve_transcoded(stream: &mut TcpStream, path: &Path, head: bool, mode: &str) {
    let header = "HTTP/1.1 200 OK\r\nContent-Type: video/mp2t\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n";
    if stream.write_all(header.as_bytes()).is_err() || head {
        return;
    }
    let (max_w, maxrate, bufsize) = if mode == "720p" { ("1280", "4M", "8M") } else { ("1920", "8M", "16M") };
    let (kind, _) = detect_hw_encoder();

    let mut child_opt = spawn_ffmpeg_transcode(path, kind, max_w, maxrate, bufsize);
    let mut first_chunk = [0u8; 65536];
    let mut first_n = 0;

    if let Some(ref mut child) = child_opt
        && let Some(ref mut out) = child.stdout
    {
        first_n = out.read(&mut first_chunk).unwrap_or(0);
    }

    // Если аппаратный ускоритель не смог декодировать файл (например, редкий 10-bit 4K), автоматически откатываемся на универсальный CPU-профиль
    if first_n == 0 && kind != "cpu" {
        if let Some(mut old_child) = child_opt.take() {
            let _ = old_child.kill();
            let _ = old_child.wait();
        }
        child_opt = spawn_ffmpeg_transcode(path, "cpu", max_w, maxrate, bufsize);
        if let Some(ref mut child) = child_opt
            && let Some(ref mut out) = child.stdout
        {
            first_n = out.read(&mut first_chunk).unwrap_or(0);
        }
    }

    let Some(mut child) = child_opt else { return };
    if first_n > 0 {
        let chunk_hdr = format!("{:x}\r\n", first_n);
        if stream.write_all(chunk_hdr.as_bytes()).is_err()
            || stream.write_all(&first_chunk[..first_n]).is_err()
            || stream.write_all(b"\r\n").is_err()
        {
            let _ = child.kill();
            let _ = child.wait();
            return;
        }
        if let Some(mut out) = child.stdout.take() {
            let mut buf = [0u8; 65536];
            while let Ok(n) = out.read(&mut buf) {
                if n == 0 { break; }
                let chunk_hdr = format!("{:x}\r\n", n);
                if stream.write_all(chunk_hdr.as_bytes()).is_err()
                    || stream.write_all(&buf[..n]).is_err()
                    || stream.write_all(b"\r\n").is_err() {
                    break;
                }
            }
        }
        let _ = stream.write_all(b"0\r\n\r\n");
    }
    let _ = child.kill();
    let _ = child.wait();
}
