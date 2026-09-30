use std::sync::RwLock;

use crate::structures::{PluginBase, SmnRequest, SmnResponse};

pub struct PluginBoard {
    buffer: RwLock<Vec<u8>>,
}

impl PluginBoard {
    pub fn new() -> Self {
        Self {
            buffer: RwLock::new(Self::create_default_board()),
        }
    }

    fn create_default_board() -> Vec<u8> {
        const WIDTH: usize = 32;
        const HEIGHT: usize = 32;

        const SKY: u8 = 1;
        const SUN: u8 = 2;
        const GRASS: u8 = 3;
        const GRASS_LIGHT: u8 = 4;
        const MOUNTAIN: u8 = 5;
        const MOUNTAIN_LIGHT: u8 = 6;

        let mut data = vec![SKY; WIDTH * HEIGHT];

        let set = |data: &mut Vec<u8>, x: usize, y: usize, color: u8| {
            if x < WIDTH && y < HEIGHT {
                data[(y * WIDTH) + x] = color;
            }
        };

        /*
         * Sun.
         */
        for y in 3..8 {
            for x in 24..29 {
                set(&mut data, x, y, SUN);
            }
        }

        /*
         * Left mountain.
         */
        for y in 12..26 {
            let row = y - 12;

            let start = 9usize.saturating_sub(row);
            let end = (9 + row).min(WIDTH - 1);

            for x in start..=end {
                set(&mut data, x, y, MOUNTAIN);
            }
        }

        /*
         * Right mountain.
         */
        for y in 15..26 {
            let row = y - 15;

            let start = 23usize.saturating_sub(row);
            let end = (23 + row).min(WIDTH - 1);

            for x in start..=end {
                set(&mut data, x, y, MOUNTAIN);
            }
        }

        /*
         * Snow/highlights.
         */
        set(&mut data, 9, 12, MOUNTAIN_LIGHT);

        set(&mut data, 8, 13, MOUNTAIN_LIGHT);
        set(&mut data, 9, 13, MOUNTAIN_LIGHT);
        set(&mut data, 10, 13, MOUNTAIN_LIGHT);

        set(&mut data, 7, 14, MOUNTAIN_LIGHT);
        set(&mut data, 8, 14, MOUNTAIN_LIGHT);
        set(&mut data, 10, 14, MOUNTAIN_LIGHT);
        set(&mut data, 11, 14, MOUNTAIN_LIGHT);

        set(&mut data, 23, 15, MOUNTAIN_LIGHT);

        set(&mut data, 22, 16, MOUNTAIN_LIGHT);
        set(&mut data, 23, 16, MOUNTAIN_LIGHT);
        set(&mut data, 24, 16, MOUNTAIN_LIGHT);

        /*
         * Grass foreground.
         */
        for y in 26..HEIGHT {
            for x in 0..WIDTH {
                let color = if (x + y) % 5 == 0 { GRASS_LIGHT } else { GRASS };

                set(&mut data, x, y, color);
            }
        }

        let colors = [
            "#172554", "#facc15", "#166534", "#22c55e", "#334155", "#94a3b8",
        ];

        let colors_json = colors
            .iter()
            .map(|color| format!("\"{}\"", color))
            .collect::<Vec<_>>()
            .join(",");

        let data_json = data
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"width\":{},\"height\":{},\"colors\":[{}],\"data\":[{}]}}",
            WIDTH, HEIGHT, colors_json, data_json
        )
        .into_bytes()
    }
    
    fn is_get_route(request: &SmnRequest) -> bool {
        request.method == "GET" && request.path == "/board/get"
    }

    fn is_set_route(request: &SmnRequest) -> bool {
        request.method == "POST" && request.path == "/board/set"
    }

    fn get_buffer(&self) -> SmnResponse {
        let buffer = match self.buffer.read() {
            Ok(buffer) => buffer,
            Err(_) => {
                return SmnResponse::new(
                    500,
                    "Internal Server Error",
                    b"Board buffer lock poisoned".to_vec(),
                )
                .with_header("Content-Type", "text/plain; charset=utf-8");
            }
        };

        SmnResponse::new(200, "OK", buffer.clone())
            .with_header("Content-Type", "application/json; charset=utf-8")
    }

    fn set_buffer(&self, request: &SmnRequest) -> SmnResponse {
        if request.body.is_empty() {
            return SmnResponse::new(400, "Bad Request", b"Board buffer is required".to_vec())
                .with_header("Content-Type", "text/plain; charset=utf-8");
        }

        if std::str::from_utf8(&request.body).is_err() {
            return SmnResponse::new(
                400,
                "Bad Request",
                b"Board buffer must be UTF-8 JSON".to_vec(),
            )
            .with_header("Content-Type", "text/plain; charset=utf-8");
        }

        let mut buffer = match self.buffer.write() {
            Ok(buffer) => buffer,
            Err(_) => {
                return SmnResponse::new(
                    500,
                    "Internal Server Error",
                    b"Board buffer lock poisoned".to_vec(),
                )
                .with_header("Content-Type", "text/plain; charset=utf-8");
            }
        };

        *buffer = request.body.clone();

        SmnResponse::new(200, "OK", b"{\"success\":true}".to_vec())
            .with_header("Content-Type", "application/json; charset=utf-8")
    }
}

impl PluginBase for PluginBoard {
    fn name(&self) -> &str {
        "board"
    }

    fn init(&mut self) {
        println!("Board plugin initialized with default 32x32 board");
    }

    fn can_serve(&self, request: &SmnRequest) -> bool {
        Self::is_get_route(request) || Self::is_set_route(request)
    }

    fn serve(&self, request: &SmnRequest) -> SmnResponse {
        if Self::is_get_route(request) {
            return self.get_buffer();
        }

        if Self::is_set_route(request) {
            return self.set_buffer(request);
        }

        SmnResponse::new(404, "Not Found", b"Not Found".to_vec())
            .with_header("Content-Type", "text/plain; charset=utf-8")
    }
}
