use bliss_traits::net::{NetHandler, Request};
use bytes::Bytes;

pub trait SchemeHandler: Send + Sync + 'static {
    fn schemes(&self) -> &[&str];
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>);
}

pub struct DataSchemeHandler;

impl SchemeHandler for DataSchemeHandler {
    fn schemes(&self) -> &[&str] {
        &["data"]
    }

    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url_str = request.url.to_string();

        let Some(rest) = url_str.strip_prefix("data:") else {
            tracing::warn!(url = %request.url, "malformed data URI");
            return;
        };

        let (metadata, data) = match rest.split_once(',') {
            Some((m, d)) => (m, d),
            None => {
                tracing::warn!(url = %request.url, "malformed data URI: missing comma");
                return;
            }
        };

        let decoded = if metadata.ends_with(";base64") {
            match base64_decode(data) {
                Some(bytes) => Bytes::from(bytes),
                None => {
                    tracing::warn!(url = %request.url, "invalid base64 in data URI");
                    return;
                }
            }
        } else {
            Bytes::from(percent_decode(data))
        };

        handler.bytes(url_str, decoded);
    }
}

pub struct FileSchemeHandler;

impl SchemeHandler for FileSchemeHandler {
    fn schemes(&self) -> &[&str] {
        &["file"]
    }

    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url_str = request.url.to_string();

        let path = match request.url.to_file_path() {
            Ok(p) => p,
            Err(()) => {
                tracing::warn!(url = %request.url, "invalid file URL");
                return;
            }
        };

        match std::fs::read(&path) {
            Ok(contents) => {
                tracing::debug!(path = %path.display(), "read file");
                handler.bytes(url_str, Bytes::from(contents));
            }
            Err(err) => {
                tracing::warn!(path = %path.display(), %err, "failed to read file");
            }
        }
    }
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in input.as_bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' | b' ' | b'\t' => continue,
            _ => return None,
        };
        buf = (buf << 6) | u32::from(val);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    Some(output)
}

fn percent_decode(input: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut bytes = input.bytes();

    while let Some(b) = bytes.next() {
        if b == b'%' {
            let decoded = bytes
                .next()
                .and_then(hex_val)
                .zip(bytes.next().and_then(hex_val))
                .map(|(hi, lo)| (hi << 4) | lo);
            match decoded {
                Some(val) => output.push(val),
                None => output.push(b),
            }
        } else {
            output.push(b);
        }
    }

    output
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
