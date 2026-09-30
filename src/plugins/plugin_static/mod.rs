use std::fs;
use std::path::PathBuf;

use crate::structures::{PluginBase, SmnRequest, SmnResponse};

pub struct PluginStatic {
    pub root: PathBuf,
}

impl PluginStatic {
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("./res/static"),
        }
    }

    fn resolve_path(&self, request_path: &str) -> Option<PathBuf> {
        let clean = request_path.trim_start_matches('/');

        if clean.contains("..") {
            return None;
        }

        if clean.is_empty() {
            return Some(self.root.join("index.html"));
        }

        let mut candidate = self.root.join(clean);

        if candidate.extension().is_none() {
            candidate.set_extension("html");
        }

        Some(candidate)
    }

    fn content_type(ext: &str) -> Option<&'static str> {
        match ext {
            "html" => Some("text/html; charset=utf-8"),
            "css" => Some("text/css; charset=utf-8"),
            "js" => Some("text/javascript; charset=utf-8"),
            "png" => Some("image/png"),
            "svg" => Some("image/svg+xml"),
            "webp" => Some("image/webp"),
            "ico" => Some("image/x-icon"),
            "webm" => Some("video/webm"),
            "mp4" => Some("video/mp4"),
            "zip" => Some("application/zip"),
            _ => None,
        }
    }

    fn is_zip_under_limit(path: &PathBuf) -> bool {
        const MAX_SIZE: u64 = 50 * 1024 * 1024;
        match fs::metadata(path) {
            Ok(meta) => meta.len() <= MAX_SIZE,
            Err(_) => false,
        }
    }
}

impl PluginBase for PluginStatic {
    fn name(&self) -> &str {
        "static_files"
    }

    fn init(&mut self) {
        println!("Static file plugin initialized at {:?}", self.root);
    }

    fn can_serve(&self, request: &SmnRequest) -> bool {
        if request.method != "GET" {
            return false;
        }

        let path = match self.resolve_path(&request.path) {
            Some(p) => p,
            None => return false,
        };

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => return false,
        };

        if Self::content_type(ext).is_none() || !path.exists() {
            return false;
        }

        if ext == "zip" && !Self::is_zip_under_limit(&path) {
            return false;
        }

        true
    }

    fn serve(&self, request: &SmnRequest) -> SmnResponse {
        let path = match self.resolve_path(&request.path) {
            Some(p) => p,
            None => {
                return SmnResponse::new(400, "Bad Request", b"Bad Request".to_vec())
                    .with_header("Content-Type", "text/plain");
            }
        };

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => {
                return SmnResponse::new(
                    415,
                    "Unsupported Media Type",
                    b"Unsupported Media Type".to_vec(),
                )
                .with_header("Content-Type", "text/plain");
            }
        };

        let content_type = match Self::content_type(ext) {
            Some(ct) => ct,
            None => {
                return SmnResponse::new(
                    415,
                    "Unsupported Media Type",
                    b"Unsupported Media Type".to_vec(),
                )
                .with_header("Content-Type", "text/plain");
            }
        };

        if ext == "zip" && !Self::is_zip_under_limit(&path) {
            return SmnResponse::new(
                413,
                "Payload Too Large",
                b"File exceeds 50MB limit".to_vec(),
            )
            .with_header("Content-Type", "text/plain");
        }

        match fs::read(&path) {
            Ok(bytes) => {
                let mut response =
                    SmnResponse::new(200, "OK", bytes).with_header("Content-Type", content_type);

                if ext == "zip" {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        response = response.with_header(
                            "Content-Disposition",
                            &format!("attachment; filename=\"{}\"", filename),
                        );
                    }
                }

                response
            }
            Err(_) => SmnResponse::new(404, "Not Found", b"Not Found".to_vec())
                .with_header("Content-Type", "text/plain"),
        }
    }
}
