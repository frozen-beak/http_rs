use std::{
    io::{self, prelude::*, BufReader},
    net::{Incoming, TcpListener, TcpStream},
};

const K_MAX_SIZE: usize = 4096;

///
/// End user
///
// struct Client;

///
/// Response to be sent
///
// struct Response;

///
/// Host of the app
///
pub struct Server {
    connection: TcpListener,
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub type Headers = std::collections::HashMap<String, String>;

///
/// Data received from the client
///
#[derive(Debug)]
pub struct Request {
    ///
    /// Represents matching routes to things that our server might
    /// know about
    ///
    pub route: String,

    ///
    /// Type of the request sent by the [Client]
    ///
    pub method: HttpMethod,

    ///
    /// KV Pairs of metadata attached to the request
    ///
    pub headers: Headers,

    ///
    /// Data attached w/ request like [HttpMethod::POST], etc.
    ///
    pub body: Vec<u8>,
}

impl Server {
    pub fn new(addr: &str) -> Server {
        let listener = TcpListener::bind(addr).expect("Unable to bind address to listener");

        Self {
            connection: listener,
        }
    }

    pub fn listen(&self) -> Incoming<'_> {
        self.connection.incoming()
    }
}

impl Request {
    pub fn new(mut stream: BufReader<TcpStream>) -> io::Result<Request> {
        // GET /index.html HTTP/1.1
        //  ^  ^                ^ version
        //  |  \ resource
        //  |
        //  \ method
        let http_metadata = Request::read_header_line(&mut stream)?;

        eprintln!("{http_metadata}");

        let mut parts = http_metadata.split_ascii_whitespace();

        let method = match parts.next().unwrap() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "DELETE" => HttpMethod::DELETE,
            "PUT" => HttpMethod::PUT,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unsupported HTTP method",
                ))
            }
        };

        let route = parts.next().unwrap().to_string();

        // version
        let _ = parts.next();

        let mut headers = Headers::new();

        loop {
            let line = Request::read_header_line(&mut stream)?;

            if line.is_empty() {
                break;
            }

            let mut parts = line.split(": ");

            // Content-Type
            let name = parts.next().unwrap().to_string();

            // text/html
            let value = parts.next().unwrap().to_string();

            headers.insert(name, value);
        }

        let mut body = Vec::with_capacity(K_MAX_SIZE);
        let _ = stream.read(&mut body)?;

        Ok(Request {
            route,
            method,
            headers,
            body,
        })
    }

    fn read_header_line(stream: &mut BufReader<TcpStream>) -> io::Result<String> {
        let mut buf: Vec<u8> = Vec::with_capacity(K_MAX_SIZE);

        while let Some(Ok(byte)) = stream.bytes().next() {
            if byte == b'\n' {
                if buf.ends_with(b"\r") {
                    buf.pop();
                }

                let header_line = String::from_utf8(buf).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Not an http header")
                })?;

                return Ok(header_line);
            }

            buf.push(byte);
        }

        Err(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "Client aborted early",
        ))
    }
}

pub struct HTTP;

impl HTTP {
    pub fn new() {
        let listener = TcpListener::bind("127.0.0.1:6969").expect("Unable to bind listener");

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            HTTP::handle_connection(stream);
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);

        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|res| res.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("Request: {http_request:#?}");

        let res = "HTTP/1.1 200 OK\r\n\r\n";

        stream.write_all(res.as_bytes()).unwrap();
    }
}
