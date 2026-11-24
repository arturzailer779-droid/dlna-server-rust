use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::PermissionsExt;

#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub parent: String,
    pub title: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Default, Clone)]
pub struct MediaIndex {
    pub entries: Vec<Entry>,
    pub by_id: HashMap<String, usize>,
    pub update_id: u32,
}

impl MediaIndex {
    pub fn scan(roots: &[PathBuf], previous: u32) -> Self {
        let mut index = Self { entries: Vec::new(), by_id: HashMap::new(), update_id: previous.wrapping_add(1) };
        let mut canonical_roots = Vec::new();
        for root in roots {
            if let Ok(path) = fs::canonicalize(root)
                && path.is_dir() && !canonical_roots.contains(&path) { canonical_roots.push(path); }
        }
        for root in canonical_roots {
            let title = root.file_name().and_then(|s| s.to_str()).unwrap_or("Video").to_string();
            let id = make_id('d', &root);
            index.add(Entry { id: id.clone(), parent: "0".into(), title, path: root.clone(), is_dir: true, size: 0 });
            index.scan_dir(&root, &id, 0);
        }
        index
    }

    fn scan_dir(&mut self, dir: &Path, parent_id: &str, depth: usize) {
        if depth > 48 { return; }
        let mut children: Vec<_> = match fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(Result::ok).collect(),
            Err(_) => return,
        };
        children.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());
        for child in children {
            let path = child.path();
            let file_type = match child.file_type() { Ok(t) => t, Err(_) => continue };
            if file_type.is_symlink() { continue; }
            if file_type.is_dir() {
                let canonical = match fs::canonicalize(&path) { Ok(p) => p, Err(_) => continue };
                let title = child.file_name().to_string_lossy().into_owned();
                let id = make_id('d', &canonical);
                self.add(Entry { id: id.clone(), parent: parent_id.into(), title, path: canonical.clone(), is_dir: true, size: 0 });
                self.scan_dir(&canonical, &id, depth + 1);
            } else if file_type.is_file() && is_video(&path) {
                let metadata = match fs::metadata(&path) { Ok(m) => m, Err(_) => continue };
                let canonical = match fs::canonicalize(&path) { Ok(p) => p, Err(_) => continue };
                let title = child.file_name().to_string_lossy().into_owned();
                let id = make_id('v', &canonical);
                self.add(Entry { id, parent: parent_id.into(), title, path: canonical, is_dir: false, size: metadata.len() });
            }
        }
    }

    fn add(&mut self, entry: Entry) {
        if self.by_id.contains_key(&entry.id) { return; }
        let pos = self.entries.len();
        self.by_id.insert(entry.id.clone(), pos);
        self.entries.push(entry);
    }

    pub fn children(&self, parent: &str) -> Vec<&Entry> {
        let mut items: Vec<&Entry> = self.entries.iter().filter(|e| e.parent == parent).collect();
        items.sort_by_key(|e| (!e.is_dir, e.title.to_lowercase()));
        items
    }

    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.by_id.get(id).and_then(|i| self.entries.get(*i))
    }

    pub fn child_count(&self, id: &str) -> usize {
        self.entries.iter().filter(|e| e.parent == id).count()
    }

    pub fn video_count(&self) -> usize { self.entries.iter().filter(|e| !e.is_dir).count() }

    pub fn load_cache(roots: &[PathBuf], cache_file: &Path) -> Option<Self> {
        if cache_file.as_os_str().is_empty() { return None; }
        let text = fs::read_to_string(cache_file).ok()?;
        let mut lines = text.lines();
        let header = lines.next()?;
        let mut parts = header.split('\t');
        if parts.next()? != "DLNAIDX1" { return None; }
        let expected_fingerprint = root_fingerprint(roots);
        if parts.next()? != expected_fingerprint.as_str() { return None; }
        let update_id = parts.next()?.parse::<u32>().ok()?;
        let canonical_roots: Vec<PathBuf> = roots.iter().filter_map(|p| fs::canonicalize(p).ok()).collect();
        let mut index = Self { entries: Vec::new(), by_id: HashMap::new(), update_id };
        for line in lines {
            let fields: Vec<&str> = line.splitn(5, '\t').collect();
            if fields.len() != 5 { continue; }
            let id = fields[0].to_string();
            let parent = fields[1].to_string();
            let is_dir = fields[2] == "D";
            if !is_dir && fields[2] != "V" { continue; }
            let encoded_path = match hex_decode(fields[4]) { Some(v) => v, None => continue };
            let path = PathBuf::from(std::ffi::OsString::from_vec(encoded_path));
            let canonical = match fs::canonicalize(&path) { Ok(p) => p, Err(_) => continue };
            if !canonical_roots.iter().any(|root| canonical.starts_with(root)) { continue; }
            if is_dir && !canonical.is_dir() || !is_dir && (!canonical.is_file() || !is_video(&canonical)) { continue; }
            if parent != "0" && !index.get(&parent).map(|e| e.is_dir).unwrap_or(false) { continue; }
            let prefix = if is_dir { 'd' } else { 'v' };
            if id != make_id(prefix, &canonical) { continue; }
            let metadata = match fs::metadata(&canonical) { Ok(m) => m, Err(_) => continue };
            let title = canonical.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Video".into());
            index.add(Entry { id, parent, title, path: canonical, is_dir, size: if is_dir { 0 } else { metadata.len() } });
        }
        if index.entries.is_empty() { return None; }
        Some(index)
    }

    pub fn save_cache(&self, roots: &[PathBuf], cache_file: &Path) -> std::io::Result<()> {
        if cache_file.as_os_str().is_empty() { return Ok(()); }
        if let Some(parent) = cache_file.parent() { fs::create_dir_all(parent)?; }
        let temp = cache_file.with_extension(format!("tmp-{}", std::process::id()));
        let _ = fs::remove_file(&temp);
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
        writeln!(file, "DLNAIDX1\t{}\t{}", root_fingerprint(roots), self.update_id)?;
        for entry in &self.entries {
            let kind = if entry.is_dir { "D" } else { "V" };
            writeln!(file, "{}\t{}\t{}\t{}\t{}", entry.id, entry.parent, kind, entry.size, hex_encode(entry.path.as_os_str().as_bytes()))?;
        }
        file.sync_all()?;
        fs::rename(&temp, cache_file)?;
        fs::set_permissions(cache_file, fs::Permissions::from_mode(0o600))?;
        Ok(())
    }
}


fn root_fingerprint(roots: &[PathBuf]) -> String {
    let mut canonical: Vec<String> = roots.iter().filter_map(|p| fs::canonicalize(p).ok()).map(|p| p.to_string_lossy().into_owned()).collect();
    canonical.sort();
    let joined = canonical.join("\n");
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in joined.as_bytes() { hash ^= *byte as u64; hash = hash.wrapping_mul(0x100000001b3); }
    format!("{:016x}", hash)
}
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes { out.push(HEX[(byte >> 4) as usize] as char); out.push(HEX[(byte & 0x0f) as usize] as char); }
    out
}
fn hex_decode(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) { return None; }
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.as_chunks::<2>().0 {
        let high = (pair[0] as char).to_digit(16)? as u8;
        let low = (pair[1] as char).to_digit(16)? as u8;
        out.push((high << 4) | low);
    }
    Some(out)
}

fn make_id(prefix: char, path: &Path) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{}{:016x}", prefix, hash)
}

pub fn is_video(path: &Path) -> bool {
    matches!(path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase().as_str(),
        "mkv" | "mp4" | "m4v" | "avi" | "mov" | "mpeg" | "mpg" | "ts" | "m2ts" | "mts" | "wmv" | "webm" | "vob" | "3gp" | "flv" | "ogv")
}

pub fn mime_for(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "mkv" => "video/x-matroska", "mp4" | "m4v" => "video/mp4", "avi" => "video/x-msvideo",
        "mov" => "video/quicktime", "mpeg" | "mpg" => "video/mpeg", "ts" | "m2ts" | "mts" => "video/mp2t",
        "wmv" => "video/x-ms-wmv", "webm" => "video/webm", "vob" => "video/dvd", "3gp" => "video/3gpp",
        "flv" => "video/x-flv", "ogv" => "video/ogg", _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn video_filter_is_video_only() {
        assert!(is_video(Path::new("episode.mkv")));
        assert!(is_video(Path::new("episode.MP4")));
        assert!(!is_video(Path::new("cover.jpg")));
        assert!(!is_video(Path::new("track.mp3")));
    }

    #[test]
    fn scan_indexes_only_video_and_keeps_nested_folders() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("dlna-core-test-{stamp}"));
        let season = root.join("Season 01");
        fs::create_dir_all(&season).unwrap();
        let mut video = fs::File::create(season.join("Episode 01.mkv")).unwrap();
        video.write_all(b"test").unwrap();
        fs::write(root.join("poster.jpg"), b"not video").unwrap();
        let index = MediaIndex::scan(std::slice::from_ref(&root), 0);
        assert_eq!(index.video_count(), 1);
        assert!(index.entries.iter().any(|e| e.title == "Season 01" && e.is_dir));
        assert!(!index.entries.iter().any(|e| e.title == "poster.jpg"));
        let cache = root.with_extension("index.tsv");
        index.save_cache(std::slice::from_ref(&root), &cache).unwrap();
        let restored = MediaIndex::load_cache(std::slice::from_ref(&root), &cache).unwrap();
        assert_eq!(restored.video_count(), 1);
        assert!(restored.entries.iter().any(|e| e.title == "Episode 01.mkv"));
        fs::remove_file(cache).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
