//!
//! A simple HTTP implementation.
//!
//! # Example
//!
//! ```rust, no_run
//! use http_rs::server::{Server, Request, Response, HttpMethod};
//! use std::io::BufReader;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!    id: u32,
//!    name: String,
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     let server = Server::new("127.0.0.1:8080")?;
//!
//!     for stream in server.listen() {
//!         match stream {
//!             Ok(mut stream) => {
//!                 let buf = BufReader::new(stream.try_clone().unwrap());
//!
//!                 if let Ok(req) = Request::new(buf) {
//!                     let response = match (req.method, req.route.as_str()) {
//!                         (HttpMethod::POST, "/users") => {
//!                             if let Some(user) = req.get_json::<User>() {
//!                                 Response::new(201).json(&user)
//!                             } else {
//!                                 Response::new(400).json(&"Invalid JSON")
//!                             }
//!                         }
//!                         _ => Response::new(404).json(&"Not Found"),
//!                     };
//!
//!                     if let Err(e) = response.send(&mut stream) {
//!                         eprintln!("Failed to send response: {}", e);
//!                     }
//!                 }
//!             }
//!             Err(e) => eprintln!("Connection failed: {}", e),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!

use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    collections::HashMap,
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

///
/// Represents HTTP methods supported by the server.
///
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST,
}

///
/// Alias for HTTP headers as KV pairs.
///
pub type Headers = HashMap<String, String>;

///
/// Alias for URL query params as KV pairs.
///
pub type QueryParams = HashMap<String, String>;

///
/// Representation of HTTP request
///
#[derive(Debug)]
pub struct Request {
    ///
    /// The requested route/path
    ///
    pub route: String,

    ///
    /// The [HttpMethod] used
    ///
    pub method: HttpMethod,

    ///
    /// HTTP request [Headers]
    ///
    pub headers: Headers,

    ///
    /// Parsed [QueryParams] from the URL
    ///
    pub query_params: QueryParams,

    ///
    /// Request body as raw bytes
    ///
    pub body: Vec<u8>,
}

///
/// Representation of HTTP response
///
#[derive(Serialize)]
pub struct Response {
    ///
    /// HTTP status code
    ///
    status: u16,

    ///
    /// Response [Headers]
    ///
    headers: Headers,

    ///
    /// Response body as a string
    ///
    /// `👉 Note:` Only json is supported
    ///
    body: String,
}

///
/// HTTP Implementation which handles TCP connections
///
pub struct Server {
    listener: TcpListener,
}

impl Server {
    ///
    /// Creates a new HTTP server bound to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` -> Address to bind to (e.g., "0.0.0.0:8080")
    ///
    /// # Returns
    ///
    /// * `io::Result<Server>` -> The server instance or an [std::io] error
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// let server = Server::new("127.0.0.1:8080")?;
    /// ```
    ///
    pub fn new(addr: &str) -> io::Result<Server> {
        let listener = TcpListener::bind(addr)?;

        Ok(Server { listener })
    }

    ///
    /// Returns an iterator over incoming TCP connections.
    ///
    /// # Returns
    ///
    /// `io::Result<TcpStream>` -> An iterator yielding for each incoming connection
    /// or an [std::io] error
    ///
    pub fn listen(&self) -> impl Iterator<Item = io::Result<TcpStream>> + '_ {
        self.listener.incoming()
    }
}

impl Request {
    ///
    /// Creates a new [Request] instance by parsing an incoming [TcpStream] yielded by [Server::listen]
    ///
    /// # Arguments
    ///
    /// * `stream` -> A buffered [TcpStream] containing the [Request]
    ///
    /// # Returns
    ///
    /// * `io::Result<Request>` -> A Result containing the parsed [Request] or an [std::io] error
    ///
    pub fn new(mut stream: BufReader<TcpStream>) -> io::Result<Request> {
        // Parse the request line (e.g., "GET /path HTTP/1.1")
        let request_line = Request::read_line(&mut stream)?;

        let mut parts = request_line.split_ascii_whitespace();

        // Parse HTTP method
        let method = match parts.next().unwrap_or("") {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Invalid HTTP method",
                ))
            }
        };

        // Parse route and query parameters
        let full_route = parts.next().unwrap_or("").to_string();
        let (route, query_params) = parse_url(&full_route);

        let mut headers = Headers::new();

        loop {
            let line = Request::read_line(&mut stream)?;

            if line.is_empty() {
                break;
            }

            if let Some((name, value)) = line.split_once(": ") {
                headers.insert(name.to_string(), value.to_string());
            }
        }

        // Extract `Content-Length` from [Request] body if present
        let content_length = headers
            .get("Content-Length")
            .and_then(|len| len.parse::<usize>().ok())
            .unwrap_or(0);

        let mut body = vec![0; content_length];

        if content_length > 0 {
            stream.read_exact(&mut body)?;
        }

        Ok(Request {
            method,
            route,
            headers,
            query_params,
            body,
        })
    }

    ///
    /// Reads a single line from the [TcpStream].
    ///
    fn read_line(stream: &mut BufReader<TcpStream>) -> io::Result<String> {
        let mut line = String::new();
        stream.read_line(&mut line)?;

        Ok(line.trim().to_string())
    }

    ///
    /// Attempts to parse the [Request] body as `JSON` into the specified type `T`.
    ///
    /// # Parameters
    ///
    /// * `T` -> The type to deserialize the `JSON` into. **Must implement Deserialize.**
    ///
    /// # Returns
    ///
    /// * `Option<T>` -> The parsed `JSON` data or None if parsing fails
    ///
    pub fn get_json<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        serde_json::from_slice(&self.body).ok()
    }
}

impl Response {
    ///
    /// Creates a new [Response] with the specified status code.
    ///
    /// # Arguments
    ///
    /// * `status` -> HTTP status code (e.g., 200, 404)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// let response = Response::new(200);
    /// ```
    ///
    pub fn new(status: u16) -> Response {
        let mut headers = Headers::new();

        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Response {
            status,
            headers,
            body: String::new(),
        }
    }

    ///
    /// Sets the [Response] body as `JSON` and returns the modified response.
    ///
    /// # Arguments
    ///
    /// * `data` -> Data to be serialized to `JSON`. **Must implement Serialize.**
    ///
    /// # Returns
    ///
    /// Modified [Response] with `JSON` body and updated `Content-Length` header
    ///
    pub fn json<T: Serialize>(mut self, data: &T) -> Response {
        self.body = serde_json::to_string(data).unwrap_or_default();

        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());

        self
    }

    ///
    /// Sends the [Response] over the [TcpStream].
    ///
    /// # Arguments
    ///
    /// * `stream` -> The [TcpStream] to write the response to
    ///
    /// # Returns
    ///
    /// * `io::Result<()>` -> Ok if the response was sent successfully or an [std::io] error
    ///
    pub fn send(self, stream: &mut TcpStream) -> io::Result<()> {
        let status_text = match self.status {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n\r\n{}",
            self.status,
            status_text,
            self.headers
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\r\n"),
            self.body,
        );

        stream.write_all(response.as_bytes())
    }
}

///
/// Parses a URL string into a route and [QueryParams].
///
/// # Arguments
///
/// * `raw_route` -> The raw URL string to parse
///
/// # Returns
///
/// `(String, QueryParams)` -> Tuple containing the route string and a
/// HashMap of [QueryParams]
///
fn parse_url(raw_route: &str) -> (String, QueryParams) {
    if let Some((path, query)) = raw_route.split_once('?') {
        let query_params = query
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');

                Some((
                    parts.next()?.to_string(),
                    parts.next().unwrap_or("").to_string(),
                ))
            })
            .collect();

        (path.to_string(), query_params)
    } else {
        (raw_route.to_string(), HashMap::new())
    }
}
