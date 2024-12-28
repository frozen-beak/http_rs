# 🦀 HTTP-RS

A lightweight and simple HTTP server implementation in Rust

## Quick Start

Here's a simple example of creating a REST API:

```rust
use std::io::{self, BufReader};
use http_rs::server::{HttpMethod, Request, Response, Server};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

fn main() -> io::Result<()> {
    let server = Server::new("127.0.0.1:6969")?;
    println!("Server running on http://127.0.0.1:6969");

    for stream in server.listen() {
        match stream {
            Ok(mut stream) => {
                let buf = BufReader::new(stream.try_clone().unwrap());

                if let Ok(req) = Request::new(buf) {
                    let response = match (req.method, req.route.as_str()) {
                        (HttpMethod::GET, "/users") => {
                            let users = vec![
                                User { id: 1, name: "Alice".to_string() },
                                User { id: 2, name: "Bob".to_string() },
                            ];
                            Response::new(200).json(&users)
                        }
                        (HttpMethod::POST, "/users") => {
                            if let Some(user) = req.get_json::<User>() {
                                Response::new(201).json(&user)
                            } else {
                                Response::new(400).json(&"Invalid JSON")
                            }
                        }
                        _ => Response::new(404).json(&"Not Found"),
                    };

                    if let Err(e) = response.send(&mut stream) {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
```

## API Documentation

### Server

```rust
// Create a new server instance
let server = Server::new("127.0.0.1:8080")?;

// Listen for incoming connections
for stream in server.listen() {
    // Handle connections
}
```

### Request

The `Request` struct provides access to:

- HTTP method (`GET`, `POST`)
- Route path
- Headers
- Query parameters
- Request body
- JSON parsing with `get_json<T>()`

### Response

The `Response` struct allows:

- Setting status codes
- Adding headers
- Sending JSON responses
- Proper HTTP formatting

## Testing

Run the test suite:

```bash
cargo test
```
