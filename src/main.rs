pub mod managers;
pub mod plugins;
pub mod structures;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

use crate::managers::PluginManager;
use crate::plugins::plugin_board::PluginBoard;
use crate::plugins::plugin_static::PluginStatic;
use crate::structures::{SmnRequest, SmnResponse};

fn handle_client(mut stream: TcpStream, plugin_manager: &PluginManager) {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 512];

    let header_end;

    loop {
        match stream.read(&mut temp) {
            Ok(0) => return,
            Ok(n) => {
                buffer.extend_from_slice(&temp[..n]);

                if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                    header_end = pos + 4;
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from connection: {}", e);
                return;
            }
        }
    }

    let content_length = match parse_content_length(&buffer[..header_end]) {
        Ok(length) => length,
        Err(()) => {
            let response = SmnResponse::new(
                400,
                "Bad Request",
                b"Invalid Content-Length".to_vec(),
            )
            .with_header("Content-Type", "text/plain");

            let _ = stream.write_all(&response.to_bytes());
            return;
        }
    };

    let total_request_length = header_end + content_length;

    while buffer.len() < total_request_length {
        match stream.read(&mut temp) {
            Ok(0) => {
                let response = SmnResponse::new(
                    400,
                    "Bad Request",
                    b"Incomplete request body".to_vec(),
                )
                .with_header("Content-Type", "text/plain");

                let _ = stream.write_all(&response.to_bytes());
                return;
            }
            Ok(n) => {
                buffer.extend_from_slice(&temp[..n]);
            }
            Err(e) => {
                eprintln!("Failed to read request body: {}", e);
                return;
            }
        }
    }

    let request_bytes = &buffer[..total_request_length];

    let request = match SmnRequest::from_buffer(request_bytes) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Failed to parse request: {}", e);

            let response = SmnResponse::new(
                400,
                "Bad Request",
                b"Bad Request".to_vec(),
            )
            .with_header("Content-Type", "text/plain");

            let _ = stream.write_all(&response.to_bytes());
            return;
        }
    };

    let response = plugin_manager.route(&request);

    let _ = stream.write_all(&response.to_bytes());
}

fn parse_content_length(header_bytes: &[u8]) -> Result<usize, ()> {
    let headers = std::str::from_utf8(header_bytes).map_err(|_| ())?;

    for line in headers.lines().skip(1) {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };

        if key.trim().eq_ignore_ascii_case("Content-Length") {
            return value.trim().parse::<usize>().map_err(|_| ());
        }
    }

    Ok(0)
}

fn parse_port() -> Result<u16, String> {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--port" {
            let value = args
                .next()
                .ok_or_else(|| "Missing value after --port".to_string())?;

            return value
                .parse::<u16>()
                .map_err(|_| format!("Invalid port: {}", value));
        }
    }

    Ok(8000)
}

fn main() -> std::io::Result<()> {
    let public = std::env::args().any(|arg| arg == "--public");

    let port = match parse_port() {
        Ok(port) => port,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(2);
        }
    };

    let host = if public {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };

    let bind_addr = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&bind_addr)?;
    println!("Server listening on http://{}", bind_addr);

    // ---- Plugin system bootstrap ----
    let mut plugin_manager = PluginManager::new();

    plugin_manager.register(Box::new(PluginBoard::new()));
    plugin_manager.register(Box::new(PluginStatic {
        root: PathBuf::from("res/static"),
    }));

    plugin_manager.init_all();
    // ---------------------------------

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, &plugin_manager);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}
