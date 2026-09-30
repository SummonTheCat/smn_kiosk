use std::process::Command;

use crate::structures::{PluginBase, SmnRequest, SmnResponse};

pub struct PluginStatus {
    port: u16,
}

impl PluginStatus {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    fn network_addresses() -> Vec<(String, String)> {
        let output = match Command::new("ip")
            .args(["-o", "addr", "show", "up"])
            .output()
        {
            Ok(output) if output.status.success() => output,
            _ => return Vec::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut addresses = Vec::new();

        for line in stdout.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 4 {
                continue;
            }

            let interface = fields[1].trim_end_matches(':');
            let family = fields[2];
            let address = fields[3].split('/').next().unwrap_or("");

            if interface == "lo" || address.is_empty() {
                continue;
            }

            if family != "inet" && family != "inet6" {
                continue;
            }

            if family == "inet6" && address.starts_with("fe80:") {
                continue;
            }

            let address = address.to_string();
            if !addresses.iter().any(|(_, existing)| existing == &address) {
                addresses.push((interface.to_string(), address));
            }
        }

        addresses
    }

    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn serve_status(&self) -> SmnResponse {
        let addresses = Self::network_addresses();
        let mut network_rows = String::new();

        if addresses.is_empty() {
            network_rows.push_str("<tr><td colspan=\"3\">No active non-loopback network addresses found.</td></tr>");
        } else {
            for (interface, address) in addresses {
                let interface = Self::escape_html(&interface);
                let address = Self::escape_html(&address);
                let host = if address.contains(':') {
                    format!("[{}]", address)
                } else {
                    address.clone()
                };
                let url = format!("http://{}:{}", host, self.port);

                network_rows.push_str(&format!(
                    "<tr><td>{}</td><td><code>{}</code></td><td><a href=\"{}\">{}</a></td></tr>",
                    interface, address, url, url
                ));
            }
        }

        let html = format!(r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>SMN Kiosk Status</title>
<style>
body {{ font-family: system-ui, sans-serif; max-width: 900px; margin: 40px auto; padding: 0 20px; background: #111; color: #eee; }}
h1 {{ margin-bottom: 8px; }}
p {{ color: #aaa; }}
table {{ width: 100%; border-collapse: collapse; margin-top: 24px; }}
th, td {{ text-align: left; padding: 12px; border-bottom: 1px solid #333; }}
th {{ color: #aaa; }}
a {{ color: #8ab4f8; }}
code {{ color: #ddd; }}
</style>
</head>
<body>
<h1>SMN Kiosk</h1>
<p>Server is running on port <strong>{port}</strong>.</p>
<table>
<thead><tr><th>Interface</th><th>Address</th><th>Server URL</th></tr></thead>
<tbody>
<tr><td>local</td><td><code>127.0.0.1</code></td><td><a href="http://127.0.0.1:{port}">http://127.0.0.1:{port}</a></td></tr>
{network_rows}
</tbody>
</table>
</body>
</html>"#, port = self.port, network_rows = network_rows);

        SmnResponse::new(200, "OK", html.into_bytes())
            .with_header("Content-Type", "text/html; charset=utf-8")
            .with_header("Cache-Control", "no-store")
    }
}

impl PluginBase for PluginStatus {
    fn name(&self) -> &str {
        "status"
    }

    fn init(&mut self) {
        println!("Status plugin initialized at /status (port {})", self.port);
    }

    fn can_serve(&self, request: &SmnRequest) -> bool {
        request.method == "GET" && request.path == "/status"
    }

    fn serve(&self, request: &SmnRequest) -> SmnResponse {
        if request.method == "GET" && request.path == "/status" {
            return self.serve_status();
        }

        SmnResponse::new(404, "Not Found", b"Not Found".to_vec())
            .with_header("Content-Type", "text/plain; charset=utf-8")
    }
}
