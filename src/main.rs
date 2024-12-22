use std::io::BufReader;

use http_rs::http::{Request, Server};

fn main() {
    let server = Server::new("127.0.0.1:6969");

    for stream in server.listen() {
        let stream = stream.unwrap();
        let buf = BufReader::new(stream);

        if let Ok(req) = Request::new(buf) {
            println!("{req:?}");
        }
    }
}
