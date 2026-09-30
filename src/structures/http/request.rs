use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SmnRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl SmnRequest {
    pub fn from_buffer(buffer: &[u8]) -> Result<Self, &'static str> {
        // Split headers and body
        let request_str = match std::str::from_utf8(buffer) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 in request"),
        };

        let (head, body) = match request_str.split_once("\r\n\r\n") {
            Some(parts) => parts,
            None => return Err("Malformed HTTP request"),
        };

        let mut lines = head.lines();

        // Parse request line: METHOD PATH VERSION
        let request_line = lines.next().ok_or("Missing request line")?;
        let mut parts = request_line.split_whitespace();

        let method = parts.next().ok_or("Missing method")?.to_string();
        let raw_path = parts.next().ok_or("Missing path")?;
        let version = parts.next().ok_or("Missing version")?.to_string();

        // Strip query string (e.g. ?utm_source=...&fbclid=...)
        let path = match raw_path.split_once('?') {
            Some((clean, _query)) => clean.to_string(),
            None => raw_path.to_string(),
        };

        // Parse headers
        let mut headers = HashMap::new();

        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(Self {
            method,
            path,
            version,
            headers,
            body: body.as_bytes().to_vec(),
        })
    }
}
