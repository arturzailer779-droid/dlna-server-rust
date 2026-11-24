use dlna_media_library::{Entry, MediaIndex, mime_for};

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

pub fn unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

pub fn tag_value(xml: &str, name: &str) -> Option<String> {
    let open_end = xml.find(&format!("<{}", name))?;
    let start = xml[open_end..].find('>')? + open_end + 1;
    let close = xml[start..].find(&format!("</{}>", name))? + start;
    Some(unescape(xml[start..close].trim()))
}

pub fn device_xml(name: &str, uuid: &str, ip: &str, port: u16) -> String {
    format!(r#"<?xml version="1.0" encoding="utf-8"?>
<root xmlns="urn:schemas-upnp-org:device-1-0" configId="3"><specVersion><major>1</major><minor>0</minor></specVersion>
<URLBase>http://{ip}:{port}/</URLBase><device><deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>
<friendlyName>{}</friendlyName><manufacturer>DLNA SERVER Project</manufacturer>
<modelDescription>Lightweight local video server</modelDescription><modelName>DLNA SERVER CORE</modelName><modelNumber>2.0</modelNumber><serialNumber>DLNA-CORE-2</serialNumber>
<UDN>uuid:{}</UDN><presentationURL>http://{ip}:{port}/</presentationURL><iconList>
<!-- Put the full-detail artwork first: several TV firmwares use only the first icon. -->
<icon><mimetype>image/png</mimetype><width>256</width><height>256</height><depth>32</depth><url>/brand-v3-256.png</url></icon>
<icon><mimetype>image/png</mimetype><width>128</width><height>128</height><depth>32</depth><url>/brand-v3-128.png</url></icon>
<icon><mimetype>image/png</mimetype><width>48</width><height>48</height><depth>32</depth><url>/brand-v3-48.png</url></icon>
</iconList><serviceList>
<service><serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType><serviceId>urn:upnp-org:serviceId:ContentDirectory</serviceId><SCPDURL>/ContentDirectory.xml</SCPDURL><controlURL>/ContentDirectory/control</controlURL><eventSubURL>/ContentDirectory/event</eventSubURL></service>
<service><serviceType>urn:schemas-upnp-org:service:ConnectionManager:1</serviceType><serviceId>urn:upnp-org:serviceId:ConnectionManager</serviceId><SCPDURL>/ConnectionManager.xml</SCPDURL><controlURL>/ConnectionManager/control</controlURL><eventSubURL>/ConnectionManager/event</eventSubURL></service>
</serviceList></device></root>"#, escape(name), uuid, ip = ip, port = port)
}

pub fn content_directory_scpd() -> &'static str {
    r#"<?xml version="1.0"?><scpd xmlns="urn:schemas-upnp-org:service-1-0"><specVersion><major>1</major><minor>0</minor></specVersion><actionList>
<action><name>Browse</name><argumentList><argument><name>ObjectID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable></argument><argument><name>BrowseFlag</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_BrowseFlag</relatedStateVariable></argument><argument><name>Filter</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Filter</relatedStateVariable></argument><argument><name>StartingIndex</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Index</relatedStateVariable></argument><argument><name>RequestedCount</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable></argument><argument><name>SortCriteria</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_SortCriteria</relatedStateVariable></argument><argument><name>Result</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_Result</relatedStateVariable></argument><argument><name>NumberReturned</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable></argument><argument><name>TotalMatches</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable></argument><argument><name>UpdateID</name><direction>out</direction><relatedStateVariable>SystemUpdateID</relatedStateVariable></argument></argumentList></action>
<action><name>GetSystemUpdateID</name><argumentList><argument><name>Id</name><direction>out</direction><relatedStateVariable>SystemUpdateID</relatedStateVariable></argument></argumentList></action><action><name>GetSearchCapabilities</name><argumentList><argument><name>SearchCaps</name><direction>out</direction><relatedStateVariable>SearchCapabilities</relatedStateVariable></argument></argumentList></action><action><name>GetSortCapabilities</name><argumentList><argument><name>SortCaps</name><direction>out</direction><relatedStateVariable>SortCapabilities</relatedStateVariable></argument></argumentList></action></actionList><serviceStateTable><stateVariable sendEvents="yes"><name>SystemUpdateID</name><dataType>ui4</dataType><defaultValue>1</defaultValue></stateVariable><stateVariable sendEvents="no"><name>SearchCapabilities</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>SortCapabilities</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_ObjectID</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_BrowseFlag</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_Filter</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_Index</name><dataType>ui4</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_Count</name><dataType>ui4</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_SortCriteria</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_Result</name><dataType>string</dataType></stateVariable></serviceStateTable></scpd>"#
}

pub fn connection_manager_scpd() -> &'static str {
    r#"<?xml version="1.0"?><scpd xmlns="urn:schemas-upnp-org:service-1-0"><specVersion><major>1</major><minor>0</minor></specVersion><actionList><action><name>GetProtocolInfo</name><argumentList><argument><name>Source</name><direction>out</direction><relatedStateVariable>SourceProtocolInfo</relatedStateVariable></argument><argument><name>Sink</name><direction>out</direction><relatedStateVariable>SinkProtocolInfo</relatedStateVariable></argument></argumentList></action><action><name>GetCurrentConnectionIDs</name><argumentList><argument><name>ConnectionIDs</name><direction>out</direction><relatedStateVariable>CurrentConnectionIDs</relatedStateVariable></argument></argumentList></action><action><name>GetCurrentConnectionInfo</name><argumentList><argument><name>ConnectionID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_ConnectionID</relatedStateVariable></argument><argument><name>RcsID</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_RcsID</relatedStateVariable></argument><argument><name>AVTransportID</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_AVTransportID</relatedStateVariable></argument><argument><name>ProtocolInfo</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_ProtocolInfo</relatedStateVariable></argument><argument><name>PeerConnectionManager</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_ConnectionManager</relatedStateVariable></argument><argument><name>PeerConnectionID</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_ConnectionID</relatedStateVariable></argument><argument><name>Direction</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_Direction</relatedStateVariable></argument><argument><name>Status</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_ConnectionStatus</relatedStateVariable></argument></argumentList></action></actionList><serviceStateTable><stateVariable sendEvents="yes"><name>SourceProtocolInfo</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="yes"><name>SinkProtocolInfo</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>CurrentConnectionIDs</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_ConnectionID</name><dataType>i4</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_RcsID</name><dataType>i4</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_AVTransportID</name><dataType>i4</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_ProtocolInfo</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_ConnectionManager</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_Direction</name><dataType>string</dataType></stateVariable><stateVariable sendEvents="no"><name>A_ARG_TYPE_ConnectionStatus</name><dataType>string</dataType></stateVariable></serviceStateTable></scpd>"#
}

pub fn is_4k_or_higher(path: &std::path::Path) -> bool {
    std::process::Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width", "-of", "csv=p=0"])
        .arg(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
        .map(|w| w > 1920)
        .unwrap_or(false)
}

pub fn needs_transcode(path: &std::path::Path) -> bool {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
    if matches!(ext.as_str(), "avi" | "wmv" | "flv" | "webm" | "vob" | "3gp" | "ogv" | "mov" | "mts" | "m2ts") {
        return true;
    }

    static CACHE: OnceLock<Mutex<HashMap<PathBuf, bool>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock()
        && let Some(&cached) = guard.get(path)
    {
        return cached;
    }

    let result = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=codec_name,width", "-of", "csv=p=0"])
        .arg(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            let line = out.trim();
            let mut parts = line.split(',');
            let codec = parts.next().unwrap_or("").trim().to_ascii_lowercase();
            let width: u32 = parts.next().and_then(|w| w.trim().parse().ok()).unwrap_or(0);
            if width > 1920 {
                return false;
            }
            !codec.is_empty() && codec != "h264"
        })
        .unwrap_or(false);

    if let Ok(mut guard) = cache.lock() {
        guard.insert(path.to_path_buf(), result);
    }
    result
}

pub fn didl_entry(entry: &Entry, index: &MediaIndex, host: &str, port: u16, transcode_mode: &str) -> String {
    let title = escape(&entry.title);
    let id = escape(&entry.id);
    let parent = escape(&entry.parent);
    if entry.is_dir {
        format!(r#"<container id="{id}" parentID="{parent}" restricted="1" childCount="{}"><dc:title>{title}</dc:title><upnp:class>object.container.storageFolder</upnp:class></container>"#, index.child_count(&entry.id))
    } else {
        let transcode = match transcode_mode { "on" | "1080p" | "720p" => true, "auto" => needs_transcode(&entry.path), _ => false };
        let mime = if transcode { "video/mp2t" } else { mime_for(&entry.path) };
        let url = format!("http://{}:{}/media/{}", host, port, entry.id);
        // Advertise the standard DLNA streaming and byte-seek capabilities.
        // The previous wildcard additionalInfo was accepted by generic HTTP
        // players, but some DLNA browsers (including strict catalogue parsers)
        // do not expose resources unless the DLNA.ORG operation/flag fields
        // are present. Do not invent DLNA.ORG_PN: it depends on the actual
        // codecs inside the file and cannot safely be inferred from extension.
        let flags = if transcode { "DLNA.ORG_OP=00;DLNA.ORG_CI=1" } else { "DLNA.ORG_OP=01;DLNA.ORG_CI=0" };
        let protocol_info = format!("http-get:*:{}:{};DLNA.ORG_FLAGS=01700000000000000000000000000000", mime, flags);
        let thumb_url = format!("http://{}:{}/thumb/{}.jpg", host, port, entry.id);
        format!(r#"<item id="{id}" parentID="{parent}" restricted="1"><dc:title>{title}</dc:title><upnp:class>object.item.videoItem</upnp:class><upnp:albumArtURI dlna:profileID="JPEG_TN">{}</upnp:albumArtURI><res protocolInfo="{}" size="{}">{}</res></item>"#, escape(&thumb_url), protocol_info, entry.size, escape(&url))
    }
}

pub fn didl_document(entries: &[&Entry], index: &MediaIndex, host: &str, port: u16, transcode_mode: &str) -> String {
    let mut out = String::from(r#"<DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/">"#);
    for entry in entries { out.push_str(&didl_entry(entry, index, host, port, transcode_mode)); }
    out.push_str("</DIDL-Lite>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escaping_round_trips() {
        let input = "Сериал & <серия> \"01\"";
        assert_eq!(unescape(&escape(input)), input);
    }


    #[test]
    fn video_entry_is_advertised_as_playable_resource() {
        use std::path::PathBuf;
        let video = Entry {
            id: "v0123456789abcdef".into(),
            parent: "d0123456789abcdef".into(),
            title: "Серия 01.mkv".into(),
            path: PathBuf::from("/media/Серия 01.mkv"),
            is_dir: false,
            size: 1234,
        };
        let index = MediaIndex::default();
        let didl = didl_document(&[&video], &index, "192.168.1.10", 8208, "off");
        assert!(didl.contains("<upnp:class>object.item.videoItem</upnp:class>"));
        assert!(didl.contains("protocolInfo=\"http-get:*:video/x-matroska:DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000\""));
        assert!(didl.contains("size=\"1234\""));
        assert!(didl.contains("http://192.168.1.10:8208/media/v0123456789abcdef"));
        assert!(didl.contains("Серия 01.mkv"));
    }

    #[test]
    fn device_advertises_full_size_brand_icon_first() {
        let device = device_xml("DLNA SERVER", "1234", "192.168.1.10", 8208);
        let first_icon = device.find("/brand-v3-256.png").unwrap();
        let small_icon = device.find("/brand-v3-48.png").unwrap();
        assert!(first_icon < small_icon);
        assert!(device.contains("<presentationURL>http://192.168.1.10:8208/</presentationURL>"));
    }

    #[test]
    fn soap_arguments_are_extracted() {
        let body = "<u:Browse><ObjectID>0</ObjectID><BrowseFlag>BrowseDirectChildren</BrowseFlag></u:Browse>";
        assert_eq!(tag_value(body, "ObjectID").as_deref(), Some("0"));
        assert_eq!(tag_value(body, "BrowseFlag").as_deref(), Some("BrowseDirectChildren"));
    }
}
