use std::io::BufReader;

use http_rs::http::{HttpMethod, Request, Response, Server};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

fn main() {
    let server = Server::new("127.0.0.1:6969");
    println!("Server running on http://127.0.0.1:6969");

    for stream in server.listen() {
        match stream {
            Ok(mut stream) => {
                let buf = BufReader::new(stream.try_clone().unwrap());

                if let Ok(req) = Request::new(buf) {
                    let response = match (req.method, req.route.as_str()) {
                        (HttpMethod::GET, "/users") => {
                            let users = vec![
                                User {
                                    id: 1,
                                    name: "Alice".to_string(),
                                },
                                User {
                                    id: 2,
                                    name: "Bob".to_string(),
                                },
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
}
