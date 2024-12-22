use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

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
