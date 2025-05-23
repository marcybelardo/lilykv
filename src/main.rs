use std::io::{BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};
mod parser;
// use parser::Resp;
// use parser::Parser;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_conn(stream);
    }
}

fn handle_conn(stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);
    let mut buf = String::new();
    let bytes_read = buf_reader.read_to_string(&mut buf).unwrap();
    assert_eq!(bytes_read, buf.len());

    println!("{buf}");
}
