use std::{
    io::{BufReader, Read},
    net::TcpListener,
};

use liten_http::Request;

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:3000").expect("Could not start server on port 3000");

    for stream in listener.incoming() {
        let mut buf_read = BufReader::new(stream.unwrap());
        let mut request_string = String::new();
        let _ = buf_read.read_to_string(&mut request_string);

        let request = Request::from_string(&request_string);

        let result = request.unwrap();

        println!("{:#?}", result.method);
        println!("{:#?}", result.header);
    }
}
