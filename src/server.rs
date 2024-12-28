use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    collections::HashMap,
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST,
}

pub type Headers = HashMap<String, String>;
pub type QueryParams = HashMap<String, String>;

#[derive(Debug)]
pub struct Request {
    pub route: String,
    pub method: HttpMethod,
    pub headers: Headers,
    pub query_params: QueryParams,
    pub body: Vec<u8>,
}

#[derive(Serialize)]
pub struct Response {
    status: u16,
    headers: Headers,
    body: String,
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Server> {
        let listener = TcpListener::bind(addr)?;

        Ok(Server { listener })
    }

    pub fn listen(&self) -> impl Iterator<Item = io::Result<TcpStream>> + '_ {
        self.listener.incoming()
    }
}

impl Request {
    pub fn new(mut stream: BufReader<TcpStream>) -> io::Result<Request> {
        let request_line = Request::read_line(&mut stream)?;
        let mut parts = request_line.split_ascii_whitespace();

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

    fn read_line(stream: &mut BufReader<TcpStream>) -> io::Result<String> {
        let mut line = String::new();
        stream.read_line(&mut line)?;

        Ok(line.trim().to_string())
    }

    pub fn get_json<T: for<'a> Deserialize<'a>>(&self) -> Option<T> {
        serde_json::from_slice(&self.body).ok()
    }
}

impl Response {
    pub fn new(status: u16) -> Response {
        let mut headers = Headers::new();
        
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Response {
            status,
            headers,
            body: String::new(),
        }
    }

    pub fn json<T: Serialize>(mut self, data: &T) -> Response {
        self.body = serde_json::to_string(data).unwrap_or_default();
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

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
