use std::{
    io::{self, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use liten_http::Request;

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:3000").expect("Could not start server on port 3000");

    loop {
        let (connection, _) = listener.accept().unwrap();

        if let Err(e) = handle_connection(connection) {
            eprintln!("failed to handle connecntion: {e}")
        }
    }
}

fn handle_connection(mut connection: TcpStream) -> io::Result<()> {
    let mut read = 0;
    let mut request = [0u8; 1024];

    loop {
        let num_bytes = connection.read(&mut request[read..])?;

        if num_bytes == 0 {
            eprintln!("client disconnected...");
            return Ok(());
        }

        read += num_bytes;

        if request.get(read - 4..read) == Some(b"\r\n\r\n") {
            break;
        }
    }

    let request_str = String::from_utf8_lossy(&request[..read]);
    let request = Request::from_string(&request_str)?;
    println!("{request}");

    let response = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Content-Length: 12\n",
        "Connection: close\r\n\r\n",
        "Hello world!"
    );

    let mut written = 0;

    loop {
        let num_bytes = connection.write(response[written..].as_bytes())?;

        if num_bytes == 0 {
            eprintln!("client disconnected...");
            return Ok(());
        }

        written += num_bytes;

        if written == response.len() {
            break;
        }
    }

    connection.flush()?;

    Ok(())
}
