use std::collections::HashMap;

#[derive(Debug)]
pub struct SmnResponse {
    pub version: String,
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl SmnResponse {
    pub fn new(
        status_code: u16,
        reason_phrase: &str,
        body: Vec<u8>,
    ) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());
        headers.insert("Connection".to_string(), "close".to_string());

        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            reason_phrase: reason_phrase.to_string(),
            headers,
            body,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Status line
        response.extend_from_slice(
            format!(
                "{} {} {}\r\n",
                self.version,
                self.status_code,
                self.reason_phrase
            )
            .as_bytes(),
        );

        // Headers
        for (key, value) in &self.headers {
            response.extend_from_slice(
                format!("{}: {}\r\n", key, value).as_bytes(),
            );
        }

        // Header/body separator
        response.extend_from_slice(b"\r\n");

        // Body
        response.extend_from_slice(&self.body);

        response
    }
}
