use std::{
    io,
    net::{TcpListener, TcpStream},
};

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
